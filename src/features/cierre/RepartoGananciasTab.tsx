import { useEffect, useState } from 'react';
import { cierreService } from './cierreService';
import { DesgloseRepartoMes, FilaPagoGanancia, FilaRepartoAnual } from './types';
import { fmt, fmtNum } from '../../core/lib/format';
import { mesActual, fechaHoy } from './util';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';
import { mensajeDeError } from '../../core/lib/errores';

const MESES_ES = [
    'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
    'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre',
];

/** "2026-01" o "2026-01-01" → "Enero 2026". */
function nombreMes(iso: string): string {
    const y = Number(iso.slice(0, 4));
    const m = Number(iso.slice(5, 7));
    return `${MESES_ES[m - 1] ?? iso.slice(5, 7)} ${y}`;
}

/** "2026-07-31T..." → "31/07/2026". */
function fechaCorta(iso: string): string {
    const d = iso.slice(0, 10);
    const [y, m, dd] = d.split('-');
    return dd && m && y ? `${dd}/${m}/${y}` : d;
}

/** Suma n meses a un "YYYY-MM-01". */
function sumarMesISO(iso: string, n: number): string {
    const y = Number(iso.slice(0, 4));
    const m = Number(iso.slice(5, 7)) - 1 + n;
    const ay = y + Math.floor(m / 12);
    const am = ((m % 12) + 12) % 12 + 1;
    return `${ay}-${String(am).padStart(2, '0')}-01`;
}

const BADGE: Record<string, { txt: string; bg: string; color: string }> = {
    PAGADA: { txt: 'Pagada', bg: '#e6f4ea', color: '#1e7e34' },
    PENDIENTE: { txt: 'Pendiente', bg: '#e7edf7', color: '#2f5c9e' },
    PROXIMO: { txt: 'Próximo mes', bg: '#eef1f4', color: '#6b7684' },
};

/** Una fila etiqueta → valor del desglose. */
function Linea({ label, valor, fuerte, acento }: {
    label: string; valor: string; fuerte?: boolean; acento?: boolean;
}) {
    return (
        <div style={{
            display: 'flex', justifyContent: 'space-between', gap: '1rem',
            padding: '0.4rem 0', fontSize: fuerte || acento ? '0.95rem' : '0.875rem',
        }}>
            <span style={{
                fontFamily: 'monospace',
                color: fuerte || acento ? 'var(--text-main)' : 'var(--text-muted)',
                fontWeight: fuerte ? 600 : 400,
            }}>{label}</span>
            <strong style={{
                whiteSpace: 'nowrap',
                color: acento ? 'var(--primary-color)' : 'var(--text-main)',
                fontWeight: fuerte || acento ? 700 : 600,
            }}>{valor}</strong>
        </div>
    );
}

const DIVISOR = <div style={{ borderTop: '1px solid var(--border-color)', margin: '0.4rem 0' }} />;

export function RepartoGananciasTab({ moneda }: { moneda: string }) {
    const [mes, setMes] = useState(mesActual());
    const [desglose, setDesglose] = useState<DesgloseRepartoMes | null>(null);
    const [desgloseErr, setDesgloseErr] = useState('');

    // La fecha de pago es hoy (los pagos se registran al día); ya no se pide en pantalla.
    const fechaPago = fechaHoy();
    const [filas, setFilas] = useState<FilaPagoGanancia[]>([]);
    const [busqueda, setBusqueda] = useState('');
    const [error, setError] = useState('');
    const [msg, setMsg] = useState('');

    const [modal, setModal] = useState<FilaPagoGanancia | null>(null);
    const [cohorteMeses, setCohorteMeses] = useState<Record<string, FilaRepartoAnual[]>>({});
    const { enviando, ejecutar } = useEnvioUnico();

    async function cargarDesglose(m: string) {
        setDesgloseErr(''); setDesglose(null);
        try {
            setDesglose(await cierreService.desgloseRepartoMes(m + '-01'));
        } catch (e) {
            setDesgloseErr(mensajeDeError(e, 'No se pudo calcular el desglose del mes'));
        }
    }

    async function cargarResumen(fecha: string) {
        setError('');
        try {
            setFilas(await cierreService.resumenPagosGanancias(fecha));
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo cargar el resumen de pagos'));
        }
    }

    useEffect(() => { cargarDesglose(mes); }, []); // eslint-disable-line
    useEffect(() => { cargarResumen(fechaPago); }, []); // eslint-disable-line

    async function abrirHistorial(fila: FilaPagoGanancia) {
        setModal(fila);
        const mapa: Record<string, FilaRepartoAnual[]> = {};
        await Promise.all(fila.cohortes.map(async c => {
            try {
                const anual = await cierreService.repartoAnual(c.mes_compra);
                mapa[c.mes_compra] = anual.meses;
            } catch {
                mapa[c.mes_compra] = [];
            }
        }));
        setCohorteMeses(mapa);
    }

    async function liquidarSocio(socioId: string) {
        setMsg(''); setError('');
        try {
            const r = await cierreService.pagarGananciasDeSocio(socioId, fechaPago);
            const total = r.reduce((a, p) => a + p.monto, 0);
            setMsg(r.length ? `Liquidada la ganancia del socio (${fmt(total, moneda)}).` : 'No había ganancias cumplidas por pagar.');
            setModal(null);
            await cargarResumen(fechaPago);
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo liquidar'));
        }
    }

    async function registrarTodos() {
        setMsg(''); setError('');
        try {
            const r = await cierreService.pagarGananciasCumplidas(fechaPago);
            setMsg(r.length ? `Se registraron los pagos de ${r.length} socio(s).` : 'No hay ganancias cumplidas por pagar.');
            await cargarResumen(fechaPago);
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudieron registrar los pagos'));
        }
    }

    const q = busqueda.trim().toLowerCase();
    const visibles = q ? filas.filter(f => f.nombre.toLowerCase().includes(q)) : filas;
    const totalAPagar = filas.reduce((a, f) => a + f.ganancia_a_pagar, 0);
    const hayPendientes = filas.some(f => f.estado === 'PENDIENTE');
    const neg = (v: number) => (v > 0 ? '−' : '') + fmt(v, moneda);

    return (
        <div style={{ display: 'grid', gap: '1.5rem', maxWidth: 1040 }}>
            {/* ── Desglose de ingresos repartibles ── */}
            <div className="detalle-card">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '1rem', flexWrap: 'wrap' }}>
                    <div className="section-title" style={{ margin: 0, letterSpacing: '0.03em' }}>
                        DESGLOSE DE INGRESOS REPARTIBLES
                    </div>
                    <input
                        className="form-input" type="month" value={mes}
                        style={{ maxWidth: 180 }}
                        onChange={e => { setMes(e.target.value); cargarDesglose(e.target.value); }}
                    />
                </div>

                {desgloseErr && <div className="text-sm text-muted mt-4">{desgloseErr}</div>}

                {desglose && (
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '1rem 2.5rem', marginTop: '1.25rem' }}>
                        {/* Columna izquierda: los ingresos */}
                        <div>
                            <Linea label="Intereses Ordinarios" valor={fmt(desglose.intereses_ordinarios, moneda)} />
                            <Linea label="Intereses de Mora" valor={fmt(desglose.intereses_mora, moneda)} />
                            <Linea label="Otros Ingresos" valor={fmt(desglose.otros_ingresos, moneda)} />
                            {DIVISOR}
                            <Linea label="Total Ingresos Repartibles" valor={fmt(desglose.total_ingresos_repartibles, moneda)} fuerte />
                        </div>

                        {/* Columna derecha: egresos, fondos y rendimiento */}
                        <div>
                            <Linea label="Otros Egresos e Incobrables" valor={neg(desglose.otros_egresos)} />
                            {DIVISOR}
                            <Linea label="Balance Gestión Mensual Bruto" valor={fmt(desglose.balance_bruto, moneda)} fuerte />
                            <Linea label={`Fondo de Incobrables (${Math.round(desglose.pct_fondo_incobrables)}%)`} valor={neg(desglose.fondo_incobrables)} />
                            <Linea label={`Fondo de Gastos (${Math.round(desglose.pct_fondo_gastos)}%)`} valor={neg(desglose.fondo_gastos)} />
                            {DIVISOR}
                            <Linea label="Balance Gestión Mensual Neto" valor={fmt(desglose.balance_neto, moneda)} fuerte />
                            <Linea label="(÷) Nº de Acciones Activas" valor={fmtNum(desglose.acciones_activas)} />
                            {DIVISOR}
                            <Linea label={`Rendimiento de este mes (${MESES_ES[Number(mes.slice(5, 7)) - 1] ?? ''})`} valor={fmt(desglose.rendimiento_mes, moneda)} acento />
                            <Linea label="Rendimiento acumulado 12 meses" valor={fmt(desglose.rendimiento_acumulado_12m, moneda)} fuerte />
                        </div>
                    </div>
                )}
            </div>

            {/* ── Reparto de ganancias del mes ── */}
            <div className="detalle-card">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', gap: '1rem', flexWrap: 'wrap' }}>
                    <h3 className="section-title" style={{ margin: 0 }}>Reparto de Ganancias — {nombreMes(mes)}</h3>
                    <span className="text-sm text-muted">
                        {desglose?.cerrado_el ? `Cerrado el ${fechaCorta(desglose.cerrado_el)}` : 'Mes abierto'}
                    </span>
                </div>

                <div className="chip-row mt-4" style={{ alignItems: 'flex-end' }}>
                    <div className="form-group custom-fg" style={{ maxWidth: 340, flex: 1 }}>
                        <label className="form-label">Buscar socio</label>
                        <input className="form-input" placeholder="Escribe el nombre del socio…"
                            value={busqueda} onChange={e => setBusqueda(e.target.value)} />
                    </div>
                </div>

                {error && <div className="error-message mt-4">{error}</div>}
                {msg && <div className="success-message mt-4">{msg}</div>}

                <table className="activity-table mt-4">
                    <thead>
                        <tr>
                            <th>Socio</th>
                            <th style={{ textAlign: 'right' }}>Acciones que cumplen año</th>
                            <th style={{ textAlign: 'right' }}>Valor ganancia/acción</th>
                            <th style={{ textAlign: 'right' }}>Ganancia a pagar</th>
                            <th style={{ textAlign: 'center' }}>Estado</th>
                            <th />
                        </tr>
                    </thead>
                    <tbody>
                        {visibles.map(f => {
                            const b = BADGE[f.estado] ?? BADGE.PROXIMO;
                            const cumple = f.acciones_cumplen > 0;
                            return (
                                <tr key={f.socio_id}>
                                    <td style={{ fontWeight: 500 }}>{f.nombre}</td>
                                    <td style={{ textAlign: 'right' }}>{cumple ? f.acciones_cumplen : '—'}</td>
                                    <td style={{ textAlign: 'right' }}>{cumple ? fmt(f.valor_ganancia_accion, moneda) : '—'}</td>
                                    <td style={{ textAlign: 'right' }}>{cumple ? fmt(f.ganancia_a_pagar, moneda) : '—'}</td>
                                    <td style={{ textAlign: 'center' }}>
                                        <span style={{
                                            background: b.bg, color: b.color, borderRadius: 999,
                                            padding: '2px 10px', fontSize: 12, fontWeight: 600, whiteSpace: 'nowrap',
                                        }}>{b.txt}</span>
                                    </td>
                                    <td style={{ textAlign: 'right' }}>
                                        <button className="btn btn-secondary" onClick={() => abrirHistorial(f)}>Historial</button>
                                    </td>
                                </tr>
                            );
                        })}
                        {visibles.length === 0 && (
                            <tr><td colSpan={6} className="text-sm text-muted" style={{ padding: '1rem' }}>
                                {filas.length === 0 ? 'Sin socios para mostrar.' : 'Ningún socio coincide con la búsqueda.'}
                            </td></tr>
                        )}
                    </tbody>
                </table>

                {filas.length > 0 && (
                    <div style={{ textAlign: 'right', marginTop: '0.75rem', fontSize: '1.05rem' }}>
                        <strong>Total: {fmt(totalAPagar, moneda)}</strong>
                    </div>
                )}

                <button className="btn btn-primary mt-4" onClick={() => ejecutar(registrarTodos)} disabled={!hayPendientes || enviando}>
                    {enviando ? 'Registrando…' : 'Registrar pagos de ganancias'}
                </button>
                {!hayPendientes && (
                    <div className="text-sm text-muted mt-4">
                        Se activa cuando hay acciones que cumplen el año y están pendientes de pago.
                    </div>
                )}
            </div>

            {/* ── Modal de historial ── */}
            {modal && (
                <div style={{
                    position: 'fixed', inset: 0, background: 'rgba(0,0,0,0.45)', display: 'flex',
                    alignItems: 'flex-start', justifyContent: 'center', padding: '3rem 1rem', zIndex: 50, overflowY: 'auto',
                }} onClick={() => setModal(null)}>
                    <div className="detalle-card" style={{ maxWidth: 720, width: '100%' }} onClick={e => e.stopPropagation()}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <h3 className="section-title" style={{ margin: 0 }}>Historial de Ganancias — {modal.nombre}</h3>
                            <button
                                onClick={() => setModal(null)}
                                aria-label="Cerrar"
                                style={{ background: 'none', border: 'none', fontSize: '1.5rem', lineHeight: 1, cursor: 'pointer', color: 'var(--text-muted)' }}
                            >×</button>
                        </div>

                        {modal.cohortes.map(c => {
                            const meses = cohorteMeses[c.mes_compra] ?? [];
                            const porMes = new Map(meses.map(m => [m.mes, m.valor_por_accion]));
                            const ventana = Array.from({ length: 12 }, (_, i) => sumarMesISO(c.mes_compra, i));
                            const faltan = Math.max(0, 12 - c.meses_transcurridos);
                            const total = ventana.reduce((a, mIso) => a + (porMes.get(mIso) ?? 0) * c.cantidad, 0);
                            return (
                                <div className="detalle-card mt-4" key={c.mes_compra} style={{ background: 'var(--surface-alt)' }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                                        <div><strong>{nombreMes(c.mes_compra)} · {c.cantidad} acciones</strong></div>
                                        <div className="text-sm text-muted">{c.meses_transcurridos} meses</div>
                                    </div>
                                    <div className="text-sm text-muted">
                                        {c.cumple_anio ? 'Cumplió el año — se puede liquidar' : `Faltan ${faltan} meses para liquidar`}
                                    </div>
                                    <table className="activity-table mt-4">
                                        <thead>
                                            <tr>
                                                <th>Mes</th>
                                                <th style={{ textAlign: 'right' }}>Rendimiento/acción</th>
                                                <th style={{ textAlign: 'right' }}>Ganancia</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {ventana.map(mIso => {
                                                const rend = porMes.get(mIso) ?? 0;
                                                return (
                                                    <tr key={mIso}>
                                                        <td>{nombreMes(mIso)}</td>
                                                        <td style={{ textAlign: 'right' }}>{fmt(rend, moneda)}</td>
                                                        <td style={{ textAlign: 'right' }}>{fmt(rend * c.cantidad, moneda)}</td>
                                                    </tr>
                                                );
                                            })}
                                        </tbody>
                                        <tfoot>
                                            <tr>
                                                <td colSpan={2} style={{ textAlign: 'right' }}><strong>Total acumulado:</strong></td>
                                                <td style={{ textAlign: 'right' }}><strong>{fmt(total, moneda)}</strong></td>
                                            </tr>
                                        </tfoot>
                                    </table>
                                </div>
                            );
                        })}

                        <button
                            className="btn btn-primary mt-4"
                            disabled={!modal.cohortes.some(c => c.cumple_anio) || modal.estado === 'PAGADA' || enviando}
                            onClick={() => ejecutar(() => liquidarSocio(modal.socio_id))}
                        >
                            {enviando ? 'Liquidando…' : 'Liquidar estas ganancias'}
                        </button>
                        {!modal.cohortes.some(c => c.cumple_anio) && (
                            <div className="text-sm text-muted mt-4">
                                Todavía ninguna de sus compras cumple el año.
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
