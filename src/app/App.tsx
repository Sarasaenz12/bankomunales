import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthPage } from '../features/auth/AuthPage';
import { AppLayout } from '../core/components/AppLayout';
import { DashboardPage } from '../features/dashboard/DashboardPage';
import { ConfigPage } from '../features/configuracion/ConfigPage';
import { SociosPage } from '../features/socios/SociosPage';
import { SocioDetallePage } from '../features/socios/SocioDetallePage';
import { CajaPage } from '../features/caja/CajaPage';
import { AccionesPage } from '../features/acciones/AccionesPage';
import { CreditosPage } from '../features/creditos/CreditosPage';
import { CierrePage } from '../features/cierre/CierrePage';
import { ReportesPage } from '../features/reportes/ReportesPage';
import { RespaldoPage } from '../features/respaldo/RespaldoPage';
import { AuditoriaPage } from '../features/auditoria/AuditoriaPage';
import '../App.css';

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
          <Route path="creditos" element={<CreditosPage />} />
          <Route path="caja" element={<CajaPage />} />
          <Route path="cierre" element={<CierrePage />} />
          <Route path="reportes" element={<ReportesPage />} />
          <Route path="respaldo" element={<RespaldoPage />} />
          <Route path="auditoria" element={<AuditoriaPage />} />
        </Route>
        {/* Redirect old /config route */}
        <Route path="/config" element={<Navigate to="/app/config" replace />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;