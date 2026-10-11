pub mod core;
pub mod modules;
mod state;

use std::sync::Arc;

use tauri::Manager;

use crate::modules::reportes::presentation::{
    guardar_excel, reporte_acciones, reporte_balance, reporte_creditos,
    reporte_fiadores, reporte_gestion, reporte_libro, reporte_socios_activos,
};
use crate::modules::respaldo::application::RespaldoService;
use crate::modules::respaldo::data::SQLiteRespaldoRepository;
use crate::modules::respaldo::presentation::{
    crear_respaldo_cmd, listar_respaldos_cmd, restaurar_respaldo_cmd,
};
use crate::modules::socios::application::SocioService;
use crate::modules::socios::data::SqliteSocios;
use core::atomicidad::SqliteAtomicidad;
use core::db::DbManager;
use core::error::AppError;
use modules::acciones::application::AccionesService;
use modules::acciones::data::{
    SqliteCierres, SqliteLotesAcciones, SqliteParametrosAcciones,
};
use modules::auditoria::data::SqliteAuditoria;
use modules::auth::application::AuthService;
use modules::auth::data::{BcryptHasher, SqliteAppSettings, SqliteBancoCatalogo};
use modules::caja::application::CajaService;
use modules::caja::puente::LibroViaCaja;
use modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
use modules::cierre::application::CierreService;
use modules::cierre::data::{
    SqliteBienesCierre, SqliteCartera,
    SqliteCierres as SqliteCierresCierre, SqliteFondosCierre, SqliteLibroCierre, SqliteLotesCierre,
    SqliteReparto, SqliteSociosCierre,
};
use modules::configuracion::application::ConfigService;
use modules::configuracion::data::SqliteConfiguracion;
use modules::creditos::application::CreditoService;
use modules::creditos::data::{
    AccionesParaCreditoAdapter, ParametrosCreditoAdapter,
    SociosParaCreditoAdapter, SqliteCreditos, SqliteSolicitudes,
};
use modules::deuda::application::DeudaService;
use modules::liquidacion::application::LiquidacionService;

use modules::deuda::data::{
    SqliteCreditoConsulta, SqliteCuotas, SqliteParametrosDeuda,
};
use modules::reportes::application::ReportesService;
use modules::reportes::data::SqliteReportes;
use state::AppState;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Ruta de datos de la app: %APPDATA%/com.estra.bankomunales. Con la variable
            // BANKOMUNALES_DATA_DIR se usa otra carpeta (demos, capturas, pruebas manuales)
            // sin tocar los datos reales.
            let app_path = match std::env::var_os("BANKOMUNALES_DATA_DIR") {
                Some(dir) => std::path::PathBuf::from(dir),
                None => app
                    .path()
                    .app_data_dir()
                    .map_err(|e| AppError::Serde(e.to_string()))?,
            };
            std::fs::create_dir_all(&app_path)?;

            let db = DbManager::new(app_path);
            // Si la app murió en medio de una operación atómica, la deja como estaba antes.
            db.recuperar_operaciones_interrumpidas()?;

            // Composición de dependencias (hexagonal): adaptadores concretos → servicios.
            let settings = Arc::new(SqliteAppSettings::new(db.clone()));
            let hasher = Arc::new(BcryptHasher);
            let catalogo = Arc::new(SqliteBancoCatalogo::new(db.clone()));

            let auth = Arc::new(AuthService::new(settings, hasher, catalogo));
            // Primer uso: siembra los hashes por defecto (contraseña genérica y clave config).
            auth.inicializar()?;

            let config: Arc<ConfigService> = Arc::new(ConfigService::new(
                Arc::new(SqliteConfiguracion::new(db.clone())),
                Arc::new(SqliteAuditoria::new(db.clone())),
            ));

            let socios: Arc<SocioService> =
                Arc::new(SocioService::new(Arc::new(SqliteSocios::new(db.clone()))));

            let caja: Arc<CajaService> = Arc::new(CajaService::new(
                Arc::new(SqliteLibro::new(db.clone())),
                Arc::new(SqliteFondoGastos::new(db.clone())),
                Arc::new(SqliteBienes::new(db.clone())),
                Arc::new(SqliteAuditoria::new(db.clone())),
            ));

            let acciones: Arc<AccionesService> = Arc::new(AccionesService::new(
                Arc::new(SqliteLotesAcciones::new(db.clone())),
                Arc::new(SqliteParametrosAcciones::new(db.clone())),
                Arc::new(LibroViaCaja::new(caja.clone())),
                Arc::new(SqliteCierres::new(db.clone())),
            ));

            let creditos: Arc<CreditoService> = Arc::new(CreditoService::new(
                Arc::new(SqliteSolicitudes::new(db.clone())),
                Arc::new(SqliteCreditos::new(db.clone())),
                Arc::new(ParametrosCreditoAdapter::new(db.clone())),
                Arc::new(AccionesParaCreditoAdapter::new(db.clone())),
                Arc::new(SociosParaCreditoAdapter::new(db.clone())),
                Arc::new(LibroViaCaja::new(caja.clone())),
            ));

            let cierre: Arc<CierreService> = Arc::new(CierreService::new(
                Arc::new(SqliteCierresCierre::new(db.clone())),
                Arc::new(SqliteReparto::new(db.clone())),
                Arc::new(SqliteLibroCierre::new(db.clone())),
                Arc::new(SqliteCartera::new(db.clone())),
                Arc::new(SqliteBienesCierre::new(db.clone())),
                Arc::new(SqliteFondosCierre::new(db.clone())),
                Arc::new(SqliteLotesCierre::new(db.clone())),
                Arc::new(SqliteSociosCierre::new(db.clone())),
                Arc::new(LibroViaCaja::new(caja.clone())),
                Arc::new(SqliteAuditoria::new(db.clone())),
                Arc::new(SqliteAtomicidad::new(db.clone())),
            ));

            let deuda: Arc<DeudaService> = Arc::new(DeudaService::new(
                Arc::new(SqliteCreditoConsulta::new(db.clone())),
                Arc::new(SqliteCuotas::new(db.clone())),
                Arc::new(LibroViaCaja::new(caja.clone())),
                Arc::new(SqliteParametrosDeuda::new(db.clone())),
                Arc::new(SqliteAuditoria::new(db.clone())),
                Arc::new(SqliteAtomicidad::new(db.clone())),
            ));

            let liquidacion: Arc<LiquidacionService> = Arc::new(LiquidacionService::new(
                socios.clone(),
                acciones.clone(),
                creditos.clone(),
                cierre.clone(),
                caja.clone(),
                config.clone(),
                Arc::new(SqliteAtomicidad::new(db.clone())),
            ));

            let reportes: Arc<ReportesService> = Arc::new(ReportesService::new(Arc::new(
                SqliteReportes::new(db.clone()),
            )));

            let respaldo: Arc<RespaldoService> =
                Arc::new(RespaldoService::new(Arc::new(SQLiteRespaldoRepository {
                    db: Arc::new(db.clone()),
                })));

            app.manage(AppState::new(
                auth, config, socios, caja, acciones, creditos, cierre, deuda, liquidacion, reportes,
                respaldo,
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            modules::auth::presentation::login,
            modules::auth::presentation::crear_bankomunal,
            modules::auth::presentation::listar_bankomunales,
            modules::auth::presentation::unico_bankomunal,
            modules::auth::presentation::seleccionar_bankomunal,
            modules::auth::presentation::volver_a_seleccion,
            modules::auth::presentation::banco_seleccionado,
            modules::auth::presentation::sesion_iniciada,
            modules::configuracion::presentation::obtener_configuracion,
            modules::configuracion::presentation::obtener_datos_generales,
            modules::configuracion::presentation::actualizar_configuracion,
            modules::configuracion::presentation::listar_auditoria,
            modules::socios::presentation::registrar_socio,
            modules::socios::presentation::actualizar_socio,
            modules::socios::presentation::obtener_socio,
            modules::socios::presentation::buscar_socio_por_cedula,
            modules::socios::presentation::listar_socios,
            modules::socios::presentation::cupo_socios,
            modules::caja::presentation::registrar_operacion_caja,
            modules::caja::presentation::registrar_donacion,
            modules::caja::presentation::corregir_operacion_caja,
            modules::caja::presentation::listar_libro,
            modules::caja::presentation::resumen_caja,
            modules::caja::presentation::registrar_bien,
            modules::caja::presentation::listar_bienes,
            modules::acciones::presentation::previsualizar_compra_acciones,
            modules::acciones::presentation::registrar_compra_acciones,
            modules::acciones::presentation::acciones_de_socio,
            modules::acciones::presentation::acciones_por_socio,
            modules::acciones::presentation::total_acciones,
            modules::acciones::presentation::historial_acciones_socio,
            modules::acciones::presentation::cupo_del_mes,
            modules::creditos::presentation::previsualizar_tabla_credito,
            modules::creditos::presentation::registrar_solicitud,
            modules::creditos::presentation::decidir_solicitud,
            modules::creditos::presentation::listar_solicitudes,
            modules::creditos::presentation::listar_solicitudes_desembolsables,
            modules::creditos::presentation::previsualizar_desembolso,
            modules::creditos::presentation::registrar_desembolso,
            modules::creditos::presentation::registrar_refinanciamiento,
            modules::creditos::presentation::buscar_credito_por_solicitud,
            modules::creditos::presentation::siguiente_numero_credito,
            modules::creditos::presentation::listar_creditos,
            modules::creditos::presentation::creditos_de_socio,
            modules::creditos::presentation::creditos_que_avala_socio,
            modules::cierre::presentation::previsualizar_cierre,
            modules::cierre::presentation::desglose_reparto_mes,
            modules::cierre::presentation::ejecutar_cierre,
            modules::cierre::presentation::listar_cierres,
            modules::cierre::presentation::detalle_cierre,
            modules::cierre::presentation::reparto_anual,
            modules::cierre::presentation::pagar_ganancias_cumplidas,
            modules::cierre::presentation::resumen_pagos_ganancias,
            modules::cierre::presentation::pagar_ganancias_de_socio,
            modules::deuda::presentation::estado_credito_deuda,
            modules::deuda::presentation::registrar_pago_cuota,
            modules::deuda::presentation::pagar_cuotas,
            modules::deuda::presentation::abonar_capital,
            modules::liquidacion::presentation::preparar_liquidacion_final,
            modules::liquidacion::presentation::calcular_liquidacion_final,
            modules::liquidacion::presentation::preparar_liquidacion_socio,
            modules::liquidacion::presentation::ejecutar_retiro_socio,
            modules::liquidacion::presentation::previsualizar_liquidacion_parcial,
            modules::liquidacion::presentation::ejecutar_liquidacion_parcial,
            modules::liquidacion::presentation::registrar_pago_deuda_pendiente,
            modules::liquidacion::presentation::listar_retirados_con_deuda,
            reporte_socios_activos,
            reporte_gestion,
            reporte_fiadores,
            reporte_creditos,
            reporte_libro,
            reporte_acciones,
            reporte_balance,
            guardar_excel,
            crear_respaldo_cmd,
            restaurar_respaldo_cmd,
            listar_respaldos_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
