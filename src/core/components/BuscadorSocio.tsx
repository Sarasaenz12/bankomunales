import { useEffect, useState } from 'react';
import { SocioResumen } from '../../features/socios/types';

/** Buscador de socio por nombre o cédula, con sugerencias desplegables. */
export function BuscadorSocio({ placeholder, socios, onSelect, valor, onBusqueda }: {
    placeholder: string;
    socios: SocioResumen[];
    onSelect: (s: SocioResumen) => void;
    valor: string;
    /** Notifica al padre el texto de búsqueda actual (para habilitar/deshabilitar acciones). */
    onBusqueda?: (busqueda: string) => void;
}) {
    const [busqueda, setBusqueda] = useState(valor);

    useEffect(() => { setBusqueda(valor); }, [valor]);

    useEffect(() => { onBusqueda?.(busqueda); }, [busqueda]);

    // Si ya hay socio seleccionado (valor = "Nombre — cédula"), no mostrar sugerencias
    // a menos que el usuario borre el texto para buscar otro.
    const coincideValor = socios.some(s =>
        valor.includes(s.nombre_completo) && valor.includes(s.cedula));

    const q = busqueda.trim().toLowerCase();
    const coincidencias = q.length > 0 && !coincideValor
        ? socios.filter(s => s.nombre_completo.toLowerCase().includes(q) || s.cedula.includes(q)).slice(0, 5)
        : [];

    return (
        <div className="form-group custom-fg" style={{ marginBottom: 0 }}>
            <input
                className="form-input"
                placeholder={placeholder}
                value={busqueda}
                onChange={e => setBusqueda(e.target.value)}
            />
            {coincidencias.length > 0 && (
                <div className="detalle-card" style={{ padding: '0.25rem', marginTop: '0.25rem', marginBottom: 0 }}>
                    {coincidencias.map(s => (
                        <button
                            key={s.id}
                            className="chip"
                            style={{ display: 'block', width: '100%', textAlign: 'left', marginBottom: 2 }}
                            onClick={() => { onSelect(s); setBusqueda(`${s.nombre_completo} — ${s.cedula}`); }}
                        >
                            {s.nombre_completo} — {s.cedula}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}