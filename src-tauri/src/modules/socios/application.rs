use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::error::AppError;

use super::domain::{
    DatosSocio, EstatusSocio, Socio, SocioPort, MAX_PROTEGIDOS, MAX_SOCIOS,
};

/// Capa de Aplicación/Servicios del módulo de Socios.
/// Orquesta los casos de uso CU-05 y CU-06 (RF-15 a RF-21) usando únicamente el puerto
/// `SocioPort`, nunca SQLite directamente.
pub struct SocioService {
    socios: Arc<dyn SocioPort>,
}

impl SocioService {
    pub fn new(socios: Arc<dyn SocioPort>) -> Self {
        Self { socios }
    }

    /// RF-15/RF-16/RF-17/RF-18/RF-20/RF-21 (CU-05): registra un socio nuevo.
    ///
    /// El estatus inicial es siempre ACTIVO y la fecha de ingreso es la de hoy: ninguno
    /// de los dos se toma del formulario.
    pub fn registrar(&self, banco_id: &str, datos: DatosSocio) -> Result<Socio, AppError> {
        let datos = normalizar(datos);
        self.validar(&datos)?;

        // RF-18: la cédula no puede repetirse dentro del mismo Bankomunal.
        if self
            .socios
            .buscar_por_cedula(banco_id, &datos.cedula)?
            .is_some()
        {
            return Err(AppError::CedulaDuplicada(datos.cedula));
        }

        // RN-01: el Bankomunal admite como máximo 19 socios. Sólo cuentan los activos:
        // un socio retirado libera su cupo.
        let activos = self.socios.contar_activos(banco_id)?;
        if activos >= MAX_SOCIOS {
            return Err(AppError::OperacionNoPermitida(format!(
                "El Bankomunal ya tiene {activos} socios activos y el máximo permitido es \
                 {MAX_SOCIOS} (RN-01)"
            )));
        }

        let socio = Socio {
            id: Uuid::new_v4().to_string(),
            cedula: datos.cedula,
            nombres: datos.nombres,
            apellidos: datos.apellidos,
            profesion: datos.profesion,
            direccion: datos.direccion,
            telefono: datos.telefono,
            celular: datos.celular,
            correo: datos.correo,
            estatus: EstatusSocio::Activo,
            fecha_ingreso: Utc::now().format("%Y-%m-%d").to_string(),
            fecha_retiro: None,
            saldo_incobrable: 0.0,
            beneficiario: datos.beneficiario,
            protegidos: datos.protegidos,
        };

        self.socios.crear(banco_id, &socio)?;
        Ok(socio)
    }

    /// RF-19 (CU-06): actualiza los datos de un socio existente.
    ///
    /// El estatus, la fecha de ingreso, la de retiro y el saldo incobrable se conservan:
    /// los gobiernan Liquidación y Créditos, no este formulario.
    pub fn actualizar(
        &self,
        banco_id: &str,
        id: &str,
        datos: DatosSocio,
    ) -> Result<Socio, AppError> {
        let datos = normalizar(datos);
        self.validar(&datos)?;

        let actual = self
            .socios
            .buscar_por_id(banco_id, id)?
            .ok_or(AppError::SocioNoEncontrado)?;

        // RF-18: si cambió la cédula, la nueva no puede ser la de otro socio.
        if datos.cedula != actual.cedula {
            if let Some(otro) = self.socios.buscar_por_cedula(banco_id, &datos.cedula)? {
                if otro.id != actual.id {
                    return Err(AppError::CedulaDuplicada(datos.cedula));
                }
            }
        }

        let socio = Socio {
            cedula: datos.cedula,
            nombres: datos.nombres,
            apellidos: datos.apellidos,
            profesion: datos.profesion,
            direccion: datos.direccion,
            telefono: datos.telefono,
            celular: datos.celular,
            correo: datos.correo,
            beneficiario: datos.beneficiario,
            protegidos: datos.protegidos,
            ..actual
        };

        self.socios.actualizar(banco_id, &socio)?;
        Ok(socio)
    }

    /// RF-19: consulta de un socio por su id.
    pub fn obtener(&self, banco_id: &str, id: &str) -> Result<Socio, AppError> {
        self.socios
            .buscar_por_id(banco_id, id)?
            .ok_or(AppError::SocioNoEncontrado)
    }

    /// RF-16: localizar por cédula, para saber si ya existe antes de registrarlo.
    pub fn buscar_por_cedula(
        &self,
        banco_id: &str,
        cedula: &str,
    ) -> Result<Option<Socio>, AppError> {
        self.socios.buscar_por_cedula(banco_id, cedula.trim())
    }

    /// Listado completo para la pantalla de Socios (RF-19).
    pub fn listar(&self, banco_id: &str) -> Result<Vec<Socio>, AppError> {
        self.socios.listar(banco_id)
    }

    /// Cuántos socios activos hay y cuántos caben todavía (RN-01).
    pub fn cupo(&self, banco_id: &str) -> Result<(usize, usize), AppError> {
        let activos = self.socios.contar_activos(banco_id)?;
        Ok((activos, MAX_SOCIOS.saturating_sub(activos)))
    }

    /// Validaciones comunes a registrar y actualizar.
    ///
    /// Sólo se exigen los datos que identifican a la persona. Profesión, dirección,
    /// teléfonos y correo se dejan opcionales a propósito: los socios son población
    /// rural y muchos no tienen correo electrónico, así que obligarlos llevaría a
    /// llenar el campo con datos falsos (RNF-04).
    fn validar(&self, datos: &DatosSocio) -> Result<(), AppError> {
        for (campo, valor) in [
            ("la cédula", &datos.cedula),
            ("los nombres", &datos.nombres),
            ("los apellidos", &datos.apellidos),
        ] {
            if valor.is_empty() {
                return Err(AppError::OperacionNoPermitida(format!(
                    "Debe indicar {campo} del socio"
                )));
            }
        }

        // El formulario ya restringe estos formatos, pero se revalidan aquí porque el
        // Dominio no puede asumir que esa pantalla sea la única entrada posible.
        if !datos.correo.is_empty() && !parece_correo(&datos.correo) {
            return Err(AppError::OperacionNoPermitida(
                "El correo electrónico debe tener el formato nombre@dominio.com".into(),
            ));
        }
        for (campo, valor) in [
            ("La cédula", &datos.cedula),
            ("El teléfono", &datos.telefono),
            ("El celular", &datos.celular),
        ] {
            if !valor.is_empty() && !solo_digitos(valor) {
                return Err(AppError::OperacionNoPermitida(format!(
                    "{campo} sólo admite números"
                )));
            }
        }

        // RF-21: la planilla contempla hasta 2 protegidos.
        if datos.protegidos.len() > MAX_PROTEGIDOS {
            return Err(AppError::OperacionNoPermitida(format!(
                "Sólo se pueden registrar hasta {MAX_PROTEGIDOS} protegidos por socio (RF-21)"
            )));
        }
        for p in &datos.protegidos {
            if p.nombre.trim().is_empty() || p.cedula.trim().is_empty() {
                return Err(AppError::OperacionNoPermitida(
                    "Cada protegido debe tener al menos nombre y cédula".into(),
                ));
            }
            if !solo_digitos(&p.cedula) || (!p.telefono.is_empty() && !solo_digitos(&p.telefono)) {
                return Err(AppError::OperacionNoPermitida(
                    "La cédula y el teléfono de los protegidos sólo admiten números".into(),
                ));
            }
        }
        if let Some(b) = &datos.beneficiario {
            if b.nombre.trim().is_empty() || b.cedula.trim().is_empty() {
                return Err(AppError::OperacionNoPermitida(
                    "El beneficiario en caso de muerte debe tener nombre y cédula (RF-20)".into(),
                ));
            }
            if !solo_digitos(&b.cedula) {
                return Err(AppError::OperacionNoPermitida(
                    "La cédula del beneficiario sólo admite números".into(),
                ));
            }
        }
        Ok(())
    }
}

fn solo_digitos(valor: &str) -> bool {
    !valor.is_empty() && valor.chars().all(|c| c.is_ascii_digit())
}

/// Comprobación mínima de que un correo tiene forma de correo: algo, una @, un dominio
/// y al menos un punto en él. No se valida contra el RFC completo a propósito — las
/// direcciones exóticas que eso rechazaría no existen aquí, y el patrón sería ilegible.
fn parece_correo(valor: &str) -> bool {
    let mut partes = valor.split('@');
    let (usuario, dominio) = match (partes.next(), partes.next(), partes.next()) {
        (Some(u), Some(d), None) => (u, d),
        _ => return false, // ninguna @, o más de una
    };
    !usuario.is_empty()
        && dominio.contains('.')
        && !dominio.starts_with('.')
        && !dominio.ends_with('.')
        && !valor.chars().any(char::is_whitespace)
}

/// Limpia espacios sobrantes antes de validar y guardar, para que " 123 " y "123" no
/// se registren como dos cédulas distintas.
fn normalizar(mut datos: DatosSocio) -> DatosSocio {
    datos.cedula = datos.cedula.trim().to_string();
    datos.nombres = datos.nombres.trim().to_string();
    datos.apellidos = datos.apellidos.trim().to_string();
    datos.profesion = datos.profesion.trim().to_string();
    datos.direccion = datos.direccion.trim().to_string();
    datos.telefono = datos.telefono.trim().to_string();
    datos.celular = datos.celular.trim().to_string();
    datos.correo = datos.correo.trim().to_string();

    // Un beneficiario totalmente vacío equivale a no haberlo diligenciado (es opcional).
    datos.beneficiario = datos.beneficiario.and_then(|mut b| {
        b.nombre = b.nombre.trim().to_string();
        b.cedula = b.cedula.trim().to_string();
        b.parentesco = b
            .parentesco
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty());
        if b.nombre.is_empty() && b.cedula.is_empty() {
            None
        } else {
            Some(b)
        }
    });

    // Ídem para las filas de protegidos que quedaron en blanco en el formulario.
    datos.protegidos = datos
        .protegidos
        .into_iter()
        .map(|mut p| {
            p.nombre = p.nombre.trim().to_string();
            p.cedula = p.cedula.trim().to_string();
            p.parentesco = p.parentesco.trim().to_string();
            p.telefono = p.telefono.trim().to_string();
            p
        })
        .filter(|p| !(p.nombre.is_empty() && p.cedula.is_empty()))
        .collect();

    datos
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::socios::data::SqliteSocios;
    use crate::modules::socios::domain::{Beneficiario, Protegido};

    fn test_service() -> (SocioService, String) {
        let dir = std::env::temp_dir().join(format!("bkn_socios_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let banco_id = Uuid::new_v4().to_string();
        db.open_banco_db(&banco_id).unwrap();
        (SocioService::new(Arc::new(SqliteSocios::new(db))), banco_id)
    }

    fn datos(cedula: &str, nombres: &str) -> DatosSocio {
        DatosSocio {
            cedula: cedula.into(),
            nombres: nombres.into(),
            apellidos: "Ríos".into(),
            profesion: "Docente".into(),
            direccion: "Calle 1".into(),
            telefono: "7460000".into(),
            celular: "3000000000".into(),
            correo: "ana@ejemplo.com".into(),
            beneficiario: None,
            protegidos: vec![],
        }
    }

    #[test]
    fn registrar_crea_socio_activo_con_fecha_de_ingreso() {
        let (svc, banco) = test_service();
        let socio = svc.registrar(&banco, datos("123", "Ana")).unwrap();

        assert_eq!(socio.estatus, EstatusSocio::Activo);
        assert!(!socio.id.is_empty());
        assert_eq!(socio.fecha_retiro, None);
        assert_eq!(socio.saldo_incobrable, 0.0);
        // fecha_ingreso la pone el sistema con formato ISO.
        assert_eq!(socio.fecha_ingreso.len(), 10);
        assert_eq!(socio.nombre_completo(), "Ana Ríos");
    }

    #[test]
    fn registrar_persiste_y_se_puede_releer() {
        let (svc, banco) = test_service();
        let creado = svc.registrar(&banco, datos("123", "Ana")).unwrap();
        let leido = svc.obtener(&banco, &creado.id).unwrap();
        assert_eq!(leido, creado);
    }

    /// RF-18: la cédula es única dentro del Bankomunal.
    #[test]
    fn registrar_rechaza_cedula_duplicada() {
        let (svc, banco) = test_service();
        svc.registrar(&banco, datos("123", "Ana")).unwrap();
        let err = svc.registrar(&banco, datos("123", "Otro")).unwrap_err();
        assert!(matches!(err, AppError::CedulaDuplicada(c) if c == "123"));
        assert_eq!(svc.listar(&banco).unwrap().len(), 1);
    }

    /// Los espacios no deben permitir colar una cédula repetida.
    #[test]
    fn la_cedula_se_normaliza_antes_de_comparar() {
        let (svc, banco) = test_service();
        svc.registrar(&banco, datos("123", "Ana")).unwrap();
        let err = svc.registrar(&banco, datos("  123  ", "Otro")).unwrap_err();
        assert!(matches!(err, AppError::CedulaDuplicada(_)));
    }

    #[test]
    fn registrar_exige_cedula_nombres_y_apellidos() {
        let (svc, banco) = test_service();

        for (campo, mut d) in [
            ("cedula", datos("", "Ana")),
            ("nombres", datos("1", "")),
            ("apellidos", datos("2", "Ana")),
        ] {
            if campo == "apellidos" {
                d.apellidos = "   ".into();
            }
            assert!(
                svc.registrar(&banco, d).is_err(),
                "debe exigirse el campo {campo}"
            );
        }
        assert!(svc.listar(&banco).unwrap().is_empty());
    }

    /// Los datos de contacto son opcionales: el socio rural puede no tener correo.
    #[test]
    fn los_datos_de_contacto_son_opcionales() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.profesion = String::new();
        d.direccion = String::new();
        d.telefono = String::new();
        d.celular = String::new();
        d.correo = String::new();
        assert!(svc.registrar(&banco, d).is_ok());
    }

    /// RN-01: el Bankomunal admite como máximo 19 socios.
    #[test]
    fn registrar_respeta_el_maximo_de_socios() {
        let (svc, banco) = test_service();
        for i in 0..MAX_SOCIOS {
            svc.registrar(&banco, datos(&format!("100{i}"), "Socio"))
                .unwrap();
        }
        let (activos, libres) = svc.cupo(&banco).unwrap();
        assert_eq!(activos, MAX_SOCIOS);
        assert_eq!(libres, 0);

        let err = svc.registrar(&banco, datos("9999", "Veinte")).unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
        assert_eq!(svc.listar(&banco).unwrap().len(), MAX_SOCIOS);
    }

    /// RF-20/RF-21: beneficiario y protegidos se guardan y se releen con el socio.
    #[test]
    fn registrar_guarda_beneficiario_y_protegidos() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.beneficiario = Some(Beneficiario {
            nombre: "Luis Ríos".into(),
            cedula: "999".into(),
            parentesco: Some("Hijo".into()),
        });
        d.protegidos = vec![
            Protegido {
                nombre: "Sara Ríos".into(),
                cedula: "111".into(),
                parentesco: "Hija".into(),
                telefono: "3001112222".into(),
            },
            Protegido {
                nombre: "Pedro Ríos".into(),
                cedula: "222".into(),
                parentesco: "Esposo".into(),
                telefono: "3003334444".into(),
            },
        ];

        let creado = svc.registrar(&banco, d).unwrap();
        let leido = svc.obtener(&banco, &creado.id).unwrap();

        assert_eq!(leido.beneficiario.as_ref().unwrap().nombre, "Luis Ríos");
        assert_eq!(leido.protegidos.len(), 2);
        assert_eq!(leido.protegidos[0].telefono, "3001112222");
    }

    #[test]
    fn registrar_rechaza_correo_con_formato_invalido() {
        let (svc, banco) = test_service();
        for malo in [
            "sin-arroba.com",
            "@sindominio.com",
            "ana@sinpunto",
            "ana@@doble.com",
            "con espacio@dominio.com",
            "ana@.com",
            "ana@dominio.",
        ] {
            let mut d = datos("123", "Ana");
            d.correo = malo.into();
            assert!(
                svc.registrar(&banco, d).is_err(),
                "el correo «{malo}» debería rechazarse"
            );
        }
    }

    #[test]
    fn registrar_acepta_correos_validos_y_tambien_vacio() {
        let (svc, banco) = test_service();
        for (i, bueno) in ["ana@ejemplo.com", "a.b+c@sub.dominio.co", ""].iter().enumerate() {
            let mut d = datos(&format!("{i}"), "Ana");
            d.correo = (*bueno).into();
            assert!(svc.registrar(&banco, d).is_ok(), "«{bueno}» debería aceptarse");
        }
    }

    #[test]
    fn registrar_rechaza_texto_en_los_campos_numericos() {
        let (svc, banco) = test_service();

        let mut d = datos("abc123", "Ana");
        assert!(svc.registrar(&banco, d).is_err(), "cédula con letras");

        d = datos("123", "Ana");
        d.telefono = "746-0000".into();
        assert!(svc.registrar(&banco, d).is_err(), "teléfono con guiones");

        d = datos("123", "Ana");
        d.celular = "300 111 2222".into();
        assert!(svc.registrar(&banco, d).is_err(), "celular con espacios");
    }

    #[test]
    fn registrar_rechaza_cedula_no_numerica_en_allegados() {
        let (svc, banco) = test_service();

        let mut d = datos("123", "Ana");
        d.beneficiario = Some(Beneficiario {
            nombre: "Luis".into(),
            cedula: "A-999".into(),
            parentesco: None,
        });
        assert!(svc.registrar(&banco, d).is_err());

        let mut d = datos("124", "Ana");
        d.protegidos = vec![Protegido {
            nombre: "Sara".into(),
            cedula: "111".into(),
            parentesco: "Hija".into(),
            telefono: "no-tengo".into(),
        }];
        assert!(svc.registrar(&banco, d).is_err());
    }

    #[test]
    fn registrar_rechaza_mas_de_dos_protegidos() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.protegidos = (0..3)
            .map(|i| Protegido {
                nombre: format!("P{i}"),
                cedula: format!("c{i}"),
                parentesco: "Hijo".into(),
                telefono: String::new(),
            })
            .collect();
        assert!(svc.registrar(&banco, d).is_err());
    }

    /// Un beneficiario que quedó en blanco en el formulario no debe guardarse vacío.
    #[test]
    fn beneficiario_en_blanco_equivale_a_no_registrarlo() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.beneficiario = Some(Beneficiario {
            nombre: "  ".into(),
            cedula: String::new(),
            parentesco: None,
        });
        let creado = svc.registrar(&banco, d).unwrap();
        assert_eq!(creado.beneficiario, None);
    }

    /// RF-20: si se diligencia a medias, se avisa en vez de guardar un dato inútil.
    #[test]
    fn beneficiario_a_medias_se_rechaza() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.beneficiario = Some(Beneficiario {
            nombre: "Luis".into(),
            cedula: String::new(),
            parentesco: None,
        });
        assert!(svc.registrar(&banco, d).is_err());
    }

    /// RF-19: actualizar cambia los datos editables y deja intacto lo que gobiernan
    /// otros módulos (estatus, fechas, saldo incobrable).
    #[test]
    fn actualizar_conserva_estatus_y_fecha_de_ingreso() {
        let (svc, banco) = test_service();
        let creado = svc.registrar(&banco, datos("123", "Ana")).unwrap();

        let mut d = datos("123", "Ana María");
        d.direccion = "Nueva dirección 45".into();
        let actualizado = svc.actualizar(&banco, &creado.id, d).unwrap();

        assert_eq!(actualizado.nombres, "Ana María");
        assert_eq!(actualizado.direccion, "Nueva dirección 45");
        assert_eq!(actualizado.id, creado.id);
        assert_eq!(actualizado.estatus, creado.estatus);
        assert_eq!(actualizado.fecha_ingreso, creado.fecha_ingreso);
        assert_eq!(actualizado.saldo_incobrable, creado.saldo_incobrable);
    }

    /// Debe poder guardarse un socio sin tocar su cédula (no chocar consigo mismo).
    #[test]
    fn actualizar_sin_cambiar_cedula_no_choca_consigo_mismo() {
        let (svc, banco) = test_service();
        let creado = svc.registrar(&banco, datos("123", "Ana")).unwrap();
        assert!(svc.actualizar(&banco, &creado.id, datos("123", "Ana")).is_ok());
    }

    /// RF-18: tampoco al actualizar puede quedar la cédula de otro socio.
    #[test]
    fn actualizar_rechaza_tomar_la_cedula_de_otro() {
        let (svc, banco) = test_service();
        svc.registrar(&banco, datos("111", "Ana")).unwrap();
        let segundo = svc.registrar(&banco, datos("222", "Luis")).unwrap();

        let err = svc
            .actualizar(&banco, &segundo.id, datos("111", "Luis"))
            .unwrap_err();
        assert!(matches!(err, AppError::CedulaDuplicada(_)));
    }

    #[test]
    fn actualizar_reemplaza_beneficiario_y_protegidos_sin_duplicarlos() {
        let (svc, banco) = test_service();
        let mut d = datos("123", "Ana");
        d.protegidos = vec![Protegido {
            nombre: "Sara".into(),
            cedula: "111".into(),
            parentesco: "Hija".into(),
            telefono: String::new(),
        }];
        let creado = svc.registrar(&banco, d).unwrap();

        let mut d2 = datos("123", "Ana");
        d2.protegidos = vec![Protegido {
            nombre: "Pedro".into(),
            cedula: "222".into(),
            parentesco: "Esposo".into(),
            telefono: String::new(),
        }];
        svc.actualizar(&banco, &creado.id, d2).unwrap();

        let leido = svc.obtener(&banco, &creado.id).unwrap();
        assert_eq!(leido.protegidos.len(), 1, "no deben acumularse los anteriores");
        assert_eq!(leido.protegidos[0].nombre, "Pedro");
    }

    #[test]
    fn obtener_socio_inexistente_devuelve_error() {
        let (svc, banco) = test_service();
        assert!(matches!(
            svc.obtener(&banco, "no-existe").unwrap_err(),
            AppError::SocioNoEncontrado
        ));
        assert!(matches!(
            svc.actualizar(&banco, "no-existe", datos("1", "X")).unwrap_err(),
            AppError::SocioNoEncontrado
        ));
    }

    /// RF-16: durante la compra de acciones se busca por cédula para saber si el socio
    /// ya existe o hay que registrarlo.
    #[test]
    fn buscar_por_cedula_localiza_o_devuelve_none() {
        let (svc, banco) = test_service();
        svc.registrar(&banco, datos("123", "Ana")).unwrap();

        assert!(svc.buscar_por_cedula(&banco, "123").unwrap().is_some());
        assert!(svc.buscar_por_cedula(&banco, " 123 ").unwrap().is_some());
        assert!(svc.buscar_por_cedula(&banco, "999").unwrap().is_none());
    }

    #[test]
    fn listar_ordena_por_apellidos_y_nombres() {
        let (svc, banco) = test_service();
        let mut zorro = datos("1", "Ana");
        zorro.apellidos = "Zorro".into();
        let mut abad = datos("2", "Luis");
        abad.apellidos = "Abad".into();
        svc.registrar(&banco, zorro).unwrap();
        svc.registrar(&banco, abad).unwrap();

        let lista = svc.listar(&banco).unwrap();
        assert_eq!(lista[0].apellidos, "Abad");
        assert_eq!(lista[1].apellidos, "Zorro");
    }

    /// RF-08: dos Bankomunales del mismo computador no comparten socios, ni siquiera
    /// con la misma cédula.
    #[test]
    fn los_socios_no_se_mezclan_entre_bankomunales() {
        let dir = std::env::temp_dir().join(format!("bkn_socios_aisl_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let svc = SocioService::new(Arc::new(SqliteSocios::new(db.clone())));

        let (pijao, tebaida) = (Uuid::new_v4().to_string(), Uuid::new_v4().to_string());
        db.open_banco_db(&pijao).unwrap();
        db.open_banco_db(&tebaida).unwrap();

        svc.registrar(&pijao, datos("123", "Ana")).unwrap();
        // La misma cédula en otro Bankomunal es válida: son archivos independientes.
        assert!(svc.registrar(&tebaida, datos("123", "Ana")).is_ok());

        assert_eq!(svc.listar(&pijao).unwrap().len(), 1);
        assert_eq!(svc.listar(&tebaida).unwrap().len(), 1);
    }
}
