import { EstatusSocio } from '../socios/types';

// ── Servicio de la Deuda: pago de cuotas (RF-63 a RF-72) ──

export interface CreditoDeuda {
    id: string;
    socio_id: string;
    numero: string;
    monto_original: number;
    tasa: number;
    saldo_pendiente: number;
    plazo_cuotas: number;
    estatus: string;
}

export interface Cuota {
    id: string;
    credito_id: string;
    numero: number;
    fecha_vencimiento: string;
    fecha_pago: string | null;
    capital: number;
    interes: number;
    valor_total: number;
    capital_pagado: number;
    interes_pagado: number;
    mora_pagada: number;
    pagada: boolean;
    dias_atraso: number;
}

export interface EstadoCreditoDeuda {
    credito: CreditoDeuda;
    socio_nombre: string;
    cuotas: Cuota[];
}

export interface AplicacionPago {
    mora: number;
    interes: number;
    capital: number;
}

export interface ResultadoPago {
    aplicacion: AplicacionPago;
    mora_debida: number;
    dias_atraso: number;
    cuota_saldada: boolean;
    nuevo_saldo: number;
    estatus_credito: string;
}

/** Resultado de un abono a capital: cuánto bajó el saldo y cuántas cuotas quedan. */
export interface ResultadoAbono {
    monto_capital: number;
    nuevo_saldo: number;
    cuotas_restantes: number;
    estatus_credito: string;
}

/** Socio retirado con deuda incobrable pendiente (RF-73 a RF-76). El backend devuelve la
 *  entidad completa; aquí se declaran sólo los campos que usa la pantalla de cobro. */
export interface SocioDeudor {
    id: string;
    cedula: string;
    nombres: string;
    apellidos: string;
    celular: string;
    estatus: EstatusSocio;
    fecha_ingreso: string;
    saldo_incobrable: number;
}
