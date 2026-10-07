import { useEffect, useState } from 'react';
import { cierreService } from './cierreService';
import { CierreMes, DetalleCierre } from './types';
import { fmt } from '../../core/lib/format';

export function HistorialTab({ moneda }: { moneda: string }) {
    const [cierres, setCierres] = useState<CierreMes[]>([]);
    const [detalle, setDetalle] = useState<Record<string, DetalleCierre>>({});
    const [visible, setVisible] = useState<string | null>(null);
    const [error, setError] = useState('');

    useEffect(() => {
        cierreService.listarCierres()
            .then(setCierres)
            .catch(e => setError(e?.toString() ?? 'Error al cargar los cierres'));
    }, []);

    async function verDetalle(id: string) {
        if (visible === id) { setVisible(null); return; }
        setError('');
        try {
            if (!detalle[id]) {
                setDetalle({ ...detalle, [id]: await cierreService.detalleCierre(id) });
            }
            setVisible(id);
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo cargar el detalle');
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            {error && <div className="error-message">{error}</div>}
            {cierres.length === 0 && !error && (
                <div className="empty-state">Todavía no hay meses cerrados.</div>
            )}

            {cierres.map(c => {
                const d = detalle[c.id];
                const abierto = visible === c.id && d;
                return (
                    <div key={c.id} className="detalle-card" style={{ padding: '1.25rem', marginBottom: '1rem' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
                            <div>
                                <strong>Mes {c.mes}</strong>
                                <div className="text-sm text-muted">Cerrado el {c.fecha_cierre.replace('T', ' ')}</div>
                            </div>
                            <div style={{ display: 'flex', gap: '1rem', alignItems: 'center', flexWrap: 'wrap' }}>
                                <div className="text-sm">
                                    <div>Activo: <strong>{fmt(c.activo_total, moneda)}</strong> · Pasivo: <strong>{fmt(c.pasivo_total, moneda)}</strong></div>
                                    <div>Colocación {c.colocacion_pct.toFixed(2)}% · PPCFC {c.ppcfc_pct.toFixed(2)}%</div>
                                </div>
                                <span className={`badge-estatus ${c.cuadra ? 'badge-activo' : 'badge-retirado'}`}>
                                    {c.cuadra ? 'CUADRA' : 'NO CUADRA'}
                                </span>
                                <button className="btn btn-secondary" onClick={() => verDetalle(c.id)}>
                                    {visible === c.id ? 'Ocultar' : 'Ver reparto'}
                                </button>
                            </div>
                        </div>

                        {abierto && d && (
                            <div className="mt-4">
                                <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>
                                    Reparto del mes — valor por acción {fmt(d.reparto.valor_por_accion, moneda)}
                                </div>
                                <div className="grid-dos">
                                    <div>Ingresos repartibles: <strong>{fmt(d.reparto.ingresos_repartibles, moneda)}</strong></div>
                                    <div>Otros egresos: <strong>{fmt(d.reparto.otros_egresos, moneda)}</strong></div>
                                    <div>Fondo gastos: <strong>{fmt(d.reparto.fondo_gastos, moneda)}</strong></div>
                                    <div>Fondo incobrables: <strong>{fmt(d.reparto.fondo_incobrables, moneda)}</strong></div>
                                    <div>Balance neto: <strong>{fmt(d.reparto.balance_neto, moneda)}</strong></div>
                                    <div>Acciones activas: <strong>{d.reparto.acciones_activas}</strong></div>
                                </div>

                                <div className="text-sm text-muted" style={{ margin: '1rem 0 0.5rem' }}>
                                    Devengo por lote ({d.devengos.length})
                                </div>
                                <table className="activity-table">
                                    <thead>
                                        <tr>
                                            <th>Acciones</th>
                                            <th>Valor/acción</th>
                                            <th>Ganancia devengada</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {d.devengos.map(g => (
                                            <tr key={g.lote_acciones_id}>
                                                <td>{g.acciones}</td>
                                                <td>{fmt(g.valor_por_accion, moneda)}</td>
                                                <td>{fmt(g.monto, moneda)}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        )}
                    </div>
                );
            })}
        </div>
    );
}