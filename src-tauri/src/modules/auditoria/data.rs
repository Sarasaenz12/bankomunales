use rusqlite::params;

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{AuditoriaPort, EntradaAuditoria};

/// Adaptador SQLite de la bitácora de Auditoría (tabla `auditoria` del `.db` del Banco).
pub struct SqliteAuditoria {
    db: DbManager,
}

impl SqliteAuditoria {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl AuditoriaPort for SqliteAuditoria {
    fn registrar(&self, banco_id: &str, entrada: &EntradaAuditoria) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO auditoria
             (id, fecha, nombre_quien_realiza, entidad_afectada, campo_modificado,
              valor_anterior, valor_nuevo, motivo, tipo_accion)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entrada.id,
                entrada.fecha,
                entrada.nombre_quien_realiza,
                entrada.entidad_afectada,
                entrada.campo_modificado,
                entrada.valor_anterior,
                entrada.valor_nuevo,
                entrada.motivo,
                entrada.tipo_accion,
            ],
        )?;
        Ok(())
    }

    fn listar(&self, banco_id: &str) -> Result<Vec<EntradaAuditoria>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, fecha, nombre_quien_realiza, entidad_afectada, campo_modificado,
                    valor_anterior, valor_nuevo, motivo, tipo_accion
             FROM auditoria ORDER BY fecha DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(EntradaAuditoria {
                id: row.get(0)?,
                fecha: row.get(1)?,
                nombre_quien_realiza: row.get(2)?,
                entidad_afectada: row.get(3)?,
                campo_modificado: row.get(4)?,
                valor_anterior: row.get(5)?,
                valor_nuevo: row.get(6)?,
                motivo: row.get(7)?,
                tipo_accion: row.get(8)?,
            })
        })?;
        let mut v = Vec::new();
        for e in rows {
            v.push(e?);
        }
        Ok(v)
    }
}
