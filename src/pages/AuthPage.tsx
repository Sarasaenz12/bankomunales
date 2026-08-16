import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../lib/api';
import { Bankomunal } from '../types';
import { Building2, Plus, ArrowRight } from 'lucide-react';

export function AuthPage() {
    const [password, setPassword] = useState('');
    const [isAuthenticated, setIsAuthenticated] = useState(false);
    const [banks, setBanks] = useState<Bankomunal[]>([]);
    const [error, setError] = useState('');
    const [showCreate, setShowCreate] = useState(false);

    // Bank creation form
    const [newBank, setNewBank] = useState({ nombre: '', ubicacion: '', moneda: 'COP' });

    const navigate = useNavigate();

    useEffect(() => {
        // Check if there is already an active bank selected in the backend session
        api.bancoSeleccionado().then(bank => {
            if (bank) navigate('/app/dashboard');
        }).catch(() => { });
    }, [navigate]);

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        try {
            const success = await api.login(password);
            if (success) {
                setIsAuthenticated(true);
                loadBanks();
            } else {
                setError('Contraseña incorrecta');
            }
        } catch (err: any) {
            setError(err?.toString() || 'Error al iniciar sesión');
        }
    };

    const loadBanks = async () => {
        try {
            const bankList = await api.listarBankomunales();
            if (bankList.length === 0) {
                setShowCreate(true);
            } else if (bankList.length === 1) {
                await api.seleccionarBankomunal(bankList[0].id);
                navigate('/app/dashboard');
            } else {
                setBanks(bankList);
            }
        } catch (err: any) {
            setError(err?.toString() || 'Error al cargar los Bankomunales');
        }
    };

    const handleSelectBank = async (id: string) => {
        try {
            await api.seleccionarBankomunal(id);
            navigate('/app/dashboard');
        } catch (err: any) {
            setError(err?.toString() || 'Error al seleccionar banco');
        }
    };

    const handleCreateBank = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        if (!newBank.nombre) return setError('El nombre es obligatorio');

        try {
            const bank = await api.crearBankomunal(newBank);
            await api.seleccionarBankomunal(bank.id);
            navigate('/app/dashboard');
        } catch (err: any) {
            setError(err?.toString() || 'Error al crear el Bankomunal');
        }
    };

    if (!isAuthenticated) {
        return (
            <div className="auth-container">
                <div className="card">
                    <div className="text-center mb-4">
                        <Building2 size={48} color="var(--primary-color)" />
                    </div>
                    <h1 className="card-title">Acceso Bankomunales</h1>
                    <form onSubmit={handleLogin}>
                        <div className="form-group">
                            <label className="form-label">Contraseña de acceso</label>
                            <input
                                type="password"
                                className="form-input"
                                value={password}
                                onChange={e => setPassword(e.target.value)}
                                placeholder="Ingresa la clave genérica"
                                autoFocus
                            />
                        </div>
                        {error && <div className="error-message">{error}</div>}
                        <button type="submit" className="btn btn-primary mt-4">Entrar</button>
                    </form>
                </div>
            </div>
        );
    }

    if (showCreate) {
        return (
            <div className="auth-container">
                <div className="card">
                    <h2 className="card-title">Crear Nuevo Bankomunal</h2>
                    <p className="text-muted text-sm text-center mb-4">Configura el primer Bankomunal en este equipo.</p>

                    <form onSubmit={handleCreateBank}>
                        <div className="form-group">
                            <label className="form-label">Nombre del Bankomunal</label>
                            <input type="text" className="form-input" value={newBank.nombre} onChange={e => setNewBank({ ...newBank, nombre: e.target.value })} placeholder="Ej. Banco Pijao" />
                        </div>
                        <div className="form-group">
                            <label className="form-label">Ubicación</label>
                            <input type="text" className="form-input" value={newBank.ubicacion} onChange={e => setNewBank({ ...newBank, ubicacion: e.target.value })} placeholder="Ciudad, Barrio..." />
                        </div>
                        <div className="form-group">
                            <label className="form-label">Moneda</label>
                            <input type="text" className="form-input" value={newBank.moneda} onChange={e => setNewBank({ ...newBank, moneda: e.target.value })} placeholder="COP" />
                        </div>

                        {error && <div className="error-message">{error}</div>}
                        <div className="form-group mt-4" style={{ display: 'flex', gap: '1rem' }}>
                            {banks.length > 0 && <button type="button" className="btn btn-secondary" onClick={() => setShowCreate(false)}>Cancelar</button>}
                            <button type="submit" className="btn btn-primary">Guardar e Ingresar</button>
                        </div>
                    </form>
                </div>
            </div>
        );
    }

    return (
        <div className="auth-container">
            <div className="card">
                <h2 className="card-title">Seleccionar Bankomunal</h2>

                <div className="bank-list">
                    {banks.map(b => (
                        <div key={b.id} className="bank-item" onClick={() => handleSelectBank(b.id)}>
                            <div>
                                <strong>{b.nombre}</strong>
                                <div className="text-sm text-muted">{b.ubicacion}</div>
                            </div>
                            <ArrowRight size={20} color="var(--primary-color)" />
                        </div>
                    ))}
                </div>

                <button className="btn btn-secondary" onClick={() => setShowCreate(true)}>
                    <Plus size={18} style={{ marginRight: '8px' }} /> Agregar Nuevo Bankomunal
                </button>
                {error && <div className="error-message mt-4">{error}</div>}
            </div>
        </div>
    );
}
