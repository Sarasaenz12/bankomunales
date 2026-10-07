// Monedas disponibles para un Bankomunal. El `codigo` (3 letras) es lo que se guarda;
// la `etiqueta` es lo que ve el usuario en el selector.
export interface Moneda {
    codigo: string;
    etiqueta: string;
}

export const MONEDAS: Moneda[] = [
    { codigo: 'COP', etiqueta: '$ COP - Peso Colombiano' },
];
