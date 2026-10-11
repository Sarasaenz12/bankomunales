import { useEffect, useState } from 'react';
import { api } from '../../core/lib/api';
import { mensajeDeError } from '../../core/lib/errores';
import { Feedback } from '../../core/components/Feedback';
import { VistaRespaldo } from './types';

type Paso = 'clave' | 'revisando' | 'confirmar' | 'importando' | 'listo';

const nombreArchivo = (ruta: string) => ruta.split(/[\\/]/).pop() || ruta;

/**
 * Asistente para dejar un Bankomunal en este computador a partir de un archivo de respaldo
 * (el que se abre con doble clic desde el Explorador, o el que se elige desde la pantalla de
 * ingreso). Pasos: clave (si aún no se ha ingresado) → reconocer el archivo → confirmar →
 * importar → entrar al Bankomunal.
 */
export function ImportarRespaldo({ ruta, onCerrar: alCerrar }: { ruta: string; onCerrar: () => void }) {
    /** El archivo ya se atendió (o se descartó): que no vuelva a ofrecerse al recargar la pantalla. */
    const onCerrar = () => { api.olvidarArchivoPendiente().catch(() => { }); alCerrar(); };
    const [paso, setPaso] = useState<Paso>('revisando');
    const [clave, setClave] = useState('');
    const [vista, setVista] = useState<VistaRespaldo | null>(null);
    const [error, setError] = useState('');

    const revisar = async () => {
        setError('');
        setPaso('revisando');
        try {
            setVista(await api.inspeccionarRespaldo(ruta));
            setPaso('confirmar');
        } catch (e) {
            const msg = mensajeDeError(e, 'No se pudo leer el archivo de respaldo');
            if (/iniciar sesi[oó]n/i.test(msg)) setPaso('clave');
            else { setError(msg); setPaso('confirmar'); }
        }
    };

    useEffect(() => { revisar(); }, [ruta]);

    const ingresar = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        try {
            if (!(await api.login(clave))) return setError('Contraseña incorrecta');
            await revisar();
        } catch (err) {
            setError(mensajeDeError(err, 'No se pudo iniciar sesión'));
        }
    };

    const importar = async () => {
        if (!vista) return;
        setError('');
        setPaso('importando');
        try {
            await api.importarRespaldo(ruta, 'Admin');
            await api.seleccionarBankomunal(vista.id);
            setPaso('listo');
            api.olvidarArchivoPendiente().catch(() => { });
            setTimeout(() => window.location.assign('/app/dashboard'), 1200);
        } catch (err) {
            setError(mensajeDeError(err, 'No se pudo importar el respaldo'));
            setPaso('confirmar');
        }
    };

    return (
        <div style={{
            position: 'fixed', inset: 0, background: 'rgba(15,30,50,0.55)', zIndex: 1000,
            display: 'flex', alignItems: 'center', justifyContent: 'center', padding: '1rem',
        }}>
            <div className="card" style={{ width: '100%', maxWidth: 460, padding: '1.75rem', background: '#fff' }}>
                <h2 style={{ marginTop: 0 }}>Importar Bankomunal</h2>
                <p className="text-sm text-muted" style={{ wordBreak: 'break-all' }}>Archivo: {nombreArchivo(ruta)}</p>

                {error && <Feedback error={error} success="" />}

                {paso === 'clave' && (
                    <form onSubmit={ingresar}>
                        <p>Para continuar, ingrese la contraseña del sistema.</p>
                        <input type="password" className="form-input" autoFocus placeholder="Contraseña"
                            value={clave} onChange={e => setClave(e.target.value)} />
                        <div style={{ display: 'flex', gap: '0.75rem', marginTop: '1rem', justifyContent: 'flex-end' }}>
                            <button type="button" className="btn btn-secondary" onClick={onCerrar}>Cancelar</button>
                            <button type="submit" className="btn btn-primary" disabled={!clave}>Continuar</button>
                        </div>
                    </form>
                )}

                {paso === 'revisando' && <p>Revisando el archivo…</p>}

                {paso === 'confirmar' && vista && (
                    <>
                        <div style={{ background: '#f1f5f9', borderRadius: 8, padding: '0.9rem 1rem', margin: '0.75rem 0' }}>
                            <div style={{ fontWeight: 600, fontSize: '1.05rem' }}>{vista.nombre}</div>
                            <div className="text-sm text-muted">{vista.ubicacion || 'Sin ubicación'} · {vista.moneda}</div>
                        </div>
                        {vista.ya_existe ? (
                            <div className="error-message" style={{ marginBottom: '1rem' }}>
                                Este Bankomunal ya está en este computador. Si continúa, sus datos actuales se
                                reemplazarán por los del archivo (se conserva una copia de lo que había).
                            </div>
                        ) : (
                            <p>Este Bankomunal no está en este computador. Se agregará con toda su información.</p>
                        )}
                        <div style={{ display: 'flex', gap: '0.75rem', justifyContent: 'flex-end' }}>
                            <button className="btn btn-secondary" onClick={onCerrar}>Cancelar</button>
                            <button className="btn btn-primary" onClick={importar}>
                                {vista.ya_existe ? 'Reemplazar e ingresar' : 'Importar e ingresar'}
                            </button>
                        </div>
                    </>
                )}

                {paso === 'confirmar' && !vista && (
                    <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '1rem' }}>
                        <button className="btn btn-secondary" onClick={onCerrar}>Cerrar</button>
                    </div>
                )}

                {paso === 'importando' && <p>Importando… espere un momento, no cierre la aplicación.</p>}

                {paso === 'listo' && vista && (
                    <div className="success-message">«{vista.nombre}» quedó listo. Entrando…</div>
                )}
            </div>
        </div>
    );
}
