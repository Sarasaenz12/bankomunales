use std::sync::{Arc, Mutex};

use crate::core::error::AppError;
use crate::modules::acciones::application::AccionesService;
use crate::modules::auth::application::AuthService;
use crate::modules::auth::domain::Bankomunal;
use crate::modules::caja::application::CajaService;
use crate::modules::cierre::application::CierreService;
use crate::modules::configuracion::application::ConfigService;
use crate::modules::creditos::application::CreditoService;
use crate::modules::deuda::application::DeudaService;
use crate::modules::liquidacion::application::LiquidacionService;
use crate::modules::reportes::application::ReportesService;
use crate::modules::respaldo::application::RespaldoService;
use crate::modules::socios::application::SocioService;

/// Estado global de sesión de la aplicación (composition root).
/// Posee los servicios de los módulos y el Bankomunal seleccionado en esta sesión
/// (RF-05/RF-06/RF-07). Compartido por todos los comandos Tauri.
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub config: Arc<ConfigService>,
    pub socios: Arc<SocioService>,
    pub caja: Arc<CajaService>,
    pub acciones: Arc<AccionesService>,
    pub creditos: Arc<CreditoService>,
    pub cierre: Arc<CierreService>,
    pub deuda: Arc<DeudaService>,
    pub liquidacion: Arc<LiquidacionService>,
    pub reportes: Arc<ReportesService>,
    pub respaldo: Arc<RespaldoService>,
    banco_seleccionado: Mutex<Option<String>>,
    /// Sesión iniciada en el backend (RF-01/RF-02). El login deja de ser sólo visual:
    /// las operaciones de entrada al Banco (crear/seleccionar) lo exigen.
    autenticado: Mutex<bool>,
    /// Respaldo con el que se abrió la app desde el Explorador (doble clic en el archivo).
    archivo_pendiente: Mutex<Option<String>>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth: Arc<AuthService>,
        config: Arc<ConfigService>,
        socios: Arc<SocioService>,
        caja: Arc<CajaService>,
        acciones: Arc<AccionesService>,
        creditos: Arc<CreditoService>,
        cierre: Arc<CierreService>,
        deuda: Arc<DeudaService>,
        liquidacion: Arc<LiquidacionService>,
        reportes: Arc<ReportesService>,
        respaldo: Arc<RespaldoService>,
    ) -> Self {
        Self {
            auth,
            config,
            socios,
            caja,
            acciones,
            creditos,
            cierre,
            deuda,
            liquidacion,
            reportes,
            respaldo,
            banco_seleccionado: Mutex::new(None),
            autenticado: Mutex::new(false),
            archivo_pendiente: Mutex::new(None),
        }
    }

    pub fn fijar_archivo_pendiente(&self, ruta: String) {
        *self.archivo_pendiente.lock().unwrap_or_else(|e| e.into_inner()) = Some(ruta);
    }

    /// Respaldo con el que se abrió la app, si lo hay (sigue pendiente hasta `olvidar_archivo_pendiente`).
    pub fn ver_archivo_pendiente(&self) -> Option<String> {
        self.archivo_pendiente.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn olvidar_archivo_pendiente(&self) {
        self.archivo_pendiente.lock().unwrap_or_else(|e| e.into_inner()).take();
    }

    /// Marca la sesión como autenticada tras un login válido.
    pub fn marcar_autenticado(&self) {
        *self.autenticado.lock().unwrap_or_else(|e| e.into_inner()) = true;
    }

    /// Indica si esta sesión ya inició sesión correctamente.
    pub fn esta_autenticado(&self) -> bool {
        *self.autenticado.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Exige sesión iniciada; error de dominio claro si no lo está.
    pub fn exigir_autenticado(&self) -> Result<(), AppError> {
        if self.esta_autenticado() {
            Ok(())
        } else {
            Err(AppError::SesionNoIniciada)
        }
    }

    pub fn banco_actual_id(&self) -> Result<String, AppError> {
        self.banco_seleccionado
            .lock()
            .map(|g| g.clone())
            .ok()
            .flatten()
            .ok_or(AppError::SinBancoSeleccionado)
    }

    pub fn set_banco_seleccionado(&self, id: Option<String>) {
        let mut g = self.banco_seleccionado.lock().unwrap_or_else(|e| e.into_inner());
        *g = id;
    }

    pub fn banco_seleccionado_entity(&self) -> Result<Option<Bankomunal>, AppError> {
        match self.banco_actual_id() {
            Ok(id) => self.auth.buscar_por_id(&id),
            Err(_) => Ok(None),
        }
    }
}
