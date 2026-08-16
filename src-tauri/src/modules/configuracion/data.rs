use chrono::Utc;
use rusqlite::{params, Connection};

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::configuracion::domain::{Configuracion, ConfiguracionPort, DatosGenerales};

fn row_to_configuracion(row: &rusqlite::Row) -> rusqlite::Result<Configuracion> {
    Ok(Configuracion {
        valor_nominal: row.get(5)?,
        pct_garantia_socio: row.get(6)?,
        pct_garantia_fiador: row.get(7)?,
        pct_fondo_gastos: row.get(8)?,
        pct_fondo_incobrables: row.get(9)?,
        tope_reserva_incobrables_pct: row.get(10)?,
        ppcfc_venta_rango1_pct: row.get(11)?,
        ppcfc_venta_rango2_pct: row.get(12)?,
        tope_individual_mensual_pct: row.get(13)?,
        plazo_maximo_cuotas: row.get(14)?,
        tasa_interes_ordinario: row.get(15)?,
        tasa_interes_mora: row.get(16)?,
        monto_maximo_credito: row.get(17)?,
    })
}

/// Adaptador SQLite de la configuración de un Bankomunal, operando sobre su propio
/// archivo `.db` (aísla datos por archivo, RF-08).
pub struct SqliteConfiguracion {
    db: DbManager,
}

impl SqliteConfiguracion {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl ConfiguracionPort for SqliteConfiguracion {
    fn obtener(&self, banco_id: &str) -> Result<Configuracion, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        match Self::leer(&conn, banco_id)? {
            Some(c) => Ok(c),
            None => {
                let def = Configuracion::default();
                let identidad = self.identidad_del_catalogo(banco_id)?;
                conn.execute(
                    "INSERT INTO configuracion
                     (id, nombre, ubicacion, moneda, fecha_creacion,
                      valor_nominal, pct_garantia_socio, pct_garantia_fiador,
                      pct_fondo_gastos, pct_fondo_incobrables,
                      tope_reserva_incobrables_pct, ppcfc_venta_rango1_pct,
                      ppcfc_venta_rango2_pct, tope_individual_mensual_pct,
                      plazo_maximo, tasa_interes_ordinario, tasa_interes_mora,
                      monto_maximo_credito)
                     VALUES (?1, ?2, ?3, ?4, ?5,
                             ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                    params![
                        banco_id,
                        identidad.nombre,
                        identidad.ubicacion,
                        identidad.moneda,
                        identidad.fecha_creacion,
                        def.valor_nominal,
                        def.pct_garantia_socio,
                        def.pct_garantia_fiador,
                        def.pct_fondo_gastos,
                        def.pct_fondo_incobrables,
                        def.tope_reserva_incobrables_pct,
                        def.ppcfc_venta_rango1_pct,
                        def.ppcfc_venta_rango2_pct,
                        def.tope_individual_mensual_pct,
                        def.plazo_maximo_cuotas,
                        def.tasa_interes_ordinario,
                        def.tasa_interes_mora,
                        def.monto_maximo_credito,
                    ],
                )?;
                Ok(def)
            }
        }
    }

    fn actualizar(&self, banco_id: &str, config: &Configuracion) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        Self::leer(&conn, banco_id)?.ok_or(AppError::ConfiguracionNoEncontrada)?;
        conn.execute(
            "UPDATE configuracion SET
                valor_nominal = ?2,
                pct_garantia_socio = ?3,
                pct_garantia_fiador = ?4,
                pct_fondo_gastos = ?5,
                pct_fondo_incobrables = ?6,
                tope_reserva_incobrables_pct = ?7,
                ppcfc_venta_rango1_pct = ?8,
                ppcfc_venta_rango2_pct = ?9,
                tope_individual_mensual_pct = ?10,
                plazo_maximo = ?11,
                tasa_interes_ordinario = ?12,
                tasa_interes_mora = ?13,
                monto_maximo_credito = ?14
            WHERE id = ?1",
            params![
                banco_id,
                config.valor_nominal,
                config.pct_garantia_socio,
                config.pct_garantia_fiador,
                config.pct_fondo_gastos,
                config.pct_fondo_incobrables,
                config.tope_reserva_incobrables_pct,
                config.ppcfc_venta_rango1_pct,
                config.ppcfc_venta_rango2_pct,
                config.tope_individual_mensual_pct,
                config.plazo_maximo_cuotas,
                config.tasa_interes_ordinario,
                config.tasa_interes_mora,
                config.monto_maximo_credito,
            ],
        )?;
        Ok(())
    }

    fn obtener_datos_generales(&self, banco_id: &str) -> Result<DatosGenerales, AppError> {
        // Siembra la config con valores por defecto en el primer uso.
        let cfg = self.obtener(banco_id)?;
        let conn = self.db.open_banco_db(banco_id)?;

        // RF-10: contadores automáticos desde las tablas de negocio del mismo .db.
        let numero_creditos_otorgados: i64 = conn.query_row(
            "SELECT COUNT(*) FROM credito",
            [],
            |r| r.get(0),
        )?;
        let monto_total_creditos: f64 = conn.query_row(
            "SELECT COALESCE(SUM(monto_original), 0) FROM credito",
            [],
            |r| r.get(0),
        )?;
        let numero_acciones_vendidas: i64 = conn.query_row(
            "SELECT COALESCE(SUM(cantidad), 0) FROM lote_acciones",
            [],
            |r| r.get(0),
        )?;
        let saldo_fondo_gastos: f64 = conn.query_row(
            "SELECT COALESCE(saldo_fondo_gastos, 0) FROM configuracion WHERE id = ?1",
            params![banco_id],
            |r| r.get(0),
        )?;
        let saldo_fondo_incobrables: f64 = conn.query_row(
            "SELECT COALESCE(saldo_fondo_incobrables, 0) FROM configuracion WHERE id = ?1",
            params![banco_id],
            |r| r.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, nombre, ubicacion, fecha_creacion, moneda
             FROM configuracion WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![banco_id])?;
        let row = rows.next()?.ok_or(AppError::ConfiguracionNoEncontrada)?;

        Ok(DatosGenerales {
            id: row.get(0)?,
            nombre: row.get(1)?,
            ubicacion: row.get(2)?,
            fecha_creacion: row.get(3)?,
            moneda: row.get(4)?,
            valor_nominal: cfg.valor_nominal,
            numero_creditos_otorgados,
            monto_total_creditos,
            numero_acciones_vendidas,
            saldo_fondo_gastos,
            saldo_fondo_incobrables,
        })
    }
}

impl SqliteConfiguracion {
    /// Lee del catálogo (`app.db`) el nombre, ubicación, moneda y fecha de creación con
    /// que se registró el Bankomunal, para sembrar con ellos su configuración.
    ///
    /// El catálogo es la única fuente de verdad de esos datos: duplicarlos a mano en el
    /// `.db` del Banco es lo que hacía que RF-09 mostrara un fragmento del UUID en vez
    /// del nombre real y una cadena sin sentido como fecha de creación.
    fn identidad_del_catalogo(&self, banco_id: &str) -> Result<IdentidadBanco, AppError> {
        let conn = self.db.open_app_db()?;
        let mut stmt = conn.prepare(
            "SELECT nombre, ubicacion, moneda, fecha_creacion
             FROM catalogo_bankomunal WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![banco_id])?;
        match rows.next()? {
            Some(row) => Ok(IdentidadBanco {
                nombre: row.get(0)?,
                ubicacion: row.get(1)?,
                moneda: row.get(2)?,
                fecha_creacion: row.get(3)?,
            }),
            // Un `.db` de Banco sin fila en el catálogo sólo ocurre en pruebas que crean
            // el archivo directamente; se siembra con valores neutros en vez de fallar.
            None => Ok(IdentidadBanco {
                nombre: String::new(),
                ubicacion: String::new(),
                moneda: "COP".into(),
                fecha_creacion: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
            }),
        }
    }

    fn leer(conn: &Connection, banco_id: &str) -> Result<Option<Configuracion>, AppError> {
        let mut stmt = conn.prepare(
            "SELECT id, nombre, ubicacion, moneda, fecha_creacion,
                    valor_nominal, pct_garantia_socio, pct_garantia_fiador,
                    pct_fondo_gastos, pct_fondo_incobrables, tope_reserva_incobrables_pct,
                    ppcfc_venta_rango1_pct, ppcfc_venta_rango2_pct, tope_individual_mensual_pct,
                    plazo_maximo, tasa_interes_ordinario, tasa_interes_mora,
                    monto_maximo_credito
             FROM configuracion WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![banco_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row_to_configuracion(row)?)),
            None => Ok(None),
        }
    }
}

/// Identidad del Bankomunal tal como quedó registrada en el catálogo del `app.db`
/// al crearlo (RF-03). Es la fuente de verdad de los Datos Generales (RF-09).
struct IdentidadBanco {
    nombre: String,
    ubicacion: String,
    moneda: String,
    fecha_creacion: String,
}
