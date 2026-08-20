import { api } from '../../core/lib/api';
import { SocioResumen } from '../socios/types';
import { CupoMensual, CalculoCompra } from './types';

export const accionesService = {
    listarSociosActivos: async (): Promise<SocioResumen[]> => {
        const lista = await api.listarSocios();
        return lista.filter(s => s.estatus === 'ACTIVO');
    },

    cupoDelMes: (fecha: string): Promise<CupoMensual> => api.cupoDelMes(fecha),

    previsualizarCompra: (socioId: string, monto: number): Promise<CalculoCompra> =>
        api.previsualizarCompraAcciones(socioId, monto),

    registrarCompra: (socioId: string, fecha: string, monto: number) =>
        api.registrarCompraAcciones({ socio_id: socioId, fecha, monto }),
};