use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// RN-14: todo crédito debe tener al menos un fiador que sea socio del Bankomunal.
pub const MIN_FIADORES: usize = 1;
/// RF-48: se registran hasta 2 fiadores por solicitud.
pub const MAX_FIADORES: usize = 2;

/// Frecuencia de pago de los créditos. El sistema en papel trabaja con pagos
/// mensuales, que es el único valor con sentido aquí (RF-60).
pub const FRECUENCIA_PAGO_MENSUAL: &str = "MENSUAL";

/// Catálogo de clases de crédito (RN-11), según el catálogo oficial
/// "Clasificación de créditos BK sistema" (D-15 b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestinoCredito {
    #[serde(rename = "AH")]
    ArticulosHogar,
    #[serde(rename = "ARE")]
    EquiposTrabajo,
    #[serde(rename = "CVR")]
    Vivienda,
    #[serde(rename = "CV")]
    CompraVenta,
    #[serde(rename = "ED")]
    Educacion,
    #[serde(rename = "GP")]
    GastosPersonales,
    #[serde(rename = "OT")]
    Otros,
    #[serde(rename = "PR")]
    Productivo,
    #[serde(rename = "SL")]
    Salud,
    #[serde(rename = "SP")]
    ServiciosPublicos,
}

impl DestinoCredito {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ArticulosHogar => "AH",
            Self::EquiposTrabajo => "ARE",
            Self::Vivienda => "CVR",
            Self::CompraVenta => "CV",
            Self::Educacion => "ED",
            Self::GastosPersonales => "GP",
            Self::Otros => "OT",
            Self::Productivo => "PR",
            Self::Salud => "SL",
            Self::ServiciosPublicos => "SP",
        }
    }

    pub fn desde_str(valor: &str) -> Option<Self> {
        Some(match valor {
            "AH" => Self::ArticulosHogar,
            "ARE" => Self::EquiposTrabajo,
            "CVR" => Self::Vivienda,
            "CV" => Self::CompraVenta,
            "ED" => Self::Educacion,
            "GP" => Self::GastosPersonales,
            "OT" => Self::Otros,
            "PR" => Self::Productivo,
            "SL" => Self::Salud,
            "SP" => Self::ServiciosPublicos,
            _ => return None,
        })
    }
}

/// Decisión de la Junta sobre una solicitud (RF-50).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoSolicitud {
    #[serde(rename = "PENDIENTE")]
    Pendiente,
    #[serde(rename = "APROBADA")]
    Aprobada,
    #[serde(rename = "MODIFICADA")]
    Modificada,
    #[serde(rename = "NEGADA")]
    Negada,
    #[serde(rename = "DIFERIDA")]
    Diferida,
}

impl EstadoSolicitud {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pendiente => "PENDIENTE",
            Self::Aprobada => "APROBADA",
            Self::Modificada => "MODIFICADA",
            Self::Negada => "NEGADA",
            Self::Diferida => "DIFERIDA",
        }
    }

    pub fn desde_str(valor: &str) -> Self {
        match valor {
            "APROBADA" => Self::Aprobada,
            "MODIFICADA" => Self::Modificada,
            "NEGADA" => Self::Negada,
            "DIFERIDA" => Self::Diferida,
            _ => Self::Pendiente,
        }
    }
}

/// Ciclo de vida de un crédito ya desembolsado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EstadoCredito {
    #[serde(rename = "VIGENTE")]
    Vigente,
    #[serde(rename = "PAGADO")]
    Pagado,
    #[serde(rename = "REFINANCIADO")]
    Refinanciado,
}

impl EstadoCredito {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Vigente => "VIGENTE",
            Self::Pagado => "PAGADO",
            Self::Refinanciado => "REFINANCIADO",
        }
    }

    pub fn desde_str(valor: &str) -> Self {
        match valor {
            "PAGADO" => Self::Pagado,
            "REFINANCIADO" => Self::Refinanciado,
            _ => Self::Vigente,
        }
    }
}

/// Rol de un socio dentro de la garantía de un crédito (RF-48, RN-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RolGarantia {
    #[serde(rename = "TITULAR")]
    Titular,
    #[serde(rename = "FIADOR")]
    Fiador,
}

impl RolGarantia {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Titular => "TITULAR",
            Self::Fiador => "FIADOR",
        }
    }

    pub fn desde_str(valor: &str) -> Self {
        match valor {
            "FIADOR" => Self::Fiador,
            _ => Self::Titular,
        }
    }
}

/// Fiador propuesto en una solicitud (RF-48): se identifica por cédula y declara
/// cuántas acciones compromete en garantía.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiadorSolicitud {
    pub cedula: String,
    pub acciones_comprometidas: f64,
}

/// Datos que captura la pantalla "Nuevo Crédito" (RF-43, RF-45, RF-47, RF-48).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevaSolicitud {
    pub socio_id: String,
    pub monto_solicitado: f64,
    pub plazo_cuotas: i64,
    pub destino: DestinoCredito,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    #[serde(default)]
    pub fiadores: Vec<FiadorSolicitud>,
}

/// Decisión de la Junta sobre una solicitud (RF-50, RF-51).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSolicitud {
    pub solicitud_id: String,
    pub decision: EstadoSolicitud,
    #[serde(default)]
    pub monto_aprobado: Option<f64>,
    #[serde(default)]
    pub observacion: Option<String>,
    #[serde(default)]
    pub decidida_por: String,
}

/// Datos para desembolsar un crédito (RF-44, RF-54): bien desde una solicitud
/// aprobada, bien cargados a mano sin solicitud previa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevoDesembolso {
    #[serde(default)]
    pub solicitud_id: Option<String>,
    pub socio_id: String,
    pub monto: f64,
    pub plazo_cuotas: i64,
    pub destino: DestinoCredito,
    #[serde(default)]
    pub fiadores: Vec<FiadorSolicitud>,
    #[serde(default)]
    pub fecha: Option<String>,
}

/// Garantía de una solicitud (socio titular o fiador, RF-48).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GarantiaSolicitud {
    pub id: String,
    pub solicitud_id: String,
    pub socio_id: String,
    pub rol: RolGarantia,
    pub acciones_comprometidas: f64,
}

/// Una solicitud de crédito con su estado y sus garantías (RF-43 a RF-52).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolicitudCredito {
    pub id: String,
    pub socio_id: String,
    pub fecha_solicitud: String,
    pub monto_solicitado: f64,
    pub plazo_cuotas: i64,
    pub destino: DestinoCredito,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    /// RF-45: Total Ingresos − Total Egresos, calculado por el sistema.
    pub capacidad_pago: f64,
    pub estado: EstadoSolicitud,
    pub monto_aprobado: Option<f64>,
    pub observacion: Option<String>,
    pub fecha_decision: Option<String>,
    pub decidida_por: Option<String>,
    pub garantias: Vec<GarantiaSolicitud>,
}

/// Garantía efectiva de un crédito desembolsado (RF-57, RN-04).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GarantiaCredito {
    pub id: String,
    pub credito_id: String,
    pub socio_id: String,
    pub rol: RolGarantia,
    pub acciones_comprometidas: f64,
}

/// Un crédito desembolsado (RF-53 a RF-62).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Credito {
    pub id: String,
    pub socio_id: String,
    /// RF-53: número secuencial asignado automáticamente.
    pub numero: String,
    pub monto_original: f64,
    pub tasa: f64,
    pub plazo_cuotas: i64,
    pub cuota_actual: i64,
    pub saldo_pendiente: f64,
    pub destino: DestinoCredito,
    pub estatus: EstadoCredito,
    pub fecha_solicitud: String,
    pub fecha_desembolso: String,
    pub frecuencia_pago: String,
    pub fecha_vencimiento: String,
    /// RF-54/D-07: si el desembolso proviene de una solicitud aprobada.
    pub solicitud_id: Option<String>,
    pub garantias: Vec<GarantiaCredito>,
}

/// Cuota calculada con el modelo de saldo decreciente (RF-46, RF-61, RN-12).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuotaPlaneada {
    pub numero: i64,
    pub fecha_vencimiento: String,
    pub capital: f64,
    pub interes: f64,
    pub valor_total: f64,
}

/// Vista previa de un desembolso (RF-46): la tabla calculada y los totales que la
/// pantalla muestra para confirmar antes de asentar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TablaCredito {
    pub cuotas: Vec<CuotaPlaneada>,
    /// Suma de capital + intereses de toda la tabla.
    pub monto_total: f64,
    pub cuota_mensual: f64,
    pub capital_cuota: f64,
    pub interes_cuota: f64,
}

/// Tabla de pagos con saldo decreciente (RN-12): interés de cada cuota = Tasa ×
/// saldo pendiente antes de esa cuota; capital fijo = Monto ÷ Nº de cuotas.
///
/// Aislada en el Dominio, como `calcular_ppcfc`, para que corregir el modelo no toque
/// nada más; la usan Solicitud (RF-46), Desembolso (RF-61) y, más adelante,
/// Refinanciamiento (RF-82).
pub fn calcular_tabla(
    monto: f64,
    tasa_mensual_pct: f64,
    plazo: i64,
    fecha_primer_vencimiento: &str,
) -> Vec<CuotaPlaneada> {
    if monto <= 0.0 || plazo <= 0 {
        return Vec::new();
    }
    let capital_fijo = redondear(monto / plazo as f64);
    let mut saldo = monto;
    let mut cuotas = Vec::with_capacity(plazo as usize);

    for i in 1..=plazo {
        let interes = redondear(saldo * tasa_mensual_pct / 100.0);
        // La última cuota absorbe el redondeo: el capital debe sumar exactamente el monto.
        let capital = if i == plazo {
            redondear(saldo)
        } else {
            capital_fijo.min(saldo)
        };
        let valor_total = redondear(capital + interes);
        let fecha = sumar_meses(fecha_primer_vencimiento, i - 1);
        cuotas.push(CuotaPlaneada {
            numero: i,
            fecha_vencimiento: fecha,
            capital,
            interes,
            valor_total,
        });
        saldo = redondear(saldo - capital);
    }

    cuotas
}

/// Resumen de una tabla calculada (RF-46).
pub fn resumir_tabla(cuotas: &[CuotaPlaneada], _plazo: i64) -> TablaCredito {
    let monto_total = redondear(cuotas.iter().map(|c| c.valor_total).sum());
    let cuota_mensual = cuotas.first().map(|c| c.valor_total).unwrap_or(0.0);
    let capital_cuota = cuotas.first().map(|c| c.capital).unwrap_or(0.0);
    let interes_cuota = cuotas.first().map(|c| c.interes).unwrap_or(0.0);
    TablaCredito {
        cuotas: cuotas.to_vec(),
        monto_total,
        cuota_mensual,
        capital_cuota,
        interes_cuota,
    }
}

/// Suma `n` meses a una fecha ISO (`YYYY-MM-DD`), recortando el día al máximo del mes
/// destino para no producir fechas inválidas. Suficiente para el vencimiento mensual.
fn sumar_meses(fecha: &str, n: i64) -> String {
    let anio: i64 = fecha.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mes: i64 = fecha.get(5..7).and_then(|s| s.parse().ok()).unwrap_or(1);
    let dia: i64 = fecha.get(8..10).and_then(|s| s.parse().ok()).unwrap_or(1);

    let total = anio * 12 + (mes - 1) + n;
    let nuevo_anio = total.div_euclid(12);
    let nuevo_mes = total.rem_euclid(12) + 1;
    let max_dia = dias_del_mes(nuevo_anio, nuevo_mes);
    let nuevo_dia = dia.min(max_dia);
    format!("{nuevo_anio:04}-{nuevo_mes:02}-{nuevo_dia:02}")
}

fn dias_del_mes(anio: i64, mes: i64) -> i64 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (anio % 4 == 0 && anio % 100 != 0) || anio % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Redondea a 2 decimales para evitar ruido de coma flotante en el dinero.
fn redondear(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// --- Puertos (hexagonal) ---

pub trait SolicitudPort: Send + Sync {
    fn crear(
        &self,
        banco_id: &str,
        solicitud: &SolicitudCredito,
        garantias: &[GarantiaSolicitud],
    ) -> Result<(), AppError>;
    /// Actualiza el estado/decisiones de la solicitud y sus garantías.
    fn actualizar(
        &self,
        banco_id: &str,
        solicitud: &SolicitudCredito,
        garantias: &[GarantiaSolicitud],
    ) -> Result<(), AppError>;
    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<SolicitudCredito>, AppError>;
    fn listar_por_estado(
        &self,
        banco_id: &str,
        estado: Option<EstadoSolicitud>,
    ) -> Result<Vec<SolicitudCredito>, AppError>;
}

pub trait CreditoPort: Send + Sync {
    /// Inserta el crédito con sus cuotas y garantías de forma atómica (RF-62).
    fn crear(
        &self,
        banco_id: &str,
        credito: &Credito,
        cuotas: &[CuotaPlaneada],
        garantias: &[GarantiaCredito],
    ) -> Result<(), AppError>;
    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Credito>, AppError>;
    fn buscar_por_solicitud(
        &self,
        banco_id: &str,
        solicitud_id: &str,
    ) -> Result<Option<Credito>, AppError>;
    fn listar(&self, banco_id: &str) -> Result<Vec<Credito>, AppError>;
    /// RF-53: siguiente número de crédito en secuencia.
    fn siguiente_numero(&self, banco_id: &str) -> Result<String, AppError>;
    /// Pares (titular, fiador) de créditos vigentes, para validar RN-05.
    fn pares_titular_fiador(&self, banco_id: &str) -> Result<Vec<(String, String)>, AppError>;
    /// RN-03: número de créditos VIGENTES de un socio.
    fn contar_vigentes(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError>;
}

/// Parámetros del Bankomunal que necesita este módulo (RF-55, RN-03/04, RF-59).
#[derive(Debug, Clone, PartialEq)]
pub struct ParametrosCredito {
    pub monto_maximo_credito: f64,
    pub tasa_interes_ordinario: f64,
    pub plazo_maximo_cuotas: i64,
    pub pct_garantia_socio: f64,
    pub pct_garantia_fiador: f64,
    pub valor_nominal: f64,
}

pub trait ParametrosCreditoPort: Send + Sync {
    fn obtener(&self, banco_id: &str) -> Result<ParametrosCredito, AppError>;
}

/// Acciones vigentes de un socio, para RN-03 y RN-04 (RF-56, RF-57).
pub trait AccionesParaCreditoPort: Send + Sync {
    fn acciones_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError>;
}

/// Acceso mínimo a socios para validar fiadores (RF-49, RN-14).
pub trait SociosParaCreditoPort: Send + Sync {
    fn buscar_por_cedula(&self, banco_id: &str, cedula: &str) -> Result<Option<String>, AppError>;
}

/// Asiento en el Libro de Ingresos y Egresos que produce un desembolso (RF-62).
pub trait LibroContablePort: Send + Sync {
    fn registrar_desembolso(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RN-12: el interés de cada cuota se calcula sobre el saldo pendiente antes de esa
    /// cuota (decreciente), y el capital fijo = Monto ÷ Nº de cuotas.
    #[test]
    fn la_tabla_usa_saldo_decreciente() {
        // Monto 1.000.000 a 3% mensual a 12 cuotas.
        let cuotas = calcular_tabla(1_000_000.0, 3.0, 12, "2026-09-01");
        assert_eq!(cuotas.len(), 12);

        // Capital fijo = 1.000.000 / 12 (redondeado).
        assert_eq!(cuotas[0].capital, 83_333.33);
        // Interés de la cuota 1 sobre el monto total.
        assert_eq!(cuotas[0].interes, 30_000.00);
        // La cuota 2 ya interesa sobre saldo menor: 1.000.000 − 83.333,33.
        assert_eq!(cuotas[1].interes, 27_500.00);

        // El interés decrece cuota a cuota.
        for w in cuotas.windows(2) {
            assert!(
                w[1].interes < w[0].interes,
                "el interés debe decrecer: cuota {} vs {}",
                w[1].numero,
                w[0].numero
            );
        }
    }

    /// El capital de todas las cuotas debe sumar exactamente el monto original.
    #[test]
    fn el_capital_suma_exactamente_el_monto() {
        let monto = 999_999.99;
        let cuotas = calcular_tabla(monto, 3.0, 12, "2026-09-01");
        let total_capital: f64 = cuotas.iter().map(|c| c.capital).sum();
        assert!((total_capital - monto).abs() < 0.01);
    }

    /// La última cuota absorbe el redondeo: puede diferir levemente del capital fijo.
    #[test]
    fn la_ultima_cuota_absorbe_el_redondeo() {
        let cuotas = calcular_tabla(100_000.0, 3.0, 3, "2026-09-01");
        let resto = 100_000.0 - 2.0 * 33_333.33;
        assert!((cuotas[2].capital - resto).abs() < 0.01);
    }

    /// Los vencimientos avanzan de mes en mes.
    #[test]
    fn los_vencimientos_avanzan_mes_a_mes() {
        let cuotas = calcular_tabla(100_000.0, 3.0, 3, "2026-09-15");
        assert_eq!(cuotas[0].fecha_vencimiento, "2026-09-15");
        assert_eq!(cuotas[1].fecha_vencimiento, "2026-10-15");
        assert_eq!(cuotas[2].fecha_vencimiento, "2026-11-15");
    }

    #[test]
    fn la_tabla_vacia_no_revienta() {
        assert!(calcular_tabla(0.0, 3.0, 12, "2026-09-01").is_empty());
        assert!(calcular_tabla(1000.0, 3.0, 0, "2026-09-01").is_empty());
    }

    #[test]
    fn el_resumen_agrega_el_total() {
        let cuotas = calcular_tabla(100_000.0, 3.0, 3, "2026-09-01");
        let resumen = resumir_tabla(&cuotas, 3);
        assert_eq!(resumen.cuota_mensual, cuotas[0].valor_total);
        assert!((resumen.monto_total - cuotas.iter().map(|c| c.valor_total).sum::<f64>()).abs() < 0.01);
    }

    #[test]
    fn los_enums_van_y_vuelven_a_texto() {
        for destino in [
            DestinoCredito::ArticulosHogar,
            DestinoCredito::EquiposTrabajo,
            DestinoCredito::Vivienda,
            DestinoCredito::CompraVenta,
            DestinoCredito::Educacion,
            DestinoCredito::GastosPersonales,
            DestinoCredito::Otros,
            DestinoCredito::Productivo,
            DestinoCredito::Salud,
            DestinoCredito::ServiciosPublicos,
        ] {
            assert_eq!(DestinoCredito::desde_str(destino.as_str()), Some(destino));
        }
        assert_eq!(DestinoCredito::desde_str("XXX"), None);
        assert_eq!(EstadoSolicitud::desde_str(EstadoSolicitud::Diferida.as_str()), EstadoSolicitud::Diferida);
        assert_eq!(EstadoCredito::desde_str(EstadoCredito::Pagado.as_str()), EstadoCredito::Pagado);
        assert_eq!(RolGarantia::desde_str(RolGarantia::Fiador.as_str()), RolGarantia::Fiador);
    }

    /// Un destino desconocido no debe admitirse (RF-47, RN-11).
    #[test]
    fn el_destino_debe_ser_del_catalogo() {
        assert!(DestinoCredito::desde_str("CRV").is_none(), "CRV era una errata de CVR (D-15)");
        assert!(DestinoCredito::desde_str("SP").is_some());
    }
}
