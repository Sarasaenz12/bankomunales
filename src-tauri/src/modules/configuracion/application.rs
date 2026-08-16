use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::error::AppError;

use crate::modules::auditoria::domain::{tipo_accion, AuditoriaPort, EntradaAuditoria};

use super::domain::{
    Configuracion, ConfiguracionPort, DatosGenerales, RETENCION_MINIMA_FONDO_PCT,
};

/// Capa de Aplicación/Servicios del módulo de Configuración.
/// Orquesta los casos de uso RF-09 a RF-14 validando Reglas de Negocio (RN-07, RN-08).
pub struct ConfigService {
    pub auditoria: Arc<dyn AuditoriaPort>,
    config: Arc<dyn ConfiguracionPort>,
}

impl ConfigService {
    pub fn new(config: Arc<dyn ConfiguracionPort>, auditoria: Arc<dyn AuditoriaPort>) -> Self {
        Self { config, auditoria }
    }

    /// RF-114: historial completo de Auditoría.
    pub fn listar_auditoria(&self, banco_id: &str) -> Result<Vec<EntradaAuditoria>, AppError> {
        self.auditoria.listar(banco_id)
    }

    /// RF-11/RF-12/RF-13: obtiene los parámetros editables del Bankomunal activo.
    pub fn obtener_configuracion(&self, banco_id: &str) -> Result<Configuracion, AppError> {
        self.config.obtener(banco_id)
    }

    /// RF-09/RF-10/RF-11: obtiene los Datos Generales de solo consulta con contadores.
    pub fn obtener_datos_generales(&self, banco_id: &str) -> Result<DatosGenerales, AppError> {
        self.config.obtener_datos_generales(banco_id)
    }

    /// RF-12/RF-13/RF-14 + RF-112/RF-113: actualiza la configuración validando RN y
    /// dejando registro en Auditoría con quien la realiza.
    pub fn actualizar_configuracion(
        &self,
        banco_id: &str,
        nueva: &Configuracion,
        nombre_quien_realiza: &str,
        motivo: &str,
    ) -> Result<Configuracion, AppError> {
        // Los dos fondos retienen un % de la ganancia del mes (D-11). Cada uno se valida
        // contra su propio rango; el tope del Fondo de Reserva para Incobrables es otra
        // magnitud (D-04) —hasta dónde crece el saldo acumulado— y no limita la retención.
        // El reglamento fijo exige apartar "no menor al 5%" en cada fondo, así que el
        // mínimo no es 0: un Bankomunal no puede dejar de alimentarlos.
        for (campo, valor) in [
            ("Fondo para Gastos", nueva.pct_fondo_gastos),
            ("Fondo de Reserva para Incobrables", nueva.pct_fondo_incobrables),
        ] {
            if !(RETENCION_MINIMA_FONDO_PCT..=100.0).contains(&valor) {
                return Err(AppError::OperacionNoPermitida(format!(
                    "El % de retención mensual del {campo} debe estar entre \
                     {RETENCION_MINIMA_FONDO_PCT}% y 100% (reglamento fijo)"
                )));
            }
        }
        // RN-07 + RN-08: entre los dos fondos no se puede retener más que la ganancia del
        // mes; lo que sobre es lo que se reparte a los socios.
        let retencion_total = nueva.pct_fondo_gastos + nueva.pct_fondo_incobrables;
        if retencion_total > 100.0 {
            return Err(AppError::OperacionNoPermitida(format!(
                "La retención combinada de los dos fondos es {retencion_total}% y no puede \
                 superar el 100% de las ganancias del mes (RN-07, RN-08)"
            )));
        }
        if !(0.0..=100.0).contains(&nueva.tope_reserva_incobrables_pct) {
            return Err(AppError::OperacionNoPermitida(
                "El tope del Fondo de Reserva para Incobrables debe estar entre 0% y 100% \
                 del capital en acciones (RN-08)"
                    .into(),
            ));
        }
        // RN-04: la garantía mínima combinada (socio + fiador) debe cubrir al menos el 40%.
        if nueva.pct_garantia_socio + nueva.pct_garantia_fiador < 40.0 {
            return Err(AppError::OperacionNoPermitida(
                "La garantía mínima combinada (socio + fiador) debe ser al menos 40% (RN-04)".into(),
            ));
        }
        if nueva.valor_nominal <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El valor nominal de la acción debe ser mayor a cero (RN-13)".into(),
            ));
        }
        // RN-09: los % autorizados a vender son proporciones del total de acciones, y el
        // tramo alto de PPCFC no puede autorizar menos que el tramo bajo.
        for (campo, valor) in [
            ("tramo bajo", nueva.ppcfc_venta_rango1_pct),
            ("tramo alto", nueva.ppcfc_venta_rango2_pct),
        ] {
            if !(0.0..=100.0).contains(&valor) {
                return Err(AppError::OperacionNoPermitida(format!(
                    "El % de acciones autorizado a vender en el {campo} del PPCFC debe \
                     estar entre 0% y 100% (RN-09)"
                )));
            }
        }
        if nueva.ppcfc_venta_rango2_pct < nueva.ppcfc_venta_rango1_pct {
            return Err(AppError::OperacionNoPermitida(
                "El % autorizado a vender con el PPCFC en el tramo alto no puede ser menor \
                 que el del tramo bajo (RN-09)"
                    .into(),
            ));
        }
        // RN-15: el tope individual mensual es un % del cupo que el PPCFC autorizó ese
        // mes, no del capital total del Bankomunal.
        if !(0.0..=100.0).contains(&nueva.tope_individual_mensual_pct) {
            return Err(AppError::OperacionNoPermitida(
                "El tope individual mensual debe estar entre 0% y 100% del cupo autorizado \
                 del mes (RN-15)"
                    .into(),
            ));
        }
        if nueva.plazo_maximo_cuotas <= 0 {
            return Err(AppError::OperacionNoPermitida(
                "El plazo máximo de un crédito debe ser de al menos 1 cuota".into(),
            ));
        }
        if nueva.monto_maximo_credito <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto máximo de un crédito debe ser mayor a cero".into(),
            ));
        }
        for (campo, valor) in [
            ("ordinario", nueva.tasa_interes_ordinario),
            ("de mora", nueva.tasa_interes_mora),
        ] {
            if !(0.0..=100.0).contains(&valor) {
                return Err(AppError::OperacionNoPermitida(format!(
                    "La tasa de interés {campo} debe estar entre 0% y 100%"
                )));
            }
        }

        let anterior = self.config.obtener(banco_id)?;
        self.config.actualizar(banco_id, nueva)?;

        // Auditoría campo a campo de lo que cambió (RF-112/RF-113).
        let cambios = diff_config(&anterior, nueva);
        for (campo, antes, despues) in cambios {
            let entrada = EntradaAuditoria {
                id: Uuid::new_v4().to_string(),
                fecha: Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                nombre_quien_realiza: nombre_quien_realiza.to_string(),
                entidad_afectada: "configuracion".into(),
                campo_modificado: Some(campo),
                valor_anterior: Some(antes),
                valor_nuevo: Some(despues),
                motivo: Some(motivo.to_string()),
                tipo_accion: tipo_accion::MODIFICACION.into(),
            };
            self.auditoria.registrar(banco_id, &entrada)?;
        }

        Ok(nueva.clone())
    }
}

fn diff_config(anterior: &Configuracion, nueva: &Configuracion) -> Vec<(String, String, String)> {
    let mut cambios = Vec::new();
    macro_rules! cmp {
        ($campo:literal, $a:expr, $n:expr) => {
            if $a != $n {
                cambios.push(($campo.to_string(), format!("{}", $a), format!("{}", $n)));
            }
        };
    }
    cmp!("valor_nominal", anterior.valor_nominal, nueva.valor_nominal);
    cmp!(
        "pct_garantia_socio",
        anterior.pct_garantia_socio,
        nueva.pct_garantia_socio
    );
    cmp!(
        "pct_garantia_fiador",
        anterior.pct_garantia_fiador,
        nueva.pct_garantia_fiador
    );
    cmp!(
        "pct_fondo_gastos",
        anterior.pct_fondo_gastos,
        nueva.pct_fondo_gastos
    );
    cmp!(
        "pct_fondo_incobrables",
        anterior.pct_fondo_incobrables,
        nueva.pct_fondo_incobrables
    );
    cmp!(
        "tope_reserva_incobrables_pct",
        anterior.tope_reserva_incobrables_pct,
        nueva.tope_reserva_incobrables_pct
    );
    cmp!(
        "ppcfc_venta_rango1_pct",
        anterior.ppcfc_venta_rango1_pct,
        nueva.ppcfc_venta_rango1_pct
    );
    cmp!(
        "ppcfc_venta_rango2_pct",
        anterior.ppcfc_venta_rango2_pct,
        nueva.ppcfc_venta_rango2_pct
    );
    cmp!(
        "tope_individual_mensual_pct",
        anterior.tope_individual_mensual_pct,
        nueva.tope_individual_mensual_pct
    );
    cmp!(
        "plazo_maximo_cuotas",
        anterior.plazo_maximo_cuotas as f64,
        nueva.plazo_maximo_cuotas as f64
    );
    cmp!(
        "tasa_interes_ordinario",
        anterior.tasa_interes_ordinario,
        nueva.tasa_interes_ordinario
    );
    cmp!(
        "tasa_interes_mora",
        anterior.tasa_interes_mora,
        nueva.tasa_interes_mora
    );
    cmp!(
        "monto_maximo_credito",
        anterior.monto_maximo_credito,
        nueva.monto_maximo_credito
    );
    cambios
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::configuracion::data::SqliteConfiguracion;
    use uuid::Uuid;

    fn test_service() -> (ConfigService, String) {
        let dir = std::env::temp_dir().join(format!("bkn_cfg_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = crate::core::db::DbManager::new(dir);
        let banco_id = Uuid::new_v4().to_string();
        // Crea el .db del Banco para poder operar (como haría ya una app real).
        db.open_banco_db(&banco_id).unwrap();
        let svc = ConfigService::new(
            Arc::new(SqliteConfiguracion::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db)),
        );
        (svc, banco_id)
    }

    #[test]
    fn obtener_siembra_defaults() {
        let (svc, banco_id) = test_service();
        let c = svc.obtener_configuracion(&banco_id).unwrap();
        // RN-13: el reglamento recomienda $10.000 como valor de la acción.
        assert_eq!(c.valor_nominal, 10000.0);
        assert_eq!(c.pct_fondo_gastos, 10.0);
        assert_eq!(c.pct_fondo_incobrables, 10.0);
        assert_eq!(c.tope_reserva_incobrables_pct, 20.0);
        // RN-09: son los % AUTORIZADOS A VENDER (10% y 15%), no los umbrales 80/90.
        assert_eq!(c.ppcfc_venta_rango1_pct, 10.0);
        assert_eq!(c.ppcfc_venta_rango2_pct, 15.0);
        assert_eq!(c.tope_individual_mensual_pct, 20.0);
    }

    /// El reglamento fijo exige apartar "no menor al 5%" en cada fondo.
    #[test]
    fn actualizar_rechaza_retencion_por_debajo_del_minimo() {
        let (svc, banco_id) = test_service();
        let base = svc.obtener_configuracion(&banco_id).unwrap();

        for menor in [0.0, 4.9] {
            let mut c = base.clone();
            c.pct_fondo_gastos = menor;
            assert!(
                svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_err(),
                "un {menor}% al Fondo de Gastos debe rechazarse"
            );

            let mut c = base.clone();
            c.pct_fondo_incobrables = menor;
            assert!(svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_err());
        }

        // El mínimo exacto sí se acepta.
        let mut c = base.clone();
        c.pct_fondo_gastos = 5.0;
        c.pct_fondo_incobrables = 5.0;
        assert!(svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_ok());
    }

    /// RN-09: el tramo alto del PPCFC no puede autorizar a vender menos que el bajo.
    #[test]
    fn actualizar_rechaza_tramos_de_ppcfc_invertidos() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.ppcfc_venta_rango1_pct = 15.0;
        c.ppcfc_venta_rango2_pct = 10.0;
        let err = svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
    }

    /// Las condiciones de crédito (RF-11) no pueden guardarse en valores imposibles.
    #[test]
    fn actualizar_rechaza_condiciones_de_credito_invalidas() {
        let (svc, banco_id) = test_service();
        let base = svc.obtener_configuracion(&banco_id).unwrap();

        let mut c = base.clone();
        c.plazo_maximo_cuotas = 0;
        assert!(svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_err());

        let mut c = base.clone();
        c.monto_maximo_credito = 0.0;
        assert!(svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_err());

        let mut c = base.clone();
        c.tasa_interes_mora = 120.0;
        assert!(svc.actualizar_configuracion(&banco_id, &c, "V", "x").is_err());
    }

    /// D-04: la retención mensual del Fondo de Reserva y el tope de su saldo acumulado
    /// son parámetros distintos. Una retención del 25% es legítima y no debe rechazarse
    /// por chocar con el tope del 20%, que aplica a otra magnitud.
    #[test]
    fn retencion_mensual_y_tope_de_reserva_son_independientes() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.pct_fondo_incobrables = 25.0;
        c.tope_reserva_incobrables_pct = 30.0;
        let guardada = svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .unwrap();
        assert_eq!(guardada.pct_fondo_incobrables, 25.0);
        assert_eq!(guardada.tope_reserva_incobrables_pct, 30.0);
        // Y persiste al releer desde el .db.
        let releida = svc.obtener_configuracion(&banco_id).unwrap();
        assert_eq!(releida.tope_reserva_incobrables_pct, 30.0);
    }

    /// D-11: los dos fondos se guardan, se releen y se auditan por separado.
    #[test]
    fn los_dos_fondos_son_independientes() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.pct_fondo_gastos = 12.0;
        c.pct_fondo_incobrables = 8.0;
        svc.actualizar_configuracion(&banco_id, &c, "Contable", "ajuste")
            .unwrap();

        let releida = svc.obtener_configuracion(&banco_id).unwrap();
        assert_eq!(releida.pct_fondo_gastos, 12.0);
        assert_eq!(releida.pct_fondo_incobrables, 8.0);

        // Cada fondo se audita con su propio nombre de campo (RF-112).
        let campos: Vec<String> = svc
            .listar_auditoria(&banco_id)
            .unwrap()
            .into_iter()
            .filter_map(|e| e.campo_modificado)
            .collect();
        assert!(campos.contains(&"pct_fondo_gastos".to_string()));
        assert!(campos.contains(&"pct_fondo_incobrables".to_string()));
    }

    /// Ejemplo del cliente: de $100.000 de ganancia, 10% a Gastos y 10% a Incobrables
    /// dejan $80.000 para repartir entre los socios.
    #[test]
    fn con_los_defaults_se_reparte_el_80_por_ciento() {
        let (svc, banco_id) = test_service();
        let c = svc.obtener_configuracion(&banco_id).unwrap();
        let ganancia = 100_000.0;
        let a_gastos = ganancia * c.pct_fondo_gastos / 100.0;
        let a_incobrables = ganancia * c.pct_fondo_incobrables / 100.0;
        assert_eq!(a_gastos, 10_000.0);
        assert_eq!(a_incobrables, 10_000.0);
        assert_eq!(ganancia - a_gastos - a_incobrables, 80_000.0);
    }

    #[test]
    fn actualizar_rechaza_retencion_combinada_mayor_a_100() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.pct_fondo_gastos = 60.0;
        c.pct_fondo_incobrables = 50.0;
        let err = svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
    }

    #[test]
    fn actualizar_rechaza_porcentajes_fuera_de_rango() {
        let (svc, banco_id) = test_service();
        let base = svc.obtener_configuracion(&banco_id).unwrap();

        let mut c = base.clone();
        c.pct_fondo_gastos = -5.0;
        assert!(svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .is_err());

        let mut c = base.clone();
        c.tope_reserva_incobrables_pct = 150.0;
        assert!(svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .is_err());
    }

    #[test]
    fn actualizar_rechaza_garantia_menor_40() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.pct_garantia_socio = 10.0;
        c.pct_garantia_fiador = 10.0;
        let err = svc
            .actualizar_configuracion(&banco_id, &c, "Verificador", "cambio")
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
    }

    #[test]
    fn actualizar_audita_cambios() {
        let (svc, banco_id) = test_service();
        let mut c = svc.obtener_configuracion(&banco_id).unwrap();
        c.valor_nominal = 200000.0;
        c.tasa_interes_ordinario = 2.5;
        let guardada = svc
            .actualizar_configuracion(&banco_id, &c, "Contable", "Ajuste semestral")
            .unwrap();
        assert_eq!(guardada.valor_nominal, 200000.0);
        let audit = svc.listar_auditoria(&banco_id).unwrap();
        // 2 campos cambiaron → 2 entradas de auditoría.
        assert_eq!(audit.len(), 2);
        assert!(audit
            .iter()
            .all(|e| e.nombre_quien_realiza == "Contable" && e.tipo_accion == "MODIFICACION"));
    }

    #[test]
    fn datos_generales_devuelve_contadores() {
        let (svc, banco_id) = test_service();
        let d = svc.obtener_datos_generales(&banco_id).unwrap();
        assert_eq!(d.numero_creditos_otorgados, 0);
        assert_eq!(d.monto_total_creditos, 0.0);
        assert_eq!(d.numero_acciones_vendidas, 0);
        assert_eq!(d.saldo_fondo_gastos, 0.0);
        assert_eq!(d.saldo_fondo_incobrables, 0.0);
    }

    /// RF-09: los Datos Generales deben mostrar el nombre, ubicación, moneda y fecha
    /// de creación reales con los que se registró el Bankomunal —no un fragmento del
    /// UUID como nombre ni una cadena sin sentido como fecha.
    #[test]
    fn datos_generales_toman_la_identidad_del_catalogo() {
        use crate::modules::auth::application::AuthService;
        use crate::modules::auth::data::{BcryptHasher, SqliteAppSettings, SqliteBancoCatalogo};
        use crate::modules::auth::domain::NuevoBankomunal;

        let dir = std::env::temp_dir().join(format!("bkn_ident_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = crate::core::db::DbManager::new(dir);

        // Se crea el Bankomunal por el flujo real (RF-03), no insertando a mano.
        let auth = AuthService::new(
            Arc::new(SqliteAppSettings::new(db.clone())),
            Arc::new(BcryptHasher),
            Arc::new(SqliteBancoCatalogo::new(db.clone())),
            db.clone(),
        );
        auth.inicializar().unwrap();
        let banco = auth
            .crear_bankomunal(NuevoBankomunal {
                nombre: "Bankomunal Pijao".into(),
                ubicacion: "Pijao, Quindío".into(),
                moneda: "COP".into(),
            })
            .unwrap();

        let svc = ConfigService::new(
            Arc::new(SqliteConfiguracion::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db)),
        );
        let d = svc.obtener_datos_generales(&banco.id).unwrap();

        assert_eq!(d.nombre, "Bankomunal Pijao");
        assert_eq!(d.ubicacion, "Pijao, Quindío");
        assert_eq!(d.moneda, "COP");
        assert_eq!(d.fecha_creacion, banco.fecha_creacion);
        // La fecha debe ser una fecha de verdad, no un fragmento del UUID.
        assert!(
            d.fecha_creacion.starts_with("20") && d.fecha_creacion.contains('-'),
            "fecha_creacion inválida: {}",
            d.fecha_creacion
        );
    }
}