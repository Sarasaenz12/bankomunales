import { api } from '../../core/lib/api';
import { DatosSocio, Socio, SocioResumen, CupoSocios } from './types';
import { AccionesDeSocio } from '../acciones/types';
import { Credito } from '../creditos/types';

export const sociosService = {
    listar: () => api.listarSocios(),

    listarActivos: async (): Promise<SocioResumen[]> => {
        const lista = await api.listarSocios();
        return lista.filter(s => s.estatus === 'ACTIVO');
    },

    cupo: () => api.cupoSocios(),

    accionesPorSocio: () => api.accionesPorSocio(),

    /** Carga el listado, el cupo de socios y las acciones por socio de una vez. */
    async cargarResumen(): Promise<{ socios: SocioResumen[]; cupo: CupoSocios; acciones: AccionesDeSocio[] }> {
        const [socios, cupo, acciones] = await Promise.all([
            api.listarSocios(),
            api.cupoSocios(),
            api.accionesPorSocio(),
        ]);
        return { socios, cupo, acciones };
    },

    obtener: (id: string): Promise<Socio> => api.obtenerSocio(id),

    crear: (datos: DatosSocio) => api.registrarSocio(datos),

    actualizar: (id: string, datos: DatosSocio) => api.actualizarSocio(id, datos),

    accionesDeSocio: (id: string) => api.accionesDeSocio(id),

    historialAcciones: (id: string) => api.historialAccionesSocio(id),

    /** Historial de créditos desembolsados de los que el socio es titular (RF-19). */
    creditosDeSocio: (id: string): Promise<Credito[]> => api.creditosDeSocio(id),

    /** Créditos de terceros que este socio avala como fiador (RF-48, RN-04). */
    creditosQueAvala: (id: string): Promise<Credito[]> => api.creditosQueAvalaSocio(id),
};