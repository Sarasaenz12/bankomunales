use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Hombre del banco viz como entidad de dominio (sin conocimiento de SQLite).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bankomunal {
    pub id: String,
    pub nombre: String,
    pub ubicacion: String,
    pub moneda: String,
    pub fecha_creacion: String,
    pub db_path: String,
}

/// Datos requeridos para crear un nuevo Bankomunal (RF-03).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoBankomunal {
    pub nombre: String,
    pub ubicacion: String,
    pub moneda: String,
}

/// Contraseña genérica de acceso por defecto (primer uso). Es una decisión del cliente
/// documentada en el manual y debe confirmarse antes de producción.
pub const DEFAULT_PASSWORD_GENERICA: &str = "admin2026";

/// Clave del app.db donde se guarda el hash de la contraseña de acceso.
pub const SETTING_PASSWORD_HASH: &str = "password_hash";

/// --- Puertos (hexagonal): contratos que implentará un adaptador SQLite. ---

/// Acceso a las parejas clave/valor del app.db (hash de contraseña, hash de clave de config).
pub trait AppSettingsPort: Send + Sync {
    fn obtener(&self, clave: &str) -> Result<Option<String>, AppError>;
    fn guardar(&self, clave: &str, valor: &str) -> Result<(), AppError>;
}

/// Hash y verificación de contraseñas/claves (bcrypt, mínimo 10 rondas).
pub trait PasswordHasherPort: Send + Sync {
    fn hash(&self, texto: &str) -> Result<String, AppError>;
    fn verify(&self, texto: &str, hash: &str) -> Result<bool, AppError>;
}

/// Catálogo de Bankomunales: crea el `.db` del Banco, inserta la fila del catálogo
/// y expone consultas de listado y búsqueda.
pub trait BancoCatalogoPort: Send + Sync {
    fn crear(&self, id: &str, fecha_creacion: &str, nuevo: &NuevoBankomunal) -> Result<Bankomunal, AppError>;
    fn listar(&self) -> Result<Vec<Bankomunal>, AppError>;
    fn buscar_por_nombre(&self, nombre: &str) -> Result<Option<Bankomunal>, AppError>;
    fn buscar_por_id(&self, id: &str) -> Result<Option<Bankomunal>, AppError>;
}