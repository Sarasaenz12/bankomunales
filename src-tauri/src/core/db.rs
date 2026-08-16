use rusqlite::Connection;
use std::path::PathBuf;

use super::error::AppError;

/// Migraciones del `app.db`, en orden. El índice + 1 es el `user_version` que deja aplicada.
const APP_MIGRATIONS: &[&str] = &[include_str!("sql/app_v1.sql")];

/// Migraciones del `.db` de un Bankomunal, en orden.
/// Nunca se edita un script ya publicado: los cambios de esquema entran como uno nuevo
/// al final de la lista (ver docs/decisiones-pendientes.md).
const BANCO_MIGRATIONS: &[&str] = &[
    include_str!("sql/banco_v1.sql"),
    include_str!("sql/banco_v2.sql"),
    include_str!("sql/banco_v3.sql"),
    include_str!("sql/banco_v4.sql"),
    include_str!("sql/banco_v5.sql"),
    include_str!("sql/banco_v6.sql"),
];

/// Gestiona las rutas y la apertura de los archivos `.db`.
/// Existe un `app.db` (nivel aplicación, catálogo de Bankomunales + config de acceso)
/// y un archivo `.db` independiente por cada Bankomunal (aislamiento por archivo, ADR-03).
/// Nunca se hacen operaciones directas de negocio aquí: este módulo es infraestructura.
#[derive(Clone)]
pub struct DbManager {
    app_data_dir: PathBuf,
}

impl DbManager {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self { app_data_dir }
    }

    /// Ruta del `app.db` (catálogo global de Bankomunales).
    pub fn app_db_path(&self) -> PathBuf {
        self.app_data_dir.join("app.db")
    }

    /// Ruta del `.db` independiente de un Bankomunal dado.
    pub fn banco_db_path(&self, banco_id: &str) -> PathBuf {
        let dir = self.app_data_dir.join("bankomunales");
        dir.join(format!("{banco_id}.db"))
    }

    /// Abre (creando si no existe) el `app.db` ejecutando las migraciones pendientes.
    pub fn open_app_db(&self) -> Result<Connection, AppError> {
        if let Some(parent) = self.app_db_path().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(self.app_db_path())?;
        self.migrate(&mut conn, APP_MIGRATIONS)?;
        Ok(conn)
    }

    /// Abre (creando si no existe) el `.db` de un Bankomunal ejecutando sus migraciones.
    pub fn open_banco_db(&self, banco_id: &str) -> Result<Connection, AppError> {
        let path = self.banco_db_path(banco_id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        self.migrate(&mut conn, BANCO_MIGRATIONS)?;
        Ok(conn)
    }

    /// Aplica en orden las migraciones que le falten a la base: lee `PRAGMA user_version`
    /// y ejecuta sólo los scripts posteriores, avanzando la versión uno a uno.
    ///
    /// Cada script corre dentro de una transacción: si falla a la mitad, la base queda
    /// en su versión anterior íntegra en vez de a medio migrar. Por eso un script ya
    /// publicado nunca se edita — sólo se agrega uno nuevo al final de la lista.
    fn migrate(&self, conn: &mut Connection, migrations: &[&str]) -> Result<(), AppError> {
        let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

        for (i, script) in migrations.iter().enumerate() {
            let version = i as i64 + 1;
            if version <= current {
                continue;
            }
            let tx = conn.transaction()?;
            tx.execute_batch(script)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("bkn_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn app_db_creates_tables_in_memory_of_path() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let conn = mgr.open_app_db().unwrap();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, 1);
        // Tablas del catálogo y settings existen.
        let t: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('app_setting','catalogo_bankomunal')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(t, 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn banco_db_creates_full_dictionary_schema() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let conn = mgr.open_banco_db(&id).unwrap();
        let t: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN
                 ('socio','beneficiario','protegido','lote_acciones','reparto_ganancias',
                  'pago_ganancia','credito','garantia_credito','cuota','movimiento_libro',
                  'cierre_mes','respaldo','auditoria','configuracion',
                  'solicitud_credito','garantia_solicitud','bien')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(t, 17);
        // Campos agregados al diccionario según decisiones del cliente.
        let cuota_pago: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('cuota') WHERE name='fecha_pago'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cuota_pago, 1);
        let socio_extra: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('socio') WHERE name IN ('fecha_ingreso','fecha_retiro','saldo_incobrable')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(socio_extra, 3);

        // La base queda en la última versión de migración disponible.
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, BANCO_MIGRATIONS.len() as i64);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// v2 cierra los vacíos del diccionario detectados contra los RF (D-04 a D-08).
    #[test]
    fn banco_db_v2_agrega_columnas_faltantes() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let conn = mgr.open_banco_db(&id).unwrap();

        let tiene = |tabla: &str, columna: &str| -> bool {
            let n: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM pragma_table_info('{tabla}') WHERE name = ?1"),
                    [columna],
                    |r| r.get(0),
                )
                .unwrap();
            n == 1
        };

        // RN-08: retención mensual y tope de la reserva son columnas distintas.
        assert!(tiene("configuracion", "pct_fondo_incobrables"));
        assert!(tiene("configuracion", "tope_reserva_incobrables_pct"));
        // RF-60: frecuencia de pago y fecha de vencimiento del crédito.
        assert!(tiene("credito", "frecuencia_pago"));
        assert!(tiene("credito", "fecha_vencimiento"));
        // RF-65/RF-72: abonos parciales sobre la cuota.
        assert!(tiene("cuota", "capital_pagado"));
        assert!(tiene("cuota", "interes_pagado"));
        assert!(tiene("cuota", "mora_pagada"));
        // RF-90/RF-97: correcciones posteriores al cierre y mes al que pertenece el asiento.
        assert!(tiene("movimiento_libro", "cierre_mes_id"));
        assert!(tiene("movimiento_libro", "corregido"));
        assert!(tiene("movimiento_libro", "motivo_correccion"));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// v3 pasa el devengo de ganancias a mensual (D-12) y guarda la colocación de cada
    /// cierre para el PPCFC (D-02); v4 deja los fondos en dos (D-11 corregida) y añade
    /// lo necesario para la ganancia no consolidada (D-13).
    #[test]
    fn banco_db_v3_devengo_mensual_y_v4_dos_fondos() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let conn = mgr.open_banco_db(&id).unwrap();

        let tiene = |tabla: &str, columna: &str| -> bool {
            let n: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM pragma_table_info('{tabla}') WHERE name = ?1"),
                    [columna],
                    |r| r.get(0),
                )
                .unwrap();
            n == 1
        };

        // D-11 corregida: dos fondos, cada uno con su % y su saldo, más el tope de la
        // Reserva para Incobrables.
        for col in [
            "pct_fondo_gastos",
            "saldo_fondo_gastos",
            "pct_fondo_incobrables",
            "saldo_fondo_incobrables",
            "tope_reserva_incobrables_pct",
        ] {
            assert!(tiene("configuracion", col), "falta configuracion.{col}");
        }
        // El tercer fondo que introdujo v3 quedó revertido en v4.
        assert!(!tiene("configuracion", "pct_fondo_reserva"));
        assert!(!tiene("configuracion", "saldo_fondo_reserva"));
        assert!(!tiene("configuracion", "tope_fondo_reserva_pct"));

        // D-12: el devengo mensual guarda la fotografía del mes.
        assert!(tiene("pago_ganancia", "acciones"));
        assert!(tiene("pago_ganancia", "valor_por_accion"));

        // D-13: ganancia liberada al liquidar antes del año.
        assert!(tiene("pago_ganancia", "fecha_liberacion"));
        assert!(tiene("lote_acciones", "fecha_liquidacion"));

        // D-02: colocación del mes y PPCFC quedan sellados en cada cierre.
        assert!(tiene("cierre_mes", "colocacion_pct"));
        assert!(tiene("cierre_mes", "ppcfc_pct"));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// D-11 corregida: al revertir el tercer fondo, un saldo que hubiera quedado en el
    /// Fondo de Reserva de v3 se consolida en el de Incobrables — no se pierde dinero.
    #[test]
    fn migracion_v3_a_v4_consolida_el_saldo_del_fondo_revertido() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let path = mgr.banco_db_path(&id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Instalación con v3 aplicada y saldo repartido entre los dos fondos separados.
        {
            let mut conn = Connection::open(&path).unwrap();
            mgr.migrate(&mut conn, &BANCO_MIGRATIONS[..3]).unwrap();
            conn.execute(
                "INSERT INTO configuracion (id, nombre, fecha_creacion,
                     saldo_fondo_reserva, saldo_fondo_incobrables)
                 VALUES (?1, 'Pijao', '2026-01-01', 180000, 120000)",
                [&id],
            )
            .unwrap();
        }

        let conn = mgr.open_banco_db(&id).unwrap();
        assert_eq!(
            conn.query_row::<i64, _, _>("PRAGMA user_version", [], |r| r.get(0))
                .unwrap(),
            BANCO_MIGRATIONS.len() as i64
        );

        let saldo: f64 = conn
            .query_row(
                "SELECT saldo_fondo_incobrables FROM configuracion WHERE id = ?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(saldo, 300_000.0, "los dos saldos deben consolidarse en uno");

        std::fs::remove_dir_all(&dir).ok();
    }

    /// D-12: `pago_ganancia.fecha_pago` pasa a admitir NULL (ganancia devengada pero
    /// aún no pagada) sin perder las filas que ya existieran.
    #[test]
    fn migracion_v2_a_v3_reconstruye_pago_ganancia_sin_perder_filas() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let path = mgr.banco_db_path(&id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Instalación existente: sólo v1 y v2 aplicadas, con un devengo ya registrado.
        {
            let mut conn = Connection::open(&path).unwrap();
            mgr.migrate(&mut conn, &BANCO_MIGRATIONS[..2]).unwrap();
            conn.execute_batch(
                "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                                    telefono, celular, correo, estatus, fecha_ingreso)
                 VALUES ('s1','123','Ana','Ríos','Docente','Calle 1','','','','ACTIVO','2026-01-01');
                 INSERT INTO lote_acciones (id, socio_id, mes_compra, cantidad, liquidada)
                 VALUES ('l1','s1','2026-01-01',5,0);
                 INSERT INTO reparto_ganancias (id, cierre_mes_id, mes) VALUES ('r1','c1','2026-01-01');
                 INSERT INTO pago_ganancia (id, lote_acciones_id, reparto_ganancias_id, monto, estado, fecha_pago)
                 VALUES ('p1','l1','r1', 5000, 'PENDIENTE', '');",
            )
            .unwrap();
        }

        let conn = mgr.open_banco_db(&id).unwrap();
        assert_eq!(
            conn.query_row::<i64, _, _>("PRAGMA user_version", [], |r| r.get(0))
                .unwrap(),
            BANCO_MIGRATIONS.len() as i64
        );

        let (monto, estado, fecha): (f64, String, Option<String>) = conn
            .query_row(
                "SELECT monto, estado, fecha_pago FROM pago_ganancia WHERE id = 'p1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(monto, 5000.0);
        assert_eq!(estado, "PENDIENTE");
        assert!(fecha.is_none(), "la fecha vacía debe normalizarse a NULL");

        // Un devengo nuevo puede insertarse sin fecha de pago.
        conn.execute(
            "INSERT INTO reparto_ganancias (id, cierre_mes_id, mes) VALUES ('r2','c2','2026-02-01')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pago_ganancia
             (id, lote_acciones_id, reparto_ganancias_id, acciones, valor_por_accion, monto)
             VALUES ('p2','l1','r2', 5, 1000, 5000)",
            [],
        )
        .unwrap();

        // Y no se puede devengar dos veces el mismo lote en el mismo mes.
        let duplicado = conn.execute(
            "INSERT INTO pago_ganancia
             (id, lote_acciones_id, reparto_ganancias_id, acciones, valor_por_accion, monto)
             VALUES ('p3','l1','r2', 5, 1000, 5000)",
            [],
        );
        assert!(duplicado.is_err(), "UNIQUE(lote, reparto) debe impedir el duplicado");

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Una base creada con el esquema v1 debe poder migrar hasta la última versión
    /// sin perder datos, saltando todas las migraciones intermedias de una vez.
    #[test]
    fn migracion_desde_v1_conserva_los_datos() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        let path = mgr.banco_db_path(&id);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();

        // Simula una instalación existente: sólo v1 aplicada, con un socio ya registrado.
        {
            let mut conn = Connection::open(&path).unwrap();
            mgr.migrate(&mut conn, &BANCO_MIGRATIONS[..1]).unwrap();
            conn.execute(
                "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                                    telefono, celular, correo, estatus, fecha_ingreso)
                 VALUES ('s1','123','Ana','Ríos','Docente','Calle 1','','','','ACTIVO','2026-01-01')",
                [],
            )
            .unwrap();
        }

        // Al reabrir, se aplican en orden todas las migraciones pendientes.
        let conn = mgr.open_banco_db(&id).unwrap();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, BANCO_MIGRATIONS.len() as i64);

        let nombres: String = conn
            .query_row("SELECT nombres FROM socio WHERE id = 's1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(nombres, "Ana", "la migración no debe perder datos existentes");

        let tope: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('configuracion')
                 WHERE name = 'tope_reserva_incobrables_pct'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tope, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn migrations_are_idempotent() {
        let dir = temp_dir();
        let mgr = DbManager::new(dir.clone());
        let id = Uuid::new_v4().to_string();
        // Abrir dos veces el mismo .db no debe duplicar ni fallar.
        mgr.open_banco_db(&id).unwrap();
        mgr.open_banco_db(&id).unwrap();
        let count: i64 = mgr
            .open_banco_db(&id)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM socio", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        std::fs::remove_dir_all(&dir).ok();
    }
}