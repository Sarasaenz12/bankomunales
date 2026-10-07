import { useEffect, useState } from 'react';
import { Credito, SolicitudCredito, DESTINOS_CREDITO, TablaCredito, ETIQUETA_ESTADO } from './types';
import { fmt } from '../../core/lib/format';
import { creditosService } from './creditosService';
import { TablaAmortizacion } from './TablaAmortizacion';

/** Resumen completo de una solicitud / crédito, consultado desde "Ver crédito" (RF-52):
 * datos del crédito, finanzas del solicitante, decisión de la Junta, garantías y la tabla
 * de pagos. */
export function DetalleCredito({ credito, solicitud, moneda, nombreSocio }: {
    credito?: Credito | null; solicitud?: SolicitudCredito | null; moneda: string; nombreSocio: (id: string) => string;
}) {
    const data = credito || solicitud;
    const monto = credito ? credito.monto_original : (solicitud?.monto_aprobado || solicitud?.monto_solicitado || 0);
    const plazo = data?.plazo_cuotas ?? 0;

    // La tabla de pagos (saldo decreciente, RN-12) la calcula el backend, para no mostrar
    // un número distinto del que se guarda.
    const [tabla, setTabla] = useState<TablaCredito | null>(null);
    useEffect(() => {
        if (!monto || !plazo) { setTabla(null); return; }
        let vigente = true;
        creditosService.previsualizarTabla(monto, plazo)
            .then(t => { if (vigente) setTabla(t); })
            .catch(() => { if (vigente) setTabla(null); });
        return () => { vigente = false; };
    }, [monto, plazo]);

    if (!data) return null;

    const destino = DESTINOS_CREDITO.find(d => d.codigo === data.destino)?.etiqueta ?? data.destino;
    const numero = credito ? credito.numero : 'No desembolsado';
    const saldo = credito ? credito.saldo_pendiente : monto;

    return (
        <div className="detalle-card mt-4" style={{ background: '#f8fafc', borderLeft: `4px solid ${credito ? 'var(--success-color)' : '#9ca3af'}` }}>
            <div>
                <h3 className="section-title" style={{ marginBottom: 0 }}>
                    {credito ? `CRÉDITO Nº ${numero}` : 'RESUMEN DE LA SOLICITUD'}
                </h3>
                <div className="text-sm text-muted">{nombreSocio(data.socio_id)}</div>
            </div>

            {/* ── Datos del crédito ── */}
            <div className="config-grid mt-4">
                <div className="stat-chip">
                    <div className="text-sm text-muted">{credito ? 'Monto original' : 'Monto (Aprobado/Solicitado)'}</div>
                    <div className="stat-num">{fmt(monto, moneda)}</div>
                </div>
                <div className="stat-chip">
                    <div className="text-sm text-muted">{credito ? 'Saldo pendiente' : 'Saldo inicial proyectado'}</div>
                    <div className="stat-num">{fmt(saldo, moneda)}</div>
                </div>
                {tabla && (
                    <div className="stat-chip">
                        <div className="text-sm text-muted">Cuota mensual</div>
                        <div className="stat-num">{fmt(tabla.cuota_mensual, moneda)}</div>
                    </div>
                )}
                {credito && (
                    <div className="stat-chip">
                        <div className="text-sm text-muted">Tasa</div>
                        <div className="stat-num">{credito.tasa}%</div>
                    </div>
                )}
                <div className="stat-chip">
                    <div className="text-sm text-muted">Plazo</div>
                    <div className="stat-num">{plazo} cuotas</div>
                </div>
                {credito && (
                    <div className="stat-chip">
                        <div className="text-sm text-muted">Cuota actual</div>
                        <div className="stat-num">{credito.cuota_actual} / {plazo}</div>
                    </div>
                )}
            </div>

            <div className="text-sm mt-4" style={{ display: 'grid', gap: '0.25rem', color: 'var(--text-muted)' }}>
                <div>Destino: <strong>{destino}</strong></div>
                {credito ? (
                    <>
                        <div>Fecha de desembolso: <strong>{credito.fecha_desembolso}</strong></div>
                        <div>Vence: <strong>{credito.fecha_vencimiento}</strong> · Frecuencia: <strong>{credito.frecuencia_pago}</strong></div>
                    </>
                ) : (
                    <div>Fecha de solicitud: <strong>{solicitud?.fecha_solicitud}</strong></div>
                )}
            </div>

            {/* ── Finanzas del solicitante (RF-45) ── */}
            {solicitud && (
                <div className="mt-4">
                    <div className="text-sm text-muted" style={{ marginBottom: '0.35rem' }}>Finanzas del solicitante</div>
                    <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                        <div className="stat-chip">
                            <div className="text-sm text-muted">Ingresos</div>
                            <div className="stat-num">{fmt(solicitud.total_ingresos, moneda)}</div>
                        </div>
                        <div className="stat-chip">
                            <div className="text-sm text-muted">Egresos</div>
                            <div className="stat-num">{fmt(solicitud.total_egresos, moneda)}</div>
                        </div>
                        <div className="stat-chip">
                            <div className="text-sm text-muted">Capacidad de pago</div>
                            <div className="stat-num">{fmt(solicitud.capacidad_pago, moneda)}</div>
                        </div>
                    </div>
                </div>
            )}

            {/* ── Decisión de la Junta ── */}
            {solicitud && solicitud.estado !== 'PENDIENTE' && (
                <div className="text-sm mt-4" style={{ display: 'grid', gap: '0.25rem', color: 'var(--text-muted)' }}>
                    <div>Decisión de la Junta: <strong>{ETIQUETA_ESTADO[solicitud.estado]}</strong></div>
                    {solicitud.monto_aprobado != null && <div>Monto aprobado: <strong>{fmt(solicitud.monto_aprobado, moneda)}</strong></div>}
                    {solicitud.decidida_por && <div>Decidida por: <strong>{solicitud.decidida_por}</strong>{solicitud.fecha_decision ? ` · ${solicitud.fecha_decision}` : ''}</div>}
                    {solicitud.observacion && <div>Observación: <strong>{solicitud.observacion}</strong></div>}
                </div>
            )}

            {/* ── Garantías ── */}
            {data.garantias.length > 0 && (
                <div className="mt-4">
                    <div className="text-sm text-muted">{credito ? 'Garantías efectivas' : 'Garantías propuestas'}</div>
                    <div className="tabla-scroll">
                        <table className="activity-table">
                            <thead>
                                <tr><th>Socio</th><th>Rol</th><th style={{ textAlign: 'right' }}>Acciones</th></tr>
                            </thead>
                            <tbody>
                                {data.garantias.map((g: any, i: number) => (
                                    <tr key={i}>
                                        <td>{nombreSocio(g.socio_id)}</td>
                                        <td>{g.rol ?? 'TITULAR/FIADOR'}</td>
                                        <td style={{ textAlign: 'right' }}>{g.acciones_comprometidas}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </div>
            )}

            {/* ── Tabla de pagos ── */}
            {tabla && (
                <div className="mt-4">
                    <div className="text-sm text-muted" style={{ marginBottom: '0.35rem' }}>Tabla de pagos (saldo decreciente)</div>
                    <TablaAmortizacion cuotas={tabla.cuotas} monto={monto} moneda={moneda} />
                    <div className="text-sm mt-4" style={{ display: 'flex', justifyContent: 'space-between' }}>
                        <span>Total a pagar: <strong>{fmt(tabla.monto_total, moneda)}</strong></span>
                        <span>Cuota mensual: <strong>{fmt(tabla.cuota_mensual, moneda)}</strong></span>
                    </div>
                </div>
            )}
        </div>
    );
}
