/** Formatea un número como moneda (es-CO, sin decimales). */
export function fmt(n: number, moneda = 'COP') {
    // Un valor que redondea a cero se muestra sin signo ("-$ 0" confunde).
    const valor = Math.round(n) === 0 ? 0 : n;
    return new Intl.NumberFormat('es-CO', {
        style: 'currency', currency: moneda, maximumFractionDigits: 0,
    }).format(valor);
}

/**
 * Formatea un entero con separador de miles (es-CO), sin símbolo de moneda:
 * `1416666` → `"1.416.666"`. Para mostrar montos dentro de campos de texto, donde
 * el dinero del Bankomunal va en pesos enteros (COP, sin centavos).
 */
export function fmtNum(n: number) {
    return new Intl.NumberFormat('es-CO', { maximumFractionDigits: 0 }).format(n);
}

/** Convierte lo tecleado en un campo de dinero a un entero: descarta todo lo que no
 *  sea dígito (puntos de miles, espacios, símbolos). `"1.416.666"` → `1416666`. */
export function parseNum(texto: string): number {
    const soloDigitos = texto.replace(/\D/g, '');
    return soloDigitos ? parseInt(soloDigitos, 10) : 0;
}