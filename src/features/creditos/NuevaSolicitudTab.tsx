import { useEffect, useMemo, useState } from 'react';
import { creditosService, validarNuevaSolicitud, cumpleRn03, cumpleRn04Socio, cumpleRn04Fiadores, maximoRn03 } from './creditosService';
import { ItemValidacion } from './ItemValidacion';
import { TablaIngresosEgresos, FilaMonto } from './TablaIngresosEgresos';
import { TablaAmortizacion } from './TablaAmortizacion';
import { BuscadorSocio } from '../../core/components/BuscadorSocio';
import { HOY } from '../../core/lib/dates';
import { fmt } from '../../core/lib/format';
import { Configuracion } from '../../core/types';
import { SocioResumen } from '../socios/types';
import {
    DestinoCredito, DESTINOS_CREDITO, NuevaSolicitud, NuevoDesembolso,
    Credito, TablaCredito, FiadorSolicitud,
} from './types';

export function NuevaSolicitudTab({ moneda }: { moneda: string }) {
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [solicitante, setSolicitante] = useState<SocioResumen | null>(null);
    const [fecha, setFecha] = useState(HOY());
    const [destino, setDestino] = useState<DestinoCredito | ''>('');

    const [ingresos, setIngresos] = useState<FilaMonto[]>([]);
    const [egresos, setEgresos] = useState<FilaMonto[]>([]);
    const [monto, setMonto] = useState('');
    const [plazo, setPlazo] = useState('');
    const [tasa, setTasa] = useState('3');

    const [fiador1, setFiador1] = useState<SocioResumen | null>(null);
    const [fiador2, setFiador2] = useState<SocioResumen | null>(null);
    const [accionesReales, setAccionesReales] = useState(0);
    const [accionesFiador1, setAccionesFiador1] = useState('');
    const [accionesFiador2, setAccionesFiador2] = useState('');

    const [config, setConfig] = useState<Configuracion | null>(null);
    const [tabla, setTabla] = useState<TablaCredito | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);

    useEffect(() => {
        creditosService.listarSociosActivos()
            .then(setSocios)
            .catch(e => setError(e?.toString() ?? 'Error al cargar socios'));
        creditosService.obtenerConfiguracion()
            .then(c => { setConfig(c); setTasa(String(c.tasa_interes_ordinario)); })
            .catch(() => {});
    }, []);

    // Las acciones del socio se leen del libro de acciones (RF-56): el titular
    // respalda con las acciones que REALMENTE posee, no con un campo escrito.
    useEffect(() => {
        if (!solicitante) { setAccionesReales(0); return; }
        let vigente = true;
        creditosService.accionesDeSocio(solicitante.id)
            .then(n => { if (vigente) setAccionesReales(n); })
            .catch(() => { if (vigente) setAccionesReales(0); });
        return () => { vigente = false; };
    }, [solicitante]);

    const totalIngresos = useMemo(() => ingresos.reduce((a, f) => a + (Number(f.monto) || 0), 0), [ingresos]);
    const totalEgresos = useMemo(() => egresos.reduce((a, f) => a + (Number(f.monto) || 0), 0), [egresos]);
    const capacidadPago = Math.max(0, totalIngresos - totalEgresos);

    // RF-46: la tabla la calcula el backend con la tasa configurada (RF-59).
    useEffect(() => {
        const importe = Number(monto);
        const cuotas = Number(plazo);
        if (!importe || importe <= 0 || !cuotas || cuotas <= 0) { setTabla(null); return; }
        let vigente = true;
        creditosService.previsualizarTabla(importe, cuotas)
            .then(t => { if (vigente) { setTabla(t); setError(''); } })
            .catch(e => { if (vigente) { setTabla(null); setError(e?.toString() ?? ''); } });
        return () => { vigente = false; };
    }, [monto, plazo]);

    const valorNominal = config?.valor_nominal ?? 10000;
    const pctSocio = config?.pct_garantia_socio ?? 20;
    const pctFiador = config?.pct_garantia_fiador ?? 20;

    // RN-03 (RF-56): relación 1 a 5.
    const valorAcciones = accionesReales * valorNominal;
    const maxRn03 = maximoRn03(accionesReales, config);
    const okRn03 = !solicitante || (Number(monto) > 0 && cumpleRn03(Number(monto), accionesReales, config));

    // RN-04 (RF-49): garantía por partes.
    const requeridoSocio = (Number(monto) || 0) * pctSocio / 100;
    const garantiaSocio = accionesReales * valorNominal;
    const okRn04Socio = !solicitante || cumpleRn04Socio(Number(monto) || 0, accionesReales, config);

    const accionesComprometidas = (Number(accionesFiador1) || 0) + (Number(accionesFiador2) || 0);
    const requeridoFiadores = (Number(monto) || 0) * pctFiador / 100;
    const garantiaFiadores = accionesComprometidas * valorNominal;
    const okRn04Fiadores = (fiador1 || fiador2) ? cumpleRn04Fiadores(Number(monto) || 0, accionesComprometidas, config) : false;

    // RN-05: sin fiadores cruzados — el fiador no puede ser el titular.
    const okRn05: boolean = !!(
        (fiador1 && fiador1.id !== solicitante?.id) || (fiador2 && fiador2.id !== solicitante?.id)
    );

    function fiadores(): FiadorSolicitud[] {
        const lista: FiadorSolicitud[] = [];
        if (fiador1) lista.push({ cedula: fiador1.cedula, acciones_comprometidas: Number(accionesFiador1) || 0 });
        if (fiador2) lista.push({ cedula: fiador2.cedula, acciones_comprometidas: Number(accionesFiador2) || 0 });
        return lista;
    }

    function validar(): string {
        return validarNuevaSolicitud({
            solicitante, monto: Number(monto), plazo: Number(plazo), destino,
            fiadores: fiadores(), accionesSolicitante: accionesReales,
            accionesComprometidas, config,
        }).mensaje;
    }

    async function enviarParaAprobacion() {
        const msg = validar();
        if (msg) { setError(msg); return; }
        setError(''); setSuccess(''); setGuardando(true);
        try {
            const sol: NuevaSolicitud = {
                socio_id: solicitante!.id,
                monto_solicitado: Number(monto),
                plazo_cuotas: Number(plazo),
                destino: destino as DestinoCredito,
                total_ingresos: totalIngresos,
                total_egresos: totalEgresos,
                fiadores: fiadores(),
            };
            const guardada = await creditosService.registrarSolicitud(sol);
            setSuccess(`Solicitud ${guardada.estado === 'PENDIENTE' ? 'enviada a la Junta' : 'registrada'}: ${
                fmt(guardada.monto_solicitado, moneda)} a ${guardada.plazo_cuotas} cuotas.`);
            setMonto(''); setPlazo(''); setTabla(null);
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo registrar la solicitud');
        } finally {
            setGuardando(false);
        }
    }

    async function registrarDirecto() {
        const msg = validar();
        if (msg) { setError(msg); return; }
        setError(''); setSuccess(''); setGuardando(true);
        try {
            const des: NuevoDesembolso = {
                solicitud_id: null,
                socio_id: solicitante!.id,
                monto: Number(monto),
                plazo_cuotas: Number(plazo),
                destino: destino as DestinoCredito,
                fiadores: fiadores(),
                fecha,
            };
            const credito: Credito = await creditosService.registrarDesembolso(des);
            setSuccess(`Crédito Nº ${credito.numero} desembolsado por ${fmt(credito.monto_original, moneda)}.`);
            setMonto(''); setPlazo(''); setTabla(null);
        } catch (err: any) {
            setError(err?.toString() ?? 'No se pudo registrar el desembolso');
        } finally {
            setGuardando(false);
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            {/* ── Solicitante ── */}
            <h3 className="section-title">SOLICITANTE</h3>
            <BuscadorSocio
                placeholder="Buscar socio…"
                socios={socios}
                valor={solicitante ? `${solicitante.nombre_completo} — ${solicitante.cedula}` : ''}
                onSelect={setSolicitante}
            />

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Destino del crédito</label>
                    <select className="form-input" value={destino} onChange={e => setDestino(e.target.value as DestinoCredito)}>
                        <option value="">Seleccione un destino…</option>
                        {DESTINOS_CREDITO.map(d => (
                            <option key={d.codigo} value={d.codigo}>{d.etiqueta}</option>
                        ))}
                    </select>
                </div>
            </div>

            {/* ── Ingresos / Egresos ── */}
            <div className="config-grid mt-4">
                <div>
                    <h4 className="text-sm" style={{ fontWeight: 600, marginBottom: '0.5rem' }}>Ingresos</h4>
                    <TablaIngresosEgresos filas={ingresos} setFilas={setIngresos} ejemplos={['Ej: Salario', 'Ej: Negocio propio']} />
                </div>
                <div>
                    <h4 className="text-sm" style={{ fontWeight: 600, marginBottom: '0.5rem' }}>Egresos</h4>
                    <TablaIngresosEgresos filas={egresos} setFilas={setEgresos} ejemplos={['Ej: Alimentación', 'Ej: Transporte']} />
                </div>
            </div>

            <div className="mt-4" style={{ display: 'flex', justifyContent: 'flex-end' }}>
                <div className="campo-lectura" style={{ textAlign: 'right' }}>
                    <span className="form-label">Capacidad de pago calculada</span>
                    <strong style={{ fontSize: '1.25rem', color: 'var(--primary-color)' }}>{fmt(capacidadPago, moneda)}/mes</strong>
                </div>
            </div>

            {/* ── Datos del Crédito ── */}
            <h3 className="section-title mt-4">DATOS DEL CRÉDITO</h3>
            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto solicitado</label>
                    <input className="form-input" inputMode="numeric" placeholder="Ej: 500000"
                        value={monto} onChange={e => setMonto(e.target.value.replace(/\D/g, ''))} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Tasa (%)</label>
                    <input className="form-input" readOnly
                        value={tasa}
                        title="La tasa se toma de la configuración del Bankomunal (RF-59)"
                        style={{ backgroundColor: '#f8fafc', color: '#64748b' }} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Plazo (cuotas)</label>
                    <input className="form-input" inputMode="numeric" placeholder="Ej: 12"
                        value={plazo} onChange={e => setPlazo(e.target.value.replace(/\D/g, ''))} />
                </div>
            </div>

            {/* ── Vista previa ── */}
            <h3 className="section-title">VISTA PREVIA — TABLA DE PAGOS (saldo decreciente)</h3>
            {tabla ? (
                <TablaAmortizacion cuotas={tabla.cuotas} monto={Number(monto)} moneda={moneda} />
            ) : (
                <div className="text-sm text-muted">Complete el monto y el plazo para ver la tabla de pagos.</div>
            )}
            {tabla && (
                <div className="text-sm text-muted mt-4" style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span>Total a pagar: <strong>{fmt(tabla.monto_total, moneda)}</strong></span>
                    <span>Cuota mensual: <strong>{fmt(tabla.cuota_mensual, moneda)}</strong></span>
                </div>
            )}

            {/* ── Garantías ── */}
            <h3 className="section-title">GARANTÍAS</h3>
            <div className="config-grid">
                <div>
                    <label className="form-label">Fiador 1</label>
                    <BuscadorSocio
                        placeholder="Buscar socio…"
                        socios={socios}
                        valor={fiador1 ? `${fiador1.nombre_completo} — ${fiador1.cedula}` : ''}
                        onSelect={setFiador1}
                    />
                </div>
                <div>
                    <label className="form-label">Fiador 2</label>
                    <BuscadorSocio
                        placeholder="Buscar socio…"
                        socios={socios}
                        valor={fiador2 ? `${fiador2.nombre_completo} — ${fiador2.cedula}` : ''}
                        onSelect={setFiador2}
                    />
                </div>
            </div>

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Acciones del solicitante</label>
                    <input className="form-input" value={`${accionesReales} acciones = ${fmt(valorAcciones, moneda)}`} readOnly title="Las acciones se leen del libro de acciones (RF-56); el solicitante respalda con las que posee realmente" />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Acciones en garantía (Fiador 1)</label>
                    <input className="form-input" inputMode="numeric"
                        value={accionesFiador1} onChange={e => setAccionesFiador1(e.target.value.replace(/\D/g, ''))} />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Acciones en garantía (Fiador 2)</label>
                    <input className="form-input" inputMode="numeric"
                        value={accionesFiador2} onChange={e => setAccionesFiador2(e.target.value.replace(/\D/g, ''))} />
                </div>
            </div>

            {/* ── Validaciones automáticas ── */}
            <h3 className="section-title">VALIDACIONES AUTOMÁTICAS</h3>
            <div style={{ display: 'grid', gap: '0.5rem' }}>
                <ItemValidacion
                    ok={okRn03}
                    titulo="Relación 1 a 5"
                    detalle={solicitante
                        ? `Solicita ${fmt(Number(monto) || 0, moneda)} → máx. ${fmt(maxRn03, moneda)} = 5 × (${accionesReales} acciones × ${fmt(valorNominal, moneda)})`
                        : 'Seleccione el socio para calcular su cupo'}
                />
                <ItemValidacion
                    ok={okRn04Socio}
                    titulo={`Cobertura del solicitante ${pctSocio}%`}
                    detalle={solicitante
                        ? `${fmt(garantiaSocio, moneda)} (${accionesReales} acciones) ${okRn04Socio ? '≥' : '<'} ${pctSocio}% × ${fmt(Number(monto) || 0, moneda)} = ${fmt(requeridoSocio, moneda)}`
                        : 'Seleccione el socio para ver su cobertura'}
                />
                <ItemValidacion
                    ok={okRn04Fiadores}
                    titulo={`Cobertura de los fiadores ${pctFiador}%`}
                    detalle={(fiador1 || fiador2)
                        ? `${fmt(garantiaFiadores, moneda)} (${accionesComprometidas} acciones comprometidas) ${okRn04Fiadores ? '≥' : '<'} ${pctFiador}% × ${fmt(Number(monto) || 0, moneda)} = ${fmt(requeridoFiadores, moneda)}`
                        : 'Agregue al menos un fiador'}
                />
                <ItemValidacion
                    ok={okRn05}
                    titulo="Sin fiadores cruzados"
                    detalle={okRn05
                        ? 'El fiador no coincide con el solicitante'
                        : 'Seleccione un fiador distinto del solicitante'}
                />
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            <div className="mt-4" style={{ display: 'flex', gap: '1rem', flexWrap: 'wrap' }}>
                <button className="btn btn-primary" onClick={enviarParaAprobacion} disabled={guardando}>
                    {guardando ? 'Guardando…' : 'Enviar para aprobación'}
                </button>
                <button className="btn btn-secondary" onClick={registrarDirecto} disabled={guardando}>
                    Registrar directo como Desembolso
                </button>
            </div>
        </div>
    );
}