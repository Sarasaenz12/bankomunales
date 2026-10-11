// ── Respaldo y restauración (RF-113) ──

/** Una entrada del historial de respaldos/restauraciones del Bankomunal. */
export interface RegistroRespaldo {
    id: string;
    fecha: string;
    usuario: string;
    ruta_archivo: string;
    tipo: string;
}

/** Lo que el sistema reconoce de un archivo de respaldo antes de importarlo. */
export interface VistaRespaldo {
    id: string;
    nombre: string;
    ubicacion: string;
    moneda: string;
    fecha_creacion: string;
    /** El Bankomunal ya está en este computador: importar reemplaza sus datos actuales. */
    ya_existe: boolean;
}

/** Extensión de los respaldos que se pueden abrir con doble clic desde el Explorador. */
export const EXTENSION_RESPALDO = 'bankomunal';

/** Nombre sugerido para guardar un respaldo: «Nombre_del_Banco_AAAA-MM-DD.bankomunal»
 *  (fecha local del día en que se hace). Quita los caracteres que Windows no admite. */
export function nombreSugeridoRespaldo(nombreBanco: string, hoy: Date = new Date()): string {
    const limpio = nombreBanco
        .replace(/[\\/:*?"<>|]/g, '')
        .trim()
        .replace(/\s+/g, '_') || 'Bankomunal';
    const dd = String(hoy.getDate()).padStart(2, '0');
    const mm = String(hoy.getMonth() + 1).padStart(2, '0');
    return `${limpio}_${hoy.getFullYear()}-${mm}-${dd}.${EXTENSION_RESPALDO}`;
}
