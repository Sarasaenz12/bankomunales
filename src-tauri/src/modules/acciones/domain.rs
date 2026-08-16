use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// RN-02: ningún socio puede poseer más del 15% del total de acciones del Bankomunal.
pub const TOPE_PARTICIPACION_PCT: f64 = 15.0;

/// RN-02, segunda parte: "Esta regla aplica a partir del tercer mes de iniciadas las
/// operaciones". Antes es aritméticamente inalcanzable —con 8 socios cada uno pasa del
/// 15%— y sin la excepción sería imposible arrancar un Bankomunal.
pub const MESES_GRACIA_TOPE_PARTICIPACION: i64 = 3;

/// Un lote de acciones compradas por un socio en un mes (RF-22, RF-27).
///
/// Se guarda por lote y no como un saldo único porque RN-10 reparte las ganancias al
/// año de *cada* compra: enero cobra en enero, febrero en febrero. El mes de compra es
/// parte de la identidad del lote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoteAcciones {
    pub id: String,
    pub socio_id: String,
    /// Primer día del mes de compra (YYYY-MM-01): el insumo del aniversario (RN-10).
    pub mes_compra: String,
    pub fecha_compra: String,
    pub cantidad: i64,
    /// Valor nominal vigente el día de la compra. Congela el capital aportado aunque
    /// después la asamblea cambie el nominal (RN-13).
    pub valor_nominal_compra: f64,
    pub monto_pagado: f64,
    pub liquidada: bool,
    pub fecha_liquidacion: Option<String>,
}

/// Datos que captura la pantalla de Venta de Certificados/Acciones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NuevaCompra {
    pub socio_id: String,
    pub fecha: String,
    /// RF-23: de aquí se deriva la cantidad de acciones, usando el valor nominal.
    pub monto: f64,
}

/// Resultado del cálculo de una compra, antes o después de registrarla (RF-23, RF-24).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculoCompra {
    pub cantidad: i64,
    pub valor_nominal: f64,
    pub monto: f64,
    /// Acciones que tendrá el socio si la compra se registra.
    pub acciones_socio_despues: i64,
    /// Total de acciones del Bankomunal si la compra se registra.
    pub total_bankomunal_despues: i64,
    /// RF-24: % de participación del socio tras la compra.
    pub participacion_pct: f64,
    /// RN-02: si la participación resultante supera el tope permitido.
    pub supera_tope_participacion: bool,
    /// Si el tope de RN-02 todavía no aplica por estar en los primeros meses.
    pub tope_en_periodo_de_gracia: bool,
}

/// Una fila del Control de Acciones por Socio, mes a mes (RF-105).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumenMesAcciones {
    pub mes: String,
    pub compradas: i64,
    pub liquidadas: i64,
    /// Acciones acumuladas del socio al cierre de ese mes.
    pub saldo: i64,
}

/// Parámetros del Bankomunal que necesita este módulo.
///
/// Se declaran aquí, y no se importa el módulo de Configuración, para que el Dominio
/// de Acciones dependa sólo de lo que usa (inversión de dependencias + ISP).
#[derive(Debug, Clone, PartialEq)]
pub struct ParametrosAcciones {
    pub valor_nominal: f64,
    /// Fecha de creación del Bankomunal, para saber si RN-02 ya aplica.
    pub fecha_creacion: String,
    /// RN-09: % del total de acciones que se autoriza vender en cada tramo del PPCFC.
    pub ppcfc_venta_rango1_pct: f64,
    pub ppcfc_venta_rango2_pct: f64,
    /// RN-15: % del cupo del mes que puede tomar un solo socio.
    pub tope_individual_mensual_pct: f64,
}

/// Cuántos meses cerrados exige el PPCFC (RN-09): "Mes 1 (A), Mes 2 (B), Mes 3 (C),
/// promedio = suma ÷ 3", según el formato "Reporte de acciones autorizadas".
pub const MESES_PPCFC: usize = 3;

/// Resultado de evaluar si este mes se pueden vender acciones (RN-09).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "estado")]
pub enum AutorizacionVenta {
    /// Todavía no hay 3 meses cerrados, así que el PPCFC no se puede promediar.
    ///
    /// **No bloquea la venta**: qué hacer en los primeros meses es una decisión abierta
    /// del cliente (D-02) y bloquear equivaldría a elegir una de las opciones por él.
    /// La pantalla lo muestra como pendiente para que la Junta autorice a criterio.
    SinDatosSuficientes { meses_cerrados: usize },
    /// PPCFC por debajo del umbral mínimo: no se venden acciones.
    NoAutoriza { ppcfc_pct: f64 },
    Autoriza {
        ppcfc_pct: f64,
        rango_desde: f64,
        rango_hasta: f64,
        /// % del total de acciones autorizado a vender este mes.
        venta_pct: f64,
        cupo_acciones: i64,
        cupo_monto: f64,
    },
}

/// Cupo de venta del mes y cuánto queda disponible (RN-09 + RN-15).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CupoMensual {
    pub mes: String,
    pub autorizacion: AutorizacionVenta,
    pub vendido_acciones: i64,
    pub vendido_monto: f64,
    /// Cupo menos lo ya vendido. `None` mientras el PPCFC no sea calculable.
    pub disponible_monto: Option<f64>,
    /// RN-15: lo máximo que un solo socio puede tomar del cupo de este mes.
    pub tope_individual_monto: Option<f64>,
}

/// Umbrales del PPCFC. Vienen del reglamento fijo, iguales para todos los Bankomunales.
pub const PPCFC_UMBRAL_MINIMO: f64 = 80.0;
pub const PPCFC_UMBRAL_MEDIO: f64 = 90.0;

/// Promedio de colocación de los últimos meses cerrados (RN-09, D-02).
///
/// `colocaciones` llega ordenada del mes más reciente al más antiguo. Si no hay al
/// menos `MESES_PPCFC` cierres, no hay PPCFC: devuelve `None` en vez de promediar con
/// menos datos, que sería inventar una autorización sobre información incompleta.
pub fn calcular_ppcfc(colocaciones: &[f64]) -> Option<f64> {
    if colocaciones.len() < MESES_PPCFC {
        return None;
    }
    let suma: f64 = colocaciones.iter().take(MESES_PPCFC).sum();
    Some(suma / MESES_PPCFC as f64)
}

/// Aplica los tramos de RN-09 sobre un PPCFC ya calculado.
pub fn tramo_de_venta(
    ppcfc_pct: f64,
    params: &ParametrosAcciones,
) -> Option<(f64, f64, f64)> {
    if ppcfc_pct < PPCFC_UMBRAL_MINIMO {
        None
    } else if ppcfc_pct < PPCFC_UMBRAL_MEDIO {
        Some((PPCFC_UMBRAL_MINIMO, PPCFC_UMBRAL_MEDIO, params.ppcfc_venta_rango1_pct))
    } else {
        Some((PPCFC_UMBRAL_MEDIO, 100.0, params.ppcfc_venta_rango2_pct))
    }
}

/// --- Puertos (hexagonal) ---

pub trait LoteAccionesPort: Send + Sync {
    fn crear(&self, banco_id: &str, lote: &LoteAcciones) -> Result<(), AppError>;
    fn listar_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<Vec<LoteAcciones>, AppError>;
    /// Acciones vigentes de un socio (las de lotes no liquidados).
    fn acciones_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError>;
    /// Acciones vigentes de todo el Bankomunal.
    fn total_acciones(&self, banco_id: &str) -> Result<i64, AppError>;
    /// Acciones vigentes por socio, para la pantalla de listado y los reportes.
    fn acciones_por_socio(&self, banco_id: &str) -> Result<Vec<(String, i64)>, AppError>;
    /// Acciones y monto vendidos en un mes dado, para saber cuánto queda del cupo.
    fn vendido_en_mes(&self, banco_id: &str, mes: &str) -> Result<(i64, f64), AppError>;
}

/// % de colocación de crédito de los meses ya cerrados, insumo del PPCFC (RN-09).
///
/// Lo produce el Cierre Mensual, que sella `colocacion_pct` en cada cierre para que
/// corregir un mes viejo no reescriba en silencio la historia del PPCFC (D-02).
pub trait CierresPort: Send + Sync {
    /// Colocaciones de los últimos cierres, del más reciente al más antiguo.
    fn colocaciones_recientes(&self, banco_id: &str, cuantos: usize) -> Result<Vec<f64>, AppError>;
}

pub trait ParametrosAccionesPort: Send + Sync {
    fn obtener(&self, banco_id: &str) -> Result<ParametrosAcciones, AppError>;
}

/// Asiento en el Libro de Ingresos y Egresos que produce una compra (RF-22).
///
/// El Libro lo lleva el módulo de Caja; Acciones sólo declara lo que necesita de él.
pub trait LibroContablePort: Send + Sync {
    fn registrar_venta_acciones(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError>;
}

/// Meses calendario transcurridos entre dos fechas ISO (`YYYY-MM-...`).
///
/// Cuenta meses, no días: el reglamento razona en meses calendario ("las acciones
/// compradas en enero reciben sus ganancias en enero del próximo año"), así que
/// comparar días daría respuestas distintas según la longitud del mes.
pub fn meses_entre(desde: &str, hasta: &str) -> Option<i64> {
    let indice = |iso: &str| -> Option<i64> {
        let anio: i64 = iso.get(0..4)?.parse().ok()?;
        let mes: i64 = iso.get(5..7)?.parse().ok()?;
        if !(1..=12).contains(&mes) {
            return None;
        }
        Some(anio * 12 + mes)
    };
    Some(indice(hasta)? - indice(desde)?)
}

/// Primer día del mes de una fecha ISO, que es como se guarda `mes_compra` (RN-10).
pub fn mes_de(fecha_iso: &str) -> Option<String> {
    let mes = fecha_iso.get(0..7)?;
    if mes.len() == 7 && mes.as_bytes()[4] == b'-' {
        Some(format!("{mes}-01"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuenta_meses_calendario_no_dias() {
        assert_eq!(meses_entre("2026-01-15", "2026-01-02"), Some(0));
        assert_eq!(meses_entre("2026-01-31", "2026-02-01"), Some(1));
        assert_eq!(meses_entre("2026-01-01", "2027-01-01"), Some(12));
        assert_eq!(meses_entre("2026-11-05", "2027-02-20"), Some(3));
    }

    #[test]
    fn meses_entre_admite_fecha_con_hora() {
        assert_eq!(meses_entre("2026-01-10T12:30:00", "2026-04-01"), Some(3));
    }

    #[test]
    fn meses_entre_rechaza_fechas_invalidas() {
        assert_eq!(meses_entre("", "2026-01-01"), None);
        assert_eq!(meses_entre("2026-13-01", "2026-01-01"), None);
        assert_eq!(meses_entre("no-es-fecha", "2026-01-01"), None);
    }

    #[test]
    fn mes_de_devuelve_el_primer_dia() {
        assert_eq!(mes_de("2026-08-17"), Some("2026-08-01".into()));
        assert_eq!(mes_de("2026-08-01T09:00:00"), Some("2026-08-01".into()));
        assert_eq!(mes_de("abc"), None);
    }

    fn params() -> ParametrosAcciones {
        ParametrosAcciones {
            valor_nominal: 10_000.0,
            fecha_creacion: "2020-01-01".into(),
            ppcfc_venta_rango1_pct: 10.0,
            ppcfc_venta_rango2_pct: 15.0,
            tope_individual_mensual_pct: 20.0,
        }
    }

    /// El formato del cliente: D = A+B+C, E = D ÷ 3.
    #[test]
    fn el_ppcfc_promedia_los_tres_ultimos_cierres() {
        assert_eq!(calcular_ppcfc(&[90.0, 85.0, 80.0]), Some(85.0));
        // Sólo cuenta los 3 más recientes aunque lleguen más.
        assert_eq!(calcular_ppcfc(&[90.0, 90.0, 90.0, 0.0, 0.0]), Some(90.0));
    }

    /// Con menos de 3 cierres no hay PPCFC: promediar con menos datos sería inventar
    /// una autorización sobre información incompleta.
    #[test]
    fn sin_tres_cierres_no_hay_ppcfc() {
        assert_eq!(calcular_ppcfc(&[]), None);
        assert_eq!(calcular_ppcfc(&[95.0]), None);
        assert_eq!(calcular_ppcfc(&[95.0, 90.0]), None);
    }

    /// RN-09: <80% no vende; 80-90% hasta el 10%; 90-100% hasta el 15%.
    #[test]
    fn los_tramos_de_venta_siguen_rn_09() {
        let p = params();
        assert_eq!(tramo_de_venta(79.99, &p), None);
        assert_eq!(tramo_de_venta(80.0, &p), Some((80.0, 90.0, 10.0)));
        assert_eq!(tramo_de_venta(89.99, &p), Some((80.0, 90.0, 10.0)));
        assert_eq!(tramo_de_venta(90.0, &p), Some((90.0, 100.0, 15.0)));
        assert_eq!(tramo_de_venta(100.0, &p), Some((90.0, 100.0, 15.0)));
    }
}
