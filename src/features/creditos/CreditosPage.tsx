import { useState } from 'react';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { NuevaSolicitudTab } from './NuevaSolicitudTab';
import { BandejaTab } from './BandejaTab';
import { DesembolsoTab } from './DesembolsoTab';
import { PagoCuotaTab } from '../deuda/PagoCuotaTab';
import { RefinanciamientoTab } from '../deuda/RefinanciamientoTab';
import { PagoDeudaPendienteTab } from '../deuda/PagoDeudaPendienteTab';

type Pestana = 'nueva' | 'bandeja' | 'desembolso' | 'pago' | 'refinanciamiento' | 'deuda';

export function CreditosPage() {
    const bank = useBanco();
    const [pestana, setPestana] = useState<Pestana>('nueva');
    const moneda = bank?.moneda ?? 'COP';

    return (
        <div className="page">
            <PageHeader titulo="Créditos" subtitulo="Solicitudes, desembolsos y pagos" />

            <Tabs
                actual={pestana}
                onChange={setPestana}
                items={[
                    { valor: 'nueva', etiqueta: 'Nueva Solicitud' },
                    { valor: 'bandeja', etiqueta: 'Bandeja de Solicitudes' },
                    { valor: 'desembolso', etiqueta: 'Desembolso' },
                    { valor: 'pago', etiqueta: 'Pago' },
                    { valor: 'refinanciamiento', etiqueta: 'Refinanciamiento' },
                    { valor: 'deuda', etiqueta: 'Deuda Pendiente' },
                ]}
            />

            {pestana === 'nueva' && <NuevaSolicitudTab moneda={moneda} />}
            {pestana === 'bandeja' && <BandejaTab moneda={moneda} />}
            {pestana === 'desembolso' && <DesembolsoTab moneda={moneda} />}

            {pestana === 'pago' && <PagoCuotaTab moneda={moneda} />}

            {pestana === 'refinanciamiento' && <RefinanciamientoTab moneda={moneda} />}

            {pestana === 'deuda' && <PagoDeudaPendienteTab moneda={moneda} />}
        </div>
    );
}