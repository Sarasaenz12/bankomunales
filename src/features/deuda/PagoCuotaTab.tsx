import { useEffect, useMemo, useState } from 'react';
import { deudaService } from './deudaService';
import { Cuota, EstadoCreditoDeuda, ResultadoAbono, ResultadoPago } from './types';
import { Credito } from '../creditos/types';
import { SocioResumen } from '../socios/types';
import { sociosService } from '../socios/sociosService';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';
import { HOY } from '../../core/lib/dates';
import { api } from '../../core/lib/api';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';
import { mensajeDeError } from '../../core/lib/errores';

type ModoPago = 'cuota' | 'abono';

export function PagoCuotaTab({ moneda }: { moneda: string }) {
    const [estado, setEstado] = useState<EstadoCreditoDeuda | null>(null);
    const [tasaMora, setTasaMora] = useState(0);
    const [error, setError] = useState('');
    const [resultadoPago, setResultadoPago] = useState<ResultadoPago | null>(null);
    const [resultadoAbono, setResultadoAbono] = useState<ResultadoAbono | null>(null);

    // ── Búsqueda en vivo ──
    const [query, setQuery] = useState('');
    const [seleccionado, setSeleccionado] = useState(false);
    const [activo, setActivo] = useState(0);
    const [creditos, setCreditos] = useState<Credito[]>([]);
    const [sociosPorId, setSociosPorId] = useState<Record<string, SocioResumen>>({});
    const [cargando, setCargando] = useState(false);

    const [fecha, setFecha] = useState(HOY());
    const { enviando, ejecutar } = useEnvioUnico();

    // ── Modo de pago ──
    const [modoPago, setModoPago] = useState<ModoPago>('cuota');
    const [cantidadCuotas, setCantidadCuotas] = useState(1);
    const [montoAbonoTxt, setMontoAbonoTxt] = useState('');

    useEffect(() => {
        api.obtenerConfiguracion().then(c => setTasaMora(c.tasa_interes_mora)).catch(() => { });
        // Precargamos créditos y socios para poder autocompletar por número, cédula o nombre.
        deudaService.listarCreditos().then(setCreditos).catch(() => { });
        sociosService.listar()
            .then(ss => setSociosPorId(Object.fromEntries(ss.map(s => [s.id, s]))))
            .catch(() => { });
    }, []);

    // Créditos pagables (vigentes) enriquecidos con el nombre/cédula del socio para buscar.
    const buscables = useMemo(() => creditos
        .filter(c => c.estatus === 'VIGENTE')
        .map(c => {
            const s = sociosPorId[c.socio_id];
            return {
                credito: c,
                nombre: s?.nombre_completo ?? '',
                cedula: s?.cedula ?? '',
            };
        }), [creditos, sociosPorId]);

    const q = query.trim().toLowerCase();
    const sugerencias = q.length > 0 && !seleccionado
        ? buscables.filter(x =>
            x.credito.numero.toLowerCase().includes(q) ||
            x.cedula.toLowerCase().includes(q) ||
            x.nombre.toLowerCase().includes(q),
        ).slice(0, 6)
        : [];

    function limpiarResultados() {
        setError(''); setResultadoPago(null); setResultadoAbono(null);
    }

    async function seleccionar(numero: string, etiqueta: string) {
        setQuery(etiqueta);
        setSeleccionado(true);
        limpiarResultados(); setEstado(null);
        setModoPago('cuota'); setCantidadCuotas(1); setMontoAbonoTxt('');
        setCargando(true);
        try {
            setEstado(await deudaService.estadoCredito(numero.trim()));
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo buscar el crédito'));
        } finally {
            setCargando(false);
        }
    }

    function onCambioBusqueda(v: string) {
        setQuery(v);
        setSeleccionado(false);
        setActivo(0);
        if (v.trim() === '') { setEstado(null); limpiarResultados(); }
    }

    function onKeyDownBusqueda(e: React.KeyboardEvent<HTMLInputElement>) {
        if (sugerencias.length === 0) return;
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            setActivo(i => Math.min(i + 1, sugerencias.length - 1));
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            setActivo(i => Math.max(i - 1, 0));
        } else if (e.key === 'Enter') {
            e.preventDefault();
            const x = sugerencias[Math.min(activo, sugerencias.length - 1)];
            if (x) seleccionar(x.credito.numero, `${x.nombre || 'Socio'} — ${x.credito.numero}`);
        } else if (e.key === 'Escape') {
            setSeleccionado(true);
        }
    }

    function cambiarModo(modo: ModoPago) {
        setModoPago(modo);
        limpiarResultados();
        setCantidadCuotas(1);
        setMontoAbonoTxt('');
    }

    /** Último día del mes de una fecha (YYYY-MM-DD). */
    function finDeMes(fecha: string): string {
        const [y, m] = fecha.slice(0, 10).split('-').map(Number);
        const ultimo = new Date(y, m, 0).getDate(); // día 0 del mes siguiente = último del actual
        return `${y}-${String(m).padStart(2, '0')}-${String(ultimo).padStart(2, '0')}`;
    }

    /** Días de mora de una cuota: se cuentan desde el fin del mes de vencimiento (hay plazo
     *  hasta fin de mes para pagar en la reunión; la mora corre desde el 1 del mes siguiente). */
    function diasMora(vencimiento: string, hoyStr: string): number {
        const v = new Date(finDeMes(vencimiento)).getTime();
        const h = new Date(hoyStr.slice(0, 10)).getTime();
        return Math.max(0, Math.floor((h - v) / 86400000));
    }

    /** Mora estimada a la fecha de pago (el backend la recalcula al registrar). */
    function moraEstimada(c: Cuota, fechaPago: string): number {
        const dias = diasMora(c.fecha_vencimiento, fechaPago);
        const bruta = c.valor_total * (tasaMora / 100) / 30 * dias;
        return Math.max(0, Math.round((bruta - c.mora_pagada) * 100) / 100);
    }

    // ── Cuotas y crédito ──
    const cuotasPendientes = estado?.cuotas.filter(c => !c.pagada) || [];
    const cuotaActual = cuotasPendientes.length > 0 ? cuotasPendientes[0] : null;
    const totalCuotas = estado?.cuotas.length ?? 0;
    const hayCredito = !!estado && !!cuotaActual;

    // ── Modo "pago de cuota": N cuotas = un solo interés (el del mes) + mora + N capitales ──
    const n = Math.max(1, Math.min(cantidadCuotas, cuotasPendientes.length || 1));
    const cuotasAPagar = cuotasPendientes.slice(0, n);
    const moraActual = cuotaActual ? moraEstimada(cuotaActual, fecha) : 0;
    const interesMes = cuotaActual ? (cuotaActual.interes - cuotaActual.interes_pagado) : 0;
    const capitalN = cuotasAPagar.reduce((acc, c) => acc + (c.capital - c.capital_pagado), 0);
    const totalCuota = moraActual + interesMes + capitalN;

    // ── Modo "abono": todo a capital; re-amortiza (mismo capital de referencia por cuota) ──
    const montoAbono = parseNum(montoAbonoTxt);
    const saldoActual = estado?.credito.saldo_pendiente ?? 0;
    const capitalRef = estado && estado.credito.plazo_cuotas > 0
        ? Math.round(estado.credito.monto_original / estado.credito.plazo_cuotas)
        : 0;
    const nuevoSaldoAbono = Math.max(0, saldoActual - montoAbono);
    const cuotasRestantesAbono = nuevoSaldoAbono <= 0 || capitalRef <= 0
        ? 0
        : Math.ceil(nuevoSaldoAbono / capitalRef);

    // Valor grande de la caja de desglose según el modo.
    const valorMostrado = modoPago === 'abono' ? montoAbono : (hayCredito ? totalCuota : 0);

    const fechaVencMostrada = hayCredito
        ? cuotasAPagar[cuotasAPagar.length - 1].fecha_vencimiento.slice(0, 10).split('-').reverse().join('/')
        : '—';

    async function pagar() {
        if (!estado || !cuotaActual) return;
        limpiarResultados();

        // ── Abono a capital ──
        if (modoPago === 'abono') {
            if (montoAbono <= 0) { setError('Ingrese el monto del abono'); return; }
            if (montoAbono > saldoActual + 0.01) {
                setError(`El abono no puede superar el saldo del crédito (${fmt(saldoActual, moneda)}).`);
                return;
            }
            try {
                const r = await deudaService.abonarCapital(estado.credito.numero, fecha, montoAbono);
                setResultadoAbono(r);
                setEstado(null); setQuery(''); setSeleccionado(false);
                setModoPago('cuota'); setMontoAbonoTxt(''); setCantidadCuotas(1);
            } catch (e) {
                setError(mensajeDeError(e, 'No se pudo registrar el abono'));
            }
            return;
        }

        // ── Pago de una o varias cuotas ──
        try {
            const r = await deudaService.pagarCuotas(estado.credito.numero, fecha, n);
            setResultadoPago(r);
            setEstado(null); setQuery(''); setSeleccionado(false);
            setCantidadCuotas(1);
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo registrar el pago'));
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 800 }}>

            <div className="form-group custom-fg mb-4" style={{ position: 'relative' }}>
                <label className="form-label" style={{ fontWeight: 600 }}>Buscar crédito (por socio, cédula o Nº de crédito)</label>
                <input
                    className="form-input"
                    value={query}
                    onChange={e => onCambioBusqueda(e.target.value)}
                    onKeyDown={onKeyDownBusqueda}
                    placeholder="Ej: María Rodríguez o CR-002"
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
                        {sugerencias.map((x, i) => (
                            <button
                                key={x.credito.id}
                                className="chip"
                                style={{
                                    display: 'flex', justifyContent: 'space-between', gap: '1rem', width: '100%', textAlign: 'left', marginBottom: 2,
                                    background: i === activo ? 'var(--primary-color)' : undefined,
                                    color: i === activo ? '#fff' : undefined,
                                }}
                                onMouseEnter={() => setActivo(i)}
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

            {resultadoPago && (
                <div className="success-message mt-4">
                    Pago registrado exitosamente — mora {fmt(resultadoPago.aplicacion.mora, moneda)}, interés{' '}
                    {fmt(resultadoPago.aplicacion.interes, moneda)}, capital {fmt(resultadoPago.aplicacion.capital, moneda)}.
                    Nuevo saldo del crédito: <strong>{fmt(resultadoPago.nuevo_saldo, moneda)}</strong>
                    {resultadoPago.estatus_credito === 'PAGADO' && ' — ¡crédito cancelado!'}
                </div>
            )}

            {resultadoAbono && (
                <div className="success-message mt-4">
                    Abono a capital de {fmt(resultadoAbono.monto_capital, moneda)} registrado.
                    Nuevo saldo del crédito: <strong>{fmt(resultadoAbono.nuevo_saldo, moneda)}</strong>
                    {resultadoAbono.estatus_credito === 'PAGADO'
                        ? ' — ¡crédito cancelado!'
                        : ` — quedan ${resultadoAbono.cuotas_restantes} cuotas.`}
                </div>
            )}

            {estado && cuotasPendientes.length === 0 && (
                <div className="success-message mt-4" style={{ background: '#f0fdf4', color: '#166534', borderLeft: '4px solid #16a34a' }}>
                    Este crédito ya está completamente pagado.
                </div>
            )}

            {/* ── Caja de desglose: siempre visible (con placeholders si no hay crédito) ── */}
            <div className="detalle-card mt-4" style={{ background: '#f1f5f9', border: 'none' }}>
                {/* Tipo de pago: saldar cuotas o abonar a capital. */}
                <div className="mb-4">
                    <div className="text-sm" style={{ color: 'var(--primary-color)', fontWeight: 600, marginBottom: '0.5rem' }}>Tipo de pago</div>
                    <div style={{ display: 'inline-flex', border: '1px solid #cbd5e1', borderRadius: '8px', overflow: 'hidden' }}>
                        {(['cuota', 'abono'] as const).map((modo, i) => (
                            <button
                                key={modo}
                                type="button"
                                className="btn"
                                onClick={() => cambiarModo(modo)}
                                style={{
                                    border: 'none', borderRadius: 0, padding: '0.4rem 1.25rem', fontWeight: 600,
                                    borderLeft: i === 1 ? '1px solid #cbd5e1' : 'none',
                                    background: modoPago === modo ? 'var(--primary-color)' : 'white',
                                    color: modoPago === modo ? '#fff' : 'inherit',
                                }}
                            >
                                {modo === 'cuota' ? 'Pago de cuota' : 'Abono a capital'}
                            </button>
                        ))}
                    </div>
                    <div className="text-sm text-muted mt-2">
                        {modoPago === 'cuota'
                            ? 'Paga una o varias cuotas: el interés del mes y el capital de las cuotas. Al pagar varias, el saldo se re-amortiza (quedan menos cuotas).'
                            : 'Abona un monto directo a capital: baja el saldo y las cuotas restantes se recalculan.'}
                    </div>
                </div>

                {modoPago === 'cuota' ? (
                    <>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem', alignItems: 'start' }}>
                            <div>
                                <div className="text-sm" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Nº de cuota</div>
                                <div style={{ fontSize: '1.1rem', fontWeight: 600, marginTop: '0.25rem' }}>
                                    {hayCredito
                                        ? `${n > 1 ? `${cuotaActual!.numero}–${cuotaActual!.numero + n - 1}` : cuotaActual!.numero} de ${totalCuotas}`
                                        : '—'}
                                </div>

                                <div className="text-sm mt-4" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Valor {n > 1 && 'total'}</div>
                                <div style={{ fontSize: '1.5rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                    {fmt(valorMostrado, moneda)}
                                </div>

                                {hayCredito && diasMora(cuotaActual!.fecha_vencimiento, fecha) > 0 && (
                                    <div className="badge-retirado mt-2" style={{ display: 'inline-block', fontSize: '0.75rem', padding: '0.25rem 0.5rem', borderRadius: '999px' }}>
                                        En mora {diasMora(cuotaActual!.fecha_vencimiento, fecha)} días
                                    </div>
                                )}
                            </div>

                            <div>
                                <div className="text-sm" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Fecha de vencimiento</div>
                                <div style={{ fontSize: '1.1rem', fontWeight: 600, marginTop: '0.25rem' }}>
                                    {fechaVencMostrada}
                                </div>

                                <div className="text-sm mt-4" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Desglose</div>
                                {hayCredito ? (
                                    <div className="text-sm text-muted mt-1" style={{ fontFamily: 'monospace' }}>
                                        Capital: {fmt(capitalN, moneda)} · Interés del mes: {fmt(interesMes, moneda)}
                                        {moraActual > 0 && ` · Mora: ${fmt(moraActual, moneda)}`}
                                    </div>
                                ) : (
                                    <div className="text-sm text-muted mt-1" style={{ fontFamily: 'monospace' }}>
                                        {cargando ? 'Buscando crédito…' : 'Busque un crédito para ver el desglose'}
                                    </div>
                                )}
                            </div>
                        </div>

                        <div className="mt-6" style={{ borderTop: '1px solid #e2e8f0', paddingTop: '1.5rem' }}>
                            <label style={{ fontWeight: 600, opacity: hayCredito ? 1 : 0.6 }}>Cuotas a pagar</label>
                            <div style={{ marginTop: '0.75rem', display: 'flex', alignItems: 'center', gap: '1rem' }}>
                                <div style={{ display: 'flex', border: '1px solid #cbd5e1', borderRadius: '6px', overflow: 'hidden', opacity: hayCredito ? 1 : 0.6 }}>
                                    <button className="btn" disabled={!hayCredito} style={{ border: 'none', background: 'white', borderRadius: 0, padding: '0.25rem 0.75rem' }}
                                        onClick={() => setCantidadCuotas(c => Math.max(1, c - 1))}>−</button>
                                    <div style={{ padding: '0.25rem 1rem', background: '#f8fafc', fontWeight: 600, minWidth: 40, textAlign: 'center', borderLeft: '1px solid #cbd5e1', borderRight: '1px solid #cbd5e1' }}>
                                        {n}
                                    </div>
                                    <button className="btn" disabled={!hayCredito} style={{ border: 'none', background: 'white', borderRadius: 0, padding: '0.25rem 0.75rem' }}
                                        onClick={() => setCantidadCuotas(c => Math.min(cuotasPendientes.length, c + 1))}>+</button>
                                </div>
                                <span className="text-sm text-muted">Máx. {cuotasPendientes.length || 0}</span>
                            </div>
                        </div>
                    </>
                ) : (
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem', alignItems: 'start' }}>
                        <div>
                            <div className="text-sm" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Saldo actual</div>
                            <div style={{ fontSize: '1.1rem', fontWeight: 600, marginTop: '0.25rem' }}>
                                {fmt(saldoActual, moneda)}
                            </div>

                            <div className="form-group custom-fg mt-4" style={{ maxWidth: 220 }}>
                                <label className="form-label">Monto del abono</label>
                                <input
                                    className="form-input"
                                    type="text"
                                    inputMode="numeric"
                                    value={montoAbonoTxt ? fmtNum(parseNum(montoAbonoTxt)) : ''}
                                    onChange={e => setMontoAbonoTxt(e.target.value.replace(/\D/g, ''))}
                                    disabled={!hayCredito}
                                    placeholder="Ej: 400.000"
                                />
                            </div>
                        </div>

                        <div>
                            <div className="text-sm" style={{ color: 'var(--primary-color)', fontWeight: 600 }}>Después del abono</div>
                            {hayCredito && montoAbono > 0 ? (
                                <>
                                    <div style={{ fontSize: '1.5rem', fontWeight: 700, marginTop: '0.25rem' }}>
                                        {fmt(nuevoSaldoAbono, moneda)}
                                    </div>
                                    <div className="text-sm text-muted mt-1">
                                        {nuevoSaldoAbono <= 0
                                            ? 'El crédito queda cancelado.'
                                            : `Quedarán ${cuotasRestantesAbono} cuotas.`}
                                    </div>
                                </>
                            ) : (
                                <div className="text-sm text-muted mt-1" style={{ fontFamily: 'monospace' }}>
                                    {hayCredito ? 'Ingrese el monto del abono' : (cargando ? 'Buscando crédito…' : 'Busque un crédito')}
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {hayCredito && estado!.socio_nombre && (
                    <div className="text-sm text-muted mt-3">Socio: <strong>{estado!.socio_nombre}</strong> · Crédito {estado!.credito.numero}</div>
                )}

                {!hayCredito && (
                    <div className="chip mt-3" style={{ display: 'inline-block' }}>Sin crédito seleccionado</div>
                )}

                <div className="form-group custom-fg mt-4" style={{ maxWidth: 200 }}>
                    <label className="form-label">Fecha de pago real</label>
                    <input className="form-input" type="date" value={fecha} onChange={e => setFecha(e.target.value)} disabled={!hayCredito} />
                </div>

                <div style={{ marginTop: '2rem', borderTop: '1px solid #e2e8f0', paddingTop: '1.5rem' }}>
                    <button
                        className="btn btn-primary"
                        onClick={() => ejecutar(pagar)}
                        disabled={enviando || !hayCredito || (modoPago === 'abono' && montoAbono <= 0)}
                        style={{ padding: '0.75rem 2rem', fontSize: '1rem' }}
                    >
                        {enviando ? 'Registrando…' : (modoPago === 'abono' ? 'Registrar abono' : 'Registrar pago')}
                    </button>
                </div>
            </div>
        </div>
    );
}
