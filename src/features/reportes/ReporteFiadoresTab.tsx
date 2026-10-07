import { useEffect, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteFiadorFila } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';

export function ReporteFiadoresTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [filas, setFilas] = useState<ReporteFiadorFila[]>([]);

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setFilas(await api.reporteFiadores());
                setError('');
            } catch (err: any) {
                setError(err.toString());
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    const exportar = () => {
        const cabeceras = ['Cédula Fiador', 'Nombre Fiador', 'Crédito Nro', 'Cédula Deudor', 'Nombre Deudor', 'Acciones Comprometidas (Garantía)'];
        const rows = filas.map(f => [
            f.fiador_cedula, f.fiador_nombres, f.credito_numero, f.deudor_cedula, f.deudor_nombres, f.acciones_comprometidas,
        ]);
        exportarCSV('Reporte_Fiadores', cabeceras, rows);
    };

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Fiadores de créditos vigentes (foto actual, sin dimensión de fecha)."
                desde="" hasta="" setDesde={() => { }} setHasta={() => { }}
                onExportar={exportar} conFechas={false} exportarActivo={filas.length > 0}
            />
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Fiador</th>
                            <th>Deudor a quien avala</th>
                            <th>Crédito Nro</th>
                            <th style={{ textAlign: 'right' }}>Acciones Comprometidas</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filas.map((f, i) => (
                            <tr key={i}>
                                <td>{f.fiador_nombres} ({f.fiador_cedula})</td>
                                <td>{f.deudor_nombres} ({f.deudor_cedula})</td>
                                <td>C-{f.credito_numero}</td>
                                <td style={{ textAlign: 'right', fontWeight: 600 }}>{f.acciones_comprometidas} acciones</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            {filas.length === 0 && (
                <div className="empty-state">No hay fiadores de créditos vigentes registrados.</div>
            )}
        </div>
    );
}
