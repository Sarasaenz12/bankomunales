use std::sync::Arc;

use crate::core::error::AppError;

use super::domain::{
    ReporteAccionFila, ReporteBalanceActual, ReporteCreditoFila, ReporteFiadorFila,
    ReporteGestionFila, ReporteLibroFila, ReporteSocioActivo, ReportesPort,
};

pub struct ReportesService {
    port: Arc<dyn ReportesPort>,
}

impl ReportesService {
    pub fn new(port: Arc<dyn ReportesPort>) -> Self {
        Self { port }
    }

    pub fn reporte_socios_activos(
        &self,
        banco_id: &str,
    ) -> Result<Vec<ReporteSocioActivo>, AppError> {
        self.port.reporte_socios_activos(banco_id)
    }

    pub fn reporte_gestion(&self, banco_id: &str) -> Result<Vec<ReporteGestionFila>, AppError> {
        self.port.reporte_gestion(banco_id)
    }

    pub fn reporte_fiadores(&self, banco_id: &str) -> Result<Vec<ReporteFiadorFila>, AppError> {
        self.port.reporte_fiadores(banco_id)
    }

    pub fn reporte_creditos(&self, banco_id: &str) -> Result<Vec<ReporteCreditoFila>, AppError> {
        self.port.reporte_creditos(banco_id)
    }

    pub fn reporte_libro(&self, banco_id: &str) -> Result<Vec<ReporteLibroFila>, AppError> {
        self.port.reporte_libro(banco_id)
    }

    pub fn reporte_acciones(&self, banco_id: &str) -> Result<Vec<ReporteAccionFila>, AppError> {
        self.port.reporte_acciones(banco_id)
    }

    pub fn reporte_balance(&self, banco_id: &str) -> Result<ReporteBalanceActual, AppError> {
        self.port.reporte_balance(banco_id)
    }
}
