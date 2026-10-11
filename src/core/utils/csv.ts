import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';

/**
 * Exporta datos a un archivo Excel (.xlsx) real. El navegador embebido de Tauri bloquea las
 * descargas por `<a download>` y un CSV plano se ve mal (todo en una columna, con comillas),
 * así que se abre el diálogo "Guardar como" y el .xlsx lo genera el backend con encabezado en
 * negrita y columnas autoajustadas.
 */
export async function exportarCSV(nombreArchivo: string, cabeceras: string[], filas: unknown[][]) {
    const ruta = await save({
        defaultPath: `${nombreArchivo}.xlsx`,
        filters: [{ name: 'Excel', extensions: ['xlsx'] }],
    });
    if (!ruta) return; // el usuario canceló el diálogo

    const filasTexto = filas.map(f =>
        f.map(v => (v === null || v === undefined) ? '' : String(v)),
    );
    await invoke('guardar_excel', { ruta, cabeceras, filas: filasTexto });
}
