use rusqlite::{params, Connection, Transaction};
use uuid::Uuid;

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{Beneficiario, EstatusSocio, Protegido, Socio, SocioPort};

/// Adaptador SQLite del módulo de Socios, operando sobre el `.db` propio de cada
/// Bankomunal (aislamiento por archivo, RF-08).
///
/// Es la única pieza del módulo que conoce SQL: el Dominio y la Aplicación sólo
/// hablan con el puerto `SocioPort`.
pub struct SqliteSocios {
    db: DbManager,
}

impl SqliteSocios {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }

    /// Escribe el beneficiario y los protegidos del socio dentro de la transacción en
    /// curso, borrando primero los que hubiera. Reemplazar en bloque es más simple y
    /// seguro que reconciliar altas y bajas fila por fila (KISS), y son a lo sumo
    /// 3 filas por socio.
    fn reemplazar_allegados(tx: &Transaction, socio: &Socio) -> Result<(), AppError> {
        tx.execute("DELETE FROM beneficiario WHERE socio_id = ?1", params![socio.id])?;
        tx.execute("DELETE FROM protegido WHERE socio_id = ?1", params![socio.id])?;

        if let Some(b) = &socio.beneficiario {
            tx.execute(
                "INSERT INTO beneficiario (id, socio_id, nombre, cedula, parentesco)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    Uuid::new_v4().to_string(),
                    socio.id,
                    b.nombre,
                    b.cedula,
                    b.parentesco
                ],
            )?;
        }

        for p in &socio.protegidos {
            tx.execute(
                "INSERT INTO protegido (id, socio_id, nombre, cedula, parentesco, telefono)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    Uuid::new_v4().to_string(),
                    socio.id,
                    p.nombre,
                    p.cedula,
                    p.parentesco,
                    p.telefono
                ],
            )?;
        }
        Ok(())
    }

    /// Reconstruye el socio completo a partir de su fila y las de sus allegados.
    fn hidratar(conn: &Connection, fila: FilaSocio) -> Result<Socio, AppError> {
        let mut stmt = conn.prepare(
            "SELECT nombre, cedula, parentesco FROM beneficiario WHERE socio_id = ?1",
        )?;
        let mut rows = stmt.query(params![fila.id])?;
        let beneficiario = match rows.next()? {
            Some(row) => Some(Beneficiario {
                nombre: row.get(0)?,
                cedula: row.get(1)?,
                parentesco: row.get(2)?,
            }),
            None => None,
        };

        let mut stmt = conn.prepare(
            "SELECT nombre, cedula, parentesco, telefono FROM protegido
             WHERE socio_id = ?1 ORDER BY rowid",
        )?;
        let protegidos = stmt
            .query_map(params![fila.id], |row| {
                Ok(Protegido {
                    nombre: row.get(0)?,
                    cedula: row.get(1)?,
                    parentesco: row.get(2)?,
                    telefono: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(fila.en_socio(beneficiario, protegidos))
    }

    fn consultar_uno(
        &self,
        banco_id: &str,
        clausula: &str,
        valor: &str,
    ) -> Result<Option<Socio>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let sql = format!("{SELECT_SOCIO} WHERE {clausula} = ?1");
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params![valor])?;
        match rows.next()? {
            Some(row) => {
                let fila = FilaSocio::desde_row(row)?;
                drop(rows);
                drop(stmt);
                Ok(Some(Self::hidratar(&conn, fila)?))
            }
            None => Ok(None),
        }
    }
}

const SELECT_SOCIO: &str = "SELECT id, cedula, nombres, apellidos, profesion, direccion,
            telefono, celular, correo, estatus, fecha_ingreso, fecha_retiro,
            saldo_incobrable
     FROM socio";

/// Proyección plana de la tabla `socio`, sin sus allegados. Existe para poder soltar el
/// `Statement` antes de lanzar las consultas de beneficiario y protegidos sobre la
/// misma conexión.
struct FilaSocio {
    id: String,
    cedula: String,
    nombres: String,
    apellidos: String,
    profesion: String,
    direccion: String,
    telefono: String,
    celular: String,
    correo: String,
    estatus: String,
    fecha_ingreso: String,
    fecha_retiro: Option<String>,
    saldo_incobrable: Option<f64>,
}

impl FilaSocio {
    fn desde_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            cedula: row.get(1)?,
            nombres: row.get(2)?,
            apellidos: row.get(3)?,
            profesion: row.get(4)?,
            direccion: row.get(5)?,
            telefono: row.get(6)?,
            celular: row.get(7)?,
            correo: row.get(8)?,
            estatus: row.get(9)?,
            fecha_ingreso: row.get(10)?,
            fecha_retiro: row.get(11)?,
            saldo_incobrable: row.get(12)?,
        })
    }

    fn en_socio(self, beneficiario: Option<Beneficiario>, protegidos: Vec<Protegido>) -> Socio {
        Socio {
            id: self.id,
            cedula: self.cedula,
            nombres: self.nombres,
            apellidos: self.apellidos,
            profesion: self.profesion,
            direccion: self.direccion,
            telefono: self.telefono,
            celular: self.celular,
            correo: self.correo,
            estatus: EstatusSocio::desde_str(&self.estatus),
            fecha_ingreso: self.fecha_ingreso,
            fecha_retiro: self.fecha_retiro,
            saldo_incobrable: self.saldo_incobrable.unwrap_or(0.0),
            beneficiario,
            protegidos,
        }
    }
}

impl SocioPort for SqliteSocios {
    fn crear(&self, banco_id: &str, socio: &Socio) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        // El socio y sus allegados entran juntos o no entra ninguno: un socio a medio
        // guardar (sin sus protegidos) sería un registro silenciosamente incompleto.
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO socio
             (id, cedula, nombres, apellidos, profesion, direccion, telefono, celular,
              correo, estatus, fecha_ingreso, fecha_retiro, saldo_incobrable)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                socio.id,
                socio.cedula,
                socio.nombres,
                socio.apellidos,
                socio.profesion,
                socio.direccion,
                socio.telefono,
                socio.celular,
                socio.correo,
                socio.estatus.as_str(),
                socio.fecha_ingreso,
                socio.fecha_retiro,
                socio.saldo_incobrable,
            ],
        )?;
        Self::reemplazar_allegados(&tx, socio)?;
        tx.commit()?;
        Ok(())
    }

    fn actualizar(&self, banco_id: &str, socio: &Socio) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;
        let filas = tx.execute(
            "UPDATE socio SET
                cedula = ?2, nombres = ?3, apellidos = ?4, profesion = ?5,
                direccion = ?6, telefono = ?7, celular = ?8, correo = ?9,
                estatus = ?10, fecha_ingreso = ?11, fecha_retiro = ?12,
                saldo_incobrable = ?13
             WHERE id = ?1",
            params![
                socio.id,
                socio.cedula,
                socio.nombres,
                socio.apellidos,
                socio.profesion,
                socio.direccion,
                socio.telefono,
                socio.celular,
                socio.correo,
                socio.estatus.as_str(),
                socio.fecha_ingreso,
                socio.fecha_retiro,
                socio.saldo_incobrable,
            ],
        )?;
        if filas == 0 {
            return Err(AppError::SocioNoEncontrado);
        }
        Self::reemplazar_allegados(&tx, socio)?;
        tx.commit()?;
        Ok(())
    }

    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Socio>, AppError> {
        self.consultar_uno(banco_id, "id", id)
    }

    fn buscar_por_cedula(&self, banco_id: &str, cedula: &str) -> Result<Option<Socio>, AppError> {
        self.consultar_uno(banco_id, "cedula", cedula)
    }

    fn listar(&self, banco_id: &str) -> Result<Vec<Socio>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let sql = format!("{SELECT_SOCIO} ORDER BY apellidos, nombres");
        let mut stmt = conn.prepare(&sql)?;
        let filas = stmt
            .query_map([], FilaSocio::desde_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        drop(stmt);

        let mut socios = Vec::with_capacity(filas.len());
        for fila in filas {
            socios.push(Self::hidratar(&conn, fila)?);
        }
        Ok(socios)
    }

    fn contar_activos(&self, banco_id: &str) -> Result<usize, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM socio WHERE estatus = ?1",
            params![EstatusSocio::Activo.as_str()],
            |r| r.get(0),
        )?;
        Ok(total as usize)
    }
}
