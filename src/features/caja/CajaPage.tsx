import { useEffect, useState } from 'react';
import { cajaService } from './cajaService';
import { Movimiento, ResumenCaja, Bien } from './types';
import { LibroTab } from './LibroTab';
import { RegistrarTab } from './RegistrarTab';
import { BienesTab } from './BienesTab';
import { fmt } from '../../core/lib/format';
import { PageHeader } from '../../core/components/PageHeader';
import { Tabs } from '../../core/components/Tabs';
import { useBanco } from '../../core/context/BankContext';

type Pestana = 'libro' | 'registrar' | 'bienes';

export function CajaPage() {
    const bank = useBanco();
    const moneda = bank?.moneda ?? 'COP';

    const [pestana, setPestana] = useState<Pestana>('libro');
    const [libro, setLibro] = useState<Movimiento[]>([]);
    const [resumen, setResumen] = useState<ResumenCaja | null>(null);
    const [bienes, setBienes] = useState<Bien[]>([]);
    const [desde, setDesde] = useState('');
    const [hasta, setHasta] = useState('');
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');

    useEffect(() => { if (bank) recargar(); }, [bank]);

    async function recargar() {
        try {
            const r = await cajaService.cargarTodo(desde, hasta);
            setLibro(r.libro);
            setResumen(r.resumen);
            setBienes(r.bienes);
        } catch (e: any) {
            setError(e?.toString() ?? 'Error al cargar la caja');
        }
    }

    function aviso(msg: string) {
        setSuccess(msg);
        setError('');
        recargar();
    }

    return (
        <div className="page">
            <PageHeader titulo="Caja y Contabilidad" subtitulo="Libro de Ingresos y Egresos del Bankomunal" />

            <div className="kpi-grid">
                <div className="kpi-card">
                    <div className="kpi-label">Saldo en caja</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.saldo_caja, moneda) : '…'}</div>
                    <div className="kpi-sub">Disponible en efectivo</div>
                </div>
                <div className="kpi-card">
                    <div className="kpi-label">Fondo para Gastos</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.saldo_fondo_gastos, moneda) : '…'}</div>
                    <div className="kpi-sub">Para gastos operativos (RN-07)</div>
                </div>
                <div className="kpi-card">
                    <div className="kpi-label">Activo fijo</div>
                    <div className="kpi-value">{resumen ? fmt(resumen.valor_activo_fijo, moneda) : '…'}</div>
                    <div className="kpi-sub">Bienes propios y en comodato</div>
                </div>
            </div>

            <Tabs
                actual={pestana}
                onChange={setPestana}
                items={[
                    { valor: 'libro', etiqueta: 'Libro de Ingresos y Egresos' },
                    { valor: 'registrar', etiqueta: 'Registrar operación' },
                    { valor: 'bienes', etiqueta: 'Bienes / Activo Fijo' },
                ]}
            />

            {error && <div className="error-message mb-4">{error}</div>}
            {success && <div className="success-message mb-4">{success}</div>}

            {pestana === 'libro' && (
                <LibroTab
                    libro={libro} moneda={moneda}
                    desde={desde} hasta={hasta}
                    setDesde={setDesde} setHasta={setHasta}
                    onFiltrar={recargar}
                    onCorregido={() => aviso('Operación corregida.')}
                    onError={setError}
                />
            )}

            {pestana === 'registrar' && (
                <RegistrarTab onHecho={aviso} onError={setError} />
            )}

            {pestana === 'bienes' && (
                <BienesTab bienes={bienes} moneda={moneda} onHecho={aviso} onError={setError} />
            )}
        </div>
    );
}