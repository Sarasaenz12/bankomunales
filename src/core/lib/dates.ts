/** Fecha de hoy en formato ISO (YYYY-MM-DD), útil para inputs de tipo date. */
export const HOY = () => new Date().toISOString().slice(0, 10);

const MESES = [
    'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
    'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
];

/** "2026-07-01" → "Julio 2026". El backend guarda el mes como primer día ISO. */
export function nombreMes(iso: string) {
    const [anio, mes] = iso.split('-');
    return `${MESES[Number(mes) - 1] ?? mes} ${anio}`;
}