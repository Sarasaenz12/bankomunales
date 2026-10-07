use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::db::DbManager;
use crate::core::error::AppError;

use super::domain::{
    AppSettingsPort, BancoCatalogoPort, Bankomunal, NuevoBankomunal, PasswordHasherPort,
    DEFAULT_PASSWORD_GENERICA, SETTING_PASSWORD_HASH,
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

    /// Inicialización de primer uso: crea el app.db y siembra el hash de la contraseña
    /// genérica por defecto. No sobrescribe un hash ya existente.
    pub fn inicializar(&self) -> Result<(), AppError> {
        if self.settings.obtener(SETTING_PASSWORD_HASH)?.is_none() {
            let hash = self.hasher.hash(DEFAULT_PASSWORD_GENERICA)?;
            self.settings.guardar(SETTING_PASSWORD_HASH, &hash)?;
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
    // - normaliza los datos una sola vez (aquí, en el borde del dominio)
    // - genera el id, registra el Bankomunal en el catálogo y crea su .db
    // - devuelve la entidad creada ya normalizada
    pub fn crear_bankomunal(
        &self,
        nuevo: NuevoBankomunal,
    ) -> Result<Bankomunal, AppError> {
        let nombre = nuevo.nombre.trim().to_string();
        // RF-04: rechaza creación con nombre vacío (sin datos no se crea el Banco).
        if nombre.is_empty() {
            return Err(AppError::DatosIncompletos);
        }
        let moneda = {
            let m = nuevo.moneda.trim();
            if m.is_empty() { "COP".to_string() } else { m.to_uppercase() }
        };
        let normalizado = NuevoBankomunal {
            nombre,
            ubicacion: nuevo.ubicacion.trim().to_string(),
            moneda,
        };

        // RF-04: el nombre no puede duplicarse (comparación sin distinción de mayúsculas).
        // Pre-chequeo para dar un error amable; la unicidad real la garantiza el índice
        // UNIQUE del catálogo, que cierra la condición de carrera.
        if self
            .catalogo
            .buscar_por_nombre(&normalizado.nombre)?
            .is_some()
        {
            return Err(AppError::NombreDuplicado);
        }

        let id = Uuid::new_v4().to_string();
        let fecha_creacion = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.catalogo.crear(&id, &fecha_creacion, &normalizado)
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

    /// RF-03/RF-05: un Bankomunal creado se devuelve con su id y conserva su ubicación y
    /// moneda en el listado — los datos que la pantalla de detalle (`/app/bancos/:id`)
    /// carga y muestra tras crearlo.
    #[test]
    fn el_bankomunal_creado_conserva_sus_datos_en_el_listado() {
        let auth = test_services();
        let creado = auth
            .crear_bankomunal(NuevoBankomunal {
                nombre: "Bankordillera".into(),
                ubicacion: "Salento, Quindío".into(),
                moneda: "USD".into(),
            })
            .unwrap();
        assert!(!creado.id.is_empty(), "el banco creado debe traer su id");

        let lista = auth.listar_bankomunales().unwrap();
        let encontrado = lista
            .iter()
            .find(|b| b.id == creado.id)
            .expect("el banco creado debe aparecer en el listado");
        assert_eq!(encontrado.nombre, "Bankordillera");
        assert_eq!(encontrado.ubicacion, "Salento, Quindío");
        assert_eq!(encontrado.moneda, "USD");
        assert!(!encontrado.fecha_creacion.is_empty(), "debe registrar la fecha de creación");
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
    fn nombre_duplicado_sin_distinguir_mayusculas_ni_espacios() {
        let auth = test_services();
        auth.crear_bankomunal(NuevoBankomunal {
            nombre: "Pijao".into(),
            ubicacion: "".into(),
            moneda: "COP".into(),
        })
        .unwrap();

        // Mismo nombre en otra caja y con espacios → sigue siendo duplicado (RF-04).
        let err = auth
            .crear_bankomunal(NuevoBankomunal {
                nombre: "  pijao ".into(),
                ubicacion: "".into(),
                moneda: "cop".into(),
            })
            .unwrap_err();
        assert!(matches!(err, AppError::NombreDuplicado));
        assert_eq!(auth.listar_bankomunales().unwrap().len(), 1);
    }

    #[test]
    fn crear_normaliza_nombre_moneda_y_ubicacion() {
        let auth = test_services();
        let banco = auth
            .crear_bankomunal(NuevoBankomunal {
                nombre: "  Salento  ".into(),
                ubicacion: "  Salento, Quindío  ".into(),
                moneda: "  usd ".into(),
            })
            .unwrap();
        // El nombre y la ubicación se recortan; la moneda se recorta y va en mayúsculas.
        assert_eq!(banco.nombre, "Salento");
        assert_eq!(banco.ubicacion, "Salento, Quindío");
        assert_eq!(banco.moneda, "USD");

        // Moneda vacía cae al valor por defecto.
        let otro = auth
            .crear_bankomunal(NuevoBankomunal {
                nombre: "Filandia".into(),
                ubicacion: "".into(),
                moneda: "   ".into(),
            })
            .unwrap();
        assert_eq!(otro.moneda, "COP");
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