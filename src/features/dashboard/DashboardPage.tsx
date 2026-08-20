import { useEffect, useState } from 'react';
import { Users, TrendingUp, CreditCard, DollarSign } from 'lucide-react';
import { dashboardService } from './dashboardService';
import { DatosGenerales } from '../../core/types';
import { fmt } from '../../core/lib/format';
import { PageHeader } from '../../core/components/PageHeader';
import { useBanco } from '../../core/context/BankContext';

// ── Placeholder recent activity (until we have a real movements table) ──
const RECENT_PLACEHOLDER = [
    { fecha: '—', operacion: 'Sin movimientos aún', socio: '—', monto: null },
];

export function DashboardPage() {
    const bank = useBanco();
    const [datos, setDatos] = useState<DatosGenerales | null>(null);
    const [auditoria, setAuditoria] = useState<any[]>([]);
    const [error, setError] = useState('');

    useEffect(() => {
        if (!bank) return;
        dashboardService.cargarDatos()
            .then(({ datos, auditoria }) => { setDatos(datos); setAuditoria(auditoria); })
            .catch(e => setError(e?.toString() || 'Error'));
    }, [bank]);

    if (error) return <div className="page-error">{error}</div>;

    return (
        <div className="page">
            <PageHeader titulo="Inicio / Dashboard" subtitulo="Resumen general del Bankomunal" />

            {/* ── KPI cards ── */}
            <div className="kpi-grid">
                <KpiCard
                    icon={<Users size={20} />}
                    label="Socios activos"
                    value={datos ? String(datos.numero_acciones_vendidas > 0 ? '—' : '0') : '…'}
                    sub="datos de socios próximamente"
                    color="blue"
                />
                <KpiCard
                    icon={<CreditCard size={20} />}
                    label="Créditos otorgados"
                    value={datos ? String(datos.numero_creditos_otorgados) : '…'}
                    sub={datos ? `Monto total: ${fmt(datos.monto_total_creditos, datos.moneda)}` : ''}
                    color="indigo"
                />
                <KpiCard
                    icon={<TrendingUp size={20} />}
                    label="Acciones vendidas"
                    value={datos ? String(datos.numero_acciones_vendidas) : '…'}
                    sub={datos ? `Valor nominal: ${fmt(datos.valor_nominal, datos.moneda)}` : ''}
                    color="violet"
                />
                <KpiCard
                    icon={<DollarSign size={20} />}
                    label="Fondo de Gastos"
                    value={datos ? fmt(datos.saldo_fondo_gastos, datos.moneda) : '…'}
                    sub={datos ? `Fondo Incobrables: ${fmt(datos.saldo_fondo_incobrables, datos.moneda)}` : ''}
                    color="emerald"
                />
            </div>

            {/* ── Recent activity ── */}
            <div className="card-panel">
                <div className="panel-header">
                    <h2 className="panel-title">Actividad reciente</h2>
                    <span className="panel-sub">Últimos 5 movimientos</span>
                </div>
                <table className="activity-table">
                    <thead>
                        <tr>
                            <th>Fecha</th>
                            <th>Operación</th>
                            <th>Entidad</th>
                            <th style={{ textAlign: 'right' }}>Detalle</th>
                        </tr>
                    </thead>
                    <tbody>
                        {auditoria.length > 0 ? auditoria.map((row, i) => (
                            <tr key={i}>
                                <td>{row.fecha?.slice(0, 10)}</td>
                                <td>{row.tipo_accion}</td>
                                <td>{row.entidad_afectada}</td>
                                <td style={{ textAlign: 'right' }}>{row.campo_modificado ?? '—'}</td>
                            </tr>
                        )) : RECENT_PLACEHOLDER.map((row, i) => (
                            <tr key={i}>
                                <td>{row.fecha}</td>
                                <td>{row.operacion}</td>
                                <td>{row.socio}</td>
                                <td style={{ textAlign: 'right' }}>—</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>

            {/* ── Quick actions ── */}
            <div className="quick-grid">
                <QuickBtn icon={<Users size={28} />} label="Socios" />
                <QuickBtn icon={<TrendingUp size={28} />} label="Acciones" />
                <QuickBtn icon={<CreditCard size={28} />} label="Créditos" />
            </div>
        </div>
    );
}

function KpiCard({ icon, label, value, sub, color }: {
    icon: React.ReactNode; label: string; value: string; sub: string; color: string;
}) {
    return (
        <div className={`kpi-card kpi-${color}`}>
            <div className="kpi-icon">{icon}</div>
            <div className="kpi-label">{label}</div>
            <div className="kpi-value">{value}</div>
            <div className="kpi-sub">{sub}</div>
        </div>
    );
}

function QuickBtn({ icon, label }: { icon: React.ReactNode; label: string }) {
    return (
        <button className="quick-btn">
            {icon}
            <span>{label}</span>
        </button>
    );
}
