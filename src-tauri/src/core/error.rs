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
    #[error("La clave de configuración es incorrecta")]
    ClaveIncorrecta,
    #[error("El nombre del Bankomunal ya existe")]
    NombreDuplicado,
    #[error("El nombre del Bankomunal es obligatorio y no puede estar vacío")]
    DatosIncompletos,
    #[error("No hay un Bankomunal seleccionado en la sesión")]
    SinBancoSeleccionado,
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

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serde(e.to_string())
    }
}