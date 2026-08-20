import { SocioResumen } from '../socios/types';

// ── Créditos (RF-43 a RF-82) ──

/** Catálogo de clases de crédito (RN-11, D-15). */
export type DestinoCredito =
  | 'AH' | 'ARE' | 'CVR' | 'CV' | 'ED'
  | 'GP' | 'OT' | 'PR' | 'SL' | 'SP';

export const DESTINOS_CREDITO: { codigo: DestinoCredito; etiqueta: string }[] = [
  { codigo: 'AH', etiqueta: 'Artículos de Hogar' },
  { codigo: 'ARE', etiqueta: 'Equipos de Trabajo' },
  { codigo: 'CVR', etiqueta: 'Vivienda' },
  { codigo: 'CV', etiqueta: 'Compra y Venta' },
  { codigo: 'ED', etiqueta: 'Educación' },
  { codigo: 'GP', etiqueta: 'Gastos Personales' },
  { codigo: 'OT', etiqueta: 'Otros' },
  { codigo: 'PR', etiqueta: 'Productivo' },
  { codigo: 'SL', etiqueta: 'Salud' },
  { codigo: 'SP', etiqueta: 'Servicios Públicos' },
];

/** Decisión de la Junta sobre una solicitud (RF-50). */
export type EstadoSolicitud =
  | 'PENDIENTE' | 'APROBADA' | 'MODIFICADA' | 'NEGADA' | 'DIFERIDA';

export const ETIQUETA_ESTADO: Record<EstadoSolicitud, string> = {
  PENDIENTE: 'Pendiente',
  APROBADA: 'Aprobada',
  MODIFICADA: 'Modificada',
  NEGADA: 'Negada',
  DIFERIDA: 'Diferida',
};

export type EstadoCredito = 'VIGENTE' | 'PAGADO' | 'REFINANCIADO';

export type RolGarantia = 'TITULAR' | 'FIADOR';

/** Fiador propuesto en una solicitud: se identifica por cédula (RF-48). */
export interface FiadorSolicitud {
  cedula: string;
  acciones_comprometidas: number;
}

/** Lo que envía la pantalla "Nuevo Crédito" (RF-43, RF-45, RF-47, RF-48). */
export interface NuevaSolicitud {
  socio_id: string;
  monto_solicitado: number;
  plazo_cuotas: number;
  destino: DestinoCredito;
  total_ingresos: number;
  total_egresos: number;
  fiadores: FiadorSolicitud[];
}

/** Decisión de la Junta (RF-50/RF-51). */
export interface DecisionSolicitud {
  solicitud_id: string;
  decision: EstadoSolicitud;
  monto_aprobado?: number | null;
  observacion?: string | null;
  decidida_por: string;
}

/** Datos para desembolsar un crédito (RF-44, RF-54). */
export interface NuevoDesembolso {
  solicitud_id?: string | null;
  socio_id: string;
  monto: number;
  plazo_cuotas: number;
  destino: DestinoCredito;
  fiadores: FiadorSolicitud[];
  fecha?: string | null;
}

/** Garantía propuesta de una solicitud (RF-48). */
export interface GarantiaSolicitud {
  socio_id: string;
  rol: RolGarantia;
  acciones_comprometidas: number;
}

/** Una solicitud de crédito (RF-43 a RF-52). */
export interface SolicitudCredito {
  id: string;
  socio_id: string;
  fecha_solicitud: string;
  monto_solicitado: number;
  plazo_cuotas: number;
  destino: DestinoCredito;
  total_ingresos: number;
  total_egresos: number;
  capacidad_pago: number;
  estado: EstadoSolicitud;
  monto_aprobado: number | null;
  observacion: string | null;
  fecha_decision: string | null;
  decidida_por: string | null;
  garantias: GarantiaSolicitud[];
}

/** Cuota calculada con saldo decreciente (RF-46, RF-61, RN-12). */
export interface CuotaPlaneada {
  numero: number;
  fecha_vencimiento: string;
  capital: number;
  interes: number;
  valor_total: number;
}

/** Vista previa de un desembolso (RF-46). */
export interface TablaCredito {
  cuotas: CuotaPlaneada[];
  monto_total: number;
  cuota_mensual: number;
  capital_cuota: number;
  interes_cuota: number;
}

/** Garantía efectiva de un crédito desembolsado (RF-57, RN-04). */
export interface GarantiaCredito {
  socio_id: string;
  rol: RolGarantia;
  acciones_comprometidas: number;
}

/** Un crédito desembolsado (RF-53 a RF-62). */
export interface Credito {
  id: string;
  socio_id: string;
  numero: string;
  monto_original: number;
  tasa: number;
  plazo_cuotas: number;
  cuota_actual: number;
  saldo_pendiente: number;
  destino: DestinoCredito;
  estatus: EstadoCredito;
  fecha_solicitud: string;
  fecha_desembolso: string;
  frecuencia_pago: string;
  fecha_vencimiento: string;
  solicitud_id: string | null;
  garantias: GarantiaCredito[];
}

export type { SocioResumen };