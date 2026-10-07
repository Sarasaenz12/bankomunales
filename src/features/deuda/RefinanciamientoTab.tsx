import { useState, useEffect, useMemo } from 'react';
import { deudaService } from './deudaService';
import { EstadoCreditoDeuda } from './types';
import { Credito } from '../creditos/types';
import { SocioResumen } from '../socios/types';
import { sociosService } from '../socios/sociosService';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';
import { HOY } from '../../core/lib/dates';
import { api } from '../../core/lib/api';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';

export function RefinanciamientoTab({ moneda }: { moneda: string }) {
    const { enviando, ejecutar } = useEnvioUnico();
    const [estado, setEstado] = useState<EstadoCreditoDeuda | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [tasaMora, setTasaMora] = useState(0);

    // ── Búsqueda en vivo ──
    const [query, setQuery] = useState('');
    const [seleccionado, setSeleccionado] = useState(false);
    const [creditos, setCreditos] = useState<Credito[]>([]);
    const [sociosPorId, setSociosPorId] = useState<Record<string, SocioResumen>>({});
    const [cargando, setCargando] = useState(false);

    // Formulario de refinanciamiento
    const [pagarEfectivo, setPagarEfectivo] = useState(true);
    const [montoNuevo, setMontoNuevo] = useState('');
    const [tasaNueva, setTasaNueva] = useState('3');
    const [plazoNuevo, setPlazoNuevo] = useState('12');

    useEffect(() => {
        api.obtenerConfiguracion().then(c => {
            setTasaMora(c.tasa_interes_mora);
            setTasaNueva(String(c.tasa_interes_ordinario));
        }).catch(() => { });
        deudaService.listarCreditos().then(setCreditos).catch(() => { });
        sociosService.listar()
            .then(ss => setSociosPorId(Object.fromEntries(ss.map(s => [s.id, s]))))
            .catch(() => { });
    }, []);

    // Créditos refinanciables (vigentes) enriquecidos con socio para poder buscar.
    const buscables = useMemo(() => creditos
        .filter(c => c.estatus === 'VIGENTE')
        .map(c => {
            const s = sociosPorId[c.socio_id];
            return { credito: c, nombre: s?.nombre_completo ?? '', cedula: s?.cedula ?? '' };
        }), [creditos, sociosPorId]);

    const q = query.trim().toLowerCase();
    const sugerencias = q.length > 0 && !seleccionado
        ? buscables.filter(x =>
            x.credito.numero.toLowerCase().includes(q) ||
            x.cedula.toLowerCase().includes(q) ||
            x.nombre.toLowerCase().includes(q),
        ).slice(0, 6)
        : [];

    async function seleccionar(numero: string, etiqueta: string) {
        setQuery(etiqueta);
        setSeleccionado(true);
        setError(''); setSuccess(''); setEstado(null);
        setCargando(true);
        try {
            const e = await deudaService.estadoCredito(numero.trim());
            if (e.credito.estatus !== 'VIGENTE') {
                setError(`El crédito debe estar VIGENTE para refinanciarse. Estatus actual: ${e.credito.estatus}`);
                return;
            }
            setEstado(e);
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo buscar el crédito');
        } finally {
            setCargando(false);
        }
    }

    function onCambioBusqueda(v: string) {
        setQuery(v);
        setSeleccionado(false);
        if (v.trim() === '') { setEstado(null); setSuccess(''); setError(''); }
    }

    // Cálculos de saldo pendiente y estimación a hoy
    const hoyStr = HOY();
    const hoyMs = new Date(hoyStr.slice(0, 10)).getTime();

    let capitalPendiente = 0;
    let interesPendiente = 0;

    if (estado) {
        // En refinanciamiento se cobra todo el saldo de capital del crédito
        capitalPendiente = estado.credito.saldo_pendiente;

        // El interés pendiente es sólo el DEVENGADO: cuotas cuyo vencimiento ya pasó. El
        // interés de las cuotas futuras se cancela con el crédito viejo (no se cobra ni se
        // capitaliza; cobrarlo sería interés no ganado). La mora existe sólo en las vencidas.
        estado.cuotas.filter(c => !c.pagada).forEach(c => {
            const vMs = new Date(c.fecha_vencimiento.slice(0, 10)).getTime();
            if (vMs <= hoyMs) {
                interesPendiente += (c.interes - c.interes_pagado);
            }
            const diasAtraso = Math.max(0, Math.floor((hoyMs - vMs) / 86400000));
            if (diasAtraso > 0) {
                const bruta = c.valor_total * (tasaMora / 100) / 30 * diasAtraso;
                interesPendiente += Math.max(0, Math.round((bruta - c.mora_pagada) * 100) / 100);
            }
        });
    }

    // Actualizar monto sugerido si cambian las opciones
    useEffect(() => {
        if (estado) {
            const minimoBase = pagarEfectivo ? capitalPendiente : capitalPendiente + interesPendiente;
            setMontoNuevo(String(Math.round(minimoBase * 100) / 100));
        }
    }, [estado, pagarEfectivo, capitalPendiente, interesPendiente]);

    async function refinanciar() {
        setError(''); setSuccess('');
        try {
            const nuevo = await api.registrarRefinanciamiento({
                credito_id_anterior: estado!.credito.id,
                monto: parseFloat(montoNuevo),
                plazo_cuotas: parseInt(plazoNuevo, 10),
                intereses_en_efectivo: pagarEfectivo,
                fiadores: [], // El backend copia los fiadores del crédito original
                fecha: HOY()
            });
            setSuccess(`Refinanciamiento exitoso. Nuevo crédito: ${nuevo.numero}`);
            setEstado(null);
            setQuery('');
            setSeleccionado(false);
        } catch (err: any) {
            setError(err?.toString() ?? 'Error al refinanciar crédito');
        }
    }

    const hayCredito = !!estado;

    return (
        <div className="detalle-card" style={{ maxWidth: 800 }}>
            <div className="form-group custom-fg mb-4" style={{ position: 'relative' }}>
                <label className="form-label" style={{ fontWeight: 600 }}>Buscar crédito a refinanciar (por socio, cédula o Nº de crédito)</label>
                <input
                    className="form-input"
                    value={query}
                    onChange={e => onCambioBusqueda(e.target.value)}
                    placeholder="Ej: CR-005 o Pedro Jiménez"
                    autoComplete="off"
                />
                {sugerencias.length > 0 && (
                    <div
                        className="detalle-card"
                        style={{
                            position: 'absolute', left: 0, right: 0, top: '100%', zIndex: 20,
                            padding: '0.25rem', marginTop: '0.25rem', marginBottom: 0,
                            maxHeight: 260, overflowY: 'auto', boxShadow: '0 6px 18px rgba(0,0,0,0.12)',
                        }}
                    >
                        {sugerencias.map(x => (
                            <button
                                key={x.credito.id}
                                className="chip"
                                style={{ display: 'flex', justifyContent: 'space-between', gap: '1rem', width: '100%', textAlign: 'left', marginBottom: 2 }}
                                onClick={() => seleccionar(x.credito.numero, `${x.nombre || 'Socio'} — ${x.credito.numero}`)}
                            >
                                <span>
                                    <strong>{x.credito.numero}</strong>
                                    {x.nombre && ` · ${x.nombre}`}
                                    {x.cedula && <span className="text-muted"> · {x.cedula}</span>}
                                </span>
                                <span className="text-muted">{fmt(x.credito.saldo_pendiente, moneda)}</span>
                            </button>
                        ))}
                    </div>
                )}
                {q.length > 0 && !seleccionado && !cargando && sugerencias.length === 0 && (
                    <div className="text-sm text-muted mt-1">No se encontró ningún crédito vigente con esos datos.</div>
                )}
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            {/* ── Estado del crédito: siempre visible (con placeholders si no hay crédito) ── */}
            <div className="detalle-card mt-4" style={{ background: '#f1f5f9', border: 'none' }}>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '0.75rem', fontSize: '1.05rem', fontWeight: 500 }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid #e2e8f0', paddingBottom: '0.5rem' }}>
                        <span>Saldo de capital pendiente</span>
                        <strong>{fmt(hayCredito ? capitalPendiente : 0, moneda)}</strong>
                    </div>
                    <div style={{ display: 'flex', justifyContent: 'space-between', borderBottom: '1px solid #e2e8f0', paddingBottom: '0.5rem' }}>
                        <span>Intereses pendientes (Ordinario + Mora)</span>
                        <strong>{fmt(hayCredito ? interesPendiente : 0, moneda)}</strong>
                    </div>
                </div>

                <div style={{borderTop: '1px solid #e2e8f0', paddingTop: '0.8rem', opacity: hayCredito ? 1 : 0.6 }}>
                    <label className="form-label" style={{ fontWeight: 600, color: 'var(--primary-color)' }}>¿Pagar intereses en efectivo ahora?</label>
                    <div style={{ display: 'flex', gap: '2rem', marginTop: '0.75rem' }}>
                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', cursor: hayCredito ? 'pointer' : 'not-allowed' }}>
                            <input type="radio" name="pagoEf" checked={pagarEfectivo} disabled={!hayCredito} onChange={() => setPagarEfectivo(true)} />
                            Sí
                        </label>
                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.4rem', cursor: hayCredito ? 'pointer' : 'not-allowed' }}>
                            <input type="radio" name="pagoEf" checked={!pagarEfectivo} disabled={!hayCredito} onChange={() => setPagarEfectivo(false)} />
                            No
                        </label>
                    </div>
                    <div className="text-xs text-muted mt-3" style={{ fontFamily: 'monospace', padding: '0.4rem'}}>
                        Sí: los intereses se pagan en efectivo aparte · No: se suman al nuevo crédito (capitalizados)
                    </div>
                </div>

                {cargando && (
                    <div className="text-sm text-muted mt-3">Buscando crédito…</div>
                )}
            </div>

            {/* ── Nuevo crédito: siempre visible (inputs deshabilitados si no hay crédito) ── */}
            <div className="mt-8">
                <h4 className="section-title text-sm" style={{ color: 'var(--primary-color)' }}>NUEVO CRÉDITO</h4>
                <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                    <div className="form-group custom-fg">
                        <label className="form-label">Monto (Mín. sugerido)</label>
                        <input className="form-input" type="text" inputMode="numeric" disabled={!hayCredito}
                            value={hayCredito && montoNuevo ? fmtNum(parseNum(montoNuevo)) : ''}
                            placeholder={fmt(0, moneda)}
                            onChange={e => setMontoNuevo(e.target.value.replace(/\D/g, ''))} />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Tasa (%)</label>
                        <input className="form-input" type="number" step="0.1" disabled={!hayCredito} value={tasaNueva} onChange={e => setTasaNueva(e.target.value)} />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Plazo (cuotas)</label>
                        <input className="form-input" type="number" disabled={!hayCredito} value={plazoNuevo} onChange={e => setPlazoNuevo(e.target.value)} />
                    </div>
                </div>

                <div className="mt-6">
                    <button className="btn btn-primary" onClick={() => ejecutar(refinanciar)} disabled={enviando || !hayCredito}>
                        {enviando ? 'Refinanciando…' : 'Confirmar refinanciamiento'}
                    </button>
                </div>
            </div>
        </div>
    );
}
