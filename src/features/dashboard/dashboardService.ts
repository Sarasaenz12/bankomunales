import { api } from '../../core/lib/api';
import { DatosGenerales } from '../../core/types';

export const dashboardService = {
    async cargarDatos(): Promise<{ datos: DatosGenerales; auditoria: any[] }> {
        const [datos, auditoria] = await Promise.all([
            api.obtenerDatosGenerales(),
            api.listarAuditoria(),
        ]);
        return { datos, auditoria: auditoria.slice(0, 5) };
    },
};