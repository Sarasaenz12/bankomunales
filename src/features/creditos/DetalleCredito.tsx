import { Credito, DESTINOS_CREDITO } from './types';
import { fmt } from '../../core/lib/format';
import { cuotaMensual } from './creditosService';

/** Información de un crédito desembolsado, consultada desde la bandeja de
 * solicitudes ("Ver crédito", RF-52). */
export function DetalleCredito({ credito, moneda, nombreSocio, onClose }: {
    credito: Credito; moneda: string; nombreSocio: (id: string) => string; onClose: () => void;
}) {
    const destino = DESTINOS_CREDITO.find(d => d.codigo === credito.destino)?.etiqueta ?? credito.destino;
    const cuota = cuotaMensual(credito.monto_original, credito.tasa, credito.plazo_cuotas);

    return (
        <div className="detalle-card mt-4" style={{ background: '#f8fafc', borderLeft: '4px solid var(--success-color)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
                <div>
                    <h3 className="section-title" style={{ marginBottom: 0 }}>CRÉDITO Nº {credito.numero}</h3>
                    <div className="text-sm text-muted">{nombreSocio(credito.socio_id)}</div>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                    <span className={`badge-estatus ${credito.estatus === 'VIGENTE' ? 'badge-activo' : 'badge-pagado'}`}>
                        {credito.estatus}
                    </span>
                    <button className="btn btn-secondary" onClick={onClose} title="Cerrar detalle">
                        Cerrar
                    </button>
                </div>
            </div>

            <div className="config-grid mt-4">
                <div className="stat-chip">
                    <div className="text-sm text-muted">Monto original</div>
                    <div className="stat-num">{fmt(credito.monto_original, moneda)}</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">Saldo pendiente</div>
                    <div className="stat-num">{fmt(credito.saldo_pendiente, moneda)}</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">Cuota mensual aprox.</div>
                    <div className="stat-num">{fmt(cuota, moneda)}</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">Tasa</div>
                    <div className="stat-num">{credito.tasa}%</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">Plazo</div>
                    <div className="stat-num">{credito.plazo_cuotas} cuotas</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">Cuota actual</div>
                    <div className="stat-num">{credito.cuota_actual} / {credito.plazo_cuotas}</div>
                </div>
            </div>

            <div className="text-sm mt-4" style={{ display: 'grid', gap: '0.25rem', color: 'var(--text-muted)' }}>
                <div>Destino: <strong>{destino}</strong></div>
                <div>Fecha de desembolso: <strong>{credito.fecha_desembolso}</strong></div>
                <div>Vence: <strong>{credito.fecha_vencimiento}</strong> · Frecuencia: <strong>{credito.frecuencia_pago}</strong></div>
            </div>

            {credito.garantias.length > 0 && (
                <div className="mt-4">
                    <div className="text-sm text-muted">Garantías</div>
                    <div className="tabla-scroll">
                        <table className="activity-table">
                            <thead>
                                <tr><th>Socio</th><th>Rol</th><th>Acciones</th></tr>
                            </thead>
                            <tbody>
                                {credito.garantias.map((g, i) => (
                                    <tr key={i}>
                                        <td>{nombreSocio(g.socio_id)}</td>
                                        <td>{g.rol}</td>
                                        <td>{g.acciones_comprometidas}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
            )}
        </div>
    );
}