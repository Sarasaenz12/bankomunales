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