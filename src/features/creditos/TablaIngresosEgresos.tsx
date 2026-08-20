import { Trash2, Plus } from 'lucide-react';
import { fmt } from '../../core/lib/format';

export interface FilaMonto { tipo: string; monto: string; }

/** Tabla editable de ingresos/egresos de una solicitud (RF-45). */
export function TablaIngresosEgresos({ filas, setFilas, ejemplos }: {
    filas: FilaMonto[];
    setFilas: (f: FilaMonto[]) => void;
    ejemplos: string[];
}) {
    const total = filas.reduce((acc, f) => acc + (Number(f.monto) || 0), 0);

    return (
        <div>
            {filas.map((fila, i) => (
                <div key={i} style={{ display: 'flex', gap: '0.5rem', marginBottom: '0.5rem' }}>
                    <input
                        className="form-input"
                        placeholder={ejemplos[i] ?? 'Tipo de ingreso/egreso'}
                        value={fila.tipo}
                        onChange={e => {
                            const nuevo = [...filas];
                            nuevo[i] = { ...nuevo[i], tipo: e.target.value };
                            setFilas(nuevo);
                        }}
                    />
                    <input
                        className="form-input"
                        style={{ maxWidth: 140 }}
                        placeholder="Monto"
                        inputMode="numeric"
                        value={fila.monto}
                        onChange={e => {
                            const nuevo = [...filas];
                            nuevo[i] = { ...nuevo[i], monto: e.target.value.replace(/\D/g, '') };
                            setFilas(nuevo);
                        }}
                    />
                    <button
                        className="btn btn-secondary"
                        style={{ padding: '0 0.75rem' }}
                        onClick={() => setFilas(filas.filter((_, j) => j !== i))}
                        aria-label="Quitar fila"
                    >
                        <Trash2 size={15} />
                    </button>
                </div>
            ))}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <button className="btn btn-secondary" style={{ padding: '0.35rem 0.75rem', fontSize: '0.8125rem' }} onClick={() => setFilas([...filas, { tipo: '', monto: '' }])}>
                    <Plus size={14} style={{ marginRight: 4 }} /> Agregar
                </button>
                <strong>{fmt(total)}</strong>
            </div>
        </div>
    );
}