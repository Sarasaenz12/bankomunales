import { useOutletContext } from 'react-router-dom';
import { Bankomunal } from '../types';

/** Contexto que AppLayout inyecta a las páginas: el Bankomunal seleccionado. */
export function useBanco(): Bankomunal | null {
    return useOutletContext<{ bank: Bankomunal | null }>().bank;
}