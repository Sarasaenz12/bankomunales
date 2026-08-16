import { invoke } from '@tauri-apps/api/core';
import {
    Bankomunal, NuevoBankomunal, Configuracion, DatosGenerales,
    Socio, SocioResumen, DatosSocio, CupoSocios,
    Movimiento, NuevaOperacion, FiltroLibro, ResumenCaja, Bien, NuevoBien,
    NuevaCompra, CalculoCompra, LoteAcciones, ResumenMesAcciones, AccionesDeSocio,
    CupoMensual,
} from '../types';

export const api = {
    // Auth & Bankomunal Selection
    login: (password: string): Promise<boolean> =>
        invoke('login', { password }),

    crearBankomunal: (nuevo: NuevoBankomunal): Promise<Bankomunal> =>
        invoke('crear_bankomunal', { nuevo }),

    listarBankomunales: (): Promise<Bankomunal[]> =>
        invoke('listar_bankomunales'),

    unicoBankomunal: (): Promise<Bankomunal | null> =>
        invoke('unico_bankomunal'),

    seleccionarBankomunal: (id: string): Promise<Bankomunal> =>
        invoke('seleccionar_bankomunal', { id }),

    volverASeleccion: (): Promise<void> =>
        invoke('volver_a_seleccion'),

    bancoSeleccionado: (): Promise<Bankomunal | null> =>
        invoke('banco_seleccionado'),

    // Configuración
    obtenerConfiguracion: (): Promise<Configuracion> =>
        invoke('obtener_configuracion'),

    actualizarConfiguracion: (configuracion: Configuracion, nombreQuienRealiza: string, motivo: string): Promise<Configuracion> =>
        invoke('actualizar_configuracion', { configuracion, nombreQuienRealiza, motivo }),

    obtenerDatosGenerales: (): Promise<DatosGenerales> =>
        invoke('obtener_datos_generales'),

    listarAuditoria: (): Promise<any[]> =>
        invoke('listar_auditoria'),

    // Socios (RF-15 a RF-21)
    registrarSocio: (datos: DatosSocio): Promise<Socio> =>
        invoke('registrar_socio', { datos }),

    actualizarSocio: (id: string, datos: DatosSocio): Promise<Socio> =>
        invoke('actualizar_socio', { id, datos }),

    obtenerSocio: (id: string): Promise<Socio> =>
        invoke('obtener_socio', { id }),

    buscarSocioPorCedula: (cedula: string): Promise<Socio | null> =>
        invoke('buscar_socio_por_cedula', { cedula }),

    listarSocios: (): Promise<SocioResumen[]> =>
        invoke('listar_socios'),

    cupoSocios: (): Promise<CupoSocios> =>
        invoke('cupo_socios'),

    // Caja y Contabilidad (RF-83 a RF-90)
    registrarOperacionCaja: (operacion: NuevaOperacion): Promise<Movimiento> =>
        invoke('registrar_operacion_caja', { operacion }),

    registrarDonacion: (fecha: string, monto: number, descripcion: string): Promise<Movimiento> =>
        invoke('registrar_donacion', { fecha, monto, descripcion }),

    corregirOperacionCaja: (
        id: string, fecha: string, monto: number, descripcion: string,
        nombreQuienRealiza?: string | null, motivo?: string | null,
    ): Promise<Movimiento> =>
        invoke('corregir_operacion_caja', {
            id, fecha, monto, descripcion,
            nombreQuienRealiza: nombreQuienRealiza ?? null,
            motivo: motivo ?? null,
        }),

    listarLibro: (filtro?: FiltroLibro): Promise<Movimiento[]> =>
        invoke('listar_libro', { filtro: filtro ?? null }),

    resumenCaja: (): Promise<ResumenCaja> =>
        invoke('resumen_caja'),

    registrarBien: (bien: NuevoBien): Promise<Bien> =>
        invoke('registrar_bien', { bien }),

    listarBienes: (): Promise<Bien[]> =>
        invoke('listar_bienes'),

    // Acciones (RF-22 a RF-27)
    previsualizarCompraAcciones: (socioId: string, monto: number): Promise<CalculoCompra> =>
        invoke('previsualizar_compra_acciones', { socioId, monto }),

    registrarCompraAcciones: (compra: NuevaCompra): Promise<LoteAcciones> =>
        invoke('registrar_compra_acciones', { compra }),

    accionesDeSocio: (socioId: string): Promise<number> =>
        invoke('acciones_de_socio', { socioId }),

    accionesPorSocio: (): Promise<AccionesDeSocio[]> =>
        invoke('acciones_por_socio'),

    totalAcciones: (): Promise<number> =>
        invoke('total_acciones'),

    historialAccionesSocio: (socioId: string): Promise<ResumenMesAcciones[]> =>
        invoke('historial_acciones_socio', { socioId }),

    cupoDelMes: (fecha?: string): Promise<CupoMensual> =>
        invoke('cupo_del_mes', { fecha: fecha ?? null }),
};
