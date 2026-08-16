-- Migración v3 del `.db` de un Bankomunal.
-- Incorpora dos aclaraciones del cliente (2026-08-12), ver docs/decisiones-pendientes.md:
--   · D-11: los fondos son TRES, no dos (Gastos, Reserva, Incobrables).
--   · D-12: las ganancias se DEVENGAN cada mes por lote y se PAGAN al cumplir el año.
--
-- NUNCA se edita una migración ya publicada: los cambios entran como un script nuevo.

-- ─────────────────────────────────────────────────────────────────────────────
-- D-11 · Tercer fondo: Fondo de Reserva
--
-- El esquema traía dos fondos (Gastos e Incobrables). El cliente confirmó que son tres.
-- Reparto de responsabilidades asumido (ver D-11, pendiente de confirmar):
--   · Fondo para Gastos      → retención mensual; se gasta en gastos operativos (RF-86).
--   · Fondo de Reserva       → retención mensual con tope sobre el capital en acciones;
--                              es el colchón del que se descuenta al absorber un
--                              incobrable (RF-36).
--   · Fondo de Incobrables   → deuda irrecuperable acumulada de socios retirados con
--                              saldo (RF-35); baja cuando el retirado paga (RF-74).
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE configuracion ADD COLUMN pct_fondo_reserva REAL NOT NULL DEFAULT 10;
ALTER TABLE configuracion ADD COLUMN saldo_fondo_reserva REAL NOT NULL DEFAULT 0;

-- El tope introducido en v2 pertenece al Fondo de Reserva, no al de Incobrables.
-- Se renombra ahora que los tres fondos están separados y el nombre sería ambiguo.
ALTER TABLE configuracion RENAME COLUMN tope_reserva_incobrables_pct TO tope_fondo_reserva_pct;

-- ─────────────────────────────────────────────────────────────────────────────
-- D-12 · Ganancias: devengo mensual, pago anual (RN-10, RF-39 a RF-42)
--
-- Cada mes el sistema calcula el valor de ganancia por acción del mes y cada lote
-- de acciones activo DEVENGA `cantidad × valor_por_accion`. Ese valor se acumula mes
-- a mes y se PAGA cuando el lote cumple un año desde su compra.
--
-- `pago_ganancia` pasa a ser el renglón de devengo mensual por lote:
--   · `fecha_pago` debe admitir NULL — mientras la ganancia está devengada pero
--     todavía no pagada no existe fecha de pago (en v1 era NOT NULL).
--   · se guardan `acciones` y `valor_por_accion` como fotografía del mes, para que
--     el histórico no cambie si luego se corrige un cierre.
--   · UNIQUE(lote, reparto) impide devengar dos veces el mismo lote en el mismo mes.
--
-- SQLite no permite alterar la nulabilidad de una columna: se reconstruye la tabla.
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE pago_ganancia_v3 (
    id TEXT PRIMARY KEY,
    lote_acciones_id TEXT NOT NULL REFERENCES lote_acciones(id),
    reparto_ganancias_id TEXT NOT NULL REFERENCES reparto_ganancias(id),
    -- Fotografía del mes devengado.
    acciones INTEGER NOT NULL DEFAULT 0,
    valor_por_accion REAL NOT NULL DEFAULT 0,
    -- monto = acciones × valor_por_accion
    monto REAL NOT NULL DEFAULT 0,
    -- PENDIENTE = devengada, aún no pagada | PAGADA = liquidada al cumplir el año.
    estado TEXT NOT NULL DEFAULT 'PENDIENTE',
    fecha_pago TEXT,
    UNIQUE (lote_acciones_id, reparto_ganancias_id)
);

INSERT INTO pago_ganancia_v3
    (id, lote_acciones_id, reparto_ganancias_id, acciones, valor_por_accion, monto, estado, fecha_pago)
SELECT
    id,
    lote_acciones_id,
    reparto_ganancias_id,
    0,
    0,
    monto,
    estado,
    -- v1 guardaba cadena vacía cuando no había pago real; se normaliza a NULL.
    CASE WHEN fecha_pago IS NULL OR fecha_pago = '' THEN NULL ELSE fecha_pago END
FROM pago_ganancia;

DROP TABLE pago_ganancia;
ALTER TABLE pago_ganancia_v3 RENAME TO pago_ganancia;

CREATE INDEX IF NOT EXISTS idx_pago_ganancia_lote ON pago_ganancia(lote_acciones_id);
CREATE INDEX IF NOT EXISTS idx_pago_ganancia_estado ON pago_ganancia(estado);

-- ─────────────────────────────────────────────────────────────────────────────
-- D-02 · PPCFC (RN-09, RF-96)
--
-- Fórmula confirmada: colocación del mes = cartera ÷ (efectivo + cartera); el PPCFC
-- es el promedio de esa razón en los últimos 3 meses cerrados. Se guarda la razón de
-- cada cierre para no tener que recalcularla —y para que un cierre corregido no
-- reescriba en silencio la historia del PPCFC.
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE cierre_mes ADD COLUMN colocacion_pct REAL NOT NULL DEFAULT 0;
ALTER TABLE cierre_mes ADD COLUMN ppcfc_pct REAL NOT NULL DEFAULT 0;
