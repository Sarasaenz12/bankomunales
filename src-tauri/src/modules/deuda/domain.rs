//! Servicio de la Deuda (RF-63 a RF-82): pago de cuotas, interés ordinario y mora.
//!
//! Fase 1 — Pago de Cuota (OR + PC + MO): al pagar una cuota, el monto se imputa en
//! **cascada mora → interés → capital** (decisión del cliente), la mora se calcula
//! **sobre el valor de la cuota** por los días de atraso, y se admiten **abonos
//! parciales**. Cada concepto se asienta en el Libro con su código (OR/PC/MO) por Caja.

use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Redondeo a peso entero, consistente con la tabla de amortización de Créditos: el
/// dinero del Bankomunal no maneja centavos (COP), tampoco en la mora ni en los abonos.
pub fn redondear(v: f64) -> f64 {
    v.round()
}

/// Una cuota del crédito con lo que se ha pagado (RF-63..72). Los `*_pagado` permiten
/// abonos parciales; `pagada` es un derivado de que capital e interés estén cubiertos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cuota {
    pub id: String,
    pub credito_id: String,
    pub numero: i64,
    pub fecha_vencimiento: String,
    pub fecha_pago: Option<String>,
    pub capital: f64,
    pub interes: f64,
    pub valor_total: f64,
    pub capital_pagado: f64,
    pub interes_pagado: f64,
    pub mora_pagada: f64,
    pub pagada: bool,
    pub dias_atraso: i64,
}

impl Cuota {
    pub fn capital_pendiente(&self) -> f64 {
        (self.capital - self.capital_pagado).max(0.0)
    }
    pub fn interes_pendiente(&self) -> f64 {
        (self.interes - self.interes_pagado).max(0.0)
    }
}

/// Datos del crédito que el pago necesita (RF-63, RF-69).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditoDeuda {
    pub id: String,
    pub socio_id: String,
    pub numero: String,
    /// Monto desembolsado original; con el plazo original define el capital de referencia
    /// por cuota (Monto ÷ plazo) usado al re-amortizar tras un abono a capital.
    pub monto_original: f64,
    /// Tasa de interés ordinario mensual del crédito (para recalcular el interés al re-amortizar).
    pub tasa: f64,
    pub saldo_pendiente: f64,
    pub plazo_cuotas: i64,
    pub estatus: String,
}

impl CreditoDeuda {
    /// Capital fijo de referencia por cuota: Monto original ÷ plazo original (peso entero).
    pub fn capital_referencia(&self) -> f64 {
        if self.plazo_cuotas <= 0 {
            return 0.0;
        }
        redondear(self.monto_original / self.plazo_cuotas as f64)
    }
}

/// Nº de cuotas en que se reparte un saldo al re-amortizar: se conserva el capital de
/// referencia por cuota; si el saldo no es múltiplo exacto, la cantidad se redondea hacia
/// arriba (así ninguna cuota supera el capital original) y luego el capital se iguala entre
/// las cuotas resultantes. Saldo cero (o sin referencia) ⇒ sin cuotas.
pub fn plazo_reamortizado(nuevo_saldo: f64, capital_referencia: f64) -> i64 {
    if nuevo_saldo <= 0.01 {
        return 0;
    }
    if capital_referencia <= 0.0 {
        return 1;
    }
    (nuevo_saldo / capital_referencia).ceil().max(1.0) as i64
}

/// Una cuota del plan re-amortizado, lista para persistir (Nº ya reasignado y consecutivo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuotaReamortizada {
    pub numero: i64,
    pub fecha_vencimiento: String,
    pub capital: f64,
    pub interes: f64,
    pub valor_total: f64,
}

/// Lo que la pantalla muestra tras un abono a capital: cuánto bajó el saldo y cuántas
/// cuotas quedan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoAbono {
    pub monto_capital: f64,
    pub nuevo_saldo: f64,
    pub cuotas_restantes: i64,
    pub estatus_credito: String,
}

/// Reparto de un pago entre mora, interés y capital (RF-65/69/72).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AplicacionPago {
    pub mora: f64,
    pub interes: f64,
    pub capital: f64,
}

impl AplicacionPago {
    pub fn total(&self) -> f64 {
        redondear(self.mora + self.interes + self.capital)
    }
}

/// Lo que la pantalla muestra tras registrar un pago de cuota (RF-65/69/72).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoPago {
    pub aplicacion: AplicacionPago,
    pub mora_debida: f64,
    pub dias_atraso: i64,
    pub cuota_saldada: bool,
    pub nuevo_saldo: f64,
    pub estatus_credito: String,
}

/// Días de atraso entre el vencimiento y la fecha de pago (0 si se paga a tiempo).
pub fn dias_atraso(vencimiento: &str, fecha_pago: &str) -> i64 {
    use chrono::NaiveDate;
    let dia = |s: &str| NaiveDate::parse_from_str(s.get(0..10).unwrap_or(s), "%Y-%m-%d").ok();
    match (dia(vencimiento), dia(fecha_pago)) {
        (Some(v), Some(p)) => (p - v).num_days().max(0),
        _ => 0,
    }
}

/// Último día del mes de `fecha` (YYYY-MM-DD). El socio tiene plazo hasta fin de mes para
/// pagar en la reunión —que no siempre cae el mismo día—, así que la mora no corre antes.
pub fn fin_de_mes(fecha: &str) -> String {
    use chrono::{Datelike, Duration, NaiveDate};
    match NaiveDate::parse_from_str(fecha.get(0..10).unwrap_or(fecha), "%Y-%m-%d") {
        Ok(d) => {
            let (ny, nm) = if d.month() == 12 { (d.year() + 1, 1) } else { (d.year(), d.month() + 1) };
            let primero_siguiente = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
            (primero_siguiente - Duration::days(1)).format("%Y-%m-%d").to_string()
        }
        Err(_) => fecha.to_string(),
    }
}

/// Días de mora: atraso contado **desde el fin del mes de vencimiento** (0 dentro de ese
/// mes). Los intereses de mora empiezan a correr el día 1 del mes siguiente, de modo que
/// un corrimiento de la reunión dentro del mismo mes no genera mora.
pub fn dias_mora(vencimiento: &str, fecha_pago: &str) -> i64 {
    dias_atraso(&fin_de_mes(vencimiento), fecha_pago)
}

/// Interés de mora de una cuota (RF-71): **sobre el valor de la cuota**, la tasa de mora
/// mensual prorrateada por día × los días de atraso.
pub fn mora_de_cuota(valor_cuota: f64, dias_atraso: i64, tasa_mora_pct: f64) -> f64 {
    if dias_atraso <= 0 || valor_cuota <= 0.0 {
        return 0.0;
    }
    redondear(valor_cuota * (tasa_mora_pct / 100.0) / 30.0 * dias_atraso as f64)
}

/// Imputa `monto` a una cuota en cascada **mora → interés → capital**, sin exceder lo
/// que se debe de cada concepto. `mora_debida` es la mora total exigible de la cuota.
pub fn aplicar_pago(cuota: &Cuota, mora_debida: f64, monto: f64) -> AplicacionPago {
    let mut restante = monto.max(0.0);
    let a_mora = restante.min((mora_debida - cuota.mora_pagada).max(0.0));
    restante -= a_mora;
    let a_interes = restante.min(cuota.interes_pendiente());
    restante -= a_interes;
    let a_capital = restante.min(cuota.capital_pendiente());
    AplicacionPago {
        mora: redondear(a_mora),
        interes: redondear(a_interes),
        capital: redondear(a_capital),
    }
}

/// Lo máximo exigible de una cuota hoy: mora pendiente + interés pendiente + capital
/// pendiente. Un pago no puede superar esto (RF-68); el excedente iría a otra cuota.
pub fn total_debido(cuota: &Cuota, mora_debida: f64) -> f64 {
    redondear(
        (mora_debida - cuota.mora_pagada).max(0.0)
            + cuota.interes_pendiente()
            + cuota.capital_pendiente(),
    )
}

/// Si la cuota queda saldada tras aplicar el pago: capital e interés cubiertos.
pub fn cuota_saldada(cuota: &Cuota, aplic: &AplicacionPago) -> bool {
    let capital_ok = cuota.capital_pagado + aplic.capital >= cuota.capital - 0.01;
    let interes_ok = cuota.interes_pagado + aplic.interes >= cuota.interes - 0.01;
    capital_ok && interes_ok
}

// ─────────────────────────────────────────────────────────────────────────────
// Puertos (hexagonal)
// ─────────────────────────────────────────────────────────────────────────────

/// Lectura de créditos (por número) que el pago necesita para confirmar (RF-63).
pub trait CreditoConsultaPort: Send + Sync {
    fn buscar_por_numero(&self, banco_id: &str, numero: &str) -> Result<Option<CreditoDeuda>, AppError>;
}

/// Cuotas del crédito y la escritura atómica del pago (cuota + crédito en una sola tx).
pub trait CuotaPort: Send + Sync {
    fn cuotas_de_credito(&self, banco_id: &str, credito_id: &str) -> Result<Vec<Cuota>, AppError>;
    /// Persiste el pago: actualiza la cuota (montos pagados, `pagada`, fecha, atraso) y
    /// el crédito (saldo, `cuota_actual`, estatus) en una sola transacción (RF-69).
    fn registrar_pago(
        &self,
        banco_id: &str,
        cuota: &Cuota,
        credito_id: &str,
        nuevo_saldo: f64,
        nueva_cuota_actual: i64,
        nuevo_estatus: &str,
    ) -> Result<(), AppError>;

    /// Re-amortiza el crédito en una sola transacción: (opcional) marca `cuota_pagada`,
    /// borra las cuotas NO pagadas con `numero >= desde_numero`, inserta el plan `nuevas`
    /// y actualiza el crédito (saldo, cuota actual, estatus). Usado por el abono a capital
    /// y por el pago de varias cuotas.
    #[allow(clippy::too_many_arguments)]
    fn reamortizar(
        &self,
        banco_id: &str,
        credito_id: &str,
        cuota_pagada: Option<&Cuota>,
        desde_numero: i64,
        nuevas: &[CuotaReamortizada],
        nuevo_saldo: f64,
        nueva_cuota_actual: i64,
        nuevo_estatus: &str,
    ) -> Result<(), AppError>;
}

/// Asientos en el Libro que produce un pago (RF-65/69/72), llevados por Caja.
pub trait LibroDeudaPort: Send + Sync {
    fn registrar_interes_ordinario(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError>;
    fn registrar_pago_capital(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError>;
    fn registrar_mora(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError>;
}

/// Tasa de mora vigente del Bankomunal (RF-59, RF-71).
pub trait ParametrosDeudaPort: Send + Sync {
    fn tasa_mora(&self, banco_id: &str) -> Result<f64, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cuota(capital: f64, interes: f64) -> Cuota {
        Cuota {
            id: "c1".into(),
            credito_id: "cred1".into(),
            numero: 1,
            fecha_vencimiento: "2026-01-31".into(),
            fecha_pago: None,
            capital,
            interes,
            valor_total: redondear(capital + interes),
            capital_pagado: 0.0,
            interes_pagado: 0.0,
            mora_pagada: 0.0,
            pagada: false,
            dias_atraso: 0,
        }
    }

    #[test]
    fn los_dias_de_atraso_no_son_negativos() {
        assert_eq!(dias_atraso("2026-01-31", "2026-02-10"), 10);
        assert_eq!(dias_atraso("2026-01-31", "2026-01-31"), 0);
        assert_eq!(dias_atraso("2026-01-31", "2026-01-20"), 0, "pago anticipado no da atraso");
        assert_eq!(dias_atraso("2026-01-31T00:00:00", "2026-02-05"), 5);
    }

    #[test]
    fn el_fin_de_mes_es_el_ultimo_dia() {
        assert_eq!(fin_de_mes("2026-09-20"), "2026-09-30");
        assert_eq!(fin_de_mes("2026-02-10"), "2026-02-28", "2026 no es bisiesto");
        assert_eq!(fin_de_mes("2026-12-01"), "2026-12-31");
    }

    /// La mora no corre dentro del mes de vencimiento (plazo hasta fin de mes para pagar en
    /// la reunión); empieza a contar el día 1 del mes siguiente.
    #[test]
    fn la_mora_tiene_gracia_hasta_fin_de_mes() {
        // Cuota vence 20-sep; la reunión se corre al 25-sep → sin mora.
        assert_eq!(dias_mora("2026-09-20", "2026-09-25"), 0);
        // Pagar el último día del mes tampoco genera mora.
        assert_eq!(dias_mora("2026-09-20", "2026-09-30"), 0);
        // A partir del 1 de octubre sí corre: el 1-oct es 1 día, el 5-oct son 5.
        assert_eq!(dias_mora("2026-09-20", "2026-10-01"), 1);
        assert_eq!(dias_mora("2026-09-20", "2026-10-05"), 5);
    }

    /// RF-71: la mora se calcula sobre el valor de la cuota, prorrateando la tasa
    /// mensual por día. Cuota de 100.000 con 10 días y mora 5% → 1.667 (peso entero).
    #[test]
    fn la_mora_es_sobre_la_cuota_prorrateada_por_dia() {
        assert_eq!(mora_de_cuota(100_000.0, 10, 5.0), 1_667.0);
        assert_eq!(mora_de_cuota(100_000.0, 0, 5.0), 0.0, "a tiempo no hay mora");
        assert_eq!(mora_de_cuota(100_000.0, 30, 5.0), 5_000.0, "un mes = la tasa completa");
    }

    /// La cascada paga primero mora, luego interés, luego capital.
    #[test]
    fn el_pago_se_imputa_en_cascada() {
        let c = cuota(50_000.0, 5_000.0);
        let mora = 1_667.0;
        // Pago completo de la cuota + su mora.
        let a = aplicar_pago(&c, mora, 56_667.0);
        assert_eq!(a.mora, 1_667.0);
        assert_eq!(a.interes, 5_000.0);
        assert_eq!(a.capital, 50_000.0);
        assert!(cuota_saldada(&c, &a));
    }

    /// Abono parcial: alcanza para la mora y parte del interés, nada de capital.
    #[test]
    fn el_abono_parcial_respeta_la_cascada() {
        let c = cuota(50_000.0, 5_000.0);
        let mora = 1_667.0;
        let a = aplicar_pago(&c, mora, 3_000.0);
        assert_eq!(a.mora, 1_667.0);
        assert_eq!(a.interes, 1_333.0);
        assert_eq!(a.capital, 0.0);
        assert!(!cuota_saldada(&c, &a), "no se saldó: falta capital");
    }

    /// Sin atraso, la cascada arranca en el interés (mora = 0).
    #[test]
    fn sin_mora_arranca_en_interes() {
        let c = cuota(50_000.0, 5_000.0);
        let a = aplicar_pago(&c, 0.0, 20_000.0);
        assert_eq!(a.mora, 0.0);
        assert_eq!(a.interes, 5_000.0);
        assert_eq!(a.capital, 15_000.0);
    }

    #[test]
    fn el_total_debido_suma_los_tres_conceptos() {
        let mut c = cuota(50_000.0, 5_000.0);
        assert_eq!(total_debido(&c, 1_667.0), 56_667.0);
        // Con parte ya pagada, sólo cuenta lo pendiente.
        c.interes_pagado = 5_000.0;
        c.capital_pagado = 10_000.0;
        assert_eq!(total_debido(&c, 1_667.0), 41_667.0);
    }

    /// El plazo re-amortizado conserva el capital de referencia; el resto no exacto sube
    /// la cantidad de cuotas (redondeo hacia arriba) para igualar luego el capital.
    #[test]
    fn el_plazo_reamortizado_redondea_hacia_arriba() {
        // 500.000 con referencia 100.000 → 5 cuotas exactas.
        assert_eq!(plazo_reamortizado(500_000.0, 100_000.0), 5);
        // 450.000 con referencia 100.000 → 5 cuotas (4,5 hacia arriba), luego 90.000 c/u.
        assert_eq!(plazo_reamortizado(450_000.0, 100_000.0), 5);
        // Saldo cero ⇒ crédito cancelado, sin cuotas.
        assert_eq!(plazo_reamortizado(0.0, 100_000.0), 0);
    }
}
