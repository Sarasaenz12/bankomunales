import { Check, X } from 'lucide-react';

/** Fila de una validación automática (RN-03, RN-04, etc.). */
export function ItemValidacion({ ok, titulo, detalle }: { ok: boolean; titulo: string; detalle: string }) {
    return (
        <div className="valida-item" style={{ borderLeft: `3px solid ${ok ? 'var(--success-color)' : 'var(--error-color)'}` }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                {ok ? <Check size={16} style={{ color: 'var(--success-color)' }} /> : <X size={16} style={{ color: 'var(--error-color)' }} />}
                <strong style={{ fontSize: '0.875rem' }}>{titulo}</strong>
            </div>
            <div className="text-sm text-muted" style={{ marginLeft: '1.5rem', fontSize: '0.8125rem' }}>{detalle}</div>
        </div>
    );
}