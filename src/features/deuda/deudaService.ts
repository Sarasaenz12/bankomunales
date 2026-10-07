import { api } from '../../core/lib/api';
import { Credito } from '../creditos/types';
import { EstadoCreditoDeuda, ResultadoAbono, ResultadoPago } from './types';

export const deudaService = {
    estadoCredito: (numero: string): Promise<EstadoCreditoDeuda> =>
        api.estadoCreditoDeuda(numero),

    /** Todos los créditos, para buscar/autocompletar por nombre o número. */
    listarCreditos: (): Promise<Credito[]> =>
        api.listarCreditos(),

    // El backend `registrar_pago_cuota` ya asienta cada concepto en el Libro (OR/PC/MO)
    // por Caja. El frontend NO debe volver a asentar el pago: hacerlo intentaría un doble
    // registro (que Caja rechaza por no ser un código registrable a mano) y además rompía
    // el pago múltiple. Un único punto de escritura del Libro (ver cuadre Activo=Pasivo).
    registrarPagoCuota: (
        numeroCredito: string,
        numeroCuota: number,
        fecha: string,
        monto: number,
    ): Promise<ResultadoPago> =>
        api.registrarPagoCuota(numeroCredito, numeroCuota, fecha, monto),

    /** Paga una o varias cuotas: un solo interés (el del mes) + capital de las N cuotas.
     *  Con varias cuotas el capital re-amortiza el saldo (quedan menos cuotas). */
    pagarCuotas: (
        numeroCredito: string,
        fecha: string,
        cantidad: number,
    ): Promise<ResultadoPago> =>
        api.pagarCuotas(numeroCredito, fecha, cantidad),

    /** Abona un monto a capital: baja el saldo y re-amortiza las cuotas pendientes. */
    abonarCapital: (
        numeroCredito: string,
        fecha: string,
        monto: number,
    ): Promise<ResultadoAbono> =>
        api.abonarCapital(numeroCredito, fecha, monto),
};
