import { useState } from 'react';
import { Tabs } from '../../core/components/Tabs';
import { LiquidacionSocioPanel } from './LiquidacionSocioPanel';
import { LiquidacionFinalPanel } from './LiquidacionFinalPanel';

type Modo = 'socio' | 'final';

export function LiquidacionTab({ moneda }: { moneda: string }) {
    const [modo, setModo] = useState<Modo>('socio');

    return (
        <>
            <Tabs
                actual={modo}
                onChange={setModo}
                items={[
                    { valor: 'socio', etiqueta: 'De un socio' },
                    { valor: 'final', etiqueta: 'Final del Bankomunal' },
                ]}
            />

            {modo === 'socio' && <LiquidacionSocioPanel moneda={moneda} />}
            {modo === 'final' && <LiquidacionFinalPanel moneda={moneda} />}
        </>
    );
}
