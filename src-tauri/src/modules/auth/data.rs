use rusqlite::{params, ErrorCode};

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{
    AppSettingsPort, BancoCatalogoPort, Bankomunal, NuevoBankomunal, PasswordHasherPort,
};

/// Traduce una violación del índice UNIQUE del nombre a un error de dominio claro
/// (RF-04). Cualquier otro fallo de SQLite se propaga tal cual.
fn mapear_nombre_duplicado(e: rusqlite::Error) -> AppError {
    if let rusqlite::Error::SqliteFailure(err, _) = &e {
        if err.code == ErrorCode::ConstraintViolation {
            return AppError::NombreDuplicado;
        }
    }
    AppError::from(e)
}

/// Adaptador SQLite para las claves de configuración a nivel aplicación (app.db).
pub struct SqliteAppSettings {
    db: DbManager,
}

impl SqliteAppSettings {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl AppSettingsPort for SqliteAppSettings {
    fn obtener(&self, clave: &str) -> Result<Option<String>, AppError> {
        let conn = self.db.open_app_db()?;
        let mut stmt = conn.prepare("SELECT valor FROM app_setting WHERE clave = ?1")?;
        let mut rows = stmt.query(params![clave])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    fn guardar(&self, clave: &str, valor: &str) -> Result<(), AppError> {
        let conn = self.db.open_app_db()?;
        conn.execute(
            "INSERT INTO app_setting (clave, valor) VALUES (?1, ?2)
             ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![clave, valor],
        )?;
        Ok(())
    }
}

/// Adaptador de hash con bcrypt (mínimo 10 rondas).
pub struct BcryptHasher;

impl PasswordHasherPort for BcryptHasher {
    fn hash(&self, texto: &str) -> Result<String, AppError> {
        Ok(bcrypt::hash(texto, bcrypt::DEFAULT_COST)?)
    }

    fn verify(&self, texto: &str, hash: &str) -> Result<bool, AppError> {
        Ok(bcrypt::verify(texto, hash)?)
    }
}

/// Adaptador SQLite del catálogo de Bankomunales: crea el `.db` del Banco y registra
/// la fila en el catálogo (RF-03, RF-04, RF-08).
pub struct SqliteBancoCatalogo {
    db: DbManager,
}

impl SqliteBancoCatalogo {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl BancoCatalogoPort for SqliteBancoCatalogo {
    fn preparar_almacen(&self, id: &str) -> Result<(), AppError> {
        self.db.open_banco_db(id).map(|_| ())
    }

    fn crear(&self, id: &str, fecha_creacion: &str, nuevo: &NuevoBankomunal) -> Result<Bankomunal, AppError> {
        let db_path = self
            .db
            .banco_db_path(id)
            .to_string_lossy()
            .into_owned();

        let banco = Bankomunal {
            id: id.to_string(),
            nombre: nuevo.nombre.clone(),
            ubicacion: nuevo.ubicacion.clone(),
            moneda: nuevo.moneda.clone(),
            fecha_creacion: fecha_creacion.to_string(),
            db_path,
        };

        // Se registra primero la fila del catálogo: si el nombre choca con el índice UNIQUE
        // (RF-04, condición de carrera incluida) el fallo ocurre ANTES de tocar el disco,
        // por lo que no quedan archivos .db huérfanos.
        let conn = self.db.open_app_db()?;
        conn.execute(
            "INSERT INTO catalogo_bankomunal
             (id, nombre, ubicacion, moneda, fecha_creacion, db_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                banco.id,
                banco.nombre,
                banco.ubicacion,
                banco.moneda,
                banco.fecha_creacion,
                banco.db_path
            ],
        )
        .map_err(mapear_nombre_duplicado)?;

        // Sólo tras registrar el catálogo se crea el .db del Banco con su esquema (RF-08).
        // Si esto fallara, la fila queda sin archivo y `open_banco_db` lo recrea de forma
        // idempotente la próxima vez que se seleccione el Banco (auto-reparable).
        self.db.open_banco_db(id)?;
        Ok(banco)
    }

    fn listar(&self) -> Result<Vec<Bankomunal>, AppError> {
        let conn = self.db.open_app_db()?;
        let mut stmt = conn.prepare(
            "SELECT id, nombre, ubicacion, moneda, fecha_creacion, db_path
             FROM catalogo_bankomunal ORDER BY nombre",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Bankomunal {
                id: row.get(0)?,
                nombre: row.get(1)?,
                ubicacion: row.get(2)?,
                moneda: row.get(3)?,
                fecha_creacion: row.get(4)?,
                db_path: row.get(5)?,
            })
        })?;
        let mut bancos = Vec::new();
        for b in rows {
            bancos.push(b?);
        }
        Ok(bancos)
    }



    fn buscar_por_nombre(&self, nombre: &str) -> Result<Option<Bankomunal>, AppError> {
        let conn = self.db.open_app_db()?;
        // COLLATE NOCASE: la unicidad del nombre no distingue mayúsculas (RF-04).
        let mut stmt = conn.prepare(
            "SELECT id, nombre, ubicacion, moneda, fecha_creacion, db_path
             FROM catalogo_bankomunal WHERE nombre = ?1 COLLATE NOCASE",
        )?;
        let mut rows = stmt.query(params![nombre])?;
        match rows.next()? {
            Some(row) => Ok(Some(Bankomunal {
                id: row.get(0)?,
                nombre: row.get(1)?,
                ubicacion: row.get(2)?,
                moneda: row.get(3)?,
                fecha_creacion: row.get(4)?,
                db_path: row.get(5)?,
            })),
            None => Ok(None),
        }
    }

    fn buscar_por_id(&self, id: &str) -> Result<Option<Bankomunal>, AppError> {
        let conn = self.db.open_app_db()?;
        let mut stmt = conn.prepare(
            "SELECT id, nombre, ubicacion, moneda, fecha_creacion, db_path
             FROM catalogo_bankomunal WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Bankomunal {
                id: row.get(0)?,
                nombre: row.get(1)?,
                ubicacion: row.get(2)?,
                moneda: row.get(3)?,
                fecha_creacion: row.get(4)?,
                db_path: row.get(5)?,
            })),
            None => Ok(None),
        }
    }
}