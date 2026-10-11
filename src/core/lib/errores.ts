/** Texto legible de un error capturado (`catch` entrega `unknown`). Los errores del backend
 *  llegan como `AppError`, cuyo `toString()` ya es el mensaje para el usuario. */
export function mensajeDeError(e: unknown, defecto = 'Ocurrió un error inesperado'): string {
    if (e == null) return defecto;
    if (typeof e === 'string') return e || defecto;
    if (typeof e === 'object' && 'message' in e && typeof e.message === 'string') {
        return e.message || defecto;
    }
    const texto = String(e);
    return texto && texto !== '[object Object]' ? texto : defecto;
}
