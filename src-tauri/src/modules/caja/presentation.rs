use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{Bien, FiltroLibro, Movimiento, NuevaOperacion, NuevoBien};

/// DTO de una línea del Libro de Ingresos y Egresos.
///
/// Se aplanan `ingreso`/`egreso` en el par que la tabla necesita y se expone `codigo`
/// ya como texto. No se filtran `socio_id`/`credito_id` porque la pantalla los usa para
/// enlazar al socio o al crédito del que salió el asiento.
#[derive(Debug, Clone, Serialize)]
pub struct MovimientoDto {
    pub id: String,
    pub numero: i64,
    pub fecha: String,
    pub codigo: String,
    pub descripcion: String,
    pub ingreso: f64,
    pub egreso: f64,
    pub saldo: f64,
    pub socio_id: Option<String>,
    pub credito_id: Option<String>,
    /// Si su mes ya fue cerrado, corregirlo exigirá nombre y motivo (RF-90).
    pub mes_cerrado: bool,
    pub corregido: bool,
    pub corregido_por: Option<String>,
    pub motivo_correccion: Option<String>,
}

impl From<Movimiento> for MovimientoDto {
    fn from(m: Movimiento) -> Self {
        Self {
            mes_cerrado: m.mes_cerrado(),
            id: m.id,
            numero: m.numero,
            fecha: m.fecha,
            codigo: m.codigo.as_str().to_string(),
            descripcion: m.descripcion,
            ingreso: m.ingreso,
            egreso: m.egreso,
            saldo: m.saldo,
            socio_id: m.socio_id,
            credito_id: m.credito_id,
            corregido: m.corregido,
            corregido_por: m.corregido_por,
            motivo_correccion: m.motivo_correccion,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BienDto {
    pub id: String,
    pub descripcion: String,
    pub fecha_adquisicion: String,
    pub valor: f64,
    pub tipo: String,
}

impl From<Bien> for BienDto {
    fn from(b: Bien) -> Self {
        Self {
            id: b.id,
            descripcion: b.descripcion,
            fecha_adquisicion: b.fecha_adquisicion,
            valor: b.valor,
            tipo: b.tipo.as_str().to_string(),
        }
    }
}

/// Resumen de cabecera de la pantalla de Caja.
#[derive(Debug, Clone, Serialize)]
pub struct ResumenCajaDto {
    pub saldo_caja: f64,
    pub saldo_fondo_gastos: f64,
    pub valor_activo_fijo: f64,
}

/// RF-83 a RF-86 (CU-16, CU-17): registrar una operación en el Libro.
#[tauri::command]
pub fn registrar_operacion_caja(
    state: State<'_, AppState>,
    operacion: NuevaOperacion,
) -> Result<MovimientoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .caja
        .registrar_operacion(&banco_id, operacion)
        .map(MovimientoDto::from)
}

/// RF-87: registrar una donación, que entra al Fondo para Gastos.
#[tauri::command]
pub fn registrar_donacion(
    state: State<'_, AppState>,
    fecha: String,
    monto: f64,
    descripcion: String,
) -> Result<MovimientoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .caja
        .registrar_donacion(&banco_id, fecha, monto, descripcion)
        .map(MovimientoDto::from)
}

/// RF-89/RF-90 (CU-18): corregir una operación. Tras el Cierre de Mes, `nombre_quien_realiza`
/// y `motivo` son obligatorios y la corrección queda en Auditoría.
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn corregir_operacion_caja(
    state: State<'_, AppState>,
    id: String,
    fecha: String,
    monto: f64,
    descripcion: String,
    nombre_quien_realiza: Option<String>,
    motivo: Option<String>,
) -> Result<MovimientoDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .caja
        .corregir_operacion(
            &banco_id, &id, fecha, monto, descripcion, nombre_quien_realiza, motivo,
        )
        .map(MovimientoDto::from)
}

/// Libro de Ingresos y Egresos, opcionalmente acotado por rango de fechas.
#[tauri::command]
pub fn listar_libro(
    state: State<'_, AppState>,
    filtro: Option<FiltroLibro>,
) -> Result<Vec<MovimientoDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(state
        .caja
        .listar_libro(&banco_id, filtro.unwrap_or_default())?
        .into_iter()
        .map(MovimientoDto::from)
        .collect())
}

/// Saldos de cabecera: caja, Fondo para Gastos y activo fijo.
#[tauri::command]
pub fn resumen_caja(state: State<'_, AppState>) -> Result<ResumenCajaDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(ResumenCajaDto {
        saldo_caja: state.caja.saldo_caja(&banco_id)?,
        saldo_fondo_gastos: state.caja.saldo_fondo_gastos(&banco_id)?,
        valor_activo_fijo: state.caja.valor_activo_fijo(&banco_id)?,
    })
}

/// RF-88: registrar un Bien Adquirido como Activo Fijo.
#[tauri::command]
pub fn registrar_bien(
    state: State<'_, AppState>,
    bien: NuevoBien,
) -> Result<BienDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.caja.registrar_bien(&banco_id, bien).map(BienDto::from)
}

#[tauri::command]
pub fn listar_bienes(state: State<'_, AppState>) -> Result<Vec<BienDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(state
        .caja
        .listar_bienes(&banco_id)?
        .into_iter()
        .map(BienDto::from)
        .collect())
}
