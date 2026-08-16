use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::error::AppError;
use crate::modules::auditoria::domain::{tipo_accion, AuditoriaPort, EntradaAuditoria};

use super::domain::{
    Bien, BienPort, CodigoOperacion, FiltroLibro, FondoGastosPort, LibroPort, Movimiento,
    NuevaOperacion, NuevoBien,
};

/// Capa de Aplicación/Servicios del módulo de Caja y Contabilidad.
/// Orquesta los casos de uso CU-16, CU-17 y CU-18 (RF-83 a RF-90).
pub struct CajaService {
    libro: Arc<dyn LibroPort>,
    fondo: Arc<dyn FondoGastosPort>,
    bienes: Arc<dyn BienPort>,
    auditoria: Arc<dyn AuditoriaPort>,
}

impl CajaService {
    pub fn new(
        libro: Arc<dyn LibroPort>,
        fondo: Arc<dyn FondoGastosPort>,
        bienes: Arc<dyn BienPort>,
        auditoria: Arc<dyn AuditoriaPort>,
    ) -> Self {
        Self { libro, fondo, bienes, auditoria }
    }

    /// RF-83 a RF-86 (CU-16, CU-17): registra una operación en el Libro.
    ///
    /// El código de operación decide de qué lado del libro cae el monto y si mueve el
    /// Fondo para Gastos; el usuario sólo indica cuál operación es, la fecha y el monto.
    pub fn registrar_operacion(
        &self,
        banco_id: &str,
        nueva: NuevaOperacion,
    ) -> Result<Movimiento, AppError> {
        if !nueva.codigo.registrable_en_caja() {
            return Err(AppError::OperacionNoPermitida(format!(
                "La operación {} la genera otro módulo y no se registra a mano desde Caja",
                nueva.codigo.as_str()
            )));
        }
        if nueva.monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto de la operación debe ser mayor a cero".into(),
            ));
        }
        if nueva.fecha.trim().is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar la fecha de la operación".into(),
            ));
        }

        // RF-86: un gasto no puede dejar el Fondo para Gastos en negativo — el
        // Bankomunal no puede gastar de un fondo lo que ese fondo no tiene.
        if nueva.codigo == CodigoOperacion::GastoBankomunal {
            let disponible = self.fondo.saldo(banco_id)?;
            if nueva.monto > disponible {
                return Err(AppError::OperacionNoPermitida(format!(
                    "El Fondo para Gastos tiene {disponible:.2} disponible y el gasto es de \
                     {:.2}. No se puede gastar más de lo acumulado en el fondo (RF-86)",
                    nueva.monto
                )));
            }
        }

        let es_ingreso = nueva.codigo.es_ingreso();
        let mov = Movimiento {
            id: Uuid::new_v4().to_string(),
            numero: self.libro.siguiente_numero(banco_id)?,
            fecha: nueva.fecha,
            codigo: nueva.codigo,
            descripcion: descripcion_o_por_defecto(nueva.codigo, &nueva.descripcion),
            ingreso: if es_ingreso { nueva.monto } else { 0.0 },
            egreso: if es_ingreso { 0.0 } else { nueva.monto },
            saldo: 0.0, // lo fija `recalcular_saldos`
            socio_id: None,
            credito_id: None,
            cierre_mes_id: None,
            corregido: false,
            corregido_por: None,
            fecha_correccion: None,
            motivo_correccion: None,
        };

        self.libro.registrar(banco_id, &mov)?;
        self.aplicar_al_fondo(banco_id, nueva.codigo, nueva.monto)?;
        self.libro.recalcular_saldos(banco_id)?;

        // Se relee para devolver el saldo ya recalculado.
        self.libro
            .buscar_por_id(banco_id, &mov.id)?
            .ok_or(AppError::MovimientoNoEncontrado)
    }

    /// Asienta en el Libro una operación generada por otro módulo (VC, CON, PC, MO…).
    ///
    /// Los módulos de Acciones y Créditos no escriben en `movimiento_libro` por su
    /// cuenta: el consecutivo, el saldo acumulado y su recálculo son responsabilidad de
    /// Caja, y duplicar esa lógica en cada módulo sería la vía más corta a un libro
    /// descuadrado. Por eso `registrar_operacion` rechaza estos códigos y existe esta
    /// puerta aparte, que no se expone como comando Tauri.
    #[allow(clippy::too_many_arguments)]
    pub fn registrar_asiento_de_modulo(
        &self,
        banco_id: &str,
        codigo: CodigoOperacion,
        fecha: String,
        monto: f64,
        descripcion: String,
        socio_id: Option<String>,
        credito_id: Option<String>,
    ) -> Result<Movimiento, AppError> {
        if codigo.registrable_en_caja() {
            return Err(AppError::OperacionNoPermitida(format!(
                "La operación {} se registra desde la pantalla de Caja, no desde otro módulo",
                codigo.as_str()
            )));
        }
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto del asiento debe ser mayor a cero".into(),
            ));
        }

        let es_ingreso = codigo.es_ingreso();
        let mov = Movimiento {
            id: Uuid::new_v4().to_string(),
            numero: self.libro.siguiente_numero(banco_id)?,
            fecha,
            codigo,
            descripcion: descripcion_o_por_defecto(codigo, &descripcion),
            ingreso: if es_ingreso { monto } else { 0.0 },
            egreso: if es_ingreso { 0.0 } else { monto },
            saldo: 0.0,
            socio_id,
            credito_id,
            cierre_mes_id: None,
            corregido: false,
            corregido_por: None,
            fecha_correccion: None,
            motivo_correccion: None,
        };

        self.libro.registrar(banco_id, &mov)?;
        self.libro.recalcular_saldos(banco_id)?;
        self.libro
            .buscar_por_id(banco_id, &mov.id)?
            .ok_or(AppError::MovimientoNoEncontrado)
    }

    /// RF-87 (CU-16, flujo alternativo): una donación no es "Otro Ingreso" sino un
    /// ingreso al Fondo para Gastos. Se expone aparte para que la pantalla no tenga que
    /// conocer esa equivalencia.
    pub fn registrar_donacion(
        &self,
        banco_id: &str,
        fecha: String,
        monto: f64,
        descripcion: String,
    ) -> Result<Movimiento, AppError> {
        let detalle = if descripcion.trim().is_empty() {
            "Donación".to_string()
        } else {
            format!("Donación — {}", descripcion.trim())
        };
        self.registrar_operacion(
            banco_id,
            NuevaOperacion {
                codigo: CodigoOperacion::IngresoFondoGastos,
                fecha,
                monto,
                descripcion: detalle,
            },
        )
    }

    /// RF-89/RF-90 (CU-18): corrige una operación ya registrada.
    ///
    /// Antes del Cierre de Mes basta con corregirla. Después, el sistema exige el nombre
    /// de quien corrige y el motivo, y deja registro en Auditoría (RNF-08).
    pub fn corregir_operacion(
        &self,
        banco_id: &str,
        id: &str,
        fecha: String,
        monto: f64,
        descripcion: String,
        nombre_quien_realiza: Option<String>,
        motivo: Option<String>,
    ) -> Result<Movimiento, AppError> {
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto corregido debe ser mayor a cero".into(),
            ));
        }

        let anterior = self
            .libro
            .buscar_por_id(banco_id, id)?
            .ok_or(AppError::MovimientoNoEncontrado)?;

        // RF-90: sobre un mes ya cerrado, la corrección exige trazabilidad.
        let cerrado = anterior.mes_cerrado();
        let (quien, porque) = match (cerrado, &nombre_quien_realiza, &motivo) {
            (false, _, _) => (None, None),
            (true, Some(q), Some(m)) if !q.trim().is_empty() && !m.trim().is_empty() => {
                (Some(q.trim().to_string()), Some(m.trim().to_string()))
            }
            (true, _, _) => {
                return Err(AppError::OperacionNoPermitida(
                    "El mes de esta operación ya está cerrado: para corregirla debe indicar \
                     su nombre y el motivo del cambio (RF-90)"
                        .into(),
                ))
            }
        };

        let monto_anterior = anterior.ingreso + anterior.egreso;
        let es_ingreso = anterior.codigo.es_ingreso();

        // Si un gasto crece, hay que comprobar que el fondo aguante la diferencia.
        if anterior.codigo == CodigoOperacion::GastoBankomunal && monto > monto_anterior {
            let disponible = self.fondo.saldo(banco_id)?;
            let extra = monto - monto_anterior;
            if extra > disponible {
                return Err(AppError::OperacionNoPermitida(format!(
                    "Corregir el gasto a {monto:.2} exige {extra:.2} más del Fondo para \
                     Gastos, que sólo tiene {disponible:.2} disponible"
                )));
            }
        }

        let corregido = Movimiento {
            fecha,
            descripcion: descripcion_o_por_defecto(anterior.codigo, &descripcion),
            ingreso: if es_ingreso { monto } else { 0.0 },
            egreso: if es_ingreso { 0.0 } else { monto },
            corregido: cerrado || anterior.corregido,
            corregido_por: quien.clone().or(anterior.corregido_por.clone()),
            fecha_correccion: if cerrado {
                Some(ahora())
            } else {
                anterior.fecha_correccion.clone()
            },
            motivo_correccion: porque.clone().or(anterior.motivo_correccion.clone()),
            ..anterior.clone()
        };

        self.libro.actualizar(banco_id, &corregido)?;

        // El Fondo para Gastos se ajusta por la diferencia, no por el monto completo.
        let delta = monto - monto_anterior;
        self.aplicar_al_fondo(banco_id, anterior.codigo, delta)?;
        self.libro.recalcular_saldos(banco_id)?;

        if cerrado {
            self.auditoria.registrar(
                banco_id,
                &EntradaAuditoria {
                    id: Uuid::new_v4().to_string(),
                    fecha: ahora(),
                    nombre_quien_realiza: quien.unwrap_or_default(),
                    entidad_afectada: "movimiento_libro".into(),
                    campo_modificado: Some(format!("operación #{}", anterior.numero)),
                    valor_anterior: Some(format!("{monto_anterior:.2}")),
                    valor_nuevo: Some(format!("{monto:.2}")),
                    motivo: porque,
                    tipo_accion: tipo_accion::CORRECCION_OPERACION.into(),
                },
            )?;
        }

        self.libro
            .buscar_por_id(banco_id, id)?
            .ok_or(AppError::MovimientoNoEncontrado)
    }

    /// Libro de Ingresos y Egresos, opcionalmente filtrado por rango de fechas.
    pub fn listar_libro(
        &self,
        banco_id: &str,
        filtro: FiltroLibro,
    ) -> Result<Vec<Movimiento>, AppError> {
        self.libro.listar(banco_id, &filtro)
    }

    /// Saldo disponible en caja: el del último asiento del Libro.
    pub fn saldo_caja(&self, banco_id: &str) -> Result<f64, AppError> {
        Ok(self
            .libro
            .listar(banco_id, &FiltroLibro::default())?
            .last()
            .map(|m| m.saldo)
            .unwrap_or(0.0))
    }

    pub fn saldo_fondo_gastos(&self, banco_id: &str) -> Result<f64, AppError> {
        self.fondo.saldo(banco_id)
    }

    /// RF-88 (CU-17, flujo alternativo): registra un Bien Adquirido como Activo Fijo.
    ///
    /// No toca el Libro ni el saldo de caja a propósito: el bien es patrimonio, no un
    /// movimiento de efectivo. Si se pagó con plata del Fondo para Gastos, ese
    /// desembolso se registra aparte como Gasto del Bankomunal (GBK).
    pub fn registrar_bien(&self, banco_id: &str, nuevo: NuevoBien) -> Result<Bien, AppError> {
        if nuevo.descripcion.trim().is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe describir el bien adquirido".into(),
            ));
        }
        if nuevo.valor < 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El valor del bien no puede ser negativo".into(),
            ));
        }
        let bien = Bien {
            id: Uuid::new_v4().to_string(),
            descripcion: nuevo.descripcion.trim().to_string(),
            fecha_adquisicion: nuevo.fecha_adquisicion,
            valor: nuevo.valor,
            tipo: nuevo.tipo,
        };
        self.bienes.registrar(banco_id, &bien)?;
        Ok(bien)
    }

    pub fn listar_bienes(&self, banco_id: &str) -> Result<Vec<Bien>, AppError> {
        self.bienes.listar(banco_id)
    }

    pub fn valor_activo_fijo(&self, banco_id: &str) -> Result<f64, AppError> {
        self.bienes.valor_total(banco_id)
    }

    fn aplicar_al_fondo(
        &self,
        banco_id: &str,
        codigo: CodigoOperacion,
        monto: f64,
    ) -> Result<(), AppError> {
        let signo = codigo.efecto_en_fondo_gastos();
        if signo != 0.0 {
            self.fondo.ajustar(banco_id, signo * monto)?;
        }
        Ok(())
    }
}

fn ahora() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

/// Si el usuario no escribió descripción, se guarda el nombre de la operación en vez de
/// dejar la celda vacía: el Libro se imprime y se lee en papel.
fn descripcion_o_por_defecto(codigo: CodigoOperacion, descripcion: &str) -> String {
    let d = descripcion.trim();
    if !d.is_empty() {
        return d.to_string();
    }
    match codigo {
        CodigoOperacion::OtroIngreso => "Otro ingreso",
        CodigoOperacion::OtroEgreso => "Otro egreso",
        CodigoOperacion::IngresoFondoGastos => "Ingreso al Fondo para Gastos",
        CodigoOperacion::GastoBankomunal => "Gasto del Bankomunal",
        otro => otro.as_str(),
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
    use crate::modules::caja::domain::TipoBien;

    fn test_service() -> (CajaService, String) {
        let (svc, banco, _) = test_service_con_db();
        (svc, banco)
    }

    fn test_service_con_db() -> (CajaService, String, DbManager) {
        let dir = std::env::temp_dir().join(format!("bkn_caja_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let banco_id = Uuid::new_v4().to_string();
        db.open_banco_db(&banco_id).unwrap();
        let svc = CajaService::new(
            Arc::new(SqliteLibro::new(db.clone())),
            Arc::new(SqliteFondoGastos::new(db.clone())),
            Arc::new(SqliteBienes::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        );
        (svc, banco_id, db)
    }

    /// Simula que el mes del asiento ya fue cerrado. El módulo de Cierre todavía no
    /// existe, pero la ruta de RF-90 sí debe quedar cubierta desde ya.
    fn marcar_mes_cerrado(db: &DbManager, banco_id: &str, mov_id: &str) {
        let conn = db.open_banco_db(banco_id).unwrap();
        conn.execute(
            "UPDATE movimiento_libro SET cierre_mes_id = 'cierre-de-prueba' WHERE id = ?1",
            [mov_id],
        )
        .unwrap();
    }

    fn op(codigo: CodigoOperacion, fecha: &str, monto: f64) -> NuevaOperacion {
        NuevaOperacion {
            codigo,
            fecha: fecha.into(),
            monto,
            descripcion: String::new(),
        }
    }

    /// RF-83/RF-84: los ingresos suman al saldo de caja y los egresos restan.
    #[test]
    fn ingresos_y_egresos_mueven_el_saldo_en_el_sentido_correcto() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 50_000.0))
            .unwrap();
        let egreso = svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroEgreso, "2026-08-02", 20_000.0))
            .unwrap();

        assert_eq!(egreso.ingreso, 0.0);
        assert_eq!(egreso.egreso, 20_000.0);
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 30_000.0);
    }

    /// El consecutivo del Libro avanza de uno en uno, como en el libro de papel.
    #[test]
    fn el_numero_de_operacion_es_consecutivo() {
        let (svc, banco) = test_service();
        for i in 1..=3 {
            let m = svc
                .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 1000.0))
                .unwrap();
            assert_eq!(m.numero, i);
        }
    }

    /// El saldo es acumulado y se ordena por fecha: un asiento con fecha anterior
    /// recoloca los saldos de los que ya estaban.
    #[test]
    fn un_asiento_con_fecha_anterior_recalcula_los_saldos() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-10", 10_000.0))
            .unwrap();
        svc.registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 5_000.0))
            .unwrap();

        let libro = svc.listar_libro(&banco, FiltroLibro::default()).unwrap();
        // Se lee en orden cronológico, no de captura.
        assert_eq!(libro[0].fecha, "2026-08-01");
        assert_eq!(libro[0].saldo, 5_000.0);
        assert_eq!(libro[1].saldo, 15_000.0);
    }

    /// RF-85: el ingreso al Fondo para Gastos sube su saldo acumulado.
    #[test]
    fn ingreso_al_fondo_sube_su_saldo() {
        let (svc, banco) = test_service();
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 0.0);
        svc.registrar_operacion(
            &banco,
            op(CodigoOperacion::IngresoFondoGastos, "2026-08-01", 30_000.0),
        )
        .unwrap();
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 30_000.0);
    }

    /// RF-86: el gasto descuenta del Fondo para Gastos.
    #[test]
    fn gasto_del_bankomunal_descuenta_del_fondo() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(
            &banco,
            op(CodigoOperacion::IngresoFondoGastos, "2026-08-01", 30_000.0),
        )
        .unwrap();
        svc.registrar_operacion(&banco, op(CodigoOperacion::GastoBankomunal, "2026-08-05", 12_000.0))
            .unwrap();

        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 18_000.0);
        // Y el gasto también sale de la caja.
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 18_000.0);
    }

    /// No se puede gastar del Fondo para Gastos más de lo que tiene acumulado.
    #[test]
    fn gasto_mayor_al_fondo_se_rechaza() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(
            &banco,
            op(CodigoOperacion::IngresoFondoGastos, "2026-08-01", 10_000.0),
        )
        .unwrap();

        let err = svc
            .registrar_operacion(&banco, op(CodigoOperacion::GastoBankomunal, "2026-08-05", 15_000.0))
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
        // Ni el fondo ni la caja quedaron tocados.
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 10_000.0);
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 10_000.0);
    }

    /// RF-87: la donación entra al Fondo para Gastos, no como Otro Ingreso.
    #[test]
    fn la_donacion_entra_al_fondo_de_gastos() {
        let (svc, banco) = test_service();
        let mov = svc
            .registrar_donacion(&banco, "2026-08-03".into(), 25_000.0, "Alcaldía".into())
            .unwrap();

        assert_eq!(mov.codigo, CodigoOperacion::IngresoFondoGastos);
        assert!(mov.descripcion.contains("Donación"));
        assert!(mov.descripcion.contains("Alcaldía"));
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 25_000.0);
    }

    #[test]
    fn registrar_rechaza_montos_no_positivos_y_fecha_vacia() {
        let (svc, banco) = test_service();
        assert!(svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 0.0))
            .is_err());
        assert!(svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", -100.0))
            .is_err());
        assert!(svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "  ", 100.0))
            .is_err());
    }

    /// Las operaciones que genera otro módulo no se teclean desde Caja.
    #[test]
    fn caja_no_permite_registrar_operaciones_de_otros_modulos() {
        let (svc, banco) = test_service();
        for codigo in [
            CodigoOperacion::VentaAcciones,
            CodigoOperacion::DesembolsoCredito,
            CodigoOperacion::PagoCuota,
        ] {
            assert!(svc
                .registrar_operacion(&banco, op(codigo, "2026-08-01", 1000.0))
                .is_err());
        }
    }

    /// RF-89: con el mes abierto la corrección no exige nombre ni motivo.
    #[test]
    fn corregir_antes_del_cierre_no_exige_nombre_ni_motivo() {
        let (svc, banco) = test_service();
        let mov = svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 10_000.0))
            .unwrap();

        let corregido = svc
            .corregir_operacion(
                &banco, &mov.id, "2026-08-01".into(), 12_000.0, "Ajuste".into(), None, None,
            )
            .unwrap();

        assert_eq!(corregido.ingreso, 12_000.0);
        assert!(!corregido.corregido, "no es una corrección tras cierre");
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 12_000.0);
        // Y no se ensucia la Auditoría con correcciones ordinarias.
        assert!(svc.auditoria.listar(&banco).unwrap().is_empty());
    }

    /// Corregir un gasto ajusta el Fondo por la diferencia, no por el monto completo.
    #[test]
    fn corregir_un_gasto_ajusta_el_fondo_por_la_diferencia() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(
            &banco,
            op(CodigoOperacion::IngresoFondoGastos, "2026-08-01", 50_000.0),
        )
        .unwrap();
        let gasto = svc
            .registrar_operacion(&banco, op(CodigoOperacion::GastoBankomunal, "2026-08-05", 10_000.0))
            .unwrap();
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 40_000.0);

        svc.corregir_operacion(
            &banco, &gasto.id, "2026-08-05".into(), 15_000.0, "Faltó el IVA".into(), None, None,
        )
        .unwrap();

        // 50.000 - 15.000, no 40.000 - 15.000.
        assert_eq!(svc.saldo_fondo_gastos(&banco).unwrap(), 35_000.0);
    }

    /// RF-90: sobre un mes ya cerrado, corregir sin nombre ni motivo se rechaza.
    #[test]
    fn corregir_tras_el_cierre_exige_nombre_y_motivo() {
        let (svc, banco, db) = test_service_con_db();
        let mov = svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-07-10", 10_000.0))
            .unwrap();
        marcar_mes_cerrado(&db, &banco, &mov.id);

        // Sin datos de trazabilidad.
        assert!(svc
            .corregir_operacion(&banco, &mov.id, "2026-07-10".into(), 9_000.0, "x".into(), None, None)
            .is_err());
        // Con el nombre pero sin motivo.
        assert!(svc
            .corregir_operacion(
                &banco, &mov.id, "2026-07-10".into(), 9_000.0, "x".into(),
                Some("Sara".into()), None,
            )
            .is_err());
        // Con ambos pero en blanco.
        assert!(svc
            .corregir_operacion(
                &banco, &mov.id, "2026-07-10".into(), 9_000.0, "x".into(),
                Some("  ".into()), Some("  ".into()),
            )
            .is_err());

        // El monto no se tocó en ninguno de los intentos fallidos.
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 10_000.0);
    }

    /// RF-90 + RNF-08: la corrección tras el cierre queda registrada en Auditoría con
    /// quién, cuándo, el valor anterior, el nuevo y el motivo.
    #[test]
    fn corregir_tras_el_cierre_deja_registro_en_auditoria() {
        let (svc, banco, db) = test_service_con_db();
        let mov = svc
            .registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-07-10", 10_000.0))
            .unwrap();
        marcar_mes_cerrado(&db, &banco, &mov.id);

        let corregido = svc
            .corregir_operacion(
                &banco,
                &mov.id,
                "2026-07-10".into(),
                8_500.0,
                "Se había digitado de más".into(),
                Some("Sara Sánchez".into()),
                Some("Error de digitación en la reunión".into()),
            )
            .unwrap();

        assert_eq!(corregido.ingreso, 8_500.0);
        assert!(corregido.corregido, "queda marcado como corregido tras cierre");
        assert_eq!(corregido.corregido_por.as_deref(), Some("Sara Sánchez"));
        assert!(corregido.fecha_correccion.is_some());
        assert_eq!(svc.saldo_caja(&banco).unwrap(), 8_500.0);

        let bitacora = svc.auditoria.listar(&banco).unwrap();
        assert_eq!(bitacora.len(), 1);
        let e = &bitacora[0];
        assert_eq!(e.nombre_quien_realiza, "Sara Sánchez");
        assert_eq!(e.entidad_afectada, "movimiento_libro");
        assert_eq!(e.tipo_accion, tipo_accion::CORRECCION_OPERACION);
        assert_eq!(e.valor_anterior.as_deref(), Some("10000.00"));
        assert_eq!(e.valor_nuevo.as_deref(), Some("8500.00"));
        assert!(e.motivo.as_deref().unwrap().contains("digitación"));
    }

    #[test]
    fn corregir_operacion_inexistente_devuelve_error() {
        let (svc, banco) = test_service();
        assert!(matches!(
            svc.corregir_operacion(&banco, "no-existe", "2026-08-01".into(), 1.0, "x".into(), None, None)
                .unwrap_err(),
            AppError::MovimientoNoEncontrado
        ));
    }

    /// RF-104: el Libro se puede acotar a un rango de fechas.
    #[test]
    fn el_libro_se_filtra_por_rango_de_fechas() {
        let (svc, banco) = test_service();
        for fecha in ["2026-07-15", "2026-08-05", "2026-09-02"] {
            svc.registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, fecha, 1000.0))
                .unwrap();
        }

        let agosto = svc
            .listar_libro(
                &banco,
                FiltroLibro { desde: Some("2026-08-01".into()), hasta: Some("2026-08-31".into()) },
            )
            .unwrap();
        assert_eq!(agosto.len(), 1);
        assert_eq!(agosto[0].fecha, "2026-08-05");

        assert_eq!(svc.listar_libro(&banco, FiltroLibro::default()).unwrap().len(), 3);
    }

    /// RF-88: el bien es patrimonio, no efectivo — no toca el Libro ni el saldo de caja.
    #[test]
    fn el_bien_adquirido_no_afecta_la_caja() {
        let (svc, banco) = test_service();
        svc.registrar_operacion(&banco, op(CodigoOperacion::OtroIngreso, "2026-08-01", 10_000.0))
            .unwrap();

        svc.registrar_bien(
            &banco,
            NuevoBien {
                descripcion: "Calculadora".into(),
                fecha_adquisicion: "2026-08-04".into(),
                valor: 80_000.0,
                tipo: TipoBien::Propio,
            },
        )
        .unwrap();

        assert_eq!(svc.saldo_caja(&banco).unwrap(), 10_000.0, "la caja no cambia");
        assert_eq!(svc.listar_libro(&banco, FiltroLibro::default()).unwrap().len(), 1);
        assert_eq!(svc.valor_activo_fijo(&banco).unwrap(), 80_000.0);
    }

    #[test]
    fn los_bienes_en_comodato_tambien_se_registran() {
        let (svc, banco) = test_service();
        svc.registrar_bien(
            &banco,
            NuevoBien {
                descripcion: "Mesa prestada".into(),
                fecha_adquisicion: "2026-08-04".into(),
                valor: 0.0,
                tipo: TipoBien::Comodato,
            },
        )
        .unwrap();
        let bienes = svc.listar_bienes(&banco).unwrap();
        assert_eq!(bienes.len(), 1);
        assert_eq!(bienes[0].tipo, TipoBien::Comodato);
    }

    #[test]
    fn registrar_bien_exige_descripcion_y_valor_no_negativo() {
        let (svc, banco) = test_service();
        let base = |desc: &str, valor: f64| NuevoBien {
            descripcion: desc.into(),
            fecha_adquisicion: "2026-08-04".into(),
            valor,
            tipo: TipoBien::Propio,
        };
        assert!(svc.registrar_bien(&banco, base("  ", 100.0)).is_err());
        assert!(svc.registrar_bien(&banco, base("Silla", -1.0)).is_err());
    }

    /// RF-08: el Libro de cada Bankomunal es independiente.
    #[test]
    fn el_libro_no_se_mezcla_entre_bankomunales() {
        let dir = std::env::temp_dir().join(format!("bkn_caja_aisl_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let svc = CajaService::new(
            Arc::new(SqliteLibro::new(db.clone())),
            Arc::new(SqliteFondoGastos::new(db.clone())),
            Arc::new(SqliteBienes::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        );
        let (pijao, tebaida) = (Uuid::new_v4().to_string(), Uuid::new_v4().to_string());
        db.open_banco_db(&pijao).unwrap();
        db.open_banco_db(&tebaida).unwrap();

        svc.registrar_operacion(&pijao, op(CodigoOperacion::OtroIngreso, "2026-08-01", 90_000.0))
            .unwrap();

        assert_eq!(svc.saldo_caja(&pijao).unwrap(), 90_000.0);
        assert_eq!(svc.saldo_caja(&tebaida).unwrap(), 0.0);
        // El consecutivo también es propio de cada Banco.
        let m = svc
            .registrar_operacion(&tebaida, op(CodigoOperacion::OtroIngreso, "2026-08-01", 1000.0))
            .unwrap();
        assert_eq!(m.numero, 1);
    }
}
