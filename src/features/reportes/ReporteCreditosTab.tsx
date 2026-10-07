import { useEffect, useMemo, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteCreditoFila } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';
import { enRango } from './rango';

const money = (v: number) =>
    new Intl.NumberFormat('es-CO', { style: 'currency', currency: 'COP', maximumFractionDigits: 0 }).format(v);

export function ReporteCreditosTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [filas, setFilas] = useState<ReporteCreditoFila[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setFilas(await api.reporteCreditos());
                setError('');
            } catch (err: any) {
                setError(err.toString());
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const filtradas = useMemo(
        () => filas.filter(f => enRango(f.fecha_desembolso, desde, hasta)),
        [filas, desde, hasta],
    );

    const exportar = () => {
        const cabeceras = ['Crédito Nro', 'Fecha Desembolso', 'Cédula Socio', 'Nombre Socio', 'Cuotas', 'Monto Original', 'Saldo Pendiente'];
        const rows = filtradas.map(f => [
            f.numero, f.fecha_desembolso, f.socio_cedula, f.socio_nombres, f.plazo_cuotas, f.monto_original, f.saldo_pendiente,
        ]);
        exportarCSV('Reporte_Saldos_Creditos', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Saldos y pendientes de los créditos vigentes (por fecha de desembolso)."
                desde={desde} hasta={hasta} setDesde={setDesde} setHasta={setHasta}
                onExportar={exportar} exportarActivo={filtradas.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Crédito Nro</th>
                            <th>Socio</th>
                            <th>Fecha Desemb.</th>
                            <th style={{ textAlign: 'right' }}>Cuotas</th>
                            <th style={{ textAlign: 'right' }}>Monto Original</th>
                            <th style={{ textAlign: 'right' }}>Saldo Pendiente</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filtradas.map((f, i) => (
                            <tr key={i}>
                                <td style={{ fontWeight: 600 }}>C-{f.numero}</td>
                                <td>{f.socio_nombres} ({f.socio_cedula})</td>
                                <td>{f.fecha_desembolso}</td>
                                <td style={{ textAlign: 'right' }}>{f.plazo_cuotas}</td>
                                <td style={{ textAlign: 'right' }}>{money(f.monto_original)}</td>
                                <td style={{ textAlign: 'right', fontWeight: 600, color: 'var(--primary-color)' }}>{money(f.saldo_pendiente)}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filtradas.length === 0 && (
                <div className="empty-state">
                    {filas.length === 0 ? 'No hay saldos de créditos vigentes.' : 'No hay créditos desembolsados en el rango seleccionado.'}
                </div>
            )}
        </div>
    );
}
