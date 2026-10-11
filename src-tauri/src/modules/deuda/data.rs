
use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;

use super::domain::{
    CreditoConsultaPort, CreditoDeuda, Cuota, CuotaPort, CuotaReamortizada,
    ParametrosDeudaPort,
};

/// Adaptador de lectura de créditos por número (RF-63).
pub struct SqliteCreditoConsulta {
    db: DbManager,
}

impl SqliteCreditoConsulta {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl CreditoConsultaPort for SqliteCreditoConsulta {
    fn buscar_por_numero(
        &self,
        banco_id: &str,
        numero: &str,
    ) -> Result<Option<CreditoDeuda>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, socio_id, numero, monto_original, tasa, saldo_pendiente, plazo_cuotas, estatus
             FROM credito WHERE numero = ?1",
        )?;
        let mut rows = stmt.query(params![numero])?;
        match rows.next()? {
            Some(row) => Ok(Some(CreditoDeuda {
                id: row.get(0)?,
                socio_id: row.get(1)?,
                numero: row.get(2)?,
                monto_original: row.get(3)?,
                tasa: row.get(4)?,
                saldo_pendiente: row.get(5)?,
                plazo_cuotas: row.get(6)?,
                estatus: row.get(7)?,
            })),
            None => Ok(None),
        }
    }
}

/// Adaptador de las cuotas y la escritura atómica del pago (RF-69).
pub struct SqliteCuotas {
    db: DbManager,
}

impl SqliteCuotas {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

fn fila_a_cuota(row: &rusqlite::Row) -> rusqlite::Result<Cuota> {
    Ok(Cuota {
        id: row.get(0)?,
        credito_id: row.get(1)?,
        numero: row.get(2)?,
        fecha_vencimiento: row.get(3)?,
        fecha_pago: row.get(4)?,
        capital: row.get(5)?,
        interes: row.get(6)?,
        valor_total: row.get(7)?,
        capital_pagado: row.get(8)?,
        interes_pagado: row.get(9)?,
        mora_pagada: row.get(10)?,
        pagada: row.get::<_, i64>(11)? != 0,
        dias_atraso: row.get(12)?,
    })
}

impl CuotaPort for SqliteCuotas {
    fn cuotas_de_credito(&self, banco_id: &str, credito_id: &str) -> Result<Vec<Cuota>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, credito_id, numero, fecha_vencimiento, fecha_pago, capital, interes,
                    valor_total, capital_pagado, interes_pagado, mora_pagada, pagada, dias_atraso
             FROM cuota WHERE credito_id = ?1 ORDER BY numero",
        )?;
        let rows = stmt.query_map(params![credito_id], fila_a_cuota)?;
        let mut v = Vec::new();
        for c in rows {
            v.push(c?);
        }
        Ok(v)
    }

    fn registrar_pago(
        &self,
        banco_id: &str,
        cuota: &Cuota,
        credito_id: &str,
        nuevo_saldo: f64,
        nueva_cuota_actual: i64,
        nuevo_estatus: &str,
    ) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE cuota SET capital_pagado = ?2, interes_pagado = ?3, mora_pagada = ?4,
                              pagada = ?5, fecha_pago = ?6, dias_atraso = ?7
             WHERE id = ?1",
            params![
                cuota.id,
                cuota.capital_pagado,
                cuota.interes_pagado,
                cuota.mora_pagada,
                i64::from(cuota.pagada),
                cuota.fecha_pago,
                cuota.dias_atraso,
            ],
        )?;
        tx.execute(
            "UPDATE credito SET saldo_pendiente = ?2, cuota_actual = ?3, estatus = ?4 WHERE id = ?1",
            params![credito_id, nuevo_saldo, nueva_cuota_actual, nuevo_estatus],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn reamortizar(
        &self,
        banco_id: &str,
        credito_id: &str,
        cuota_pagada: Option<&Cuota>,
        desde_numero: i64,
        nuevas: &[CuotaReamortizada],
        nuevo_saldo: f64,
        nueva_cuota_actual: i64,
        nuevo_estatus: &str,
    ) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;

        // 1) Si el pago saldó una cuota (pago de cuota), se persiste su estado antes de tocar el resto.
        if let Some(c) = cuota_pagada {
            tx.execute(
                "UPDATE cuota SET capital_pagado = ?2, interes_pagado = ?3, mora_pagada = ?4,
                                  pagada = ?5, fecha_pago = ?6, dias_atraso = ?7
                 WHERE id = ?1",
                params![
                    c.id,
                    c.capital_pagado,
                    c.interes_pagado,
                    c.mora_pagada,
                    i64::from(c.pagada),
                    c.fecha_pago,
                    c.dias_atraso,
                ],
            )?;
        }

        // 2) Se descartan las cuotas pendientes que el plan nuevo reemplaza (nunca las ya pagadas).
        tx.execute(
            "DELETE FROM cuota WHERE credito_id = ?1 AND pagada = 0 AND numero >= ?2",
            params![credito_id, desde_numero],
        )?;

        // 3) Se inserta el plan re-amortizado.
        for c in nuevas {
            tx.execute(
                "INSERT INTO cuota
                 (id, credito_id, numero, fecha_vencimiento, capital, interes, valor_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    credito_id,
                    c.numero,
                    c.fecha_vencimiento,
                    c.capital,
                    c.interes,
                    c.valor_total,
                ],
            )?;
        }

        // 4) El crédito queda con el nuevo saldo, la cuota en curso y el estatus resultante.
        tx.execute(
            "UPDATE credito SET saldo_pendiente = ?2, cuota_actual = ?3, estatus = ?4 WHERE id = ?1",
            params![credito_id, nuevo_saldo, nueva_cuota_actual, nuevo_estatus],
        )?;

        tx.commit()?;
        Ok(())
    }
}

/// Adaptador de la tasa de mora vigente (RF-59, RF-71).
pub struct SqliteParametrosDeuda {
    config: SqliteConfiguracion,
}

impl SqliteParametrosDeuda {
    pub fn new(db: DbManager) -> Self {
        Self { config: SqliteConfiguracion::new(db) }
    }
}

impl ParametrosDeudaPort for SqliteParametrosDeuda {
    fn tasa_mora(&self, banco_id: &str) -> Result<f64, AppError> {
        Ok(self.config.obtener(banco_id)?.tasa_interes_mora)
    }
}
