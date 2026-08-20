/** Pestañas de una pantalla. Cada una es un botón con clase `tab`. */
export function Tabs<T extends string>({ actual, onChange, items }: {
    actual: T;
    onChange: (t: T) => void;
    items: { valor: T; etiqueta: string }[];
}) {
    return (
        <div className="tabs">
            {items.map(it => (
                <button
                    key={it.valor}
                    className={`tab ${actual === it.valor ? 'active' : ''}`}
                    onClick={() => onChange(it.valor)}
                >
                    {it.etiqueta}
                </button>
            ))}
        </div>
    );
}