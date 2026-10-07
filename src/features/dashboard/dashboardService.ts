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

    async cargarResumenGeneral(): Promise<{ datos: DatosGenerales; auditoria: any[] }> {
        const hoy = new Date();
        const mes = String(hoy.getMonth() + 1).padStart(2, '0'); // getMonth es 0-indexado
        const anno = hoy.getFullYear();
        const fechaCierre = `${anno}-${mes}-01`;
        const [datos, informe] = await Promise.all([
            api.obtenerDatosGenerales(),
            api.previsualizarCierre(fechaCierre),
        ]);
        // Merge informe data into datos if available
        if (informe?.informe) {
            const i = informe.informe;
            return {
                datos: {
                    ...datos,
                    socios_activos: i.socios_activos ?? datos.socios_activos,
                    creditos_vigentes: i.creditos_vigentes ?? datos.creditos_vigentes,
                },
                auditoria: informe.cuadre ? [informe.cuadre] : [],
            };
        }
        return { datos, auditoria: [] };
    },
};
