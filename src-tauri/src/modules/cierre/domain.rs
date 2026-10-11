use serde::{Deserialize, Serialize};

use crate::core::error::AppError;

/// Códigos del Libro que suman a los Ingresos Repartibles (RF-40): Intereses
/// Ordinarios (OR) + Intereses de Mora (MO) + Otros Ingresos (OI).
pub const CODIGOS_INGRESOS_REPARTIBLES: [&str; 3] = ["OR", "MO", "OI"];

/// Código de los Otros Egresos que se restan del Balance de Gestión (RF-41, D-20).
/// Se resta sólo "EG": los GBK (Gastos del Bankomunal) ya salen del Fondo para
/// Gastos —que se retiene aparte— y restarlos otra vez sería una doble resta.
pub const CODIGO_OTROS_EGRESOS: &str = "EG";

/// Una fila del Libro con lo que el Cierre necesita: el código y de qué lado cae.
#[derive(Debug, Clone, PartialEq)]
pub struct FilaLibro {
    pub codigo: String,
    pub ingreso: f64,
    pub egreso: f64,
}

/// Balance del mes: Activo vs Pasivo (RF-91, RF-93).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuadreMes {
    pub mes: String,
    pub efectivo: f64,
    pub cartera: f64,
    pub bienes: f64,
    pub activo_total: f64,
    pub pasivo_total: f64,
    pub cuadra: bool,
}

/// Informe de Gestión Mensual (RF-92): socios, créditos, rendimiento y efectivo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InformeGestionMes {
    pub mes: String,
    pub socios_activos: i64,
    pub creditos_vigentes: i64,
    pub monto_cartera: f64,
    pub efectivo_disponible: f64,
    /// Rendimiento estimado: la ganancia por acción que se repartirá al cierre.
    pub rendimiento_estimado: f64,
}

/// Cálculo del reparto de ganancias del mes (RF-40, RF-41, RF-42, D-20).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepartoMes {
    pub ingresos_repartibles: f64,
    pub otros_egresos: f64,
    pub fondo_gastos: f64,
    pub fondo_incobrables: f64,
    pub balance_neto: f64,
    pub acciones_activas: i64,
    pub valor_por_accion: f64,
}

/// Desglose del Reparto de Ganancias de un mes tal como lo muestra la pantalla (RF-40/41):
/// abre los ingresos repartibles en sus tres orígenes (OR/MO/OI), el balance bruto y neto,
/// el rendimiento por acción del mes y el acumulado de los últimos 12 meses. Es sólo lectura
/// (se recalcula de los movimientos del mes); los fondos son % de los ingresos repartibles (D-20).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesgloseRepartoMes {
    pub mes: String,
    pub intereses_ordinarios: f64,
    pub intereses_mora: f64,
    pub otros_ingresos: f64,
    pub total_ingresos_repartibles: f64,
    pub otros_egresos: f64,
    pub balance_bruto: f64,
    pub fondo_incobrables: f64,
    pub fondo_gastos: f64,
    pub pct_fondo_incobrables: f64,
    pub pct_fondo_gastos: f64,
    pub balance_neto: f64,
    pub acciones_activas: i64,
    pub rendimiento_mes: f64,
    pub rendimiento_acumulado_12m: f64,
    /// Fecha de cierre si el mes ya está cerrado; `None` si sigue abierto.
    pub cerrado_el: Option<String>,
}

/// Una fila mensual de la "hoja" de reparto anual por cohorte (RF-39, D-12): lo que
/// el cliente ve en su Excel, una columna por mes de la ventana de 12 meses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilaRepartoAnual {
    pub mes: String,
    pub acciones_activas: i64,
    pub ingresos_repartibles: f64,
    pub otros_egresos: f64,
    pub fondo_gastos: f64,
    pub fondo_incobrables: f64,
    pub balance_neto: f64,
    /// Ganancia por acción del mes; puede ser negativa si el balance del mes lo fue.
    pub valor_por_accion: f64,
}

/// El pago acumulado de un socio para una cohorte (mes de compra): sus acciones de ese
/// mes y la ganancia devengada durante la ventana de 12 meses (RF-42).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PagoAnualSocio {
    pub socio_id: String,
    pub acciones: i64,
    pub monto_acumulado: f64,
    /// Cuánto de ese acumulado sigue sin pagarse (estado PENDIENTE/LIBERADA).
    pub monto_pendiente: f64,
}

/// La "hoja" de reparto anual de una cohorte: la tabla mes a mes, el acumulado por
/// acción de los 12 meses y el listado de pago por socio (RF-39 a RF-42).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepartoAnual {
    pub mes_cohorte: String,
    pub mes_fin_ventana: String,
    /// Acciones compradas en el mes de la cohorte (Σ cantidad de sus lotes).
    pub acciones_cohorte: i64,
    /// Ganancia por acción acumulada de la ventana = Σ de `valor_por_accion` mensual.
    pub acumulado_por_accion: f64,
    /// Total a pagar a la cohorte = Σ del devengo de sus socios.
    pub total_a_pagar: f64,
    pub meses: Vec<FilaRepartoAnual>,
    pub pagos: Vec<PagoAnualSocio>,
}

/// Devengo acumulado de un lote (total y lo que sigue pendiente de pago), insumo del
/// resumen de pagos de ganancias (RF-42).
#[derive(Debug, Clone, PartialEq)]
pub struct LoteResumenDevengo {
    pub lote_id: String,
    pub socio_id: String,
    pub mes_compra: String,
    pub cantidad: i64,
    pub accrued_total: f64,
    pub accrued_pendiente: f64,
}

/// Una cohorte (lote) de un socio en el historial de ganancias: el mes en que compró,
/// cuántas acciones, cuántos meses lleva y si ya cumplió el año (RN-10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohorteSocio {
    pub mes_compra: String,
    pub cantidad: i64,
    pub meses_transcurridos: i64,
    pub cumple_anio: bool,
    pub ganancia_acumulada: f64,
}

/// Fila del resumen de pagos de ganancias del mes (RF-42, RN-10): lo que se le paga a un
/// socio cuyas acciones cumplen el año, con el detalle de sus cohortes para el historial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocioPagoGanancia {
    pub socio_id: String,
    pub acciones_cumplen: i64,
    pub valor_ganancia_accion: f64,
    pub ganancia_a_pagar: f64,
    /// PAGADA (ya se pagó), PENDIENTE (cumplió y falta pagar) o PROXIMO (aún no cumple).
    pub estado: String,
    pub cohortes: Vec<CohorteSocio>,
}

/// Un renglón de devengo mensual por lote (RF-39, D-12).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Devengo {
    pub lote_acciones_id: String,
    pub socio_id: String,
    pub acciones: i64,
    pub valor_por_accion: f64,
    pub monto: f64,
}

/// Un cierre de mes ya ejecutado (RF-91 a RF-97).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CierreMes {
    pub id: String,
    pub mes: String,
    pub efectivo: f64,
    pub cartera: f64,
    pub bienes: f64,
    pub activo_total: f64,
    pub pasivo_total: f64,
    pub cuadra: bool,
    pub colocacion_pct: f64,
    pub ppcfc_pct: f64,
    pub fecha_cierre: String,
}

/// El renglón del reparto, con su id, para guardar en `reparto_ganancias`.
#[derive(Debug, Clone, PartialEq)]
pub struct RepartoGuardado {
    pub id: String,
    pub cierre_mes_id: String,
    pub mes: String,
    pub total_ingresos_repartibles: f64,
    pub otros_egresos: f64,
    pub fondo_incobrables: f64,
    pub fondo_gastos: f64,
    pub balance_neto: f64,
    pub acciones_activas: i64,
    pub valor_por_accion: f64,
}

/// Todo lo que persiste un Cierre de Mes, en una sola transacción.
#[derive(Debug, Clone, PartialEq)]
pub struct GuardarCierre {
    pub cierre: CierreMes,
    pub reparto: RepartoGuardado,
    pub devengos: Vec<Devengo>,
    pub delta_fondo_gastos: f64,
    pub delta_fondo_incobrables: f64,
}

/// Resultado completo de ejecutar un cierre, lo que la pantalla muestra al terminar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultadoCierre {
    pub cierre: CierreMes,
    pub reparto: RepartoMes,
    pub devengos: Vec<Devengo>,
}

/// Detalle de un cierre ya guardado (historial).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetalleCierre {
    pub cierre: CierreMes,
    pub reparto: RepartoMes,
    pub devengos: Vec<Devengo>,
}

/// Un lote activo, con lo que el devengo necesita.
#[derive(Debug, Clone, PartialEq)]
pub struct LoteActivo {
    pub id: String,
    pub socio_id: String,
    pub cantidad: i64,
}

/// Un lote con su mes de compra, para decidir el aniversario (RN-10).
#[derive(Debug, Clone, PartialEq)]
pub struct LoteCompraMes {
    pub id: String,
    pub socio_id: String,
    pub mes_compra: String,
}

/// Pago de ganancias al cumplir el año (RF-42): cuánto se le entregó a cada socio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PagoGananciaRegistrado {
    pub socio_id: String,
    pub monto: f64,
}

/// Parámetros que el Cierre necesita de la configuración.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametrosFondos {
    pub pct_fondo_gastos: f64,
    pub pct_fondo_incobrables: f64,
    pub tope_reserva_incobrables_pct: f64,
    pub saldo_fondo_gastos: f64,
    pub saldo_fondo_incobrables: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Funciones puras de dominio
// ─────────────────────────────────────────────────────────────────────────────

/// Si un código del Libro suma a los Ingresos Repartibles (RF-40).
pub fn es_ingreso_repartible(codigo: &str) -> bool {
    CODIGOS_INGRESOS_REPARTIBLES.contains(&codigo)
}

/// Ingresos Repartibles del mes = OR + MO + OI (RF-40).
pub fn ingresos_repartibles(filas: &[FilaLibro]) -> f64 {
    filas
        .iter()
        .filter(|f| es_ingreso_repartible(&f.codigo))
        .map(|f| f.ingreso)
        .sum()
}

/// Otros Egresos del mes = sólo código EG (RF-41, D-20).
pub fn otros_egresos(filas: &[FilaLibro]) -> f64 {
    filas
        .iter()
        .filter(|f| f.codigo == CODIGO_OTROS_EGRESOS)
        .map(|f| f.egreso)
        .sum()
}

/// Cálculo del reparto del mes (RF-41, D-20):
///
/// ```text
/// fondo_gastos      = ingresos_repartibles × pct_fondo_gastos%
/// fondo_incobrables = ingresos_repartibles × pct_fondo_incobrables%
///                      recortado por el tope (D-11): el saldo acumulado del fondo
///                      no puede pasar del tope % del capital en acciones.
/// balance_neto      = ingresos − otros_egresos − fondo_gastos − fondo_incobrables
/// ```
///
/// Devuelve `(fondo_gastos, fondo_incobrables, balance_neto)`.
pub fn calcular_reparto(
    ingresos: f64,
    otros_egresos: f64,
    params: &ParametrosFondos,
    capital_en_acciones: f64,
) -> (f64, f64, f64) {
    // El dinero del Bankomunal se maneja en pesos enteros (COP, sin centavos): cada
    // retención y el balance se redondean a peso entero.
    let fondo_gastos = (ingresos * params.pct_fondo_gastos / 100.0).round();

    // RN-08: el Fondo de Reserva para Incobrables deja de crecer al llegar a su
    // tope (% del capital en acciones). La retención del mes se recorta ahí.
    let tope_fondo = capital_en_acciones * params.tope_reserva_incobrables_pct / 100.0;
    let margen = (tope_fondo - params.saldo_fondo_incobrables).max(0.0);
    let bruto_incobrables = ingresos * params.pct_fondo_incobrables / 100.0;
    let fondo_incobrables = bruto_incobrables.min(margen).round();

    let balance_neto =
        (ingresos - otros_egresos - fondo_gastos - fondo_incobrables).round();
    (fondo_gastos, fondo_incobrables, balance_neto)
}

/// Valor de ganancia por acción (RF-42, D-12): balance neto ÷ acciones activas,
/// redondeado a peso entero (COP, sin centavos): la ganancia por acción no se paga
/// en centavos y el pequeño residuo del redondeo no se reparte.
pub fn valor_por_accion(balance_neto: f64, acciones_activas: i64) -> f64 {
    if acciones_activas <= 0 {
        return 0.0;
    }
    (balance_neto / acciones_activas as f64).round()
}

/// Colocación del mes (RN-09, D-02): cartera ÷ (efectivo + cartera).
pub fn colocacion_pct(cartera: f64, efectivo: f64) -> f64 {
    let total = efectivo + cartera;
    if total <= 0.0 {
        return 0.0;
    }
    (cartera / total) * 100.0
}

/// Mes anterior a un `YYYY-MM-01`.
pub fn mes_anterior(mes: &str) -> Option<String> {
    let anio: i64 = mes.get(0..4)?.parse().ok()?;
    let num: i64 = mes.get(5..7)?.parse().ok()?;
    if !(1..=12).contains(&num) {
        return None;
    }
    let (a, m) = if num == 1 { (anio - 1, 12) } else { (anio, num - 1) };
    Some(format!("{a:04}-{m:02}-01"))
}

/// Suma `n` meses a un `YYYY-MM-01` (n ≥ 0). Sirve para acotar la ventana de 12 meses
/// del reparto anual: `sumar_meses(mes, 11)` es el último mes que devenga la cohorte.
pub fn sumar_meses(mes: &str, n: u32) -> Option<String> {
    let anio: i64 = mes.get(0..4)?.parse().ok()?;
    let num: i64 = mes.get(5..7)?.parse().ok()?;
    if !(1..=12).contains(&num) {
        return None;
    }
    // Índice 0-based del mes total, sumamos y volvemos a año/mes.
    let total = (anio * 12 + (num - 1)) + n as i64;
    let a = total.div_euclid(12);
    let m = total.rem_euclid(12) + 1;
    Some(format!("{a:04}-{m:02}-01"))
}

/// Último día del mes de un `YYYY-MM-01`, para acotar fechas (fin del mes).
pub fn fin_de_mes(mes: &str) -> Option<String> {
    let anio: i64 = mes.get(0..4)?.parse().ok()?;
    let num: i64 = mes.get(5..7)?.parse().ok()?;
    if !(1..=12).contains(&num) {
        return None;
    }
    let dias = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][num as usize - 1];
    let mut dias = dias;
    if num == 2 {
        let bisiesto = (anio % 4 == 0 && anio % 100 != 0) || anio % 400 == 0;
        if bisiesto {
            dias += 1;
        }
    }
    Some(format!("{anio:04}-{num:02}-{dias:02}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Puertos (hexagonal)
// ─────────────────────────────────────────────────────────────────────────────

/// Tabla `cierre_mes`: lecturas del historial y la persistencia completa del cierre.
pub trait CierresPort: Send + Sync {
    fn existe_mes(&self, banco_id: &str, mes: &str) -> Result<bool, AppError>;
    fn listar(&self, banco_id: &str) -> Result<Vec<CierreMes>, AppError>;
    fn obtener(&self, banco_id: &str, id: &str) -> Result<Option<CierreMes>, AppError>;
    /// % de colocación de los cierres ya hechos, del más reciente al más antiguo.
    fn colocaciones_recientes(&self, banco_id: &str, cuantos: usize) -> Result<Vec<f64>, AppError>;
    /// Persiste el cierre completo en una sola transacción (RF-95, RF-96, RF-97):
    /// cierre + reparto + devengo de cada lote + marcar los movimientos del mes +
    /// ajustar los saldos de los dos fondos.
    fn guardar_cierre(&self, banco_id: &str, datos: &GuardarCierre) -> Result<(), AppError>;
}

/// Tabla `reparto_ganancias` y `pago_ganancia`: devengo, lectura y pago (RF-39, RF-42).
pub trait RepartoPort: Send + Sync {
    fn obtener_de_cierre(
        &self,
        banco_id: &str,
        cierre_id: &str,
    ) -> Result<Option<RepartoMes>, AppError>;
    fn devengos_de_reparto(
        &self,
        banco_id: &str,
        reparto_id: &str,
    ) -> Result<Vec<Devengo>, AppError>;
    /// Id del reparto de un cierre dado (para leer luego sus devengos).
    fn id_de_cierre(
        &self,
        banco_id: &str,
        cierre_id: &str,
    ) -> Result<Option<String>, AppError>;
    /// "Ganancias no repartidas" del Pasivo = PENDIENTE + LIBERADA (D-13).
    fn ganancias_no_repartidas(&self, banco_id: &str) -> Result<f64, AppError>;
    /// Devengo acumulado por lote activo (total y pendiente), para el resumen de pagos
    /// de ganancias del mes (RF-42).
    fn resumen_por_lote(&self, banco_id: &str) -> Result<Vec<LoteResumenDevengo>, AppError>;
    /// Filas mensuales del reparto (de `reparto_ganancias`) entre dos meses, inclusive,
    /// ordenadas por mes: la tabla de la "hoja" anual del cliente.
    fn filas_reparto_entre(
        &self,
        banco_id: &str,
        desde: &str,
        hasta: &str,
    ) -> Result<Vec<FilaRepartoAnual>, AppError>;
    /// Pago acumulado por socio de una cohorte (lotes comprados en `mes_cohorte`).
    fn pagos_de_cohorte(
        &self,
        banco_id: &str,
        mes_cohorte: &str,
    ) -> Result<Vec<PagoAnualSocio>, AppError>;
    /// Marca como PAGADAS las ganancias PENDIENTES de un lote y devuelve el total
    /// entregado. Es idempotente: un lote ya pagado no vuelve a pagarse.
    fn pagar_lote(&self, banco_id: &str, lote_id: &str, fecha_pago: &str)
        -> Result<f64, AppError>;
    /// D-13: al liquidar un lote ANTES de cumplir el año, sus devengos PENDIENTES pasan a
    /// LIBERADA: no se le pagan al socio que se retira, pero siguen como ganancia colectiva
    /// del Banko (permanecen en "ganancias no repartidas" del Pasivo). Devuelve lo liberado.
    fn liberar_lote(&self, banco_id: &str, lote_id: &str, fecha_liberacion: &str)
        -> Result<f64, AppError>;
}

/// El Libro que el Cierre necesita leer (movimientos y saldo acumulado).
pub trait LibroCierrePort: Send + Sync {
    /// Saldo de caja acumulado hasta el fin del mes (lo que hay en efectivo).
    fn saldo_caja_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError>;
    /// Los movimientos del mes, con su código y su lado del Libro.
    fn movimientos_del_mes(&self, banco_id: &str, mes: &str) -> Result<Vec<FilaLibro>, AppError>;
}

/// Cartera de créditos, para el Balance y la colocación del PPCFC (RN-09).
pub trait CarteraPort: Send + Sync {
    fn saldo_cartera_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError>;
    fn creditos_vigentes_hasta(&self, banco_id: &str, fin: &str) -> Result<i64, AppError>;
}

/// Activo fijo, para el Balance del mes (RF-88, RF-93).
pub trait BienesCierrePort: Send + Sync {
    fn valor_total_hasta(&self, banco_id: &str, fin: &str) -> Result<f64, AppError>;
}

/// Los dos fondos: sus saldos y los % configurados (RF-95).
pub trait FondosPort: Send + Sync {
    fn parametros(&self, banco_id: &str) -> Result<ParametrosFondos, AppError>;
    /// Ajusta el saldo del Fondo de Reserva para Incobrables por `delta` (RF-35/RN-08):
    /// negativo al absorber la deuda incobrable de un socio que se retira debiendo,
    /// positivo al recuperarla luego (pago de deuda pendiente, RF-74).
    fn ajustar_incobrables(&self, banco_id: &str, delta: f64) -> Result<(), AppError>;
}

/// Lotes de acciones, para el devengo y el tope del Fondo de Incobrables.
pub trait LotesPort: Send + Sync {
    fn lotes_activos(&self, banco_id: &str) -> Result<Vec<LoteActivo>, AppError>;
    fn acciones_activas(&self, banco_id: &str) -> Result<i64, AppError>;
    /// Capital en acciones en circulación: Σ cantidad × nominal de compra.
    fn capital_en_acciones(&self, banco_id: &str) -> Result<f64, AppError>;
    fn lotes_con_mes_compra(&self, banco_id: &str) -> Result<Vec<LoteCompraMes>, AppError>;
}

/// Conteo de socios activos, para el Informe de Gestión (RF-92).
pub trait SociosCierrePort: Send + Sync {
    fn socios_activos(&self, banco_id: &str) -> Result<i64, AppError>;
}

/// Asiento que produce el pago de ganancias en el Libro (RF-42).
pub trait LibroContablePort: Send + Sync {
    fn registrar_ganancia_repartida(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
    ) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fila(codigo: &str, ingreso: f64, egreso: f64) -> FilaLibro {
        FilaLibro { codigo: codigo.into(), ingreso, egreso }
    }

    /// RF-40: los ingresos repartibles son OR + MO + OI.
    #[test]
    fn los_ingresos_repartibles_suman_or_mo_oi() {
        let filas = vec![
            fila("OR", 40_000.0, 0.0),
            fila("MO", 10_000.0, 0.0),
            fila("OI", 25_000.0, 0.0),
            // Lo demás no cuenta: aportes, cuotas de capital, donaciones…
            fila("VC", 500_000.0, 0.0),
            fila("PC", 80_000.0, 0.0),
            fila("IFG", 5_000.0, 0.0),
        ];
        assert_eq!(ingresos_repartibles(&filas), 75_000.0);
    }

    /// D-20: sólo el código EG se resta como Otros Egresos; los GBK no, para no
    /// restar dos veces los gastos que ya salen del Fondo para Gastos.
    #[test]
    fn solo_eg_cuenta_como_otros_egresos() {
        let filas = vec![
            fila("EG", 0.0, 19_000.0),
            fila("GBK", 0.0, 8_000.0),
        ];
        assert_eq!(otros_egresos(&filas), 19_000.0);
    }

    /// El ejemplo del cliente: 75.000 repartibles, 19.000 de egresos y dos fondos
    /// del 10% dan un balance neto de 41.000 y, con 76 acciones, 539 por acción
    /// (41.000 ÷ 76 = 539,47 redondeado a peso entero).
    #[test]
    fn el_reparto_sigue_el_ejemplo_del_cliente() {
        let params = ParametrosFondos {
            pct_fondo_gastos: 10.0,
            pct_fondo_incobrables: 10.0,
            tope_reserva_incobrables_pct: 20.0,
            saldo_fondo_gastos: 0.0,
            saldo_fondo_incobrables: 0.0,
        };
        let (fg, fi, bn) = calcular_reparto(75_000.0, 19_000.0, &params, 760_000.0);
        assert_eq!(fg, 7_500.0);
        assert_eq!(fi, 7_500.0);
        assert_eq!(bn, 41_000.0);
        assert_eq!(valor_por_accion(bn, 76), 539.0);
    }

    /// D-11: el Fondo de Incobrables deja de crecer al llegar a su tope (% del
    /// capital en acciones); la retención del mes se recorta, no el saldo existente.
    #[test]
    fn el_fondo_de_incobrables_respeta_su_tope() {
        // Capital de 1.000.000, tope 20% = 200.000; el fondo ya tiene 195.000.
        let params = ParametrosFondos {
            pct_fondo_gastos: 10.0,
            pct_fondo_incobrables: 10.0,
            tope_reserva_incobrables_pct: 20.0,
            saldo_fondo_gastos: 0.0,
            saldo_fondo_incobrables: 195_000.0,
        };
        // El 10% de 200.000 sería 20.000, pero sólo caben 5.000 hasta el tope.
        let (_fg, fi, _bn) = calcular_reparto(200_000.0, 0.0, &params, 1_000_000.0);
        assert_eq!(fi, 5_000.0);

        // Al tope, la retención es cero.
        let params = ParametrosFondos {
            saldo_fondo_incobrables: 200_000.0,
            ..params
        };
        let (_fg, fi, _bn) = calcular_reparto(200_000.0, 0.0, &params, 1_000_000.0);
        assert_eq!(fi, 0.0);
    }

    /// RN-09 (D-02): colocación = cartera ÷ (efectivo + cartera).
    #[test]
    fn la_colocacion_divide_cartera_entre_efectivo_mas_cartera() {
        assert_eq!(colocacion_pct(0.0, 0.0), 0.0, "sin fondo no hay colocación");
        assert_eq!(colocacion_pct(60_000.0, 40_000.0), 60.0);
        assert_eq!(colocacion_pct(0.0, 100_000.0), 0.0);
        assert_eq!(colocacion_pct(100_000.0, 0.0), 100.0);
    }

    #[test]
    fn sumar_meses_avanza_la_ventana_anual() {
        // La ventana de 12 meses de una cohorte: del mes de compra a +11.
        assert_eq!(sumar_meses("2018-07-01", 11).as_deref(), Some("2019-06-01"));
        assert_eq!(sumar_meses("2026-08-01", 0).as_deref(), Some("2026-08-01"));
        assert_eq!(sumar_meses("2026-08-01", 5).as_deref(), Some("2027-01-01"));
        assert_eq!(sumar_meses("2026-12-01", 1).as_deref(), Some("2027-01-01"));
        assert_eq!(sumar_meses("abc", 3), None);
    }

    #[test]
    fn mes_anterior_y_fin_de_mes() {
        assert_eq!(mes_anterior("2026-08-01").as_deref(), Some("2026-07-01"));
        assert_eq!(mes_anterior("2026-01-01").as_deref(), Some("2025-12-01"));
        assert_eq!(fin_de_mes("2026-08-01").as_deref(), Some("2026-08-31"));
        assert_eq!(fin_de_mes("2026-02-01").as_deref(), Some("2026-02-28"));
        assert_eq!(fin_de_mes("2028-02-01").as_deref(), Some("2028-02-29"));
        assert_eq!(fin_de_mes("2026-04-01").as_deref(), Some("2026-04-30"));
        assert_eq!(mes_anterior("abc"), None);
    }
}