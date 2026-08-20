import { CuotaPlaneada } from './types';
import { fmt } from '../../core/lib/format';

/** RF-46/RF-61: tabla con saldo decreciente. El saldo se calcula restando el
 * capital pagado de cada cuota al monto inicial. */
export function TablaAmortizacion({ cuotas, monto, moneda }: { cuotas: CuotaPlaneada[]; monto: number; moneda: string }) {
    let saldo = monto;
    const filas = cuotas.map(c => {
        const saldoAntes = saldo;
        saldo = Math.max(0, saldo - c.capital);
        return { ...c, saldo: saldoAntes };
    });

    return (
        <div className="tabla-scroll">
            <table className="activity-table">
                <thead>
                    <tr>
                        <th>N°</th>
                        <th>Cuota</th>
                        <th>Capital</th>
                        <th>Interés</th>
                        <th>Saldo</th>
                    </tr>
                </thead>
                <tbody>
                    {filas.map(c => (
                        <tr key={c.numero}>
                            <td>{c.numero}</td>
                            <td>{fmt(c.valor_total, moneda)}</td>
                            <td>{fmt(c.capital, moneda)}</td>
                            <td>{fmt(c.interes, moneda)}</td>
                            <td>{fmt(c.saldo, moneda)}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}