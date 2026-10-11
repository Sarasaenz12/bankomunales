use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Respaldo {
    pub id: String,
    pub fecha: String,
    pub usuario: String,
    pub ruta_archivo: String,
    pub tipo: String,
}

pub trait RespaldoPort: Send + Sync {
    fn registrar_respaldo(
        &self,
        banco_id: &str,
        respaldo: Respaldo,
    ) -> Result<(), crate::core::error::AppError>;
    fn crear_copia_backup(
        &self,
        banco_id: &str,
        destino_ruta: &str,
    ) -> Result<(), crate::core::error::AppError>;
    fn validar_y_restaurar_backup(
        &self,
        banco_id: &str,
        origen_ruta: &str,
    ) -> Result<(), crate::core::error::AppError>;
    fn listar_respaldos(
        &self,
        banco_id: &str,
    ) -> Result<Vec<Respaldo>, crate::core::error::AppError>;
}
