use rusqlite::{params, Connection, OptionalExtension, Transaction};
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
        tx.execute(
            "DELETE FROM beneficiario WHERE socio_id = ?1",
            params![socio.id],
        )?;
        tx.execute(
            "DELETE FROM protegido WHERE socio_id = ?1",
            params![socio.id],
        )?;

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
        let mut stmt = conn
            .prepare("SELECT nombre, cedula, parentesco FROM beneficiario WHERE socio_id = ?1")?;
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
            saldo_incobrable, numero
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
    numero: Option<i64>,
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
            numero: row.get(13)?,
        })
    }

    fn en_socio(self, beneficiario: Option<Beneficiario>, protegidos: Vec<Protegido>) -> Socio {
        Socio {
            id: self.id,
            numero: self.numero.unwrap_or(0),
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
              correo, estatus, fecha_ingreso, fecha_retiro, saldo_incobrable, numero)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
                socio.numero,
            ],
        )?;
        Self::reemplazar_allegados(&tx, socio)?;
        tx.commit()?;
        Ok(())
    }

    fn siguiente_numero(&self, banco_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let siguiente: i64 = conn.query_row(
            "SELECT COALESCE(MAX(numero), 0) + 1 FROM socio",
            [],
            |r| r.get(0),
        )?;
        Ok(siguiente)
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

        // Roster: NO se hidratan beneficiario ni protegidos (ver contrato del puerto).
        // El detalle completo se obtiene con `buscar_por_id`. Así se evitan las N+1
        // consultas por socio que, además, todos los consumidores del listado descartan.
        Ok(filas
            .into_iter()
            .map(|fila| fila.en_socio(None, Vec::new()))
            .collect())
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

    fn marcar_retiro(
        &self,
        banco_id: &str,
        socio_id: &str,
        estatus: EstatusSocio,
        fecha_retiro: &str,
        saldo_incobrable: f64,
    ) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let filas = conn.execute(
            "UPDATE socio SET estatus = ?2, fecha_retiro = ?3, saldo_incobrable = ?4 WHERE id = ?1",
            params![socio_id, estatus.as_str(), fecha_retiro, saldo_incobrable],
        )?;
        // Un id inexistente no debe pasar como retiro exitoso (consistente con `actualizar`).
        if filas == 0 {
            return Err(AppError::SocioNoEncontrado);
        }
        Ok(())
    }

    fn listar_retirados_con_deuda(&self, banco_id: &str) -> Result<Vec<Socio>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let sql = format!("{SELECT_SOCIO} WHERE estatus = ?1 ORDER BY apellidos, nombres");
        let mut stmt = conn.prepare(&sql)?;
        let filas = stmt
            .query_map(
                params![EstatusSocio::RetiradoConDeuda.as_str()],
                FilaSocio::desde_row,
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        // Roster (sin allegados), igual que `listar`: su único consumidor —la bandeja de
        // deuda incobrable— sólo necesita nombre, cédula y saldo.
        Ok(filas
            .into_iter()
            .map(|fila| fila.en_socio(None, Vec::new()))
            .collect())
    }

    fn abonar_deuda_incobrable(
        &self,
        banco_id: &str,
        socio_id: &str,
        monto: f64,
    ) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let saldo_actual: f64 = conn
            .query_row(
                "SELECT COALESCE(saldo_incobrable, 0) FROM socio WHERE id = ?1",
                params![socio_id],
                |r| r.get(0),
            )
            .optional()?
            .ok_or(AppError::SocioNoEncontrado)?;
        // El dinero del Bankomunal va en pesos enteros (COP): el saldo incobrable también,
        // consistente con el cuadre y con [[dinero-pesos-enteros]]. Nada de centavos.
        let nuevo_saldo = (saldo_actual - monto).max(0.0).round();
        let nuevo_estatus = if nuevo_saldo <= 0.0 {
            EstatusSocio::RetiradoDeudaSaldada
        } else {
            EstatusSocio::RetiradoConDeuda
        };
        conn.execute(
            "UPDATE socio SET saldo_incobrable = ?2, estatus = ?3 WHERE id = ?1",
            params![socio_id, nuevo_saldo, nuevo_estatus.as_str()],
        )?;
        Ok(nuevo_saldo)
    }
}
