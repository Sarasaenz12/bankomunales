import { api } from '../../core/lib/api';
import { Bankomunal, NuevoBankomunal } from '../../core/types';

export const authService = {
    login: (password: string) => api.login(password),

    listarBankomunales: () => api.listarBankomunales(),

    /** RF-06: si existe exactamente un Bankomunal lo devuelve; si no, `null`. */
    unicoBankomunal: () => api.unicoBankomunal(),

    crearBankomunal: (nuevo: NuevoBankomunal) => api.crearBankomunal(nuevo),

    seleccionarBankomunal: (id: string) => api.seleccionarBankomunal(id),

    volverASeleccion: () => api.volverASeleccion(),

    bancoSeleccionado: () => api.bancoSeleccionado(),

    /** Si esta sesión ya inició sesión con la contraseña (no volver a pedirla). */
    sesionIniciada: () => api.sesionIniciada(),

    /** Si hay un banco activo lo devuelve; si no, devuelve `null`. */
    async bancoActual(): Promise<Bankomunal | null> {
        return api.bancoSeleccionado();
    },
};