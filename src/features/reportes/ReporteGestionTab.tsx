import { useEffect, useMemo, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteGestionFila } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';
import { enRango } from './rango';
import { mensajeDeError } from '../../core/lib/errores';

const money = (v: number) =>
    new Intl.NumberFormat('es-CO', { style: 'currency', currency: 'COP', maximumFractionDigits: 0 }).format(v);

export function ReporteGestionTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [filas, setFilas] = useState<ReporteGestionFila[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setFilas(await api.reporteGestion());
                setError('');
            } catch (err) {
                setError(mensajeDeError(err));
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const filtradas = useMemo(
        () => filas.filter(f => enRango(f.fecha_cierre, desde, hasta)),
        [filas, desde, hasta],
    );

    const exportar = () => {
        const cabeceras = [
            'Mes', 'Fecha de Cierre', 'Activo Total', 'Pasivo Total',
            'Ingresos del Mes', 'Egresos del Mes', 'Balance Neto', 'Valor por Acción',
        ];
        const rows = filtradas.map(f => [
            f.mes, f.fecha_cierre, f.activo_total, f.pasivo_total,
            f.ingresos_del_mes, f.egresos_del_mes, f.balance_neto, f.valor_por_accion,
        ]);
        exportarCSV('Reporte_de_Gestion', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Histórico mensual de cierres (por fecha de cierre)."
                desde={desde} hasta={hasta} setDesde={setDesde} setHasta={setHasta}
                onExportar={exportar} exportarActivo={filtradas.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Mes</th>
                            <th style={{ textAlign: 'right' }}>Activo Total</th>
                            <th style={{ textAlign: 'right' }}>Pasivo Total</th>
                            <th style={{ textAlign: 'right' }}>Ingresos Mes</th>
                            <th style={{ textAlign: 'right' }}>Egresos Mes</th>
                            <th style={{ textAlign: 'right' }}>Balance Neto</th>
                            <th style={{ textAlign: 'right' }}>Valor por Acción</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtradas.map(fila => (
                            <tr key={fila.mes}>
                                <td>{fila.mes}</td>
                                <td style={{ textAlign: 'right' }}>{money(fila.activo_total)}</td>
                                <td style={{ textAlign: 'right' }}>{money(fila.pasivo_total)}</td>
                                <td style={{ textAlign: 'right' }}>{money(fila.ingresos_del_mes)}</td>
                                <td style={{ textAlign: 'right' }}>{money(fila.egresos_del_mes)}</td>
                                <td style={{ textAlign: 'right', fontWeight: 600 }}>{money(fila.balance_neto)}</td>
                                <td style={{ textAlign: 'right', fontWeight: 600, color: 'var(--primary-color)' }}>{money(fila.valor_por_accion)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filtradas.length === 0 && (
                <div className="empty-state">
                    {filas.length === 0 ? 'No hay cierres de mes registrados.' : 'No hay cierres en el rango de fechas seleccionado.'}
                </div>
            )}
        </div>
    );
}
