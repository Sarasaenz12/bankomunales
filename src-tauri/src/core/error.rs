use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error de base de datos: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Error de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error de hash: {0}")]
    Hash(#[from] bcrypt::BcryptError),
    #[error("Base de datos no inicializada")]
    NotInitialized,
    #[error("Banco no encontrado con el nombre o id especificado")]
    BancoNoEncontrado,
    #[error("El nombre del Bankomunal ya existe")]
    NombreDuplicado,
    #[error("El nombre del Bankomunal es obligatorio y no puede estar vacío")]
    DatosIncompletos,
    #[error("No hay un Bankomunal seleccionado en la sesión")]
    SinBancoSeleccionado,
    #[error("Debe iniciar sesión antes de realizar esta operación")]
    SesionNoIniciada,
    #[error("Configuración no encontrada para el Banco")]
    ConfiguracionNoEncontrada,
    #[error("Ya existe un socio registrado con la cédula {0}")]
    CedulaDuplicada(String),
    #[error("No se encontró el socio solicitado")]
    SocioNoEncontrado,
    #[error("No se encontró la operación en el Libro de Ingresos y Egresos")]
    MovimientoNoEncontrado,
    #[error("Error de serialización: {0}")]
    Serde(String),
    #[error("Operación no permitida: {0}")]
    OperacionNoPermitida(String),
}

impl AppError {
    /// Código estable por variante. Permite al Frontend ramificar por tipo de error
    /// sin depender del texto (que puede cambiar o traducirse).
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Db(_) => "DB",
            AppError::Io(_) => "IO",
            AppError::Hash(_) => "HASH",
            AppError::NotInitialized => "NOT_INITIALIZED",
            AppError::BancoNoEncontrado => "BANCO_NO_ENCONTRADO",
            AppError::NombreDuplicado => "NOMBRE_DUPLICADO",
            AppError::DatosIncompletos => "DATOS_INCOMPLETOS",
            AppError::SinBancoSeleccionado => "SIN_BANCO_SELECCIONADO",
            AppError::SesionNoIniciada => "SESION_NO_INICIADA",
            AppError::ConfiguracionNoEncontrada => "CONFIGURACION_NO_ENCONTRADA",
            AppError::CedulaDuplicada(_) => "CEDULA_DUPLICADA",
            AppError::SocioNoEncontrado => "SOCIO_NO_ENCONTRADO",
            AppError::MovimientoNoEncontrado => "MOVIMIENTO_NO_ENCONTRADO",
            AppError::Serde(_) => "SERDE",
            AppError::OperacionNoPermitida(_) => "OPERACION_NO_PERMITIDA",
        }
    }
}

/// Se serializa como `{ "code": ..., "message": ... }` para el Frontend: `code` es
/// estable y ramificable; `message` conserva el texto legible (`Display`).
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut estado = serializer.serialize_struct("AppError", 2)?;
        estado.serialize_field("code", self.code())?;
        estado.serialize_field("message", &self.to_string())?;
        estado.end()
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e.to_string())
    }
}
