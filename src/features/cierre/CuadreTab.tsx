import { useState } from 'react';
import { cierreService } from './cierreService';
import { PrevisualizacionCierre } from './types';
import { fmt } from '../../core/lib/format';
import { mesActual } from './util';
import { mensajeDeError } from '../../core/lib/errores';

export function CuadreTab({ moneda }: { moneda: string }) {
    const [mes, setMes] = useState(mesActual());
    const [datos, setDatos] = useState<PrevisualizacionCierre | null>(null);
    const [error, setError] = useState('');

    async function previsualizar() {
        setError(''); setDatos(null);
        try {
            setDatos(await cierreService.previsualizarCierre(mes + '-01'));
        } catch (e) {
            setError(mensajeDeError(e, 'No se pudo calcular el cuadre'));
        }
    }

    const c = datos?.cuadre;
    const i = datos?.informe;
    const r = datos?.reparto;

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            <div className="chip-row" style={{ alignItems: 'flex-end', gap: '1rem', flexWrap: 'wrap' }}>
                <div className="form-group custom-fg" style={{ maxWidth: 240, marginBottom: 0 }}>
                    <label className="form-label">Mes a cuadrar</label>
                    <input className="form-input" type="month" value={mes} onChange={e => setMes(e.target.value)} />
                </div>
                <button className="btn btn-primary" onClick={previsualizar}>
                    Previsualizar cuadre
                </button>
            </div>

            {error && <div className="error-message mt-4">{error}</div>}

            {c && (
                <>
                    <div className="text-sm text-muted mt-4" style={{ marginBottom: '0.5rem' }}>
                        Cuadre del mes <strong>{c.mes}</strong>
                    </div>
                    <div className="grid-cuadre">
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Activo</div>
                            <div>Efectivo: <strong>{fmt(c.efectivo, moneda)}</strong></div>
                            <div>Cartera: <strong>{fmt(c.cartera, moneda)}</strong></div>
                            <div>Bienes: <strong>{fmt(c.bienes, moneda)}</strong></div>
                            <div className="total-line">Total: <strong>{fmt(c.activo_total, moneda)}</strong></div>
                        </div>
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Pasivo + Patrimonio</div>
                            <div>Capital, fondos y ganancias: <strong>{fmt(c.pasivo_total, moneda)}</strong></div>
                            <div className="total-line">Total: <strong>{fmt(c.pasivo_total, moneda)}</strong></div>
                        </div>
                        <div className="detalle-card">
                            <div className="text-sm text-muted">Resultado</div>
                            <span className={`badge-estatus ${c.cuadra ? 'badge-activo' : 'badge-retirado'}`}>
                                {c.cuadra ? 'CUADRA' : 'NO CUADRA'}
                            </span>
                            <div className="text-sm mt-4">
                                Diferencia: <strong>{fmt(Math.abs(c.activo_total - c.pasivo_total), moneda)}</strong>
                            </div>
                        </div>
                    </div>
                </>
            )}

            {i && (
                <div className="detalle-card mt-4">
                    <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>
                        Informe de Gestión del mes
                    </div>
                    <div className="grid-dos">
                        <div>Socios activos: <strong>{i.socios_activos}</strong></div>
                        <div>Créditos vigentes: <strong>{i.creditos_vigentes}</strong></div>
                        <div>Monto en cartera: <strong>{fmt(i.monto_cartera, moneda)}</strong></div>
                        <div>Efectivo disponible: <strong>{fmt(i.efectivo_disponible, moneda)}</strong></div>
                        <div>Rendimiento estimado: <strong>{fmt(i.rendimiento_estimado, moneda)}/acción</strong></div>
                    </div>
                </div>
            )}

            {r && (
                <div className="detalle-card mt-4">
                    <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>
                        Reparto de Ganancias
                    </div>
                    <div className="grid-dos">
                        <div>Ingresos repartibles (intereses, mora y otros ingresos): <strong>{fmt(r.ingresos_repartibles, moneda)}</strong></div>
                        <div>Otros egresos: <strong>{fmt(r.otros_egresos, moneda)}</strong></div>
                        <div>Fondo para Gastos (10%): <strong>{fmt(r.fondo_gastos, moneda)}</strong></div>
                        <div>Fondo de Incobrables (10%, con tope): <strong>{fmt(r.fondo_incobrables, moneda)}</strong></div>
                        <div>Balance neto: <strong>{fmt(r.balance_neto, moneda)}</strong></div>
                        <div>Valor por acción: <strong>{fmt(r.valor_por_accion, moneda)}</strong></div>
                        <div>Acciones activas: <strong>{r.acciones_activas}</strong></div>
                    </div>
                    <div className="text-sm text-muted mt-4">
                        El cuadre no toca datos definitivos: puede repetirlo cuantas veces quiera antes de cerrar.
                    </div>
                </div>
            )}
        </div>
    );
}