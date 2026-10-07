import { useEffect, useState } from 'react';
import { liquidacionService } from './liquidacionService';
import { DatosLiquidacion, LiquidacionFinal, ParametrosLiquidacion, SocioLiquidacion } from './types';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';

// El dinero del Bankomunal va en pesos enteros (COP, sin centavos): el campo muestra el
// valor con separador de miles y sólo admite dígitos.
function CampoNumero({ label, valor, onChange, sufijo, inline }: {
    label?: string; valor: number; onChange: (n: number) => void; sufijo?: string; inline?: boolean
}) {
    return (
        <div className={inline ? "" : "form-group custom-fg"}>
            {label && <label className="form-label">{label}{sufijo ? ` (${sufijo})` : ''}</label>}
            <input
                className="form-input"
                type="text"
                inputMode="numeric"
                style={inline ? { maxWidth: '100%', textAlign: 'right' } : {}}
                value={fmtNum(valor)}
                onChange={e => onChange(parseNum(e.target.value))}
            />
        </div>
    );
}

export function LiquidacionFinalPanel({ moneda }: { moneda: string }) {
    const [datos, setDatos] = useState<DatosLiquidacion | null>(null);
    const [resultado, setResultado] = useState<LiquidacionFinal | null>(null);
    const [error, setError] = useState('');
    const [mostrarActas, setMostrarActas] = useState(false);

    useEffect(() => {
        liquidacionService.prepararLiquidacionFinal()
            .then(d => { setDatos(d); setResultado(null); })
            .catch(e => setError(e?.toString() ?? 'No se pudieron cargar los datos'));
    }, []);

    function setParam<K extends keyof ParametrosLiquidacion>(campo: K, valor: number) {
        if (!datos) return;
        setDatos({ ...datos, parametros: { ...datos.parametros, [campo]: valor } });
        setResultado(null);
    }

    function setSocio(i: number, campo: keyof SocioLiquidacion, valor: number) {
        if (!datos) return;
        const socios = datos.socios.map((s, idx) => idx === i ? { ...s, [campo]: valor } : s);
        setDatos({ ...datos, socios });
        setResultado(null);
    }

    async function calcular() {
        if (!datos) return;
        setError('');
        try {
            setResultado(await liquidacionService.calcularLiquidacionFinal(datos.parametros, datos.socios));
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo calcular la liquidación');
        }
    }

    if (error && !datos) return <div className="error-message">{error}</div>;
    if (!datos) return <div className="text-sm text-muted">Cargando datos de la liquidación…</div>;

    const p = datos.parametros;

    return (
        <div className="detalle-card" style={{ maxWidth: 1150 }}>
            <div className="text-sm text-muted" style={{ marginBottom: '1rem' }}>
                <strong>Liquidación Final del Bankomunal</strong> — disolución: se reparte todo el
                patrimonio entre los socios y se cierra el Banko. Los valores llegan prellenados con
                lo que el sistema conoce (caja, créditos y acciones); la Asamblea ajusta los
                incobrables y los fondos de reserva al liquidar. Es sólo un cálculo: no mueve dinero.
            </div>

            {/* ── Insumos del Banko ── */}
            <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>Insumos del Banko</div>
            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <CampoNumero label="Créditos por cobrar" valor={p.creditos_por_cobrar} onChange={v => setParam('creditos_por_cobrar', v)} />
                <CampoNumero label="Caja disponible" valor={p.caja} onChange={v => setParam('caja', v)} />
                <CampoNumero label="Incobrables (créditos vencidos)" valor={p.incobrables} onChange={v => setParam('incobrables', v)} />
                <CampoNumero label="% Fondo de Gastos" valor={p.pct_fondo_gastos} onChange={v => setParam('pct_fondo_gastos', v)} sufijo="%" />
                <CampoNumero label="% Fondo de Incobrables" valor={p.pct_fondo_incobrables} onChange={v => setParam('pct_fondo_incobrables', v)} sufijo="%" />
                <CampoNumero label="Valor nominal de la acción" valor={p.valor_nominal} onChange={v => setParam('valor_nominal', v)} />
            </div>

            {/* ── Socios (editable) ── */}
            <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>
                Socios — acciones y deuda (editable)
            </div>
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Socio</th>
                            <th>Cédula</th>
                            <th style={{ textAlign: 'right' }}>Caps activas</th>
                            <th style={{ textAlign: 'right' }}>Caps con ganancia</th>
                            <th style={{ textAlign: 'right' }}>Crédito por pagar</th>
                        </tr>
                    </thead>
                    <tbody>
                        {datos.socios.map((s, i) => (
                            <tr key={s.socio_id}>
                                <td>{s.nombre}</td>
                                <td>{s.cedula}</td>
                                <td style={{ textAlign: 'right' }}>
                                    <div style={{ maxWidth: 100, marginLeft: 'auto' }}>
                                        <CampoNumero inline valor={s.caps_activas} onChange={v => setSocio(i, 'caps_activas', v)} />
                                    </div>
                                </td>
                                <td style={{ textAlign: 'right' }}>
                                    <div style={{ maxWidth: 100, marginLeft: 'auto' }}>
                                        <CampoNumero inline valor={s.caps_ganancia} onChange={v => setSocio(i, 'caps_ganancia', v)} />
                                    </div>
                                </td>
                                <td style={{ textAlign: 'right' }}>
                                    <div style={{ maxWidth: 140, marginLeft: 'auto' }}>
                                        <CampoNumero inline valor={s.credito} onChange={v => setSocio(i, 'credito', v)} />
                                    </div>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>

            <button className="btn btn-primary mt-4" onClick={calcular}>Calcular liquidación</button>
            {error && <div className="error-message mt-4">{error}</div>}

            {resultado && (
                <>
                    {/* ── Totales del Banko ── */}
                    <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>Resultado del Banko</div>
                    <div className="grid-dos">
                        <div>Fondo de crédito (créditos + caja): <strong>{fmt(resultado.fondo_credito, moneda)}</strong></div>
                        <div>− Incobrables: <strong>{fmt(resultado.incobrables, moneda)}</strong></div>
                        <div>Fondo sano: <strong>{fmt(resultado.fondo_sano, moneda)}</strong></div>
                        <div>− Reservas (gastos + incobrables): <strong>{fmt(resultado.reserva_gastos + resultado.reserva_incobrables, moneda)}</strong></div>
                        <div>Capital real del BK: <strong>{fmt(resultado.capital_real, moneda)}</strong></div>
                        <div>− Monto de acciones ({resultado.acciones_activas} × nominal): <strong>{fmt(resultado.monto_acciones, moneda)}</strong></div>
                        <div className="total-line">Ganancia general del BK: <strong>{fmt(resultado.ganancia_general, moneda)}</strong></div>
                        <div className="total-line">Ganancia por acción: <strong>{fmt(resultado.valor_ganancia_accion, moneda)}</strong> ({resultado.total_caps_ganancia} caps)</div>
                    </div>

                    {/* ── Detalle por socio ── */}
                    <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>Liquidación por socio</div>
                    <div style={{ overflowX: 'auto' }}>
                        <table className="activity-table">
                            <thead>
                                <tr>
                                    <th>Socio</th>
                                    <th style={{ textAlign: 'right' }}>Valor acciones</th>
                                    <th style={{ textAlign: 'right' }}>Ganancia</th>
                                    <th style={{ textAlign: 'right' }}>− Crédito</th>
                                    <th style={{ textAlign: 'right' }}>Valor a pagar</th>
                                    <th style={{ textAlign: 'center' }}>Estado</th>
                                </tr>
                            </thead>
                            <tbody>
                                {resultado.socios.map(s => (
                                    <tr key={s.socio_id}>
                                        <td>{s.nombre}</td>
                                        <td style={{ textAlign: 'right' }}>{fmt(s.valor_caps, moneda)}</td>
                                        <td style={{ textAlign: 'right' }}>{fmt(s.ganancia, moneda)}</td>
                                        <td style={{ textAlign: 'right' }}>{fmt(s.credito, moneda)}</td>
                                        <td style={{ textAlign: 'right' }}><strong>{fmt(s.valor_a_pagar, moneda)}</strong></td>
                                        <td style={{ textAlign: 'center' }}>
                                            <span className={`badge-estatus ${s.queda_debiendo ? 'badge-retirado' : 'badge-activo'}`}>
                                                {s.queda_debiendo ? 'Queda debiendo' : 'Le pagan'}
                                            </span>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                            <tfoot>
                                <tr>
                                    <td><strong>Total a pagar</strong></td>
                                    <td colSpan={3} />
                                    <td style={{ textAlign: 'right' }}><strong>{fmt(resultado.total_a_pagar, moneda)}</strong></td>
                                    <td />
                                </tr>
                            </tfoot>
                        </table>
                    </div>

                    {/* ── Actas por socio ── */}
                    <label className="text-sm mt-4" style={{ display: 'inline-flex', gap: '0.5rem', alignItems: 'center' }}>
                        <input type="checkbox" checked={mostrarActas} onChange={e => setMostrarActas(e.target.checked)} />
                        Mostrar acta de liquidación por socio
                    </label>

                    {mostrarActas && (
                        <div className="mt-4" style={{ display: 'grid', gap: '1rem' }}>
                            {resultado.socios.map(s => (
                                <div className="detalle-card" key={s.socio_id}>
                                    <div style={{ fontWeight: 600 }}>Acta de liquidación — {s.nombre}</div>
                                    {s.cedula && <div className="text-sm text-muted">C.C. {s.cedula}</div>}
                                    <div className="grid-dos mt-4">
                                        <div>Acciones activas: <strong>{s.caps_activas}</strong></div>
                                        <div>Valor de acciones: <strong>{fmt(s.valor_caps, moneda)}</strong></div>
                                        <div>Acciones con ganancia: <strong>{s.caps_ganancia}</strong></div>
                                        <div>Ganancia: <strong>{fmt(s.ganancia, moneda)}</strong></div>
                                        <div>Crédito (descuento): <strong>{fmt(s.credito, moneda)}</strong></div>
                                        <div className="total-line">
                                            Valor a pagar: <strong>{fmt(s.valor_a_pagar, moneda)}</strong>
                                            {s.queda_debiendo && <span className="text-sm text-muted"> — el socio queda debiendo</span>}
                                        </div>
                                    </div>
                                    {!s.queda_debiendo && (
                                        <div className="text-sm text-muted mt-4">
                                            Paz y salvo. Le quedan 0 acciones. Firma: _______________________
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                    )}
                </>
            )}
        </div>
    );
}
