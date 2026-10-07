use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::application::{
    DatosLiquidacion, DatosLiquidacionParcial, DatosLiquidacionSocio, DatosPagoDeudaPendiente,
    DatosRetiroSocio, LiquidacionService, PreviewLiquidacionParcial, ResultadoLiquidacionParcial,
};
use super::domain::{calcular_liquidacion, LiquidacionFinal, ParametrosLiquidacion, SocioLiquidacion};

/// Arma el servicio de Liquidación con los servicios de la sesión (clones de Arc baratos).
fn servicio(state: &AppState) -> LiquidacionService {
    LiquidacionService::new(
        state.socios.clone(),
        state.acciones.clone(),
        state.creditos.clone(),
        state.cierre.clone(),
        state.caja.clone(),
        state.config.clone(),
    )
}

#[tauri::command]
pub fn preparar_liquidacion_final(
    state: State<'_, AppState>,
) -> Result<DatosLiquidacion, AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).preparar_liquidacion_final(&banco_id)
}

/// Calcula la Liquidación Final con los insumos ya ajustados por la Asamblea. Es una
/// operación de sólo cálculo: no mueve dinero ni cierra el Banko.
#[tauri::command]
pub fn calcular_liquidacion_final(
    parametros: ParametrosLiquidacion,
    socios: Vec<SocioLiquidacion>,
) -> Result<LiquidacionFinal, AppError> {
    Ok(calcular_liquidacion(&parametros, &socios))
}

#[tauri::command]
pub fn preparar_liquidacion_socio(
    state: State<'_, AppState>,
    socio_id: String,
) -> Result<DatosLiquidacionSocio, AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).preparar_liquidacion_socio(&banco_id, &socio_id)
}

/// RF-28 / RF-38: ejecuta el retiro (liquidación total) de un socio.
#[tauri::command]
pub fn ejecutar_retiro_socio(
    state: State<'_, AppState>,
    datos: DatosRetiroSocio,
) -> Result<(), AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).ejecutar_retiro_socio(&banco_id, datos)
}

/// RF-28 (venta parcial): vista previa del capital y la ganancia a liquidar.
#[tauri::command]
pub fn previsualizar_liquidacion_parcial(
    state: State<'_, AppState>,
    socio_id: String,
    cantidad: i64,
    fecha: String,
) -> Result<PreviewLiquidacionParcial, AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).previsualizar_liquidacion_parcial(&banco_id, &socio_id, cantidad, &fecha)
}

/// RF-28 (venta parcial): ejecuta la venta de acciones libres (el socio sigue activo).
#[tauri::command]
pub fn ejecutar_liquidacion_parcial(
    state: State<'_, AppState>,
    datos: DatosLiquidacionParcial,
) -> Result<ResultadoLiquidacionParcial, AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).ejecutar_liquidacion_parcial(&banco_id, datos)
}

/// RF-74 a RF-76: Pago de Deuda Pendiente.
#[tauri::command]
pub fn registrar_pago_deuda_pendiente(
    state: State<'_, AppState>,
    datos: DatosPagoDeudaPendiente,
) -> Result<f64, AppError> {
    let banco_id = state.banco_actual_id()?;
    servicio(&state).registrar_pago_deuda_pendiente(&banco_id, datos)
}

#[tauri::command]
pub fn listar_retirados_con_deuda(
    state: State<'_, AppState>,
) -> Result<Vec<crate::modules::socios::domain::Socio>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.socios.listar_retirados_con_deuda(&banco_id)
}
