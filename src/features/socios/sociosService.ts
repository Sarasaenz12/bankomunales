import { api } from '../../core/lib/api';
import { DatosSocio, Socio, SocioResumen, CupoSocios } from './types';
import { AccionesDeSocio } from '../acciones/types';

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
};