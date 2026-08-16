use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Códigos de operación del Libro de Ingresos y Egresos.
///
/// Son los mismos que usa el sistema en papel y que dan nombre a cada módulo del
/// Documento de Entendimiento. Se declara el catálogo completo aunque hoy sólo se
/// registren los cuatro de Caja: cuando lleguen Acciones y Créditos, esos módulos
/// escribirán en el mismo Libro con su propio código, sin tocar este enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodigoOperacion {
    // ── Los que registra el módulo de Caja (RF-83 a RF-87) ──
    /// Otros Ingresos (RF-83).
    #[serde(rename = "OI")]
    OtroIngreso,
    /// Otros Egresos (RF-84).
    #[serde(rename = "EG")]
    OtroEgreso,
    /// Ingreso al Fondo para Gastos (RF-85). También es el destino de las donaciones.
    #[serde(rename = "IFG")]
    IngresoFondoGastos,
    /// Gastos del Bankomunal (RF-86), pagados con el Fondo para Gastos.
    #[serde(rename = "GBK")]
    GastoBankomunal,

    // ── Los que registrarán otros módulos ──
    /// Venta de Certificados/Acciones (RF-22).
    #[serde(rename = "VC")]
    VentaAcciones,
    /// Liquidación de Acciones (RF-28).
    #[serde(rename = "LC")]
    LiquidacionAcciones,
    /// Ganancias Repartidas (RF-42).
    #[serde(rename = "UR")]
    GananciaRepartida,
    /// Desembolso de Crédito (RF-62).
    #[serde(rename = "CON")]
    DesembolsoCredito,
    /// Intereses Ordinarios (RF-65).
    #[serde(rename = "OR")]
    InteresOrdinario,
    /// Pago de Cuota (RF-69).
    #[serde(rename = "PC")]
    PagoCuota,
    /// Intereses de Mora (RF-72).
    #[serde(rename = "MO")]
    InteresMora,
    /// Pago de Deuda Pendiente (RF-74).
    #[serde(rename = "PDP")]
    PagoDeudaPendiente,
    /// Refinanciamiento (RF-79).
    #[serde(rename = "COR")]
    Refinanciamiento,
}

impl CodigoOperacion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OtroIngreso => "OI",
            Self::OtroEgreso => "EG",
            Self::IngresoFondoGastos => "IFG",
            Self::GastoBankomunal => "GBK",
            Self::VentaAcciones => "VC",
            Self::LiquidacionAcciones => "LC",
            Self::GananciaRepartida => "UR",
            Self::DesembolsoCredito => "CON",
            Self::InteresOrdinario => "OR",
            Self::PagoCuota => "PC",
            Self::InteresMora => "MO",
            Self::PagoDeudaPendiente => "PDP",
            Self::Refinanciamiento => "COR",
        }
    }

    pub fn desde_str(valor: &str) -> Option<Self> {
        Some(match valor {
            "OI" => Self::OtroIngreso,
            "EG" => Self::OtroEgreso,
            "IFG" => Self::IngresoFondoGastos,
            "GBK" => Self::GastoBankomunal,
            "VC" => Self::VentaAcciones,
            "LC" => Self::LiquidacionAcciones,
            "UR" => Self::GananciaRepartida,
            "CON" => Self::DesembolsoCredito,
            "OR" => Self::InteresOrdinario,
            "PC" => Self::PagoCuota,
            "MO" => Self::InteresMora,
            "PDP" => Self::PagoDeudaPendiente,
            "COR" => Self::Refinanciamiento,
            _ => return None,
        })
    }

    /// Si el movimiento entra dinero a la caja (ingreso) o sale (egreso).
    pub fn es_ingreso(&self) -> bool {
        matches!(
            self,
            Self::OtroIngreso
                | Self::IngresoFondoGastos
                | Self::VentaAcciones
                | Self::InteresOrdinario
                | Self::PagoCuota
                | Self::InteresMora
                | Self::PagoDeudaPendiente
        )
    }

    /// Códigos que puede registrar directamente el usuario desde la pantalla de Caja.
    /// Los demás son consecuencia de una operación de otro módulo y no se teclean aquí.
    pub fn registrable_en_caja(&self) -> bool {
        matches!(
            self,
            Self::OtroIngreso | Self::OtroEgreso | Self::IngresoFondoGastos | Self::GastoBankomunal
        )
    }

    /// Si el movimiento mueve el saldo acumulado del Fondo para Gastos (RF-85, RF-86).
    /// Devuelve el signo con que lo afecta: +1 suma, -1 resta, 0 no lo toca.
    pub fn efecto_en_fondo_gastos(&self) -> f64 {
        match self {
            Self::IngresoFondoGastos => 1.0,
            Self::GastoBankomunal => -1.0,
            _ => 0.0,
        }
    }
}

/// Un asiento del Libro de Ingresos y Egresos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movimiento {
    pub id: String,
    /// Consecutivo dentro del Bankomunal, como en el libro de papel.
    pub numero: i64,
    pub fecha: String,
    pub codigo: CodigoOperacion,
    pub descripcion: String,
    pub ingreso: f64,
    pub egreso: f64,
    /// Saldo de caja después de este asiento. Es un valor derivado: lo recalcula el
    /// sistema en cada escritura, nunca lo escribe el usuario.
    pub saldo: f64,
    pub socio_id: Option<String>,
    pub credito_id: Option<String>,
    /// Cierre al que pertenece el asiento. `None` = el mes sigue abierto (RF-89).
    pub cierre_mes_id: Option<String>,
    /// RF-90: marca de que el asiento fue corregido después de cerrado su mes.
    pub corregido: bool,
    pub corregido_por: Option<String>,
    pub fecha_correccion: Option<String>,
    pub motivo_correccion: Option<String>,
}

impl Movimiento {
    /// Si el asiento pertenece a un mes ya cerrado, corregirlo exige nombre y motivo
    /// y deja registro en Auditoría (RF-90, RF-97).
    pub fn mes_cerrado(&self) -> bool {
        self.cierre_mes_id.is_some()
    }
}

/// Datos que captura el formulario de una operación de Caja (RF-83 a RF-86).
///
/// No lleva `numero`, `saldo` ni `ingreso`/`egreso` por separado: el consecutivo y el
/// saldo los calcula el sistema, y de qué lado del libro va el monto lo decide el
/// código de operación, no el usuario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevaOperacion {
    pub codigo: CodigoOperacion,
    pub fecha: String,
    pub monto: f64,
    #[serde(default)]
    pub descripcion: String,
}

/// Bien adquirido, propio o en comodato (RF-88).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bien {
    pub id: String,
    pub descripcion: String,
    pub fecha_adquisicion: String,
    pub valor: f64,
    pub tipo: TipoBien,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoBien {
    #[serde(rename = "PROPIO")]
    Propio,
    #[serde(rename = "COMODATO")]
    Comodato,
}

impl TipoBien {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Propio => "PROPIO",
            Self::Comodato => "COMODATO",
        }
    }

    pub fn desde_str(valor: &str) -> Self {
        match valor {
            "COMODATO" => Self::Comodato,
            _ => Self::Propio,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoBien {
    pub descripcion: String,
    pub fecha_adquisicion: String,
    pub valor: f64,
    pub tipo: TipoBien,
}

/// Filtro del Libro por rango de fechas (insumo de RF-104).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FiltroLibro {
    #[serde(default)]
    pub desde: Option<String>,
    #[serde(default)]
    pub hasta: Option<String>,
}

/// --- Puertos (hexagonal) ---

pub trait LibroPort: Send + Sync {
    fn registrar(&self, banco_id: &str, mov: &Movimiento) -> Result<(), AppError>;
    fn actualizar(&self, banco_id: &str, mov: &Movimiento) -> Result<(), AppError>;
    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Movimiento>, AppError>;
    fn listar(&self, banco_id: &str, filtro: &FiltroLibro) -> Result<Vec<Movimiento>, AppError>;
    /// Siguiente consecutivo del Libro para este Bankomunal.
    fn siguiente_numero(&self, banco_id: &str) -> Result<i64, AppError>;
    /// Recalcula la columna `saldo` de todo el Libro en orden cronológico.
    /// Se ejecuta tras cada escritura: un asiento con fecha anterior o una corrección
    /// desplazan todos los saldos siguientes.
    fn recalcular_saldos(&self, banco_id: &str) -> Result<(), AppError>;
}

/// Saldo acumulado del Fondo para Gastos (RF-85, RF-86, RF-11).
///
/// Vive en la tabla de configuración, pero Caja no depende del módulo de Configuración:
/// declara aquí lo que necesita y el adaptador lo resuelve (inversión de dependencias).
pub trait FondoGastosPort: Send + Sync {
    fn saldo(&self, banco_id: &str) -> Result<f64, AppError>;
    fn ajustar(&self, banco_id: &str, delta: f64) -> Result<(), AppError>;
}

pub trait BienPort: Send + Sync {
    fn registrar(&self, banco_id: &str, bien: &Bien) -> Result<(), AppError>;
    fn listar(&self, banco_id: &str) -> Result<Vec<Bien>, AppError>;
    /// Valor total del activo fijo, que alimenta la columna `bienes` del Balance del Mes.
    fn valor_total(&self, banco_id: &str) -> Result<f64, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_codigos_van_y_vuelven_a_texto() {
        for c in [
            CodigoOperacion::OtroIngreso,
            CodigoOperacion::OtroEgreso,
            CodigoOperacion::IngresoFondoGastos,
            CodigoOperacion::GastoBankomunal,
            CodigoOperacion::VentaAcciones,
            CodigoOperacion::DesembolsoCredito,
            CodigoOperacion::Refinanciamiento,
        ] {
            assert_eq!(CodigoOperacion::desde_str(c.as_str()), Some(c));
        }
        assert_eq!(CodigoOperacion::desde_str("XXX"), None);
    }

    #[test]
    fn solo_las_cuatro_operaciones_de_caja_se_registran_a_mano() {
        assert!(CodigoOperacion::OtroIngreso.registrable_en_caja());
        assert!(CodigoOperacion::GastoBankomunal.registrable_en_caja());
        // Éstas son consecuencia de otro módulo, no se teclean en Caja.
        assert!(!CodigoOperacion::VentaAcciones.registrable_en_caja());
        assert!(!CodigoOperacion::PagoCuota.registrable_en_caja());
    }

    #[test]
    fn solo_ifg_y_gbk_mueven_el_fondo_de_gastos() {
        assert_eq!(CodigoOperacion::IngresoFondoGastos.efecto_en_fondo_gastos(), 1.0);
        assert_eq!(CodigoOperacion::GastoBankomunal.efecto_en_fondo_gastos(), -1.0);
        assert_eq!(CodigoOperacion::OtroIngreso.efecto_en_fondo_gastos(), 0.0);
        assert_eq!(CodigoOperacion::OtroEgreso.efecto_en_fondo_gastos(), 0.0);
    }

    #[test]
    fn el_codigo_decide_de_que_lado_del_libro_va_el_monto() {
        assert!(CodigoOperacion::OtroIngreso.es_ingreso());
        assert!(CodigoOperacion::PagoCuota.es_ingreso());
        assert!(!CodigoOperacion::OtroEgreso.es_ingreso());
        assert!(!CodigoOperacion::GastoBankomunal.es_ingreso());
        assert!(!CodigoOperacion::DesembolsoCredito.es_ingreso());
    }
}
