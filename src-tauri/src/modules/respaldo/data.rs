use super::domain::{Respaldo, RespaldoPort, VistaRespaldo};
use rusqlite::params;
use crate::core::db::DbManager;
use crate::core::error::AppError;

pub struct SQLiteRespaldoRepository {
    pub db: std::sync::Arc<DbManager>,
}

impl SQLiteRespaldoRepository {
    /// Nombre de OTRO Bankomunal del catálogo que choca con `nombre` (sin distinguir mayúsculas).
    fn buscar_ocupante(&self, nombre: &str, id: &str) -> Result<Option<String>, AppError> {
        let conn = self.db.open_app_db()?;
        let mut stmt = conn.prepare(
            "SELECT nombre FROM catalogo_bankomunal WHERE nombre = ?1 COLLATE NOCASE AND id <> ?2",
        )?;
        let mut filas = stmt.query(params![nombre, id])?;
        Ok(match filas.next()? {
            Some(f) => Some(f.get(0)?),
            None => None,
        })
    }
}

impl RespaldoPort for SQLiteRespaldoRepository {
    fn inspeccionar_respaldo(&self, origen_ruta: &str) -> Result<VistaRespaldo, AppError> {
        let invalido = || {
            AppError::OperacionNoPermitida(
                "El archivo seleccionado no parece ser un respaldo válido de Bankomunales.".into(),
            )
        };
        let conn = rusqlite::Connection::open_with_flags(
            origen_ruta,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|_| invalido())?;
        let (id, nombre, ubicacion, moneda, fecha_creacion): (String, String, String, String, String) = conn
            .query_row(
                "SELECT id, nombre, ubicacion, moneda, fecha_creacion FROM configuracion LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .map_err(|_| invalido())?;
        let ya_existe = self.db.open_app_db()?.query_row(
            "SELECT COUNT(*) FROM catalogo_bankomunal WHERE id = ?1",
            params![id],
            |r| r.get::<_, i64>(0),
        )? > 0;
        Ok(VistaRespaldo { id, nombre, ubicacion, moneda, fecha_creacion, ya_existe })
    }

    fn importar_respaldo(&self, origen_ruta: &str) -> Result<VistaRespaldo, AppError> {
        let vista = self.inspeccionar_respaldo(origen_ruta)?;
        if let Some(ocupante) = self.buscar_ocupante(&vista.nombre, &vista.id)? {
            return Err(AppError::OperacionNoPermitida(format!(
                "Ya existe en este computador otro Bankomunal llamado «{ocupante}». \
                 Cámbiele el nombre antes de importar este respaldo."
            )));
        }
        let destino = self.db.banco_db_path(&vista.id);
        if let Some(carpeta) = destino.parent() {
            std::fs::create_dir_all(carpeta)?;
        }
        if vista.ya_existe {
            // Copia de seguridad de lo que hay hoy, por si el archivo importado no era el esperado.
            if destino.exists() {
                std::fs::copy(&destino, destino.with_extension("db.antes-de-importar"))?;
            }
        } else {
            self.db.open_app_db()?.execute(
                "INSERT INTO catalogo_bankomunal
                 (id, nombre, ubicacion, moneda, fecha_creacion, db_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    vista.id,
                    vista.nombre,
                    vista.ubicacion,
                    vista.moneda,
                    vista.fecha_creacion,
                    destino.to_string_lossy().into_owned()
                ],
            )?;
        }
        std::fs::copy(origen_ruta, &destino).map_err(|e| {
            AppError::OperacionNoPermitida(format!("No se pudo copiar el respaldo: {e}"))
        })?;
        // Aplica las migraciones pendientes si el respaldo venía de una versión anterior.
        self.db.open_banco_db(&vista.id)?;
        Ok(vista)
    }

    fn registrar_respaldo(&self, banco_id: &str, respaldo: Respaldo) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO respaldo (id, fecha, usuario, ruta_archivo, tipo) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![respaldo.id, respaldo.fecha, respaldo.usuario, respaldo.ruta_archivo, respaldo.tipo],
        )?;
        Ok(())
    }

    fn crear_copia_backup(&self, banco_id: &str, destino_ruta: &str) -> Result<(), AppError> {
        // `VACUUM INTO` produce una copia transaccionalmente CONSISTENTE del `.db` (mejor
        // que copiar el archivo a nivel de sistema, que podría capturarlo a medio escribir),
        // y además la deja compactada. Exige que el destino no exista; el diálogo
        // "guardar como" ya implica sobrescribir, así que se borra primero si estaba.
        if std::path::Path::new(destino_ruta).exists() {
            std::fs::remove_file(destino_ruta)?;
        }
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute("VACUUM INTO ?1", rusqlite::params![destino_ruta])
            .map_err(|e| {
                AppError::OperacionNoPermitida(format!(
                    "Error creando el respaldo en {destino_ruta}: {e}"
                ))
            })?;
        Ok(())
    }

    fn validar_y_restaurar_backup(
        &self,
        banco_id: &str,
        origen_ruta: &str,
    ) -> Result<(), AppError> {
        // Verificar que el backup pertenezca al mismo bankomunal
        let backup_conn = rusqlite::Connection::open(origen_ruta).map_err(|e| {
            AppError::OperacionNoPermitida(format!("Error abriendo archivo de respaldo: {}", e))
        })?;

        let mut id_backup = String::new();
        backup_conn
            .query_row("SELECT id FROM configuracion LIMIT 1", [], |row| {
                id_backup = row.get(0)?;
                Ok(())
            })
            .map_err(|_| {
                AppError::OperacionNoPermitida(
                    "El archivo seleccionado no parece ser un respaldo válido de Bankomunales."
                        .to_string(),
                )
            })?;

        if id_backup != banco_id {
            return Err(AppError::OperacionNoPermitida(
                "El archivo de respaldo pertenece a otro Bankomunal.".to_string(),
            ));
        }

        // Drop the backup connection so we don't lock the file
        drop(backup_conn);

        let banco_db_path = self.db.banco_db_path(banco_id);
        std::fs::copy(origen_ruta, &banco_db_path).map_err(|e| {
            AppError::OperacionNoPermitida(format!(
                "Error restaurando el archivo de base de datos de {origen_ruta}: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn listar_respaldos(&self, banco_id: &str) -> Result<Vec<Respaldo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, fecha, usuario, ruta_archivo, tipo FROM respaldo ORDER BY fecha DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Respaldo {
                id: row.get(0)?,
                fecha: row.get(1)?,
                usuario: row.get(2)?,
                ruta_archivo: row.get(3)?,
                tipo: row.get(4)?,
            })
        })?;
        let mut resultado = Vec::new();
        for r in rows {
            resultado.push(r?);
        }
        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn repo() -> (SQLiteRespaldoRepository, DbManager, String, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("bkn_respaldo_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir.clone());
        let banco = Uuid::new_v4().to_string();
        db.open_banco_db(&banco).unwrap();
        let repo = SQLiteRespaldoRepository { db: std::sync::Arc::new(db.clone()) };
        (repo, db, banco, dir)
    }

    /// El respaldo con `VACUUM INTO` produce un archivo válido y consistente que conserva
    /// los datos, y sobrescribe un destino existente.
    #[test]
    fn el_respaldo_es_consistente_y_sobrescribe() {
        let (repo, db, banco, dir) = repo();
        // La restauración valida el banco por su fila de configuración: debe existir.
        db.open_banco_db(&banco).unwrap().execute(
            "INSERT INTO configuracion (id, nombre, fecha_creacion) VALUES (?1, 'Test', '2026-01-01')",
            [&banco],
        ).unwrap();
        // Un dato para verificar que viaja al respaldo.
        db.open_banco_db(&banco).unwrap().execute(
            "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                                telefono, celular, correo, estatus, fecha_ingreso)
             VALUES ('s1','123','Ana','Ríos','','','','','','ACTIVO','2026-01-01')",
            [],
        ).unwrap();

        let destino = dir.join("copia.db");
        let destino_str = destino.to_string_lossy().to_string();

        // Se puede crear aunque el destino ya exista (se sobrescribe).
        std::fs::write(&destino, b"basura previa").unwrap();
        repo.crear_copia_backup(&banco, &destino_str).unwrap();

        // El respaldo es una base SQLite válida con el socio.
        let copia = rusqlite::Connection::open(&destino).unwrap();
        let n: i64 = copia
            .query_row("SELECT COUNT(*) FROM socio WHERE id = 's1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);

        // Restaurar desde ese respaldo valida el banco y repone los datos.
        drop(copia);
        repo.validar_y_restaurar_backup(&banco, &destino_str).unwrap();
        let n: i64 = db
            .open_banco_db(&banco)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM socio WHERE id = 's1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Restaurar un archivo de OTRO Bankomunal se rechaza (RF-113).
    #[test]
    fn restaurar_respaldo_de_otro_banco_se_rechaza() {
        let (repo, db, banco, dir) = repo();
        // Respaldo de un banco distinto: se le siembra su propia configuración.
        let otro = Uuid::new_v4().to_string();
        db.open_banco_db(&otro).unwrap().execute(
            "INSERT INTO configuracion (id, nombre, fecha_creacion) VALUES (?1, 'Otro', '2026-01-01')",
            [&otro],
        ).unwrap();
        let destino = dir.join("otro.db");
        let destino_str = destino.to_string_lossy().to_string();
        repo.crear_copia_backup(&otro, &destino_str).unwrap();

        let err = repo.validar_y_restaurar_backup(&banco, &destino_str).unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Crea un Bankomunal con su configuración en `db` y devuelve el archivo de respaldo.
    fn respaldo_de(db: &DbManager, id: &str, nombre: &str, destino: &std::path::Path) -> String {
        db.open_banco_db(id).unwrap().execute(
            "INSERT INTO configuracion (id, nombre, ubicacion, moneda, fecha_creacion)
             VALUES (?1, ?2, 'Pijao', 'COP', '2026-01-01')",
            rusqlite::params![id, nombre],
        ).unwrap();
        let repo = SQLiteRespaldoRepository { db: std::sync::Arc::new(db.clone()) };
        let ruta = destino.to_string_lossy().to_string();
        repo.crear_copia_backup(id, &ruta).unwrap();
        ruta
    }

    fn otro_equipo() -> (SQLiteRespaldoRepository, DbManager, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("bkn_importar_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir.clone());
        (SQLiteRespaldoRepository { db: std::sync::Arc::new(db.clone()) }, db, dir)
    }

    #[test]
    fn importar_en_un_equipo_nuevo_registra_el_banco_y_deja_sus_datos() {
        let (origen, dir_o) = {
            let (_, db, dir) = otro_equipo();
            let id = Uuid::new_v4().to_string();
            let ruta = respaldo_de(&db, &id, "La Esperanza", &dir.join("a.bankomunal"));
            ((id, ruta), dir)
        };
        let (repo, db, dir) = otro_equipo();

        let vista = repo.inspeccionar_respaldo(&origen.1).unwrap();
        assert_eq!(vista.nombre, "La Esperanza");
        assert!(!vista.ya_existe);

        let importado = repo.importar_respaldo(&origen.1).unwrap();
        assert_eq!(importado.id, origen.0);

        let en_catalogo: i64 = db.open_app_db().unwrap()
            .query_row("SELECT COUNT(*) FROM catalogo_bankomunal WHERE nombre = 'La Esperanza'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(en_catalogo, 1);
        assert!(db.banco_db_path(&origen.0).exists());
        assert!(repo.inspeccionar_respaldo(&origen.1).unwrap().ya_existe);

        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&dir_o).ok();
    }

    #[test]
    fn importar_un_banco_que_ya_existe_lo_reemplaza_y_guarda_copia_de_lo_anterior() {
        let (repo, db, dir) = otro_equipo();
        let id = Uuid::new_v4().to_string();
        let ruta = respaldo_de(&db, &id, "Banco Uno", &dir.join("b.bankomunal"));
        // El banco vive en el catálogo de este equipo, con un socio posterior al respaldo.
        db.open_app_db().unwrap().execute(
            "INSERT INTO catalogo_bankomunal (id, nombre, ubicacion, moneda, fecha_creacion, db_path)
             VALUES (?1, 'Banco Uno', '', 'COP', '2026-01-01', '')", [&id]).unwrap();
        db.open_banco_db(&id).unwrap().execute(
            "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion, telefono,
                                celular, correo, estatus, fecha_ingreso)
             VALUES ('s1','1','A','B','','','','','','ACTIVO','2026-01-01')", []).unwrap();

        assert!(repo.inspeccionar_respaldo(&ruta).unwrap().ya_existe);
        repo.importar_respaldo(&ruta).unwrap();

        let socios: i64 = db.open_banco_db(&id).unwrap()
            .query_row("SELECT COUNT(*) FROM socio", [], |r| r.get(0)).unwrap();
        assert_eq!(socios, 0, "el respaldo no tenía ese socio");
        assert!(db.banco_db_path(&id).with_extension("db.antes-de-importar").exists());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn importar_con_el_nombre_de_otro_banco_se_rechaza() {
        let (_, db_o, dir_o) = otro_equipo();
        let ruta = respaldo_de(&db_o, &Uuid::new_v4().to_string(), "Mismo Nombre", &dir_o.join("c.bankomunal"));
        let (repo, db, dir) = otro_equipo();
        db.open_app_db().unwrap().execute(
            "INSERT INTO catalogo_bankomunal (id, nombre, ubicacion, moneda, fecha_creacion, db_path)
             VALUES ('otro-id', 'mismo nombre', '', 'COP', '2026-01-01', '')", []).unwrap();

        let err = repo.importar_respaldo(&ruta).unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(m) if m.contains("Ya existe")));

        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&dir_o).ok();
    }

    #[test]
    fn un_archivo_que_no_es_respaldo_se_rechaza() {
        let (repo, _db, dir) = otro_equipo();
        let basura = dir.join("x.bankomunal");
        std::fs::write(&basura, b"esto no es sqlite").unwrap();
        assert!(repo.inspeccionar_respaldo(&basura.to_string_lossy()).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
