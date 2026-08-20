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