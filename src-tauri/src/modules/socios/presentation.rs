use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{Beneficiario, DatosSocio, Protegido, Socio};

/// DTO de salida para el LISTADO de socios.
///
/// La tabla de la pantalla sólo necesita identificar y contactar al socio: no se
/// exponen beneficiario, protegidos ni saldo incobrable, que son datos sensibles y
/// además harían la respuesta N veces más grande sin que nadie los use.
#[derive(Debug, Clone, Serialize)]
pub struct SocioResumenDto {
    pub id: String,
    pub cedula: String,
    pub nombre_completo: String,
    pub celular: String,
    pub estatus: String,
    pub fecha_ingreso: String,
}

impl From<&Socio> for SocioResumenDto {
    fn from(s: &Socio) -> Self {
        Self {
            id: s.id.clone(),
            cedula: s.cedula.clone(),
            nombre_completo: s.nombre_completo(),
            celular: s.celular.clone(),
            estatus: s.estatus.as_str().to_string(),
            fecha_ingreso: s.fecha_ingreso.clone(),
        }
    }
}

/// DTO de salida para el DETALLE de un socio: el formulario completo.
#[derive(Debug, Clone, Serialize)]
pub struct SocioDto {
    pub id: String,
    pub cedula: String,
    pub nombres: String,
    pub apellidos: String,
    pub profesion: String,
    pub direccion: String,
    pub telefono: String,
    pub celular: String,
    pub correo: String,
    pub estatus: String,
    pub fecha_ingreso: String,
    pub fecha_retiro: Option<String>,
    pub beneficiario: Option<Beneficiario>,
    pub protegidos: Vec<Protegido>,
}

impl From<Socio> for SocioDto {
    fn from(s: Socio) -> Self {
        Self {
            id: s.id,
            cedula: s.cedula,
            nombres: s.nombres,
            apellidos: s.apellidos,
            profesion: s.profesion,
            direccion: s.direccion,
            telefono: s.telefono,
            celular: s.celular,
            correo: s.correo,
            estatus: s.estatus.as_str().to_string(),
            fecha_ingreso: s.fecha_ingreso,
            fecha_retiro: s.fecha_retiro,
            beneficiario: s.beneficiario,
            protegidos: s.protegidos,
            // `saldo_incobrable` se omite a propósito: lo gobierna Liquidación y no
            // tiene sentido mostrarlo ni editarlo desde el formulario del socio.
        }
    }
}

/// Cupo de socios del Bankomunal (RN-01), para que la pantalla avise antes de que el
/// usuario llene un formulario que va a ser rechazado.
#[derive(Debug, Clone, Serialize)]
pub struct CupoSociosDto {
    pub activos: usize,
    pub disponibles: usize,
    pub minimo: usize,
    pub maximo: usize,
}

/// RF-15/RF-17/RF-18/RF-20/RF-21 (CU-05): registrar un socio nuevo.
#[tauri::command]
pub fn registrar_socio(
    state: State<'_, AppState>,
    datos: DatosSocio,
) -> Result<SocioDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .socios
        .registrar(&banco_id, datos)
        .map(SocioDto::from)
}

/// RF-19 (CU-06): actualizar los datos de un socio existente.
#[tauri::command]
pub fn actualizar_socio(
    state: State<'_, AppState>,
    id: String,
    datos: DatosSocio,
) -> Result<SocioDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .socios
        .actualizar(&banco_id, &id, datos)
        .map(SocioDto::from)
}

/// RF-19: detalle de un socio para el formulario de consulta/edición.
#[tauri::command]
pub fn obtener_socio(state: State<'_, AppState>, id: String) -> Result<SocioDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.socios.obtener(&banco_id, &id).map(SocioDto::from)
}

/// RF-16: buscar por cédula para saber si el socio ya existe antes de registrarlo.
#[tauri::command]
pub fn buscar_socio_por_cedula(
    state: State<'_, AppState>,
    cedula: String,
) -> Result<Option<SocioDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(state
        .socios
        .buscar_por_cedula(&banco_id, &cedula)?
        .map(SocioDto::from))
}

/// RF-19: listado de socios para la pantalla principal del módulo.
#[tauri::command]
pub fn listar_socios(state: State<'_, AppState>) -> Result<Vec<SocioResumenDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(state
        .socios
        .listar(&banco_id)?
        .iter()
        .map(SocioResumenDto::from)
        .collect())
}

/// RN-01: cuántos socios activos hay y cuántos caben todavía.
#[tauri::command]
pub fn cupo_socios(state: State<'_, AppState>) -> Result<CupoSociosDto, AppError> {
    use super::domain::{MAX_SOCIOS, MIN_SOCIOS};
    let banco_id = state.banco_actual_id()?;
    let (activos, disponibles) = state.socios.cupo(&banco_id)?;
    Ok(CupoSociosDto {
        activos,
        disponibles,
        minimo: MIN_SOCIOS,
        maximo: MAX_SOCIOS,
    })
}
