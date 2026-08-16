use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;
use crate::modules::configuracion::data::SqliteConfiguracion;
use crate::modules::configuracion::domain::ConfiguracionPort;

use super::domain::{
    Bien, BienPort, CodigoOperacion, FiltroLibro, FondoGastosPort, LibroPort, Movimiento, TipoBien,
};

const SELECT_MOV: &str = "SELECT id, numero, fecha, codigo, descripcion, ingreso, egreso, saldo,
            socio_id, credito_id, cierre_mes_id, corregido, corregido_por,
            fecha_correccion, motivo_correccion
     FROM movimiento_libro";

/// El Libro se lee siempre en orden cronológico y, a igual fecha, por consecutivo:
/// es el orden en que se acumula el saldo y en que se imprime en papel.
const ORDEN_LIBRO: &str = "ORDER BY fecha, numero";

fn fila_a_movimiento(row: &rusqlite::Row) -> rusqlite::Result<Movimiento> {
    let codigo: String = row.get(3)?;
    Ok(Movimiento {
        id: row.get(0)?,
        numero: row.get(1)?,
        fecha: row.get(2)?,
        // Un código desconocido no debe impedir leer el Libro: se degrada a "Otro
        // ingreso/egreso" según el signo, y el asiento sigue visible.
        codigo: CodigoOperacion::desde_str(&codigo).unwrap_or_else(|| {
            let ingreso: f64 = row.get(5).unwrap_or(0.0);
            if ingreso > 0.0 {
                CodigoOperacion::OtroIngreso
            } else {
                CodigoOperacion::OtroEgreso
            }
        }),
        descripcion: row.get(4)?,
        ingreso: row.get(5)?,
        egreso: row.get(6)?,
        saldo: row.get(7)?,
        socio_id: row.get(8)?,
        credito_id: row.get(9)?,
        cierre_mes_id: row.get(10)?,
        corregido: row.get::<_, i64>(11)? != 0,
        corregido_por: row.get(12)?,
        fecha_correccion: row.get(13)?,
        motivo_correccion: row.get(14)?,
    })
}

/// Adaptador SQLite del Libro de Ingresos y Egresos.
pub struct SqliteLibro {
    db: DbManager,
}

impl SqliteLibro {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl LibroPort for SqliteLibro {
    fn registrar(&self, banco_id: &str, mov: &Movimiento) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO movimiento_libro
             (id, socio_id, credito_id, numero, fecha, codigo, descripcion,
              ingreso, egreso, saldo, cierre_mes_id, corregido)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0)",
            params![
                mov.id,
                mov.socio_id,
                mov.credito_id,
                mov.numero,
                mov.fecha,
                mov.codigo.as_str(),
                mov.descripcion,
                mov.ingreso,
                mov.egreso,
                mov.saldo,
                mov.cierre_mes_id,
            ],
        )?;
        Ok(())
    }

    fn actualizar(&self, banco_id: &str, mov: &Movimiento) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let filas = conn.execute(
            "UPDATE movimiento_libro SET
                fecha = ?2, descripcion = ?3, ingreso = ?4, egreso = ?5,
                corregido = ?6, corregido_por = ?7, fecha_correccion = ?8,
                motivo_correccion = ?9
             WHERE id = ?1",
            params![
                mov.id,
                mov.fecha,
                mov.descripcion,
                mov.ingreso,
                mov.egreso,
                i64::from(mov.corregido),
                mov.corregido_por,
                mov.fecha_correccion,
                mov.motivo_correccion,
            ],
        )?;
        if filas == 0 {
            return Err(AppError::MovimientoNoEncontrado);
        }
        Ok(())
    }

    fn buscar_por_id(&self, banco_id: &str, id: &str) -> Result<Option<Movimiento>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let sql = format!("{SELECT_MOV} WHERE id = ?1");
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(fila_a_movimiento(row)?)),
            None => Ok(None),
        }
    }

    fn listar(&self, banco_id: &str, filtro: &FiltroLibro) -> Result<Vec<Movimiento>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        // Las fechas se guardan como texto ISO (YYYY-MM-DD), así que comparar como
        // cadena equivale a comparar cronológicamente.
        let sql = format!(
            "{SELECT_MOV}
             WHERE (?1 IS NULL OR fecha >= ?1) AND (?2 IS NULL OR fecha <= ?2)
             {ORDEN_LIBRO}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![filtro.desde, filtro.hasta], fila_a_movimiento)?;
        let mut v = Vec::new();
        for m in rows {
            v.push(m?);
        }
        Ok(v)
    }

    fn siguiente_numero(&self, banco_id: &str) -> Result<i64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let maximo: i64 = conn.query_row(
            "SELECT COALESCE(MAX(numero), 0) FROM movimiento_libro",
            [],
            |r| r.get(0),
        )?;
        Ok(maximo + 1)
    }

    fn recalcular_saldos(&self, banco_id: &str) -> Result<(), AppError> {
        let mut conn = self.db.open_banco_db(banco_id)?;
        let tx = conn.transaction()?;

        let filas: Vec<(String, f64, f64)> = {
            let sql = format!("SELECT id, ingreso, egreso FROM movimiento_libro {ORDEN_LIBRO}");
            let mut stmt = tx.prepare(&sql)?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };

        let mut acumulado = 0.0;
        for (id, ingreso, egreso) in filas {
            acumulado += ingreso - egreso;
            tx.execute(
                "UPDATE movimiento_libro SET saldo = ?2 WHERE id = ?1",
                params![id, acumulado],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

/// Adaptador del saldo acumulado del Fondo para Gastos.
///
/// El dato vive en la tabla `configuracion`, pero el módulo de Caja no depende del de
/// Configuración: su Dominio sólo conoce el puerto `FondoGastosPort`. El acoplamiento
/// queda aquí, en la capa de datos, que es donde vive el conocimiento de las tablas.
pub struct SqliteFondoGastos {
    db: DbManager,
    /// Se reutiliza el adaptador de Configuración para garantizar que la fila exista
    /// antes de tocarla. Sembrarla aquí a mano dejaría un registro sin nombre ni fecha,
    /// y entonces RF-09 volvería a mostrar datos vacíos: la siembra correcta —leyendo
    /// la identidad del catálogo— debe seguir ocurriendo en un solo lugar.
    config: SqliteConfiguracion,
}

impl SqliteFondoGastos {
    pub fn new(db: DbManager) -> Self {
        Self {
            config: SqliteConfiguracion::new(db.clone()),
            db,
        }
    }

    /// Un Bankomunal recién creado no tiene fila de configuración hasta que alguien
    /// abre esa pantalla. Registrar una donación antes de eso es perfectamente posible,
    /// así que el fondo se asegura de que exista.
    fn asegurar_configuracion(&self, banco_id: &str) -> Result<(), AppError> {
        self.config.obtener(banco_id).map(|_| ())
    }
}

impl FondoGastosPort for SqliteFondoGastos {
    fn saldo(&self, banco_id: &str) -> Result<f64, AppError> {
        self.asegurar_configuracion(banco_id)?;
        let conn = self.db.open_banco_db(banco_id)?;
        let saldo: f64 = conn.query_row(
            "SELECT COALESCE(saldo_fondo_gastos, 0) FROM configuracion WHERE id = ?1",
            params![banco_id],
            |r| r.get(0),
        )?;
        Ok(saldo)
    }

    fn ajustar(&self, banco_id: &str, delta: f64) -> Result<(), AppError> {
        self.asegurar_configuracion(banco_id)?;
        let conn = self.db.open_banco_db(banco_id)?;
        let filas = conn.execute(
            "UPDATE configuracion
                SET saldo_fondo_gastos = COALESCE(saldo_fondo_gastos, 0) + ?2
              WHERE id = ?1",
            params![banco_id, delta],
        )?;
        if filas == 0 {
            return Err(AppError::ConfiguracionNoEncontrada);
        }
        Ok(())
    }
}

/// Adaptador de los Bienes Adquiridos / Activo Fijo (RF-88).
pub struct SqliteBienes {
    db: DbManager,
}

impl SqliteBienes {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl BienPort for SqliteBienes {
    fn registrar(&self, banco_id: &str, bien: &Bien) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO bien (id, descripcion, fecha_adquisicion, valor, tipo)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                bien.id,
                bien.descripcion,
                bien.fecha_adquisicion,
                bien.valor,
                bien.tipo.as_str()
            ],
        )?;
        Ok(())
    }

    fn listar(&self, banco_id: &str) -> Result<Vec<Bien>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, descripcion, fecha_adquisicion, valor, tipo
             FROM bien ORDER BY fecha_adquisicion, rowid",
        )?;
        let rows = stmt.query_map([], |row| {
            let tipo: String = row.get(4)?;
            Ok(Bien {
                id: row.get(0)?,
                descripcion: row.get(1)?,
                fecha_adquisicion: row.get(2)?,
                valor: row.get(3)?,
                tipo: TipoBien::desde_str(&tipo),
            })
        })?;
        let mut v = Vec::new();
        for b in rows {
            v.push(b?);
        }
        Ok(v)
    }

    fn valor_total(&self, banco_id: &str) -> Result<f64, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(valor), 0) FROM bien",
            [],
            |r| r.get(0),
        )?;
        Ok(total)
    }
}
