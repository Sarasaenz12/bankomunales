import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import {
    Bankomunal, NuevoBankomunal, Configuracion, DatosGenerales, EntradaAuditoria,
} from '../types';
import { Socio, SocioResumen, DatosSocio, CupoSocios } from '../../features/socios/types';
import {
    Movimiento, NuevaOperacion, FiltroLibro, ResumenCaja, Bien, NuevoBien,
} from '../../features/caja/types';
import {
    LoteAcciones, ResumenMesAcciones, AccionesDeSocio, CupoMensual,
    NuevaCompra, CalculoCompra,
} from '../../features/acciones/types';
import {
    NuevaSolicitud, SolicitudCredito, DecisionSolicitud, NuevoDesembolso,
    Credito, TablaCredito, EstadoSolicitud, NuevoRefinanciamiento,
} from '../../features/creditos/types';
import {
    CierreMes, DesgloseRepartoMes, DetalleCierre, FilaPagoGanancia, PagoGananciaRegistrado,
    PrevisualizacionCierre, RepartoAnual, ResultadoCierre,
} from '../../features/cierre/types';
import {
    DatosLiquidacion, DatosLiquidacionSocio, DestinoLiquidacionParcial, LiquidacionFinal,
    ParametrosLiquidacion, PreviewLiquidacionParcial, ResultadoLiquidacionParcial, SocioLiquidacion,
} from '../../features/liquidacion/types';
import { EstadoCreditoDeuda, ResultadoAbono, ResultadoPago, SocioDeudor } from '../../features/deuda/types';
import { RegistroRespaldo, VistaRespaldo } from '../../features/respaldo/types';
import {
    ReporteSocioActivo, ReporteGestionFila, ReporteFiadorFila, ReporteCreditoFila,
    ReporteLibroFila, ReporteAccionFila, ReporteBalanceActual,
} from '../../features/reportes/types';

/**
 * Error de dominio del backend. Se serializa como `{ code, message }`:
 * `code` es estable y permite ramificar por tipo (p. ej. `NOMBRE_DUPLICADO`);
 * `message` es el texto legible. `toString()` devuelve el mensaje para conservar
 * la compatibilidad con el código que ya hace `err?.toString()`.
 */
export class AppError {
    constructor(
        public readonly code: string,
        public readonly message: string,
    ) {}
    toString(): string {
        return this.message;
    }
}

/**
 * Deja el mensaje listo para mostrarse al usuario: quita las referencias internas de
 * reglamento —(RN-03), (RF-56), (D-02), (RNF-08)…— y pone separador de miles a los montos
 * (`$2000000.00` → `$2.000.000`), sin centavos.
 */
function limpiarMensaje(m: string): string {
    return m
        .replace(/\s*\((?:RN|RF|RNF|D)-[\d/]+(?:\s*,\s*(?:RN|RF|RNF|D)-[\d/]+)*\)/g, '')
        .replace(/\$\s?(\d+)(?:\.\d+)?/g, (_m, d: string) => '$' + d.replace(/\B(?=(\d{3})+(?!\d))/g, '.'))
        .replace(/\s{2,}/g, ' ')
        .trim();
}

function normalizarError(e: unknown): AppError {
    if (e && typeof e === 'object' && 'code' in e && 'message' in e) {
        const o = e as { code: unknown; message: unknown };
        return new AppError(String(o.code), limpiarMensaje(String(o.message)));
    }
    // Compatibilidad con errores que aún llegaran como texto plano.
    return new AppError('DESCONOCIDO', limpiarMensaje(typeof e === 'string' ? e : String(e)));
}

/** Envuelve `invoke` para que todo rechazo llegue como `AppError` tipado. */
function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return tauriInvoke<T>(cmd, args).catch((e) => {
        throw normalizarError(e);
    });
}

export const api = {
    // Auth & Bankomunal Selection
    login: (password: string): Promise<boolean> =>
        call('login', { password }),

    crearBankomunal: (nuevo: NuevoBankomunal): Promise<Bankomunal> =>
        call('crear_bankomunal', { nuevo }),

    listarBankomunales: (): Promise<Bankomunal[]> =>
        call('listar_bankomunales'),

    unicoBankomunal: (): Promise<Bankomunal | null> =>
        call('unico_bankomunal'),

    seleccionarBankomunal: (id: string): Promise<Bankomunal> =>
        call('seleccionar_bankomunal', { id }),

    volverASeleccion: (): Promise<void> =>
        call('volver_a_seleccion'),

    bancoSeleccionado: (): Promise<Bankomunal | null> =>
        call('banco_seleccionado'),

    sesionIniciada: (): Promise<boolean> =>
        call('sesion_iniciada'),

    // Configuración
    obtenerConfiguracion: (): Promise<Configuracion> =>
        call('obtener_configuracion'),

    actualizarConfiguracion: (configuracion: Configuracion, nombreQuienRealiza: string, motivo: string): Promise<Configuracion> =>
        call('actualizar_configuracion', { configuracion, nombreQuienRealiza, motivo }),

    obtenerDatosGenerales: (): Promise<DatosGenerales> =>
        call('obtener_datos_generales'),

    listarAuditoria: (): Promise<EntradaAuditoria[]> =>
        call('listar_auditoria'),

    // Socios (RF-15 a RF-21)
    registrarSocio: (datos: DatosSocio): Promise<Socio> =>
        call('registrar_socio', { datos }),

    actualizarSocio: (id: string, datos: DatosSocio): Promise<Socio> =>
        call('actualizar_socio', { id, datos }),

    obtenerSocio: (id: string): Promise<Socio> =>
        call('obtener_socio', { id }),

    buscarSocioPorCedula: (cedula: string): Promise<Socio | null> =>
        call('buscar_socio_por_cedula', { cedula }),

    listarSocios: (): Promise<SocioResumen[]> =>
        call('listar_socios'),

    cupoSocios: (): Promise<CupoSocios> =>
        call('cupo_socios'),

    // Caja y Contabilidad (RF-83 a RF-90)
    registrarOperacionCaja: (operacion: NuevaOperacion): Promise<Movimiento> =>
        call('registrar_operacion_caja', { operacion }),

    registrarDonacion: (fecha: string, monto: number, descripcion: string): Promise<Movimiento> =>
        call('registrar_donacion', { fecha, monto, descripcion }),

    corregirOperacionCaja: (
        id: string, fecha: string, monto: number, descripcion: string,
        nombreQuienRealiza?: string | null, motivo?: string | null,
    ): Promise<Movimiento> =>
        call('corregir_operacion_caja', {
            id, fecha, monto, descripcion,
            nombreQuienRealiza: nombreQuienRealiza ?? null,
            motivo: motivo ?? null,
        }),

    listarLibro: (filtro?: FiltroLibro): Promise<Movimiento[]> =>
        call('listar_libro', { filtro: filtro ?? null }),

    resumenCaja: (): Promise<ResumenCaja> =>
        call('resumen_caja'),

    registrarBien: (bien: NuevoBien): Promise<Bien> =>
        call('registrar_bien', { bien }),

    listarBienes: (): Promise<Bien[]> =>
        call('listar_bienes'),

    // Acciones (RF-22 a RF-27)
    previsualizarCompraAcciones: (socioId: string, monto: number): Promise<CalculoCompra> =>
        call('previsualizar_compra_acciones', { socioId, monto }),

    registrarCompraAcciones: (compra: NuevaCompra): Promise<LoteAcciones> =>
        call('registrar_compra_acciones', { compra }),

    accionesDeSocio: (socioId: string): Promise<number> =>
        call('acciones_de_socio', { socioId }),

    accionesPorSocio: (): Promise<AccionesDeSocio[]> =>
        call('acciones_por_socio'),

    totalAcciones: (): Promise<number> =>
        call('total_acciones'),

    historialAccionesSocio: (socioId: string): Promise<ResumenMesAcciones[]> =>
        call('historial_acciones_socio', { socioId }),

    cupoDelMes: (fecha?: string): Promise<CupoMensual> =>
        call('cupo_del_mes', { fecha: fecha ?? null }),

    // Créditos (RF-43 a RF-62)
    previsualizarTablaCredito: (monto: number, plazo: number): Promise<TablaCredito> =>
        call('previsualizar_tabla_credito', { monto, plazo }),

    registrarSolicitud: (solicitud: NuevaSolicitud): Promise<SolicitudCredito> =>
        call('registrar_solicitud', { solicitud }),

    decidirSolicitud: (decision: DecisionSolicitud): Promise<SolicitudCredito> =>
        call('decidir_solicitud', { decision }),

    listarSolicitudes: (estado?: EstadoSolicitud | null): Promise<SolicitudCredito[]> =>
        call('listar_solicitudes', { estado: estado ?? null }),

    listarSolicitudesDesembolsables: (): Promise<SolicitudCredito[]> =>
        call('listar_solicitudes_desembolsables'),

    previsualizarDesembolso: (monto: number, plazo: number): Promise<TablaCredito> =>
        call('previsualizar_desembolso', { monto, plazo }),

    registrarDesembolso: (desembolso: NuevoDesembolso): Promise<Credito> =>
        call('registrar_desembolso', { desembolso }),

    buscarCreditoPorSolicitud: (solicitudId: string): Promise<Credito | null> =>
        call('buscar_credito_por_solicitud', { solicitudId }),

    siguienteNumeroCredito: (): Promise<string> =>
        call('siguiente_numero_credito'),

    listarCreditos: (): Promise<Credito[]> =>
        call('listar_creditos'),

    creditosDeSocio: (socioId: string): Promise<Credito[]> =>
        call('creditos_de_socio', { socioId }),

    creditosQueAvalaSocio: (socioId: string): Promise<Credito[]> =>
        call('creditos_que_avala_socio', { socioId }),

    registrarRefinanciamiento: (refinanciamiento: NuevoRefinanciamiento): Promise<Credito> =>
        call('registrar_refinanciamiento', { refinanciamiento }),

    // Cierre de Mes (RF-91 a RF-97) y Reparto de Ganancias (RF-39 a RF-42)
    previsualizarCierre: (mes: string): Promise<PrevisualizacionCierre> =>
        call('previsualizar_cierre', { mes }),

    desgloseRepartoMes: (mes: string): Promise<DesgloseRepartoMes> =>
        call('desglose_reparto_mes', { mes }),

    ejecutarCierre: (mes: string, nombreQuienRealiza: string): Promise<ResultadoCierre> =>
        call('ejecutar_cierre', { mes, nombreQuienRealiza }),

    listarCierres: (): Promise<CierreMes[]> =>
        call('listar_cierres'),

    detalleCierre: (id: string): Promise<DetalleCierre> =>
        call('detalle_cierre', { id }),

    repartoAnual: (mes: string): Promise<RepartoAnual> =>
        call('reparto_anual', { mes }),

    pagarGananciasCumplidas: (fecha: string): Promise<PagoGananciaRegistrado[]> =>
        call('pagar_ganancias_cumplidas', { fecha }),

    resumenPagosGanancias: (fecha: string): Promise<FilaPagoGanancia[]> =>
        call('resumen_pagos_ganancias', { fecha }),

    pagarGananciasDeSocio: (socioId: string, fecha: string): Promise<PagoGananciaRegistrado[]> =>
        call('pagar_ganancias_de_socio', { socioId, fecha }),

    // Liquidación Final del Bankomunal (disolución)
    prepararLiquidacionFinal: (): Promise<DatosLiquidacion> =>
        call('preparar_liquidacion_final'),

    calcularLiquidacionFinal: (
        parametros: ParametrosLiquidacion,
        socios: SocioLiquidacion[],
    ): Promise<LiquidacionFinal> =>
        call('calcular_liquidacion_final', { parametros, socios }),

    prepararLiquidacionSocio: (socioId: string): Promise<DatosLiquidacionSocio> =>
        call('preparar_liquidacion_socio', { socioId }),

    ejecutarRetiroSocio: (datos: { socio_id: string; fecha: string }): Promise<void> =>
        call('ejecutar_retiro_socio', { datos }),

    previsualizarLiquidacionParcial: (
        socioId: string, cantidad: number, fecha: string,
    ): Promise<PreviewLiquidacionParcial> =>
        call('previsualizar_liquidacion_parcial', { socioId, cantidad, fecha }),

    ejecutarLiquidacionParcial: (datos: {
        socio_id: string; cantidad: number; fecha: string; destino: DestinoLiquidacionParcial;
    }): Promise<ResultadoLiquidacionParcial> =>
        call('ejecutar_liquidacion_parcial', { datos }),

    listarRetiradosConDeuda: (): Promise<SocioDeudor[]> =>
        call('listar_retirados_con_deuda'),

    registrarPagoDeudaPendiente: (datos: { socio_id: string; fecha: string; monto: number }): Promise<number> =>
        call('registrar_pago_deuda_pendiente', { datos }),

    // Servicio de la Deuda (pago de cuotas, RF-63..72)
    estadoCreditoDeuda: (numero: string): Promise<EstadoCreditoDeuda> =>
        call('estado_credito_deuda', { numero }),

    registrarPagoCuota: (
        numeroCredito: string,
        numeroCuota: number,
        fecha: string,
        monto: number,
    ): Promise<ResultadoPago> =>
        call('registrar_pago_cuota', { numeroCredito, numeroCuota, fecha, monto }),

    /** Pago de una o varias cuotas: un solo interés (el del mes) + mora + capital de las N cuotas. */
    pagarCuotas: (
        numeroCredito: string,
        fecha: string,
        cantidad: number,
    ): Promise<ResultadoPago> =>
        call('pagar_cuotas', { numeroCredito, fecha, cantidad }),

    /** Abono a capital: baja el saldo y re-amortiza las cuotas pendientes. */
    abonarCapital: (
        numeroCredito: string,
        fecha: string,
        monto: number,
    ): Promise<ResultadoAbono> =>
        call('abonar_capital', { numeroCredito, fecha, monto }),

    // Respaldo y restauración (RF-113)
    listarRespaldos: (): Promise<RegistroRespaldo[]> =>
        call('listar_respaldos_cmd'),

    crearRespaldo: (destinoRuta: string, usuario: string): Promise<void> =>
        call('crear_respaldo_cmd', { destinoRuta, usuario }),

    restaurarRespaldo: (origenRuta: string, usuario: string): Promise<void> =>
        call('restaurar_respaldo_cmd', { origenRuta, usuario }),

    inspeccionarRespaldo: (ruta: string): Promise<VistaRespaldo> =>
        call('inspeccionar_respaldo_cmd', { ruta }),

    importarRespaldo: (ruta: string, usuario: string): Promise<VistaRespaldo> =>
        call('importar_respaldo_cmd', { ruta, usuario }),

    /** Respaldo con el que se abrió la app desde el Explorador, mientras siga pendiente. */
    archivoPendiente: (): Promise<string | null> =>
        call('archivo_pendiente_cmd'),

    olvidarArchivoPendiente: (): Promise<void> =>
        call('olvidar_archivo_pendiente_cmd'),

    // Reportes (RF-98..RF-111)
    reporteSociosActivos: (): Promise<ReporteSocioActivo[]> =>
        call('reporte_socios_activos'),
    reporteGestion: (): Promise<ReporteGestionFila[]> =>
        call('reporte_gestion'),
    reporteFiadores: (): Promise<ReporteFiadorFila[]> =>
        call('reporte_fiadores'),
    reporteCreditos: (): Promise<ReporteCreditoFila[]> =>
        call('reporte_creditos'),
    reporteLibro: (): Promise<ReporteLibroFila[]> =>
        call('reporte_libro'),
    reporteAcciones: (): Promise<ReporteAccionFila[]> =>
        call('reporte_acciones'),
    reporteBalance: (): Promise<ReporteBalanceActual> =>
        call('reporte_balance'),
};