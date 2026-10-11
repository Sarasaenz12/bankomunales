import { useState } from 'react';
import { cajaService } from './cajaService';
import { CodigoOperacion, OPERACIONES_CAJA } from './types';
import { HOY } from '../../core/lib/dates';
import { fmtNum, parseNum } from '../../core/lib/format';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';
import { mensajeDeError } from '../../core/lib/errores';

export function RegistrarTab({ onHecho, onError }: {
    onHecho: (m: string) => void;
    onError: (m: string) => void;
}) {
    const [codigo, setCodigo] = useState<CodigoOperacion | 'DONACION'>('OI');
    const [fecha, setFecha] = useState(HOY());
    const [monto, setMonto] = useState('');
    const [descripcion, setDescripcion] = useState('');
    const { enviando, ejecutar } = useEnvioUnico();

    const ayuda = codigo === 'DONACION'
        ? 'Las donaciones entran automáticamente al Fondo para Gastos'
        : OPERACIONES_CAJA.find(o => o.codigo === codigo)?.ayuda ?? '';

    async function guardar() {
        try {
            if (codigo === 'DONACION') {
                await cajaService.registrarDonacion(fecha, Number(monto), descripcion);
            } else {
                await cajaService.registrar({ codigo, fecha, monto: Number(monto), descripcion });
            }
            setMonto('');
            setDescripcion('');
            onHecho('Operación registrada en el Libro.');
        } catch (err) {
            onError(mensajeDeError(err, 'No se pudo registrar la operación'));
        }
    }

    return (
        <form className="detalle-card" onSubmit={e => { e.preventDefault(); ejecutar(guardar); }} style={{ maxWidth: 720 }}>
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
                    <div className="text-sm text-muted mt-1" style={{ fontSize: 11, padding: '0.2rem'}}>{ayuda}</div>
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                </div>
            </div>

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input type="text" inputMode="numeric" className="form-input" value={monto ? fmtNum(parseNum(monto)) : ''} onChange={e => setMonto(e.target.value.replace(/\D/g, ''))} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Descripción</label>
                    <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} placeholder="Ej. fotocopias de formatos" />
                </div>
            </div>

            <button type="submit" className="btn btn-primary" style={{ marginTop: '1rem' }} disabled={enviando}>
                {enviando ? 'Registrando…' : 'Registrar operación'}
            </button>
        </form>
    );
}