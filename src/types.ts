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

// ── Socios (RF-15 a RF-21) ──

export type EstatusSocio =
  | 'ACTIVO'
  | 'RETIRADO_VOLUNTARIO'
  | 'RETIRADO_CON_DEUDA'
  | 'RETIRADO_DEUDA_SALDADA';

/** Beneficiario en caso de muerte (RF-20). El parentesco no lo pide la planilla. */
export interface Beneficiario {
  nombre: string;
  cedula: string;
  parentesco: string | null;
}

/** Protegido del fondo de protección (RF-21). Hasta 2 por socio. */
export interface Protegido {
  nombre: string;
  cedula: string;
  parentesco: string;
  telefono: string;
}

/** Fila del listado de socios: sólo lo que muestra la tabla. */
export interface SocioResumen {
  id: string;
  cedula: string;
  nombre_completo: string;
  celular: string;
  estatus: EstatusSocio;
  fecha_ingreso: string;
}

/** Detalle completo de un socio, para el formulario. */
export interface Socio {
  id: string;
  cedula: string;
  nombres: string;
  apellidos: string;
  profesion: string;
  direccion: string;
  telefono: string;
  celular: string;
  correo: string;
  estatus: EstatusSocio;
  fecha_ingreso: string;
  fecha_retiro: string | null;
  beneficiario: Beneficiario | null;
  protegidos: Protegido[];
}

/** Lo que envía el formulario. Sin id ni estatus: los gobierna el backend. */
export interface DatosSocio {
  cedula: string;
  nombres: string;
  apellidos: string;
  profesion: string;
  direccion: string;
  telefono: string;
  celular: string;
  correo: string;
  beneficiario: Beneficiario | null;
  protegidos: Protegido[];
}

/** Cupo de socios del Bankomunal (RN-01: mínimo 8, máximo 19). */
export interface CupoSocios {
  activos: number;
  disponibles: number;
  minimo: number;
  maximo: number;
}

// ── Acciones (RF-22 a RF-27) ──

export interface NuevaCompra {
  socio_id: string;
  fecha: string;
  monto: number;
}

/** Cálculo previo a cobrar: cantidad, participación resultante y tope de RN-02. */
export interface CalculoCompra {
  cantidad: number;
  valor_nominal: number;
  monto: number;
  acciones_socio_despues: number;
  total_bankomunal_despues: number;
  participacion_pct: number;
  supera_tope_participacion: boolean;
  /** RN-02 no aplica en los primeros 3 meses de operación. */
  tope_en_periodo_de_gracia: boolean;
}

export interface LoteAcciones {
  id: string;
  socio_id: string;
  mes_compra: string;
  fecha_compra: string;
  cantidad: number;
  valor_nominal_compra: number;
  monto_pagado: number;
}

/** Una fila del Control de Acciones por Socio (RF-105). */
export interface ResumenMesAcciones {
  mes: string;
  compradas: number;
  liquidadas: number;
  saldo: number;
}

export interface AccionesDeSocio {
  socio_id: string;
  acciones: number;
}

/** RN-09: autorización de venta del mes según el PPCFC. */
export type AutorizacionVenta =
  /** Faltan meses cerrados para promediar el PPCFC (D-02, pendiente del cliente). */
  | { estado: 'SinDatosSuficientes'; meses_cerrados: number }
  /** PPCFC por debajo del 80%: no se venden acciones. */
  | { estado: 'NoAutoriza'; ppcfc_pct: number }
  | {
      estado: 'Autoriza';
      ppcfc_pct: number;
      rango_desde: number;
      rango_hasta: number;
      venta_pct: number;
      cupo_acciones: number;
      cupo_monto: number;
    };

export interface CupoMensual {
  mes: string;
  autorizacion: AutorizacionVenta;
  vendido_acciones: number;
  vendido_monto: number;
  disponible_monto: number | null;
  /** RN-15: máximo que un solo socio puede tomar del cupo del mes. */
  tope_individual_monto: number | null;
}

// ── Caja y Contabilidad (RF-83 a RF-90) ──

/** Códigos del Libro de Ingresos y Egresos. Los cuatro primeros se registran a mano. */
export type CodigoOperacion =
  | 'OI' | 'EG' | 'IFG' | 'GBK'
  | 'VC' | 'LC' | 'UR' | 'CON' | 'OR' | 'PC' | 'MO' | 'PDP' | 'COR';

/** Los que el usuario puede teclear desde la pantalla de Caja. */
export const OPERACIONES_CAJA: { codigo: CodigoOperacion; etiqueta: string; ayuda: string }[] = [
  { codigo: 'OI', etiqueta: 'Otro Ingreso', ayuda: 'Entra dinero a la caja (RF-83)' },
  { codigo: 'EG', etiqueta: 'Otro Egreso', ayuda: 'Sale dinero de la caja (RF-84)' },
  { codigo: 'IFG', etiqueta: 'Ingreso al Fondo para Gastos', ayuda: 'Suma al Fondo para Gastos (RF-85)' },
  { codigo: 'GBK', etiqueta: 'Gasto del Bankomunal', ayuda: 'Se descuenta del Fondo para Gastos (RF-86)' },
];

export interface Movimiento {
  id: string;
  numero: number;
  fecha: string;
  codigo: CodigoOperacion;
  descripcion: string;
  ingreso: number;
  egreso: number;
  saldo: number;
  socio_id: string | null;
  credito_id: string | null;
  /** Si su mes ya cerró, corregirlo exige nombre y motivo (RF-90). */
  mes_cerrado: boolean;
  corregido: boolean;
  corregido_por: string | null;
  motivo_correccion: string | null;
}

export interface NuevaOperacion {
  codigo: CodigoOperacion;
  fecha: string;
  monto: number;
  descripcion: string;
}

export interface FiltroLibro {
  desde?: string | null;
  hasta?: string | null;
}

export interface ResumenCaja {
  saldo_caja: number;
  saldo_fondo_gastos: number;
  valor_activo_fijo: number;
}

export type TipoBien = 'PROPIO' | 'COMODATO';

export interface Bien {
  id: string;
  descripcion: string;
  fecha_adquisicion: string;
  valor: number;
  tipo: TipoBien;
}

export interface NuevoBien {
  descripcion: string;
  fecha_adquisicion: string;
  valor: number;
  tipo: TipoBien;
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
