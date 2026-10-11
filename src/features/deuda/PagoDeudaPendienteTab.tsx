import { useEffect, useRef, useState } from 'react';
import { api } from '../../core/lib/api';
import { SocioDeudor } from './types';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';
import { mensajeDeError } from '../../core/lib/errores';

export function PagoDeudaPendienteTab({ moneda }: { moneda: string }) {
    const [sociosDeudores, setSociosDeudores] = useState<SocioDeudor[]>([]);
    const [socioId, setSocioId] = useState('');

    const [montoStr, setMontoStr] = useState('');
    const [fecha, setFecha] = useState(() => new Date().toISOString().split('T')[0]);
    const [pagando, setPagando] = useState(false);
    const [exito, setExito] = useState('');
    const [error, setError] = useState('');
    const enviando = useRef(false);

    async function cargar() {
        try {
            const retirados = await api.listarRetiradosConDeuda();
            setSociosDeudores(retirados);
            if (socioId && !retirados.find(r => r.id === socioId)) {
                setSocioId('');
            }
        } catch (e) {
            setError(mensajeDeError(e, 'Error cargando endeudados'));
        }
    }

    useEffect(() => {
        cargar();
    }, []);

    const socioElegido = sociosDeudores.find(s => s.id === socioId);
    const haySocio = !!socioElegido;

    async function procederAlPago() {
        if (enviando.current || pagando) return;
        if (!socioElegido) return;
        const monto = Number(montoStr);
        if (!monto || monto <= 0) {
            setError('El monto debe ser numérico y mayor a cero.');
            return;
        }

        enviando.current = true;
        setPagando(true);
        setError('');
        setExito('');
        try {
            const nuevoSaldo = await api.registrarPagoDeudaPendiente({
                socio_id: socioElegido.id,
                fecha,
                monto,
            });
            setExito(`Pago registrado exitosamente. ${nuevoSaldo === 0
                ? 'El socio ha saldado toda su deuda (paz y salvo).'
                : `Nuevo saldo: ${fmt(nuevoSaldo, moneda)}`}`);
            setMontoStr('');
            // Recargar la lista (el socio puede desaparecer si el saldo llegó a 0)
            await cargar();
        } catch (e) {
            setError(mensajeDeError(e, 'Error al procesar pago'));
        } finally {
            enviando.current = false;
            setPagando(false);
        }
    }

    return (
        <div className="detalle-card" style={{ maxWidth: 700 }}>
            {/* ── Socio ── */}
            <div className="form-group custom-fg mb-4">
                <label className="form-label" style={{ fontWeight: 600 }}>Socio retirado con deuda</label>
                <select
                    className="form-input"
                    value={socioId}
                    onChange={e => { setSocioId(e.target.value); setExito(''); setError(''); setMontoStr(''); }}
                >
                    <option value="">— Selecciona un socio —</option>
                    {sociosDeudores.map(s => (
                        <option key={s.id} value={s.id}>
                            {`${s.nombres} ${s.apellidos}`.trim()} — Cédula: {s.cedula}
                        </option>
                    ))}
                </select>
                {sociosDeudores.length === 0 && (
                    <div className="text-sm text-muted mt-2">No hay socios retirados con deuda en este momento.</div>
                )}
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {exito && <div className="success-message mt-4">{exito}</div>}

            {/* ── Caja de saldo: siempre visible (placeholder si no hay socio) ── */}
            <div className="detalle-card mt-4" style={{ background: '#f1f5f9', border: 'none' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '1.05rem', fontWeight: 500 }}>
                    <span>Saldo pendiente</span>
                    <strong style={{ fontSize: '1.15rem', color: haySocio ? 'var(--error-color)' : 'inherit' }}>
                        {fmt(haySocio ? socioElegido!.saldo_incobrable : 0, moneda)}
                    </strong>
                </div>
            </div>

            {/* ── Monto a abonar ── */}
            <div className="form-group custom-fg mt-4">
                <label className="form-label">Monto a abonar</label>
                <input
                    className="form-input"
                    type="text"
                    inputMode="numeric"
                    disabled={!haySocio}
                    placeholder="Ej: 85000"
                    value={montoStr ? fmtNum(parseNum(montoStr)) : ''}
                    onChange={e => setMontoStr(e.target.value.replace(/\D/g, ''))}
                />
            </div>

            <div className="form-group custom-fg mt-4" style={{ maxWidth: 200 }}>
                <label className="form-label">Fecha del pago</label>
                <input className="form-input" type="date" value={fecha} disabled={!haySocio} onChange={e => setFecha(e.target.value)} />
            </div>

            {haySocio && Number(montoStr) >= socioElegido!.saldo_incobrable && Number(montoStr) > 0 && (
                <div className="text-sm mt-2" style={{ color: 'var(--success-color)' }}>
                    Este pago dejará al socio al día (paz y salvo) y cambiará su estatus.
                </div>
            )}

            <div className="mt-6">
                <button className="btn btn-primary" onClick={procederAlPago} disabled={pagando || !haySocio} style={{ padding: '0.75rem 2rem', fontSize: '1rem' }}>
                    {pagando ? 'Registrando…' : 'Registrar pago'}
                </button>
            </div>
        </div>
    );
}
