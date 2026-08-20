import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { sociosService } from './sociosService';
import { CupoSocios } from './types';
import { AccionesDeSocio } from '../acciones/types';
import { BadgeEstatus } from '../../core/components/BadgeEstatus';
import { Chip } from '../../core/components/Chip';
import { PageHeader } from '../../core/components/PageHeader';
import { useBanco } from '../../core/context/BankContext';

type Filtro = 'todos' | 'activos' | 'retirados';

export function SociosPage() {
    const navigate = useNavigate();
    const bank = useBanco();
    const [socios, setSocios] = useState<Awaited<ReturnType<typeof sociosService.listar>>>([]);
    const [cupo, setCupo] = useState<CupoSocios | null>(null);
    const [acciones, setAcciones] = useState<AccionesDeSocio[]>([]);
    const [busqueda, setBusqueda] = useState('');
    const [filtro, setFiltro] = useState<Filtro>('todos');
    const [error, setError] = useState('');

    useEffect(() => {
        if (!bank) return;
        sociosService.cargarResumen()
            .then(({ socios, cupo, acciones }) => { setSocios(socios); setCupo(cupo); setAcciones(acciones); })
            .catch(e => setError(e?.toString() ?? 'Error al cargar los socios'));
    }, [bank]);

    const accionesDe = (id: string) => acciones.find(a => a.socio_id === id)?.acciones ?? 0;

    const activos = socios.filter(s => s.estatus === 'ACTIVO');
    const retirados = socios.filter(s => s.estatus !== 'ACTIVO');

    const porFiltro =
        filtro === 'activos' ? activos : filtro === 'retirados' ? retirados : socios;

    const q = busqueda.trim().toLowerCase();
    const visibles = porFiltro.filter(
        s => !q || s.nombre_completo.toLowerCase().includes(q) || s.cedula.toLowerCase().includes(q),
    );

    // RN-01: el reglamento no permite pasar de 19 socios activos.
    const sinCupo = cupo?.disponibles === 0;

    return (
        <div className="page">
            <PageHeader titulo="Socios" subtitulo="Listado de socios activos y retirados" />

            <div className="list-toolbar">
                <input
                    className="form-input"
                    placeholder="Buscar por nombre o cédula..."
                    value={busqueda}
                    onChange={e => setBusqueda(e.target.value)}
                />
                <button
                    className="btn btn-primary"
                    onClick={() => navigate('/app/socios/nuevo')}
                    disabled={sinCupo}
                    title={sinCupo ? `Máximo ${cupo?.maximo} socios activos (RN-01)` : undefined}
                >
                    + Nuevo Socio
                </button>
            </div>

            <div className="chip-row">
                <Chip activo={filtro === 'todos'} onClick={() => setFiltro('todos')}>
                    Todos ({socios.length})
                </Chip>
                <Chip activo={filtro === 'activos'} onClick={() => setFiltro('activos')}>
                    Activos ({activos.length})
                </Chip>
                <Chip activo={filtro === 'retirados'} onClick={() => setFiltro('retirados')}>
                    Retirados ({retirados.length})
                </Chip>
            </div>

            {sinCupo && (
                <div className="error-message mb-4">
                    El Bankomunal alcanzó el máximo de {cupo?.maximo} socios activos que permite el
                    reglamento (RN-01). Para registrar uno nuevo, primero debe retirarse otro.
                </div>
            )}
            {error && <div className="error-message mb-4">{error}</div>}

            <table className="activity-table">
                <thead>
                    <tr>
                        <th>Nombre</th>
                        <th>Cédula</th>
                        <th>Estatus</th>
                        <th style={{ textAlign: 'right' }}>Acciones activas</th>
                        <th style={{ width: 80 }}></th>
                    </tr>
                </thead>
                <tbody>
                    {visibles.map(s => (
                        <tr key={s.id}>
                            <td style={{ fontWeight: 500 }}>{s.nombre_completo}</td>
                            <td>{s.cedula}</td>
                            <td><BadgeEstatus estatus={s.estatus} /></td>
                            <td style={{ textAlign: 'right' }}>{accionesDe(s.id)}</td>
                            <td style={{ textAlign: 'right' }}>
                                <button
                                    className="link-ver"
                                    onClick={() => navigate(`/app/socios/${s.id}`)}
                                >
                                    Ver →
                                </button>
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>

            {visibles.length === 0 && (
                <div className="empty-state">
                    {socios.length === 0
                        ? 'Todavía no hay socios registrados.'
                        : 'Ningún socio coincide con la búsqueda.'}
                </div>
            )}
        </div>
    );
}
