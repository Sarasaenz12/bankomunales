use rusqlite::OptionalExtension;

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{
    ReporteAccionFila, ReporteBalanceActual, ReporteCreditoFila, ReporteFiadorFila,
    ReporteGestionFila, ReporteLibroFila, ReporteSocioActivo, ReportesPort,
};

pub struct SqliteReportes {
    db: DbManager,
}

impl SqliteReportes {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl ReportesPort for SqliteReportes {
    fn reporte_socios_activos(&self, banco_id: &str) -> Result<Vec<ReporteSocioActivo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT 
                s.id, s.cedula, s.nombres, s.apellidos, s.fecha_ingreso,
                COALESCE(SUM(CASE WHEN l.liquidada = 0 THEN l.cantidad ELSE 0 END), 0) as acciones_activas,
                COALESCE(SUM(CASE WHEN l.liquidada = 1 THEN l.cantidad ELSE 0 END), 0) as acciones_liquidadas
             FROM socio s
             LEFT JOIN lote_acciones l ON s.id = l.socio_id
             WHERE s.estatus = 'ACTIVO'
             GROUP BY s.id
             ORDER BY s.apellidos, s.nombres"
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteSocioActivo {
                    id: row.get(0)?,
                    cedula: row.get(1)?,
                    nombres: row.get(2)?,
                    apellidos: row.get(3)?,
                    fecha_ingreso: row.get(4)?,
                    certificados_activos: row.get(5)?,
                    liquidados_parcialmente: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_gestion(&self, banco_id: &str) -> Result<Vec<ReporteGestionFila>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT 
                c.mes, c.fecha_cierre, c.activo_total, c.pasivo_total,
                r.total_ingresos_repartibles, r.otros_egresos, r.balance_neto, r.valor_por_accion
             FROM cierre_mes c
             LEFT JOIN reparto_ganancias r ON c.id = r.cierre_mes_id
             ORDER BY c.mes DESC",
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteGestionFila {
                    mes: row.get(0)?,
                    fecha_cierre: row.get(1)?,
                    activo_total: row.get(2)?,
                    pasivo_total: row.get(3)?,
                    ingresos_del_mes: row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
                    egresos_del_mes: row.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                    balance_neto: row.get::<_, Option<f64>>(6)?.unwrap_or(0.0),
                    valor_por_accion: row.get::<_, Option<f64>>(7)?.unwrap_or(0.0),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_fiadores(&self, banco_id: &str) -> Result<Vec<ReporteFiadorFila>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT 
                f.cedula, f.apellidos || ', ' || f.nombres,
                s.cedula, s.apellidos || ', ' || s.nombres,
                c.numero, g.rol, g.acciones_comprometidas
             FROM garantia_credito g
             JOIN credito c ON c.id = g.credito_id
             JOIN socio f ON f.id = g.socio_id
             JOIN socio s ON s.id = c.socio_id
             WHERE c.estatus = 'VIGENTE' AND g.rol = 'FIADOR'
             ORDER BY f.apellidos, f.nombres",
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteFiadorFila {
                    fiador_cedula: row.get(0)?,
                    fiador_nombres: row.get(1)?,
                    deudor_cedula: row.get(2)?,
                    deudor_nombres: row.get(3)?,
                    credito_numero: row.get(4)?,
                    rol: row.get(5)?,
                    acciones_comprometidas: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_creditos(&self, banco_id: &str) -> Result<Vec<ReporteCreditoFila>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT 
                c.numero, s.cedula, s.apellidos || ', ' || s.nombres,
                c.monto_original, c.saldo_pendiente, c.plazo_cuotas, c.fecha_desembolso
             FROM credito c
             JOIN socio s ON s.id = c.socio_id
             WHERE c.estatus = 'VIGENTE'
             ORDER BY c.fecha_desembolso DESC",
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteCreditoFila {
                    numero: row.get(0)?,
                    socio_cedula: row.get(1)?,
                    socio_nombres: row.get(2)?,
                    monto_original: row.get(3)?,
                    saldo_pendiente: row.get(4)?,
                    plazo_cuotas: row.get(5)?,
                    fecha_desembolso: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_libro(&self, banco_id: &str) -> Result<Vec<ReporteLibroFila>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT fecha, descripcion, ingreso, egreso
             FROM movimiento_libro
             ORDER BY fecha DESC",
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteLibroFila {
                    fecha: row.get(0)?,
                    operacion: row.get(1)?,
                    ingreso: row.get(2)?,
                    egreso: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_acciones(&self, banco_id: &str) -> Result<Vec<ReporteAccionFila>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT 
                s.cedula, s.apellidos || ', ' || s.nombres,
                l.cantidad, l.valor_nominal_compra, l.mes_compra, l.fecha_compra
             FROM lote_acciones l
             JOIN socio s ON s.id = l.socio_id
             WHERE l.liquidada = 0
             ORDER BY l.fecha_compra DESC",
        )?;

        let filas = stmt
            .query_map([], |row| {
                Ok(ReporteAccionFila {
                    socio_cedula: row.get(0)?,
                    socio_nombres: row.get(1)?,
                    cantidad_acciones: row.get(2)?,
                    valor_compra: row.get(3)?,
                    mes_compra: row.get(4)?,
                    fecha_compra: row.get(5)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(filas)
    }

    fn reporte_balance(&self, banco_id: &str) -> Result<ReporteBalanceActual, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;

        // Estas sumas SIEMPRE devuelven una fila (COALESCE(SUM(...),0)); un error aquí es un
        // error real de BD y debe propagarse, no enmascararse como 0 en un reporte financiero.
        let efectivo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(ingreso - egreso), 0) FROM movimiento_libro",
            [],
            |r| r.get(0),
        )?;
        let cartera: f64 = conn.query_row(
            "SELECT COALESCE(SUM(saldo_pendiente), 0) FROM credito WHERE estatus = 'VIGENTE'",
            [],
            |r| r.get(0),
        )?;
        let bienes: f64 = conn.query_row(
            "SELECT COALESCE(SUM(valor), 0) FROM bien WHERE tipo = 'PROPIO'",
            [],
            |r| r.get(0),
        )?;
        // La fila de configuración puede no existir aún en un banco recién creado: ahí los
        // saldos de fondos son 0. Un error distinto de "sin filas" sí se propaga.
        let (fondo_gastos, fondo_incobrables): (f64, f64) = conn
            .query_row(
                "SELECT COALESCE(saldo_fondo_gastos, 0), COALESCE(saldo_fondo_incobrables, 0)
                 FROM configuracion LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .unwrap_or((0.0, 0.0));
        let capital_acciones: f64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad * valor_nominal_compra), 0) FROM lote_acciones WHERE liquidada = 0",
            [],
            |r| r.get(0),
        )?;
        let ganancias_pasivo: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto), 0) FROM pago_ganancia WHERE estado IN ('PENDIENTE', 'LIBERADA')",
            [],
            |r| r.get(0),
        )?;

        // Resultado del período ABIERTO (movimientos aún sin cerrar): ingresos repartibles
        // (OR/MO/OI) menos otros egresos (EG). Ya está en la caja pero no se ha repartido;
        // debe contarse en el Pasivo para que el Balance cuadre igual que en el Cierre (D-20).
        let resultado_no_distribuido: f64 = conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN codigo IN ('OR','MO','OI') THEN ingreso ELSE 0 END), 0)
                  - COALESCE(SUM(CASE WHEN codigo = 'EG' THEN egreso ELSE 0 END), 0)
             FROM movimiento_libro WHERE cierre_mes_id IS NULL",
            [],
            |r| r.get(0),
        )?;

        Ok(ReporteBalanceActual {
            efectivo,
            cartera,
            bienes,
            fondo_gastos,
            fondo_incobrables,
            capital_acciones,
            ganancias_pasivo,
            resultado_no_distribuido,
        })
    }
}
