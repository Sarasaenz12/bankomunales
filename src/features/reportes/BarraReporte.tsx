/** Barra superior común a todos los reportes: rango de fechas (opcional), una
 *  descripción y el botón de exportar. El filtrado por fecha es en vivo. */
export function BarraReporte({
    descripcion, desde, hasta, setDesde, setHasta, onExportar, conFechas = true, exportarActivo = true,
}: {
    descripcion: string;
    desde: string;
    hasta: string;
    setDesde: (v: string) => void;
    setHasta: (v: string) => void;
    onExportar: () => void;
    /** Los reportes de foto fija (Balance, Fiadores) no tienen dimensión de fecha. */
    conFechas?: boolean;
    exportarActivo?: boolean;
}) {
    return (
        <div style={{ display: 'flex', alignItems: 'flex-end', gap: '1rem', flexWrap: 'wrap', marginBottom: '1.25rem' }}>
            {conFechas && (
                <>
                    <div className="form-group custom-fg" style={{ margin: 0 }}>
                        <label className="form-label">Desde</label>
                        <input type="date" className="form-input" value={desde} onChange={e => setDesde(e.target.value)} />
                    </div>
                    <div className="form-group custom-fg" style={{ margin: 0 }}>
                        <label className="form-label">Hasta</label>
                        <input type="date" className="form-input" value={hasta} onChange={e => setHasta(e.target.value)} />
                    </div>
                    {(desde || hasta) && (
                        <button className="btn btn-secondary" onClick={() => { setDesde(''); setHasta(''); }}>
                            Limpiar
                        </button>
                    )}
                </>
            )}
            <span className="text-sm text-muted" style={{ flex: 1, minWidth: 160 }}>{descripcion}</span>
            <button className="btn btn-secondary" onClick={onExportar} disabled={!exportarActivo}>
                Exportar a Excel
            </button>
        </div>
    );
}
