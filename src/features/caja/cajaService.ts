import { api } from '../../core/lib/api';
import {
    Movimiento, ResumenCaja, Bien, FiltroLibro, CodigoOperacion, TipoBien,
} from './types';

export const cajaService = {
    /** Carga el libro (con filtro), el resumen de caja y los bienes de una vez. */
    async cargarTodo(desde: string, hasta: string): Promise<{
        libro: Movimiento[]; resumen: ResumenCaja; bienes: Bien[];
    }> {
        const filtro: FiltroLibro = { desde: desde || null, hasta: hasta || null };
        const [libro, resumen, bienes] = await Promise.all([
            api.listarLibro(filtro),
            api.resumenCaja(),
            api.listarBienes(),
        ]);
        return { libro, resumen, bienes };
    },

    registrar: (operacion: { codigo: CodigoOperacion; fecha: string; monto: number; descripcion: string }) =>
        api.registrarOperacionCaja(operacion),

    registrarDonacion: (fecha: string, monto: number, descripcion: string) =>
        api.registrarDonacion(fecha, monto, descripcion),

    corregir: (
        id: string, fecha: string, monto: number, descripcion: string,
        nombreQuienRealiza: string | null, motivo: string | null,
    ) => api.corregirOperacionCaja(id, fecha, monto, descripcion, nombreQuienRealiza, motivo),

    registrarBien: (bien: { descripcion: string; fecha: string; valor: number; tipo: TipoBien }) =>
        api.registrarBien({
            descripcion: bien.descripcion,
            fecha_adquisicion: bien.fecha,
            valor: bien.valor,
            tipo: bien.tipo,
        }),
};