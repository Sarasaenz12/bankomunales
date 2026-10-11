use super::domain::{Respaldo, RespaldoPort};
use crate::core::error::AppError;
use chrono::Local;
use std::sync::Arc;
use uuid::Uuid;

pub struct RespaldoService {
    repo: Arc<dyn RespaldoPort>,
}

impl RespaldoService {
    pub fn new(repo: Arc<dyn RespaldoPort>) -> Self {
        Self { repo }
    }

    pub fn generar_respaldo(
        &self,
        banco_id: &str,
        destino_ruta: &str,
        usuario: &str,
    ) -> Result<(), AppError> {
        // Hacemos la copia
        self.repo.crear_copia_backup(banco_id, destino_ruta)?;

        // Registramos en BD como auditoría
        let respaldo = Respaldo {
            id: Uuid::new_v4().to_string(),
            fecha: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usuario: usuario.to_string(),
            ruta_archivo: destino_ruta.to_string(),
            tipo: "MANUAL".to_string(),
        };

        self.repo.registrar_respaldo(banco_id, respaldo)?;

        // TODO: Sería ideal registrar en la tabla auditoria, pero por RF la tabla "respaldo"
        // ya guarda esto explícitamente. Daremos por sentado que con eso sirve,
        // o si es necesario llamamos a AuditoriaPort.

        Ok(())
    }

    pub fn restaurar_respaldo(
        &self,
        banco_id: &str,
        origen_ruta: &str,
        _usuario: &str,
    ) -> Result<(), AppError> {
        // La restauración sobreescribe el db activo, por lo que el "registro" posterior
        // en la base de datos se hará en la BD recién restaurada.
        self.repo
            .validar_y_restaurar_backup(banco_id, origen_ruta)?;

        // TODO: registrar en historial que se restauró (ahora en la BD vieja,
        // pero esa BD es la que se acaba de copiar, así que registramos la novedad).
        let registro = Respaldo {
            id: Uuid::new_v4().to_string(),
            fecha: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            usuario: _usuario.to_string(),
            ruta_archivo: origen_ruta.to_string(),
            tipo: "RESTAURACION".to_string(),
        };
        self.repo.registrar_respaldo(banco_id, registro)?;

        Ok(())
    }

    pub fn listar_historial(&self, banco_id: &str) -> Result<Vec<Respaldo>, AppError> {
        self.repo.listar_respaldos(banco_id)
    }
}
