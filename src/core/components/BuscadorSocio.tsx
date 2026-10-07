import { useEffect, useState } from 'react';
import { SocioResumen } from '../../features/socios/types';

/** Buscador de socio por nombre o cédula, con sugerencias desplegables y navegación por
 *  teclado (↑ ↓ para moverse, Enter para elegir, Esc para cerrar). */
export function BuscadorSocio({ placeholder, socios, onSelect, valor, onBusqueda }: {
    placeholder: string;
    socios: SocioResumen[];
    onSelect: (s: SocioResumen) => void;
    valor: string;
    /** Notifica al padre el texto de búsqueda actual (para habilitar/deshabilitar acciones). */
    onBusqueda?: (busqueda: string) => void;
}) {
    const [busqueda, setBusqueda] = useState(valor);
    const [activo, setActivo] = useState(0);
    const [cerrado, setCerrado] = useState(false);

    useEffect(() => { setBusqueda(valor); }, [valor]);

    useEffect(() => { onBusqueda?.(busqueda); }, [busqueda]);

    // Solo se considera "socio ya elegido" cuando el texto del campo es EXACTAMENTE su
    // etiqueta completa "Nombre — cédula" (lo que deja onSelect). En ese caso no se
    // sugiere nada. Si el texto es una búsqueda parcial, se muestran coincidencias.
    const muestraSeleccion = socios.some(s => busqueda === `${s.nombre_completo} — ${s.cedula}`);

    const q = busqueda.trim().toLowerCase();
    const coincidencias = q.length > 0 && !muestraSeleccion && !cerrado
        ? socios.filter(s => s.nombre_completo.toLowerCase().includes(q) || s.cedula.includes(q)).slice(0, 6)
        : [];

    function elegir(s: SocioResumen) {
        onSelect(s);
        setBusqueda(`${s.nombre_completo} — ${s.cedula}`);
        setCerrado(true);
    }

    function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
        if (coincidencias.length === 0) return;
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setActivo(i => Math.min(i + 1, coincidencias.length - 1));
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setActivo(i => Math.max(i - 1, 0));
        } else if (e.key === 'Enter') {
            e.preventDefault();
            const s = coincidencias[Math.min(activo, coincidencias.length - 1)];
            if (s) elegir(s);
        } else if (e.key === 'Escape') {
            setCerrado(true);
        }
    }

    return (
        <div className="form-group custom-fg" style={{ marginBottom: 0, position: 'relative' }}>
            <input
                className="form-input"
                placeholder={placeholder}
                value={busqueda}
                onChange={e => { setBusqueda(e.target.value); setActivo(0); setCerrado(false); }}
                onKeyDown={onKeyDown}
                autoComplete="off"
            />
            {coincidencias.length > 0 && (
                <div className="detalle-card" style={{
                    padding: '0.25rem', marginTop: '0.25rem', marginBottom: 0,
                    position: 'absolute', left: 0, right: 0, top: '100%', zIndex: 20,
                    boxShadow: '0 6px 18px rgba(0,0,0,0.12)', maxHeight: 260, overflowY: 'auto',
                }}>
                    {coincidencias.map((s, i) => (
                        <button
                            key={s.id}
                            className="chip"
                            style={{
                                display: 'block', width: '100%', textAlign: 'left', marginBottom: 2,
                                background: i === activo ? 'var(--primary-color)' : undefined,
                                color: i === activo ? '#fff' : undefined,
                            }}
                            onMouseEnter={() => setActivo(i)}
                            onClick={() => elegir(s)}
                        >
                            {s.nombre_completo} — {s.cedula}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}
