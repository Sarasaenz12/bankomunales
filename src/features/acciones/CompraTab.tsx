import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { accionesService } from './accionesService';
import { sociosService } from '../socios/sociosService';
import { CalculoCompra, CupoMensual, AutorizacionVenta } from './types';
import { SocioResumen } from '../socios/types';
import { fmt } from '../../core/lib/format';

export function CompraTab({ moneda }: { moneda: string }) {
    const navigate = useNavigate();
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [todos, setTodos] = useState<SocioResumen[]>([]);
    const [busqueda, setBusqueda] = useState('');
    const [socio, setSocio] = useState<SocioResumen | null>(null);

    const [cupo, setCupo] = useState<CupoMensual | null>(null);
    const [fecha, setFecha] = useState(new Date().toISOString().slice(0, 10));
    const [monto, setMonto] = useState('');
    const [calculo, setCalculo] = useState<CalculoCompra | null>(null);

    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);

    useEffect(() => { recargar(); }, []);

    async function recargar() {
        try {
            setSocios(await accionesService.listarSociosActivos());
            setTodos(await sociosService.listar());
            setCupo(await accionesService.cupoDelMes(fecha));
        } catch (e: any) {
            setError(e?.toString() ?? 'Error al cargar la información');
        }
    }

    // RF-23/RF-24: el cálculo lo hace el backend, para que la pantalla nunca muestre
    // un número distinto del que después se guarda.
    useEffect(() => {
        const importe = Number(monto);
        if (!socio || !importe || importe <= 0) { setCalculo(null); return; }
        let vigente = true;
        accionesService.previsualizarCompra(socio.id, importe)
            .then(c => { if (vigente) { setCalculo(c); setError(''); } })
            .catch(e => { if (vigente) { setCalculo(null); setError(e?.toString() ?? ''); } });
        return () => { vigente = false; };
    }, [socio, monto]);

    async function registrar(e: React.FormEvent) {
        e.preventDefault();
        if (!socio) return;
        setError(''); setSuccess(''); setGuardando(true);
        try {
            const lote = await accionesService.registrarCompra(socio.id, fecha, Number(monto));
            setSuccess(`Compra registrada: ${lote.cantidad} acciones por ${fmt(lote.monto_pagado, moneda)}.`);
            setMonto(''); setCalculo(null);
            await recargar();
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo registrar la compra');
        } finally {
            setGuardando(false);
        }
    }

    const q = busqueda.trim().toLowerCase();
    const coincide = (s: SocioResumen) =>
        s.nombre_completo.toLowerCase().includes(q) || s.cedula.toLowerCase().includes(q);
    const coincidencias = q && !socio
        ? socios.filter(coincide).slice(0, 6)
        : [];
    // Existe alguien ya registrado en el sistema con esos datos (activo o no).
    const yaRegistrado = q && !socio && todos.some(coincide);

    const puedeRegistrar = !!socio && !!calculo && !calculo.supera_tope_participacion && !guardando;
    const puedeCrearNuevo = !!q && !socio && !yaRegistrado;

    return (
        <div className="detalle-card" style={{ maxWidth: 820 }}>
            {/* ── Socio ── */}
            <h3 className="section-title">SOCIO</h3>
            <div className="list-toolbar">
                <input
                    className="form-input"
                    placeholder="Buscar por cédula o nombre..."
                    value={socio ? `${socio.nombre_completo} — ${socio.cedula}` : busqueda}
                    onChange={e => { setSocio(null); setBusqueda(e.target.value); }}
                />
                <button
                    className="btn btn-secondary"
                    disabled={!puedeCrearNuevo}
                    title={puedeCrearNuevo
                        ? 'Crear un socio nuevo'
                        : 'Busque por nombre o cédula; si no hay nadie registrado con esos datos, se habilita esta opción'}
                    onClick={() => navigate('/app/socios/nuevo')}
                >
                    + Nuevo Socio
                </button>
            </div>

            {coincidencias.length > 0 && (
                <div className="detalle-card" style={{ padding: '0.5rem', marginBottom: '1rem' }}>
                    {coincidencias.map(s => (
                        <button
                            key={s.id}
                            className="chip"
                            style={{ display: 'block', width: '100%', textAlign: 'left', marginBottom: 4 }}
                            onClick={() => { setSocio(s); setBusqueda(''); }}
                        >
                            {s.nombre_completo} — {s.cedula}
                        </button>
                    ))}
                </div>
            )}
            {q && !socio && coincidencias.length === 0 && (
                <div className="text-sm text-muted mb-4">
                    {yaRegistrado
                        ? 'Ya existe un socio registrado con esos datos (puede estar retirado).'
                        : 'No hay nadie registrado con esos datos. Use "+ Nuevo Socio" para crearlo.'}
                </div>
            )}

            {/* ── PPCFC y cupo del mes (RF-26, RN-09, RN-15) ── */}
            {cupo && <PanelCupo cupo={cupo} moneda={moneda} />}

            {/* ── Compra ── */}
            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto a invertir</label>
                    <input
                        type="number" className="form-input" min="1" step="any"
                        placeholder="Ej: 100000"
                        value={monto} onChange={e => setMonto(e.target.value)}
                    />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Acciones equivalentes</label>
                    <input
                        className="form-input" readOnly
                        value={calculo ? String(calculo.cantidad) : '—'}
                        style={{ backgroundColor: '#f8fafc', color: '#64748b' }}
                    />
                    {calculo && (
                        <div className="text-sm text-muted mt-1" style={{ fontSize: 11 }}>
                            Valor nominal: {fmt(calculo.valor_nominal, moneda)}
                        </div>
                    )}
                </div>
            </div>

            {/* ── Participación del socio (RN-02) ── */}
            {socio && calculo && (
                <div style={{ marginTop: '1rem' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
                        <span className="text-sm">Participación del socio tras la compra</span>
                        <strong style={{
                            fontSize: '1.125rem',
                            color: calculo.supera_tope_participacion ? 'var(--error-color)' : 'var(--text-main)',
                        }}>
                            {calculo.participacion_pct.toFixed(2)}%
                        </strong>
                    </div>
                    <Barra
                        pct={calculo.participacion_pct}
                        alerta={calculo.supera_tope_participacion}
                    />
                    <div className="text-sm text-muted" style={{ fontSize: 11 }}>
                        Límite 15% (RN-02)
                        {calculo.tope_en_periodo_de_gracia &&
                            ' — todavía no aplica: el tope rige desde el tercer mes de operaciones'}
                    </div>
                </div>
            )}

            {calculo?.supera_tope_participacion && (
                <div className="error-message mt-4">
                    Esta compra dejaría al socio con el {calculo.participacion_pct.toFixed(2)}% de las
                    acciones. Ningún socio puede superar el 15% del total (RN-02).
                </div>
            )}
            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            <form onSubmit={registrar}>
                <button type="submit" className="btn btn-primary" style={{ marginTop: '1.5rem' }} disabled={!puedeRegistrar}>
                    {guardando ? 'Registrando…' : 'Registrar compra'}
                </button>
            </form>
        </div>
    );
}

/** RF-26: PPCFC del mes, tramo de RN-09 y cupo disponible. */
function PanelCupo({ cupo, moneda }: { cupo: CupoMensual; moneda: string }) {
    const a: AutorizacionVenta = cupo.autorizacion;

    return (
        <div className="detalle-card" style={{ marginBottom: 0 }}>
            {a.estado === 'Autoriza' && (
                <>
                    <Fila label="PPCFC actual del mes" valor={`${a.ppcfc_pct.toFixed(0)}%`} />
                    <Fila label="Rango PPCFC" valor={`${a.rango_desde}% – ${a.rango_hasta}%`} />
                    <Fila
                        label="Autorizado a vender"
                        valor={`${a.venta_pct}% del total de acciones`}
                        acento
                    />
                    <Barra pct={a.ppcfc_pct} />

                    <div style={{ marginTop: '1.5rem' }}>
                        <div className="text-sm" style={{ fontWeight: 600, marginBottom: '0.5rem' }}>
                            Cupo del mes
                        </div>
                        <Fila label="Autorizado" valor={fmt(a.cupo_monto, moneda)} />
                        <Fila label="Ya vendido" valor={fmt(cupo.vendido_monto, moneda)} />
                        <Fila
                            label="Disponible"
                            valor={fmt(cupo.disponible_monto ?? 0, moneda)}
                            acento
                        />
                        {cupo.tope_individual_monto !== null && (
                            <div className="text-sm text-muted mt-1" style={{ fontSize: 11 }}>
                                Máximo por socio este mes: {fmt(cupo.tope_individual_monto, moneda)} (RN-15)
                            </div>
                        )}
                    </div>
                </>
            )}

            {a.estado === 'NoAutoriza' && (
                <div className="error-message" style={{ margin: 0 }}>
                    El PPCFC del mes es {a.ppcfc_pct.toFixed(0)}%, por debajo del 80%: este mes no
                    se venden acciones (RN-09).
                </div>
            )}

            {a.estado === 'SinDatosSuficientes' && (
                <div className="pendiente-nota" style={{ textAlign: 'left' }}>
                    <strong>PPCFC pendiente.</strong> Se calcula promediando la colocación de los{' '}
                    <strong>3 últimos meses cerrados</strong> y por ahora hay {a.meses_cerrados}.
                    Mientras tanto el sistema no limita el cupo: la autorización de venta queda a
                    criterio de la Junta.
                </div>
            )}
        </div>
    );
}

function Fila({ label, valor, acento }: { label: string; valor: string; acento?: boolean }) {
    return (
        <div style={{
            display: 'flex', justifyContent: 'space-between',
            padding: '0.375rem 0', fontSize: '0.875rem',
        }}>
            <span style={{ fontFamily: 'monospace', color: 'var(--text-muted)' }}>{label}</span>
            <strong style={{ color: acento ? 'var(--primary-color)' : 'var(--text-main)' }}>{valor}</strong>
        </div>
    );
}

function Barra({ pct, alerta }: { pct: number; alerta?: boolean }) {
    return (
        <div style={{
            height: 6, borderRadius: 3, background: 'var(--border-color)',
            overflow: 'hidden', margin: '0.5rem 0',
        }}>
            <div style={{
                width: `${Math.min(Math.max(pct, 0), 100)}%`,
                height: '100%',
                background: alerta ? 'var(--error-color)' : '#6482a5',
            }} />
        </div>
    );
}