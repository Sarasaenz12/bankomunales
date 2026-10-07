-- Migración v7 del `.db` de un Bankomunal: Cierre de Mes (RF-91 a RF-97).
--
--   · D-20: el Balance de Gestión Mensual Neto resta los Otros Egresos (código EG)
--     además de los dos fondos. La columna queda guardada en `reparto_ganancias`
--     para que el histórico diga cuánto se restó en cada mes, igual que se guardan
--     los ingresos repartibles y el balance neto.
--
-- NUNCA se edita una migración ya publicada: los cambios entran como un script nuevo.

ALTER TABLE reparto_ganancias ADD COLUMN otros_egresos REAL NOT NULL DEFAULT 0;