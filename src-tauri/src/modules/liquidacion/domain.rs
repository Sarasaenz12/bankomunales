//! Liquidación Final del Bankomunal (disolución): se reparte todo el patrimonio entre
//! los socios y se cierra el Banko. Es el proceso de la hoja "LIQUIDACIÓN FINAL" del
//! cliente. El cálculo es una función pura sobre los insumos (caja, créditos por cobrar,
//! acciones y deudas de cada socio) más los ajustes que decide la Asamblea al disolver
//! (incobrables a castigar y los dos fondos de reserva).

use serde::{Deserialize, Serialize};

/// Insumos a nivel Banko que decide la Asamblea al liquidar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParametrosLiquidacion {
    /// Σ del saldo pendiente de los créditos por cobrar.
    pub creditos_por_cobrar: f64,
    /// Dinero disponible en caja.
    pub caja: f64,
    /// Créditos vencidos que se castigan (no se cobrarán): salen del fondo.
    pub incobrables: f64,
    /// % del fondo sano que se retiene como Fondo de Gastos al cerrar.
    pub pct_fondo_gastos: f64,
    /// % del fondo sano que se retiene como Reserva de Incobrables al cerrar.
    pub pct_fondo_incobrables: f64,
    /// Valor nominal de la acción (RN-13).
    pub valor_nominal: f64,
}

/// Un socio en la liquidación: sus acciones y su deuda. `caps_ganancia` son las acciones
/// que participan del reparto de la ganancia (normalmente sus acciones activas; puede
/// incluir las de un socio que se retiró dentro del período pero sigue con ganancia por
/// cobrar). `caps_activas` son las que hoy tiene vigentes y por las que recibe su capital.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocioLiquidacion {
    pub socio_id: String,
    pub nombre: String,
    pub cedula: String,
    pub caps_ganancia: i64,
    pub caps_activas: i64,
    pub credito: f64,
}

/// El resultado por socio: cuánto recibe (o queda debiendo) al liquidar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoSocio {
    pub socio_id: String,
    pub nombre: String,
    pub cedula: String,
    pub caps_activas: i64,
    pub caps_ganancia: i64,
    /// Valor de sus acciones = caps_activas × nominal.
    pub valor_caps: f64,
    /// Su parte de la ganancia = caps_ganancia × valor de ganancia por acción.
    pub ganancia: f64,
    pub credito: f64,
    /// Ingresos totales − crédito. Positivo: el Banko le paga. Negativo: queda debiendo.
    pub valor_a_pagar: f64,
    pub queda_debiendo: bool,
}

/// La liquidación completa: los totales del Banko y el detalle por socio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiquidacionFinal {
    pub creditos_por_cobrar: f64,
    pub caja: f64,
    pub fondo_credito: f64,
    pub incobrables: f64,
    pub fondo_sano: f64,
    pub reserva_gastos: f64,
    pub reserva_incobrables: f64,
    pub capital_real: f64,
    pub acciones_activas: i64,
    pub monto_acciones: f64,
    pub ganancia_general: f64,
    pub total_caps_ganancia: i64,
    pub valor_ganancia_accion: f64,
    pub total_a_pagar: f64,
    pub socios: Vec<ResultadoSocio>,
}

/// Calcula la liquidación final (función pura). Sigue la hoja del cliente:
///
/// ```text
/// fondo de crédito = créditos por cobrar + caja
/// fondo sano       = fondo de crédito − incobrables
/// capital real     = fondo sano − reserva de gastos − reserva de incobrables
/// ganancia general = capital real − (acciones activas × nominal)
/// ganancia/acción  = ganancia general ÷ total de acciones con ganancia   (piso a peso)
/// por socio:
///   valor a pagar  = (caps activas × nominal) + (caps ganancia × ganancia/acción) − crédito
/// ```
pub fn calcular_liquidacion(
    p: &ParametrosLiquidacion,
    socios: &[SocioLiquidacion],
) -> LiquidacionFinal {
    let fondo_credito = p.creditos_por_cobrar + p.caja;
    let fondo_sano = fondo_credito - p.incobrables;
    let reserva_gastos = fondo_sano * p.pct_fondo_gastos / 100.0;
    let reserva_incobrables = fondo_sano * p.pct_fondo_incobrables / 100.0;
    let capital_real = fondo_sano - reserva_gastos - reserva_incobrables;

    let acciones_activas: i64 = socios.iter().map(|s| s.caps_activas).sum();
    let monto_acciones = acciones_activas as f64 * p.valor_nominal;
    let ganancia_general = capital_real - monto_acciones;

    let total_caps_ganancia: i64 = socios.iter().map(|s| s.caps_ganancia).sum();
    // Piso a peso entero, como la hoja del cliente: la ganancia por acción no se paga en
    // centavos y el pequeño residual del redondeo no se reparte.
    let valor_ganancia_accion = if total_caps_ganancia > 0 {
        (ganancia_general / total_caps_ganancia as f64).floor()
    } else {
        0.0
    };

    let socios: Vec<ResultadoSocio> = socios
        .iter()
        .map(|s| {
            let valor_caps = s.caps_activas as f64 * p.valor_nominal;
            let ganancia = s.caps_ganancia as f64 * valor_ganancia_accion;
            let valor_a_pagar = valor_caps + ganancia - s.credito;
            ResultadoSocio {
                socio_id: s.socio_id.clone(),
                nombre: s.nombre.clone(),
                cedula: s.cedula.clone(),
                caps_activas: s.caps_activas,
                caps_ganancia: s.caps_ganancia,
                valor_caps,
                ganancia,
                credito: s.credito,
                valor_a_pagar,
                queda_debiendo: valor_a_pagar < 0.0,
            }
        })
        .collect();

    let total_a_pagar = socios.iter().map(|s| s.valor_a_pagar).sum();

    LiquidacionFinal {
        creditos_por_cobrar: p.creditos_por_cobrar,
        caja: p.caja,
        fondo_credito,
        incobrables: p.incobrables,
        fondo_sano,
        reserva_gastos,
        reserva_incobrables,
        capital_real,
        acciones_activas,
        monto_acciones,
        ganancia_general,
        total_caps_ganancia,
        valor_ganancia_accion,
        total_a_pagar,
        socios,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn socio(nombre: &str, caps_g: i64, caps_a: i64, credito: f64) -> SocioLiquidacion {
        SocioLiquidacion {
            socio_id: nombre.into(),
            nombre: nombre.into(),
            cedula: String::new(),
            caps_ganancia: caps_g,
            caps_activas: caps_a,
            credito,
        }
    }

    /// El ejemplo real del cliente (BK Bankordillera): reproduce los totales del Banko y
    /// el valor a pagar de cada socio de la hoja "LIQUIDACIÓN FINAL".
    #[test]
    fn reproduce_la_liquidacion_de_bankordillera() {
        let p = ParametrosLiquidacion {
            creditos_por_cobrar: 9_187_000.0,
            caja: 7_370_350.0,
            incobrables: 0.0,
            pct_fondo_gastos: 0.0,
            pct_fondo_incobrables: 0.0,
            valor_nominal: 10_000.0,
        };
        let socios = vec![
            socio("OLIVA", 104, 104, 800_000.0),
            socio("BLANCA", 112, 112, 1_000_000.0),
            socio("DANIEL", 62, 62, 1_000_000.0),
            socio("MARIBEL", 90, 90, 300_000.0),
            socio("FERNANDO URAN", 389, 389, 3_400_000.0),
            socio("LUZ MERY", 35, 35, 1_000_000.0),
            socio("MIRYAM", 291, 291, 300_000.0),
            socio("FERNANDO CORREA", 118, 0, 0.0),
            socio("MAURA", 115, 115, 787_000.0),
            socio("SANDRA", 30, 30, 600_000.0),
            socio("JOSE TIQUE", 2, 2, 0.0),
        ];

        let r = calcular_liquidacion(&p, &socios);

        assert_eq!(r.fondo_credito, 16_557_350.0);
        assert_eq!(r.capital_real, 16_557_350.0);
        assert_eq!(r.acciones_activas, 1_230);
        assert_eq!(r.monto_acciones, 12_300_000.0);
        assert_eq!(r.ganancia_general, 4_257_350.0);
        assert_eq!(r.total_caps_ganancia, 1_348);
        assert_eq!(r.valor_ganancia_accion, 3_158.0);

        // Oliva recibe 568.432; Fernando Correa (retirado, 0 caps activas) cobra su
        // ganancia; Daniel queda debiendo 184.204.
        let oliva = &r.socios[0];
        assert_eq!(oliva.ganancia, 328_432.0);
        assert_eq!(oliva.valor_a_pagar, 568_432.0);
        assert!(!oliva.queda_debiendo);

        let correa = r.socios.iter().find(|s| s.nombre == "FERNANDO CORREA").unwrap();
        assert_eq!(correa.valor_caps, 0.0);
        assert_eq!(correa.valor_a_pagar, 372_644.0);

        let daniel = r.socios.iter().find(|s| s.nombre == "DANIEL").unwrap();
        assert_eq!(daniel.valor_a_pagar, -184_204.0);
        assert!(daniel.queda_debiendo);
    }

    #[test]
    fn sin_acciones_con_ganancia_no_divide_por_cero() {
        let p = ParametrosLiquidacion {
            creditos_por_cobrar: 0.0,
            caja: 100_000.0,
            incobrables: 0.0,
            pct_fondo_gastos: 0.0,
            pct_fondo_incobrables: 0.0,
            valor_nominal: 10_000.0,
        };
        let r = calcular_liquidacion(&p, &[]);
        assert_eq!(r.valor_ganancia_accion, 0.0);
        assert_eq!(r.total_a_pagar, 0.0);
    }
}
