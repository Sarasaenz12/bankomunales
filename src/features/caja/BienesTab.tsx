import { useState } from 'react';
import { cajaService } from './cajaService';
import { Bien, TipoBien } from './types';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';
import { HOY } from '../../core/lib/dates';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';
import { mensajeDeError } from '../../core/lib/errores';

export function BienesTab({ bienes, moneda, onHecho, onError }: {
    bienes: Bien[];
    moneda: string;
    onHecho: (m: string) => void;
    onError: (m: string) => void;
}) {
    const [descripcion, setDescripcion] = useState('');
    const [fecha, setFecha] = useState(HOY());
    const [valor, setValor] = useState('');
    const [tipo, setTipo] = useState<TipoBien>('PROPIO');
    const { enviando, ejecutar } = useEnvioUnico();

    async function guardar() {
        try {
            await cajaService.registrarBien({
                descripcion, fecha, valor: Number(valor || 0), tipo,
            });
            setDescripcion('');
            setValor('');
            onHecho('Bien registrado como activo fijo.');
        } catch (err) {
            onError(mensajeDeError(err, 'No se pudo registrar el bien'));
        }
    }

    return (
        <>
            <form className="detalle-card" onSubmit={e => { e.preventDefault(); ejecutar(guardar); }} style={{ maxWidth: 900 }}>
                <h3 className="section-title">REGISTRAR BIEN ADQUIRIDO</h3>
                <p className="text-sm text-muted" style={{ marginTop: '-0.5rem', marginBottom: '1rem' }}>
                    Un bien <strong>propio</strong> se compra con la caja: al registrarlo, su valor sale
                    del efectivo y entra al activo fijo (permuta, no gasto — no reduce la ganancia
                    repartible). Un bien en <strong>comodato</strong> es prestado por un tercero: no
                    afecta la caja ni cuenta como activo propio, sólo queda registrado.
                </p>
                <div className="config-grid" style={{ gridTemplateColumns: 'repeat(4, 1fr)' }}>
                    <div className="form-group custom-fg">
                        <label className="form-label">Descripción</label>
                        <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Fecha de adquisición</label>
                        <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Valor</label>
                        <input type="text" inputMode="numeric" className="form-input" value={valor ? fmtNum(parseNum(valor)) : ''} onChange={e => setValor(e.target.value.replace(/\D/g, ''))} />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Tipo</label>
                        <select className="form-input" value={tipo} onChange={e => setTipo(e.target.value as TipoBien)}>
                            <option value="PROPIO">Propio</option>
                            <option value="COMODATO">En comodato</option>
                        </select>
                    </div>
                </div>
                <button type="submit" className="btn btn-primary" style={{ marginTop: '1rem' }} disabled={enviando}>
                    {enviando ? 'Registrando…' : 'Registrar bien'}
                </button>
            </form>

            <table className="activity-table">
                <thead>
                    <tr>
                        <th>Descripción</th>
                        <th>Adquisición</th>
                        <th>Tipo</th>
                        <th style={{ textAlign: 'right' }}>Valor</th>
                    </tr>
                </thead>
                <tbody>
                    {bienes.map(b => (
                        <tr key={b.id}>
                            <td>{b.descripcion}</td>
                            <td>{b.fecha_adquisicion}</td>
                            <td>{b.tipo === 'COMODATO' ? 'En comodato' : 'Propio'}</td>
                            <td style={{ textAlign: 'right' }}>{fmt(b.valor, moneda)}</td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {bienes.length === 0 && (
                <div className="empty-state">Todavía no hay bienes registrados.</div>
            )}
        </>
    );
}