import { useEffect, useMemo, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteAccionFila } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';
import { enRango } from './rango';

const money = (v: number) =>
    new Intl.NumberFormat('es-CO', { style: 'currency', currency: 'COP', maximumFractionDigits: 0 }).format(v);

export function ReporteAccionesTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [filas, setFilas] = useState<ReporteAccionFila[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setFilas(await api.reporteAcciones());
                setError('');
            } catch (err: any) {
                setError(err.toString());
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const filtradas = useMemo(
        () => filas.filter(f => enRango(f.fecha_compra, desde, hasta)),
        [filas, desde, hasta],
    );

    const exportar = () => {
        const cabeceras = ['Cédula Socio', 'Nombre Socio', 'Acciones', 'Valor de Compra Unitario', 'Valor Lote Inicial', 'Mes Compra (Cohorte)', 'Fecha Efectiva'];
        const rows = filtradas.map(f => [
            f.socio_cedula, f.socio_nombres, f.cantidad_acciones, f.valor_compra, f.valor_compra * f.cantidad_acciones, f.mes_compra, f.fecha_compra,
        ]);
        exportarCSV('Reporte_Acciones', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Control de lotes de acciones activos (por fecha de compra)."
                desde={desde} hasta={hasta} setDesde={setDesde} setHasta={setHasta}
                onExportar={exportar} exportarActivo={filtradas.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Socio</th>
                            <th>Mes Compra (Cohorte)</th>
                            <th>Fecha Compra</th>
                            <th style={{ textAlign: 'right' }}>Cant. Acciones</th>
                            <th style={{ textAlign: 'right' }}>Valor Lote</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtradas.map((f, i) => (
                            <tr key={i}>
                                <td>{f.socio_nombres} ({f.socio_cedula})</td>
                                <td style={{ fontWeight: 600 }}>{f.mes_compra}</td>
                                <td>{f.fecha_compra}</td>
                                <td style={{ textAlign: 'right' }}>{f.cantidad_acciones}</td>
                                <td style={{ textAlign: 'right' }}>{money(f.valor_compra * f.cantidad_acciones)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filtradas.length === 0 && (
                <div className="empty-state">
                    {filas.length === 0 ? 'No hay acciones registradas en circulación.' : 'No hay compras de acciones en el rango seleccionado.'}
                </div>
            )}
        </div>
    );
}
