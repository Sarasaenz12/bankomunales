use crate::core::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn crear_respaldo_cmd(
    destino_ruta: String,
    usuario: String,
    estado: State<'_, AppState>,
) -> Result<(), AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado
        .respaldo
        .generar_respaldo(&banco_id, &destino_ruta, &usuario)
}

#[tauri::command]
pub async fn restaurar_respaldo_cmd(
    origen_ruta: String,
    usuario: String,
    estado: State<'_, AppState>,
) -> Result<(), AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado
        .respaldo
        .restaurar_respaldo(&banco_id, &origen_ruta, &usuario)
}

#[tauri::command]
pub async fn listar_respaldos_cmd(
    estado: State<'_, AppState>,
) -> Result<Vec<super::domain::Respaldo>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.respaldo.listar_historial(&banco_id)
}
