import { useEffect, useState } from 'react';
import { NavLink, Outlet, useNavigate } from 'react-router-dom';
import {
    LayoutDashboard, Settings, Users, TrendingUp, CreditCard,
    Calculator, CalendarCheck, BarChart2, HardDrive, ClipboardList,
    ArrowLeftRight, Building2
} from 'lucide-react';
import { api } from '../lib/api';
import { Bankomunal } from '../types';

export function AppLayout() {
    const navigate = useNavigate();
    const [bank, setBank] = useState<Bankomunal | null>(null);

    useEffect(() => {
        api.bancoSeleccionado().then(b => {
            if (!b) { navigate('/'); return; }
            setBank(b);
        }).catch(() => navigate('/'));
    }, [navigate]);

    const handleSwitch = async () => {
        await api.volverASeleccion();
        navigate('/');
    };

    return (
        <div className="app-shell">
            {/* ── SIDEBAR ── */}
            <aside className="sidebar">
                <div className="sidebar-brand">
                    <Building2 size={22} />
                    <div>
                        <div className="sidebar-brand-title">Bankomunales</div>
                        <div className="sidebar-brand-sub">{bank?.nombre ?? '...'}</div>
                    </div>
                </div>

                <nav className="sidebar-nav">
                    <div className="nav-section-label">General</div>
                    <NavLink to="/app/dashboard" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <LayoutDashboard size={17} /> Inicio / Dashboard
                    </NavLink>
                    <NavLink to="/app/config" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <Settings size={17} /> Datos del Bankomunal
                    </NavLink>

                    <div className="nav-section-label">Gestión</div>
                    <NavLink to="/app/socios" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <Users size={17} /> Socios
                    </NavLink>
                    <NavLink to="/app/acciones" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <TrendingUp size={17} /> Acciones
                    </NavLink>
                    <NavLink to="/app/creditos" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <CreditCard size={17} /> Créditos
                    </NavLink>

                    <div className="nav-section-label">Administración</div>
                    <NavLink to="/app/caja" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <Calculator size={17} /> Caja y Contabilidad
                    </NavLink>
                    <NavLink to="/app/cierre" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <CalendarCheck size={17} /> Cuadre y Cierre de Mes
                    </NavLink>
                    <NavLink to="/app/reportes" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <BarChart2 size={17} /> Reportes
                    </NavLink>
                    <NavLink to="/app/respaldo" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <HardDrive size={17} /> Respaldo y Restauración
                    </NavLink>
                    <NavLink to="/app/auditoria" className={({ isActive }) => `nav-item ${isActive ? 'active' : ''}`}>
                        <ClipboardList size={17} /> Auditoría
                    </NavLink>
                </nav>

                <button className="sidebar-switch" onClick={handleSwitch}>
                    <ArrowLeftRight size={15} /> Cambiar de Bankomunal
                </button>
            </aside>

            {/* ── MAIN CONTENT ── */}
            <main className="app-main">
                <Outlet context={{ bank }} />
            </main>
        </div>
    );
}
