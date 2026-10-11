import { useEffect, useState } from 'react';
import { save, open, confirm } from '@tauri-apps/plugin-dialog';
import { api } from '../../core/lib/api';
import { mensajeDeError } from '../../core/lib/errores';
import { RegistroRespaldo } from './types';
import { Feedback } from '../../core/components/Feedback';
import { PageHeader } from '../../core/components/PageHeader';

export function RespaldoPage() {
    const [cargando, setCargando] = useState(false);
    const [error, setError] = useState('');
    const [exito, setExito] = useState('');
    const [historial, setHistorial] = useState<RegistroRespaldo[]>([]);
    const [archivoRestaurar, setArchivoRestaurar] = useState('');
    const usuario = 'Admin';

    /** Solo el nombre del archivo, para mostrarlo en el campo (la ruta completa va debajo). */
    const nombreArchivo = (ruta: string) => ruta.split(/[\\/]/).pop() || ruta;

    const cargarHistorial = async () => {
        try {
            setHistorial(await api.listarRespaldos());
        } catch (_) {
            // silencioso si no hay datos aún
        }
    };

    useEffect(() => {
        cargarHistorial();
    }, []);

    const handleCrearRespaldo = async () => {
        try {
            setError('');
            setExito('');

            const ruta = await save({
                title: 'Guardar Respaldo del Bankomunal',
                filters: [{ name: 'Base de Datos de Bankomunales', extensions: ['db', 'bk'] }],
                defaultPath: 'bankomunal_respaldo.db',
            });

            if (!ruta) return;

            setCargando(true);
            await api.crearRespaldo(ruta, usuario);
            setExito('Respaldo creado de forma exitosa en la ubicación seleccionada.');
            await cargarHistorial();
        } catch (err) {
            setError('Error al crear el respaldo: ' + mensajeDeError(err));
        } finally {
            setCargando(false);
        }
    };

    /** Paso 1: elegir el archivo de respaldo (diálogo nativo). Solo lo selecciona; no restaura aún. */
    const seleccionarArchivo = async () => {
        try {
            setError('');
            setExito('');
            const seleccion = await open({
                title: 'Seleccionar archivo de respaldo',
                filters: [{ name: 'Base de Datos de Bankomunales', extensions: ['db', 'bk'] }],
                multiple: false,
                directory: false,
            });
            if (!seleccion) return;
            setArchivoRestaurar(seleccion as string);
        } catch (err) {
            setError('Error al seleccionar el archivo: ' + mensajeDeError(err));
        }
    };

    /** Paso 2: confirmar y cargar el archivo ya seleccionado sobre la base actual. */
    const handleRestaurar = async () => {
        if (!archivoRestaurar) return;
        try {
            setError('');
            setExito('');

            const confirmado = await confirm(
                'Restaurar este respaldo reemplazará TODA la información actual del Bankomunal ' +
                '(socios, créditos, acciones y contabilidad) con los datos del archivo seleccionado. ' +
                'Esta acción no se puede deshacer.',
                { title: '¿Sobrescribir los datos actuales?', kind: 'warning', okLabel: 'Sí, cargar y restaurar', cancelLabel: 'Cancelar' },
            );
            if (!confirmado) return;

            setCargando(true);
            await api.restaurarRespaldo(archivoRestaurar, usuario);
            setExito('Respaldo restaurado con éxito. Reiniciando la aplicación para refrescar los datos...');

            setTimeout(() => {
                window.location.reload();
            }, 3000);
        } catch (err) {
            setError('Error al restaurar: ' + mensajeDeError(err));
        } finally {
            setCargando(false);
        }
    };

    return (
        <div className="page">
            <PageHeader
                titulo="Respaldo y Restauración"
                subtitulo="Protege y recupera la información de tu Bankomunal"
            />

            {(error || exito) && (
                <div className="mb-4">
                    <Feedback error={error} success={exito} />
                </div>
            )}

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '1.5rem' }}>
                {/* ── Tarjeta: Generar respaldo ── */}
                <div className="detalle-card" style={{ display: 'flex', flexDirection: 'column', textAlign: 'center', alignItems: 'center', marginBottom: 0 }}>
                    <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: 'center' }}>
                        <IconoSubir />
                        <h2 className="section-title" style={{ border: 'none', marginTop: '0.75rem', marginBottom: '0.5rem' }}>Generar respaldo</h2>
                        <p className="text-sm text-muted" style={{ marginBottom: 0, maxWidth: 360 }}>
                            Crea una copia de seguridad con toda la información actual del Bankomunal
                            (socios, créditos, acciones y contabilidad). Guárdala en una memoria USB o
                            en tu computador.
                        </p>
                    </div>
                    <button
                        className="btn btn-primary"
                        onClick={handleCrearRespaldo}
                        disabled={cargando}
                        style={{ width: '100%', marginTop: '1.5rem', padding: '0.75rem 1rem' }}
                    >
                        {cargando ? 'Guardando…' : 'Generar respaldo'}
                    </button>
                </div>

                {/* ── Tarjeta: Restaurar respaldo ── */}
                <div className="detalle-card" style={{ display: 'flex', flexDirection: 'column', textAlign: 'center', alignItems: 'center', marginBottom: 0, border: '1px solid var(--error-color)' }}>
                    <IconoBajar />
                    <h2 className="section-title" style={{ border: 'none', marginTop: '0.75rem', marginBottom: '0.5rem', color: 'var(--error-color)' }}>Restaurar respaldo</h2>
                    <p className="text-sm text-muted" style={{ marginBottom: '1.25rem', maxWidth: 360 }}>
                        Carga un archivo de respaldo previamente guardado (.db / .bk).{' '}
                        <strong style={{ color: 'var(--error-color)' }}>¡Cuidado!</strong> Esta acción borrará
                        los datos actuales y los reemplazará con los del archivo seleccionado.
                    </p>

                    {/* Campo para cargar el archivo (selección con el diálogo del sistema) */}
                    <div style={{ width: '100%', textAlign: 'left', marginBottom: '1.25rem' }}>
                        <label className="form-label">Archivo de respaldo</label>
                        <div style={{ display: 'flex', gap: '0.5rem' }}>
                            <input
                                className="form-input"
                                readOnly
                                value={archivoRestaurar ? nombreArchivo(archivoRestaurar) : ''}
                                placeholder="Ningún archivo seleccionado"
                                title={archivoRestaurar}
                                style={{ flex: 1 }}
                            />
                            <button className="btn btn-secondary" onClick={seleccionarArchivo} disabled={cargando}>
                                Seleccionar…
                            </button>
                        </div>
                        {archivoRestaurar && (
                            <div className="text-xs text-muted mt-1" style={{ wordBreak: 'break-all' }}>{archivoRestaurar}</div>
                        )}
                    </div>

                    <button
                        className="btn"
                        onClick={handleRestaurar}
                        disabled={cargando || !archivoRestaurar}
                        style={{ width: '100%', marginTop: 'auto', padding: '0.75rem 1rem', backgroundColor: 'var(--error-color)', color: 'white' }}
                    >
                        {cargando ? 'Restaurando…' : 'Cargar y restaurar'}
                    </button>
                </div>
            </div>

            {/* ── Historial de respaldos ── */}
            <div className="detalle-card" style={{ marginTop: '1.5rem' }}>
                <h3 className="section-title">Historial de respaldos</h3>
                {historial.length === 0 ? (
                    <div className="empty-state">No se han realizado respaldos aún.</div>
                ) : (
                    <div style={{ overflowX: 'auto' }}>
                        <table className="activity-table">
                            <thead>
                                <tr>
                                    <th>Fecha</th>
                                    <th>Tipo</th>
                                    <th>Usuario</th>
                                    <th>Ubicación del archivo</th>
                                </tr>
                            </thead>
                            <tbody>
                                {historial.map(r => (
                                    <tr key={r.id}>
                                        <td>{r.fecha}</td>
                                        <td>
                                            <span style={{ color: r.tipo === 'RESTAURACION' ? 'var(--error-color)' : 'var(--primary-color)', fontWeight: 600 }}>
                                                {r.tipo === 'MANUAL' ? 'Respaldo' : 'Restauración'}
                                            </span>
                                        </td>
                                        <td>{r.usuario}</td>
                                        <td style={{ fontSize: '0.8rem', color: 'var(--text-muted)', wordBreak: 'break-all' }}>{r.ruta_archivo}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </div>
    );
}

/** Nube con flecha hacia arriba (generar respaldo). */
function IconoSubir() {
    return (
        <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="var(--primary-color)" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
            <path d="M12 16V9" />
            <path d="m9 12 3-3 3 3" />
            <path d="M20 17.6A4 4 0 0 0 18 10h-1.3A7 7 0 1 0 5 16.3" />
        </svg>
    );
}

/** Nube con flecha hacia abajo (restaurar respaldo). */
function IconoBajar() {
    return (
        <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="var(--error-color)" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
            <path d="M12 9v7" />
            <path d="m9 13 3 3 3-3" />
            <path d="M20 17.6A4 4 0 0 0 18 10h-1.3A7 7 0 1 0 5 16.3" />
        </svg>
    );
}
