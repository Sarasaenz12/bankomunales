import { useState } from 'react';
import { cajaService } from './cajaService';
import { CodigoOperacion, OPERACIONES_CAJA } from './types';
import { HOY } from '../../core/lib/dates';

export function RegistrarTab({ onHecho, onError }: {
    onHecho: (m: string) => void;
    onError: (m: string) => void;
}) {
    const [codigo, setCodigo] = useState<CodigoOperacion | 'DONACION'>('OI');
    const [fecha, setFecha] = useState(HOY());
    const [monto, setMonto] = useState('');
    const [descripcion, setDescripcion] = useState('');

    const ayuda = codigo === 'DONACION'
        ? 'Las donaciones entran automáticamente al Fondo para Gastos (RF-87)'
        : OPERACIONES_CAJA.find(o => o.codigo === codigo)?.ayuda ?? '';

    async function guardar(e: React.FormEvent) {
        e.preventDefault();
        try {
            if (codigo === 'DONACION') {
                await cajaService.registrarDonacion(fecha, Number(monto), descripcion);
            } else {
                await cajaService.registrar({ codigo, fecha, monto: Number(monto), descripcion });
            }
            setMonto('');
            setDescripcion('');
            onHecho('Operación registrada en el Libro.');
        } catch (err: any) {
            onError(err?.toString() ?? 'No se pudo registrar la operación');
        }
    }

    return (
        <form className="detalle-card" onSubmit={guardar} style={{ maxWidth: 720 }}>
            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Tipo de operación</label>
                    <select
                        className="form-input"
                        value={codigo}
                        onChange={e => setCodigo(e.target.value as CodigoOperacion | 'DONACION')}
                    >
                        {OPERACIONES_CAJA.map(o => (
                            <option key={o.codigo} value={o.codigo}>{o.etiqueta}</option>
                        ))}
                        <option value="DONACION">Donación</option>
                    </select>
                    <div className="text-sm text-muted mt-1" style={{ fontSize: 11 }}>{ayuda}</div>
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                </div>
            </div>

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input type="number" className="form-input" min="1" step="any" value={monto} onChange={e => setMonto(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Descripción</label>
                    <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} placeholder="Ej. fotocopias de formatos" />
                </div>
            </div>

            <button type="submit" className="btn btn-primary" style={{ marginTop: '1rem' }}>
                Registrar operación
            </button>
        </form>
    );
}