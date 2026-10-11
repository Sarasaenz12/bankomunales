
use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;

use super::domain::{
    BienesCierrePort, CarteraPort, CierreMes, CierresPort, Devengo, FilaLibro, FilaRepartoAnual,
    FondosPort, GuardarCierre, LibroCierrePort, LoteActivo, LoteCompraMes,
    LoteResumenDevengo, LotesPort, PagoAnualSocio, ParametrosFondos, RepartoMes, RepartoPort,
    SociosCierrePort,
};

const SELECT_CIERRE: &str = "SELECT id, mes, efectivo, cartera, bienes, activo_total,
            pasivo_total, cuadra, colocacion_pct, ppcfc_pct, fecha_cierre
     FROM cierre_mes";

fn fila_a_cierre(row: &rusqlite::Row) -> rusqlite::Result<CierreMes> {
    Ok(CierreMes {
        id: row.get(0)?,
        mes: row.get(1)?,
        efectivo: row.get(2)?,
        cartera: row.get(3)?,
        bienes: row.get(4)?,
        activo_total: row.get(5)?,
        pasivo_total: row.get(6)?,
        cuadra: row.get::<_, i64>(7)? != 0,
        colocacion_pct: row.get(8)?,
        ppcfc_pct: row.get(9)?,
        fecha_cierre: row.get(10)?,
    })
}

/// Adaptador SQLite de la tabla `cierre_mes`.
pub struct SqliteCierres {
    db: DbManager,
}

impl SqliteCierres {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl CierresPort for SqliteCierres {
    fn existe_mes(&self, banco_id: &str, mes: &str) -> Result<bool, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM cierre_mes WHERE mes = ?1",
            params![mes],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    fn listar(&self, banco_id: &str) -> Result<Vec<CierreMes>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_CIERRE} ORDER BY mes DESC"))?;
        let rows = stmt.query_map([], fila_a_cierre)?;
        let mut v = Vec::new();
        for c in rows {
            v.push(c?);
        }
        Ok(v)
    }

    fn obtener(&self, banco_id: &str, id: &str) -> Result<Option<CierreMes>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_CIERRE} WHERE id = ?1"))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(fila_a_cierre(row)?)),
            None => Ok(None),
        }
    }

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

    /// RF-95 a RF-97 + RF-39: persiste el cierre completo en una sola transacción.
    fn guardar_cierre(&self, banco_id: &str, datos: &GuardarCierre) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO cierre_mes
             (id, mes, efectivo, cartera, bienes, activo_total, pasivo_total, cuadra,
              colocacion_pct, ppcfc_pct, fecha_cierre)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                datos.cierre.id,
                datos.cierre.mes,
                datos.cierre.efectivo,
                datos.cierre.cartera,
                datos.cierre.bienes,
                datos.cierre.activo_total,
                datos.cierre.pasivo_total,
                i64::from(datos.cierre.cuadra),
                datos.cierre.colocacion_pct,
                datos.cierre.ppcfc_pct,
                datos.cierre.fecha_cierre,
            ],
        )?;

        tx.execute(
            "INSERT INTO reparto_ganancias
             (id, cierre_mes_id, mes, total_ingresos_repartibles, otros_egresos,
              fondo_incobrables, fondo_gastos, balance_neto, acciones_activas, valor_por_accion)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                datos.reparto.id,
                datos.reparto.cierre_mes_id,
                datos.reparto.mes,
                datos.reparto.total_ingresos_repartibles,
                datos.reparto.otros_egresos,
                datos.reparto.fondo_incobrables,
                datos.reparto.fondo_gastos,
                datos.reparto.balance_neto,
                datos.reparto.acciones_activas,
                datos.reparto.valor_por_accion,
            ],
        )?;

        // RF-39 (D-12): el devengo mensual de cada lote activo. El UNIQUE
        // (lote, reparto) impide duplicar el devengo si el cierre se repitiera.
        for d in &datos.devengos {
            tx.execute(
                "INSERT INTO pago_ganancia
                 (id, lote_acciones_id, reparto_ganancias_id, acciones, valor_por_accion, monto,
                  estado, fecha_pago)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'PENDIENTE', NULL)",
                params![
                    uuid::Uuid::new_v4().to_string(),
                    d.lote_acciones_id,
                    datos.reparto.id,
                    d.acciones,
                    d.valor_por_accion,
                    d.monto,
                ],
            )?;
        }

        // RF-97: los movimientos del mes quedan atados a su cierre; corregirlos a
        // partir de ahora exige nombre y motivo, y deja registro en Auditoría.
        let fin = super::domain::fin_de_mes(&datos.cierre.mes)
            .ok_or_else(|| AppError::OperacionNoPermitida("Mes inválido".into()))?;
        tx.execute(
            "UPDATE movimiento_libro SET cierre_mes_id = ?1
             WHERE fecha >= ?2 AND fecha <= ?3",
            params![datos.cierre.id, datos.cierre.mes, fin],
        )?;

        // RF-95 (D-20): la retención de los fondos sólo mueve los saldos de las
        // cuentas; no se asienta IFG en el Libro porque no hay movimiento físico.
        tx.execute(
            "UPDATE configuracion SET
                saldo_fondo_gastos     = COALESCE(saldo_fondo_gastos, 0) + ?2,
                saldo_fondo_incobrables = COALESCE(saldo_fondo_incobrables, 0) + ?3
             WHERE id = ?1",
            params![
                banco_id,
                datos.delta_fondo_gastos,
                datos.delta_fondo_incobrables,
            ],
        )?;

        tx.commit()?;
        Ok(())
    }
}

const SELECT_REPARTO: &str = "SELECT cierre_mes_id, mes, total_ingresos_repartibles,
            otros_egresos, fondo_incobrables, fondo_gastos, balance_neto,
            acciones_activas, valor_por_accion
     FROM reparto_ganancias";

fn fila_a_reparto(row: &rusqlite::Row) -> rusqlite::Result<RepartoMes> {
    Ok(RepartoMes {
        ingresos_repartibles: row.get(2)?,
        otros_egresos: row.get(3)?,
        fondo_incobrables: row.get(4)?,
        fondo_gastos: row.get(5)?,
        balance_neto: row.get(6)?,
        acciones_activas: row.get(7)?,
        valor_por_accion: row.get(8)?,
    })
}

/// Adaptador SQLite de `reparto_ganancias` y `pago_ganancia`.
pub struct SqliteReparto {
    db: DbManager,
}

impl SqliteReparto {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl RepartoPort for SqliteReparto {
    fn obtener_de_cierre(
        &self,
        banco_id: &str,
        cierre_id: &str,
    ) -> Result<Option<RepartoMes>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(&format!("{SELECT_REPARTO} WHERE cierre_mes_id = ?1"))?;
        let mut rows = stmt.query(params![cierre_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(fila_a_reparto(row)?)),
            None => Ok(None),
        }
    }

    fn devengos_de_reparto(
        &self,
        banco_id: &str,
        reparto_id: &str,
    ) -> Result<Vec<Devengo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT lote_acciones_id, lote.socio_id, pg.acciones, pg.valor_por_accion, pg.monto
             FROM pago_ganancia pg
             JOIN lote_acciones lote ON lote.id = pg.lote_acciones_id
             WHERE pg.reparto_ganancias_id = ?1
             ORDER BY pg.acciones DESC, pg.monto DESC",
        )?;
        let rows = stmt.query_map(params![reparto_id], |row| {
            Ok(Devengo {
                lote_acciones_id: row.get(0)?,
                socio_id: row.get(1)?,
                acciones: row.get(2)?,
                valor_por_accion: row.get(3)?,
                monto: row.get(4)?,
            })
        })?;
        let mut v = Vec::new();
        for d in rows {
            v.push(d?);
        }
        Ok(v)
    }

    fn id_de_cierre(
        &self,
        banco_id: &str,
        cierre_id: &str,
    ) -> Result<Option<String>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id FROM reparto_ganancias WHERE cierre_mes_id = ?1",
        )?;
        let mut rows = stmt.query(params![cierre_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// D-13: "Ganancias no repartidas" del Pasivo = PENDIENTE + LIBERADA.
    fn ganancias_no_repartidas(&self, banco_id: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto), 0) FROM pago_ganancia
             WHERE estado IN ('PENDIENTE', 'LIBERADA')",
            [],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    /// Filas mensuales del reparto entre dos meses (inclusive), para la hoja anual.
    fn filas_reparto_entre(
        &self,
        banco_id: &str,
        desde: &str,
        hasta: &str,
    ) -> Result<Vec<FilaRepartoAnual>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT mes, acciones_activas, total_ingresos_repartibles, otros_egresos,
                    fondo_gastos, fondo_incobrables, balance_neto, valor_por_accion
             FROM reparto_ganancias
             WHERE mes >= ?1 AND mes <= ?2
             ORDER BY mes",
        )?;
        let rows = stmt.query_map(params![desde, hasta], |row| {
            Ok(FilaRepartoAnual {
                mes: row.get(0)?,
                acciones_activas: row.get(1)?,
                ingresos_repartibles: row.get(2)?,
                otros_egresos: row.get(3)?,
                fondo_gastos: row.get(4)?,
                fondo_incobrables: row.get(5)?,
                balance_neto: row.get(6)?,
                valor_por_accion: row.get(7)?,
            })
        })?;
        let mut v = Vec::new();
        for f in rows {
            v.push(f?);
        }
        Ok(v)
    }

    /// Pago acumulado por socio de una cohorte (lotes comprados en `mes_cohorte`).
    /// La ganancia devengada sale de `pago_ganancia`; las acciones, de `lote_acciones`.
    /// El LEFT JOIN a una preagregación por lote evita multiplicar `cantidad` por el
    /// número de meses devengados.
    fn pagos_de_cohorte(
        &self,
        banco_id: &str,
        mes_cohorte: &str,
    ) -> Result<Vec<PagoAnualSocio>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT lote.socio_id,
                    SUM(lote.cantidad) AS acciones,
                    COALESCE(SUM(d.monto_total), 0) AS acumulado,
                    COALESCE(SUM(d.monto_pend), 0) AS pendiente
             FROM lote_acciones lote
             LEFT JOIN (
                 SELECT lote_acciones_id,
                        SUM(monto) AS monto_total,
                        SUM(CASE WHEN estado IN ('PENDIENTE','LIBERADA') THEN monto ELSE 0 END)
                            AS monto_pend
                 FROM pago_ganancia
                 GROUP BY lote_acciones_id
             ) d ON d.lote_acciones_id = lote.id
             WHERE lote.mes_compra = ?1
             GROUP BY lote.socio_id
             ORDER BY acumulado DESC, acciones DESC",
        )?;
        let rows = stmt.query_map(params![mes_cohorte], |row| {
            Ok(PagoAnualSocio {
                socio_id: row.get(0)?,
                acciones: row.get(1)?,
                monto_acumulado: row.get(2)?,
                monto_pendiente: row.get(3)?,
            })
        })?;
        let mut v = Vec::new();
        for p in rows {
            v.push(p?);
        }
        Ok(v)
    }

    /// Devengo acumulado por lote activo: total y lo que sigue pendiente (RF-42). El
    /// LEFT JOIN incluye lotes aún sin devengo (recién comprados, sin cierres todavía).
    fn resumen_por_lote(&self, banco_id: &str) -> Result<Vec<LoteResumenDevengo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT lote.id, lote.socio_id, lote.mes_compra, lote.cantidad,
                    COALESCE(SUM(pg.monto), 0) AS total,
                    COALESCE(SUM(CASE WHEN pg.estado IN ('PENDIENTE','LIBERADA')
                                      THEN pg.monto ELSE 0 END), 0) AS pendiente
             FROM lote_acciones lote
             LEFT JOIN pago_ganancia pg ON pg.lote_acciones_id = lote.id
             WHERE lote.liquidada = 0
             GROUP BY lote.id
             ORDER BY lote.mes_compra, lote.fecha_compra",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(LoteResumenDevengo {
                lote_id: row.get(0)?,
                socio_id: row.get(1)?,
                mes_compra: row.get(2)?,
                cantidad: row.get(3)?,
                accrued_total: row.get(4)?,
                accrued_pendiente: row.get(5)?,
            })
        })?;
        let mut v = Vec::new();
        for r in rows {
            v.push(r?);
        }
        Ok(v)
    }

    /// RF-42 (D-12): entrega la ganancia acumulada de un lote que cumplió su año.
    fn pagar_lote(
        &self,
        banco_id: &str,
        lote_id: &str,
        fecha_pago: &str,
    ) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let pendientes: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto), 0) FROM pago_ganancia
             WHERE lote_acciones_id = ?1 AND estado = 'PENDIENTE'",
            params![lote_id],
            |r| r.get(0),
        )?;
        if pendientes <= 0.0 {
            return Ok(0.0);
        }
        let filas = conn.execute(
            "UPDATE pago_ganancia SET estado = 'PAGADA', fecha_pago = ?2
             WHERE lote_acciones_id = ?1 AND estado = 'PENDIENTE'",
            params![lote_id, fecha_pago],
        )?;
        debug_assert!(filas > 0);
        Ok(pendientes)
    }

    fn liberar_lote(
        &self,
        banco_id: &str,
        lote_id: &str,
        fecha_liberacion: &str,
    ) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let pendientes: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto), 0) FROM pago_ganancia
             WHERE lote_acciones_id = ?1 AND estado = 'PENDIENTE'",
            params![lote_id],
            |r| r.get(0),
        )?;
        if pendientes <= 0.0 {
            return Ok(0.0);
        }
        conn.execute(
            "UPDATE pago_ganancia SET estado = 'LIBERADA', fecha_liberacion = ?2
             WHERE lote_acciones_id = ?1 AND estado = 'PENDIENTE'",
            params![lote_id, fecha_liberacion],
        )?;
        Ok(pendientes)
    }
}

/// Adaptador SQLite de las lecturas del Libro que necesita el Cierre.
pub struct SqliteLibroCierre {
    db: DbManager,
}

impl SqliteLibroCierre {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl LibroCierrePort for SqliteLibroCierre {
    fn saldo_caja_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        // El saldo de caja es acumulado: todo lo que entró menos todo lo que salió
        // hasta el fin del mes, sin importar el mes del asiento.
        let saldo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(ingreso - egreso), 0) FROM movimiento_libro
             WHERE fecha <= ?1",
            params![fin],
            |r| r.get(0),
        )?;
        Ok(saldo)
    }

    fn movimientos_del_mes(&self, banco_id: &str, mes: &str) -> Result<Vec<FilaLibro>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let fin = super::domain::fin_de_mes(mes)
            .ok_or_else(|| AppError::OperacionNoPermitida("Mes inválido".into()))?;
        let mut stmt = conn.prepare(
            "SELECT codigo, ingreso, egreso FROM movimiento_libro
             WHERE fecha >= ?1 AND fecha <= ?2",
        )?;
        let rows = stmt.query_map(params![mes, fin], |row| {
            Ok(FilaLibro { codigo: row.get(0)?, ingreso: row.get(1)?, egreso: row.get(2)? })
        })?;
        let mut v = Vec::new();
        for f in rows {
            v.push(f?);
        }
        Ok(v)
    }
}

/// Adaptador SQLite de la cartera de créditos (Balance y PPCFC).
pub struct SqliteCartera {
    db: DbManager,
}

impl SqliteCartera {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl CarteraPort for SqliteCartera {
    fn saldo_cartera_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let saldo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(saldo_pendiente), 0) FROM credito
             WHERE estatus = 'VIGENTE' AND fecha_desembolso <= ?1",
            params![fin],
            |r| r.get(0),
        )?;
        Ok(saldo)
    }

    fn creditos_vigentes_hasta(&self, banco_id: &str, fin: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM credito
             WHERE estatus = 'VIGENTE' AND fecha_desembolso <= ?1",
            params![fin],
            |r| r.get(0),
        )?;
        Ok(n)
    }
}

/// Adaptador SQLite del activo fijo.
pub struct SqliteBienesCierre {
    db: DbManager,
}

impl SqliteBienesCierre {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl BienesCierrePort for SqliteBienesCierre {
    fn valor_total_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            // Sólo PROPIO entra al Activo del Balance: el comodato es de un tercero
            // (D-20 c). Su compra ya bajó la caja con un egreso CAF, así que aquí
            // `bienes` es una permuta neta de cero contra el efectivo.
            "SELECT COALESCE(SUM(valor), 0) FROM bien
             WHERE fecha_adquisicion <= ?1 AND tipo = 'PROPIO'",
            params![fin],
            |r| r.get(0),
        )?;
        Ok(total)
    }
}

/// Adaptador SQLite de los saldos y % de los fondos (RF-95).
pub struct SqliteFondosCierre {
    db: DbManager,
    config: SqliteConfiguracion,
}

impl SqliteFondosCierre {
    pub fn new(db: DbManager) -> Self {
        Self { config: SqliteConfiguracion::new(db.clone()), db }
    }
}

impl FondosPort for SqliteFondosCierre {
    fn parametros(&self, banco_id: &str) -> Result<ParametrosFondos, AppError> {
        let cfg = self.config.obtener(banco_id)?;
        let conn = self.db.open_banco_db(banco_id)?;
        let (saldo_gastos, saldo_incobrables): (f64, f64) = conn.query_row(
            "SELECT COALESCE(saldo_fondo_gastos, 0), COALESCE(saldo_fondo_incobrables, 0)
             FROM configuracion WHERE id = ?1",
            params![banco_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        Ok(ParametrosFondos {
            pct_fondo_gastos: cfg.pct_fondo_gastos,
            pct_fondo_incobrables: cfg.pct_fondo_incobrables,
            tope_reserva_incobrables_pct: cfg.tope_reserva_incobrables_pct,
            saldo_fondo_gastos: saldo_gastos,
            saldo_fondo_incobrables: saldo_incobrables,
        })
    }

    fn ajustar_incobrables(&self, banco_id: &str, delta: f64) -> Result<(), AppError> {
        self.config.obtener(banco_id)?; // asegura que la fila exista
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "UPDATE configuracion
                SET saldo_fondo_incobrables = COALESCE(saldo_fondo_incobrables, 0) + ?2
              WHERE id = ?1",
            params![banco_id, delta],
        )?;
        Ok(())
    }
}

/// Adaptador SQLite de los lotes de acciones.
pub struct SqliteLotesCierre {
    db: DbManager,
}

impl SqliteLotesCierre {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl LotesPort for SqliteLotesCierre {
    fn lotes_activos(&self, banco_id: &str) -> Result<Vec<LoteActivo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, socio_id, cantidad FROM lote_acciones
             WHERE liquidada = 0 ORDER BY mes_compra, fecha_compra",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(LoteActivo { id: row.get(0)?, socio_id: row.get(1)?, cantidad: row.get(2)? })
        })?;
        let mut v = Vec::new();
        for l in rows {
            v.push(l?);
        }
        Ok(v)
    }

    fn acciones_activas(&self, banco_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones WHERE liquidada = 0",
            [],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    fn capital_en_acciones(&self, banco_id: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad * valor_nominal_compra), 0) FROM lote_acciones
             WHERE liquidada = 0",
            [],
            |r| r.get(0),
        )?;
        Ok(total)
    }

    fn lotes_con_mes_compra(&self, banco_id: &str) -> Result<Vec<LoteCompraMes>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, socio_id, mes_compra FROM lote_acciones WHERE liquidada = 0",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(LoteCompraMes { id: row.get(0)?, socio_id: row.get(1)?, mes_compra: row.get(2)? })
        })?;
        let mut v = Vec::new();
        for l in rows {
            v.push(l?);
        }
        Ok(v)
    }
}

/// Adaptador SQLite del conteo de socios activos (RF-92).
pub struct SqliteSociosCierre {
    db: DbManager,
}

impl SqliteSociosCierre {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl SociosCierrePort for SqliteSociosCierre {
    fn socios_activos(&self, banco_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM socio WHERE estatus = 'ACTIVO'",
            [],
            |r| r.get(0),
        )?;
        Ok(n)
    }
}

