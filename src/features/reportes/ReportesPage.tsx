import { useState } from 'react';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { ReporteSociosActivosTab } from './ReporteSociosActivosTab';
import { ReporteGestionTab } from './ReporteGestionTab';
import { ReporteBalanceTab } from './ReporteBalanceTab';
import { ReporteFiadoresTab } from './ReporteFiadoresTab';
import { ReporteCreditosTab } from './ReporteCreditosTab';
import { ReporteLibroTab } from './ReporteLibroTab';
import { ReporteAccionesTab } from './ReporteAccionesTab';

type TipoReporte = 'socios' | 'gestion' | 'balance' | 'fiadores' | 'creditos' | 'libro' | 'acciones';

const REPORTES_TABS = [
    { valor: 'socios' as const, etiqueta: 'Socios Activos' },
    { valor: 'gestion' as const, etiqueta: 'Gestión' },
    { valor: 'balance' as const, etiqueta: 'Balance General' },
    { valor: 'fiadores' as const, etiqueta: 'Fiadores' },
    { valor: 'creditos' as const, etiqueta: 'Saldos de Créditos' },
    { valor: 'libro' as const, etiqueta: 'Ingresos y Egresos' },
    { valor: 'acciones' as const, etiqueta: 'Control de Acciones' },
];

export function ReportesPage() {
    const [tabActual, setTabActual] = useState<TipoReporte>('socios');

    return (
        <div className="page">
            <PageHeader titulo="Reportes" subtitulo="Consulta y exportación de información" />

            <Tabs actual={tabActual} onChange={setTabActual} items={REPORTES_TABS} />

            {tabActual === 'socios' && <ReporteSociosActivosTab />}
            {tabActual === 'gestion' && <ReporteGestionTab />}
            {tabActual === 'balance' && <ReporteBalanceTab />}
            {tabActual === 'fiadores' && <ReporteFiadoresTab />}
            {tabActual === 'creditos' && <ReporteCreditosTab />}
            {tabActual === 'libro' && <ReporteLibroTab />}
            {tabActual === 'acciones' && <ReporteAccionesTab />}
        </div>
    );
}