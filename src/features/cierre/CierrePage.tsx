import { useState } from 'react';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { CuadreTab } from './CuadreTab';
import { EjecutarTab } from './EjecutarTab';
import { HistorialTab } from './HistorialTab';

type Pestana = 'cuadre' | 'ejecutar' | 'historial';

export function CierrePage() {
    const bank = useBanco();
    const [pestana, setPestana] = useState<Pestana>('cuadre');
    const moneda = bank?.moneda ?? 'COP';

    return (
        <div className="page">
            <PageHeader
                titulo="Cuadre y Cierre de Mes"
                subtitulo="Cierre mensual y cuadre de balances"
            />

            <Tabs
                actual={pestana}
                onChange={setPestana}
                items={[
                    { valor: 'cuadre', etiqueta: 'Cuadre y Balance' },
                    { valor: 'ejecutar', etiqueta: 'Ejecutar Cierre' },
                    { valor: 'historial', etiqueta: 'Historial de Cierres' },
                ]}
            />

            {pestana === 'cuadre' && <CuadreTab moneda={moneda} />}
            {pestana === 'ejecutar' && <EjecutarTab moneda={moneda} />}
            {pestana === 'historial' && <HistorialTab moneda={moneda} />}
        </div>
    );
}