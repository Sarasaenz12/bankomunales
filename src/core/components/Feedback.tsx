/** Muestra los mensajes de error y éxito repetidos en todas las pantallas. */
export function Feedback({ error, success }: { error: string; success: string }) {
    return (
        <>
            {error && <div className="error-message mb-4">{error}</div>}
            {success && <div className="success-message mb-4">{success}</div>}
        </>
    );
}