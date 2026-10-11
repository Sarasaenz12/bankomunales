import { useEffect, useRef, useState } from 'react';
import { liquidacionService } from './liquidacionService';
import { DatosLiquidacionSocio, DestinoLiquidacionParcial, PreviewLiquidacionParcial } from './types';
import { fmt } from '../../core/lib/format';
import { sociosService } from '../socios/sociosService';
import { SocioResumen } from '../socios/types';
import { BuscadorSocio } from '../../core/components/BuscadorSocio';
import { mensajeDeError } from '../../core/lib/errores';

type Modo = 'total' | 'parcial';

export function LiquidacionSocioPanel({ moneda }: { moneda: string }) {
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [socioId, setSocioId] = useState('');
    const [datos, setDatos] = useState<DatosLiquidacionSocio | null>(null);
    const [modo, setModo] = useState<Modo>('total');

    // Venta parcial: cuántas acciones y qué hace con el valor (efectivo o abono a deuda).
    const [capsLiquidar, setCapsLiquidar] = useState(1);
    const [destino, setDestino] = useState<DestinoLiquidacionParcial>('efectivo');
    const [preview, setPreview] = useState<PreviewLiquidacionParcial | null>(null);

    const [fecha, setFecha] = useState(() => new Date().toISOString().split('T')[0]);
    const [ejecutando, setEjecutando] = useState(false);
    const [exito, setExito] = useState('');
    const [error, setError] = useState('');
    // Candado sincrónico: evita que un doble clic rápido dispare la operación dos veces
    // antes de que el estado `ejecutando` deshabilite el botón.
    const enviando = useRef(false);

    useEffect(() => {
        sociosService.listar().then(setSocios).catch(() => { });
    }, []);

    async function elegir(id: string) {
        setSocioId(id);
        setDatos(null);
        setPreview(null);
        setError('');
        setExito('');
        setModo('total');
        setDestino('efectivo');
        if (!id) return;
        try {
            const d = await liquidacionService.prepararLiquidacionSocio(id);
            setDatos(d);
            const comprometidas = Math.ceil(d.acciones_comprometidas ?? 0);
            const libres = Math.max(0, (d.caps_activas ?? 0) - comprometidas);
            setCapsLiquidar(Math.max(1, libres));
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo cargar la liquidación del socio'));
        }
    }

    // Vista previa de la venta parcial: la calcula el backend (la ganancia depende del
    // aniversario de cada lote, RN-10), para que la pantalla nunca muestre un número
    // distinto del que se guardará.
    useEffect(() => {
        if (!datos || modo !== 'parcial' || !socioId || capsLiquidar < 1) {
            setPreview(null);
            return;
        }
        let vigente = true;
        const t = setTimeout(() => {
            liquidacionService.previsualizarLiquidacionParcial(socioId, capsLiquidar, fecha)
                .then(p => { if (vigente) { setPreview(p); setError(''); } })
                .catch(e => { if (vigente) { setPreview(null); setError(mensajeDeError(e, '')); } });
        }, 300);
        return () => { vigente = false; clearTimeout(t); };
    }, [datos, modo, socioId, capsLiquidar, fecha]);

    async function ejecutar() {
        // Doble candado (ref sincrónico + estado) contra el doble clic.
        if (!datos || enviando.current || ejecutando) return;
        enviando.current = true;
        setEjecutando(true);
        setError('');
        setExito('');
        try {
            if (modo === 'total') {
                await liquidacionService.ejecutarRetiroSocio({ socio_id: datos.socio_id, fecha });
                setExito('✓ Operación realizada: el socio fue retirado.');
            } else {
                const r = await liquidacionService.ejecutarLiquidacionParcial({
                    socio_id: datos.socio_id, cantidad: capsLiquidar, fecha, destino,
                });
                const partes = [`se liquidaron ${r.acciones_liquidadas} acciones por ${fmt(r.valor, moneda)}`];
                if (r.abonado_deuda > 0) partes.push(`abono a deuda ${fmt(r.abonado_deuda, moneda)}`);
                if (r.pagado_efectivo > 0) partes.push(`pago en efectivo ${fmt(r.pagado_efectivo, moneda)}`);
                setExito(`✓ Operación realizada: ${partes.join(' · ')}. El socio queda con ${r.acciones_restantes} acciones.`);
            }
            // El formulario se reinicia tras la operación (así no se puede repetir por error);
            // el aviso de éxito queda visible arriba.
            setSocioId('');
            setDatos(null);
            setPreview(null);
            setModo('total');
            setDestino('efectivo');
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo ejecutar la liquidación'));
        } finally {
            enviando.current = false;
            setEjecutando(false);
        }
    }

    const nominal = datos?.valor_nominal ?? 0;
    // RF-29: las libres = activas − comprometidas en garantía (se redondea hacia arriba
    // lo comprometido para nunca liberar de más).
    const comprometidas = Math.ceil(datos?.acciones_comprometidas ?? 0);
    const libres = Math.max(0, (datos?.caps_activas ?? 0) - comprometidas);

    // ── Cálculo del retiro TOTAL ──
    const gananciaTotal = Math.round(datos?.ganancia_devengada ?? 0);
    const deuda = Math.round(datos?.deuda ?? 0);
    const capitalTotal = (datos?.caps_activas ?? 0) * nominal;
    const valorTotal = capitalTotal + gananciaTotal - deuda;
    const quedaDebiendo = valorTotal < 0;

    // Si la liquidación es total (retiro), el socio sí puede usar sus acciones COMPROMETIDAS COMO TITULAR para pagar la deuda.
    // Lo que NO puede hacer es retirarse si sus acciones están comprometidas COMO FIADOR de otro crédito.
    const comprometidasFiador = datos?.acciones_comprometidas_fiador ?? 0;
    const bloqueoTotal = comprometidasFiador > 0;

    // ── Distribución del valor en la venta PARCIAL ──
    const valorParcial = preview?.valor ?? 0;
    const abono = destino === 'abono_deuda' ? Math.min(valorParcial, deuda) : 0;
    const excedente = destino === 'abono_deuda' ? Math.max(0, valorParcial - abono) : valorParcial;
    const nuevaDeuda = Math.max(0, deuda - abono);

    return (
        <div className="detalle-card" style={{ maxWidth: 760 }}>
            <div className="text-sm text-muted" style={{ marginBottom: '1rem' }}>
                <strong>Liquidación de un socio.</strong> Puede ser <strong>total</strong> (retiro
                completo: recibe el capital de todas sus acciones + la ganancia que devengó, menos su
                deuda) o <strong>parcial</strong> (vende algunas acciones libres y sigue siendo socio).
            </div>

            <div className="form-group custom-fg" style={{ maxWidth: 360 }}>
                <label className="form-label">Socio</label>
                <BuscadorSocio
                    placeholder="Buscar por cédula o nombre..."
                    socios={socios}
                    valor={socioId ? socios.find(s => s.id === socioId)?.nombre_completo + ' — ' + socios.find(s => s.id === socioId)?.cedula : ''}
                    onSelect={s => elegir(s.id)}
                />
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {exito && <div className="success-message mt-4">{exito}</div>}

            {/* ── Tipo de liquidación (arriba) ── */}
            <div className="chip-row mt-4" style={{ gap: '1.5rem' }}>
                <label style={{ display: 'inline-flex', gap: '0.4rem', alignItems: 'center', cursor: 'pointer' }}>
                    <input type="radio" checked={modo === 'total'} onChange={() => setModo('total')} />
                    Total (retiro completo)
                </label>
                <label style={{ display: 'inline-flex', gap: '0.4rem', alignItems: 'center', cursor: 'pointer' }}>
                    <input type="radio" checked={modo === 'parcial'} onChange={() => setModo('parcial')} />
                    Parcial (algunas acciones)
                </label>
            </div>

            {/* ── Valores del socio, en recuadro gris (se llenan al elegir el socio) ── */}
            <div className="mt-4" style={{
                background: '#e9eef4', border: '1px solid #d6deea',
                borderRadius: 12, padding: '1.25rem 1.5rem',
            }}>
                <div className="grid-dos">
                    <div>Acciones activas: <strong>{datos?.caps_activas ?? 0}</strong></div>
                    <div>Comprometidas en garantía: <strong>{comprometidas}</strong></div>
                    <div>Acciones libres: <strong>{libres}</strong></div>
                    <div>Ganancia devengada: <strong>{fmt(datos?.ganancia_devengada ?? 0, moneda)}</strong></div>
                    <div>Deuda vigente: <strong>{fmt(datos?.deuda ?? 0, moneda)}</strong></div>
                    <div>Valor nominal: <strong>{fmt(datos?.valor_nominal ?? 0, moneda)}</strong></div>
                </div>
            </div>

            {datos?.caps_activas === 0 && (
                <div className="text-sm text-muted mt-4">
                    Este socio no tiene acciones activas.
                </div>
            )}

            {/* ═══════════════ RETIRO TOTAL ═══════════════ */}
            {datos && modo === 'total' && (
                <>
                    <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                        <div className="form-group custom-fg">
                            <label className="form-label">Ganancia</label>
                            <input className="form-input" readOnly
                                title="Sólo la ganancia de acciones que cumplieron el año. El resto queda como ganancia colectiva."
                                value={fmt(gananciaTotal, moneda)} />
                        </div>
                        <div className="form-group custom-fg">
                            <label className="form-label">Deuda (descuento)</label>
                            <input className="form-input" readOnly
                                title="La toma el sistema del saldo vigente de los créditos del socio."
                                value={fmt(deuda, moneda)} />
                        </div>
                        <div className="form-group custom-fg">
                            <label className="form-label">Fecha del Retiro</label>
                            <input className="form-input" type="date" value={fecha}
                                onChange={e => setFecha(e.target.value)} />
                        </div>
                    </div>

                    {bloqueoTotal && (
                        <div className="error-message mt-4">
                            No se puede retirar por completo: {comprometidasFiador} de sus acciones respaldan
                            un crédito vigente como FIADOR. El crédito debe ser saldado o debe reemplazarse la garantía antes.
                        </div>
                    )}

                    {!bloqueoTotal && (
                        <div className="detalle-card mt-4">
                            <div style={{ fontWeight: 600 }}>Acta de retiro — {datos.nombre}</div>
                            {datos.cedula && <div className="text-sm text-muted">C.C. {datos.cedula}</div>}
                            <div className="grid-dos mt-4">
                                <div>Acciones a liquidar: <strong>{datos.caps_activas}</strong></div>
                                <div>Capital ({datos.caps_activas} × {fmt(nominal, moneda)}): <strong>{fmt(capitalTotal, moneda)}</strong></div>
                                <div>+ Ganancia: <strong>{fmt(gananciaTotal, moneda)}</strong></div>
                                <div>− Deuda: <strong>{fmt(deuda, moneda)}</strong></div>
                                <div className="total-line">
                                    Valor a pagar: <strong>{fmt(valorTotal, moneda)}</strong>
                                    {quedaDebiendo && <span className="text-sm text-muted"> — el socio queda debiendo</span>}
                                </div>
                                <div>
                                    Estado:{' '}
                                    <span className={`badge-estatus ${quedaDebiendo ? 'badge-retirado' : 'badge-activo'}`}>
                                        {quedaDebiendo ? 'Queda debiendo' : 'Le pagan'}
                                    </span>
                                </div>
                            </div>
                            {!quedaDebiendo && (
                                <div className="text-sm text-muted mt-4">
                                    Paz y salvo. Le quedan 0 acciones. Firma: _______________________
                                </div>
                            )}
                            <button className="btn btn-primary mt-4" onClick={ejecutar} disabled={ejecutando}>
                                {ejecutando ? 'Ejecutando...' : (quedaDebiendo ? 'Registrar Retiro con Deuda' : 'Ejecutar Retiro de Socio')}
                            </button>
                        </div>
                    )}
                </>
            )}

            {/* ═══════════════ VENTA PARCIAL ═══════════════ */}
            {datos && modo === 'parcial' && (
                <>
                    <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                        <div className="form-group custom-fg">
                            <label className="form-label">Acciones a liquidar (máx. {libres} libres)</label>
                            <input className="form-input" type="text" inputMode="numeric"
                                placeholder={`Máx. ${libres}`}
                                value={capsLiquidar ? String(capsLiquidar) : ''}
                                onChange={e => setCapsLiquidar(Number(e.target.value.replace(/\D/g, '')) || 0)}
                                disabled={!datos} />
                        </div>
                        <div className="form-group custom-fg">
                            <label className="form-label">Fecha</label>
                            <input className="form-input" type="date" value={fecha}
                                onChange={e => setFecha(e.target.value)} />
                        </div>
                    </div>

                    {/* ¿Qué hace el socio con el valor de sus acciones? */}
                    <div className="form-group custom-fg mt-2">
                        <label className="form-label">¿Qué hace con el valor de las acciones?</label>
                        <div className="chip-row" style={{ gap: '1.5rem' }}>
                            <label style={{ display: 'inline-flex', gap: '0.4rem', alignItems: 'center', cursor: 'pointer' }}>
                                <input type="radio" checked={destino === 'efectivo'} onChange={() => setDestino('efectivo')} />
                                Recibirlo en efectivo
                            </label>
                            <label style={{ display: 'inline-flex', gap: '0.4rem', alignItems: 'center', cursor: 'pointer',
                                opacity: deuda > 0 ? 1 : 0.5 }}>
                                <input type="radio" checked={destino === 'abono_deuda'} disabled={deuda <= 0}
                                    onChange={() => setDestino('abono_deuda')} />
                                Abonarlo a su deuda{deuda <= 0 ? ' (sin deuda)' : ''}
                            </label>
                        </div>
                    </div>

                    {preview?.bloqueo && <div className="error-message mt-4">{preview.bloqueo}</div>}

                    {preview && !preview.bloqueo && (
                        <div className="detalle-card mt-4">
                            <div style={{ fontWeight: 600 }}>Acta de venta parcial — {datos.nombre}</div>
                            {datos.cedula && <div className="text-sm text-muted">C.C. {datos.cedula}</div>}
                            <div className="grid-dos mt-4">
                                <div>Acciones a liquidar: <strong>{preview.cantidad}</strong></div>
                                <div>Capital ({preview.cantidad} × {fmt(nominal, moneda)}): <strong>{fmt(preview.capital, moneda)}</strong></div>
                                <div>+ Ganancia (acciones que cumplieron el año): <strong>{fmt(preview.ganancia, moneda)}</strong></div>
                                <div className="total-line">
                                    Valor de las acciones: <strong>{fmt(preview.valor, moneda)}</strong>
                                </div>
                            </div>

                            <div className="mt-4" style={{ borderTop: '1px solid var(--border-color)', paddingTop: '0.75rem' }}>
                                {destino === 'efectivo' ? (
                                    <div>Se le paga en efectivo: <strong>{fmt(preview.valor, moneda)}</strong></div>
                                ) : (
                                    <div className="grid-dos">
                                        <div>Abono a la deuda: <strong>{fmt(abono, moneda)}</strong></div>
                                        <div>Nueva deuda: <strong>{fmt(nuevaDeuda, moneda)}</strong></div>
                                        {excedente > 0 && (
                                            <div>Excedente en efectivo: <strong>{fmt(excedente, moneda)}</strong></div>
                                        )}
                                    </div>
                                )}
                            </div>

                            <div className="text-sm text-muted mt-4">
                                El socio sigue activo: le quedarán {libres - preview.cantidad} acciones libres
                                {comprometidas > 0 ? ` (más ${comprometidas} comprometidas en garantía)` : ''}.
                            </div>

                            <button className="btn btn-primary mt-4" onClick={ejecutar} disabled={ejecutando}>
                                {ejecutando ? 'Ejecutando...' : 'Ejecutar venta parcial'}
                            </button>
                        </div>
                    )}
                </>
            )}

            <div className="text-sm text-muted mt-4">
                Las acciones comprometidas en garantía de un crédito vigente no se
                pueden liquidar; sólo las libres. Antes del año, la ganancia de las acciones
                que se venden se reparte al colectivo; después del año, se le paga al socio.
            </div>
        </div>
    );
}
