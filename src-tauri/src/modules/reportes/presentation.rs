use tauri::State;

use crate::core::error::AppError;
use crate::state::AppState;

use super::domain::{
    ReporteAccionFila, ReporteBalanceActual, ReporteCreditoFila, ReporteFiadorFila,
    ReporteGestionFila, ReporteLibroFila, ReporteSocioActivo,
};

#[tauri::command]
pub fn reporte_socios_activos(
    estado: State<'_, AppState>,
) -> Result<Vec<ReporteSocioActivo>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_socios_activos(&banco_id)
}

/// Guarda un reporte como archivo Excel (.xlsx) real en la ruta que eligió el usuario. El
/// navegador embebido de Tauri bloquea las descargas por `<a download>`, y un CSV plano se ve
/// mal (todo en una columna, con comillas); por eso se genera un .xlsx con el backend:
/// encabezado en negrita y columnas autoajustadas. Las celdas se escriben como texto para no
/// corromper valores como las cédulas (que Excel convertiría a notación científica).
#[tauri::command]
pub fn guardar_excel(
    ruta: String,
    cabeceras: Vec<String>,
    filas: Vec<Vec<String>>,
) -> Result<(), AppError> {
    use rust_xlsxwriter::{Format, Workbook};

    let mut workbook = Workbook::new();
    let hoja = workbook.add_worksheet();

    let fmt_cab = Format::new()
        .set_bold()
        .set_background_color(0xE7EDF7)
        .set_border(rust_xlsxwriter::FormatBorder::Thin);
    let err = |e: rust_xlsxwriter::XlsxError| {
        AppError::OperacionNoPermitida(format!("No se pudo generar el Excel: {e}"))
    };

    for (c, cab) in cabeceras.iter().enumerate() {
        hoja.write_string_with_format(0, c as u16, cab.as_str(), &fmt_cab)
            .map_err(err)?;
    }
    for (r, fila) in filas.iter().enumerate() {
        for (c, valor) in fila.iter().enumerate() {
            hoja.write_string((r + 1) as u32, c as u16, valor.as_str())
                .map_err(err)?;
        }
    }
    hoja.autofit();

    workbook.save(&ruta).map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn reporte_gestion(estado: State<'_, AppState>) -> Result<Vec<ReporteGestionFila>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_gestion(&banco_id)
}

#[tauri::command]
pub fn reporte_fiadores(estado: State<'_, AppState>) -> Result<Vec<ReporteFiadorFila>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_fiadores(&banco_id)
}

#[tauri::command]
pub fn reporte_creditos(estado: State<'_, AppState>) -> Result<Vec<ReporteCreditoFila>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_creditos(&banco_id)
}

#[tauri::command]
pub fn reporte_libro(estado: State<'_, AppState>) -> Result<Vec<ReporteLibroFila>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_libro(&banco_id)
}

#[tauri::command]
pub fn reporte_acciones(estado: State<'_, AppState>) -> Result<Vec<ReporteAccionFila>, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_acciones(&banco_id)
}

#[tauri::command]
pub fn reporte_balance(estado: State<'_, AppState>) -> Result<ReporteBalanceActual, AppError> {
    let banco_id = estado.banco_actual_id()?;
    estado.reportes.reporte_balance(&banco_id)
}
