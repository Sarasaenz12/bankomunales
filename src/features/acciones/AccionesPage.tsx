import { useState } from 'react';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { CompraTab } from './CompraTab';
import { LiquidacionTab } from '../liquidacion/LiquidacionTab';
import { RepartoGananciasTab } from '../cierre/RepartoGananciasTab';

type Pestana = 'compra' | 'liquidacion' | 'ganancias';

export function AccionesPage() {
    const bank = useBanco();
    const [pestana, setPestana] = useState<Pestana>('compra');

    return (
        <div className="page">
            <PageHeader titulo="Acciones" subtitulo="Compra, liquidación y reparto de ganancias" />

            <Tabs
                actual={pestana}
                onChange={setPestana}
                items={[
                    { valor: 'compra', etiqueta: 'Compra de Acciones' },
                    { valor: 'liquidacion', etiqueta: 'Liquidación' },
                    { valor: 'ganancias', etiqueta: 'Reparto de Ganancias' },
                ]}
            />

            {pestana === 'compra' && <CompraTab moneda={bank?.moneda ?? 'COP'} />}
            {pestana === 'liquidacion' && <LiquidacionTab moneda={bank?.moneda ?? 'COP'} />}
            {pestana === 'ganancias' && <RepartoGananciasTab moneda={bank?.moneda ?? 'COP'} />}
        </div>
    );
}