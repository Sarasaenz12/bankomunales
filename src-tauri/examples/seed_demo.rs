//! Siembra un "Banco Demo" con varios meses ya cerrados, para probar validaciones que
//! necesitan historia (PPCFC, reparto de ganancias, aniversario RN-10) y el respaldo/restaurar.
//!
//! Escribe en la MISMA carpeta de datos que usa la app (%APPDATA%/com.estra.bankomunales),
//! así que al abrir la app el banco aparece en la pantalla de selección.
//!
//! Uso (desde la carpeta src-tauri):
//!     cargo run --example seed_demo
//!
//! Requiere haber abierto la app al menos una vez (para que exista el app.db con la clave).

use std::sync::Arc;

use bankomunales_lib::core::db::DbManager;
use bankomunales_lib::modules::acciones::application::AccionesService;
use bankomunales_lib::modules::acciones::domain::NuevaCompra;
use bankomunales_lib::modules::auditoria::data::SqliteAuditoria;
use bankomunales_lib::modules::auth::application::AuthService;
use bankomunales_lib::modules::auth::data::{BcryptHasher, SqliteAppSettings, SqliteBancoCatalogo};
use bankomunales_lib::modules::auth::domain::NuevoBankomunal;
use bankomunales_lib::modules::caja::application::CajaService;
use bankomunales_lib::modules::caja::domain::CodigoOperacion;
use bankomunales_lib::modules::cierre::application::CierreService;
use bankomunales_lib::modules::configuracion::application::ConfigService;
use bankomunales_lib::modules::creditos::application::CreditoService;
use bankomunales_lib::modules::creditos::domain::{
    DestinoCredito, FiadorSolicitud, NuevoDesembolso,
};
use bankomunales_lib::modules::socios::application::SocioService;
use bankomunales_lib::modules::socios::domain::DatosSocio;

fn main() {
    // ── Carpeta de datos real de la app ──
    let appdata = std::env::var("APPDATA").expect("No se encontró %APPDATA%");
    let app_path = std::path::PathBuf::from(appdata).join("com.estra.bankomunales");
    std::fs::create_dir_all(&app_path).unwrap();
    println!("Carpeta de datos: {}", app_path.display());

    let db = DbManager::new(app_path);

    // ── Registrar el banco en el catálogo (app.db) ──
    let auth = AuthService::new(
        Arc::new(SqliteAppSettings::new(db.clone())),
        Arc::new(BcryptHasher),
        Arc::new(SqliteBancoCatalogo::new(db.clone())),
    );
    auth.inicializar().unwrap();

    let sufijo = chrono::Utc::now().format("%H%M%S").to_string();
    let nombre = format!("Banco Demo {sufijo}");
    let banco = auth
        .crear_bankomunal(NuevoBankomunal {
            nombre: nombre.clone(),
            ubicacion: "Ciudad Demo".into(),
            moneda: "COP".into(),
        })
        .expect("no se pudo crear el banco");
    let banco = banco.id;
    db.open_banco_db(&banco).unwrap();

    // ── Servicios sobre el banco (misma composición que la app) ──
    let caja = Arc::new(CajaService::new(
        Arc::new(bankomunales_lib::modules::caja::data::SqliteLibro::new(db.clone())),
        Arc::new(bankomunales_lib::modules::caja::data::SqliteFondoGastos::new(db.clone())),
        Arc::new(bankomunales_lib::modules::caja::data::SqliteBienes::new(db.clone())),
        Arc::new(SqliteAuditoria::new(db.clone())),
    ));
    let config = Arc::new(ConfigService::new(
        Arc::new(bankomunales_lib::modules::configuracion::data::SqliteConfiguracion::new(db.clone())),
        Arc::new(SqliteAuditoria::new(db.clone())),
    ));
    let socios = Arc::new(SocioService::new(Arc::new(
        bankomunales_lib::modules::socios::data::SqliteSocios::new(db.clone()),
    )));
    let acciones = Arc::new(AccionesService::new(
        Arc::new(bankomunales_lib::modules::acciones::data::SqliteLotesAcciones::new(db.clone())),
        Arc::new(bankomunales_lib::modules::acciones::data::SqliteParametrosAcciones::new(db.clone())),
        Arc::new(bankomunales_lib::modules::caja::puente::LibroViaCaja::new(caja.clone())),
        Arc::new(bankomunales_lib::modules::acciones::data::SqliteCierres::new(db.clone())),
    ));
    let creditos = Arc::new(CreditoService::new(
        Arc::new(bankomunales_lib::modules::creditos::data::SqliteSolicitudes::new(db.clone())),
        Arc::new(bankomunales_lib::modules::creditos::data::SqliteCreditos::new(db.clone())),
        Arc::new(bankomunales_lib::modules::creditos::data::ParametrosCreditoAdapter::new(db.clone())),
        Arc::new(bankomunales_lib::modules::creditos::data::AccionesParaCreditoAdapter::new(db.clone())),
        Arc::new(bankomunales_lib::modules::creditos::data::SociosParaCreditoAdapter::new(db.clone())),
        Arc::new(bankomunales_lib::modules::caja::puente::LibroViaCaja::new(caja.clone())),
    ));
    let cierre = CierreService::new(
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteCierres::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteReparto::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteLibroCierre::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteCartera::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteBienesCierre::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteFondosCierre::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteLotesCierre::new(db.clone())),
        Arc::new(bankomunales_lib::modules::cierre::data::SqliteSociosCierre::new(db.clone())),
        Arc::new(bankomunales_lib::modules::caja::puente::LibroViaCaja::new(caja.clone())),
        Arc::new(SqliteAuditoria::new(db.clone())),
        Arc::new(bankomunales_lib::core::atomicidad::SqliteAtomicidad::new(db.clone())),
    );

    // Config por defecto (valor nominal 10.000, tasas, % de garantía 20/20) y fecha de
    // creación en enero 2025 para que ya haya pasado el año (RN-10) y RN-02 aplique.
    config.obtener_configuracion(&banco).unwrap();
    db.open_banco_db(&banco)
        .unwrap()
        .execute(
            "UPDATE configuracion SET fecha_creacion = '2025-01-01' WHERE id = ?1",
            [banco.as_str()],
        )
        .unwrap();

    // ── Socios ──
    let socio = |ced: &str, nom: &str, ape: &str| {
        socios
            .registrar(
                &banco,
                DatosSocio {
                    cedula: ced.into(),
                    nombres: nom.into(),
                    apellidos: ape.into(),
                    profesion: "Comerciante".into(),
                    direccion: "Calle 1".into(),
                    telefono: String::new(),
                    celular: "3000000000".into(),
                    correo: String::new(),
                    beneficiario: None,
                    protegidos: vec![],
                },
            )
            .unwrap()
            .id
    };
    let ana = socio("111", "Ana", "Ruiz");
    let beto = socio("222", "Beto", "Páez");
    let carla = socio("333", "Carla", "Díaz");
    let dario = socio("444", "Darío", "León");

    // ── Compras de acciones en enero 2025 (100 c/u = $1.000.000 c/u) ──
    for (s, ced) in [(&ana, "05"), (&beto, "06"), (&carla, "07"), (&dario, "08")] {
        acciones
            .registrar_compra(
                &banco,
                NuevaCompra {
                    socio_id: s.clone(),
                    fecha: format!("2025-01-{ced}"),
                    monto: 1_000_000.0,
                },
            )
            .unwrap();
    }

    // ── Créditos en enero 2025 (colocación alta para que el PPCFC autorice) ──
    // Ana debe $1.6M (fiador Beto) y Carla $1.6M (fiador Darío). Efectivo $0.8M, cartera $3.2M
    // → colocación 80%.
    creditos
        .registrar_desembolso(
            &banco,
            NuevoDesembolso {
                solicitud_id: None,
                socio_id: ana.clone(),
                monto: 1_800_000.0,
                plazo_cuotas: 12,
                destino: DestinoCredito::Productivo,
                fiadores: vec![FiadorSolicitud { cedula: "222".into(), acciones_comprometidas: 36.0 }],
                fecha: Some("2025-01-20".into()),
            },
        )
        .unwrap();
    creditos
        .registrar_desembolso(
            &banco,
            NuevoDesembolso {
                solicitud_id: None,
                socio_id: carla.clone(),
                monto: 1_800_000.0,
                plazo_cuotas: 12,
                destino: DestinoCredito::Productivo,
                fiadores: vec![FiadorSolicitud { cedula: "444".into(), acciones_comprometidas: 36.0 }],
                fecha: Some("2025-01-21".into()),
            },
        )
        .unwrap();

    // ── Ingreso de intereses cada mes + cierre en orden (ene..jun 2025) ──
    for mes in 1..=6u32 {
        // Interés ordinario del mes (ingreso repartible), salvo enero para que sólo sea capital.
        if mes > 1 {
            caja.registrar_asiento_de_modulo(
                &banco,
                CodigoOperacion::InteresOrdinario,
                format!("2025-{mes:02}-15"),
                40_000.0,
                format!("Interés del mes {mes:02}"),
                Some(ana.clone()),
                None,
            )
            .unwrap();
        }
        cierre
            .ejecutar_cierre(&banco, &format!("2025-{mes:02}-01"), "Siembra Demo")
            .expect("no se pudo cerrar el mes");
    }

    println!("\n✅ Listo. Se creó «{nombre}» (id {banco}) con:");
    println!("   • 4 socios, 400 acciones, 2 créditos vigentes");
    println!("   • 6 meses cerrados (ene–jun 2025) → el PPCFC ya es calculable (colocación ~80%)");
    println!("   • intereses devengados → reparto de ganancias con datos");
    println!("\nAbre la app: el banco aparece en la lista. Prueba:");
    println!("   - Acciones → Compra: verás el cupo del mes (PPCFC).");
    println!("   - Reportes / Cuadre: el balance con historia.");
    println!("   - Respaldo: crea un backup de este banco y luego restáuralo.");
}
