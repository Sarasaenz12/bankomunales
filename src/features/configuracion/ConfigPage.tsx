import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { configuracionService } from './configuracionService';
import { Configuracion, DatosGenerales } from '../../core/types';
import { fmt, fmtNum, parseNum } from '../../core/lib/format';
import { PageHeader } from '../../core/components/PageHeader';
import { useBanco } from '../../core/context/BankContext';

/** Todos los campos numéricos de la Configuración. Fuente única para convertir el
 *  formulario (que edita en texto) hacia y desde la entidad tipada. */
const CAMPOS_CONFIG: (keyof Configuracion)[] = [
    'valor_nominal', 'pct_garantia_socio', 'pct_garantia_fiador', 'pct_fondo_gastos',
    'pct_fondo_incobrables', 'tope_reserva_incobrables_pct', 'ppcfc_venta_rango1_pct',
    'ppcfc_venta_rango2_pct', 'tope_individual_mensual_pct', 'plazo_maximo_cuotas',
    'tasa_interes_ordinario', 'tasa_interes_mora', 'monto_maximo_credito',
];

/** El formulario guarda los valores como texto mientras se teclea (un input vacío o "3."
 *  no es un número válido todavía). No se finge que sea `Configuracion`. */
type ConfigForm = Record<keyof Configuracion, string>;

const aFormulario = (c: Configuracion): ConfigForm =>
    Object.fromEntries(CAMPOS_CONFIG.map(k => [k, String(c[k])])) as ConfigForm;

const aConfiguracion = (f: ConfigForm): Configuracion => {
    const num = (s: string) => (Number.isNaN(Number(s)) ? 0 : Number(s));
    return {
        valor_nominal: num(f.valor_nominal),
        pct_garantia_socio: num(f.pct_garantia_socio),
        pct_garantia_fiador: num(f.pct_garantia_fiador),
        pct_fondo_gastos: num(f.pct_fondo_gastos),
        pct_fondo_incobrables: num(f.pct_fondo_incobrables),
        tope_reserva_incobrables_pct: num(f.tope_reserva_incobrables_pct),
        ppcfc_venta_rango1_pct: num(f.ppcfc_venta_rango1_pct),
        ppcfc_venta_rango2_pct: num(f.ppcfc_venta_rango2_pct),
        tope_individual_mensual_pct: num(f.tope_individual_mensual_pct),
        plazo_maximo_cuotas: num(f.plazo_maximo_cuotas),
        tasa_interes_ordinario: num(f.tasa_interes_ordinario),
        tasa_interes_mora: num(f.tasa_interes_mora),
        monto_maximo_credito: num(f.monto_maximo_credito),
    };
};

export function ConfigPage() {
    const navigate = useNavigate();
    const bank = useBanco();
    const [datos, setDatos] = useState<DatosGenerales | null>(null);
    const [config, setConfig] = useState<ConfigForm | null>(null);

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
            const { config, datos } = await configuracionService.cargarTodo();
            setConfig(aFormulario(config));
            setDatos(datos);
        } catch (err: any) {
            setError(err?.toString() || 'Error al cargar configuración');
        }
    };

    const handleSave = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setSuccess('');

        if (!quienRealiza.trim()) {
            return setError('Debes indicar tu nombre para confirmar el cambio.');
        }
        if (!config) return;

        // El formulario edita en texto; aquí se parsea a la entidad tipada (números).
        const updatedConfig = aConfiguracion(config);

        // Validaciones clave de los fondos.
        if (updatedConfig.pct_fondo_gastos + updatedConfig.pct_fondo_incobrables > 100) {
            return setError('La suma del Fondo de Gastos y Fondo de Incobrables no puede ser mayor a 100%.');
        }
        if (updatedConfig.valor_nominal <= 0) {
            return setError('El valor nominal de la acción debe ser mayor a 0.');
        }
        if (updatedConfig.plazo_maximo_cuotas <= 0) {
            return setError('El plazo máximo de crédito debe ser mayor a 0.');
        }
        if (updatedConfig.monto_maximo_credito <= 0) {
            return setError('El monto máximo de crédito debe ser mayor a 0.');
        }
        if (updatedConfig.tasa_interes_ordinario < 0 || updatedConfig.tasa_interes_mora < 0) {
            return setError('Las tasas de interés no pueden ser negativas.');
        }

        const pctFields = ['pct_garantia_socio', 'pct_garantia_fiador', 'pct_fondo_gastos', 'pct_fondo_incobrables', 'tope_reserva_incobrables_pct', 'ppcfc_venta_rango1_pct', 'ppcfc_venta_rango2_pct', 'tope_individual_mensual_pct'];
        for (const field of pctFields) {
            if (updatedConfig[field as keyof Configuracion] < 0 || updatedConfig[field as keyof Configuracion] > 100) {
                return setError('Los porcentajes de fondos y garantías deben estar entre 0 y 100.');
            }
        }

        try {
            await configuracionService.guardar(updatedConfig, quienRealiza.trim(), motivo.trim() || 'Modificación de parámetros');
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
        setConfig(prev => prev ? { ...prev, [name]: value } : null);
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
            <PageHeader titulo="Datos del Bankomunal" subtitulo="Información general y configuración" />

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
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmt(datos.monto_total_creditos)}</div>
                        </div>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Acciones vendidas</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{datos.numero_acciones_vendidas}</div>
                        </div>
                        <div className="card" style={{ padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Fondo de Gastos</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmt(datos.saldo_fondo_gastos)}</div>
                        </div>
                        <div className="card" style={{ gridColumn: '1 / -1', padding: '1.5rem', width: '100%', maxWidth: 'none', boxShadow: 'none', border: '1px solid var(--border-color)' }}>
                            <div className="text-sm text-muted mb-1">Fondo de Incobrables</div>
                            <div style={{ fontSize: '1.5rem', fontWeight: 600 }}>{fmt(datos.saldo_fondo_incobrables)}</div>
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
                                <input type="text" inputMode="numeric" name="monto_maximo_credito" className="form-input"
                                    value={config.monto_maximo_credito ? fmtNum(parseNum(config.monto_maximo_credito)) : ''}
                                    onChange={e => setConfig(prev => prev ? { ...prev, monto_maximo_credito: e.target.value.replace(/\D/g, '') } : null)} />
                            </div>
                        </div>
                    </div>

                    {/* GARANTÍAS */}
                    <div className="config-section">
                        <h3 className="section-title">GARANTÍAS</h3>
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
                        <h3 className="section-title">FONDOS</h3>
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
                        <h3 className="section-title">ACCIONES</h3>
                        <div className="form-group custom-fg" style={{ maxWidth: '300px' }}>
                            <label className="form-label">Valor nominal de la acción</label>
                            <input type="text" inputMode="numeric" name="valor_nominal" className="form-input"
                                value={config.valor_nominal ? fmtNum(parseNum(config.valor_nominal)) : ''}
                                onChange={e => setConfig(prev => prev ? { ...prev, valor_nominal: e.target.value.replace(/\D/g, '') } : null)} required />
                        </div>
                    </div>

                    {/* VENTA DE ACCIONES */}
                    <div className="config-section">
                        <h3 className="section-title">VENTA DE ACCIONES</h3>

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
                                <label className="form-label">Tope individual mensual por socio (%)</label>
                                <input type="number" name="tope_individual_mensual_pct" className="form-input" value={config.tope_individual_mensual_pct} onChange={handleInputChange} min="0" max="100" />
                                <div className="text-sm text-muted mt-1" style={{ fontSize: '11px' }}>
                                    % del cupo que el PPCFC autorizó vender este mes que puede tomar un
                                    solo socio — no del capital total. Distinto del 15% de participación
                                    acumulada; ambos topes aplican.
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
