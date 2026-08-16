import { useEffect, useState } from 'react';
import { useOutletContext } from 'react-router-dom';
import { api } from '../lib/api';
import {
    Bankomunal, Movimiento, ResumenCaja, Bien, CodigoOperacion, TipoBien,
    OPERACIONES_CAJA,
} from '../types';

type OutletCtx = { bank: Bankomunal | null };
type Pestana = 'libro' | 'registrar' | 'bienes';

const HOY = () => new Date().toISOString().slice(0, 10);

function fmt(n: number, moneda = 'COP') {
    return new Intl.NumberFormat('es-CO', {
        style: 'currency', currency: moneda, maximumFractionDigits: 0,
    }).format(n);
}

const ETIQUETA_CODIGO: Record<string, string> = {
    OI: 'Otro Ingreso', EG: 'Otro Egreso',
    IFG: 'Ingreso Fondo Gastos', GBK: 'Gasto del Bankomunal',
    VC: 'Venta de Acciones', LC: 'Liquidación de Acciones', UR: 'Ganancia Repartida',
    CON: 'Desembolso de Crédito', OR: 'Interés Ordinario', PC: 'Pago de Cuota',
    MO: 'Interés de Mora', PDP: 'Pago Deuda Pendiente', COR: 'Refinanciamiento',
};

export function CajaPage() {
    const { bank } = useOutletContext<OutletCtx>();
    const moneda = bank?.moneda ?? 'COP';

    const [pestana, setPestana] = useState<Pestana>('libro');
    const [libro, setLibro] = useState<Movimiento[]>([]);
    const [resumen, setResumen] = useState<ResumenCaja | null>(null);
    const [bienes, setBienes] = useState<Bien[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');

    useEffect(() => { if (bank) recargar(); }, [bank]);

    async function recargar() {
        try {
            setLibro(await api.listarLibro({ desde: desde || null, hasta: hasta || null }));
            setResumen(await api.resumenCaja());
            setBienes(await api.listarBienes());
        } catch (e: any) {
            setError(e?.toString() ?? 'Error al cargar la caja');
        }
    }

    function aviso(msg: string) {
        setSuccess(msg);
        setError('');
        recargar();
    }

    return (
        <div className="page">
            <div className="page-header">
                <div>
                    <h1 className="page-title">Caja y Contabilidad</h1>
                    <p className="page-subtitle">Libro de Ingresos y Egresos del Bankomunal</p>
                </div>
                <span className="badge-mes">Mes abierto</span>
            </div>

            <div className="kpi-grid">
                <div className="kpi-card">
                    <div className="kpi-label">Saldo en caja</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.saldo_caja, moneda) : '…'}</div>
                    <div className="kpi-sub">Disponible en efectivo</div>
                </div>
                <div className="kpi-card">
                    <div className="kpi-label">Fondo para Gastos</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.saldo_fondo_gastos, moneda) : '…'}</div>
                    <div className="kpi-sub">Para gastos operativos (RN-07)</div>
                </div>
                <div className="kpi-card">
                    <div className="kpi-label">Activo fijo</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.valor_activo_fijo, moneda) : '…'}</div>
                    <div className="kpi-sub">Bienes propios y en comodato</div>
                </div>
            </div>

            <div className="tabs">
                <button className={`tab ${pestana === 'libro' ? 'active' : ''}`} onClick={() => setPestana('libro')}>
                    Libro de Ingresos y Egresos
                </button>
                <button className={`tab ${pestana === 'registrar' ? 'active' : ''}`} onClick={() => setPestana('registrar')}>
                    Registrar operación
                </button>
                <button className={`tab ${pestana === 'bienes' ? 'active' : ''}`} onClick={() => setPestana('bienes')}>
                    Bienes / Activo Fijo
                </button>
            </div>

            {error && <div className="error-message mb-4">{error}</div>}
            {success && <div className="success-message mb-4">{success}</div>}

            {pestana === 'libro' && (
                <LibroTab
                    libro={libro} moneda={moneda}
                    desde={desde} hasta={hasta}
                    setDesde={setDesde} setHasta={setHasta}
                    onFiltrar={recargar}
                    onCorregido={() => aviso('Operación corregida.')}
                    onError={setError}
                />
            )}

            {pestana === 'registrar' && (
                <RegistrarTab onHecho={aviso} onError={setError} />
            )}

            {pestana === 'bienes' && (
                <BienesTab bienes={bienes} moneda={moneda} onHecho={aviso} onError={setError} />
            )}
        </div>
    );
}

// ─────────────────────── Libro ───────────────────────

function LibroTab({ libro, moneda, desde, hasta, setDesde, setHasta, onFiltrar, onCorregido, onError }: {
    libro: Movimiento[];
    moneda: string;
    desde: string; hasta: string;
    setDesde: (v: string) => void; setHasta: (v: string) => void;
    onFiltrar: () => void;
    onCorregido: () => void;
    onError: (m: string) => void;
}) {
    const [corrigiendo, setCorrigiendo] = useState<Movimiento | null>(null);

    return (
        <>
            <div className="list-toolbar">
                <div className="form-group custom-fg" style={{ margin: 0 }}>
                    <label className="form-label">Desde</label>
                    <input type="date" className="form-input" value={desde} onChange={e => setDesde(e.target.value)} />
                </div>
                <div className="form-group custom-fg" style={{ margin: 0 }}>
                    <label className="form-label">Hasta</label>
                    <input type="date" className="form-input" value={hasta} onChange={e => setHasta(e.target.value)} />
                </div>
                <button className="btn btn-secondary" onClick={onFiltrar} style={{ alignSelf: 'flex-end' }}>
                    Filtrar
                </button>
            </div>

            <table className="activity-table">
                <thead>
                    <tr>
                        <th style={{ width: 50 }}>N°</th>
                        <th>Fecha</th>
                        <th>Operación</th>
                        <th>Descripción</th>
                        <th style={{ textAlign: 'right' }}>Ingreso</th>
                        <th style={{ textAlign: 'right' }}>Egreso</th>
                        <th style={{ textAlign: 'right' }}>Saldo</th>
                        <th style={{ width: 90 }}></th>
                    </tr>
                </thead>
                <tbody>
                    {libro.map(m => (
                        <tr key={m.id}>
                            <td>{m.numero}</td>
                            <td>{m.fecha}</td>
                            <td>
                                {ETIQUETA_CODIGO[m.codigo] ?? m.codigo}
                                {m.corregido && (
                                    <span
                                        className="badge-estatus badge-deuda"
                                        style={{ marginLeft: 6, fontSize: '0.7rem' }}
                                        title={m.motivo_correccion ?? 'Corregida tras el cierre'}
                                    >
                                        corregida
                                    </span>
                                )}
                            </td>
                            <td>{m.descripcion}</td>
                            <td style={{ textAlign: 'right' }}>{m.ingreso ? fmt(m.ingreso, moneda) : '—'}</td>
                            <td style={{ textAlign: 'right' }}>{m.egreso ? fmt(m.egreso, moneda) : '—'}</td>
                            <td style={{ textAlign: 'right', fontWeight: 500 }}>{fmt(m.saldo, moneda)}</td>
                            <td style={{ textAlign: 'right' }}>
                                <button className="link-ver" onClick={() => setCorrigiendo(m)}>Corregir</button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {libro.length === 0 && (
                <div className="empty-state">No hay operaciones registradas en este rango.</div>
            )}

            {corrigiendo && (
                <CorregirForm
                    mov={corrigiendo}
                    onCancelar={() => setCorrigiendo(null)}
                    onListo={() => { setCorrigiendo(null); onCorregido(); }}
                    onError={onError}
                />
            )}
        </>
    );
}

/** RF-89/RF-90: tras el cierre, la corrección exige nombre y motivo para Auditoría. */
function CorregirForm({ mov, onCancelar, onListo, onError }: {
    mov: Movimiento;
    onCancelar: () => void;
    onListo: () => void;
    onError: (m: string) => void;
}) {
    const [fecha, setFecha] = useState(mov.fecha);
    const [monto, setMonto] = useState(String(mov.ingreso || mov.egreso));
    const [descripcion, setDescripcion] = useState(mov.descripcion);
    const [quien, setQuien] = useState('');
    const [motivo, setMotivo] = useState('');

    async function guardar(e: React.FormEvent) {
        e.preventDefault();
        try {
            await api.corregirOperacionCaja(
                mov.id, fecha, Number(monto), descripcion,
                mov.mes_cerrado ? quien : null,
                mov.mes_cerrado ? motivo : null,
            );
            onListo();
        } catch (err: any) {
            onError(err?.toString() ?? 'No se pudo corregir la operación');
        }
    }

    return (
        <form className="detalle-card mt-4" onSubmit={guardar}>
            <h3 className="section-title">CORREGIR OPERACIÓN N° {mov.numero}</h3>

            {mov.mes_cerrado && (
                <div className="error-message mb-4">
                    El mes de esta operación ya está cerrado. Para corregirla debe indicar su
                    nombre y el motivo; el cambio quedará registrado en Auditoría (RF-90).
                </div>
            )}

            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input type="number" className="form-input" min="1" step="any" value={monto} onChange={e => setMonto(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Descripción</label>
                    <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} />
                </div>
            </div>

            {mov.mes_cerrado && (
                <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                    <div className="form-group custom-fg">
                        <label className="form-label">Tu nombre *</label>
                        <input className="form-input" value={quien} onChange={e => setQuien(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Motivo de la corrección *</label>
                        <input className="form-input" value={motivo} onChange={e => setMotivo(e.target.value)} required />
                    </div>
                </div>
            )}

            <div style={{ display: 'flex', gap: '0.75rem', marginTop: '1rem' }}>
                <button type="submit" className="btn btn-primary">Guardar corrección</button>
                <button type="button" className="btn btn-secondary" onClick={onCancelar}>Cancelar</button>
            </div>
        </form>
    );
}

// ─────────────────────── Registrar ───────────────────────

function RegistrarTab({ onHecho, onError }: {
    onHecho: (m: string) => void;
    onError: (m: string) => void;
}) {
    const [codigo, setCodigo] = useState<CodigoOperacion | 'DONACION'>('OI');
    const [fecha, setFecha] = useState(HOY());
    const [monto, setMonto] = useState('');
    const [descripcion, setDescripcion] = useState('');

    const ayuda = codigo === 'DONACION'
        ? 'Las donaciones entran automáticamente al Fondo para Gastos (RF-87)'
        : OPERACIONES_CAJA.find(o => o.codigo === codigo)?.ayuda ?? '';

    async function guardar(e: React.FormEvent) {
        e.preventDefault();
        try {
            if (codigo === 'DONACION') {
                await api.registrarDonacion(fecha, Number(monto), descripcion);
            } else {
                await api.registrarOperacionCaja({ codigo, fecha, monto: Number(monto), descripcion });
            }
            setMonto('');
            setDescripcion('');
            onHecho('Operación registrada en el Libro.');
        } catch (err: any) {
            onError(err?.toString() ?? 'No se pudo registrar la operación');
        }
    }

    return (
        <form className="detalle-card" onSubmit={guardar} style={{ maxWidth: 720 }}>
            <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Tipo de operación</label>
                    <select
                        className="form-input"
                        value={codigo}
                        onChange={e => setCodigo(e.target.value as CodigoOperacion | 'DONACION')}
                    >
                        {OPERACIONES_CAJA.map(o => (
                            <option key={o.codigo} value={o.codigo}>{o.etiqueta}</option>
                        ))}
                        <option value="DONACION">Donación</option>
                    </select>
                    <div className="text-sm text-muted mt-1" style={{ fontSize: 11 }}>{ayuda}</div>
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Fecha</label>
                    <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                </div>
            </div>

            <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                <div className="form-group custom-fg">
                    <label className="form-label">Monto</label>
                    <input type="number" className="form-input" min="1" step="any" value={monto} onChange={e => setMonto(e.target.value)} required />
                </div>
                <div className="form-group custom-fg">
                    <label className="form-label">Descripción</label>
                    <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} placeholder="Ej. fotocopias de formatos" />
                </div>
            </div>

            <button type="submit" className="btn btn-primary" style={{ marginTop: '1rem' }}>
                Registrar operación
            </button>
        </form>
    );
}

// ─────────────────────── Bienes ───────────────────────

function BienesTab({ bienes, moneda, onHecho, onError }: {
    bienes: Bien[];
    moneda: string;
    onHecho: (m: string) => void;
    onError: (m: string) => void;
}) {
    const [descripcion, setDescripcion] = useState('');
    const [fecha, setFecha] = useState(HOY());
    const [valor, setValor] = useState('');
    const [tipo, setTipo] = useState<TipoBien>('PROPIO');

    async function guardar(e: React.FormEvent) {
        e.preventDefault();
        try {
            await api.registrarBien({
                descripcion, fecha_adquisicion: fecha, valor: Number(valor || 0), tipo,
            });
            setDescripcion('');
            setValor('');
            onHecho('Bien registrado como activo fijo.');
        } catch (err: any) {
            onError(err?.toString() ?? 'No se pudo registrar el bien');
        }
    }

    return (
        <>
            <form className="detalle-card" onSubmit={guardar} style={{ maxWidth: 900 }}>
                <h3 className="section-title">REGISTRAR BIEN ADQUIRIDO</h3>
                <p className="text-sm text-muted" style={{ marginTop: '-0.5rem', marginBottom: '1rem' }}>
                    El bien se contabiliza como activo fijo y no afecta el saldo de caja (RF-88).
                    Si se pagó con el Fondo para Gastos, registre además el gasto en la pestaña anterior.
                </p>
                <div className="config-grid" style={{ gridTemplateColumns: 'repeat(4, 1fr)' }}>
                    <div className="form-group custom-fg">
                        <label className="form-label">Descripción</label>
                        <input className="form-input" value={descripcion} onChange={e => setDescripcion(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Fecha de adquisición</label>
                        <input type="date" className="form-input" value={fecha} onChange={e => setFecha(e.target.value)} required />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Valor</label>
                        <input type="number" className="form-input" min="0" step="any" value={valor} onChange={e => setValor(e.target.value)} />
                    </div>
                    <div className="form-group custom-fg">
                        <label className="form-label">Tipo</label>
                        <select className="form-input" value={tipo} onChange={e => setTipo(e.target.value as TipoBien)}>
                            <option value="PROPIO">Propio</option>
                            <option value="COMODATO">En comodato</option>
                        </select>
                    </div>
                </div>
                <button type="submit" className="btn btn-primary" style={{ marginTop: '1rem' }}>
                    Registrar bien
                </button>
            </form>

            <table className="activity-table">
                <thead>
                    <tr>
                        <th>Descripción</th>
                        <th>Adquisición</th>
                        <th>Tipo</th>
                        <th style={{ textAlign: 'right' }}>Valor</th>
                    </tr>
                </thead>
                <tbody>
                    {bienes.map(b => (
                        <tr key={b.id}>
                            <td>{b.descripcion}</td>
                            <td>{b.fecha_adquisicion}</td>
                            <td>{b.tipo === 'COMODATO' ? 'En comodato' : 'Propio'}</td>
                            <td style={{ textAlign: 'right' }}>{fmt(b.valor, moneda)}</td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {bienes.length === 0 && (
                <div className="empty-state">Todavía no hay bienes registrados.</div>
            )}
        </>
    );
}
