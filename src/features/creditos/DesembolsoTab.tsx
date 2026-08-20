import { useEffect, useState } from 'react';
import { creditosService, cumpleRn03, cumpleRn04Socio, cumpleRn04Fiadores, maximoRn03 } from './creditosService';
import { ItemValidacion } from './ItemValidacion';
import { TablaAmortizacion } from './TablaAmortizacion';
import { BuscadorSocio } from '../../core/components/BuscadorSocio';
import { HOY } from '../../core/lib/dates';
import { fmt } from '../../core/lib/format';
import { Configuracion } from '../../core/types';
import { SocioResumen } from '../socios/types';
import { sociosService } from '../socios/sociosService';
import {
    DestinoCredito, DESTINOS_CREDITO, NuevoDesembolso, SolicitudCredito, TablaCredito,
} from './types';

export function DesembolsoTab({ moneda }: { moneda: string }) {
    const [solicitudes, setSolicitudes] = useState<SolicitudCredito[]>([]);
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [solicitudId, setSolicitudId] = useState('');
    const [solicitante, setSolicitante] = useState<SocioResumen | null>(null);
    const [fecha, setFecha] = useState(HOY());
    const [destino, setDestino] = useState<DestinoCredito | ''>('');
    const [monto, setMonto] = useState('');
    const [plazo, setPlazo] = useState('');
    const [fiador, setFiador] = useState<SocioResumen | null>(null);
    const [accionesFiador, setAccionesFiador] = useState('');
    const [accionesReales, setAccionesReales] = useState(0);
    const [accionesRealesFiador, setAccionesRealesFiador] = useState(0);
    const [config, setConfig] = useState<Configuracion | null>(null);
    const [tabla, setTabla] = useState<TablaCredito | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);

    function recargar() {
        creditosService.listarSolicitudesDesembolsables().then(setSolicitudes).catch(() => {});
        sociosService.listar().then(l => setSocios(l.filter(s => s.estatus === 'ACTIVO'))).catch(() => {});
    }
    useEffect(() => {
        recargar();
        creditosService.obtenerConfiguracion().then(setConfig).catch(() => {});
    }, []);

    useEffect(() => {
        if (!solicitante) { setAccionesReales(0); return; }
        let vigente = true;
        creditosService.accionesDeSocio(solicitante.id)
            .then(n => { if (vigente) setAccionesReales(n); })
            .catch(() => { if (vigente) setAccionesReales(0); });
        return () => { vigente = false; };
    }, [solicitante]);

    useEffect(() => {
        if (!fiador) { setAccionesRealesFiador(0); return; }
        let vigente = true;
        creditosService.accionesDeSocio(fiador.id)
            .then(n => { if (vigente) setAccionesRealesFiador(n); })
            .catch(() => { if (vigente) setAccionesRealesFiador(0); });
        return () => { vigente = false; };
    }, [fiador]);

    // RF-59: la tasa es la configurada; la tabla la calcula el backend.
    useEffect(() => {
        const importe = Number(monto);
        const cuotas = Number(plazo);
        if (!importe || importe <= 0 || !cuotas || cuotas <= 0) { setTabla(null); return; }
        let vigente = true;
        creditosService.previsualizarDesembolso(importe, cuotas)
            .then(t => { if (vigente) { setTabla(t); setError(''); } })
            .catch(e => { if (vigente) { setTabla(null); setError(e?.toString() ?? ''); } });
        return () => { vigente = false; };
    }, [monto, plazo]);

    function elegirSolicitud(id: string) {
        setSolicitudId(id);
        const sol = solicitudes.find(s => s.id === id);
        if (!sol) return;
        const socio = socios.find(s => s.id === sol.socio_id);
        setSolicitante(socio ?? null);
        setMonto(String(sol.monto_aprobado ?? sol.monto_solicitado));
        setPlazo(String(sol.plazo_cuotas));
        setDestino(sol.destino);
    }

    async function desembolsar() {
        if (!solicitante) { setError('Debe seleccionar el socio'); return; }
        if (!destino) { setError('Debe seleccionar el destino'); return; }
        if (!Number(monto) || Number(monto) <= 0) { setError('El monto debe ser mayor a cero'); return; }
        if (!Number(plazo) || Number(plazo) <= 0) { setError('El plazo debe ser de al menos 1 cuota'); return; }

        const importe = Number(monto);
        const pSocio = config?.pct_garantia_socio ?? 20;
        const pFiador = config?.pct_garantia_fiador ?? 20;
        if (!solicitudId) {
            if (!cumpleRn03(importe, accionesReales, config)) {
                setError('El monto supera el cupo de la relación 1 a 5: máx. ' + fmt(maximoRn03(accionesReales, config), moneda));
                return;
            }
            if (!cumpleRn04Socio(importe, accionesReales, config)) {
                setError(`El solicitante no cubre su ${pSocio}% con sus acciones`);
                return;
            }
            if (!fiador || !cumpleRn04Fiadores(importe, Number(accionesFiador) || 0, config)) {
                setError(`Los fiadores no cubren su ${pFiador}% con las acciones comprometidas`);
                return;
            }
        }

        setError(''); setSuccess(''); setGuardando(true);
        try {
            const des: NuevoDesembolso = {
                solicitud_id: solicitudId || null,
                socio_id: solicitante.id,
                monto: Number(monto),
                plazo_cuotas: Number(plazo),
                destino: destino as DestinoCredito,
                fiadores: fiador ? [{ cedula: fiador.cedula, acciones_comprometidas: Number(accionesFiador) || 0 }] : [],
                fecha,
            };
            const credito = await creditosService.registrarDesembolso(des);
            setSuccess(`Crédito Nº ${credito.numero} desembolsado por ${fmt(credito.monto_original, moneda)}.`);
            setMonto(''); setPlazo(''); setSolicitudId(''); setTabla(null);
            recargar();
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo desembolsar');
        } finally {
            setGuardando(false);
        }
    }

    const vn = config?.valor_nominal ?? 10000;
    const pctSocio = config?.pct_garantia_socio ?? 20;
    const pctFiador = config?.pct_garantia_fiador ?? 20;

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            <h3 className="section-title">ORIGEN</h3>
            <div className="form-group custom-fg">
                <label className="form-label">Desembolsar desde solicitud aprobada</label>
                <select className="form-input" value={solicitudId} onChange={e => elegirSolicitud(e.target.value)}>
                    <option value="">— Directo, sin solicitud —</option>
                    {solicitudes.map(sol => {
                        const socio = socios.find(s => s.id === sol.socio_id);
                        return (
                            <option key={sol.id} value={sol.id}>
                                {socio?.nombre_completo ?? sol.socio_id} · {fmt(sol.monto_aprobado ?? sol.monto_solicitado, moneda)} · {sol.plazo_cuotas} cuotas
                            </option>
                        );
                    })}
                </select>
            </div>

            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div>
                    <label className="form-label">Socio</label>
                    <BuscadorSocio
                        placeholder="Buscar socio…"
                        socios={socios}
                        valor={solicitante ? `${solicitante.nombre_completo} — ${solicitante.cedula}` : ''}
                        onSelect={s => { setSolicitante(s); setSolicitudId(''); }}
                    />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} />
                </div>
            </div>

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input className="form-input" inputMode="numeric" placeholder="Ej: 500000"
                        value={monto} onChange={e => setMonto(e.target.value.replace(/\D/g, ''))} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Plazo (cuotas)</label>
                    <input className="form-input" inputMode="numeric" placeholder="Ej: 12"
                        value={plazo} onChange={e => setPlazo(e.target.value.replace(/\D/g, ''))} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Destino del crédito</label>
                    <select className="form-input" value={destino} onChange={e => setDestino(e.target.value as DestinoCredito)}>
                        <option value="">Seleccione…</option>
                        {DESTINOS_CREDITO.map(d => (
                            <option key={d.codigo} value={d.codigo}>{d.etiqueta}</option>
                        ))}
                    </select>
                </div>
            </div>

            {!solicitudId && (
                <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                    <div>
                        <label className="form-label">Fiador</label>
                        <BuscadorSocio
                            placeholder="Buscar socio…"
                            socios={socios}
                            valor={fiador ? `${fiador.nombre_completo} — ${fiador.cedula}` : ''}
                            onSelect={setFiador}
                        />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Acciones en garantía (fiador)</label>
                        <input className="form-input" inputMode="numeric"
                            value={accionesFiador} onChange={e => setAccionesFiador(e.target.value.replace(/\D/g, ''))} />
                    </div>
                </div>
            )}

            {!solicitudId && (
                <>
                    <h3 className="section-title mt-4">VALIDACIONES AUTOMÁTICAS</h3>
                    <div style={{ display: 'grid', gap: '0.5rem' }}>
                        <ItemValidacion
                            ok={!solicitante || cumpleRn03(Number(monto) || 0, accionesReales, config)}
                            titulo="Relación 1 a 5"
                            detalle={solicitante
                                ? `Desembolsa ${fmt(Number(monto) || 0, moneda)} → máx. ${fmt(maximoRn03(accionesReales, config), moneda)} (${accionesReales} acciones × 5)`
                                : 'Seleccione el socio'}
                        />
                        <ItemValidacion
                            ok={!solicitante || cumpleRn04Socio(Number(monto) || 0, accionesReales, config)}
                            titulo={`Cobertura del solicitante ${pctSocio}%`}
                            detalle={solicitante
                                ? `${fmt(accionesReales * vn, moneda)} (${accionesReales} acciones) ${cumpleRn04Socio(Number(monto) || 0, accionesReales, config) ? '≥' : '<'} ${pctSocio}% × ${fmt(Number(monto) || 0, moneda)}`
                                : 'Seleccione el socio'}
                        />
                        <ItemValidacion
                            ok={!fiador || cumpleRn04Fiadores(Number(monto) || 0, Number(accionesFiador) || 0, config)}
                            titulo={`Cobertura del fiador ${pctFiador}%`}
                            detalle={fiador
                                ? `${fmt((Number(accionesFiador) || 0) * vn, moneda)} (${Number(accionesFiador) || 0} acciones comprometidas; tiene ${accionesRealesFiador}) ${cumpleRn04Fiadores(Number(monto) || 0, Number(accionesFiador) || 0, config) ? '≥' : '<'} ${pctFiador}% × ${fmt(Number(monto) || 0, moneda)}`
                                : 'Seleccione el fiador'}
                        />
                        <ItemValidacion
                            ok={!fiador || !solicitante || fiador.id !== solicitante.id}
                            titulo="Sin fiadores cruzados"
                            detalle={!fiador || !solicitante || fiador.id !== solicitante.id
                                ? 'El fiador no coincide con el solicitante'
                                : 'Seleccione un fiador distinto del solicitante'}
                        />
                    </div>
                </>
            )}

            <h3 className="section-title mt-4">VISTA PREVIA — TABLA DE PAGOS</h3>
            {tabla ? (
                <TablaAmortizacion cuotas={tabla.cuotas} monto={Number(monto)} moneda={moneda} />
            ) : (
                <div className="text-sm text-muted">Complete monto y plazo para ver la tabla.</div>
            )}

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            <button className="btn btn-primary mt-4" onClick={desembolsar} disabled={guardando}>
                {guardando ? 'Desembolsando…' : 'Desembolsar crédito'}
            </button>
        </div>
    );
}