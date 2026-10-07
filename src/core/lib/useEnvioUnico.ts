import { useCallback, useRef, useState } from 'react';

/**
 * Evita el doble registro por doble clic. `ejecutar(fn)` corre `fn` sólo si no hay otro
 * envío en curso: usa un candado sincrónico (`ref`, que bloquea aun el doble clic más
 * rápido, antes de que React re-renderice) además del estado `enviando`, pensado para
 * deshabilitar el botón y cambiar su texto. Si ya hay un envío en curso, ignora la llamada.
 */
export function useEnvioUnico() {
    const bloqueo = useRef(false);
    const [enviando, setEnviando] = useState(false);

    const ejecutar = useCallback(async (fn: () => Promise<void> | void) => {
        if (bloqueo.current) return;
        bloqueo.current = true;
        setEnviando(true);
        try {
            await fn();
        } finally {
            bloqueo.current = false;
            setEnviando(false);
        }
    }, []);

    return { enviando, ejecutar };
}
