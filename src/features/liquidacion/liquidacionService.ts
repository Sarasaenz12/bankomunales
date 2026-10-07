import { api } from '../../core/lib/api';
import {
    DatosLiquidacion, DatosLiquidacionSocio, DestinoLiquidacionParcial, LiquidacionFinal,
    ParametrosLiquidacion, PreviewLiquidacionParcial, ResultadoLiquidacionParcial, SocioLiquidacion,
} from './types';

export const liquidacionService = {
    prepararLiquidacionFinal: (): Promise<DatosLiquidacion> => api.prepararLiquidacionFinal(),

    calcularLiquidacionFinal: (
        parametros: ParametrosLiquidacion,
        socios: SocioLiquidacion[],
    ): Promise<LiquidacionFinal> => api.calcularLiquidacionFinal(parametros, socios),

    prepararLiquidacionSocio: (socioId: string): Promise<DatosLiquidacionSocio> =>
        api.prepararLiquidacionSocio(socioId),

    ejecutarRetiroSocio: (datos: { socio_id: string, fecha: string }) =>
        api.ejecutarRetiroSocio(datos),

    previsualizarLiquidacionParcial: (
        socioId: string, cantidad: number, fecha: string,
    ): Promise<PreviewLiquidacionParcial> =>
        api.previsualizarLiquidacionParcial(socioId, cantidad, fecha),

    ejecutarLiquidacionParcial: (datos: {
        socio_id: string, cantidad: number, fecha: string, destino: DestinoLiquidacionParcial,
    }): Promise<ResultadoLiquidacionParcial> =>
        api.ejecutarLiquidacionParcial(datos),
};
