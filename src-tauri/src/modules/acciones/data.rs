
use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;

use super::domain::{
    CierresPort, LoteAcciones, LoteAccionesPort, LoteLiquidado,
    ParametrosAcciones, ParametrosAccionesPort,
};

/// Fila de `pago_ganancia` que se reparte al partir un lote: (id, reparto, acciones,
/// valor_por_accion, monto, estado, fecha_pago, fecha_liberacion).
type FilaPagoGanancia = (String, String, i64, f64, f64, String, Option<String>, Option<String>);

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

    fn eliminar(&self, banco_id: &str, lote_id: &str) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute("DELETE FROM lote_acciones WHERE id = ?1", params![lote_id])?;
        Ok(())
    }

    fn listar_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
    ) -> Result<Vec<LoteAcciones>, AppError> {
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

    fn capital_activo_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto_pagado), 0) FROM lote_acciones
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

    fn monto_de_socio_en_mes(
        &self,
        banco_id: &str,
        socio_id: &str,
        mes: &str,
    ) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto_pagado), 0)
             FROM lote_acciones WHERE socio_id = ?1 AND mes_compra = ?2",
            params![socio_id, mes],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    fn liquidar_lotes_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
        fecha: &str,
    ) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let acciones: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones WHERE socio_id = ?1 AND liquidada = 0",
            params![socio_id],
            |r| r.get(0),
        )?;
        conn.execute(
            "UPDATE lote_acciones SET liquidada = 1, fecha_liquidacion = ?2 WHERE socio_id = ?1 AND liquidada = 0",
            params![socio_id, fecha],
        )?;
        Ok(acciones)
    }

    fn liquidar_acciones_parcial(
        &self,
        banco_id: &str,
        socio_id: &str,
        cantidad: i64,
        fecha: &str,
    ) -> Result<Vec<LoteLiquidado>, AppError> {
        if cantidad < 1 {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar cuántas acciones liquidar (al menos 1)".into(),
            ));
        }
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;

        // Lotes vivos del socio, del más antiguo al más nuevo (FIFO).
        let lotes: Vec<LoteAcciones> = {
            let sql = format!(
                "{SELECT_LOTE} WHERE socio_id = ?1 AND liquidada = 0 \
                 ORDER BY mes_compra, fecha_compra"
            );
            let mut stmt = tx.prepare(&sql)?;
            let rows = stmt.query_map(params![socio_id], fila_a_lote)?;
            let mut v = Vec::new();
            for l in rows {
                v.push(l?);
            }
            v
        };

        let disponibles: i64 = lotes.iter().map(|l| l.cantidad).sum();
        if cantidad > disponibles {
            return Err(AppError::OperacionNoPermitida(format!(
                "El socio sólo tiene {disponibles} acciones activas; no puede liquidar {cantidad}"
            )));
        }

        let mut restante = cantidad;
        let mut liquidados: Vec<LoteLiquidado> = Vec::new();

        for lote in &lotes {
            if restante <= 0 {
                break;
            }
            let take = restante.min(lote.cantidad);

            if take == lote.cantidad {
                // Se consume el lote entero: se marca liquidado tal cual.
                tx.execute(
                    "UPDATE lote_acciones SET liquidada = 1, fecha_liquidacion = ?2 WHERE id = ?1",
                    params![lote.id, fecha],
                )?;
                liquidados.push(LoteLiquidado {
                    lote_id: lote.id.clone(),
                    mes_compra: lote.mes_compra.clone(),
                    cantidad: take,
                    capital: lote.monto_pagado,
                });
            } else {
                // Se parte el lote: un trozo nuevo (liquidado) con `take` acciones y el
                // original se queda con el resto, activo. El capital se reparte por el
                // nominal congelado; el residuo del redondeo se deja en el lote original.
                let capital_trozo = (take as f64 * lote.valor_nominal_compra).round();
                let nuevo_id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO lote_acciones
                     (id, socio_id, mes_compra, fecha_compra, cantidad, valor_nominal_compra,
                      monto_pagado, liquidada, fecha_liquidacion)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
                    params![
                        nuevo_id,
                        lote.socio_id,
                        lote.mes_compra,
                        lote.fecha_compra,
                        take,
                        lote.valor_nominal_compra,
                        capital_trozo,
                        fecha,
                    ],
                )?;
                tx.execute(
                    "UPDATE lote_acciones SET cantidad = cantidad - ?2, monto_pagado = monto_pagado - ?3
                     WHERE id = ?1",
                    params![lote.id, take, capital_trozo],
                )?;

                // Los devengos del lote se prorratean al trozo liquidado: por cada fila de
                // pago_ganancia se mueve `take/C` del monto (y de las acciones) a una copia
                // que apunta al trozo nuevo, dejando el resto en el original. El residuo del
                // redondeo se queda en la fila original, así Σ(monto) por reparto no cambia
                // y las "ganancias no repartidas" del Pasivo se conservan exactas.
                let filas: Vec<FilaPagoGanancia> = {
                    let mut stmt = tx.prepare(
                        "SELECT id, reparto_ganancias_id, acciones, valor_por_accion, monto,
                                estado, fecha_pago, fecha_liberacion
                         FROM pago_ganancia WHERE lote_acciones_id = ?1",
                    )?;
                    let rows = stmt.query_map(params![lote.id], |r| {
                        Ok((
                            r.get(0)?,
                            r.get(1)?,
                            r.get(2)?,
                            r.get(3)?,
                            r.get(4)?,
                            r.get(5)?,
                            r.get(6)?,
                            r.get(7)?,
                        ))
                    })?;
                    let mut v = Vec::new();
                    for f in rows {
                        v.push(f?);
                    }
                    v
                };
                let c = lote.cantidad as f64;
                for (id, reparto_id, acc, vpa, monto, estado, fpago, flib) in filas {
                    let monto_trozo = (monto * take as f64 / c).round();
                    let acc_trozo = ((acc as f64) * take as f64 / c).floor() as i64;
                    if monto_trozo.abs() < 0.005 && acc_trozo == 0 {
                        continue;
                    }
                    tx.execute(
                        "INSERT INTO pago_ganancia
                         (id, lote_acciones_id, reparto_ganancias_id, acciones, valor_por_accion,
                          monto, estado, fecha_pago, fecha_liberacion)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                        params![
                            uuid::Uuid::new_v4().to_string(),
                            nuevo_id,
                            reparto_id,
                            acc_trozo,
                            vpa,
                            monto_trozo,
                            estado,
                            fpago,
                            flib,
                        ],
                    )?;
                    tx.execute(
                        "UPDATE pago_ganancia SET acciones = acciones - ?2, monto = monto - ?3
                         WHERE id = ?1",
                        params![id, acc_trozo, monto_trozo],
                    )?;
                }

                liquidados.push(LoteLiquidado {
                    lote_id: nuevo_id,
                    mes_compra: lote.mes_compra.clone(),
                    cantidad: take,
                    capital: capital_trozo,
                });
            }

            restante -= take;
        }

        tx.commit()?;
        Ok(liquidados)
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
        let mut stmt =
            conn.prepare("SELECT colocacion_pct FROM cierre_mes ORDER BY mes DESC LIMIT ?1")?;
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
        // `fecha_creacion` no forma parte de los parámetros de `Configuracion`
        // (vive en DatosGenerales), por eso se lee aquí de su columna.
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

