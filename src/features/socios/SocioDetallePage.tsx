import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { sociosService } from './sociosService';
import { validarFormularioSocio } from './validaciones';
import { DatosSocio, Protegido, Socio } from './types';
import { ResumenMesAcciones } from '../acciones/types';
import { BadgeEstatus } from '../../core/components/BadgeEstatus';
import { Campo } from '../../core/components/Campo';
import { nombreMes } from '../../core/lib/dates';
import { fmt } from '../../core/lib/format';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';

type Pestana = 'datos' | 'acciones' | 'creditos' | 'avala';

const PROTEGIDO_VACIO: Protegido = { nombre: '', cedula: '', parentesco: '', telefono: '' };

const FORM_VACIO: DatosSocio = {
    cedula: '', nombres: '', apellidos: '', profesion: '', direccion: '',
    telefono: '', celular: '', correo: '',
    beneficiario: null,
    protegidos: [],
};

/**
 * Detalle de un socio (RF-19) y alta de uno nuevo (RF-15).
 *
 * Con `id` en la ruta muestra las 4 pestañas; en `/nuevo` sólo el formulario, porque
 * un socio que todavía no existe no tiene historial que consultar.
 */
export function SocioDetallePage() {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const esNuevo = !id;
    const bank = useBanco();
    const moneda = bank?.moneda ?? 'COP';

    const [socio, setSocio] = useState<Socio | null>(null);
    const [form, setForm] = useState<DatosSocio>(FORM_VACIO);
    const [pestana, setPestana] = useState<Pestana>('datos');
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);
    const [cargando, setCargando] = useState(!esNuevo);
    /** Errores de formato por campo, calculados al intentar guardar. */
    const [errores, setErrores] = useState<Record<string, string>>({});
    const [acciones, setAcciones] = useState(0);
    const [historial, setHistorial] = useState<ResumenMesAcciones[]>([]);

    useEffect(() => {
        if (esNuevo) return;
        sociosService.obtener(id!)
            .then(s => {
                setSocio(s);
                setForm({
                    cedula: s.cedula, nombres: s.nombres, apellidos: s.apellidos,
                    profesion: s.profesion, direccion: s.direccion, telefono: s.telefono,
                    celular: s.celular, correo: s.correo,
                    beneficiario: s.beneficiario, protegidos: s.protegidos,
                });
            })
            .catch(e => setError(e?.toString() ?? 'No se pudo cargar el socio'))
            .finally(() => setCargando(false));

        sociosService.accionesDeSocio(id!).then(setAcciones).catch(() => { });
        sociosService.historialAcciones(id!).then(setHistorial).catch(() => { });
    }, [id, esNuevo]);

    async function guardar(ev: React.FormEvent) {
        ev.preventDefault();
        setError('');
        setSuccess('');

        const fallos = validarFormularioSocio(form);
        setErrores(fallos);
        if (Object.keys(fallos).length > 0) {
            setError('Revisa los campos marcados en rojo antes de guardar.');
            return;
        }

        setGuardando(true);
        try {
            if (esNuevo) {
                const creado = await sociosService.crear(form);
                navigate(`/app/socios/${creado.id}`, { replace: true });
            } else {
                setSocio(await sociosService.actualizar(id!, form));
                setSuccess('Datos del socio actualizados.');
            }
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo guardar el socio');
        } finally {
            setGuardando(false);
        }
    }

    const set = (campo: keyof DatosSocio) => (valor: string) =>
        setForm(f => ({ ...f, [campo]: valor }));

    if (cargando) return <div className="page">Cargando…</div>;

    const titulo = esNuevo
        ? 'Nuevo Socio'
        : `${form.nombres} ${form.apellidos}`.trim() || 'Socio';

    return (
        <div className="page">
            <PageHeader
                titulo={esNuevo ? 'Nuevo Socio' : 'Detalle del Socio'}
                subtitulo={esNuevo ? 'Registro de un socio' : titulo}
            />

            <div className="panel-header" style={{ marginBottom: '1.5rem' }}>
                <h2 className="panel-title" style={{ fontSize: '1.25rem' }}>{titulo}</h2>
                <button className="link-ver" onClick={() => navigate('/app/socios')}>
                    ← Volver
                </button>
            </div>

            {!esNuevo && (
                <div className="tabs">
                    <Tab actual={pestana} valor="datos" onClick={setPestana}>Datos del Socio</Tab>
                    <Tab actual={pestana} valor="acciones" onClick={setPestana}>Historial de Acciones</Tab>
                    <Tab actual={pestana} valor="creditos" onClick={setPestana}>Historial de Créditos</Tab>
                    <Tab actual={pestana} valor="avala" onClick={setPestana}>Créditos que avala</Tab>
                </div>
            )}

            {error && <div className="error-message mb-4">{error}</div>}
            {success && <div className="success-message mb-4">{success}</div>}

            {pestana === 'datos' && (
                <form onSubmit={guardar}>
                    <div className="detalle-card">
                        <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                            <Campo label="Nombres" value={form.nombres} onChange={set('nombres')} required />
                            <Campo label="Apellidos" value={form.apellidos} onChange={set('apellidos')} required />
                            <Campo
                                label="Cédula" value={form.cedula} onChange={set('cedula')}
                                tipo="numero" required error={errores.cedula}
                            />

                            {/* Estatus y acciones activas no se editan aquí: los gobiernan
                                Liquidación y Acciones, no el formulario del socio. */}
                            <div className="campo-lectura">
                                <label className="form-label">Estatus</label>
                                <div>
                                    {socio
                                        ? <BadgeEstatus estatus={socio.estatus} />
                                        : <span className="badge-estatus badge-activo">Activo</span>}
                                </div>
                            </div>
                        </div>

                        <div className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                            <Campo label="Profesión u oficio" value={form.profesion} onChange={set('profesion')} />
                            <Campo label="Dirección" value={form.direccion} onChange={set('direccion')} />
                            <Campo
                                label="Teléfono" value={form.telefono} onChange={set('telefono')}
                                tipo="numero" error={errores.telefono}
                            />
                            <Campo
                                label="Celular" value={form.celular} onChange={set('celular')}
                                tipo="numero" error={errores.celular}
                            />
                            <Campo
                                label="Correo electrónico" value={form.correo} onChange={set('correo')}
                                tipo="email" error={errores.correo}
                            />
                        </div>

                        {!esNuevo && (
                            <div className="campo-lectura mt-4">
                                <label className="form-label">Acciones activas</label>
                                <div style={{ fontSize: '1.25rem', fontWeight: 600 }}>{acciones}</div>
                            </div>
                        )}
                    </div>

                    {/* RF-20 */}
                    <div className="detalle-card">
                        <h3 className="section-title">BENEFICIARIO EN CASO DE MUERTE (opcional)</h3>
                        <p className="text-sm text-muted" style={{ marginTop: '-0.5rem', marginBottom: '1rem' }}>
                            Persona a la que se le ceden las acciones del socio.
                        </p>
                        <div className="config-grid" style={{ gridTemplateColumns: 'repeat(3, 1fr)' }}>
                            <Campo
                                label="Nombres y apellidos"
                                value={form.beneficiario?.nombre ?? ''}
                                onChange={v => setForm(f => ({
                                    ...f,
                                    beneficiario: { parentesco: null, cedula: '', ...f.beneficiario, nombre: v },
                                }))}
                            />
                            <Campo
                                label="Cédula"
                                tipo="numero"
                                error={errores.ben_cedula}
                                value={form.beneficiario?.cedula ?? ''}
                                onChange={v => setForm(f => ({
                                    ...f,
                                    beneficiario: { parentesco: null, nombre: '', ...f.beneficiario, cedula: v },
                                }))}
                            />
                            <Campo
                                label="Parentesco"
                                value={form.beneficiario?.parentesco ?? ''}
                                onChange={v => setForm(f => ({
                                    ...f,
                                    beneficiario: { nombre: '', cedula: '', ...f.beneficiario, parentesco: v },
                                }))}
                            />
                        </div>
                    </div>

                    {/* RF-21 */}
                    <div className="detalle-card">
                        <h3 className="section-title">PROTEGIDOS (hasta 2, opcional)</h3>
                        {[0, 1].map(i => {
                            const p = form.protegidos[i] ?? PROTEGIDO_VACIO;
                            const setP = (campo: keyof Protegido) => (v: string) =>
                                setForm(f => {
                                    const lista = [0, 1].map(j => ({ ...(f.protegidos[j] ?? PROTEGIDO_VACIO) }));
                                    lista[i] = { ...lista[i], [campo]: v };
                                    // Las filas que quedan en blanco no se envían.
                                    return { ...f, protegidos: lista.filter(x => x.nombre.trim() || x.cedula.trim()) };
                                });
                            return (
                                <div key={i} className="config-grid mt-4" style={{ gridTemplateColumns: 'repeat(4, 1fr)' }}>
                                    <Campo label={`${i + 1}. Nombres y apellidos`} value={p.nombre} onChange={setP('nombre')} />
                                    <Campo
                                        label="Cédula" value={p.cedula} onChange={setP('cedula')}
                                        tipo="numero" error={errores[`prot${i}_cedula`]}
                                    />
                                    <Campo label="Parentesco" value={p.parentesco} onChange={setP('parentesco')} />
                                    <Campo
                                        label="Teléfono" value={p.telefono} onChange={setP('telefono')}
                                        tipo="numero" error={errores[`prot${i}_telefono`]}
                                    />
                                </div>
                            );
                        })}
                    </div>

                    <button type="submit" className="btn btn-primary" disabled={guardando}>
                        {guardando ? 'Guardando…' : esNuevo ? 'Registrar socio' : 'Guardar cambios'}
                    </button>
                </form>
            )}

            {pestana === 'acciones' && (
                <>
                    <table className="activity-table">
                        <thead>
                            <tr>
                                <th>Mes</th>
                                <th style={{ textAlign: 'right' }}>Acciones compradas</th>
                                <th style={{ textAlign: 'right' }}>Acciones liquidadas</th>
                                <th style={{ textAlign: 'right' }}>Valor pagado</th>
                                <th style={{ textAlign: 'right' }}>Saldo</th>
                            </tr>
                        </thead>
                        <tbody>
                            {historial.map(m => (
                                <tr key={m.mes}>
                                    <td style={{ fontWeight: 500 }}>{nombreMes(m.mes)}</td>
                                    <td style={{ textAlign: 'right' }}>{m.compradas}</td>
                                    <td style={{ textAlign: 'right' }}>{m.liquidadas}</td>
                                    <td style={{ textAlign: 'right' }}>{fmt(m.monto_pagado, moneda)}</td>
                                    <td style={{ textAlign: 'right', fontWeight: 500 }}>{m.saldo}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {historial.length === 0 && (
                        <div className="empty-state">
                            Este socio todavía no ha comprado acciones.
                        </div>
                    )}
                </>
            )}

            {pestana === 'creditos' && (
                <TablaPendiente
                    columnas={['N° Crédito', 'Monto', 'Saldo pendiente', 'Estatus']}
                    alineadas={[false, true, true, false]}
                    modulo="Créditos"
                    requisitos="RF-43 a RF-62"
                />
            )}

            {pestana === 'avala' && (
                <TablaPendiente
                    columnas={['N° Crédito', 'A nombre de', 'Monto', 'Estatus']}
                    alineadas={[false, false, true, false]}
                    modulo="Créditos"
                    requisitos="RF-48, RF-101"
                />
            )}
        </div>
    );
}

/**
 * Cabecera de tabla del mockup con un aviso explícito de que el módulo que la alimenta
 * todavía no existe.
 *
 * A propósito NO se inventan filas de ejemplo: un historial de acciones o de créditos
 * con datos falsos se ve idéntico a uno real, y sobre esos números se toman decisiones
 * de plata en la reunión de socios.
 */
function TablaPendiente({ columnas, alineadas, modulo, requisitos }: {
    columnas: string[];
    alineadas: boolean[];
    modulo: string;
    requisitos: string;
}) {
    return (
        <>
            <table className="activity-table">
                <thead>
                    <tr>
                        {columnas.map((c, i) => (
                            <th key={c} style={{ textAlign: alineadas[i] ? 'right' : 'left' }}>{c}</th>
                        ))}
                    </tr>
                </thead>
            </table>
            <div className="pendiente-nota mt-4">
                Esta información la produce el módulo de <strong>{modulo}</strong> ({requisitos}),
                que todavía no está construido.
            </div>
        </>
    );
}

function Tab({ actual, valor, onClick, children }: {
    actual: Pestana;
    valor: Pestana;
    onClick: (p: Pestana) => void;
    children: React.ReactNode;
}) {
    return (
        <button
            type="button"
            className={`tab ${actual === valor ? 'active' : ''}`}
            onClick={() => onClick(valor)}
        >
            {children}
        </button>
    );
}

