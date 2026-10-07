export interface ReporteSocioActivo {
    id: string;
    cedula: string;
    nombres: string;
    apellidos: string;
    fecha_ingreso: string;
    certificados_activos: number;
    liquidados_parcialmente: number;
}

export interface ReporteGestionFila {
    mes: string;
    fecha_cierre: string;
    activo_total: number;
    pasivo_total: number;
    ingresos_del_mes: number;
    egresos_del_mes: number;
    balance_neto: number;
    valor_por_accion: number;
}

export interface ReporteFiadorFila {
    fiador_cedula: string;
    fiador_nombres: string;
    deudor_cedula: string;
    deudor_nombres: string;
    credito_numero: string;
    rol: string;
    acciones_comprometidas: number;
}

export interface ReporteCreditoFila {
    numero: string;
    socio_cedula: string;
    socio_nombres: string;
    monto_original: number;
    saldo_pendiente: number;
    plazo_cuotas: number;
    fecha_desembolso: string;
}

export interface ReporteLibroFila {
    fecha: string;
    operacion: string;
    ingreso: number;
    egreso: number;
}

export interface ReporteAccionFila {
    socio_cedula: string;
    socio_nombres: string;
    cantidad_acciones: number;
    valor_compra: number;
    mes_compra: string;
    fecha_compra: string;
}

export interface ReporteBalanceActual {
    efectivo: number;
    cartera: number;
    bienes: number;
    fondo_gastos: number;
    fondo_incobrables: number;
    capital_acciones: number;
    ganancias_pasivo: number;
    resultado_no_distribuido: number;
}
