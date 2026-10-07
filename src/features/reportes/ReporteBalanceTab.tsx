import { useEffect, useState } from 'react';
import { api } from '../../core/lib/api';
import { ReporteBalanceActual } from './types';
import { Feedback } from '../../core/components/Feedback';
import { exportarCSV } from '../../core/utils/csv';
import { BarraReporte } from './BarraReporte';

const money = (v: number) =>
    new Intl.NumberFormat('es-CO', { style: 'currency', currency: 'COP', maximumFractionDigits: 0 }).format(v);

/** Tabla de dos columnas (concepto / valor) con fila de total opcional. */
function TablaBalance({ titulo, filas, total }: {
    titulo: string;
    filas: { concepto: string; valor: number }[];
    total: { etiqueta: string; valor: number };
}) {
    return (
        <div style={{ marginBottom: '2rem' }}>
            <h4 className="section-title">{titulo}</h4>
            <div style={{ overflowX: 'auto' }}>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Concepto</th>
                            <th style={{ textAlign: 'right' }}>Valor</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filas.map(f => (
                            <tr key={f.concepto}>
                                <td>{f.concepto}</td>
                                <td style={{ textAlign: 'right' }}>{money(f.valor)}</td>
                            </tr>
                        ))}
                        <tr>
                            <td style={{ fontWeight: 700 }}>{total.etiqueta}</td>
                            <td style={{ textAlign: 'right', fontWeight: 700, color: 'var(--primary-color)' }}>{money(total.valor)}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    );
}

export function ReporteBalanceTab() {
    const [cargando, setCargando] = useState(true);
    const [error, setError] = useState('');
    const [balance, setBalance] = useState<ReporteBalanceActual | null>(null);

    useEffect(() => {
        (async () => {
            try {
                setCargando(true);
                setBalance(await api.reporteBalance());
                setError('');
            } catch (err: any) {
                setError(err.toString());
            } finally {
                setCargando(false);
            }
        })();
    }, []);

    if (cargando) return <div className="detalle-card">Cargando reporte…</div>;
    if (error) return <div className="detalle-card"><Feedback error={error} success="" /></div>;
    if (!balance) return null;

    const activoTotal = balance.efectivo + balance.cartera + balance.bienes;
    const pasivoTotal = balance.fondo_gastos + balance.fondo_incobrables + balance.capital_acciones
        + balance.ganancias_pasivo + balance.resultado_no_distribuido;
    const diferencia = activoTotal - pasivoTotal;

    const activos = [
        { concepto: 'Caja (Efectivo)', valor: balance.efectivo },
        { concepto: 'Cartera Vigente', valor: balance.cartera },
        { concepto: 'Bienes Propios', valor: balance.bienes },
    ];
    const pasivos = [
        { concepto: 'Fondo para Gastos', valor: balance.fondo_gastos },
        { concepto: 'Fondo de Incobrables', valor: balance.fondo_incobrables },
        { concepto: 'Capital en Acciones', valor: balance.capital_acciones },
        { concepto: 'Ganancias por Pagar / Liberadas', valor: balance.ganancias_pasivo },
        { concepto: 'Resultado del mes en curso (sin repartir)', valor: balance.resultado_no_distribuido },
    ];

    const exportar = () => {
        const rows = [
            ...activos.map(a => ['Activo', a.concepto, a.valor]),
            ['Activo', 'Total Activos', activoTotal],
            ...pasivos.map(p => ['Pasivo', p.concepto, p.valor]),
            ['Pasivo', 'Total Pasivos y Patrimonio', pasivoTotal],
            ['Balance', 'Diferencia (Activo − Pasivo)', diferencia],
        ];
        exportarCSV('Reporte_Balance_General', ['Grupo', 'Concepto', 'Valor'], rows);
    };

    return (
        <div className="detalle-card">
            <BarraReporte
                descripcion="Estado de activo vs. pasivo (foto actual del bankomunal)."
                desde="" hasta="" setDesde={() => { }} setHasta={() => { }}
                onExportar={exportar} conFechas={false}
            />

            <TablaBalance titulo="Activos" filas={activos} total={{ etiqueta: 'Total Activos', valor: activoTotal }} />
            <TablaBalance titulo="Pasivos y Patrimonio" filas={pasivos} total={{ etiqueta: 'Total Pasivos y Patrimonio', valor: pasivoTotal }} />

            <div
                className="detalle-card"
                style={{
                    background: diferencia === 0 ? '#f0fdf4' : '#fef2f2',
                    border: 'none', display: 'flex', justifyContent: 'space-between', alignItems: 'center',
                    fontWeight: 700, marginBottom: 0,
                }}
            >
                <span>Diferencia (Activo − Pasivo)</span>
                <span style={{ color: diferencia === 0 ? '#166534' : 'var(--error-color)' }}>{money(diferencia)}</span>
            </div>
        </div>
    );
}
