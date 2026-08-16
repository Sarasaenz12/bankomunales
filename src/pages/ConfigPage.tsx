import { useState, useEffect } from 'react';
import { useNavigate, useOutletContext } from 'react-router-dom';
import { api } from '../lib/api';
import { Bankomunal, Configuracion, DatosGenerales } from '../types';

type OutletCtx = { bank: Bankomunal | null };

function fmt(n: number, moneda = 'COP') {
    return new Intl.NumberFormat('es-CO', { style: 'currency', currency: moneda, maximumFractionDigits: 0 }).format(n);
}
function fmts(n: number) {
    if (n >= 1000000) return `$${(n / 1000000).toFixed(2)}M`;
    if (n >= 1000) return `$${(n / 1000).toFixed(1)}K`;
    return `$${n}`;
}

export function ConfigPage() {
    const navigate = useNavigate();
    const { bank } = useOutletContext<OutletCtx>();
    const [datos, setDatos] = useState<DatosGenerales | null>(null);
    const [config, setConfig] = useState<Configuracion | null>(null);

    const [quienRealiza, setQuienRealiza] = useState('');
    const [motivo, setMotivo] = useState('');
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [activeTab, setActiveTab] = useState<'generales' | 'configuracion'>('configuracion');

    useEffect(() => {
        if (!bank) return;
        loadData();
    }, [bank]);

    const loadData = async () => {
        try {
            const configData = await api.obtenerConfiguracion();
            setConfig(configData);
            const generalData = await api.obtenerDatosGenerales();
            setDatos(generalData);
        } catch (err: any) {
            setError(err?.toString() || 'Error al cargar configuración');
        }
    };

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setSuccess('');

        if (!quienRealiza) {
            return setError('Debes indicar tu nombre para confirmar el cambio.');
        }
        if (!config) return;

        try {
            await api.actualizarConfiguracion(config, quienRealiza, motivo || 'Modificación de parámetros');
            setSuccess('Configuración guardada exitosamente.');
            setQuienRealiza('');
            setMotivo('');
            loadData();
        } catch (err: any) {
            setError(err?.toString() || 'Error al guardar la configuración');
        }
    };

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        const { name, value } = e.target;
        setConfig(prev => prev ? ({ ...prev, [name]: parseFloat(value) || 0 }) : null);
    };

    if (!config || !datos) {
        if (error) return (
            <div className="page">
                <div className="error-message">{error}</div>
                <button className="btn btn-secondary mt-4" onClick={() => navigate('/')}>Volver</button>
            </div>
        );
        return <div className="page">Cargando...</div>;
    }

    return (
        <div className="page" style={{ maxWidth: '1000px' }}>
            <div className="page-header">
                <div>
                    <h1 className="page-title">Datos del Bankomunal</h1>
                    <p className="page-subtitle">Información general y configuración</p>
                </div>
                <span className="badge-mes">Mes abierto</span>
            </div>

            <div className="tabs">
                <button
                    className={`tab ${activeTab === 'generales' ? 'active' : ''}`}
                    onClick={() => setActiveTab('generales')}
                >
                    Datos Generales
                </button>
                <button
                    className={`tab ${activeTab === 'configuracion' ? 'active' : ''}`}
                    onClick={() => setActiveTab('configuracion')}
                >
                    Configuración
                </button>
            </div>

            {activeTab === 'generales' && (
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1.5fr', gap: '2rem', alignItems: 'start' }}>
                    <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', backgroundColor: '#f8fafc', border: '1px solid var(--border-color)', boxShadow: 'none' }}>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <div className="text-sm text-primary font-medium" style={{ color: 'var(--primary-hover)', fontWeight: 600 }}>Código</div>
                            <div className="font-semibold" style={{ fontWeight: 600 }}>BK-{datos.id.substring(0, 4).toUpperCase()}</div>
                        </div>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <div className="text-sm text-primary font-medium" style={{ color: 'var(--primary-hover)', fontWeight: 600 }}>Nombre</div>
                            <div className="font-semibold" style={{ fontWeight: 600 }}>{datos.nombre}</div>
                        </div>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <div className="text-sm text-primary font-medium" style={{ color: 'var(--primary-hover)', fontWeight: 600 }}>Ubicación</div>
                            <div>{datos.ubicacion || '—'}</div>
                        </div>
                        <div style={{ marginBottom: '1.5rem' }}>
                            <div className="text-sm text-primary font-medium" style={{ color: 'var(--primary-hover)', fontWeight: 600 }}>Moneda</div>
                            <div>$ {datos.moneda}</div>
                        </div>
                        <div>
                            <div className="text-sm text-primary font-medium" style={{ color: 'var(--primary-hover)', fontWeight: 600 }}>Fecha de creación</div>
                            <div>{datos.fecha_creacion?.slice(0, 10)}</div>
                        </div>
                    </div>

                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Créditos otorgados</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{datos.numero_creditos_otorgados}</div>
                        </div>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Monto total créditos</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmts(datos.monto_total_creditos)}</div>
                        </div>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Acciones vendidas</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{datos.numero_acciones_vendidas}</div>
                        </div>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Fondo de Gastos</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmts(datos.saldo_fondo_gastos)}</div>
                        </div>
                        <div className="card" style={{ gridColumn: '1 / -1', padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Fondo de Incobrables</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmts(datos.saldo_fondo_incobrables)}</div>
                        </div>
                    </div>
                </div>
            )}

            {activeTab === 'configuracion' && (
                <form onSubmit={handleSave}>
                    {/* CONDICIONES DE CRÉDITO */}
                    <div className="config-section">
                        <h3 className="section-title">CONDICIONES DE CRÉDITO</h3>
                        <div className="config-grid" style={{ gridTemplateColumns: 'repeat(4, 1fr)' }}>
                            <div className="form-group custom-fg">
                                <label className="form-label">Plazo máximo (meses)</label>
                                <input type="number" name="plazo_maximo_cuotas" className="form-input" value={config.plazo_maximo_cuotas} onChange={handleInputChange} min="1" />
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">Tasa interés ordinario (%)</label>
                                <input type="number" name="tasa_interes_ordinario" className="form-input" value={config.tasa_interes_ordinario} onChange={handleInputChange} min="0" step="any" />
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">Tasa interés de mora (%)</label>
                                <input type="number" name="tasa_interes_mora" className="form-input" value={config.tasa_interes_mora} onChange={handleInputChange} min="0" step="any" />
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">Monto máximo</label>
                                <input type="number" name="monto_maximo_credito" className="form-input" value={config.monto_maximo_credito} onChange={handleInputChange} min="0" />
                            </div>
                        </div>
                    </div>

                    {/* GARANTÍAS */}
                    <div className="config-section">
                        <h3 className="section-title">GARANTÍAS (RN-04)</h3>
                        <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                            <div className="form-group custom-fg">
                                <label className="form-label">% exigido al socio</label>
                                <input type="number" name="pct_garantia_socio" className="form-input" value={config.pct_garantia_socio} onChange={handleInputChange} min="0" max="100" />
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">% exigido al fiador</label>
                                <input type="number" name="pct_garantia_fiador" className="form-input" value={config.pct_garantia_fiador} onChange={handleInputChange} min="0" max="100" />
                            </div>
                        </div>
                    </div>

                    {/* FONDOS */}
                    <div className="config-section">
                        <h3 className="section-title">FONDOS (RN-07, RN-08)</h3>
                        <p className="text-sm text-muted" style={{ fontSize: '12px', marginTop: '-0.5rem', marginBottom: '1rem' }}>
                            Cada fondo aparta un % de las ganancias del mes en el Cierre. Lo que queda se reparte entre los socios.
                        </p>
                        <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                            <div className="form-group custom-fg">
                                <label className="form-label">% Fondo de Gastos</label>
                                <input type="number" name="pct_fondo_gastos" className="form-input" value={config.pct_fondo_gastos} onChange={handleInputChange} min="0" max="100" />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    Papelería, transporte y demás gastos operativos
                                </div>
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">% Fondo de Incobrables</label>
                                <input type="number" name="pct_fondo_incobrables" className="form-input" value={config.pct_fondo_incobrables} onChange={handleInputChange} min="0" max="100" />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    Colchón por si un socio se retira debiendo
                                </div>
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">% Tope de la Reserva</label>
                                <input type="number" name="tope_reserva_incobrables_pct" className="form-input" value={config.tope_reserva_incobrables_pct} onChange={handleInputChange} min="0" max="100" />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    Al llegar a este % del capital en acciones, deja de crecer
                                </div>
                            </div>
                        </div>
                        <div className="form-group custom-fg mt-4">
                            <label className="form-label">Reserva por Incobrables actual</label>
                            <input type="text" className="form-input" readOnly value={fmt(datos.saldo_fondo_incobrables, datos.moneda)} style={{ backgroundColor: '#f8fafc', color: '#64748b' }} />
                            <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                Solo lectura — se descuenta automáticamente al liquidar socios con deuda mayor a su valor
                            </div>
                        </div>
                    </div>

                    {/* ACCIONES */}
                    <div className="config-section">
                        <h3 className="section-title">ACCIONES (RN-13)</h3>
                        <div className="form-group custom-fg" style={{ maxWidth: '300px' }}>
                            <label className="form-label">Valor nominal de la acción</label>
                            <input type="number" name="valor_nominal" className="form-input" value={config.valor_nominal} onChange={handleInputChange} min="1" step="any" required />
                        </div>
                    </div>

                    {/* VENTA DE ACCIONES */}
                    <div className="config-section">
                        <h3 className="section-title">VENTA DE ACCIONES (RN-09, RN-15)</h3>

                        <table style={{ width: '100%', marginBottom: '1.5rem', borderCollapse: 'collapse' }}>
                            <thead>
                                <tr>
                                    <th style={{ textAlign: 'left', fontSize: '11px', color: '#64748b', letterSpacing: '0.05em', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-color)' }}>RANGO DE PPCFC</th>
                                    <th style={{ textAlign: 'right', fontSize: '11px', color: '#64748b', letterSpacing: '0.05em', paddingBottom: '0.5rem', borderBottom: '1px solid var(--border-color)' }}>% AUTORIZADO A VENDER</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <td style={{ padding: '1rem 0', borderBottom: '1px solid var(--border-color)', fontSize: '0.875rem' }}>80% — 90%</td>
                                    <td style={{ padding: '0.5rem 0', borderBottom: '1px solid var(--border-color)', textAlign: 'right' }}>
                                        <input type="number" name="ppcfc_venta_rango1_pct" className="form-input" value={config.ppcfc_venta_rango1_pct} onChange={handleInputChange} style={{ width: '80px', display: 'inline-block', textAlign: 'center' }} />
                                    </td>
                                </tr>
                                <tr>
                                    <td style={{ padding: '1rem 0', borderBottom: '1px solid var(--border-color)', fontSize: '0.875rem' }}>90% — 100%</td>
                                    <td style={{ padding: '0.5rem 0', borderBottom: '1px solid var(--border-color)', textAlign: 'right' }}>
                                        <input type="number" name="ppcfc_venta_rango2_pct" className="form-input" value={config.ppcfc_venta_rango2_pct} onChange={handleInputChange} style={{ width: '80px', display: 'inline-block', textAlign: 'center' }} />
                                    </td>
                                </tr>
                            </tbody>
                        </table>

                        <div className="config-grid" style={{ gridTemplateColumns: '1fr 1fr' }}>
                            <div className="form-group custom-fg">
                                <label className="form-label">Tope individual mensual por socio (%) — RN-15</label>
                                <input type="number" name="tope_individual_mensual_pct" className="form-input" value={config.tope_individual_mensual_pct} onChange={handleInputChange} min="0" max="100" />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    % del cupo que el PPCFC autorizó vender este mes que puede tomar un
                                    solo socio — no del capital total. Distinto del 15% de participación
                                    acumulada (RN-02); ambos topes aplican.
                                </div>
                            </div>
                            <div className="form-group custom-fg">
                                <label className="form-label">Cupo mensual autorizado consumido</label>
                                <input type="text" className="form-input" readOnly value="-" style={{ backgroundColor: '#f8fafc', color: '#64748b' }} />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    Solo lectura — calculado automáticamente
                                </div>
                            </div>
                        </div>
                    </div>

                    <div style={{ marginTop: '2rem' }}>
                        <div className="form-group custom-fg" style={{ maxWidth: '400px' }}>
                            <label className="form-label" style={{ fontWeight: 600, color: '#0f172a' }}>Tu nombre *</label>
                            <input type="text" className="form-input" value={quienRealiza} onChange={e => setQuienRealiza(e.target.value)} placeholder="Nombre de quien hace el cambio" />
                        </div>

                        {error && <div className="error-message mt-4 mb-4">{error}</div>}
                        {success && <div className="success-message mt-4 mb-4">{success}</div>}

                        <button type="submit" className="btn" style={{ backgroundColor: '#6482a5', color: '#fff', padding: '0.75rem 1.5rem' }}>
                            Guardar cambios
                        </button>
                    </div>
                </form>
            )}
        </div>
    );
}
