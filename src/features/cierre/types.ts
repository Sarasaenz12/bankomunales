// ── Cierre de Mes (RF-91 a RF-97) y Reparto de Ganancias (RF-39 a RF-42) ──

export interface CuadreMes {
    mes: string;
    efectivo: number;
    cartera: number;
    bienes: number;
    activo_total: number;
    pasivo_total: number;
    cuadra: boolean;
}

export interface InformeGestionMes {
    mes: string;
    socios_activos: number;
    creditos_vigentes: number;
    monto_cartera: number;
    efectivo_disponible: number;
    rendimiento_estimado: number;
}

export interface RepartoMes {
    ingresos_repartibles: number;
    otros_egresos: number;
    fondo_gastos: number;
    fondo_incobrables: number;
    balance_neto: number;
    acciones_activas: number;
    valor_por_accion: number;
}

export interface PrevisualizacionCierre {
    cuadre: CuadreMes;
    informe: InformeGestionMes;
    reparto: RepartoMes;
}

export interface DesgloseRepartoMes {
    mes: string;
    intereses_ordinarios: number;
    intereses_mora: number;
    otros_ingresos: number;
    total_ingresos_repartibles: number;
    otros_egresos: number;
    balance_bruto: number;
    fondo_incobrables: number;
    fondo_gastos: number;
    pct_fondo_incobrables: number;
    pct_fondo_gastos: number;
    balance_neto: number;
    acciones_activas: number;
    rendimiento_mes: number;
    rendimiento_acumulado_12m: number;
    cerrado_el: string | null;
}

export interface CierreMes {
    id: string;
    mes: string;
    efectivo: number;
    cartera: number;
    bienes: number;
    activo_total: number;
    pasivo_total: number;
    cuadra: boolean;
    colocacion_pct: number;
    ppcfc_pct: number;
    fecha_cierre: string;
}

export interface Devengo {
    lote_acciones_id: string;
    socio_id: string;
    acciones: number;
    valor_por_accion: number;
    monto: number;
}

export interface DetalleCierre {
    cierre: CierreMes;
    reparto: RepartoMes;
    devengos: Devengo[];
}

export interface ResultadoCierre {
    cierre: CierreMes;
    reparto: RepartoMes;
    devengos: Devengo[];
}

export interface PagoGananciaRegistrado {
    socio_id: string;
    monto: number;
}

// ── Reparto anual por cohorte (la "hoja" del Excel del cliente) ──

export interface FilaRepartoAnual {
    mes: string;
    acciones_activas: number;
    ingresos_repartibles: number;
    otros_egresos: number;
    fondo_gastos: number;
    fondo_incobrables: number;
    balance_neto: number;
    valor_por_accion: number;
}

export interface PagoAnualSocio {
    socio_id: string;
    acciones: number;
    monto_acumulado: number;
    monto_pendiente: number;
}

export interface RepartoAnual {
    mes_cohorte: string;
    mes_fin_ventana: string;
    acciones_cohorte: number;
    acumulado_por_accion: number;
    total_a_pagar: number;
    meses: FilaRepartoAnual[];
    pagos: PagoAnualSocio[];
}

// ── Resumen de pagos de ganancias (RF-42): quién cumple el año este mes ──

export interface CohorteSocio {
    mes_compra: string;
    cantidad: number;
    meses_transcurridos: number;
    cumple_anio: boolean;
    ganancia_acumulada: number;
}

export interface FilaPagoGanancia {
    socio_id: string;
    nombre: string;
    acciones_cumplen: number;
    valor_ganancia_accion: number;
    ganancia_a_pagar: number;
    estado: string; // PAGADA | PENDIENTE | PROXIMO
    cohortes: CohorteSocio[];
}