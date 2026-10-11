import { useEffect, useMemo, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteSocioActivo } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';
import { enRango } from './rango';
import { mensajeDeError } from '../../core/lib/errores';

export function ReporteSociosActivosTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [socios, setSocios] = useState<ReporteSocioActivo[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setSocios(await api.reporteSociosActivos());
                setError('');
            } catch (err) {
                setError(mensajeDeError(err));
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const filtrados = useMemo(
        () => socios.filter(s => enRango(s.fecha_ingreso, desde, hasta)),
        [socios, desde, hasta],
    );

    const exportar = () => {
        const cabeceras = ['Cédula', 'Nombres', 'Apellidos', 'Fecha Ingreso', 'Certificados Activos', 'Liquidados Parcialmente'];
        const rows = filtrados.map(s => [
            s.cedula, s.nombres, s.apellidos, s.fecha_ingreso, s.certificados_activos, s.liquidados_parcialmente,
        ]);
        exportarCSV('Reporte_Socios_Activos', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Socios actualmente activos en el bankomunal (por fecha de ingreso)."
                desde={desde} hasta={hasta} setDesde={setDesde} setHasta={setHasta}
                onExportar={exportar} exportarActivo={filtrados.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Cédula</th>
                            <th>Socio</th>
                            <th>Fecha Ingreso</th>
                            <th style={{ textAlign: 'right' }}>Certificados Activos</th>
                            <th style={{ textAlign: 'right' }}>Liq. Parcialmente</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtrados.map(socio => (
                            <tr key={socio.id}>
                                <td>{socio.cedula}</td>
                                <td>{socio.apellidos}, {socio.nombres}</td>
                                <td>{socio.fecha_ingreso}</td>
                                <td style={{ textAlign: 'right' }}>{socio.certificados_activos}</td>
                                <td style={{ textAlign: 'right' }}>{socio.liquidados_parcialmente}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filtrados.length === 0 && (
                <div className="empty-state">
                    {socios.length === 0 ? 'No hay socios activos registrados.' : 'Ningún socio ingresó en el rango de fechas seleccionado.'}
                </div>
            )}
        </div>
    );
}
