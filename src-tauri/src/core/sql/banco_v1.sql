-- Esquema de un Bankomunal (su propio archivo .db): todas las tablas del diccionario.
-- Versión 1. Incluye los campos agregados a socios/cuota y la tabla de configuración.

CREATE TABLE IF NOT EXISTS configuracion (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    ubicacion TEXT NOT NULL DEFAULT '',
    moneda TEXT NOT NULL DEFAULT 'COP',
    fecha_creacion TEXT NOT NULL,
    valor_nominal REAL NOT NULL DEFAULT 0,
    pct_garantia_socio REAL NOT NULL DEFAULT 20,
    pct_garantia_fiador REAL NOT NULL DEFAULT 20,
    pct_fondo_gastos REAL NOT NULL DEFAULT 10,
    saldo_fondo_gastos REAL NOT NULL DEFAULT 0,
    pct_fondo_incobrables REAL NOT NULL DEFAULT 10,
    saldo_fondo_incobrables REAL NOT NULL DEFAULT 0,
    ppcfc_rango1_pct REAL NOT NULL DEFAULT 80,
    ppcfc_rango2_pct REAL NOT NULL DEFAULT 90,
    tope_individual_mensual_pct REAL NOT NULL DEFAULT 20,
    tasa_interes_ordinario REAL NOT NULL DEFAULT 0,
    tasa_interes_mora REAL NOT NULL DEFAULT 0,
    plazo_maximo INTEGER NOT NULL DEFAULT 0,
    monto_maximo_credito REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS socio (
    id TEXT PRIMARY KEY,
    cedula TEXT NOT NULL UNIQUE,
    nombres TEXT NOT NULL,
    apellidos TEXT NOT NULL,
    profesion TEXT NOT NULL,
    direccion TEXT NOT NULL,
    telefono TEXT NOT NULL,
    celular TEXT NOT NULL,
    correo TEXT NOT NULL,
    estatus TEXT NOT NULL,
    fecha_ingreso TEXT NOT NULL,
    fecha_retiro TEXT,
    saldo_incobrable REAL
);

CREATE TABLE IF NOT EXISTS beneficiario (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    nombre TEXT NOT NULL,
    cedula TEXT NOT NULL,
    parentesco TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS protegido (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    nombre TEXT NOT NULL,
    cedula TEXT NOT NULL,
    parentesco TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lote_acciones (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    mes_compra TEXT NOT NULL,
    cantidad INTEGER NOT NULL,
    liquidada INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS reparto_ganancias (
    id TEXT PRIMARY KEY,
    cierre_mes_id TEXT NOT NULL,
    mes TEXT NOT NULL,
    total_ingresos_repartibles REAL NOT NULL DEFAULT 0,
    fondo_incobrables REAL NOT NULL DEFAULT 0,
    fondo_gastos REAL NOT NULL DEFAULT 0,
    balance_neto REAL NOT NULL DEFAULT 0,
    acciones_activas INTEGER NOT NULL DEFAULT 0,
    valor_por_accion REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS pago_ganancia (
    id TEXT PRIMARY KEY,
    lote_acciones_id TEXT NOT NULL REFERENCES lote_acciones(id),
    reparto_ganancias_id TEXT NOT NULL REFERENCES reparto_ganancias(id),
    monto REAL NOT NULL DEFAULT 0,
    estado TEXT NOT NULL DEFAULT 'PENDIENTE',
    fecha_pago TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS credito (
    id TEXT PRIMARY KEY,
    socio_id TEXT NOT NULL REFERENCES socio(id),
    numero TEXT NOT NULL UNIQUE,
    monto_original REAL NOT NULL DEFAULT 0,
    tasa REAL NOT NULL DEFAULT 0,
    plazo_cuotas INTEGER NOT NULL DEFAULT 0,
    cuota_actual INTEGER NOT NULL DEFAULT 1,
    saldo_pendiente REAL NOT NULL DEFAULT 0,
    destino TEXT NOT NULL,
    estatus TEXT NOT NULL,
    fecha_solicitud TEXT NOT NULL,
    fecha_desembolso TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS garantia_credito (
    id TEXT PRIMARY KEY,
    credito_id TEXT NOT NULL REFERENCES credito(id),
    socio_id TEXT NOT NULL REFERENCES socio(id),
    rol TEXT NOT NULL,
    acciones_comprometidas REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS cuota (
    id TEXT PRIMARY KEY,
    credito_id TEXT NOT NULL REFERENCES credito(id),
    numero INTEGER NOT NULL,
    fecha_vencimiento TEXT NOT NULL,
    fecha_pago TEXT,
    capital REAL NOT NULL DEFAULT 0,
    interes REAL NOT NULL DEFAULT 0,
    valor_total REAL NOT NULL DEFAULT 0,
    pagada INTEGER NOT NULL DEFAULT 0,
    dias_atraso INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS movimiento_libro (
    id TEXT PRIMARY KEY,
    socio_id TEXT,
    credito_id TEXT,
    numero INTEGER NOT NULL,
    fecha TEXT NOT NULL,
    codigo TEXT NOT NULL,
    descripcion TEXT NOT NULL,
    ingreso REAL NOT NULL DEFAULT 0,
    egreso REAL NOT NULL DEFAULT 0,
    saldo REAL NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS cierre_mes (
    id TEXT PRIMARY KEY,
    mes TEXT NOT NULL,
    efectivo REAL NOT NULL DEFAULT 0,
    cartera REAL NOT NULL DEFAULT 0,
    bienes REAL NOT NULL DEFAULT 0,
    activo_total REAL NOT NULL DEFAULT 0,
    pasivo_total REAL NOT NULL DEFAULT 0,
    cuadra INTEGER NOT NULL DEFAULT 0,
    fecha_cierre TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS respaldo (
    id TEXT PRIMARY KEY,
    fecha TEXT NOT NULL,
    usuario TEXT NOT NULL,
    ruta_archivo TEXT NOT NULL,
    tipo TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS auditoria (
    id TEXT PRIMARY KEY,
    fecha TEXT NOT NULL,
    nombre_quien_realiza TEXT NOT NULL,
    entidad_afectada TEXT NOT NULL,
    campo_modificado TEXT,
    valor_anterior TEXT,
    valor_nuevo TEXT,
    motivo TEXT,
    tipo_accion TEXT NOT NULL
);