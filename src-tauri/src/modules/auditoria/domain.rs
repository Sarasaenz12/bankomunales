use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Tipos de acción que quedan registrados en la bitácora (RNF-08).
///
/// La Auditoría no es un control de acceso —el sistema no bloquea por rol (ADR-04)—
/// sino la trazabilidad que lo compensa: deja constancia de quién hizo qué.
pub mod tipo_accion {
    /// Cambio de un parámetro de Configuración (RF-112).
    pub const MODIFICACION: &str = "MODIFICACION";
    /// Corrección de una operación del Libro tras el Cierre de Mes (RF-90, RF-97).
    pub const CORRECCION_OPERACION: &str = "CORRECCION_OPERACION";
    /// Ejecución del Cierre de Mes (RF-91 a RF-97).
    pub const CIERRE_MES: &str = "CIERRE_MES";
    /// Pago de las ganancias devengadas al cumplir el año (RF-42).
    pub const PAGO_GANANCIAS: &str = "PAGO_GANANCIAS";
    /// Pago de una cuota de crédito: interés, capital y/o mora (RF-63 a RF-72).
    pub const PAGO_CUOTA: &str = "PAGO_CUOTA";
}

/// Entrada de la bitácora de Auditoría (RF-112, RF-113, RF-114).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntradaAuditoria {
    pub id: String,
    pub fecha: String,
    /// RF-113: el nombre lo escribe la persona antes de confirmar. Es sólo para
    /// trazabilidad; no se valida contra ningún registro de usuarios porque no existen.
    pub nombre_quien_realiza: String,
    pub entidad_afectada: String,
    pub campo_modificado: Option<String>,
    pub valor_anterior: Option<String>,
    pub valor_nuevo: Option<String>,
    pub motivo: Option<String>,
    pub tipo_accion: String,
}

/// Puerto de la bitácora. Vive en su propio módulo porque lo usan Configuración
/// (RF-112), Caja (RF-90), Cierre (RF-97) y Respaldo (RF-113): es transversal.
pub trait AuditoriaPort: Send + Sync {
    fn registrar(&self, banco_id: &str, entrada: &EntradaAuditoria) -> Result<(), AppError>;
    fn listar(&self, banco_id: &str) -> Result<Vec<EntradaAuditoria>, AppError>;
}
