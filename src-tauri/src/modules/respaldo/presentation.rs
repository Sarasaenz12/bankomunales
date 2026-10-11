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

/// Reconoce un archivo de respaldo (de qué Bankomunal es) antes de importarlo.
#[tauri::command]
pub async fn inspeccionar_respaldo_cmd(
    ruta: String,
    estado: State<'_, AppState>,
) -> Result<super::domain::VistaRespaldo, AppError> {
    estado.exigir_autenticado()?;
    estado.respaldo.inspeccionar(&ruta)
}

/// Importa un respaldo en este computador: crea el Bankomunal si no estaba o reemplaza sus datos.
#[tauri::command]
pub async fn importar_respaldo_cmd(
    ruta: String,
    usuario: String,
    estado: State<'_, AppState>,
) -> Result<super::domain::VistaRespaldo, AppError> {
    estado.exigir_autenticado()?;
    estado.respaldo.importar(&ruta, &usuario)
}

/// Respaldo con el que se abrió la app desde el Explorador de archivos. Sigue pendiente (aunque la
/// pantalla se recargue) hasta que se llame a `olvidar_archivo_pendiente_cmd`.
#[tauri::command]
pub fn archivo_pendiente_cmd(estado: State<'_, AppState>) -> Option<String> {
    estado.ver_archivo_pendiente()
}

#[tauri::command]
pub fn olvidar_archivo_pendiente_cmd(estado: State<'_, AppState>) {
    estado.olvidar_archivo_pendiente();
}
