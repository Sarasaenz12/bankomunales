import { useState } from 'react';
import { cajaService } from './cajaService';
import { Movimiento } from './types';
import { fmt } from '../../core/lib/format';

export const ETIQUETA_CODIGO: Record<string, string> = {
    OI: 'Otro Ingreso', EG: 'Otro Egreso',
    IFG: 'Ingreso Fondo Gastos', GBK: 'Gasto del Bankomunal',
    VC: 'Venta de Acciones', LC: 'Liquidación de Acciones', UR: 'Ganancia Repartida',
    CON: 'Desembolso de Crédito', OR: 'Interés Ordinario', PC: 'Pago de Cuota',
    MO: 'Interés de Mora', PDP: 'Pago Deuda Pendiente', COR: 'Refinanciamiento',
};

export function LibroTab({ libro, moneda, desde, hasta, setDesde, setHasta, onFiltrar, onCorregido, onError }: {
    libro: Movimiento[];
    moneda: string;
    desde: string; hasta: string;
    setDesde: (v: string) => void; setHasta: (v: string) => void;
    onFiltrar: () => void;
    onCorregido: () => void;
    onError: (m: string) => void;
}) {
    const [corrigiendo, setCorrigiendo] = useState<Movimiento | null>(null);

    return (
        <>
            <div className="list-toolbar">
                <div className="form-group custom-fg" style={{ margin: 0 }}>
                    <label className="form-label">Desde</label>
                    <input type="date" className="form-input" value={desde} onChange={e => setDesde(e.target.value)} />
                </div>
                <div className="form-group custom-fg" style={{ margin: 0 }}>
                    <label className="form-label">Hasta</label>
                    <input type="date" className="form-input" value={hasta} onChange={e => setHasta(e.target.value)} />
                </div>
                <button className="btn btn-secondary" onClick={onFiltrar} style={{ alignSelf: 'flex-end' }}>
                    Filtrar
                </button>
            </div>

            <table className="activity-table">
                <thead>
                    <tr>
                        <th style={{ width: 50 }}>N°</th>
                        <th>Fecha</th>
                        <th>Operación</th>
                        <th>Descripción</th>
                        <th style={{ textAlign: 'right' }}>Ingreso</th>
                        <th style={{ textAlign: 'right' }}>Egreso</th>
                        <th style={{ textAlign: 'right' }}>Saldo</th>
                        <th style={{ width: 90 }}></th>
                    </tr>
                </thead>
                <tbody>
                    {libro.map(m => (
                        <tr key={m.id}>
                            <td>{m.numero}</td>
                            <td>{m.fecha}</td>
                            <td>
                                {ETIQUETA_CODIGO[m.codigo] ?? m.codigo}
                                {m.corregido && (
                                    <span
                                        className="badge-estatus badge-deuda"
                                        style={{ marginLeft: 6, fontSize: '0.7rem' }}
                                        title={m.motivo_correccion ?? 'Corregida tras el cierre'}
                                    >
                                        corregida
                                    </span>
                                )}
                            </td>
                            <td>{m.descripcion}</td>
                            <td style={{ textAlign: 'right' }}>{m.ingreso ? fmt(m.ingreso, moneda) : '—'}</td>
                            <td style={{ textAlign: 'right' }}>{m.egreso ? fmt(m.egreso, moneda) : '—'}</td>
                            <td style={{ textAlign: 'right', fontWeight: 500 }}>{fmt(m.saldo, moneda)}</td>
                            <td style={{ textAlign: 'right' }}>
                                <button className="link-ver" onClick={() => setCorrigiendo(m)}>Corregir</button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {libro.length === 0 && (
                <div className="empty-state">No hay operaciones registradas en este rango.</div>
            )}

            {corrigiendo && (
                <CorregirForm
                    mov={corrigiendo}
                    onCancelar={() => setCorrigiendo(null)}
                    onListo={() => { setCorrigiendo(null); onCorregido(); }}
                    onError={onError}
                />
            )}
        </>
    );
}

/** RF-89/RF-90: tras el cierre, la corrección exige nombre y motivo para Auditoría. */
function CorregirForm({ mov, onCancelar, onListo, onError }: {
    mov: Movimiento;
    onCancelar: () => void;
    onListo: () => void;
    onError: (m: string) => void;
}) {
    const [fecha, setFecha] = useState(mov.fecha);
    const [monto, setMonto] = useState(String(mov.ingreso || mov.egreso));
    const [descripcion, setDescripcion] = useState(mov.descripcion);
    const [quien, setQuien] = useState('');
    const [motivo, setMotivo] = useState('');

    async function guardar(e: React.FormEvent) {
        e.preventDefault();
        try {
            await cajaService.corregir(
                mov.id, fecha, Number(monto), descripcion,
                mov.mes_cerrado ? quien : null,
                mov.mes_cerrado ? motivo : null,
            );
            onListo();
        } catch (err: any) {
            onError(err?.toString() ?? 'No se pudo corregir la operación');
        }
    }

    return (
        <form className="detalle-card mt-4" onSubmit={guardar}>
            <h3 className="section-title">CORREGIR OPERACIÓN N° {mov.numero}</h3>

            {mov.mes_cerrado && (
                <div className="error-message mb-4">
                    El mes de esta operación ya está cerrado. Para corregirla debe indicar su
                    nombre y el motivo; el cambio quedará registrado en Auditoría (RF-90).
                </div>
            )}

            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input type="number" className="form-input" min="1" step="any" value={monto} onChange={e => setMonto(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Descripción</label>
                    <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} />
                </div>
            </div>

            {mov.mes_cerrado && (
                <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                    <div className="form-group custom-fg">
                        <label className="form-label">Tu nombre *</label>
                        <input className="form-input" value={quien} onChange={e => setQuien(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Motivo de la corrección *</label>
                        <input className="form-input" value={motivo} onChange={e => setMotivo(e.target.value)} required />
                    </div>
                </div>
            )}

            <div style={{ display: 'flex', gap: '0.75rem', marginTop: '1rem' }}>
                <button type="submit" className="btn btn-primary">Guardar corrección</button>
                <button type="button" className="btn btn-secondary" onClick={onCancelar}>Cancelar</button>
            </div>
        </form>
    );
}