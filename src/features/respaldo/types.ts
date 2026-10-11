// ── Respaldo y restauración (RF-113) ──

/** Una entrada del historial de respaldos/restauraciones del Bankomunal. */
export interface RegistroRespaldo {
    id: string;
    fecha: string;
    usuario: string;
    ruta_archivo: string;
    tipo: string;
}

/** Nombre sugerido para guardar un respaldo: «Nombre_del_Banco_AAAA-MM-DD.db»
 *  (fecha local del día en que se hace). Quita los caracteres que Windows no admite. */
export function nombreSugeridoRespaldo(nombreBanco: string, hoy: Date = new Date()): string {
    const limpio = nombreBanco
        .replace(/[\/:*?"<>|]/g, '')
        .trim()
        .replace(/\s+/g, '_') || 'Bankomunal';
    const dd = String(hoy.getDate()).padStart(2, '0');
    const mm = String(hoy.getMonth() + 1).padStart(2, '0');
    return `${limpio}_${hoy.getFullYear()}-${mm}-${dd}.db`;
}
