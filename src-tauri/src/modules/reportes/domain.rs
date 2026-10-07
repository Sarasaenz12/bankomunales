use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteSocioActivo {
    pub id: String,
    pub cedula: String,
    pub nombres: String,
    pub apellidos: String,
    pub fecha_ingreso: String,
    pub certificados_activos: i64,
    pub liquidados_parcialmente: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteGestionFila {
    pub mes: String,
    pub fecha_cierre: String,
    pub activo_total: f64,
    pub pasivo_total: f64,
    pub ingresos_del_mes: f64,
    pub egresos_del_mes: f64,
    pub balance_neto: f64,
    pub valor_por_accion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteFiadorFila {
    pub fiador_cedula: String,
    pub fiador_nombres: String,
    pub deudor_cedula: String,
    pub deudor_nombres: String,
    pub credito_numero: String,
    pub rol: String,
    pub acciones_comprometidas: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteCreditoFila {
    pub numero: String,
    pub socio_cedula: String,
    pub socio_nombres: String,
    pub monto_original: f64,
    pub saldo_pendiente: f64,
    pub plazo_cuotas: i64,
    pub fecha_desembolso: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteLibroFila {
    pub fecha: String,
    pub operacion: String,
    pub ingreso: f64,
    pub egreso: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteAccionFila {
    pub socio_cedula: String,
    pub socio_nombres: String,
    pub cantidad_acciones: i64,
    pub valor_compra: f64,
    pub mes_compra: String,
    pub fecha_compra: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporteBalanceActual {
    pub efectivo: f64,
    pub cartera: f64,
    pub bienes: f64,
    pub fondo_gastos: f64,
    pub fondo_incobrables: f64,
    pub capital_acciones: f64,
    pub ganancias_pasivo: f64,
    /// Resultado del mes/período aún ABIERTO (sin cerrar): ingresos repartibles (OR/MO/OI)
    /// menos otros egresos (EG) de los movimientos sin cierre. Ese dinero ya está en la caja
    /// (Activo) pero todavía no se repartió a fondos ni a ganancias; forma parte del Pasivo
    /// hasta que se cierre el mes (D-20). Sin sumarlo, el Balance descuadraría por ese monto.
    pub resultado_no_distribuido: f64,
}

pub trait ReportesPort: Send + Sync {
    fn reporte_socios_activos(&self, banco_id: &str) -> Result<Vec<ReporteSocioActivo>, AppError>;
    fn reporte_gestion(&self, banco_id: &str) -> Result<Vec<ReporteGestionFila>, AppError>;
    fn reporte_fiadores(&self, banco_id: &str) -> Result<Vec<ReporteFiadorFila>, AppError>;
    fn reporte_creditos(&self, banco_id: &str) -> Result<Vec<ReporteCreditoFila>, AppError>;
    fn reporte_libro(&self, banco_id: &str) -> Result<Vec<ReporteLibroFila>, AppError>;
    fn reporte_acciones(&self, banco_id: &str) -> Result<Vec<ReporteAccionFila>, AppError>;
    fn reporte_balance(&self, banco_id: &str) -> Result<ReporteBalanceActual, AppError>;
}
