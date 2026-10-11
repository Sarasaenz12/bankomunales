use super::db::DbManager;
use super::error::AppError;

/// Garantiza que un caso de uso que escribe en varios módulos se aplique completo o no se
/// aplique. Pensado para operaciones que mueven dinero por más de una tabla o módulo (pago
/// de cuotas, retiro de socio, venta parcial): si algo falla a la mitad, todo vuelve a su
/// estado previo. Puede anidarse: la operación interna se une a la externa.
pub trait AtomicidadPort: Send + Sync {
    fn ejecutar(
        &self,
        banco_id: &str,
        operacion: &mut dyn FnMut() -> Result<(), AppError>,
    ) -> Result<(), AppError>;
}

/// Ejecuta `f` de forma atómica y devuelve su resultado (envoltorio cómodo del puerto).
pub fn atomico<T>(
    puerto: &dyn AtomicidadPort,
    banco_id: &str,
    mut f: impl FnMut() -> Result<T, AppError>,
) -> Result<T, AppError> {
    let mut resultado = None;
    puerto.ejecutar(banco_id, &mut || {
        resultado = Some(f()?);
        Ok(())
    })?;
    resultado.ok_or_else(|| AppError::Serde("la operación atómica no produjo resultado".into()))
}

/// Atomicidad sobre SQLite: copia previa del `.db` y restauración si la operación falla.
pub struct SqliteAtomicidad {
    db: DbManager,
}

impl SqliteAtomicidad {
    pub fn new(db: DbManager) -> Self {
        Self { db }
    }
}

impl AtomicidadPort for SqliteAtomicidad {
    fn ejecutar(
        &self,
        banco_id: &str,
        operacion: &mut dyn FnMut() -> Result<(), AppError>,
    ) -> Result<(), AppError> {
        self.db.en_operacion_atomica(banco_id, operacion)
    }
}
