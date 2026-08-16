use std::sync::Arc;

use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::caja::application::CajaService;
use crate::modules::caja::domain::CodigoOperacion;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;

use super::domain::{
    CierresPort, LibroContablePort, LoteAcciones, LoteAccionesPort, ParametrosAcciones,
    ParametrosAccionesPort,
};

const SELECT_LOTE: &str = "SELECT id, socio_id, mes_compra, fecha_compra, cantidad,
            valor_nominal_compra, monto_pagado, liquidada, fecha_liquidacion
     FROM lote_acciones";

fn fila_a_lote(row: &rusqlite::Row) -> rusqlite::Result<LoteAcciones> {
    Ok(LoteAcciones {
        id: row.get(0)?,
        socio_id: row.get(1)?,
        mes_compra: row.get(2)?,
        fecha_compra: row.get(3)?,
        cantidad: row.get(4)?,
        valor_nominal_compra: row.get(5)?,
        monto_pagado: row.get(6)?,
        liquidada: row.get::<_, i64>(7)? != 0,
        fecha_liquidacion: row.get(8)?,
    })
}

/// Adaptador SQLite de los lotes de acciones.
pub struct SqliteLotesAcciones {
    db: DbManager,
}

impl SqliteLotesAcciones {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl LoteAccionesPort for SqliteLotesAcciones {
    fn crear(&self, banco_id: &str, lote: &LoteAcciones) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO lote_acciones
             (id, socio_id, mes_compra, fecha_compra, cantidad, valor_nominal_compra,
              monto_pagado, liquidada, fecha_liquidacion)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                lote.id,
                lote.socio_id,
                lote.mes_compra,
                lote.fecha_compra,
                lote.cantidad,
                lote.valor_nominal_compra,
                lote.monto_pagado,
                i64::from(lote.liquidada),
                lote.fecha_liquidacion,
            ],
        )?;
        Ok(())
    }

    fn listar_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<Vec<LoteAcciones>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let sql = format!("{SELECT_LOTE} WHERE socio_id = ?1 ORDER BY mes_compra, fecha_compra");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![socio_id], fila_a_lote)?;
        let mut v = Vec::new();
        for l in rows {
            v.push(l?);
        }
        Ok(v)
    }

    fn acciones_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones
             WHERE socio_id = ?1 AND liquidada = 0",
            params![socio_id],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    fn total_acciones(&self, banco_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones WHERE liquidada = 0",
            [],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    fn acciones_por_socio(&self, banco_id: &str) -> Result<Vec<(String, i64)>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT socio_id, COALESCE(SUM(cantidad), 0) FROM lote_acciones
             WHERE liquidada = 0 GROUP BY socio_id",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut v = Vec::new();
        for p in rows {
            v.push(p?);
        }
        Ok(v)
    }

    fn vendido_en_mes(&self, banco_id: &str, mes: &str) -> Result<(i64, f64), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        // Cuenta lo vendido, se haya liquidado después o no: el cupo del mes se
        // consume al vender, y liquidar en otro momento no lo devuelve.
        conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0), COALESCE(SUM(monto_pagado), 0)
             FROM lote_acciones WHERE mes_compra = ?1",
            params![mes],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(AppError::from)
    }
}

/// Adaptador de las colocaciones selladas en cada Cierre de Mes (RN-09, D-02).
///
/// Mientras el módulo de Cierre Mensual no exista, `cierre_mes` está vacía y este
/// adaptador devuelve una lista vacía —que es la verdad: no hay meses cerrados—.
pub struct SqliteCierres {
    db: DbManager,
}

impl SqliteCierres {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl CierresPort for SqliteCierres {
    fn colocaciones_recientes(&self, banco_id: &str, cuantos: usize) -> Result<Vec<f64>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT colocacion_pct FROM cierre_mes ORDER BY mes DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![cuantos as i64], |r| r.get(0))?;
        let mut v = Vec::new();
        for c in rows {
            v.push(c?);
        }
        Ok(v)
    }
}

/// Adaptador de los parámetros que Acciones necesita de la Configuración.
///
/// El acoplamiento con el módulo de Configuración vive aquí, en la capa de datos; el
/// Dominio de Acciones sólo conoce el puerto `ParametrosAccionesPort`.
pub struct SqliteParametrosAcciones {
    db: DbManager,
    config: SqliteConfiguracion,
}

impl SqliteParametrosAcciones {
    pub fn new(db: DbManager) -> Self {
        Self {
            config: SqliteConfiguracion::new(db.clone()),
            db,
        }
    }
}

impl ParametrosAccionesPort for SqliteParametrosAcciones {
    fn obtener(&self, banco_id: &str) -> Result<ParametrosAcciones, AppError> {
        // Siembra la configuración si el Bankomunal es nuevo, igual que hace Caja.
        let cfg = self.config.obtener(banco_id)?;
        let conn = self.db.open_banco_db(banco_id)?;
        let fecha_creacion: String = conn.query_row(
            "SELECT fecha_creacion FROM configuracion WHERE id = ?1",
            params![banco_id],
            |r| r.get(0),
        )?;
        Ok(ParametrosAcciones {
            valor_nominal: cfg.valor_nominal,
            fecha_creacion,
            ppcfc_venta_rango1_pct: cfg.ppcfc_venta_rango1_pct,
            ppcfc_venta_rango2_pct: cfg.ppcfc_venta_rango2_pct,
            tope_individual_mensual_pct: cfg.tope_individual_mensual_pct,
        })
    }
}

/// Puente entre Acciones y el Libro de Ingresos y Egresos.
///
/// El Libro —su consecutivo, su saldo acumulado y su recálculo— es responsabilidad del
/// módulo de Caja. Acciones no reimplementa nada de eso: le pide a `CajaService` que
/// asiente la venta, de modo que la contabilidad siga teniendo un solo dueño.
pub struct LibroViaCaja {
    caja: Arc<CajaService>,
}

impl LibroViaCaja {
    pub fn new(caja: Arc<CajaService>) -> Self {
        Self { caja }
    }
}

impl LibroContablePort for LibroViaCaja {
    fn registrar_venta_acciones(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError> {
        self.caja
            .registrar_asiento_de_modulo(
                banco_id,
                CodigoOperacion::VentaAcciones,
                fecha.to_string(),
                monto,
                descripcion.to_string(),
                Some(socio_id.to_string()),
                None,
            )
            .map(|_| ())
    }
}
