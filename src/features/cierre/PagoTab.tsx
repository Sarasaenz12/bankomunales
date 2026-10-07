import { useEffect, useState } from 'react';
import { cierreService } from './cierreService';
import { PagoGananciaRegistrado } from './types';
import { fmt } from '../../core/lib/format';
import { fechaHoy } from './util';
import { sociosService } from '../socios/sociosService';
import { SocioResumen } from '../socios/types';

export function PagoTab({ moneda }: { moneda: string }) {
    const [fecha, setFecha] = useState(fechaHoy());
    const [pagos, setPagos] = useState<PagoGananciaRegistrado[]>([]);
    const [socios, setSocios] = useState<SocioResumen[]>([]);
    const [error, setError] = useState('');
    const [success, setSuccess] = useState('');

    useEffect(() => {
        sociosService.listar().then(setSocios).catch(() => {});
    }, []);

    const nombreSocio = (id: string) => {
        const s = socios.find(x => x.id === id);
        return s ? s.nombre_completo : id;
    };

    async function pagar() {
        setError(''); setSuccess(''); setPagos([]);
        try {
            const r = await cierreService.pagarGananciasCumplidas(fecha);
            setPagos(r);
            setSuccess(r.length
                ? `Se pagaron las ganancias de ${r.length} socio(s).`
                : 'No hay lotes que hayan cumplido su año todavía.');
        } catch (e: any) {
            setError(e?.toString() ?? 'No se pudo pagar las ganancias');
        }
    }

    const total = pagos.reduce((acc, p) => acc + p.monto, 0);

    return (
        <div className="detalle-card" style={{ maxWidth: 900 }}>
            <div className="chip-row" style={{ alignItems: 'flex-end', gap: '1rem', flexWrap: 'wrap' }}>
                <div className="form-group custom-fg" style={{ maxWidth: 240 }}>
                    <label className="form-label">Fecha del pago</label>
                    <input className="form-input" type="date" value={fecha} onChange={e => setFecha(e.target.value)} />
                </div>
                <button className="btn btn-primary" onClick={pagar}>
                    Pagar ganancias cumplidas
                </button>
            </div>

            <div className="text-sm text-muted mt-4">
                Paga la ganancia acumulada de cada lote que cumplió un año desde su compra.
                Cada pago se asienta en el Libro como UR (egreso). Es idempotente: un lote
                ya pagado no vuelve a pagarse.
            </div>

            {error && <div className="error-message mt-4">{error}</div>}
            {success && <div className="success-message mt-4">{success}</div>}

            {pagos.length > 0 && (
                <>
                    <table className="activity-table mt-4">
                        <thead>
                            <tr>
                                <th>Socio</th>
                                <th style={{ textAlign: 'right' }}>Monto pagado</th>
                            </tr>
                        </thead>
                        <tbody>
                            {pagos.map(p => (
                                <tr key={p.socio_id}>
                                    <td>{nombreSocio(p.socio_id)}</td>
                                    <td style={{ textAlign: 'right' }}>{fmt(p.monto, moneda)}</td>
                                </tr>
                            ))}
                        </tbody>
                        <tfoot>
                            <tr>
                                <td><strong>Total</strong></td>
                                <td style={{ textAlign: 'right' }}><strong>{fmt(total, moneda)}</strong></td>
                            </tr>
                        </tfoot>
                    </table>
                </>
            )}
        </div>
    );
}