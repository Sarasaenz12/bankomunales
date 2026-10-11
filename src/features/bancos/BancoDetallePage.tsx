import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { api } from '../../core/lib/api';
import { Bankomunal } from '../../core/types';
import { Campo } from '../../core/components/Campo';
import { PageHeader } from '../../core/components/PageHeader';
import { MONEDAS } from '../../core/lib/monedas';
import { mensajeDeError } from '../../core/lib/errores';

type BancoForm = {
    nombre: string;
    ubicacion: string;
    moneda: string;
};

const FORM_VACIO: BancoForm = {
    nombre: '',
    ubicacion: '',
    moneda: 'COP',
};

export function BancoDetallePage() {
    const { id } = useParams<{ id: string }>();
    const navigate = useNavigate();
    const esNuevo = !id;

    const [form, setForm] = useState<BancoForm>(FORM_VACIO);
    const [banco, setBanco] = useState<Bankomunal | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const [guardando, setGuardando] = useState(false);
    const [cargando, setCargando] = useState(!esNuevo);

    // Detalle de un banco existente: se obtiene del listado (no hay comando de "obtener
    // uno" ni de "actualizar", así que el detalle es de sólo lectura).
    useEffect(() => {
        if (esNuevo) return;
        api.listarBankomunales()
            .then(lista => {
                const b = lista.find(x => x.id === id) ?? null;
                setBanco(b);
                if (!b) setError('No se encontró el Bankomunal.');
            })
            .catch(e => setError(mensajeDeError(e, 'No se pudo cargar el Bankomunal')))
            .finally(() => setCargando(false));
    }, [id, esNuevo]);

    async function guardar(ev: React.FormEvent) {
        ev.preventDefault();
        setError('');
        setSuccess('');

        const nombre = form.nombre.trim();
        if (!nombre) {
            setError('El nombre del banco es obligatorio.');
            return;
        }

        setGuardando(true);
        try {
            const creado = await api.crearBankomunal({
                nombre,
                ubicacion: form.ubicacion.trim(),
                moneda: form.moneda.trim() || 'COP',
            });
            setSuccess(`Bankomunal "${creado.nombre}" creado correctamente.`);
            setTimeout(() => navigate(`/app/bancos/${creado.id}`, { replace: true }), 1200);
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo crear el Bankomunal.'));
        } finally {
            setGuardando(false);
        }
    }

    async function seleccionar() {
        if (!banco) return;
        try {
            await api.seleccionarBankomunal(banco.id);
            navigate('/app/dashboard');
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo seleccionar el Bankomunal.'));
        }
    }

    const set = (campo: keyof BancoForm) => (valor: string) =>
        setForm(f => ({ ...f, [campo]: valor }));

    if (cargando) return <div className="page">Cargando…</div>;

    // ── Detalle de un Bankomunal existente (solo lectura) ──
    if (!esNuevo) {
        return (
            <div className="page">
                <PageHeader titulo="Detalle del Bankomunal" subtitulo={banco?.nombre ?? '—'} />

                {error && <div className="error-message mb-4">{error}</div>}

                {banco && (
                    <div className="detalle-card" style={{ maxWidth: 640 }}>
                        <div className="grid-dos">
                            <div>Nombre: <strong>{banco.nombre}</strong></div>
                            <div>Ubicación: <strong>{banco.ubicacion || '—'}</strong></div>
                            <div>Moneda: <strong>{banco.moneda}</strong></div>
                            <div>Creado: <strong>{banco.fecha_creacion?.slice(0, 10) || '—'}</strong></div>
                        </div>

                        <div className="chip-row mt-4" style={{ gap: '0.75rem' }}>
                            <button className="btn btn-primary" onClick={seleccionar}>
                                Seleccionar este Bankomunal
                            </button>
                            <button className="btn btn-secondary" onClick={() => navigate('/')}>
                                ← Volver a la selección
                            </button>
                        </div>
                    </div>
                )}
            </div>
        );
    }

    // ── Crear un nuevo Bankomunal ──
    return (
        <div className="page">
            <PageHeader titulo="Nuevo Bankomunal" subtitulo="Registro de un nuevo Bankomunal" />

            {error && <div className="error-message mb-4">{error}</div>}
            {success && <div className="success-message mb-4">{success}</div>}

            <form onSubmit={guardar}>
                <div className="detalle-card" style={{ maxWidth: 640 }}>
                    <div className="config-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)' }}>
                        <Campo
                            label="Nombre del Bankomunal"
                            value={form.nombre}
                            onChange={set('nombre')}
                            required
                        />
                        <Campo
                            label="Ubicación"
                            value={form.ubicacion}
                            onChange={set('ubicacion')}
                        />
                        <div className="form-group">
                            <label className="form-label">Moneda</label>
                            <select
                                className="form-input"
                                value={form.moneda}
                                onChange={e => set('moneda')(e.target.value)}
                            >
                                {MONEDAS.map(m => (
                                    <option key={m.codigo} value={m.codigo}>{m.etiqueta}</option>
                                ))}
                            </select>
                        </div>
                    </div>

                    <div className="chip-row mt-4" style={{ gap: '0.75rem' }}>
                        <button type="submit" className="btn btn-primary" disabled={guardando}>
                            {guardando ? 'Creando…' : 'Crear Bankomunal'}
                        </button>
                        <button
                            type="button"
                            className="btn btn-secondary"
                            onClick={() => navigate('/')}
                            disabled={guardando}
                        >
                            Cancelar
                        </button>
                    </div>
                </div>
            </form>
        </div>
    );
}
