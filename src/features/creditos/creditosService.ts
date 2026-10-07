import { api } from '../../core/lib/api';
import { Configuracion } from '../../core/types';
import { SocioResumen } from '../socios/types';
import {
    NuevaSolicitud, SolicitudCredito, DecisionSolicitud, NuevoDesembolso,
    Credito, TablaCredito, DestinoCredito, EstadoSolicitud, FiadorSolicitud,
} from './types';

export interface ResultadoValidacion { ok: boolean; mensaje: string; }

/** RN-03 (RF-56): relación 1 a 5 — el monto no puede superar 5 veces el valor
 * de las acciones que el socio posee realmente (acciones × valor nominal).
 *
 * Estas comprobaciones son sólo para el feedback inmediato de la pantalla; la
 * validación autoritativa la hace el backend al enviar. Exigen la `Configuracion`
 * real cargada — sin valores por defecto que puedan mentir si la config difiere. */
export function maximoRn03(accionesReales: number, config: Configuracion): number {
    return 5 * accionesReales * config.valor_nominal;
}

export function cumpleRn03(monto: number, accionesReales: number, config: Configuracion): boolean {
    return monto > 0 && monto <= maximoRn03(accionesReales, config);
}

/** RN-04 (RF-49): garantía por partes. El solicitante cubre con sus acciones reales
 * el pct_garantia_socio% y los fiadores, con las comprometidas, el pct_garantia_fiador%. */
export function cumpleRn04Socio(monto: number, accionesReales: number, config: Configuracion): boolean {
    return accionesReales * config.valor_nominal >= monto * config.pct_garantia_socio / 100;
}

export function cumpleRn04Fiadores(
    monto: number, accionesComprometidas: number, config: Configuracion,
): boolean {
    return accionesComprometidas * config.valor_nominal >= monto * config.pct_garantia_fiador / 100;
}

/** Validación de una solicitud nueva o desembolso directo (RN-03, RN-04, RN-14). */
export function validarNuevaSolicitud(params: {
    solicitante: SocioResumen | null;
    monto: number;
    plazo: number;
    destino: DestinoCredito | '';
    fiadores: FiadorSolicitud[];
    accionesSolicitante: number;
    accionesComprometidas: number;
    config: Configuracion;
}): ResultadoValidacion {
    const {
        solicitante, monto, plazo, destino, fiadores, accionesSolicitante,
        accionesComprometidas, config,
    } = params;

    if (!solicitante) return { ok: false, mensaje: 'Debe seleccionar el socio solicitante' };
    if (!destino) return { ok: false, mensaje: 'Debe seleccionar el destino del crédito' };
    if (!monto || monto <= 0) return { ok: false, mensaje: 'El monto solicitado debe ser mayor a cero' };
    if (!plazo || plazo <= 0) return { ok: false, mensaje: 'El plazo debe ser de al menos 1 cuota' };
    if (fiadores.length === 0) return { ok: false, mensaje: 'Todo crédito debe tener al menos un fiador' };
    if (fiadores.length > 2) return { ok: false, mensaje: 'Se registran hasta 2 fiadores' };

    if (!cumpleRn03(monto, accionesSolicitante, config)) {
        return { ok: false, mensaje: `El monto supera el cupo de la relación 1 a 5: máx. ${maximoRn03(accionesSolicitante, config)}` };
    }
    if (!cumpleRn04Socio(monto, accionesSolicitante, config)) {
        return { ok: false, mensaje: `El solicitante no cubre su ${config.pct_garantia_socio}% con sus acciones` };
    }
    if (!cumpleRn04Fiadores(monto, accionesComprometidas, config)) {
        return { ok: false, mensaje: `Los fiadores no cubren su ${config.pct_garantia_fiador}% con las acciones comprometidas` };
    }

    return { ok: true, mensaje: '' };
}

export const creditosService = {
    obtenerConfiguracion: () => api.obtenerConfiguracion(),

    listarSociosActivos: async (): Promise<SocioResumen[]> => {
        const lista = await api.listarSocios();
        return lista.filter(s => s.estatus === 'ACTIVO');
    },

    accionesDeSocio: (socioId: string) => api.accionesDeSocio(socioId),

    previsualizarTabla: (monto: number, plazo: number): Promise<TablaCredito> =>
        api.previsualizarTablaCredito(monto, plazo),

    previsualizarDesembolso: (monto: number, plazo: number): Promise<TablaCredito> =>
        api.previsualizarDesembolso(monto, plazo),

    registrarSolicitud: (solicitud: NuevaSolicitud): Promise<SolicitudCredito> =>
        api.registrarSolicitud(solicitud),

    decidir: (decision: DecisionSolicitud): Promise<SolicitudCredito> =>
        api.decidirSolicitud(decision),

    listarSolicitudes: (estado?: EstadoSolicitud | null): Promise<SolicitudCredito[]> =>
        api.listarSolicitudes(estado),

    listarSolicitudesDesembolsables: (): Promise<SolicitudCredito[]> =>
        api.listarSolicitudesDesembolsables(),

    registrarDesembolso: (desembolso: NuevoDesembolso): Promise<Credito> =>
        api.registrarDesembolso(desembolso),

    buscarCreditoPorSolicitud: (solicitudId: string): Promise<Credito | null> =>
        api.buscarCreditoPorSolicitud(solicitudId),

    siguienteNumeroCredito: (): Promise<string> =>
        api.siguienteNumeroCredito(),
};