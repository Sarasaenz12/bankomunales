import { api } from '../../core/lib/api';
import { Bankomunal, NuevoBankomunal } from '../../core/types';

export const authService = {
    login: (password: string) => api.login(password),

    listarBankomunales: () => api.listarBankomunales(),

    crearBankomunal: (nuevo: NuevoBankomunal) => api.crearBankomunal(nuevo),

    seleccionarBankomunal: (id: string) => api.seleccionarBankomunal(id),

    volverASeleccion: () => api.volverASeleccion(),

    bancoSeleccionado: () => api.bancoSeleccionado(),

    /** Si hay un banco activo lo devuelve; si no, devuelve `null`. */
    async bancoActual(): Promise<Bankomunal | null> {
        return api.bancoSeleccionado();
    },
};