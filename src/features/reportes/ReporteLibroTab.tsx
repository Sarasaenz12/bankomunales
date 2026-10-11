import { useEffect, useMemo, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteLibroFila } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';
import { enRango } from './rango';
import { mensajeDeError } from '../../core/lib/errores';

const money = (v: number) =>
    new Intl.NumberFormat('es-CO', { style: 'currency', currency: 'COP', maximumFractionDigits: 0 }).format(v);

export function ReporteLibroTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [filas, setFilas] = useState<ReporteLibroFila[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setFilas(await api.reporteLibro());
                setError('');
            } catch (err) {
                setError(mensajeDeError(err));
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const filtradas = useMemo(
        () => filas.filter(f => enRango(f.fecha, desde, hasta)),
        [filas, desde, hasta],
    );

    const exportar = () => {
        const cabeceras = ['Fecha', 'Operación', 'Ingreso', 'Egreso'];
        const rows = filtradas.map(f => [f.fecha, f.operacion, f.ingreso, f.egreso]);
        exportarCSV('Reporte_Libro_Visor', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Libro contable de ingresos y egresos (Caja)."
                desde={desde} hasta={hasta} setDesde={setDesde} setHasta={setHasta}
                onExportar={exportar} exportarActivo={filtradas.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Fecha</th>
                            <th>Detalle Operación</th>
                            <th style={{ textAlign: 'right' }}>Ingresos</th>
                            <th style={{ textAlign: 'right' }}>Egresos</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtradas.map((f, i) => (
                            <tr key={i}>
                                <td>{f.fecha}</td>
                                <td>{f.operacion}</td>
                                <td style={{ textAlign: 'right' }}>
                                    {f.ingreso > 0 ? <span style={{ color: 'var(--success-color)', fontWeight: 600 }}>+{money(f.ingreso)}</span> : '—'}
                                </td>
                                <td style={{ textAlign: 'right' }}>
                                    {f.egreso > 0 ? <span style={{ fontWeight: 600 }}>-{money(f.egreso)}</span> : '—'}
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filtradas.length === 0 && (
                <div className="empty-state">
                    {filas.length === 0 ? 'No hay movimientos en el libro.' : 'No hay movimientos en el rango de fechas seleccionado.'}
                </div>
            )}
        </div>
    );
}
