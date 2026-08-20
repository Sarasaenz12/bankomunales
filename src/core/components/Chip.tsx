/** Chip de filtro/opción con estado activo. */
export function Chip({ activo, onClick, children, title }: {
    activo: boolean;
    onClick: () => void;
    children: React.ReactNode;
    title?: string;
}) {
    return (
        <button className={`chip ${activo ? 'active' : ''}`} onClick={onClick} title={title}>
            {children}
        </button>
    );
}