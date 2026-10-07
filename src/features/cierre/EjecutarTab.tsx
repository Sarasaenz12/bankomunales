import { useState } from 'react';
import { cierreService } from './cierreService';
import { ResultadoCierre } from './types';
import { fmt } from '../../core/lib/format';
import { mesActual } from './util';
import { useEnvioUnico } from '../../core/lib/useEnvioUnico';

export function EjecutarTab({ moneda }: { moneda: string }) {
    const [mes, setMes] = useState(mesActual());
    const [nombre, setNombre] = useState('');
    const [resultado, setResultado] = useState<ResultadoCierre | null>(null);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');
    const { enviando, ejecutar: enviarUnico } = useEnvioUnico();

    async function ejecutar() {
        setError(''); setSuccess(''); setResultado(null);
        try {
            const r = await cierreService.ejecutarCierre(mes + '-01', nombre);
            setResultado(r);
            setSuccess(`Mes ${r.cierre.mes} cerrado.`);
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo ejecutar el cierre');
        }
    }

    const r = resultado?.reparto;

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            <div className="chip-row" style={{ alignItems: 'flex-end', gap: '1rem', flexWrap: 'wrap' }}>
                <div className="form-group custom-fg" style={{ maxWidth: 240 }}>
                    <label className="form-label">Mes a cerrar</label>
                    <input className="form-input" type="month" value={mes} onChange={e => setMes(e.target.value)} />
                </div>
                <div className="form-group custom-fg" style={{ flex: 1, minWidth: 260 }}>
                    <label className="form-label">Quién ejecuta el cierre</label>
                    <input className="form-input" placeholder="Nombre de quien realiza el cierre"
                        value={nombre} onChange={e => setNombre(e.target.value)} />
                </div>
                <button className="btn btn-primary" onClick={() => enviarUnico(ejecutar)} disabled={enviando}>
                    {enviando ? 'Cerrando…' : 'Ejecutar cierre'}
                </button>
            </div>

            <div className="text-sm text-muted mt-4">
                El cierre es un acto de la Junta: congela el mes, marca los movimientos del Libro
                , retiene los fondos y devenga las ganancias del mes. No se
                puede cerrar un mes si el anterior sigue abierto.
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            {resultado && (
                <>
                    <div className="detalle-card mt-4">
                        <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>
                            Balance del cierre {resultado.cierre.mes}
                        </div>
                        <div className="grid-dos">
                            <div>Activo total: <strong>{fmt(resultado.cierre.activo_total, moneda)}</strong></div>
                            <div>Pasivo total: <strong>{fmt(resultado.cierre.pasivo_total, moneda)}</strong></div>
                            <div>Colocación: <strong>{resultado.cierre.colocacion_pct.toFixed(2)}%</strong></div>
                            <div>PPCFC: <strong>{resultado.cierre.ppcfc_pct.toFixed(2)}%</strong></div>
                            <span className={`badge-estatus ${resultado.cierre.cuadra ? 'badge-activo' : 'badge-retirado'}`}>
                                {resultado.cierre.cuadra ? 'CUADRA' : 'NO CUADRA'}
                            </span>
                        </div>
                    </div>

                    {r && (
                        <div className="detalle-card mt-4">
                            <div className="text-sm text-muted" style={{ marginBottom: '0.5rem' }}>
                                Reparto devengado
                            </div>
                            <div className="grid-dos">
                                <div>Balance neto: <strong>{fmt(r.balance_neto, moneda)}</strong></div>
                                <div>Valor por acción: <strong>{fmt(r.valor_por_accion, moneda)}</strong></div>
                                <div>Acciones activas: <strong>{r.acciones_activas}</strong></div>
                            </div>
                            <div className="text-sm text-muted mt-4">
                                El devengo quedó registrado como PENDIENTE para cada lote activo; se
                                pagará al cumplir su año.
                            </div>
                        </div>
                    )}
                </>
            )}
        </div>
    );
}