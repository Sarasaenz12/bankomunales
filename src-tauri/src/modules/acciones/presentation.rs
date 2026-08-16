use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{
    CalculoCompra, CupoMensual, LoteAcciones, NuevaCompra, ResumenMesAcciones,
};

/// DTO de un lote registrado. No expone `liquidada`/`fecha_liquidacion`: los gobierna
/// Liquidación de Acciones, que aún no existe, y hoy siempre valen lo mismo.
#[derive(Debug, Clone, Serialize)]
pub struct LoteAccionesDto {
    pub id: String,
    pub socio_id: String,
    pub mes_compra: String,
    pub fecha_compra: String,
    pub cantidad: i64,
    pub valor_nominal_compra: f64,
    pub monto_pagado: f64,
}

impl From<LoteAcciones> for LoteAccionesDto {
    fn from(l: LoteAcciones) -> Self {
        Self {
            id: l.id,
            socio_id: l.socio_id,
            mes_compra: l.mes_compra,
            fecha_compra: l.fecha_compra,
            cantidad: l.cantidad,
            valor_nominal_compra: l.valor_nominal_compra,
            monto_pagado: l.monto_pagado,
        }
    }
}

/// RF-23/RF-24/RF-25: calcular la compra antes de cobrarla.
#[tauri::command]
pub fn previsualizar_compra_acciones(
    state: State<'_, AppState>,
    socio_id: String,
    monto: f64,
) -> Result<CalculoCompra, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .acciones
        .previsualizar_compra(&banco_id, &socio_id, monto)
}

/// RF-22/RF-27 (CU-07): registrar la compra de acciones.
#[tauri::command]
pub fn registrar_compra_acciones(
    state: State<'_, AppState>,
    compra: NuevaCompra,
) -> Result<LoteAccionesDto, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .acciones
        .registrar_compra(&banco_id, compra)
        .map(LoteAccionesDto::from)
}

/// Acciones vigentes de un socio, para el detalle del socio.
#[tauri::command]
pub fn acciones_de_socio(
    state: State<'_, AppState>,
    socio_id: String,
) -> Result<i64, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.acciones.acciones_de_socio(&banco_id, &socio_id)
}

/// Acciones vigentes por socio, para la columna del listado de Socios.
#[derive(Debug, Clone, Serialize)]
pub struct AccionesDeSocioDto {
    pub socio_id: String,
    pub acciones: i64,
}

#[tauri::command]
pub fn acciones_por_socio(
    state: State<'_, AppState>,
) -> Result<Vec<AccionesDeSocioDto>, AppError> {
    let banco_id = state.banco_actual_id()?;
    Ok(state
        .acciones
        .acciones_por_socio(&banco_id)?
        .into_iter()
        .map(|(socio_id, acciones)| AccionesDeSocioDto { socio_id, acciones })
        .collect())
}

/// Total de acciones vigentes del Bankomunal.
#[tauri::command]
pub fn total_acciones(state: State<'_, AppState>) -> Result<i64, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.acciones.total_acciones(&banco_id)
}

/// RF-105: Control de Acciones del socio, mes a mes.
#[tauri::command]
pub fn historial_acciones_socio(
    state: State<'_, AppState>,
    socio_id: String,
) -> Result<Vec<ResumenMesAcciones>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.acciones.historial_de_socio(&banco_id, &socio_id)
}

/// RF-26 (RN-09 + RN-15): cupo de venta de acciones del mes.
#[tauri::command]
pub fn cupo_del_mes(
    state: State<'_, AppState>,
    fecha: Option<String>,
) -> Result<CupoMensual, AppError> {
    let banco_id = state.banco_actual_id()?;
    let fecha = fecha.unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
    state.acciones.cupo_del_mes(&banco_id, &fecha)
}
