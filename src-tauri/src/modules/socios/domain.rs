use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// RN-01: "El número de socios que integren el Bankomunal debe ser mínimo 8 y máximo 19
/// integrantes" (reglamento fijo). El mínimo no se puede exigir al registrar —un
/// Bankomunal arranca desde cero y va sumando— pero el máximo sí es un tope duro.
pub const MIN_SOCIOS: usize = 8;
pub const MAX_SOCIOS: usize = 19;

/// La planilla de registro del socio contempla hasta 2 beneficiarios del fondo de
/// protección (RF-21).
pub const MAX_PROTEGIDOS: usize = 2;

/// Estatus de un socio a lo largo de su ciclo de vida.
///
/// Sólo `Activo` se usa hoy: los tres estados de retiro los asigna el módulo de
/// Liquidación de Acciones (RF-37, RF-76), que aún no existe. Se declaran completos
/// para que el dominio no tenga que cambiar cuando ese módulo llegue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstatusSocio {
    #[serde(rename = "ACTIVO")]
    Activo,
    #[serde(rename = "RETIRADO_VOLUNTARIO")]
    RetiradoVoluntario,
    #[serde(rename = "RETIRADO_CON_DEUDA")]
    RetiradoConDeuda,
    #[serde(rename = "RETIRADO_DEUDA_SALDADA")]
    RetiradoDeudaSaldada,
}

impl EstatusSocio {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Activo => "ACTIVO",
            Self::RetiradoVoluntario => "RETIRADO_VOLUNTARIO",
            Self::RetiradoConDeuda => "RETIRADO_CON_DEUDA",
            Self::RetiradoDeudaSaldada => "RETIRADO_DEUDA_SALDADA",
        }
    }

    /// Convierte el texto guardado en SQLite. Un valor desconocido no debe reventar la
    /// aplicación: se degrada a `Activo`, que es el estado neutro.
    pub fn desde_str(valor: &str) -> Self {
        match valor {
            "RETIRADO_VOLUNTARIO" => Self::RetiradoVoluntario,
            "RETIRADO_CON_DEUDA" => Self::RetiradoConDeuda,
            "RETIRADO_DEUDA_SALDADA" => Self::RetiradoDeudaSaldada,
            _ => Self::Activo,
        }
    }

    pub fn esta_retirado(&self) -> bool {
        !matches!(self, Self::Activo)
    }
}

/// Beneficiario en caso de muerte del socio (RF-20), a quien se le ceden las acciones.
///
/// La planilla original sólo declara "cedo mis acciones a ____ identificado con cédula
/// ____", por eso el parentesco es opcional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Beneficiario {
    pub nombre: String,
    pub cedula: String,
    pub parentesco: Option<String>,
}

/// Protegido del fondo de protección (RF-21). La planilla pide nombre, cédula,
/// parentesco y teléfono.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Protegido {
    pub nombre: String,
    pub cedula: String,
    pub parentesco: String,
    pub telefono: String,
}

/// Un socio del Bankomunal con sus allegados (RF-17, RF-20, RF-21).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Socio {
    pub id: String,
    pub cedula: String,
    pub nombres: String,
    pub apellidos: String,
    pub profesion: String,
    pub direccion: String,
    pub telefono: String,
    pub celular: String,
    pub correo: String,
    pub estatus: EstatusSocio,
    pub fecha_ingreso: String,
    pub fecha_retiro: Option<String>,
    /// Deuda irrecuperable que quedó a su nombre al retirarse (RF-35).
    pub saldo_incobrable: f64,
    pub beneficiario: Option<Beneficiario>,
    pub protegidos: Vec<Protegido>,
}

impl Socio {
    pub fn nombre_completo(&self) -> String {
        format!("{} {}", self.nombres.trim(), self.apellidos.trim())
            .trim()
            .to_string()
    }

    /// Acciones que el socio tiene libres, descontando las comprometidas como garantía
    /// de un crédito vigente (RF-29, RN-06).
    ///
    /// PENDIENTE: depende del módulo de Acciones y del de Créditos, que aún no existen.
    /// No se implementa con lógica provisional a propósito: devolver 0 o el total sería
    /// una respuesta plausible pero falsa, y las Liquidaciones se calcularían mal sin
    /// que nadie lo note.
    pub fn acciones_libres(&self) -> i64 {
        todo!("Requiere el módulo de Acciones (lote_acciones) y de Créditos (garantia_credito)")
    }

    /// Total de acciones activas del socio, sumando sus lotes no liquidados (RF-24).
    ///
    /// PENDIENTE: depende del módulo de Acciones.
    pub fn acciones_activas(&self) -> i64 {
        todo!("Requiere el módulo de Acciones (lote_acciones)")
    }

    /// Si el socio tiene algún crédito sin cancelar (RF-32).
    ///
    /// PENDIENTE: depende del módulo de Créditos.
    pub fn tiene_credito_vigente(&self) -> bool {
        todo!("Requiere el módulo de Créditos (credito.estatus)")
    }
}

/// Datos que captura el formulario de socio, tanto al crear (RF-15) como al
/// actualizar (RF-19). No lleva `id` ni `estatus`: el id lo genera el sistema y el
/// estatus lo gobiernan las reglas de negocio, no el formulario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatosSocio {
    pub cedula: String,
    pub nombres: String,
    pub apellidos: String,
    #[serde(default)]
    pub profesion: String,
    #[serde(default)]
    pub direccion: String,
    #[serde(default)]
    pub telefono: String,
    #[serde(default)]
    pub celular: String,
    #[serde(default)]
    pub correo: String,
    #[serde(default)]
    pub beneficiario: Option<Beneficiario>,
    #[serde(default)]
    pub protegidos: Vec<Protegido>,
}

/// --- Puerto (hexagonal): contrato que implementa el adaptador SQLite. ---
///
/// El Dominio no sabe que existe SQLite; sólo conoce este contrato. Cada método recibe
/// el id del Banco activo porque cada Bankomunal vive en su propio archivo (RF-08).
pub trait SocioPort: Send + Sync {
    /// Inserta el socio con su beneficiario y protegidos de forma atómica.
    fn crear(&self, banco_id: &str, socio: &Socio) -> Result<(), AppError>;
    /// Reemplaza los datos del socio y sus allegados de forma atómica.
    fn actualizar(&self, banco_id: &str, socio: &Socio) -> Result<(), AppError>;
    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Socio>, AppError>;
    fn buscar_por_cedula(&self, banco_id: &str, cedula: &str) -> Result<Option<Socio>, AppError>;
    /// Todos los socios, ordenados por apellidos y nombres.
    fn listar(&self, banco_id: &str) -> Result<Vec<Socio>, AppError>;
    /// Cuántos socios cuentan para el cupo de RN-01 (los que no están retirados).
    fn contar_activos(&self, banco_id: &str) -> Result<usize, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estatus_ida_y_vuelta_a_texto() {
        for estatus in [
            EstatusSocio::Activo,
            EstatusSocio::RetiradoVoluntario,
            EstatusSocio::RetiradoConDeuda,
            EstatusSocio::RetiradoDeudaSaldada,
        ] {
            assert_eq!(EstatusSocio::desde_str(estatus.as_str()), estatus);
        }
    }

    #[test]
    fn estatus_desconocido_no_revienta_y_cae_en_activo() {
        assert_eq!(EstatusSocio::desde_str("BASURA"), EstatusSocio::Activo);
        assert_eq!(EstatusSocio::desde_str(""), EstatusSocio::Activo);
    }

    #[test]
    fn solo_activo_cuenta_como_no_retirado() {
        assert!(!EstatusSocio::Activo.esta_retirado());
        assert!(EstatusSocio::RetiradoVoluntario.esta_retirado());
        assert!(EstatusSocio::RetiradoConDeuda.esta_retirado());
        assert!(EstatusSocio::RetiradoDeudaSaldada.esta_retirado());
    }
}
