import { useEffect, useRef, useState } from 'react';
import { creditosService } from './creditosService';
import { TablaAmortizacion } from './TablaAmortizacion';
import { DetalleCredito } from './DetalleCredito';
import { HOY } from '../../core/lib/dates';
import { fmt } from '../../core/lib/format';
import { SocioResumen } from '../socios/types';
import { sociosService } from '../socios/sociosService';
import { NuevoDesembolso, SolicitudCredito, TablaCredito } from './types';

export function DesembolsoTab({ moneda }: { moneda: string }) {
    const [solicitudes, setSolicitudes] = useState<SolicitudCredito[]>([]);
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [solicitudId, setSolicitudId] = useState('');
    const [fecha, setFecha] = useState(HOY());
    const [numeroPreview, setNumeroPreview] = useState('');
    const [verDetalle, setVerDetalle] = useState(false);
    const [tabla, setTabla] = useState<TablaCredito | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);
    const enviando = useRef(false);

    function recargar() {
        creditosService.listarSolicitudesDesembolsables().then(setSolicitudes).catch(() => {});
        sociosService.listar().then(setSocios).catch(() => {});
        creditosService.siguienteNumeroCredito().then(setNumeroPreview).catch(() => setNumeroPreview(''));
    }
    useEffect(() => { recargar(); }, []);

    const nombreSocio = (id: string) => socios.find(s => s.id === id)?.nombre_completo ?? id;

    const sol = solicitudes.find(s => s.id === solicitudId) ?? null;
    const monto = sol ? (sol.monto_aprobado ?? sol.monto_solicitado) : 0;
    const plazo = sol ? sol.plazo_cuotas : 0;

    // La tabla de pagos (saldo decreciente, RN-12) la calcula el backend.
    useEffect(() => {
        if (!monto || !plazo) { setTabla(null); return; }
        let vigente = true;
        creditosService.previsualizarDesembolso(monto, plazo)
            .then(t => { if (vigente) setTabla(t); })
            .catch(() => { if (vigente) setTabla(null); });
        return () => { vigente = false; };
    }, [monto, plazo]);

    async function confirmar() {
        if (enviando.current || guardando) return;
        if (!sol) { setError('Elija un crédito aprobado para desembolsar'); return; }
        setError(''); setSuccess('');
        enviando.current = true;
        setGuardando(true);
        try {
            // Al venir de una solicitud aprobada, el backend toma los fiadores y valida las
            // reglas (RN-03/04/05) de nuevo; aquí sólo se confirma.
            const des: NuevoDesembolso = {
                solicitud_id: sol.id,
                socio_id: sol.socio_id,
                monto,
                plazo_cuotas: plazo,
                destino: sol.destino,
                fiadores: [],
                fecha,
            };
            const credito = await creditosService.registrarDesembolso(des);
            setSuccess(`✓ Crédito Nº ${credito.numero} desembolsado por ${fmt(credito.monto_original, moneda)}.`);
            setSolicitudId('');
            setTabla(null);
            recargar();
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo desembolsar');
        } finally {
            enviando.current = false;
            setGuardando(false);
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            {/* ── Crédito a desembolsar ── */}
            <div className="form-group custom-fg">
                <label className="form-label">Crédito aprobado</label>
                <select className="form-input" value={solicitudId} onChange={e => { setSolicitudId(e.target.value); setError(''); setSuccess(''); setVerDetalle(false); }}>
                    <option value="">Seleccione un crédito aprobado…</option>
                    {solicitudes.map(s => (
                        <option key={s.id} value={s.id}>
                            {nombreSocio(s.socio_id)} — {fmt(s.monto_aprobado ?? s.monto_solicitado, moneda)} — {s.plazo_cuotas} cuotas
                        </option>
                    ))}
                </select>
            </div>

            {solicitudes.length === 0 && (
                <div className="text-sm text-muted mt-4">
                    No hay solicitudes aprobadas pendientes de desembolso. Apruebe una en la Bandeja de Solicitudes.
                </div>
            )}

            {sol && (
                <>
                    {/* ── Datos del crédito (sólo lectura; las validaciones ya se hicieron al
                          crear y aprobar la solicitud) ── */}
                    <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                        <div className="form-group custom-fg">
                            <label className="form-label">N° de crédito</label>
                            <input className="form-input" readOnly value={numeroPreview || '—'}
                                title="Número que se asignará al confirmar el desembolso" />
                        </div>
                        <div className="form-group custom-fg">
                            <label className="form-label">Monto a desembolsar</label>
                            <input className="form-input" readOnly value={fmt(monto, moneda)} />
                        </div>
                    </div>

                    <div className="mt-4" style={{ display: 'flex', gap: '1rem', alignItems: 'flex-end', flexWrap: 'wrap' }}>
                        <div className="form-group custom-fg" style={{ maxWidth: 220, marginBottom: 0 }}>
                            <label className="form-label">Fecha de desembolso</label>
                            <input className="form-input" type="date" value={fecha} onChange={e => setFecha(e.target.value)} />
                        </div>
                        <button className="btn btn-secondary" onClick={() => setVerDetalle(v => !v)}>
                            {verDetalle ? 'Ocultar crédito' : 'Ver crédito'}
                        </button>
                    </div>

                    {verDetalle && (
                        <DetalleCredito solicitud={sol} moneda={moneda} nombreSocio={nombreSocio} />
                    )}

                    {/* ── Tabla de pagos ── */}
                    <h3 className="section-title mt-4">TABLA DE PAGOS</h3>
                    {tabla ? (
                        <>
                            <TablaAmortizacion cuotas={tabla.cuotas} monto={monto} moneda={moneda} />
                            <div className="text-sm mt-4" style={{ display: 'flex', justifyContent: 'space-between' }}>
                                <span>Total a pagar: <strong>{fmt(tabla.monto_total, moneda)}</strong></span>
                                <span>Cuota mensual: <strong>{fmt(tabla.cuota_mensual, moneda)}</strong></span>
                            </div>
                        </>
                    ) : (
                        <div className="text-sm text-muted">Calculando la tabla de pagos…</div>
                    )}
                </>
            )}

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            <button className="btn btn-primary mt-4" onClick={confirmar} disabled={guardando || !sol}>
                {guardando ? 'Desembolsando…' : 'Confirmar desembolso'}
            </button>
        </div>
    );
}
