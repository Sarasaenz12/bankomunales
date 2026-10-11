import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { authService } from './authService';
import { Bankomunal } from '../../core/types';
import { MONEDAS } from '../../core/lib/monedas';
import { mensajeDeError } from '../../core/lib/errores';
import { open } from '@tauri-apps/plugin-dialog';
import { ImportarRespaldo } from '../respaldo/ImportarRespaldo';
import { EXTENSION_RESPALDO } from '../respaldo/types';

export function AuthPage() {
    const [password, setPassword] = useState('');
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [banks, setBanks] = useState<Bankomunal[]>([]);
    const [error, setError] = useState('');
    const [showCreate, setShowCreate] = useState(false);
    const [rutaImportar, setRutaImportar] = useState<string | null>(null);

    // Bank creation form
    const [newBank, setNewBank] = useState({ nombre: '', ubicacion: '', moneda: 'COP' });

    const navigate = useNavigate();

    useEffect(() => {
        // Si ya hay un banco activo, entrar directo. Si no, pero la sesión YA inició
        // sesión (p. ej. tras "Cambiar de banco"), saltar la contraseña y mostrar la
        // selección: la contraseña se pide una sola vez por sesión (RF-01/RF-02).
        authService.bancoSeleccionado().then(bank => {
            if (bank) { navigate('/app/dashboard'); return; }
            authService.sesionIniciada().then(ok => {
                if (ok) { setIsAuthenticated(true); loadBanks(); }
            }).catch(() => { });
        }).catch(() => { });
    }, [navigate]);

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        try {
            const success = await authService.login(password);
            if (success) {
                setIsAuthenticated(true);
                loadBanks();
            } else {
                setError('Contraseña incorrecta');
            }
        } catch (err) {
            setError(mensajeDeError(err, 'Error al iniciar sesión'));
        }
    };

    const loadBanks = async () => {
        try {
            // RF-06: con un único Bankomunal se entra directo (el backend es la fuente de verdad).
            const unico = await authService.unicoBankomunal();
            if (unico) {
                await handleSelectBank(unico.id);
                return;
            }
            const bankList = await authService.listarBankomunales();
            if (bankList.length === 0) {
                setShowCreate(true); // RF-03: sin bancos, arranca el flujo de creación.
            } else {
                setBanks(bankList); // RF-05: dos o más → pantalla de selección.
            }
        } catch (err) {
            setError(mensajeDeError(err, 'Error al cargar los Bankomunales'));
        }
    };

    const handleSelectBank = async (id: string) => {
        try {
            await authService.seleccionarBankomunal(id);
            navigate('/app/dashboard');
        } catch (err) {
            setError(mensajeDeError(err, 'Error al seleccionar banco'));
        }
    };

    /** Elegir un archivo de respaldo para dejar su Bankomunal en este equipo. */
    const elegirRespaldo = async () => {
        try {
            const ruta = await open({
                title: 'Seleccionar respaldo de un Bankomunal',
                filters: [{ name: 'Respaldo de Bankomunal', extensions: [EXTENSION_RESPALDO, 'db', 'bk'] }],
                multiple: false, directory: false,
            });
            if (ruta) setRutaImportar(ruta as string);
        } catch (err) {
            setError(mensajeDeError(err, 'No se pudo abrir el archivo'));
        }
    };

    const handleCreateBank = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        // El backend normaliza y valida de forma autoritativa (trim, moneda por defecto,
        // unicidad). Aquí sólo un chequeo amable para no enviar un nombre en blanco.
        if (!newBank.nombre.trim()) return setError('El nombre es obligatorio');

        try {
            const bank = await authService.crearBankomunal({
                nombre: newBank.nombre.trim(),
                ubicacion: newBank.ubicacion.trim(),
                moneda: newBank.moneda.trim() || 'COP',
            });
            await authService.seleccionarBankomunal(bank.id);
            navigate('/app/dashboard');
        } catch (err) {
            setError(mensajeDeError(err, 'Error al crear el Bankomunal'));
        }
    };

    if (!isAuthenticated) {
        return (
            <div className="auth-container">
                <div className="auth-card">
                    <img className="auth-logo" src="/bankomunales.svg" alt="" />
                    <h1 className="auth-title">Bankomunales</h1>
                    <p className="auth-sub">Administración de Bancos Comunitarios</p>
                    <form onSubmit={handleLogin}>
                        <div className="form-group">
                            <label className="form-label">Contraseña</label>
                            <input
                                type="password"
                                className="form-input"
                                value={password}
                                onChange={e => setPassword(e.target.value)}
                                placeholder="Ingrese la contraseña"
                                autoFocus
                            />
                        </div>
                        {error && <div className="error-message mb-4">{error}</div>}
                        <button type="submit" className="btn btn-primary">Ingresar</button>
                    </form>
                </div>
            </div>
        );
    }

    if (showCreate) {
        return (
            <div className="auth-container">
                <div className="auth-card">
                    <img className="auth-logo" src="/bankomunales.svg" alt="" />
                    <h1 className="auth-title">Nuevo Bankomunal</h1>
                    <p className="auth-sub">Registra un Bankomunal en este equipo</p>

                    <form onSubmit={handleCreateBank}>
                        <div className="form-group">
                            <label className="form-label">Nombre del Bankomunal</label>
                            <input type="text" className="form-input" value={newBank.nombre} onChange={e => setNewBank({ ...newBank, nombre: e.target.value })} placeholder="Ej. Banco Comunal La Esperanza" />
                        </div>
                        <div className="form-group">
                            <label className="form-label">Ubicación</label>
                            <input type="text" className="form-input" value={newBank.ubicacion} onChange={e => setNewBank({ ...newBank, ubicacion: e.target.value })} placeholder="Ciudad, País" />
                        </div>
                        <div className="form-group">
                            <label className="form-label">Moneda</label>
                            <select className="form-input" value={newBank.moneda} onChange={e => setNewBank({ ...newBank, moneda: e.target.value })}>
                                {MONEDAS.map(m => (
                                    <option key={m.codigo} value={m.codigo}>{m.etiqueta}</option>
                                ))}
                            </select>
                        </div>

                        {error && <div className="error-message mb-4">{error}</div>}
                        <div className="form-group" style={{ display: 'flex', gap: '1rem', marginBottom: 0 }}>
                            {banks.length > 0 && <button type="button" className="btn btn-secondary" onClick={() => setShowCreate(false)}>Cancelar</button>}
                            <button type="submit" className="btn btn-primary" style={{ flex: 1 }}>Guardar e ingresar</button>
                        </div>
                    </form>
                    <button type="button" className="create-bank-btn" style={{ marginTop: '1rem' }} onClick={elegirRespaldo}>
                        ¿Tiene un respaldo? Importar respaldo…
                    </button>
                </div>
                {rutaImportar && <ImportarRespaldo ruta={rutaImportar} onCerrar={() => setRutaImportar(null)} />}
            </div>
        );
    }

    return (
        <div className="auth-container">
            <div className="select-wrap">
                <img className="auth-logo" src="/bankomunales.svg" alt="" />
                <h1 className="auth-title">Bankomunales</h1>
                <p className="auth-sub">Seleccione un Bankomunal para trabajar</p>

                {error && <div className="error-message mb-4">{error}</div>}

                <div className="bank-grid">
                    {banks.map((b, i) => (
                        <div key={b.id} className="bank-card" onClick={() => handleSelectBank(b.id)}>
                            <div className="bank-card-name">{b.nombre}</div>
                            <div className="bank-card-meta">
                                Código: BK-{String(i + 1).padStart(3, '0')}
                                {b.ubicacion ? ` · ${b.ubicacion}` : ''}
                            </div>
                        </div>
                    ))}
                </div>

                <button className="create-bank-btn" onClick={() => setShowCreate(true)}>
                    + Crear otro Bankomunal
                </button>
                <button className="create-bank-btn" style={{ marginLeft: '0.75rem' }} onClick={elegirRespaldo}>
                    Importar respaldo…
                </button>
            </div>
            {rutaImportar && <ImportarRespaldo ruta={rutaImportar} onCerrar={() => setRutaImportar(null)} />}
        </div>
    );
}
