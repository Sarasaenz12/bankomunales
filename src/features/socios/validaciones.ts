import { validarCampo } from '../../core/components/Campo';
import { DatosSocio } from './types';

/** Valida el formato de todo el formulario antes de enviarlo al backend. */
export function validarFormularioSocio(form: DatosSocio): Record<string, string> {
    const e: Record<string, string> = {};
    const revisar = (clave: string, tipo: 'numero' | 'email', valor: string, label: string) => {
        const msg = validarCampo(tipo, valor, label);
        if (msg) e[clave] = msg;
    };

    revisar('cedula', 'numero', form.cedula, 'La cédula');
    revisar('telefono', 'numero', form.telefono, 'El teléfono');
    revisar('celular', 'numero', form.celular, 'El celular');
    revisar('correo', 'email', form.correo, 'El correo');

    if (form.beneficiario) {
        revisar('ben_cedula', 'numero', form.beneficiario.cedula, 'La cédula del beneficiario');
    }
    form.protegidos.forEach((p, i) => {
        revisar(`prot${i}_cedula`, 'numero', p.cedula, `La cédula del protegido ${i + 1}`);
        revisar(`prot${i}_telefono`, 'numero', p.telefono, `El teléfono del protegido ${i + 1}`);
    });
    return e;
}