import { api } from '../../core/lib/api';
import { Configuracion, DatosGenerales } from '../../core/types';

export const configuracionService = {
    async cargarTodo(): Promise<{ config: Configuracion; datos: DatosGenerales }> {
        const [config, datos] = await Promise.all([
            api.obtenerConfiguracion(),
            api.obtenerDatosGenerales(),
        ]);
        return { config, datos };
    },

    guardar: (config: Configuracion, quienRealiza: string, motivo: string) =>
        api.actualizarConfiguracion(config, quienRealiza, motivo),
};