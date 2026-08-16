use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{
    AppSettingsPort, BancoCatalogoPort, Bankomunal, NuevoBankomunal, PasswordHasherPort,
    SETTING_CLAVE_CONFIG_HASH, SETTING_PASSWORD_HASH, DEFAULT_CLAVE_CONFIGURACION,
    DEFAULT_PASSWORD_GENERICA,
};

/// Capa de Aplicación/Servicios del módulo de Autenticación.
/// Orquesta los casos de uso RF-01 a RF-08 usando únicamente puertos (traits),
/// nunca SQLite directamente.
pub struct AuthService {
    settings: Arc<dyn AppSettingsPort>,
    hasher: Arc<dyn PasswordHasherPort>,
    catalogo: Arc<dyn BancoCatalogoPort>,
    db: DbManager,
}

impl AuthService {
    pub fn new(
        settings: Arc<dyn AppSettingsPort>,
        hasher: Arc<dyn PasswordHasherPort>,
        catalogo: Arc<dyn BancoCatalogoPort>,
        db: DbManager,
    ) -> Self {
        Self {
            settings,
            hasher,
            catalogo,
            db,
        }
    }

    /// Inicialización de primer uso: crea el app.db y siembra las claves por defecto
    /// (contraseña genérica y clave de configuración inicial) con su hash.
    /// No sobrescribe valores ya existentes.
    pub fn inicializar(&self) -> Result<(), AppError> {
        if self.settings.obtener(SETTING_PASSWORD_HASH)?.is_none() {
            let hash = self.hasher.hash(DEFAULT_PASSWORD_GENERICA)?;
            self.settings.guardar(SETTING_PASSWORD_HASH, &hash)?;
        }
        if self.settings.obtener(SETTING_CLAVE_CONFIG_HASH)?.is_none() {
            let hash = self.hasher.hash(DEFAULT_CLAVE_CONFIGURACION)?;
            self.settings.guardar(SETTING_CLAVE_CONFIG_HASH, &hash)?;
        }
        Ok(())
    }

    /// RF-01 / RF-02: valida la contraseña genérica compartida contra su hash.
    pub fn login(&self, password: &str) -> Result<bool, AppError> {
        let hash = self
            .settings
            .obtener(SETTING_PASSWORD_HASH)?
            .ok_or(AppError::NotInitialized)?;
        self.hasher.verify(password, &hash)
    }

    // RF-03 / RF-04: crear un nuevo Bankomunal.
    // - genera el id y crea el .db independiente del Banco
    // - registra el Bankomunal en el catálogo y devuelve la entidad creada
    pub fn crear_bankomunal(
        &self,
        nuevo: NuevoBankomunal,
    ) -> Result<Bankomunal, AppError> {
        // RF-04: rechaza creación con nombre vacío (sin datos no se crea el Banco).
        if nuevo.nombre.trim().is_empty() {
            return Err(AppError::DatosIncompletos);
        }
        // RF-04: el nombre no puede estar duplicado entre Bankomunales.
        if self
            .catalogo
            .buscar_por_nombre(&nuevo.nombre)?
            .is_some()
        {
            return Err(AppError::NombreDuplicado);
        }

        let id = Uuid::new_v4().to_string();
        let fecha_creacion = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.catalogo.crear(&id, &fecha_creacion, &nuevo)
    }

    /// RF-05 / RF-06: listar Bankomunales para la pantalla de selección.
    pub fn listar_bankomunales(&self) -> Result<Vec<Bankomunal>, AppError> {
        self.catalogo.listar()
    }

    /// RF-06: al iniciar sesión, si hay un solo Bankomunal se entra directo.
    pub fn unico_bankomunal(&self) -> Result<Option<Bankomunal>, AppError> {
        let mut list = self.catalogo.listar()?;
        if list.len() == 1 {
            Ok(list.pop())
        } else {
            Ok(None)
        }
    }

    /// Busca un Bankomunal por id (para validar la selección de RF-05/RF-07).
    pub fn buscar_por_id(&self, id: &str) -> Result<Option<Bankomunal>, AppError> {
        self.catalogo.buscar_por_id(id)
    }

    /// Abre (crea si falta) el `.db` del Bankomunal activo. Usado por otros módulos
    /// para operar sobre el archivo independiente del Banco (RF-08).
    pub fn abrir_banco_db(&self, banco_id: &str) -> Result<(), AppError> {
        self.db.open_banco_db(banco_id).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::auth::data::{BcryptHasher, SqliteAppSettings, SqliteBancoCatalogo};

    fn test_services() -> AuthService {
        let dir = std::env::temp_dir().join(format!("bkn_auth_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let auth = AuthService::new(
            Arc::new(SqliteAppSettings::new(db.clone())),
            Arc::new(BcryptHasher),
            Arc::new(SqliteBancoCatalogo::new(db.clone())),
            db,
        );
        auth.inicializar().unwrap();
        auth
    }

    #[test]
    fn primer_uso_siembra_claves_por_defecto() {
        let auth = test_services();
        // La contraseña genérica por defecto funciona (RF-01).
        assert!(auth.login(DEFAULT_PASSWORD_GENERICA).unwrap());
        // La creación de un Banco funciona (RF-03).
        let nuevo = NuevoBankomunal {
            nombre: "Pijao".into(),
            ubicacion: "Pijao, Quindío".into(),
            moneda: "COP".into(),
        };
        assert!(auth.crear_bankomunal(nuevo).is_ok());
    }

    #[test]
    fn password_incorrecta_devuelve_false() {
        let auth = test_services();
        assert!(!auth.login("clave-mal".into()).unwrap());
    }

    // (test eliminado: la clave de configuración ya no existe)

    #[test]
    fn nombre_duplicado_no_se_crea() {
        let auth = test_services();
        let nuevo = |nombre: &str| NuevoBankomunal {
            nombre: nombre.into(),
            ubicacion: "".into(),
            moneda: "COP".into(),
        };
        auth.crear_bankomunal(nuevo("Banco Test")).unwrap();
        let err = auth.crear_bankomunal(nuevo("Banco Test")).unwrap_err();
        assert!(matches!(err, AppError::NombreDuplicado));
    }

    #[test]
    fn unico_bankomunal_y_seleccion() {
        let auth = test_services();
        // Sin bancos: no hay selección directa.
        assert!(auth.unico_bankomunal().unwrap().is_none());
        let banco = auth
            .crear_bankomunal(
                NuevoBankomunal {
                    nombre: "Único".into(),
                    ubicacion: "".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap();
        // Con un solo Banco, se entra directo (RF-06).
        assert_eq!(auth.unico_bankomunal().unwrap().unwrap().id, banco.id);
        // Selección por id devuelve el Banco y su .db existe (RF-08).
        let encontrado = auth.buscar_por_id(&banco.id).unwrap().unwrap();
        assert_eq!(encontrado.nombre, "Único");
        auth.abrir_banco_db(&banco.id).unwrap();
    }

    #[test]
    fn cero_bankomunales_activa_flujo_de_creacion() {
        let auth = test_services();
        // RF-03: sin Bankomunales creados no hay nada que seleccionar: el sistema debe
        // activar el flujo de "crear el primer Bankomunal", no la pantalla de selección.
        let lista = auth.listar_bankomunales().unwrap();
        assert!(lista.is_empty(), "no debe existir ningún Bankomunal aún");
        assert!(
            auth.unico_bankomunal().unwrap().is_none(),
            "no debe ofrecerse selección directa sin Bankomunales"
        );
        // La creación del primer Bankomunal debe estar disponible (flujo RF-03).
        let banco = auth
            .crear_bankomunal(
                NuevoBankomunal {
                    nombre: "Primer Banco".into(),
                    ubicacion: "La Tebaida, Quindío".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap();
        assert_eq!(banco.nombre, "Primer Banco");
        // Tras crear el primero ya hay exactamente uno.
        assert_eq!(auth.listar_bankomunales().unwrap().len(), 1);
    }

    #[test]
    fn varios_bankomunales_requieren_seleccion() {
        let auth = test_services();
        for nombre in ["Banco PIJ", "Banco TEBA", "Banco CAL"] {
            auth.crear_bankomunal(
                NuevoBankomunal {
                    nombre: nombre.into(),
                    ubicacion: "".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap();
        }
        // RF-05: con más de un Bankomunal se muestra la pantalla de selección
        // (no se entra directo a ninguno).
        let lista = auth.listar_bankomunales().unwrap();
        assert_eq!(lista.len(), 3);
        assert!(auth.unico_bankomunal().unwrap().is_none());
        // Cada Banco creado existe y se puede localizar por su id.
        for banco in &lista {
            let encontrado = auth.buscar_por_id(&banco.id).unwrap().unwrap();
            assert_eq!(encontrado.nombre, banco.nombre);
            auth.abrir_banco_db(&banco.id).unwrap();
        }
    }

    #[test]
    fn solo_nombre_duplicado_no_se_crea() {
        let auth = test_services();
        auth.crear_bankomunal(NuevoBankomunal {
            nombre: "Pijao".into(),
            ubicacion: "".into(),
            moneda: "COP".into(),
        })
        .unwrap();

        // Mismo nombre → no se crea (RF-04).
        let err = auth
            .crear_bankomunal(
                NuevoBankomunal {
                    nombre: "Pijao".into(),
                    ubicacion: "".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, AppError::NombreDuplicado));

        // Ninguno se registró por duplicado: sigue habiendo un solo Banco.
        assert_eq!(auth.listar_bankomunales().unwrap().len(), 1);
    }

    #[test]
    fn intento_de_creacion_con_datos_vacios_rechazado() {
        let auth = test_services();

        // Nombre vacío (solo espacios).
        let err = auth
            .crear_bankomunal(
                NuevoBankomunal {
                    nombre: "   ".into(),
                    ubicacion: "".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, AppError::DatosIncompletos));

        // Vacío.
        let err = auth
            .crear_bankomunal(
                NuevoBankomunal {
                    nombre: "".into(),
                    ubicacion: "".into(),
                    moneda: "COP".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, AppError::DatosIncompletos));

        // No se registró ningún Banco.
        assert!(auth.listar_bankomunales().unwrap().is_empty());
    }
}