// ── Respaldo y restauración (RF-113) ──

/** Una entrada del historial de respaldos/restauraciones del Bankomunal. */
export interface RegistroRespaldo {
    id: string;
    fecha: string;
    usuario: string;
    ruta_archivo: string;
    tipo: string;
}
