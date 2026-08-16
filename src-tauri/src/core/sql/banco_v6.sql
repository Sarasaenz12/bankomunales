-- Migración v6 del `.db` de un Bankomunal: Compra de Acciones (RF-22 a RF-27).
--
-- NUNCA se edita una migración ya publicada: los cambios entran como un script nuevo.

-- ─────────────────────────────────────────────────────────────────────────────
-- El lote guarda el valor nominal con el que se compró, no sólo la cantidad.
--
-- RN-13 permite cambiar el valor nominal de la acción cuando la asamblea lo decida.
-- Si el lote sólo guardara la cantidad, al liquidar habría que multiplicar por el
-- nominal *actual* y el socio recibiría más —o menos— de lo que efectivamente puso.
-- Guardar el nominal de la compra congela el capital aportado (D-13: "se le devuelve
-- el valor nominal invertido").
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE lote_acciones ADD COLUMN valor_nominal_compra REAL NOT NULL DEFAULT 0;
ALTER TABLE lote_acciones ADD COLUMN monto_pagado REAL NOT NULL DEFAULT 0;

-- `mes_compra` (v1) es el insumo de RN-10 para el aniversario; la fecha exacta hace
-- falta para el asiento del Libro y para el orden dentro del mes.
ALTER TABLE lote_acciones ADD COLUMN fecha_compra TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS idx_lote_acciones_mes ON lote_acciones(mes_compra);
