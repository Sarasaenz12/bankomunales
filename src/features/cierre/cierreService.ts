import { api } from '../../core/lib/api';
import {
    CierreMes, DesgloseRepartoMes, DetalleCierre, FilaPagoGanancia, PagoGananciaRegistrado,
    PrevisualizacionCierre, RepartoAnual, ResultadoCierre,
} from './types';

export const cierreService = {
    previsualizarCierre: (mes: string): Promise<PrevisualizacionCierre> =>
        api.previsualizarCierre(mes),

    desgloseRepartoMes: (mes: string): Promise<DesgloseRepartoMes> =>
        api.desgloseRepartoMes(mes),

    ejecutarCierre: (mes: string, nombreQuienRealiza: string): Promise<ResultadoCierre> =>
        api.ejecutarCierre(mes, nombreQuienRealiza),

    listarCierres: (): Promise<CierreMes[]> => api.listarCierres(),

    detalleCierre: (id: string): Promise<DetalleCierre> => api.detalleCierre(id),

    repartoAnual: (mes: string): Promise<RepartoAnual> => api.repartoAnual(mes),

    pagarGananciasCumplidas: (fecha: string): Promise<PagoGananciaRegistrado[]> =>
        api.pagarGananciasCumplidas(fecha),

    resumenPagosGanancias: (fecha: string): Promise<FilaPagoGanancia[]> =>
        api.resumenPagosGanancias(fecha),

    pagarGananciasDeSocio: (socioId: string, fecha: string): Promise<PagoGananciaRegistrado[]> =>
        api.pagarGananciasDeSocio(socioId, fecha),
};