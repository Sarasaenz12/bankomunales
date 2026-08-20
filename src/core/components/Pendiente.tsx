/** Aviso de que un módulo todavía no está construido. */
export function Pendiente({ modulo, requisitos, razon }: {
    modulo: string; requisitos: string; razon: string;
}) {
    return (
        <div className="pendiente-nota">
            <strong>{modulo}</strong> ({requisitos}) todavía no está construido.
            <div style={{ marginTop: '0.5rem' }}>{razon}</div>
        </div>
    );
}