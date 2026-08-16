use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;
use crate::modules::auditoria::domain::EntradaAuditoria;
use crate::modules::configuracion::domain::{Configuracion, DatosGenerales};

/// RF-11/RF-12/RF-13: obtener la configuración editable del Bankomunal activo.
#[tauri::command]
pub fn obtener_configuracion(state: State<'_, AppState>) -> Result<Configuracion, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.config.obtener_configuracion(&banco_id)
}

/// RF-09/RF-10/RF-11: obtener Datos Generales de solo consulta con contadores.
#[tauri::command]
pub fn obtener_datos_generales(state: State<'_, AppState>) -> Result<DatosGenerales, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.config.obtener_datos_generales(&banco_id)
}

/// RF-12/RF-13/RF-14 + RF-112/RF-113: guardar cambios de configuración (con confirmación
/// y registro en Auditoría). Devuelve la configuración guardada para mostrar confirmación.
#[tauri::command]
pub fn actualizar_configuracion(
    state: State<'_, AppState>,
    configuracion: Configuracion,
    nombre_quien_realiza: String,
    motivo: String,
) -> Result<Configuracion, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .config
        .actualizar_configuracion(&banco_id, &configuracion, &nombre_quien_realiza, &motivo)
}

/// RF-114: consultar el historial completo de Auditoría.
#[tauri::command]
pub fn listar_auditoria(state: State<'_, AppState>) -> Result<Vec<EntradaAuditoria>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.config.listar_auditoria(&banco_id)
}