/** Cabecera común de página: título, subtítulo y el badge "Mes abierto". */
export function PageHeader({ titulo, subtitulo }: { titulo: string; subtitulo: string }) {
    return (
        <div className="page-header">
            <div>
                <h1 className="page-title">{titulo}</h1>
                <p className="page-subtitle">{subtitulo}</p>
            </div>
            <span className="badge-mes">Mes abierto</span>
        </div>
    );
}