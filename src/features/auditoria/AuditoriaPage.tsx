import { useEffect, useState } from 'react';
import { EntradaAuditoria } from '../../core/types';
import { useBanco } from '../../core/context/BankContext';
import { api } from '../../core/lib/api';
import { PageHeader } from '../../core/components/PageHeader';
import { mensajeDeError } from '../../core/lib/errores';

// El backend guarda las fechas de auditoría en UTC sin marcador (`Utc::now()`), p. ej.
// "2026-09-01T19:30:00". Se interpretan como UTC (añadiendo la Z si falta) y se muestran
// en hora de Bogotá. El intento anterior las parseaba como hora local y volvía a ajustar
// el offset, lo que daba horas equivocadas.
const formatearFechaHora = (fechaStr: string): string => {
    if (!fechaStr) return '—';
    try {
        const tieneZona = /[zZ]|[+-]\d\d:?\d\d$/.test(fechaStr);
        const fecha = new Date(tieneZona ? fechaStr : `${fechaStr}Z`);
        if (isNaN(fecha.getTime())) return '—';
        return fecha.toLocaleString('es-CO', {
            timeZone: 'America/Bogota',
            day: '2-digit', month: '2-digit', year: 'numeric',
            hour: '2-digit', minute: '2-digit', hour12: false,
        });
    } catch {
        return '—';
    }
};

// Etiquetas legibles para el tipo de acción registrado (ver tipo_accion en el backend).
const ETIQUETA_TIPO: Record<string, string> = {
    MODIFICACION: 'Configuración',
    CORRECCION_OPERACION: 'Corrección contable',
    CIERRE_MES: 'Cierre de mes',
    PAGO_GANANCIAS: 'Pago de ganancias',
    PAGO_CUOTA: 'Pago de cuota',
    RESTAURACION: 'Restauración',
};

function BadgeTipo({ tipo }: { tipo: string }) {
    const etiqueta = ETIQUETA_TIPO[tipo] ?? tipo;
    return (
        <span style={{
            display: 'inline-block', background: '#e2e8f0', color: '#334155',
            fontSize: '0.8rem', fontWeight: 600, padding: '0.3rem 0.7rem', borderRadius: '999px',
        }}>
            {etiqueta}
        </span>
    );
}

export function AuditoriaPage() {
    const [auditorias, setAuditorias] = useState<EntradaAuditoria[]>([]);
    const [error, setError] = useState('');
    const [cargando, setCargando] = useState(true);
    const bancoId = useBanco()?.id;

    useEffect(() => {
        if (!bancoId) return;
        api.listarAuditoria()
            .then(setAuditorias)
            .catch((e: unknown) => setError(mensajeDeError(e, 'Error al cargar la bitácora')))
            .finally(() => setCargando(false));
    }, [bancoId]);

    if (cargando) return <div className="page">Cargando bitácora…</div>;

    return (
        <div className="page">
            <PageHeader titulo="Auditoría" subtitulo="Bitácora de cambios sensibles" />

            <p className="text-sm text-muted" style={{ marginBottom: '1.25rem' }}>
                Cada cambio de configuración, corrección contable o restauración queda registrado.
            </p>

            {error && <div className="error-message mb-4">{error}</div>}

            <div className="detalle-card">
                {auditorias.length === 0 ? (
                    <div className="empty-state">Aún no hay cambios registrados en el sistema.</div>
                ) : (
                    <div style={{ overflowX: 'auto' }}>
                        <table className="activity-table">
                            <thead>
                                <tr>
                                    <th>Fecha</th>
                                    <th>Quién realizó el cambio</th>
                                    <th>Qué se modificó</th>
                                    <th>Valor anterior → Valor nuevo</th>
                                    <th>Tipo de cambio</th>
                                </tr>
                            </thead>
                            <tbody>
                                {auditorias.map((a, i) => {
                                    const que = a.campo_modificado
                                        ? `${a.entidad_afectada ?? ''}${a.entidad_afectada ? ' — ' : ''}${a.campo_modificado}`
                                        : (a.entidad_afectada || '—');
                                    const tieneValores = a.valor_anterior != null || a.valor_nuevo != null;
                                    return (
                                        <tr key={a.id || i}>
                                            <td style={{ whiteSpace: 'nowrap' }}>{formatearFechaHora(a.fecha)}</td>
                                            <td>{a.nombre_quien_realiza || '—'}</td>
                                            <td>{que}</td>
                                            <td>
                                                {tieneValores ? (
                                                    <span>
                                                        <span className="text-muted">{a.valor_anterior ?? '—'}</span>
                                                        {' → '}
                                                        <strong>{a.valor_nuevo ?? '—'}</strong>
                                                    </span>
                                                ) : '—'}
                                            </td>
                                            <td>{a.tipo_accion && <BadgeTipo tipo={a.tipo_accion} />}</td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </div>
    );
}
