use serde::Serialize;
use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{CreditoDeuda, Cuota, ResultadoAbono, ResultadoPago};

/// Estado de un crédito para la pantalla de pago: sus datos, el socio y sus cuotas.
#[derive(Debug, Clone, Serialize)]
pub struct EstadoCreditoDeuda {
    pub credito: CreditoDeuda,
    pub socio_nombre: String,
    pub cuotas: Vec<Cuota>,
}

/// RF-63: busca un crédito por número y devuelve sus cuotas para registrar el pago.
#[tauri::command]
pub fn estado_credito_deuda(
    state: State<'_, AppState>,
    numero: String,
) -> Result<EstadoCreditoDeuda, AppError> {
    let banco_id = state.banco_actual_id()?;
    let (credito, cuotas) = state.deuda.estado_credito(&banco_id, &numero)?;
    let socio_nombre = state
        .socios
        .obtener(&banco_id, &credito.socio_id)
        .map(|s| s.nombre_completo())
        .unwrap_or_else(|_| credito.socio_id.clone());
    Ok(EstadoCreditoDeuda { credito, socio_nombre, cuotas })
}

/// RF-64 a RF-72: registra el pago de una cuota (interés + capital + mora en cascada).
#[tauri::command]
pub fn registrar_pago_cuota(
    state: State<'_, AppState>,
    numero_credito: String,
    numero_cuota: i64,
    fecha: String,
    monto: f64,
) -> Result<ResultadoPago, AppError> {
    let banco_id = state.banco_actual_id()?;
    state
        .deuda
        .registrar_pago_cuota(&banco_id, &numero_credito, numero_cuota, &fecha, monto)
}

/// Pago de una o varias cuotas: un solo interés (el del mes) + mora si hay atraso + el
/// capital de las N cuotas, que re-amortiza el saldo restante.
#[tauri::command]
pub fn pagar_cuotas(
    state: State<'_, AppState>,
    numero_credito: String,
    fecha: String,
    cantidad: i64,
) -> Result<ResultadoPago, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.deuda.pagar_cuotas(&banco_id, &numero_credito, &fecha, cantidad)
}

/// Abono a capital: el monto va todo a capital, baja el saldo y re-amortiza las cuotas
/// pendientes (menos cuotas del mismo capital de referencia).
#[tauri::command]
pub fn abonar_capital(
    state: State<'_, AppState>,
    numero_credito: String,
    fecha: String,
    monto: f64,
) -> Result<ResultadoAbono, AppError> {
    let banco_id = state.banco_actual_id()?;
    state.deuda.abonar_capital(&banco_id, &numero_credito, &fecha, monto)
}
