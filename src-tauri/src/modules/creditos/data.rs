use std::sync::Arc;

use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::caja::application::CajaService;
use crate::modules::caja::domain::CodigoOperacion;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;
use crate::modules::creditos::domain::{
    AccionesParaCreditoPort, Credito, CreditoPort, CuotaPlaneada, DestinoCredito, EstadoCredito,
    EstadoSolicitud, GarantiaCredito, GarantiaSolicitud, LibroContablePort, ParametrosCredito,
    ParametrosCreditoPort, RolGarantia, SociosParaCreditoPort, SolicitudCredito, SolicitudPort,
};

/// Adaptador SQLite de las solicitudes de crédito (RF-43 a RF-52).
pub struct SqliteSolicitudes {
    db: DbManager,
}

impl SqliteSolicitudes {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

fn fila_a_solicitud(row: &rusqlite::Row) -> rusqlite::Result<SolicitudCredito> {
    Ok(SolicitudCredito {
        id: row.get(0)?,
        socio_id: row.get(1)?,
        fecha_solicitud: row.get(2)?,
        monto_solicitado: row.get(3)?,
        plazo_cuotas: row.get(4)?,
        destino: DestinoCredito::desde_str(&row.get::<_, String>(5)?).unwrap_or(DestinoCredito::Otros),
        total_ingresos: row.get(6)?,
        total_egresos: row.get(7)?,
        capacidad_pago: row.get(8)?,
        estado: EstadoSolicitud::desde_str(&row.get::<_, String>(9)?),
        monto_aprobado: row.get(10)?,
        observacion: row.get(11)?,
        fecha_decision: row.get(12)?,
        decidida_por: row.get(13)?,
        garantias: Vec::new(),
    })
}

const SELECT_SOLICITUD: &str =
    "SELECT id, socio_id, fecha_solicitud, monto_solicitado, plazo_cuotas, destino,
            total_ingresos, total_egresos, capacidad_pago, estado,
            monto_aprobado, observacion, fecha_decision, decidida_por
     FROM solicitud_credito";

fn garantias_de_solicitud(
    conn: &rusqlite::Connection,
    solicitud_id: &str,
) -> Result<Vec<GarantiaSolicitud>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, solicitud_id, socio_id, rol, acciones_comprometidas
         FROM garantia_solicitud WHERE solicitud_id = ?1",
    )?;
    let rows = stmt.query_map(params![solicitud_id], |row| {
        Ok(GarantiaSolicitud {
            id: row.get(0)?,
            solicitud_id: row.get(1)?,
            socio_id: row.get(2)?,
            rol: RolGarantia::desde_str(&row.get::<_, String>(3)?),
            acciones_comprometidas: row.get(4)?,
        })
    })?;
    let mut v = Vec::new();
    for g in rows {
        v.push(g?);
    }
    Ok(v)
}

impl SolicitudPort for SqliteSolicitudes {
    fn crear(
        &self,
        banco_id: &str,
        solicitud: &SolicitudCredito,
        garantias: &[GarantiaSolicitud],
    ) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO solicitud_credito
             (id, socio_id, fecha_solicitud, monto_solicitado, plazo_cuotas, destino,
              total_ingresos, total_egresos, capacidad_pago, estado,
              monto_aprobado, observacion, fecha_decision, decidida_por)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                solicitud.id,
                solicitud.socio_id,
                solicitud.fecha_solicitud,
                solicitud.monto_solicitado,
                solicitud.plazo_cuotas,
                solicitud.destino.as_str(),
                solicitud.total_ingresos,
                solicitud.total_egresos,
                solicitud.capacidad_pago,
                solicitud.estado.as_str(),
                solicitud.monto_aprobado,
                solicitud.observacion,
                solicitud.fecha_decision,
                solicitud.decidida_por,
            ],
        )?;
        insertar_garantias_solicitud(&tx, garantias)?;
        tx.commit()?;
        Ok(())
    }

    fn actualizar(
        &self,
        banco_id: &str,
        solicitud: &SolicitudCredito,
        garantias: &[GarantiaSolicitud],
    ) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE solicitud_credito SET
                socio_id = ?2, fecha_solicitud = ?3, monto_solicitado = ?4,
                plazo_cuotas = ?5, destino = ?6, total_ingresos = ?7, total_egresos = ?8,
                capacidad_pago = ?9, estado = ?10, monto_aprobado = ?11,
                observacion = ?12, fecha_decision = ?13, decidida_por = ?14
             WHERE id = ?1",
            params![
                solicitud.id,
                solicitud.socio_id,
                solicitud.fecha_solicitud,
                solicitud.monto_solicitado,
                solicitud.plazo_cuotas,
                solicitud.destino.as_str(),
                solicitud.total_ingresos,
                solicitud.total_egresos,
                solicitud.capacidad_pago,
                solicitud.estado.as_str(),
                solicitud.monto_aprobado,
                solicitud.observacion,
                solicitud.fecha_decision,
                solicitud.decidida_por,
            ],
        )?;
        tx.execute(
            "DELETE FROM garantia_solicitud WHERE solicitud_id = ?1",
            params![solicitud.id],
        )?;
        insertar_garantias_solicitud(&tx, garantias)?;
        tx.commit()?;
        Ok(())
    }

    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<SolicitudCredito>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_SOLICITUD} WHERE id = ?1"))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => {
                let mut s = fila_a_solicitud(row)?;
                s.garantias = garantias_de_solicitud(&conn, id)?;
                Ok(Some(s))
            }
            None => Ok(None),
        }
    }

    fn listar_por_estado(
        &self,
        banco_id: &str,
        estado: Option<EstadoSolicitud>,
    ) -> Result<Vec<SolicitudCredito>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = match estado {
            Some(_e) => conn.prepare(&format!("{SELECT_SOLICITUD} WHERE estado = ?1 ORDER BY fecha_solicitud DESC"))?,
            None => conn.prepare(&format!("{SELECT_SOLICITUD} ORDER BY fecha_solicitud DESC"))?,
        };
        let rows = match estado {
            Some(e) => stmt.query_map(params![e.as_str()], fila_a_solicitud)?,
            None => stmt.query_map([], fila_a_solicitud)?,
        };
        let mut v = Vec::new();
        for r in rows {
            let mut s = r?;
            s.garantias = garantias_de_solicitud(&conn, &s.id)?;
            v.push(s);
        }
        Ok(v)
    }
}

fn insertar_garantias_solicitud(
    tx: &rusqlite::Transaction,
    garantias: &[GarantiaSolicitud],
) -> Result<(), AppError> {
    for g in garantias {
        tx.execute(
            "INSERT INTO garantia_solicitud
             (id, solicitud_id, socio_id, rol, acciones_comprometidas)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![g.id, g.solicitud_id, g.socio_id, g.rol.as_str(), g.acciones_comprometidas],
        )?;
    }
    Ok(())
}

/// Adaptador SQLite de los créditos desembolsados (RF-53 a RF-62).
pub struct SqliteCreditos {
    db: DbManager,
}

impl SqliteCreditos {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

fn fila_a_credito(row: &rusqlite::Row) -> rusqlite::Result<Credito> {
    Ok(Credito {
        id: row.get(0)?,
        socio_id: row.get(1)?,
        numero: row.get(2)?,
        monto_original: row.get(3)?,
        tasa: row.get(4)?,
        plazo_cuotas: row.get(5)?,
        cuota_actual: row.get(6)?,
        saldo_pendiente: row.get(7)?,
        destino: DestinoCredito::desde_str(&row.get::<_, String>(8)?).unwrap_or(DestinoCredito::Otros),
        estatus: EstadoCredito::desde_str(&row.get::<_, String>(9)?),
        fecha_solicitud: row.get(10)?,
        fecha_desembolso: row.get(11)?,
        frecuencia_pago: row.get(12)?,
        fecha_vencimiento: row.get(13)?,
        solicitud_id: row.get(14)?,
        garantias: Vec::new(),
    })
}

const SELECT_CREDITO: &str =
    "SELECT id, socio_id, numero, monto_original, tasa, plazo_cuotas, cuota_actual,
            saldo_pendiente, destino, estatus, fecha_solicitud, fecha_desembolso,
            frecuencia_pago, fecha_vencimiento, solicitud_id
     FROM credito";

fn garantias_de_credito(
    conn: &rusqlite::Connection,
    credito_id: &str,
) -> Result<Vec<GarantiaCredito>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, credito_id, socio_id, rol, acciones_comprometidas
         FROM garantia_credito WHERE credito_id = ?1",
    )?;
    let rows = stmt.query_map(params![credito_id], |row| {
        Ok(GarantiaCredito {
            id: row.get(0)?,
            credito_id: row.get(1)?,
            socio_id: row.get(2)?,
            rol: RolGarantia::desde_str(&row.get::<_, String>(3)?),
            acciones_comprometidas: row.get(4)?,
        })
    })?;
    let mut v = Vec::new();
    for g in rows {
        v.push(g?);
    }
    Ok(v)
}

impl CreditoPort for SqliteCreditos {
    fn crear(
        &self,
        banco_id: &str,
        credito: &Credito,
        cuotas: &[CuotaPlaneada],
        garantias: &[GarantiaCredito],
    ) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO credito
             (id, socio_id, numero, monto_original, tasa, plazo_cuotas, cuota_actual,
              saldo_pendiente, destino, estatus, fecha_solicitud, fecha_desembolso,
              frecuencia_pago, fecha_vencimiento, solicitud_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                credito.id,
                credito.socio_id,
                credito.numero,
                credito.monto_original,
                credito.tasa,
                credito.plazo_cuotas,
                credito.cuota_actual,
                credito.saldo_pendiente,
                credito.destino.as_str(),
                credito.estatus.as_str(),
                credito.fecha_solicitud,
                credito.fecha_desembolso,
                credito.frecuencia_pago,
                credito.fecha_vencimiento,
                credito.solicitud_id,
            ],
        )?;

        for c in cuotas {
            tx.execute(
                "INSERT INTO cuota
                 (id, credito_id, numero, fecha_vencimiento, capital, interes, valor_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    credito.id,
                    c.numero,
                    c.fecha_vencimiento,
                    c.capital,
                    c.interes,
                    c.valor_total,
                ],
            )?;
        }

        for g in garantias {
            tx.execute(
                "INSERT INTO garantia_credito
                 (id, credito_id, socio_id, rol, acciones_comprometidas)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![g.id, g.credito_id, g.socio_id, g.rol.as_str(), g.acciones_comprometidas],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Credito>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_CREDITO} WHERE id = ?1"))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => {
                let mut c = fila_a_credito(row)?;
                c.garantias = garantias_de_credito(&conn, id)?;
                Ok(Some(c))
            }
            None => Ok(None),
        }
    }

    fn buscar_por_solicitud(
        &self,
        banco_id: &str,
        solicitud_id: &str,
    ) -> Result<Option<Credito>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_CREDITO} WHERE solicitud_id = ?1"))?;
        let mut rows = stmt.query(params![solicitud_id])?;
        match rows.next()? {
            Some(row) => {
                let mut c = fila_a_credito(row)?;
                c.garantias = garantias_de_credito(&conn, &c.id)?;
                Ok(Some(c))
            }
            None => Ok(None),
        }
    }

    fn listar(&self, banco_id: &str) -> Result<Vec<Credito>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_CREDITO} ORDER BY numero"))?;
        let rows = stmt.query_map([], fila_a_credito)?;
        let mut v = Vec::new();
        for r in rows {
            let mut c = r?;
            c.garantias = garantias_de_credito(&conn, &c.id)?;
            v.push(c);
        }
        Ok(v)
    }

    /// RF-53: el número se guarda como texto (p. ej. "001"), en secuencia.
    fn siguiente_numero(&self, banco_id: &str) -> Result<String, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let max: i64 = conn.query_row(
            "SELECT COALESCE(MAX(CAST(numero AS INTEGER)), 0) FROM credito",
            [],
            |r| r.get(0),
        )?;
        Ok(format!("{:03}", max + 1))
    }

    /// RF-58 (RN-05): pares (titular, fiador) de créditos vigentes, para detectar
    /// fiadores cruzados.
    fn pares_titular_fiador(&self, banco_id: &str) -> Result<Vec<(String, String)>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT t.socio_id, f.socio_id
             FROM garantia_credito t
             JOIN credito c ON c.id = t.credito_id
             JOIN garantia_credito f ON f.credito_id = t.credito_id AND f.rol = 'FIADOR'
             WHERE t.rol = 'TITULAR' AND c.estatus = 'VIGENTE'",
        )?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    /// RN-03 (RF-56): cuántos créditos VIGENTES tiene el socio.
    fn contar_vigentes(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM credito WHERE socio_id = ?1 AND estatus = 'VIGENTE'",
            params![socio_id],
            |r| r.get(0),
        )?;
        Ok(n)
    }
}

/// Adaptador de los parámetros del Bankomunal que Créditos necesita (RF-55, RN-03/04).
///
/// El acoplamiento con el módulo de Configuración vive aquí, en la capa de datos; el
/// Dominio de Créditos sólo conoce el puerto `ParametrosCreditoPort`.
pub struct ParametrosCreditoAdapter {
    config: SqliteConfiguracion,
}

impl ParametrosCreditoAdapter {
    pub fn new(db: DbManager) -> Self {
        Self {
            config: SqliteConfiguracion::new(db),
        }
    }
}

impl ParametrosCreditoPort for ParametrosCreditoAdapter {
    fn obtener(&self, banco_id: &str) -> Result<ParametrosCredito, AppError> {
        let cfg = self.config.obtener(banco_id)?;
        Ok(ParametrosCredito {
            monto_maximo_credito: cfg.monto_maximo_credito,
            tasa_interes_ordinario: cfg.tasa_interes_ordinario,
            plazo_maximo_cuotas: cfg.plazo_maximo_cuotas,
            pct_garantia_socio: cfg.pct_garantia_socio,
            pct_garantia_fiador: cfg.pct_garantia_fiador,
            valor_nominal: cfg.valor_nominal,
        })
    }
}

/// Acciones vigentes de un socio, leídas de los lotes no liquidados (RF-49, RN-03/04).
pub struct AccionesParaCreditoAdapter {
    db: DbManager,
}

impl AccionesParaCreditoAdapter {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl AccionesParaCreditoPort for AccionesParaCreditoAdapter {
    fn acciones_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let n: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones
             WHERE socio_id = ?1 AND liquidada = 0",
            params![socio_id],
            |r| r.get(0),
        )?;
        Ok(n)
    }
}

/// Localiza socios por cédula para validar fiadores (RF-49, RN-14).
pub struct SociosParaCreditoAdapter {
    db: DbManager,
}

impl SociosParaCreditoAdapter {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl SociosParaCreditoPort for SociosParaCreditoAdapter {
    fn buscar_por_cedula(&self, banco_id: &str, cedula: &str) -> Result<Option<String>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare("SELECT id FROM socio WHERE cedula = ?1")?;
        let mut rows = stmt.query(params![cedula])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }
}

/// Puente entre Créditos y el Libro de Ingresos y Egresos (RF-62).
///
/// El Libro —su consecutivo, su saldo acumulado y su recálculo— es responsabilidad del
/// módulo de Caja. Créditos no reimplementa nada de eso: le pide a `CajaService` que
/// asiente el desembolso, de modo que la contabilidad siga teniendo un solo dueño.
pub struct LibroViaCaja {
    caja: Arc<CajaService>,
}

impl LibroViaCaja {
    pub fn new(caja: Arc<CajaService>) -> Self {
        Self { caja }
    }
}

impl LibroContablePort for LibroViaCaja {
    fn registrar_desembolso(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError> {
        self.caja
            .registrar_asiento_de_modulo(
                banco_id,
                CodigoOperacion::DesembolsoCredito,
                fecha.to_string(),
                monto,
                descripcion.to_string(),
                Some(socio_id.to_string()),
                Some(credito_id.to_string()),
            )
            .map(|_| ())
    }
}
