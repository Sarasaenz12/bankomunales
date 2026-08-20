import { useState } from 'react';
import { useBanco } from '../../core/context/BankContext';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { Pendiente } from '../../core/components/Pendiente';
import { CompraTab } from './CompraTab';

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

            {pestana === 'liquidacion' && (
                <Pendiente
                    modulo="Liquidación de Acciones"
                    requisitos="RF-28 a RF-38"
                    razon="Calcular el valor a favor del socio exige conocer su deuda y las acciones
                           comprometidas como garantía, que produce el módulo de Créditos."
                />
            )}

            {pestana === 'ganancias' && (
                <Pendiente
                    modulo="Reparto de Ganancias"
                    requisitos="RF-39 a RF-42"
                    razon="El valor de ganancia por acción sale del Balance de Gestión Mensual Neto,
                           que produce el Cuadre y Cierre de Mes."
                />
            )}
        </div>
    );
}