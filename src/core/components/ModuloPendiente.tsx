import { PageHeader } from '../../core/components/PageHeader';

/** Página provisional para los módulos aún no construidos. */
export function ModuloPendiente({ titulo, requisitos }: { titulo: string; requisitos: string }) {
    return (
        <div className="page">
            <PageHeader titulo={titulo} subtitulo={requisitos} />
            <div className="pendiente-nota">
                <strong>{titulo}</strong> todavía no está construido. Próximamente.
            </div>
        </div>
    );
}