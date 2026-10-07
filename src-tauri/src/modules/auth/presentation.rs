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

/// RF-01/RF-02: login con contraseña genérica. Si es válida, marca la sesión como
/// autenticada en el backend (las operaciones de entrada al Banco lo exigen).
#[tauri::command]
pub fn login(state: State<'_, AppState>, password: String) -> Result<bool, AppError> {
    let ok = state.auth.login(&password)?;
    if ok {
        state.marcar_autenticado();
    }
    Ok(ok)
}

/// RF-03: crear un nuevo Bankomunal. Requiere sesión iniciada.
#[tauri::command]
pub fn crear_bankomunal(
    state: State<'_, AppState>,
    nuevo: NuevoBankomunal,
) -> Result<BancoDto, AppError> {
    state.exigir_autenticado()?;
    state
        .auth
        .crear_bankomunal(nuevo)
        .map(BancoDto::from)
}

/// RF-05: listar Bankomunales (pantalla de selección).
#[tauri::command]
pub fn listar_bankomunales(state: State<'_, AppState>) -> Result<Vec<BancoDto>, AppError> {
    state
        .auth
        .listar_bankomunales()
        .map(|v| v.into_iter().map(BancoDto::from).collect())
}

/// RF-06: si solo existe un Bankomunal, devolverlo para entrar directo.
#[tauri::command]
pub fn unico_bankomunal(state: State<'_, AppState>) -> Result<Option<BancoDto>, AppError> {
    Ok(state.auth.unico_bankomunal()?.map(BancoDto::from))
}

/// RF-05/RF-06/RF-07: seleccionar el Bankomunal activo de la sesión. Requiere sesión iniciada.
#[tauri::command]
pub fn seleccionar_bankomunal(
    state: State<'_, AppState>,
    id: String,
) -> Result<BancoDto, AppError> {
    state.exigir_autenticado()?;
    let banco = state
        .auth
        .buscar_por_id(&id)?
        .ok_or(AppError::BancoNoEncontrado)?;
    // Abre el .db independiente del Banco para verificar que exista (RF-08).
    state.auth.abrir_banco_db(&banco.id)?;
    state.set_banco_seleccionado(Some(banco.id.clone()));
    Ok(BancoDto::from(banco))
}

/// RF-07: volver a la pantalla de selección (sin cerrar la app). La sesión sigue
/// iniciada: cambiar de banco no vuelve a pedir la contraseña (RF-01/RF-02).
#[tauri::command]
pub fn volver_a_seleccion(state: State<'_, AppState>) -> Result<(), AppError> {
    state.set_banco_seleccionado(None);
    Ok(())
}

/// Indica si esta sesión ya inició sesión con la contraseña. La pantalla de selección la
/// consulta para no volver a pedir la contraseña al cambiar de banco.
#[tauri::command]
pub fn sesion_iniciada(state: State<'_, AppState>) -> Result<bool, AppError> {
    Ok(state.esta_autenticado())
}

/// Devuelve el Bankomunal seleccionado en la sesión, si lo hay.
#[tauri::command]
pub fn banco_seleccionado(state: State<'_, AppState>) -> Result<Option<BancoDto>, AppError> {
    Ok(state.banco_seleccionado_entity()?.map(BancoDto::from))
}