import { useEffect, useState } from 'react';
import { cierreService } from './cierreService';
import { RepartoAnual } from './types';
import { fmt } from '../../core/lib/format';
import { mesActual } from './util';
import { sociosService } from '../socios/sociosService';
import { SocioResumen } from '../socios/types';

/** Estado de pago de la ganancia acumulada de un socio para la cohorte. */
function estadoPago(acumulado: number, pendiente: number): string {
    if (Math.abs(acumulado) < 0.01) return '—';
    if (Math.abs(pendiente) < 0.01) return 'Pagado';
    if (Math.abs(pendiente - acumulado) < 0.01) return 'Pendiente';
    return 'Parcial';
}

export function RepartoAnualTab({ moneda }: { moneda: string }) {
    const [mes, setMes] = useState(mesActual());
    const [datos, setDatos] = useState<RepartoAnual | null>(null);
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [error, setError] = useState('');

    useEffect(() => {
        sociosService.listar().then(setSocios).catch(() => {});
    }, []);

    const nombreSocio = (id: string) => socios.find(s => s.id === id)?.nombre_completo ?? id;

    async function consultar() {
        setError(''); setDatos(null);
        try {
            setDatos(await cierreService.repartoAnual(mes + '-01'));
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo calcular el reparto anual');
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 1100 }}>
            <div className="chip-row" style={{ alignItems: 'flex-end', gap: '1rem', flexWrap: 'wrap' }}>
                <div className="form-group custom-fg" style={{ maxWidth: 240 }}>
                    <label className="form-label">Mes de compra (cohorte)</label>
                    <input className="form-input" type="month" value={mes} onChange={e => setMes(e.target.value)} />
                </div>
                <button className="btn btn-primary" onClick={consultar}>
                    Ver reparto anual
                </button>
            </div>

            <div className="text-sm text-muted mt-4">
                Acumula la ganancia por acción mes a mes durante los 12 meses de la ventana
                (mes de compra + 11), tal como la hoja de ganancias del Bankomunal. El pago por
                socio es sus acciones de ese mes × la ganancia devengada.
            </div>

            {error && <div className="error-message mt-4">{error}</div>}

            {datos && (
                <>
                    <div className="grid-cuadre mt-4">
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Cohorte</div>
                            <div><strong>{datos.mes_cohorte.slice(0, 7)}</strong> → {datos.mes_fin_ventana.slice(0, 7)}</div>
                            <div className="text-sm mt-4">Acciones de la cohorte: <strong>{datos.acciones_cohorte}</strong></div>
                        </div>
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Ganancia acumulada por acción</div>
                            <div className="total-line"><strong>{fmt(datos.acumulado_por_accion, moneda)}</strong></div>
                        </div>
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Total a pagar a la cohorte</div>
                            <div className="total-line"><strong>{fmt(datos.total_a_pagar, moneda)}</strong></div>
                        </div>
                    </div>

                    {datos.meses.length === 0 ? (
                        <div className="empty-state mt-4">
                            Todavía no hay cierres en la ventana de esta cohorte. El reparto anual
                            se llena a medida que se cierran los meses.
                        </div>
                    ) : (
                        <>
                            <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>
                                Ganancia por acción mes a mes
                            </div>
                            <div style={{ overflowX: 'auto' }}>
                                <table className="activity-table">
                                    <thead>
                                        <tr>
                                            <th>Mes</th>
                                            <th style={{ textAlign: 'right' }}>Acciones activas</th>
                                            <th style={{ textAlign: 'right' }}>Ingresos repartibles</th>
                                            <th style={{ textAlign: 'right' }}>F. Gastos</th>
                                            <th style={{ textAlign: 'right' }}>F. Incobrables</th>
                                            <th style={{ textAlign: 'right' }}>Otros egresos</th>
                                            <th style={{ textAlign: 'right' }}>Balance neto</th>
                                            <th style={{ textAlign: 'right' }}>Ganancia/acción</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {datos.meses.map(f => (
                                            <tr key={f.mes}>
                                                <td>{f.mes.slice(0, 7)}</td>
                                                <td style={{ textAlign: 'right' }}>{f.acciones_activas}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.ingresos_repartibles, moneda)}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.fondo_gastos, moneda)}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.fondo_incobrables, moneda)}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.otros_egresos, moneda)}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.balance_neto, moneda)}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(f.valor_por_accion, moneda)}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                    <tfoot>
                                        <tr>
                                            <td colSpan={7}><strong>Acumulado por acción (año)</strong></td>
                                            <td style={{ textAlign: 'right' }}><strong>{fmt(datos.acumulado_por_accion, moneda)}</strong></td>
                                        </tr>
                                    </tfoot>
                                </table>
                            </div>

                            <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>
                                Pago por socio (acciones compradas en {datos.mes_cohorte.slice(0, 7)})
                            </div>
                            {datos.pagos.length === 0 ? (
                                <div className="empty-state">Ningún socio compró acciones en este mes.</div>
                            ) : (
                                <table className="activity-table">
                                    <thead>
                                        <tr>
                                            <th>Socio</th>
                                            <th style={{ textAlign: 'right' }}>Acciones</th>
                                            <th style={{ textAlign: 'right' }}>Ganancia acumulada</th>
                                            <th style={{ textAlign: 'center' }}>Estado</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {datos.pagos.map(p => (
                                            <tr key={p.socio_id}>
                                                <td>{nombreSocio(p.socio_id)}</td>
                                                <td style={{ textAlign: 'right' }}>{p.acciones}</td>
                                                <td style={{ textAlign: 'right' }}>{fmt(p.monto_acumulado, moneda)}</td>
                                                <td style={{ textAlign: 'center' }}>{estadoPago(p.monto_acumulado, p.monto_pendiente)}</td>
                                            </tr>
                                        ))}
                                    </tbody>
                                    <tfoot>
                                        <tr>
                                            <td><strong>Total</strong></td>
                                            <td style={{ textAlign: 'right' }}><strong>{datos.acciones_cohorte}</strong></td>
                                            <td style={{ textAlign: 'right' }}><strong>{fmt(datos.total_a_pagar, moneda)}</strong></td>
                                            <td />
                                        </tr>
                                    </tfoot>
                                </table>
                            )}
                        </>
                    )}
                </>
            )}
        </div>
    );
}
