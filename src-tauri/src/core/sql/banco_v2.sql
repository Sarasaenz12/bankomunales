-- Migración v2 del `.db` de un Bankomunal.
-- Cierra los vacíos del diccionario de datos detectados al contrastar el esquema v1
-- contra los RF (ver docs/decisiones-pendientes.md, D-04 a D-08).
--
-- NUNCA se edita banco_v1.sql: cada cambio de esquema entra como una migración nueva.

-- ── D-04 (RN-08): el tope de la Reserva acumulada es un parámetro distinto del
-- % de retención mensual. `pct_fondo_incobrables` es cuánto se retiene cada mes;
-- `tope_reserva_incobrables_pct` es hasta dónde puede crecer la reserva acumulada,
-- expresado como % del capital en acciones.
ALTER TABLE configuracion ADD COLUMN tope_reserva_incobrables_pct REAL NOT NULL DEFAULT 20;

-- ── D-06 (RF-60): un crédito debe registrar su frecuencia de pago y su fecha de
-- vencimiento; además queda enlazado a la solicitud que lo originó (si la hubo).
ALTER TABLE credito ADD COLUMN frecuencia_pago TEXT NOT NULL DEFAULT 'MENSUAL';
ALTER TABLE credito ADD COLUMN fecha_vencimiento TEXT NOT NULL DEFAULT '';
ALTER TABLE credito ADD COLUMN solicitud_id TEXT;

-- ── D-06 (RF-65, RF-68, RF-72): las cuotas admiten abonos parciales. `capital`,
-- `interes` y `valor_total` son lo *calculado*; estas columnas son lo *efectivamente
-- pagado*, que puede diferir. `pagada` pasa a ser un derivado de que lo pagado
-- cubra lo calculado.
ALTER TABLE cuota ADD COLUMN capital_pagado REAL NOT NULL DEFAULT 0;
ALTER TABLE cuota ADD COLUMN interes_pagado REAL NOT NULL DEFAULT 0;
ALTER TABLE cuota ADD COLUMN mora_pagada REAL NOT NULL DEFAULT 0;

-- ── D-07 (RF-89, RF-90, RF-94, RF-97): el Libro de Ingresos y Egresos necesita saber
-- si su mes ya fue cerrado y si el asiento fue corregido después del cierre.
-- `cierre_mes_id` NULL = el movimiento pertenece a un mes todavía abierto.
ALTER TABLE movimiento_libro ADD COLUMN cierre_mes_id TEXT;
ALTER TABLE movimiento_libro ADD COLUMN corregido INTEGER NOT NULL DEFAULT 0;
ALTER TABLE movimiento_libro ADD COLUMN corregido_por TEXT;
ALTER TABLE movimiento_libro ADD COLUMN fecha_correccion TEXT;
ALTER TABLE movimiento_libro ADD COLUMN motivo_correccion TEXT;

-- ── D-05 (RF-43, RF-45, RF-48, RF-50, RF-51): la Solicitud de Crédito se persiste.
-- Es una entidad propia porque una solicitud puede vivir sin crédito (Negada, Diferida)
-- y las Diferidas deben permanecer visibles hasta resolverse (RF-51).
CREATE TABLE IF NOT EXISTS solicitud_credito (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    fecha_solicitud TEXT NOT NULL,
    monto_solicitado REAL NOT NULL DEFAULT 0,
    plazo_cuotas INTEGER NOT NULL DEFAULT 0,
    -- Clase de crédito / destino del dinero (RN-11).
    destino TEXT NOT NULL,
    -- Capacidad de pago (RF-45): capacidad_pago = total_ingresos - total_egresos.
    total_ingresos REAL NOT NULL DEFAULT 0,
    total_egresos REAL NOT NULL DEFAULT 0,
    capacidad_pago REAL NOT NULL DEFAULT 0,
    -- PENDIENTE | APROBADA | MODIFICADA | NEGADA | DIFERIDA (RF-50).
    estado TEXT NOT NULL DEFAULT 'PENDIENTE',
    -- Monto que aprobó la Junta; difiere del solicitado cuando el estado es MODIFICADA.
    monto_aprobado REAL,
    -- Observación obligatoria cuando el estado es DIFERIDA (RF-51).
    observacion TEXT,
    fecha_decision TEXT,
    decidida_por TEXT
);

-- Fiadores propuestos en la solicitud (RF-48: hasta 2 fiadores + el titular).
-- Se mantiene separada de `garantia_credito` porque una solicitud Negada nunca
-- llega a producir un crédito, y las garantías definitivas se fijan al desembolsar.
CREATE TABLE IF NOT EXISTS garantia_solicitud (
    id TEXT PRIMARY KEY,
    solicitud_id TEXT NOT NULL REFERENCES solicitud_credito(id),
    socio_id TEXT NOT NULL REFERENCES socio(id),
    -- TITULAR | FIADOR
    rol TEXT NOT NULL,
    acciones_comprometidas REAL NOT NULL DEFAULT 0
);

-- ── D-08 (RF-88): Bienes Adquiridos como Activo Fijo. No afectan el saldo de caja,
-- pero sí alimentan la columna `bienes` del Balance del Mes (cierre_mes.bienes).
CREATE TABLE IF NOT EXISTS bien (
    id TEXT PRIMARY KEY,
    descripcion TEXT NOT NULL,
    fecha_adquisicion TEXT NOT NULL,
    valor REAL NOT NULL DEFAULT 0,
    -- PROPIO | COMODATO
    tipo TEXT NOT NULL DEFAULT 'PROPIO'
);

-- Índices de apoyo a las consultas más frecuentes de reportes y cierre.
CREATE INDEX IF NOT EXISTS idx_movimiento_libro_fecha ON movimiento_libro(fecha);
CREATE INDEX IF NOT EXISTS idx_movimiento_libro_cierre ON movimiento_libro(cierre_mes_id);
CREATE INDEX IF NOT EXISTS idx_cuota_credito ON cuota(credito_id);
CREATE INDEX IF NOT EXISTS idx_lote_acciones_socio ON lote_acciones(socio_id);
CREATE INDEX IF NOT EXISTS idx_credito_socio ON credito(socio_id);
CREATE INDEX IF NOT EXISTS idx_solicitud_estado ON solicitud_credito(estado);
