import { api } from '../../core/lib/api';
import { DatosGenerales, EntradaAuditoria } from '../../core/types';

export const dashboardService = {
    async cargarDatos(): Promise<{ datos: DatosGenerales; auditoria: EntradaAuditoria[] }> {
        const [datos, auditoria] = await Promise.all([
            api.obtenerDatosGenerales(),
            api.listarAuditoria(),
        ]);
        return { datos, auditoria: auditoria.slice(0, 5) };
    },

    /** Resumen del mes en curso: los datos generales con los conteos del informe de cierre. */
    async cargarResumenGeneral(): Promise<{ datos: DatosGenerales }> {
        const hoy = new Date();
        const mes = String(hoy.getMonth() + 1).padStart(2, '0'); // getMonth es 0-indexado
        const fechaCierre = `${hoy.getFullYear()}-${mes}-01`;
        const [datos, informe] = await Promise.all([
            api.obtenerDatosGenerales(),
            api.previsualizarCierre(fechaCierre),
        ]);
        const i = informe?.informe;
        if (!i) return { datos };
        return {
            datos: {
                ...datos,
                socios_activos: i.socios_activos ?? datos.socios_activos,
                creditos_vigentes: i.creditos_vigentes ?? datos.creditos_vigentes,
            },
        };
    },
};
