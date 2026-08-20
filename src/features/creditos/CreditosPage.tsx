import { useState } from 'react';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { Pendiente } from '../../core/components/Pendiente';
import { NuevaSolicitudTab } from './NuevaSolicitudTab';
import { BandejaTab } from './BandejaTab';
import { DesembolsoTab } from './DesembolsoTab';

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

            {pestana === 'pago' && (
                <Pendiente modulo="Pago de Créditos" requisitos="RF-63 a RF-72"
                    razon="El pago de cuotas exige las cuotas ya desembolsadas de este módulo y la
                           cartera vigente; entra en la siguiente fase (RF-63..RF-72)." />
            )}
            {pestana === 'refinanciamiento' && (
                <Pendiente modulo="Refinanciamiento" requisitos="RF-82"
                    razon="Refinanciar un crédito salda el anterior y abre uno nuevo; depende del
                           registro de pagos (RF-63..RF-72), que aún no está construido." />
            )}
            {pestana === 'deuda' && (
                <Pendiente modulo="Deuda Pendiente" requisitos="RF-73 a RF-79"
                    razon="El reporte de deuda pendiente sale de las cuotas en mora, que produce el
                           módulo de pagos (RF-63..RF-72)." />
            )}
        </div>
    );
}