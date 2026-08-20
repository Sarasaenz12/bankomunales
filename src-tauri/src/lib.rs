mod core;
mod modules;
mod state;

use std::sync::Arc;

use tauri::Manager;

use core::db::DbManager;
use core::error::AppError;
use modules::auth::application::AuthService;
use modules::auth::data::{BcryptHasher, SqliteAppSettings, SqliteBancoCatalogo};
use modules::configuracion::application::ConfigService;
use modules::acciones::application::AccionesService;
use modules::acciones::data::{
    LibroViaCaja, SqliteCierres, SqliteLotesAcciones, SqliteParametrosAcciones,
};
use modules::auditoria::data::SqliteAuditoria;
use modules::caja::application::CajaService;
use modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
use modules::configuracion::data::SqliteConfiguracion;
use modules::creditos::application::CreditoService;
use modules::creditos::data::{
    AccionesParaCreditoAdapter, LibroViaCaja as LibroViaCajaCreditos, ParametrosCreditoAdapter,
    SociosParaCreditoAdapter, SqliteCreditos, SqliteSolicitudes,
};
use modules::socios::application::SocioService;
use modules::socios::data::SqliteSocios;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Ruta de datos de la app: %APPDATA%/com.estra.bankomunales
            let app_path = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Serde(e.to_string()))?;
            std::fs::create_dir_all(&app_path)?;

            let db = DbManager::new(app_path);

            // Composición de dependencias (hexagonal): adaptadores concretos → servicios.
            let settings = Arc::new(SqliteAppSettings::new(db.clone()));
            let hasher = Arc::new(BcryptHasher);
            let catalogo = Arc::new(SqliteBancoCatalogo::new(db.clone()));

            let auth = Arc::new(AuthService::new(settings, hasher, catalogo, db.clone()));
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
                Arc::new(LibroViaCajaCreditos::new(caja.clone())),
            ));

            app.manage(AppState::new(auth, config, socios, caja, acciones, creditos));
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
            modules::creditos::presentation::buscar_credito_por_solicitud,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}