-- App-level (app.db): registros de configuración global y catálogo de Bankomunales.
CREATE TABLE IF NOT EXISTS app_setting (
    clave TEXT PRIMARY KEY,
    valor TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS catalogo_bankomunal (
    id TEXT PRIMARY KEY,
    nombre TEXT NOT NULL,
    ubicacion TEXT NOT NULL DEFAULT '',
    moneda TEXT NOT NULL DEFAULT 'COP',
    fecha_creacion TEXT NOT NULL,
    db_path TEXT NOT NULL
);