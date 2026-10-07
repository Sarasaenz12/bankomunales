use super::domain::{Respaldo, RespaldoPort};
use crate::core::db::DbManager;
use crate::core::error::AppError;

pub struct SQLiteRespaldoRepository {
    pub db: std::sync::Arc<DbManager>,
}

impl RespaldoPort for SQLiteRespaldoRepository {
    fn registrar_respaldo(&self, banco_id: &str, respaldo: Respaldo) -> Result<(), AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute(
            "INSERT INTO respaldo (id, fecha, usuario, ruta_archivo, tipo) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![respaldo.id, respaldo.fecha, respaldo.usuario, respaldo.ruta_archivo, respaldo.tipo],
        )?;
        Ok(())
    }

    fn crear_copia_backup(&self, banco_id: &str, destino_ruta: &str) -> Result<(), AppError> {
        // `VACUUM INTO` produce una copia transaccionalmente CONSISTENTE del `.db` (mejor
        // que copiar el archivo a nivel de sistema, que podría capturarlo a medio escribir),
        // y además la deja compactada. Exige que el destino no exista; el diálogo
        // "guardar como" ya implica sobrescribir, así que se borra primero si estaba.
        if std::path::Path::new(destino_ruta).exists() {
            std::fs::remove_file(destino_ruta)?;
        }
        let conn = self.db.open_banco_db(banco_id)?;
        conn.execute("VACUUM INTO ?1", rusqlite::params![destino_ruta])
            .map_err(|e| {
                AppError::OperacionNoPermitida(format!(
                    "Error creando el respaldo en {destino_ruta}: {e}"
                ))
            })?;
        Ok(())
    }

    fn validar_y_restaurar_backup(
        &self,
        banco_id: &str,
        origen_ruta: &str,
    ) -> Result<(), AppError> {
        // Verificar que el backup pertenezca al mismo bankomunal
        let backup_conn = rusqlite::Connection::open(origen_ruta).map_err(|e| {
            AppError::OperacionNoPermitida(format!("Error abriendo archivo de respaldo: {}", e))
        })?;

        let mut id_backup = String::new();
        backup_conn
            .query_row("SELECT id FROM configuracion LIMIT 1", [], |row| {
                id_backup = row.get(0)?;
                Ok(())
            })
            .map_err(|_| {
                AppError::OperacionNoPermitida(
                    "El archivo seleccionado no parece ser un respaldo válido de Bankomunales."
                        .to_string(),
                )
            })?;

        if id_backup != banco_id {
            return Err(AppError::OperacionNoPermitida(
                "El archivo de respaldo pertenece a otro Bankomunal.".to_string(),
            ));
        }

        // Drop the backup connection so we don't lock the file
        drop(backup_conn);

        let banco_db_path = self.db.banco_db_path(banco_id);
        std::fs::copy(origen_ruta, &banco_db_path).map_err(|e| {
            AppError::OperacionNoPermitida(format!(
                "Error restaurando el archivo de base de datos de {origen_ruta}: {}",
                e
            ))
        })?;

        Ok(())
    }

    fn listar_respaldos(&self, banco_id: &str) -> Result<Vec<Respaldo>, AppError> {
        let conn = self.db.open_banco_db(banco_id)?;
        let mut stmt = conn.prepare(
            "SELECT id, fecha, usuario, ruta_archivo, tipo FROM respaldo ORDER BY fecha DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Respaldo {
                id: row.get(0)?,
                fecha: row.get(1)?,
                usuario: row.get(2)?,
                ruta_archivo: row.get(3)?,
                tipo: row.get(4)?,
            })
        })?;
        let mut resultado = Vec::new();
        for r in rows {
            resultado.push(r?);
        }
        Ok(resultado)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn repo() -> (SQLiteRespaldoRepository, DbManager, String, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("bkn_respaldo_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir.clone());
        let banco = Uuid::new_v4().to_string();
        db.open_banco_db(&banco).unwrap();
        let repo = SQLiteRespaldoRepository { db: std::sync::Arc::new(db.clone()) };
        (repo, db, banco, dir)
    }

    /// El respaldo con `VACUUM INTO` produce un archivo válido y consistente que conserva
    /// los datos, y sobrescribe un destino existente.
    #[test]
    fn el_respaldo_es_consistente_y_sobrescribe() {
        let (repo, db, banco, dir) = repo();
        // La restauración valida el banco por su fila de configuración: debe existir.
        db.open_banco_db(&banco).unwrap().execute(
            "INSERT INTO configuracion (id, nombre, fecha_creacion) VALUES (?1, 'Test', '2026-01-01')",
            [&banco],
        ).unwrap();
        // Un dato para verificar que viaja al respaldo.
        db.open_banco_db(&banco).unwrap().execute(
            "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                                telefono, celular, correo, estatus, fecha_ingreso)
             VALUES ('s1','123','Ana','Ríos','','','','','','ACTIVO','2026-01-01')",
            [],
        ).unwrap();

        let destino = dir.join("copia.db");
        let destino_str = destino.to_string_lossy().to_string();

        // Se puede crear aunque el destino ya exista (se sobrescribe).
        std::fs::write(&destino, b"basura previa").unwrap();
        repo.crear_copia_backup(&banco, &destino_str).unwrap();

        // El respaldo es una base SQLite válida con el socio.
        let copia = rusqlite::Connection::open(&destino).unwrap();
        let n: i64 = copia
            .query_row("SELECT COUNT(*) FROM socio WHERE id = 's1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);

        // Restaurar desde ese respaldo valida el banco y repone los datos.
        drop(copia);
        repo.validar_y_restaurar_backup(&banco, &destino_str).unwrap();
        let n: i64 = db
            .open_banco_db(&banco)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM socio WHERE id = 's1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Restaurar un archivo de OTRO Bankomunal se rechaza (RF-113).
    #[test]
    fn restaurar_respaldo_de_otro_banco_se_rechaza() {
        let (repo, db, banco, dir) = repo();
        // Respaldo de un banco distinto: se le siembra su propia configuración.
        let otro = Uuid::new_v4().to_string();
        db.open_banco_db(&otro).unwrap().execute(
            "INSERT INTO configuracion (id, nombre, fecha_creacion) VALUES (?1, 'Otro', '2026-01-01')",
            [&otro],
        ).unwrap();
        let destino = dir.join("otro.db");
        let destino_str = destino.to_string_lossy().to_string();
        repo.crear_copia_backup(&otro, &destino_str).unwrap();

        let err = repo.validar_y_restaurar_backup(&banco, &destino_str).unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));

        std::fs::remove_dir_all(&dir).ok();
    }
}
