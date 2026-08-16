use std::sync::{Arc, Mutex};

use crate::core::error::AppError;
use crate::modules::acciones::application::AccionesService;
use crate::modules::auth::application::AuthService;
use crate::modules::auth::domain::Bankomunal;
use crate::modules::caja::application::CajaService;
use crate::modules::configuracion::application::ConfigService;
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
    banco_seleccionado: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(
        auth: Arc<AuthService>,
        config: Arc<ConfigService>,
        socios: Arc<SocioService>,
        caja: Arc<CajaService>,
        acciones: Arc<AccionesService>,
    ) -> Self {
        Self {
            auth,
            config,
            socios,
            caja,
            acciones,
            banco_seleccionado: Mutex::new(None),
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
        let mut g = self.banco_seleccionado.lock().unwrap();
        *g = id;
    }

    pub fn banco_seleccionado_entity(&self) -> Result<Option<Bankomunal>, AppError> {
        match self.banco_actual_id() {
            Ok(id) => self.auth.buscar_por_id(&id),
            Err(_) => Ok(None),
        }
    }
}