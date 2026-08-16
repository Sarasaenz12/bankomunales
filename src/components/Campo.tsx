/** Tipo de dato del campo, que determina qué se puede teclear y cómo se valida. */
export type TipoCampo = 'texto' | 'numero' | 'email';

/**
 * Un correo válido debe tener texto antes de la @, un dominio después y al menos un
 * punto en él. No se usa una expresión exhaustiva del RFC a propósito: las direcciones
 * raras que ésta rechazaría no existen en la práctica, y un patrón ilegible sería peor
 * de mantener que uno estricto de más.
 */
const PATRON_CORREO = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

/** Devuelve el mensaje de error del valor, o `null` si es válido. */
export function validarCampo(tipo: TipoCampo, valor: string, label: string): string | null {
    const v = valor.trim();
    if (!v) return null; // Los campos vacíos los valida `required`, no el formato.

    if (tipo === 'email' && !PATRON_CORREO.test(v)) {
        return 'El correo debe tener el formato nombre@dominio.com';
    }
    if (tipo === 'numero' && !/^\d+$/.test(v)) {
        return `${label} sólo admite números`;
    }
    return null;
}

export function Campo({
    label, value, onChange, tipo = 'texto', required, error, maxLength,
}: {
    label: string;
    value: string;
    onChange: (valor: string) => void;
    tipo?: TipoCampo;
    required?: boolean;
    /** Error de validación a mostrar bajo el campo (se calcula al enviar). */
    error?: string | null;
    maxLength?: number;
}) {
    // En los campos numéricos se descarta cualquier carácter que no sea dígito a medida
    // que se teclea: es más claro que dejar escribir y regañar después al guardar.
    const manejar = (e: React.ChangeEvent<HTMLInputElement>) => {
        const bruto = e.target.value;
        onChange(tipo === 'numero' ? bruto.replace(/\D/g, '') : bruto);
    };

    return (
        <div className="form-group custom-fg">
            <label className="form-label">
                {label}{required && ' *'}
            </label>
            <input
                className={`form-input ${error ? 'input-error' : ''}`}
                // `type="email"` aporta el teclado adecuado en pantallas táctiles, pero la
                // validación visible es la nuestra, para que el mensaje salga en español.
                type={tipo === 'email' ? 'email' : 'text'}
                inputMode={tipo === 'numero' ? 'numeric' : undefined}
                value={value}
                onChange={manejar}
                required={required}
                maxLength={maxLength}
                aria-invalid={!!error}
            />
            {error && <div className="campo-error">{error}</div>}
        </div>
    );
}
