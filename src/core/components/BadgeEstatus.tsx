import { EstatusSocio } from '../../features/socios/types';

/** Etiqueta legible de cada estatus del socio (RF-37, RF-76). */
const ETIQUETA: Record<EstatusSocio, string> = {
    ACTIVO: 'Activo',
    RETIRADO_VOLUNTARIO: 'Retirado Voluntario',
    RETIRADO_CON_DEUDA: 'Retirado con Deuda',
    RETIRADO_DEUDA_SALDADA: 'Retirado, Deuda Pagada',
};

/**
 * "Retirado con Deuda" se resalta porque implica plata pendiente en Incobrables
 * (RF-35): es el único estatus sobre el que hay que actuar.
 */
const CLASE: Record<EstatusSocio, string> = {
    ACTIVO: 'badge-activo',
    RETIRADO_VOLUNTARIO: 'badge-retirado',
    RETIRADO_CON_DEUDA: 'badge-deuda',
    RETIRADO_DEUDA_SALDADA: 'badge-retirado',
};

export function BadgeEstatus({ estatus }: { estatus: EstatusSocio }) {
    return <span className={`badge-estatus ${CLASE[estatus]}`}>{ETIQUETA[estatus]}</span>;
}
