use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{
    CierreMes, CohorteSocio, CuadreMes, DesgloseRepartoMes, DetalleCierre, InformeGestionMes,
    PagoGananciaRegistrado, RepartoAnual, RepartoMes, ResultadoCierre,
};

/// Fila del resumen de pagos de ganancias, con el nombre del socio ya resuelto.
#[derive(Debug, Clone, Serialize)]
pub struct FilaPagoGanancia {
    pub socio_id: String,
    pub nombre: String,
    pub acciones_cumplen: i64,
    pub valor_ganancia_accion: f64,
    pub ganancia_a_pagar: f64,
    pub estado: String,
    pub cohortes: Vec<CohorteSocio>,
}

/// Lo que devuelve la previsualización del cuadre (RF-91 a RF-93).
#[derive(Debug, Clone, Serialize)]
pub struct PrevisualizacionCierre {
    pub cuadre: CuadreMes,
    pub informe: InformeGestionMes,
    pub reparto: RepartoMes,
}

/// RF-91 a RF-93: cuadre y reparto del mes sin tocar datos definitivos.
#[tauri::command]
pub fn previsualizar_cierre(
    state: State<'_, AppState>,
    mes: String,
) -> Result<PrevisualizacionCierre, AppError> {
    let banco_id = state.banco_actual_id()?;
    let (cuadre, informe, reparto) = state.cierre.previsualizar_cuadre(&banco_id, &mes)?;
    Ok(PrevisualizacionCierre { cuadre, informe, reparto })
}

/// RF-40/RF-41: desglose del Reparto de Ganancias del mes para la pantalla de Acciones.
#[tauri::command]
pub fn desglose_reparto_mes(
    state: State<'_, AppState>,
    mes: String,
) -> Result<DesgloseRepartoMes, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.desglose_reparto_mes(&banco_id, &mes)
}

/// RF-94 a RF-97 + RF-39: ejecuta el Cierre de Mes y devenga las ganancias.
#[tauri::command]
pub fn ejecutar_cierre(
    state: State<'_, AppState>,
    mes: String,
    nombre_quien_realiza: String,
) -> Result<ResultadoCierre, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.ejecutar_cierre(&banco_id, &mes, &nombre_quien_realiza)
}

/// Historial de cierres del Bankomunal.
#[tauri::command]
pub fn listar_cierres(state: State<'_, AppState>) -> Result<Vec<CierreMes>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.listar_cierres(&banco_id)
}

/// Detalle de un cierre: balance, reparto y devengo.
#[tauri::command]
pub fn detalle_cierre(
    state: State<'_, AppState>,
    id: String,
) -> Result<DetalleCierre, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.detalle_cierre(&banco_id, &id)
}

/// RF-39 a RF-42: la hoja de reparto anual de una cohorte (mes de compra).
#[tauri::command]
pub fn reparto_anual(state: State<'_, AppState>, mes: String) -> Result<RepartoAnual, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.reparto_anual(&banco_id, &mes)
}

/// RF-42: paga la ganancia acumulada a los lotes que cumplieron un año.
#[tauri::command]
pub fn pagar_ganancias_cumplidas(
    state: State<'_, AppState>,
    fecha: String,
) -> Result<Vec<PagoGananciaRegistrado>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.pagar_ganancias_cumplidas(&banco_id, &fecha)
}

/// RF-42 (RN-10): resumen de pagos de ganancias a una fecha, por socio, con el nombre
/// resuelto y el detalle de cohortes para el historial.
#[tauri::command]
pub fn resumen_pagos_ganancias(
    state: State<'_, AppState>,
    fecha: String,
) -> Result<Vec<FilaPagoGanancia>, AppError> {
    let banco_id = state.banco_actual_id()?;
    let resumen = state.cierre.resumen_pagos_ganancias(&banco_id, &fecha)?;
    let socios = state.socios.listar(&banco_id)?;
    let filas = resumen
        .into_iter()
        .map(|r| {
            let nombre = socios
                .iter()
                .find(|s| s.id == r.socio_id)
                .map(|s| s.nombre_completo())
                .unwrap_or_else(|| r.socio_id.clone());
            FilaPagoGanancia {
                socio_id: r.socio_id,
                nombre,
                acciones_cumplen: r.acciones_cumplen,
                valor_ganancia_accion: r.valor_ganancia_accion,
                ganancia_a_pagar: r.ganancia_a_pagar,
                estado: r.estado,
                cohortes: r.cohortes,
            }
        })
        .collect();
    Ok(filas)
}

/// RF-42 (RN-10): paga la ganancia acumulada sólo de un socio (sus lotes que cumplieron).
#[tauri::command]
pub fn pagar_ganancias_de_socio(
    state: State<'_, AppState>,
    socio_id: String,
    fecha: String,
) -> Result<Vec<PagoGananciaRegistrado>, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.cierre.pagar_ganancias_de_socio(&banco_id, &socio_id, &fecha)
}