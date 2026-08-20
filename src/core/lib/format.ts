/** Formatea un número como moneda (es-CO, sin decimales). */
export function fmt(n: number, moneda = 'COP') {
    return new Intl.NumberFormat('es-CO', {
        style: 'currency', currency: moneda, maximumFractionDigits: 0,
    }).format(n);
}