use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;
use crate::modules::auth::domain::{Bankomunal, NuevoBankomunal};

/// DTO de salida (presentación): expone solo los campos que el Frontend necesita en la
/// pantalla de selección/creación de Bankomunal. Nunca expone `db_path` ni otros campos internos.
#[derive(Debug, Clone, Serialize)]
pub struct BancoDto {
    pub id: String,
    pub nombre: String,
    pub ubicacion: String,
    pub moneda: String,
    pub fecha_creacion: String,
}

impl From<Bankomunal> for BancoDto {
    fn from(b: Bankomunal) -> Self {
        Self {
            id: b.id,
            nombre: b.nombre,
            ubicacion: b.ubicacion,
            moneda: b.moneda,
            fecha_creacion: b.fecha_creacion,
        }
    }
}

fn to_dto(banco: Bankomunal) -> BancoDto {
    BancoDto::from(banco)
}

/// RF-01/RF-02: login con contraseña genérica.
#[tauri::command]
pub fn login(state: State<'_, AppState>, password: String) -> Result<bool, AppError> {
    state.auth.login(&password)
}

/// RF-03: crear un nuevo Bankomunal con la clave de configuración inicial.
#[tauri::command]
pub fn crear_bankomunal(
    state: State<'_, AppState>,
    nuevo: NuevoBankomunal,
) -> Result<BancoDto, AppError> {
    state
        .auth
        .crear_bankomunal(nuevo)
        .map(to_dto)
}

/// RF-05: listar Bankomunales (pantalla de selección).
#[tauri::command]
pub fn listar_bankomunales(state: State<'_, AppState>) -> Result<Vec<BancoDto>, AppError> {
    state
        .auth
        .listar_bankomunales()
        .map(|v| v.into_iter().map(to_dto).collect())
}

/// RF-06: si solo existe un Bankomunal, devolverlo para entrar directo.
#[tauri::command]
pub fn unico_bankomunal(state: State<'_, AppState>) -> Result<Option<BancoDto>, AppError> {
    Ok(state.auth.unico_bankomunal()?.map(to_dto))
}

/// RF-05/RF-06/RF-07: seleccionar el Bankomunal activo de la sesión.
#[tauri::command]
pub fn seleccionar_bankomunal(
    state: State<'_, AppState>,
    id: String,
) -> Result<BancoDto, AppError> {
    let banco = state
        .auth
        .buscar_por_id(&id)?
        .ok_or(AppError::BancoNoEncontrado)?;
    // Abre el .db independiente del Banco para verificar que exista (RF-08).
    state.auth.abrir_banco_db(&banco.id)?;
    state.set_banco_seleccionado(Some(banco.id.clone()));
    Ok(to_dto(banco))
}

/// RF-07: volver a la pantalla de selección (sin cerrar la app).
#[tauri::command]
pub fn volver_a_seleccion(state: State<'_, AppState>) -> Result<(), AppError> {
    state.set_banco_seleccionado(None);
    Ok(())
}

/// Devuelve el Bankomunal seleccionado en la sesión, si lo hay.
#[tauri::command]
pub fn banco_seleccionado(state: State<'_, AppState>) -> Result<Option<BancoDto>, AppError> {
    Ok(state.banco_seleccionado_entity()?.map(to_dto))
}