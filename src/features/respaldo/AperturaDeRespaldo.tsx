import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { api } from '../../core/lib/api';
import { ImportarRespaldo } from './ImportarRespaldo';

/**
 * Atiende el doble clic en un archivo de respaldo desde el Explorador de Windows: si la app
 * se abrió con el archivo, o si ya estaba abierta y recibe uno nuevo, muestra el asistente
 * de importación sobre cualquier pantalla.
 */
export function AperturaDeRespaldo() {
    const [ruta, setRuta] = useState<string | null>(null);

    useEffect(() => {
        let activo = true;
        api.archivoPendiente().then(r => { if (activo && r) setRuta(r); }).catch(() => { });
        const quitar = listen<string>('archivo-pendiente', e => setRuta(e.payload));
        return () => { activo = false; quitar.then(f => f()).catch(() => { }); };
    }, []);

    if (!ruta) return null;
    return <ImportarRespaldo ruta={ruta} onCerrar={() => setRuta(null)} />;
}
