import { useEffect, useState } from 'react';
import { creditosService } from './creditosService';
import { DetalleCredito } from './DetalleCredito';
import { ETIQUETA_ESTADO, DESTINOS_CREDITO, Credito, SolicitudCredito, EstadoSolicitud, DecisionSolicitud } from './types';
import { fmt } from '../../core/lib/format';
import { SocioResumen } from '../socios/types';
import { sociosService } from '../socios/sociosService';

export function BandejaTab({ moneda }: { moneda: string }) {
    const [solicitudes, setSolicitudes] = useState<SolicitudCredito[]>([]);
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [filtro, setFiltro] = useState<EstadoSolicitud | 'TODAS'>('TODAS');
    const [decisiones, setDecisiones] = useState<Record<string, string>>({});
    const [observaciones, setObservaciones] = useState<Record<string, string>>({});
    const [montosAprobados, setMontosAprobados] = useState<Record<string, string>>({});
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [creditoVisto, setCreditoVisto] = useState<{ solId: string; credito: Credito | null } | null>(null);

    function recargar() {
        creditosService.listarSolicitudes(null)
            .then(setSolicitudes)
            .catch(e => setError(e?.toString() ?? 'Error al cargar solicitudes'));
        sociosService.listar().then(setSocios).catch(() => {});
    }
    useEffect(() => { recargar(); }, []);

    const nombreSocio = (id: string) => {
        const s = socios.find(x => x.id === id);
        return s ? s.nombre_completo : id;
    };

    const visibles = filtro === 'TODAS' ? solicitudes : solicitudes.filter(s => s.estado === filtro);

    async function verCredito(sol: SolicitudCredito) {
        if (creditoVisto?.solId === sol.id) { setCreditoVisto(null); return; }
        setError(''); setSuccess('');
        try {
            const credito = await creditosService.buscarCreditoPorSolicitud(sol.id);
            setCreditoVisto({ solId: sol.id, credito });
            if (!credito) setError('Esta solicitud todavía no tiene un crédito desembolsado.');
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo consultar el crédito');
        }
    }

    async function decidir(sol: SolicitudCredito) {
        const decision = decisiones[sol.id];
        if (!decision) { setError('Elija la decisión de la Junta'); return; }
        setError(''); setSuccess('');
        try {
            const d: DecisionSolicitud = {
                solicitud_id: sol.id,
                decision: decision as EstadoSolicitud,
                monto_aprobado: decision === 'MODIFICADA' ? Number(montosAprobados[sol.id]) || null : null,
                observacion: observaciones[sol.id] || null,
                decidida_por: 'Junta Directiva',
            };
            const actualizada = await creditosService.decidir(d);
            setSuccess(`Solicitud ${actualizada.estado}: ${actualizada.monto_aprobado ? fmt(actualizada.monto_aprobado, moneda) : ''} ${actualizada.observacion ? `— ${actualizada.observacion}` : ''}`.trim());
            recargar();
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo registrar la decisión');
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            <div className="chip-row">
                {(['TODAS', 'PENDIENTE', 'APROBADA', 'MODIFICADA', 'NEGADA', 'DIFERIDA'] as const).map(f => (
                    <button key={f} className={`chip ${filtro === f ? 'active' : ''}`} onClick={() => setFiltro(f)}>
                        {f === 'TODAS' ? `Todas (${solicitudes.length})` : `${ETIQUETA_ESTADO[f]} (${solicitudes.filter(s => s.estado === f).length})`}
                    </button>
                ))}
            </div>

            {visibles.length === 0 && <div className="empty-state">No hay solicitudes {filtro !== 'TODAS' ? ETIQUETA_ESTADO[filtro as EstadoSolicitud].toLowerCase() : ''}.</div>}

            {visibles.map(sol => (
                <div key={sol.id} className="detalle-card" style={{ padding: '1.25rem', marginBottom: '1rem' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '0.5rem' }}>
                        <div>
                            <strong>{nombreSocio(sol.socio_id)}</strong>
                            <div className="text-sm text-muted">{sol.fecha_solicitud}</div>
                        </div>
                        <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                            <div className="text-sm">
                                <div>Solicita: <strong>{fmt(sol.monto_solicitado, moneda)}</strong></div>
                                <div>{sol.plazo_cuotas} cuotas · {DESTINOS_CREDITO.find(d => d.codigo === sol.destino)?.etiqueta ?? sol.destino}</div>
                            </div>
                            <span className={`badge-estatus ${sol.estado === 'PENDIENTE' ? 'badge-activo' : sol.estado === 'DIFERIDA' ? 'badge-deuda' : 'badge-retirado'}`}>
                                {ETIQUETA_ESTADO[sol.estado]}
                            </span>
                            {sol.estado !== 'PENDIENTE' && (
                                <button className="btn btn-secondary" onClick={() => verCredito(sol)}>
                                    {creditoVisto?.solId === sol.id ? 'Ocultar crédito' : 'Ver crédito'}
                                </button>
                            )}
                        </div>
                    </div>

                    {sol.estado === 'PENDIENTE' && (
                        <div className="mt-4">
                            <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                                {(['APROBADA', 'MODIFICADA', 'NEGADA', 'DIFERIDA'] as const).map(d => (
                                    <button
                                        key={d}
                                        className={`chip ${decisiones[sol.id] === d ? 'active' : ''}`}
                                        onClick={() => setDecisiones({ ...decisiones, [sol.id]: d })}
                                    >
                                        {ETIQUETA_ESTADO[d]}
                                    </button>
                                ))}
                            </div>

                            {decisiones[sol.id] === 'MODIFICADA' && (
                                <div className="form-group custom-fg mt-4" style={{ maxWidth: 300 }}>
                                    <label className="form-label">Monto aprobado</label>
                                    <input className="form-input" inputMode="numeric" placeholder="Ej: 400000"
                                        value={montosAprobados[sol.id] ?? ''}
                                        onChange={e => setMontosAprobados({ ...montosAprobados, [sol.id]: e.target.value.replace(/\D/g, '') })} />
                                </div>
                            )}
                            {decisiones[sol.id] === 'DIFERIDA' && (
                                <div className="form-group custom-fg mt-4">
                                    <label className="form-label">Observación (obligatoria)</label>
                                    <input className="form-input" placeholder="Motivo de la diferida (RF-51)"
                                        value={observaciones[sol.id] ?? ''}
                                        onChange={e => setObservaciones({ ...observaciones, [sol.id]: e.target.value })} />
                                </div>
                            )}

                            <button className="btn btn-primary mt-4" onClick={() => decidir(sol)}>
                                Registrar decisión
                            </button>
                        </div>
                    )}

                    {sol.estado !== 'PENDIENTE' && sol.monto_aprobado && (
                        <div className="text-sm text-muted mt-4">
                            Aprobado: <strong>{fmt(sol.monto_aprobado, moneda)}</strong>
                            {sol.observacion && ` — ${sol.observacion}`}
                            {sol.decidida_por && ` · por ${sol.decidida_por}`}
                        </div>
                    )}

                    {creditoVisto?.solId === sol.id && creditoVisto.credito && (
                        <DetalleCredito
                            credito={creditoVisto.credito}
                            moneda={moneda}
                            nombreSocio={nombreSocio}
                            onClose={() => setCreditoVisto(null)}
                        />
                    )}
                </div>
            ))}

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}
        </div>
    );
}