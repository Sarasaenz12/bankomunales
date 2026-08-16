-- Migración v5 del `.db` de un Bankomunal.
--
-- Correcciones derivadas de los documentos originales que entregó el cliente
-- (reglamento fijo, planilla de registro del socio, formato de acciones autorizadas):
--   · D-14: los % de PPCFC guardaban umbrales (80/90) donde van los % autorizados
--           a vender (10/15).
--   · D-15: la planilla de registro pide teléfono del protegido y NO pide parentesco
--           del beneficiario.
--
-- NUNCA se edita una migración ya publicada: los cambios entran como un script nuevo.

-- ─────────────────────────────────────────────────────────────────────────────
-- D-14 · PPCFC: umbrales fijos vs. % autorizado a vender (RN-09)
--
-- El formato "Reporte de acciones autorizadas para la venta" deja claro que hay dos
-- cosas distintas:
--   · Los UMBRALES (80%, 90%, 100%) son parte del reglamento FIJO de la metodología
--     Bankomunales — no los cambia cada Bankomunal, así que viven como constantes
--     de dominio, no en la tabla.
--   · Los % AUTORIZADOS A VENDER en cada tramo (10% y 15% del total de acciones) son
--     lo que sí se configura, y son los que la pantalla ya estaba editando.
--
-- Las columnas guardaban 80 y 90 —los umbrales— mientras la pantalla las presentaba
-- como el % a vender. Se renombran para que el nombre diga lo que contienen y se
-- corrigen los valores sembrados por defecto.
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE configuracion RENAME COLUMN ppcfc_rango1_pct TO ppcfc_venta_rango1_pct;
ALTER TABLE configuracion RENAME COLUMN ppcfc_rango2_pct TO ppcfc_venta_rango2_pct;

-- Sólo se corrigen las filas que aún tienen el valor sembrado erróneo; si alguien ya
-- configuró un valor propio, se respeta.
UPDATE configuracion SET ppcfc_venta_rango1_pct = 10 WHERE ppcfc_venta_rango1_pct = 80;
UPDATE configuracion SET ppcfc_venta_rango2_pct = 15 WHERE ppcfc_venta_rango2_pct = 90;

-- ─────────────────────────────────────────────────────────────────────────────
-- D-15 · Datos del socio según la PLANILLA DE REGISTRO DEL SOCIO
--
-- La planilla original pide, para cada uno de los hasta 2 protegidos del fondo de
-- protección: nombres y apellidos, cédula, parentesco Y TELÉFONO. Faltaba el teléfono.
--
-- Para el beneficiario en caso de muerte la planilla sólo dice: "Declaro que en caso
-- de muerte cedo mis acciones a ____ identificado con cédula ____". No pregunta
-- parentesco, así que la columna pasa a ser opcional (era NOT NULL).
-- ─────────────────────────────────────────────────────────────────────────────
ALTER TABLE protegido ADD COLUMN telefono TEXT NOT NULL DEFAULT '';

-- SQLite no permite quitar un NOT NULL: se reconstruye la tabla.
CREATE TABLE beneficiario_v5 (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    nombre TEXT NOT NULL,
    cedula TEXT NOT NULL,
    parentesco TEXT
);

INSERT INTO beneficiario_v5 (id, socio_id, nombre, cedula, parentesco)
SELECT id, socio_id, nombre, cedula, NULLIF(parentesco, '') FROM beneficiario;

DROP TABLE beneficiario;
ALTER TABLE beneficiario_v5 RENAME TO beneficiario;

-- Índices de apoyo al módulo de Socios (búsqueda por cédula, RF-19).
CREATE INDEX IF NOT EXISTS idx_socio_cedula ON socio(cedula);
CREATE INDEX IF NOT EXISTS idx_socio_estatus ON socio(estatus);
CREATE INDEX IF NOT EXISTS idx_beneficiario_socio ON beneficiario(socio_id);
CREATE INDEX IF NOT EXISTS idx_protegido_socio ON protegido(socio_id);
