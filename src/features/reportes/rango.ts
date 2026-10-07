/** Filtro de rango de fechas para los reportes (client-side, sobre la fecha que ya
 *  trae cada fila). Las fechas se comparan como texto ISO `YYYY-MM-DD`, que ordena
 *  cronológicamente. Un extremo vacío no acota por ese lado. */
export function enRango(fechaISO: string | null | undefined, desde: string, hasta: string): boolean {
    const f = (fechaISO ?? '').slice(0, 10);
    if (!f) return true;
    if (desde && f < desde) return false;
    if (hasta && f > hasta) return false;
    return true;
}
