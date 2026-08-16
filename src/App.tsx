import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthPage } from './pages/AuthPage';
import { AppLayout } from './components/AppLayout';
import { DashboardPage } from './pages/DashboardPage';
import { ConfigPage } from './pages/ConfigPage';
import { SociosPage } from './pages/SociosPage';
import { SocioDetallePage } from './pages/SocioDetallePage';
import { CajaPage } from './pages/CajaPage';
import { AccionesPage } from './pages/AccionesPage';
import './App.css';

function ComingSoon({ label }: { label: string }) {
  return (
    <div className="page">
      <div className="page-header">
        <h1 className="page-title">{label}</h1>
        <p className="page-subtitle" style={{ color: 'var(--text-muted)' }}>Módulo en desarrollo — próximamente</p>
      </div>
    </div>
  );
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<AuthPage />} />
        <Route path="/app" element={<AppLayout />}>
          <Route index element={<Navigate to="dashboard" replace />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="config" element={<ConfigPage />} />
          <Route path="socios" element={<SociosPage />} />
          <Route path="socios/nuevo" element={<SocioDetallePage />} />
          <Route path="socios/:id" element={<SocioDetallePage />} />
          <Route path="acciones" element={<AccionesPage />} />
          <Route path="creditos" element={<ComingSoon label="Créditos" />} />
          <Route path="caja" element={<CajaPage />} />
          <Route path="cierre" element={<ComingSoon label="Cuadre y Cierre de Mes" />} />
          <Route path="reportes" element={<ComingSoon label="Reportes" />} />
          <Route path="respaldo" element={<ComingSoon label="Respaldo y Restauración" />} />
          <Route path="auditoria" element={<ComingSoon label="Auditoría" />} />
        </Route>
        {/* Redirect old /config route */}
        <Route path="/config" element={<Navigate to="/app/config" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
