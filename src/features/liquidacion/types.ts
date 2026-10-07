// ── Liquidación Final del Bankomunal (disolución) ──

export interface ParametrosLiquidacion {
    creditos_por_cobrar: number;
    caja: number;
    incobrables: number;
    pct_fondo_gastos: number;
    pct_fondo_incobrables: number;
    valor_nominal: number;
}

export interface SocioLiquidacion {
    socio_id: string;
    nombre: string;
    cedula: string;
    caps_ganancia: number;
    caps_activas: number;
    credito: number;
}

export interface DatosLiquidacion {
    parametros: ParametrosLiquidacion;
    socios: SocioLiquidacion[];
}

export interface ResultadoSocio {
    socio_id: string;
    nombre: string;
    cedula: string;
    caps_activas: number;
    caps_ganancia: number;
    valor_caps: number;
    ganancia: number;
    credito: number;
    valor_a_pagar: number;
    queda_debiendo: boolean;
}

export interface DatosLiquidacionSocio {
    socio_id: string;
    nombre: string;
    cedula: string;
    caps_activas: number;
    acciones_comprometidas: number;
    acciones_comprometidas_fiador: number;
    ganancia_devengada: number;
    deuda: number;
    valor_nominal: number;
}

// ── Venta / liquidación PARCIAL de acciones (el socio sigue activo) ──

export type DestinoLiquidacionParcial = 'efectivo' | 'abono_deuda';

export interface PreviewLiquidacionParcial {
    cantidad: number;
    libres: number;
    capital: number;
    ganancia: number;
    valor: number;
    deuda: number;
    bloqueo: string | null;
}

export interface ResultadoLiquidacionParcial {
    capital: number;
    ganancia: number;
    valor: number;
    abonado_deuda: number;
    pagado_efectivo: number;
    nueva_deuda: number;
    acciones_liquidadas: number;
    acciones_restantes: number;
}

export interface LiquidacionFinal {
    creditos_por_cobrar: number;
    caja: number;
    fondo_credito: number;
    incobrables: number;
    fondo_sano: number;
    reserva_gastos: number;
    reserva_incobrables: number;
    capital_real: number;
    acciones_activas: number;
    monto_acciones: number;
    ganancia_general: number;
    total_caps_ganancia: number;
    valor_ganancia_accion: number;
    total_a_pagar: number;
    socios: ResultadoSocio[];
}
