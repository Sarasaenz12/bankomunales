use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{
    Credito, CuotaPlaneada, DecisionSolicitud, EstadoSolicitud, NuevaSolicitud, NuevoDesembolso,
    SolicitudCredito, TablaCredito,
};

/// DTO de una cuota calculada (RF-46/RF-61). Idéntico a `CuotaPlaneada` del dominio,
/// pero expuesto aparte para que la API no cambie si el modelo interno evoluciona.
#[derive(Debug, Clone, Serialize)]
pub struct CuotaDto {
    pub numero: i64,
    pub fecha_vencimiento: String,
    pub capital: f64,
    pub interes: f64,
    pub valor_total: f64,
}

impl From<&CuotaPlaneada> for CuotaDto {
    fn from(c: &CuotaPlaneada) -> Self {
        Self {
            numero: c.numero,
            fecha_vencimiento: c.fecha_vencimiento.clone(),
            capital: c.capital,
            interes: c.interes,
            valor_total: c.valor_total,
        }
    }
}

/// DTO de la tabla de amortización (RF-46).
#[derive(Debug, Clone, Serialize)]
pub struct TablaCreditoDto {
    pub cuotas: Vec<CuotaDto>,
    pub monto_total: f64,
    pub cuota_mensual: f64,
    pub capital_cuota: f64,
    pub interes_cuota: f64,
}

impl From<TablaCredito> for TablaCreditoDto {
    fn from(t: TablaCredito) -> Self {
        Self {
            cuotas: t.cuotas.iter().map(CuotaDto::from).collect(),
            monto_total: t.monto_total,
            cuota_mensual: t.cuota_mensual,
            capital_cuota: t.capital_cuota,
            interes_cuota: t.interes_cuota,
        }
    }
}

/// DTO de una solicitud de crédito (RF-43..RF-52).
#[derive(Debug, Clone, Serialize)]
pub struct SolicitudDto {
    pub id: String,
    pub socio_id: String,
    pub fecha_solicitud: String,
    pub monto_solicitado: f64,
    pub plazo_cuotas: i64,
    pub destino: &'static str,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    pub capacidad_pago: f64,
    pub estado: &'static str,
    pub monto_aprobado: Option<f64>,
    pub observacion: Option<String>,
    pub fecha_decision: Option<String>,
    pub decidida_por: Option<String>,
    pub garantias: Vec<GarantiaSolicitudDto>,
}

impl From<SolicitudCredito> for SolicitudDto {
    fn from(s: SolicitudCredito) -> Self {
        Self {
            id: s.id,
            socio_id: s.socio_id,
            fecha_solicitud: s.fecha_solicitud,
            monto_solicitado: s.monto_solicitado,
            plazo_cuotas: s.plazo_cuotas,
            destino: s.destino.as_str(),
            total_ingresos: s.total_ingresos,
            total_egresos: s.total_egresos,
            capacidad_pago: s.capacidad_pago,
            estado: s.estado.as_str(),
            monto_aprobado: s.monto_aprobado,
            observacion: s.observacion,
            fecha_decision: s.fecha_decision,
            decidida_por: s.decidida_por,
            garantias: s.garantias.into_iter().map(GarantiaSolicitudDto::from).collect(),
        }
    }
}

/// DTO de una garantía de solicitud (RF-48).
#[derive(Debug, Clone, Serialize)]
pub struct GarantiaSolicitudDto {
    pub socio_id: String,
    pub rol: &'static str,
    pub acciones_comprometidas: f64,
}

impl From<super::domain::GarantiaSolicitud> for GarantiaSolicitudDto {
    fn from(g: super::domain::GarantiaSolicitud) -> Self {
        Self {
            socio_id: g.socio_id,
            rol: g.rol.as_str(),
            acciones_comprometidas: g.acciones_comprometidas,
        }
    }
}

/// DTO de un crédito desembolsado (RF-53..RF-62).
#[derive(Debug, Clone, Serialize)]
pub struct CreditoDto {
    pub id: String,
    pub socio_id: String,
    pub numero: String,
    pub monto_original: f64,
    pub tasa: f64,
    pub plazo_cuotas: i64,
    pub cuota_actual: i64,
    pub saldo_pendiente: f64,
    pub destino: &'static str,
    pub estatus: &'static str,
    pub fecha_solicitud: String,
    pub fecha_desembolso: String,
    pub frecuencia_pago: String,
    pub fecha_vencimiento: String,
    pub solicitud_id: Option<String>,
    pub garantias: Vec<GarantiaCreditoDto>,
}

impl From<Credito> for CreditoDto {
    fn from(c: Credito) -> Self {
        Self {
            id: c.id,
            socio_id: c.socio_id,
            numero: c.numero,
            monto_original: c.monto_original,
            tasa: c.tasa,
            plazo_cuotas: c.plazo_cuotas,
            cuota_actual: c.cuota_actual,
            saldo_pendiente: c.saldo_pendiente,
            destino: c.destino.as_str(),
            estatus: c.estatus.as_str(),
            fecha_solicitud: c.fecha_solicitud,
            fecha_desembolso: c.fecha_desembolso,
            frecuencia_pago: c.frecuencia_pago,
            fecha_vencimiento: c.fecha_vencimiento,
            solicitud_id: c.solicitud_id,
            garantias: c.garantias.into_iter().map(GarantiaCreditoDto::from).collect(),
        }
    }
}

/// DTO de una garantía de crédito (RF-57, RN-04).
#[derive(Debug, Clone, Serialize)]
pub struct GarantiaCreditoDto {
    pub socio_id: String,
    pub rol: &'static str,
    pub acciones_comprometidas: f64,
}

impl From<super::domain::GarantiaCredito> for GarantiaCreditoDto {
    fn from(g: super::domain::GarantiaCredito) -> Self {
        Self {
            socio_id: g.socio_id,
            rol: g.rol.as_str(),
            acciones_comprometidas: g.acciones_comprometidas,
        }
    }
}

/// RF-46 (CU-08): tabla de amortización para la pantalla de Solicitud.
#[tauri::command]
pub fn previsualizar_tabla_credito(
    state: State<'_, AppState>,
    monto: f64,
    plazo: i64,
) -> Result<TablaCreditoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.creditos.previsualizar_tabla(&banco_id, monto, plazo).map(TablaCreditoDto::from)
}

/// RF-43..RF-49 (CU-08): registrar una solicitud de crédito.
#[tauri::command]
pub fn registrar_solicitud(
    state: State<'_, AppState>,
    solicitud: NuevaSolicitud,
) -> Result<SolicitudDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.creditos.registrar_solicitud(&banco_id, solicitud).map(SolicitudDto::from)
}

/// RF-50/RF-51 (CU-08): decisión de la Junta sobre una solicitud.
#[tauri::command]
pub fn decidir_solicitud(
    state: State<'_, AppState>,
    decision: DecisionSolicitud,
) -> Result<SolicitudDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.creditos.decidir_solicitud(&banco_id, decision).map(SolicitudDto::from)
}

/// RF-52 (CU-08): listado de solicitudes, opcionalmente por estado.
#[tauri::command]
pub fn listar_solicitudes(
    state: State<'_, AppState>,
    estado: Option<EstadoSolicitud>,
) -> Result<Vec<SolicitudDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .creditos
        .listar_solicitudes(&banco_id, estado)
        .map(|v| v.into_iter().map(SolicitudDto::from).collect())
}

/// RF-52/RF-54: solicitudes aprobadas que aún NO tienen crédito desembolsado,
/// para el selector de la pantalla de Desembolso.
#[tauri::command]
pub fn listar_solicitudes_desembolsables(
    state: State<'_, AppState>,
) -> Result<Vec<SolicitudDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .creditos
        .listar_solicitudes_desembolsables(&banco_id)
        .map(|v| v.into_iter().map(SolicitudDto::from).collect())
}

/// RF-54 (CU-09): vista previa del desembolso antes de confirmar.
#[tauri::command]
pub fn previsualizar_desembolso(
    state: State<'_, AppState>,
    monto: f64,
    plazo: i64,
) -> Result<TablaCreditoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.creditos.previsualizar_desembolso(&banco_id, monto, plazo).map(TablaCreditoDto::from)
}

/// RF-52: consulta el crédito desembolsado de una solicitud (para "Ver crédito").
#[tauri::command]
pub fn buscar_credito_por_solicitud(
    state: State<'_, AppState>,
    solicitudId: String,
) -> Result<Option<CreditoDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .creditos
        .buscar_credito_por_solicitud(&banco_id, &solicitudId)
        .map(|c| c.map(CreditoDto::from))
}

/// RF-53..RF-62 (CU-09): desembolsar el crédito y asentarlo en caja (CON).
#[tauri::command]
pub fn registrar_desembolso(
    state: State<'_, AppState>,
    desembolso: NuevoDesembolso,
) -> Result<CreditoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.creditos.registrar_desembolso(&banco_id, desembolso).map(CreditoDto::from)
}
