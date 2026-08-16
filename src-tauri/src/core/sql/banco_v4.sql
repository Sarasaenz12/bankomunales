-- Migración v4 del `.db` de un Bankomunal.
--
--   · D-11 (corregida): los fondos son DOS, no tres. Revierte el Fondo de Reserva
--     que v3 había separado del de Incobrables — son el mismo fondo.
--   · D-13: la ganancia del año en curso no consolidada no se paga al liquidar
--     anticipadamente, pero tampoco se pierde: queda como ganancia colectiva.
--
-- NUNCA se edita una migración ya publicada: los cambios entran como un script nuevo.

-- ─────────────────────────────────────────────────────────────────────────────
-- D-11 corregida · Vuelven a ser dos fondos
--
-- RN-07 → Fondo para Gastos: cubre los gastos operativos del Bankomunal
--         (fotocopias, transporte, papelería). Se llena con su % mensual.
-- RN-08 → Fondo de Reserva para Incobrables: colchón para cuando un socio se retira
--         debiendo más de lo que valen sus acciones. Se llena con su % mensual y deja
--         de crecer al llegar a su tope (% del capital total en acciones).
--
-- v3 los había separado en tres por una respuesta previa del cliente; su explicación
-- detallada del 2026-08-12, con ejemplo numérico, confirma que "Fondo de Reserva" y
-- "Fondo de Incobrables" son dos nombres del mismo fondo.
-- ─────────────────────────────────────────────────────────────────────────────

-- Defensivo: si el fondo separado de v3 llegó a acumular saldo, se consolida en el
-- de Incobrables antes de eliminar la columna, para no perder dinero registrado.
UPDATE configuracion
   SET saldo_fondo_incobrables = COALESCE(saldo_fondo_incobrables, 0)
                               + COALESCE(saldo_fondo_reserva, 0)
 WHERE COALESCE(saldo_fondo_reserva, 0) <> 0;

ALTER TABLE configuracion DROP COLUMN pct_fondo_reserva;
ALTER TABLE configuracion DROP COLUMN saldo_fondo_reserva;
ALTER TABLE configuracion RENAME COLUMN tope_fondo_reserva_pct TO tope_reserva_incobrables_pct;

-- ─────────────────────────────────────────────────────────────────────────────
-- D-13 · Ganancia no consolidada al liquidar antes del año (RN-10)
--
-- El reglamento dice: "Las ganancias de las acciones se repartirán al año de vencida
-- cada acción". Es una regla de todo o nada, como un CDT: si el lote no cumple su año,
-- no genera derecho a la ganancia de ese año en curso.
--
-- Al liquidar un lote antes de su aniversario, sus renglones devengados del año en
-- curso pasan a estado LIBERADA: no se le pagan al socio que se retira, pero tampoco
-- desaparecen del sistema —el dinero ya está físicamente en caja— sino que quedan como
-- ganancia colectiva del Bankomunal.
--
-- Estados de `pago_ganancia`:
--   PENDIENTE → devengada, el lote sigue activo y aún no cumple el año.
--   PAGADA    → el lote cumplió el año y la ganancia se entregó al socio.
--   LIBERADA  → el lote se liquidó antes de cumplir el año; queda para el colectivo.
--
-- "Ganancias no repartidas" del Pasivo = suma de PENDIENTE + LIBERADA.
-- `fecha_pago` sólo se llena en PAGADA; en LIBERADA queda NULL y se registra en su
-- lugar la fecha en que el lote fue liquidado.
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE pago_ganancia ADD COLUMN fecha_liberacion TEXT;

CREATE INDEX IF NOT EXISTS idx_pago_ganancia_reparto ON pago_ganancia(reparto_ganancias_id);

-- Fecha en que el lote fue liquidado (RF-28: parcial o total). Permite decidir qué
-- renglones devengados se liberan y comprobar si el lote alcanzó su aniversario.
ALTER TABLE lote_acciones ADD COLUMN fecha_liquidacion TEXT;
