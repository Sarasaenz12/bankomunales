export interface Bankomunal {
  id: string;
  nombre: string;
  ubicacion: string;
  fecha_creacion: string;
  moneda: string;
}

export interface NuevoBankomunal {
  nombre: string;
  ubicacion: string;
  moneda: string;
}

export interface Configuracion {
  valor_nominal: number;
  pct_garantia_socio: number;
  pct_garantia_fiador: number;
  /** % de las ganancias del mes retenido al Fondo para Gastos (RN-07). */
  pct_fondo_gastos: number;
  /** % de las ganancias del mes retenido al Fondo de Reserva para Incobrables (RN-08). */
  pct_fondo_incobrables: number;
  /** Tope de la Reserva acumulada, como % del capital total en acciones (RN-08). */
  tope_reserva_incobrables_pct: number;
  /** % del total de acciones autorizado a vender con el PPCFC entre 80% y 90% (RN-09). */
  ppcfc_venta_rango1_pct: number;
  /** % del total de acciones autorizado a vender con el PPCFC entre 90% y 100% (RN-09). */
  ppcfc_venta_rango2_pct: number;
  tope_individual_mensual_pct: number;
  plazo_maximo_cuotas: number;
  tasa_interes_ordinario: number;
  tasa_interes_mora: number;
  monto_maximo_credito: number;
}

export interface DatosGenerales {
  id: string;
  nombre: string;
  ubicacion: string;
  fecha_creacion: string;
  moneda: string;
  valor_nominal: number;
  numero_creditos_otorgados: number;
  monto_total_creditos: number;
  numero_acciones_vendidas: number;
  saldo_fondo_gastos: number;
  saldo_fondo_incobrables: number;
}