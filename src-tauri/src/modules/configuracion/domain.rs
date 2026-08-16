use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Umbrales del PPCFC (RN-09). Vienen del **reglamento fijo** de la metodología
/// Bankomunales —"Si E es menor que 80% no se venderán acciones; entre 80% y 90%
/// hasta un 10%; entre 90% y 100% hasta un 15%"— así que no los configura cada
/// Bankomunal: son constantes de dominio. Lo configurable son los % a vender.
pub const PPCFC_UMBRAL_MINIMO: f64 = 80.0;
pub const PPCFC_UMBRAL_MEDIO: f64 = 90.0;

/// Retención mínima de cada fondo, según el reglamento fijo: "Se apartará un fondo de
/// gastos, no menor al 5% de las ganancias totales de cada mes" (ídem incobrables).
pub const RETENCION_MINIMA_FONDO_PCT: f64 = 5.0;

/// Parámetros editables del Bankomunal (RF-11, RF-12, RF-13, RN-04/07/08/09/13).
/// Son parte de la Regla de Negocio: el dominio los valida y expone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Configuracion {
    /// Valor nominal de la acción (RN-13).
    pub valor_nominal: f64,
    /// % de garantía exigido al socio titular (RN-04, RF-12).
    pub pct_garantia_socio: f64,
    /// % de garantía exigido al fiador (RN-04, RF-12).
    pub pct_garantia_fiador: f64,
    // ── Los dos fondos del Bankomunal (D-11). Cada uno retiene su % de las ganancias
    // del mes en el Cierre; sólo el de Incobrables tiene además un tope de crecimiento.
    /// % de las ganancias mensuales retenido al Fondo para Gastos (RN-07, RF-13).
    /// Cubre los gastos operativos del Bankomunal —papelería, transporte, fotocopias—
    /// y se consume con los Gastos del Bankomunal (RF-86).
    pub pct_fondo_gastos: f64,
    /// % de las ganancias mensuales retenido al Fondo de Reserva para Incobrables
    /// (RN-08, RF-13). Es el colchón de seguridad para cuando un socio se retira
    /// debiendo más de lo que valen sus acciones (RF-36).
    pub pct_fondo_incobrables: f64,
    /// Tope hasta el que puede crecer el Fondo de Reserva para Incobrables acumulado,
    /// como % del capital total en acciones (RN-08, por defecto 20%). Al alcanzarlo el
    /// fondo deja de crecer: ya es colchón suficiente. Es otra magnitud distinta de
    /// `pct_fondo_incobrables` —uno dice cuánto se aparta cada mes, este hasta dónde
    /// se sigue apartando (D-04).
    pub tope_reserva_incobrables_pct: f64,
    /// % del total de acciones autorizado a vender cuando el PPCFC cae en el tramo
    /// bajo (entre `PPCFC_UMBRAL_MINIMO` y `PPCFC_UMBRAL_MEDIO`). Por defecto 10%.
    pub ppcfc_venta_rango1_pct: f64,
    /// % del total de acciones autorizado a vender cuando el PPCFC cae en el tramo
    /// alto (entre `PPCFC_UMBRAL_MEDIO` y 100%). Por defecto 15%.
    pub ppcfc_venta_rango2_pct: f64,
    /// RN-15: tope que un solo socio puede comprar en un mismo mes, como % del cupo que
    /// el PPCFC autorizó vender ese mes (RN-09) — no del capital total del Bankomunal.
    /// Por defecto 20%.
    ///
    /// No confundir con RN-02, que limita al 15% la participación *acumulada* de un
    /// socio sobre el total de acciones. Son dos topes distintos y ambos aplican: RN-15
    /// mira el cupo del mes, RN-02 mira el histórico.
    pub tope_individual_mensual_pct: f64,
    /// Condiciones de los créditos (RF-11).
    pub plazo_maximo_cuotas: i64,
    pub tasa_interes_ordinario: f64,
    pub tasa_interes_mora: f64,
    pub monto_maximo_credito: f64,
}

impl Default for Configuracion {
    fn default() -> Self {
        Self {
            // El reglamento fijo recomienda explícitamente $10.000 por acción (RN-13).
            // Es sólo el valor inicial: cada Bankomunal lo configura a su medida.
            valor_nominal: 10000.0,
            pct_garantia_socio: 20.0,
            pct_garantia_fiador: 20.0,
            pct_fondo_gastos: 10.0,
            pct_fondo_incobrables: 10.0,
            tope_reserva_incobrables_pct: 20.0,
            ppcfc_venta_rango1_pct: 10.0,
            ppcfc_venta_rango2_pct: 15.0,
            tope_individual_mensual_pct: 20.0,
            plazo_maximo_cuotas: 36,         // 36 meses
            tasa_interes_ordinario: 3.0,     // 3%
            tasa_interes_mora: 5.0,          // 5%
            monto_maximo_credito: 5000000.0, // 5 millones
        }
    }
}

/// Datos Generales de solo consulta (RF-09, RF-10, RF-11) + contadores automáticos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatosGenerales {
    pub id: String,
    pub nombre: String,
    pub ubicacion: String,
    pub fecha_creacion: String,
    pub moneda: String,
    pub valor_nominal: f64,
    /// Contadores automáticos (RF-10).
    pub numero_creditos_otorgados: i64,
    pub monto_total_creditos: f64,
    pub numero_acciones_vendidas: i64,
    /// Saldos acumulados de los dos fondos (RF-11, D-11).
    pub saldo_fondo_gastos: f64,
    pub saldo_fondo_incobrables: f64,
}

/// Puertos (hexagonal) del módulo de Configuración.
///
/// Operan sobre el `.db` del Bankomunal "activo" de la sesión. El adaptador recibe
/// el id del Banco en cada llamada para mantener consistencia con el aislamiento por archivo (RF-08).
pub trait ConfiguracionPort: Send + Sync {
    /// Devuelve (o crea con valores por defecto) la config del Banco.
    fn obtener(&self, banco_id: &str) -> Result<Configuracion, AppError>;
    fn actualizar(&self, banco_id: &str, config: &Configuracion) -> Result<(), AppError>;
    fn obtener_datos_generales(&self, banco_id: &str) -> Result<DatosGenerales, AppError>;
}

