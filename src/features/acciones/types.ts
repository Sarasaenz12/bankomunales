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
  monto_pagado: number;
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