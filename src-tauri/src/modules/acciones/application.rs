use std::sync::Arc;

use uuid::Uuid;

use crate::core::error::AppError;

use super::domain::{
    calcular_ppcfc, mes_de, meses_entre, tramo_de_venta, AutorizacionVenta, CalculoCompra,
    CierresPort, CupoMensual, LibroContablePort, LoteAcciones, LoteAccionesPort, NuevaCompra,
    ParametrosAccionesPort, ResumenMesAcciones, MESES_GRACIA_TOPE_PARTICIPACION, MESES_PPCFC,
    TOPE_PARTICIPACION_PCT,
};

/// Capa de Aplicación/Servicios del módulo de Acciones.
/// Orquesta el caso de uso CU-07 — Comprar Acciones (RF-22 a RF-27).
pub struct AccionesService {
    lotes: Arc<dyn LoteAccionesPort>,
    parametros: Arc<dyn ParametrosAccionesPort>,
    libro: Arc<dyn LibroContablePort>,
    cierres: Arc<dyn CierresPort>,
}

impl AccionesService {
    pub fn new(
        lotes: Arc<dyn LoteAccionesPort>,
        parametros: Arc<dyn ParametrosAccionesPort>,
        libro: Arc<dyn LibroContablePort>,
        cierres: Arc<dyn CierresPort>,
    ) -> Self {
        Self { lotes, parametros, libro, cierres }
    }

    /// RF-26 (RN-09 + RN-15): cupo de venta de acciones del mes.
    ///
    /// Mientras no haya 3 meses cerrados el PPCFC no es calculable y el cupo queda en
    /// `SinDatosSuficientes`, **sin bloquear la venta**: qué hacer en los primeros
    /// meses es una decisión abierta del cliente (D-02), y bloquear sería elegir por él.
    pub fn cupo_del_mes(&self, banco_id: &str, fecha: &str) -> Result<CupoMensual, AppError> {
        let mes = mes_de(fecha).ok_or_else(|| {
            AppError::OperacionNoPermitida(format!("La fecha «{fecha}» no es válida"))
        })?;
        let params = self.parametros.obtener(banco_id)?;
        let (vendido_acciones, vendido_monto) = self.lotes.vendido_en_mes(banco_id, &mes)?;
        let colocaciones = self.cierres.colocaciones_recientes(banco_id, MESES_PPCFC)?;

        let autorizacion = match calcular_ppcfc(&colocaciones) {
            None => AutorizacionVenta::SinDatosSuficientes {
                meses_cerrados: colocaciones.len(),
            },
            Some(ppcfc_pct) => match tramo_de_venta(ppcfc_pct, &params) {
                None => AutorizacionVenta::NoAutoriza { ppcfc_pct },
                Some((desde, hasta, venta_pct)) => {
                    // El cupo es un % del total de acciones del Bankomunal.
                    let total = self.lotes.total_acciones(banco_id)?;
                    let cupo_acciones = (total as f64 * venta_pct / 100.0).floor() as i64;
                    AutorizacionVenta::Autoriza {
                        ppcfc_pct,
                        rango_desde: desde,
                        rango_hasta: hasta,
                        venta_pct,
                        cupo_acciones,
                        cupo_monto: cupo_acciones as f64 * params.valor_nominal,
                    }
                }
            },
        };

        let (disponible_monto, tope_individual_monto) = match &autorizacion {
            AutorizacionVenta::Autoriza { cupo_monto, .. } => (
                Some((cupo_monto - vendido_monto).max(0.0)),
                // RN-15: el tope individual es un % del cupo del mes, no del capital.
                Some(cupo_monto * params.tope_individual_mensual_pct / 100.0),
            ),
            _ => (None, None),
        };

        Ok(CupoMensual {
            mes,
            autorizacion,
            vendido_acciones,
            vendido_monto,
            disponible_monto,
            tope_individual_monto,
        })
    }

    /// RF-23/RF-24/RF-25: calcula una compra sin registrarla.
    ///
    /// Existe para que la pantalla muestre la cantidad de acciones y el nuevo % de
    /// participación *antes* de recibir el dinero, que es el orden en que ocurre en la
    /// reunión: primero el Verificador revisa, después el Cajero cobra.
    pub fn previsualizar_compra(
        &self,
        banco_id: &str,
        socio_id: &str,
        monto: f64,
    ) -> Result<CalculoCompra, AppError> {
        self.calcular(banco_id, socio_id, monto, &hoy())
    }

    /// RF-22 (CU-07): registra la compra de acciones de un socio.
    ///
    /// Deja el lote guardado con su mes de compra (RF-27) y el asiento VC en el Libro
    /// de Ingresos y Egresos.
    pub fn registrar_compra(
        &self,
        banco_id: &str,
        nueva: NuevaCompra,
    ) -> Result<LoteAcciones, AppError> {
        let fecha = nueva.fecha.trim().to_string();
        if fecha.is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar la fecha de la compra".into(),
            ));
        }
        let mes_compra = mes_de(&fecha).ok_or_else(|| {
            AppError::OperacionNoPermitida(format!("La fecha «{fecha}» no es válida"))
        })?;

        let calculo = self.calcular(banco_id, &nueva.socio_id, nueva.monto, &fecha)?;

        // RN-02: el tope del 15% bloquea la compra, salvo en los primeros meses.
        if calculo.supera_tope_participacion {
            return Err(AppError::OperacionNoPermitida(format!(
                "Con esta compra el socio quedaría con el {:.2}% de las acciones del \
                 Bankomunal y ningún socio puede superar el {TOPE_PARTICIPACION_PCT}% (RN-02)",
                calculo.participacion_pct
            )));
        }

        let lote = LoteAcciones {
            id: Uuid::new_v4().to_string(),
            socio_id: nueva.socio_id.clone(),
            mes_compra,
            fecha_compra: fecha.clone(),
            cantidad: calculo.cantidad,
            valor_nominal_compra: calculo.valor_nominal,
            monto_pagado: calculo.monto,
            liquidada: false,
            fecha_liquidacion: None,
        };

        self.lotes.crear(banco_id, &lote)?;

        // RF-22: la compra entra a la caja como Venta de Certificados (VC).
        self.libro.registrar_venta_acciones(
            banco_id,
            &fecha,
            calculo.monto,
            &nueva.socio_id,
            &format!("Venta de {} acciones", calculo.cantidad),
        )?;

        Ok(lote)
    }

    /// Acciones vigentes de un socio. Reemplaza al método pendiente de `Socio`: la
    /// pregunta se responde con los lotes, que son de este módulo.
    pub fn acciones_de_socio(&self, banco_id: &str, socio_id: &str) -> Result<i64, AppError> {
        self.lotes.acciones_de_socio(banco_id, socio_id)
    }

    /// Total de acciones vigentes del Bankomunal.
    pub fn total_acciones(&self, banco_id: &str) -> Result<i64, AppError> {
        self.lotes.total_acciones(banco_id)
    }

    /// Acciones vigentes por socio, para la columna del listado de Socios.
    pub fn acciones_por_socio(&self, banco_id: &str) -> Result<Vec<(String, i64)>, AppError> {
        self.lotes.acciones_por_socio(banco_id)
    }

    /// RF-105: Control de Acciones del socio, mes a mes, con su saldo acumulado.
    pub fn historial_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
    ) -> Result<Vec<ResumenMesAcciones>, AppError> {
        let lotes = self.lotes.listar_de_socio(banco_id, socio_id)?;

        // Se agrupa por mes conservando el orden cronológico que trae el adaptador.
        let mut meses: Vec<ResumenMesAcciones> = Vec::new();
        for lote in &lotes {
            let fila = match meses.iter_mut().find(|m| m.mes == lote.mes_compra) {
                Some(f) => f,
                None => {
                    meses.push(ResumenMesAcciones {
                        mes: lote.mes_compra.clone(),
                        compradas: 0,
                        liquidadas: 0,
                        saldo: 0,
                    });
                    meses.last_mut().expect("recién insertado")
                }
            };
            fila.compradas += lote.cantidad;
            if lote.liquidada {
                fila.liquidadas += lote.cantidad;
            }
        }

        // El saldo es acumulado: lo que el socio tiene al cierre de cada mes.
        let mut acumulado = 0;
        for fila in meses.iter_mut() {
            acumulado += fila.compradas - fila.liquidadas;
            fila.saldo = acumulado;
        }
        Ok(meses)
    }

    /// Cálculo compartido por la previsualización y el registro, para que la pantalla
    /// nunca muestre un número distinto del que después se guarda (DRY).
    fn calcular(
        &self,
        banco_id: &str,
        socio_id: &str,
        monto: f64,
        fecha: &str,
    ) -> Result<CalculoCompra, AppError> {
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto de la compra debe ser mayor a cero".into(),
            ));
        }

        let params = self.parametros.obtener(banco_id)?;
        if params.valor_nominal <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El valor nominal de la acción debe configurarse antes de vender acciones \
                 (RN-13)"
                    .into(),
            ));
        }

        // RF-23: la cantidad sale del monto y el valor nominal. Las acciones son
        // unidades enteras, así que el monto debe ser múltiplo exacto del nominal: si
        // se aceptara un monto intermedio habría que quedarse con el sobrante o
        // regalar una fracción de acción, y ninguna de las dos cosas es correcta.
        let exactas = monto / params.valor_nominal;
        let cantidad = exactas.floor() as i64;
        if (exactas - exactas.floor()).abs() > 1e-9 || cantidad < 1 {
            let bajo = cantidad.max(0) as f64 * params.valor_nominal;
            let alto = (cantidad + 1) as f64 * params.valor_nominal;
            return Err(AppError::OperacionNoPermitida(format!(
                "Con un valor nominal de {:.2} el monto debe ser múltiplo exacto: \
                 {:.2} compra {} acciones y {:.2} compra {}",
                params.valor_nominal,
                bajo,
                cantidad.max(0),
                alto,
                cantidad + 1
            )));
        }

        let acciones_socio = self.lotes.acciones_de_socio(banco_id, socio_id)?;
        let total_actual = self.lotes.total_acciones(banco_id)?;

        let acciones_socio_despues = acciones_socio + cantidad;
        let total_despues = total_actual + cantidad;
        let participacion_pct = if total_despues > 0 {
            acciones_socio_despues as f64 * 100.0 / total_despues as f64
        } else {
            0.0
        };

        // RN-02 sólo aplica a partir del tercer mes de operaciones.
        let meses_operando = meses_entre(&params.fecha_creacion, fecha).unwrap_or(0);
        let en_gracia = meses_operando < MESES_GRACIA_TOPE_PARTICIPACION;

        Ok(CalculoCompra {
            cantidad,
            valor_nominal: params.valor_nominal,
            monto,
            acciones_socio_despues,
            total_bankomunal_despues: total_despues,
            participacion_pct,
            supera_tope_participacion: !en_gracia
                && participacion_pct > TOPE_PARTICIPACION_PCT + 1e-9,
            tope_en_periodo_de_gracia: en_gracia,
        })
    }
}

fn hoy() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::acciones::data::{SqliteLotesAcciones, SqliteParametrosAcciones};
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::caja::application::CajaService;
    use crate::modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
    use crate::modules::caja::domain::FiltroLibro;
    use crate::modules::configuracion::application::ConfigService;
    use crate::modules::configuracion::data::SqliteConfiguracion;
    use crate::modules::socios::application::SocioService;
    use crate::modules::socios::data::SqliteSocios;
    use crate::modules::socios::domain::DatosSocio;

    struct Contexto {
        acciones: AccionesService,
        caja: Arc<CajaService>,
        config: ConfigService,
        socios: SocioService,
        banco: String,
        db: DbManager,
    }

    impl Contexto {
        /// El período de gracia de RN-02 se mide desde la creación del Bankomunal,
        /// así que los tests la mueven en el tiempo para probar ambos lados.
        fn creado_el(&self, fecha: &str) {
            let conn = self.db.open_banco_db(&self.banco).unwrap();
            conn.execute(
                "UPDATE configuracion SET fecha_creacion = ?2 WHERE id = ?1",
                [self.banco.as_str(), fecha],
            )
            .unwrap();
        }
    }

    /// Monta el módulo con sus dependencias reales. El Bankomunal se crea "hace un año"
    /// para que RN-02 ya aplique; los tests del período de gracia lo ajustan aparte.
    fn contexto() -> Contexto {
        let dir = std::env::temp_dir().join(format!("bkn_acc_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let banco = Uuid::new_v4().to_string();
        db.open_banco_db(&banco).unwrap();

        let caja = Arc::new(CajaService::new(
            Arc::new(SqliteLibro::new(db.clone())),
            Arc::new(SqliteFondoGastos::new(db.clone())),
            Arc::new(SqliteBienes::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        ));
        let config = ConfigService::new(
            Arc::new(SqliteConfiguracion::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        );
        let acciones = AccionesService::new(
            Arc::new(SqliteLotesAcciones::new(db.clone())),
            Arc::new(SqliteParametrosAcciones::new(db.clone())),
            Arc::new(crate::modules::acciones::data::LibroViaCaja::new(caja.clone())),
            Arc::new(crate::modules::acciones::data::SqliteCierres::new(db.clone())),
        );
        let socios = SocioService::new(Arc::new(SqliteSocios::new(db.clone())));
        let ctx = Contexto { acciones, caja, config, socios, banco, db };

        // Por defecto el Bankomunal acaba de arrancar, que es el único estado en que se
        // puede formar el capital inicial: RN-02 haría imposible la primera compra —el
        // primer socio tiene por definición el 100%— si no fuera por su período de
        // gracia. Los tests que verifican el tope mueven la fecha después de sembrar.
        ctx.config.obtener_configuracion(&ctx.banco).unwrap();
        ctx.creado_el("2026-08-01T00:00:00");
        ctx
    }

    /// Registra el socio n (o devuelve el ya creado) y da su id.
    ///
    /// `lote_acciones.socio_id` es una clave foránea a `socio`: usar ids inventados
    /// dejaría pasar tests que en la aplicación real fallarían al insertar.
    fn socio(ctx: &Contexto, n: u8) -> String {
        let cedula = format!("100{n}");
        if let Some(s) = ctx.socios.buscar_por_cedula(&ctx.banco, &cedula).unwrap() {
            return s.id;
        }
        ctx.socios
            .registrar(
                &ctx.banco,
                DatosSocio {
                    cedula,
                    nombres: format!("Socio {n}"),
                    apellidos: "De Prueba".into(),
                    profesion: String::new(),
                    direccion: String::new(),
                    telefono: String::new(),
                    celular: String::new(),
                    correo: String::new(),
                    beneficiario: None,
                    protegidos: vec![],
                },
            )
            .unwrap()
            .id
    }

    fn compra(ctx: &Contexto, socio_id: &str, monto: f64) -> Result<LoteAcciones, AppError> {
        ctx.acciones.registrar_compra(
            &ctx.banco,
            NuevaCompra { socio_id: socio_id.into(), fecha: "2026-08-05".into(), monto },
        )
    }

    /// RF-23: con nominal de $10.000, $50.000 compran 5 acciones.
    #[test]
    fn la_cantidad_sale_del_monto_y_el_valor_nominal() {
        let ctx = contexto();
        let lote = compra(&ctx, &socio(&ctx, 1), 50_000.0).unwrap();
        assert_eq!(lote.cantidad, 5);
        assert_eq!(lote.valor_nominal_compra, 10_000.0);
        assert_eq!(lote.monto_pagado, 50_000.0);
    }

    /// RF-27: el lote guarda el mes de compra, insumo del aniversario (RN-10).
    #[test]
    fn el_lote_guarda_el_mes_de_compra() {
        let ctx = contexto();
        let lote = compra(&ctx, &socio(&ctx, 1), 20_000.0).unwrap();
        assert_eq!(lote.mes_compra, "2026-08-01");
        assert_eq!(lote.fecha_compra, "2026-08-05");
        assert!(!lote.liquidada);
    }

    /// El monto debe ser múltiplo del nominal: no existen fracciones de acción.
    #[test]
    fn un_monto_que_no_es_multiplo_del_nominal_se_rechaza() {
        let ctx = contexto();
        let err = compra(&ctx, &socio(&ctx, 1), 25_000.0).unwrap_err();
        let msg = err.to_string();
        // El mensaje orienta al cajero con los dos montos válidos más cercanos.
        assert!(msg.contains("20000.00"), "{msg}");
        assert!(msg.contains("30000.00"), "{msg}");
        assert_eq!(ctx.acciones.total_acciones(&ctx.banco).unwrap(), 0);
    }

    #[test]
    fn rechaza_montos_no_positivos_menores_a_una_accion_y_fecha_vacia() {
        let ctx = contexto();
        assert!(compra(&ctx, &socio(&ctx, 1), 0.0).is_err());
        assert!(compra(&ctx, &socio(&ctx, 1), -10_000.0).is_err());
        // Menos de una acción completa.
        assert!(compra(&ctx, &socio(&ctx, 1), 5_000.0).is_err());
        assert!(ctx
            .acciones
            .registrar_compra(
                &ctx.banco,
                NuevaCompra { socio_id: socio(&ctx, 1), fecha: "  ".into(), monto: 10_000.0 },
            )
            .is_err());
    }

    /// RF-24: el % de participación se calcula sobre el total tras la compra.
    #[test]
    fn calcula_el_porcentaje_de_participacion() {
        let ctx = contexto();
        compra(&ctx, &socio(&ctx, 1), 30_000.0).unwrap(); // 3 acciones
        compra(&ctx, &socio(&ctx, 2), 70_000.0).unwrap(); // 7 acciones

        assert_eq!(ctx.acciones.total_acciones(&ctx.banco).unwrap(), 10);
        assert_eq!(ctx.acciones.acciones_de_socio(&ctx.banco, &socio(&ctx, 1)).unwrap(), 3);

        // El socio 1 compra 2 más: 5 de 12.
        let previo = ctx
            .acciones
            .previsualizar_compra(&ctx.banco, &socio(&ctx, 1), 20_000.0)
            .unwrap();
        assert_eq!(previo.cantidad, 2);
        assert_eq!(previo.acciones_socio_despues, 5);
        assert_eq!(previo.total_bankomunal_despues, 12);
        assert!((previo.participacion_pct - 41.666).abs() < 0.01);
    }

    /// RF-25 + RN-02: la compra que deja al socio por encima del 15% se bloquea.
    #[test]
    fn bloquea_la_compra_que_supera_el_tope_del_15_por_ciento() {
        let ctx = contexto();
        // 9 socios con 10 acciones cada uno = 90 acciones, formadas durante el arranque.
        for n in 1..=9 {
            compra(&ctx, &socio(&ctx, n), 100_000.0).unwrap();
        }
        assert_eq!(ctx.acciones.total_acciones(&ctx.banco).unwrap(), 90);
        // Pasado el período de gracia, RN-02 ya rige.
        ctx.creado_el("2020-01-01T00:00:00");

        // El socio 1 quiere 10 más: quedaría con 20 de 100 = 20% > 15%.
        let err = compra(&ctx, &socio(&ctx, 1), 100_000.0).unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
        assert!(err.to_string().contains("RN-02"));

        // Nada se registró: ni el lote ni el asiento en el Libro.
        assert_eq!(ctx.acciones.total_acciones(&ctx.banco).unwrap(), 90);
        assert_eq!(ctx.caja.listar_libro(&ctx.banco, FiltroLibro::default()).unwrap().len(), 9);
    }

    /// Justo en el 15% todavía se permite: el tope es "más del 15%".
    #[test]
    fn el_tope_permite_exactamente_el_15_por_ciento() {
        let ctx = contexto();
        // 85 acciones repartidas entre otros socios.
        for n in 2..=10 {
            compra(&ctx, &socio(&ctx, n), 90_000.0).unwrap(); // 9 c/u = 81
        }
        compra(&ctx, &socio(&ctx, 11), 40_000.0).unwrap(); // +4 = 85
        ctx.creado_el("2020-01-01T00:00:00");

        // El socio 1 compra 15: 15 de 100 = 15% exacto.
        let lote = compra(&ctx, &socio(&ctx, 1), 150_000.0).unwrap();
        assert_eq!(lote.cantidad, 15);
    }

    /// El primer socio de un Bankomunal recién creado tiene el 100% de las acciones.
    /// Sin el período de gracia de RN-02 sería imposible arrancar.
    #[test]
    fn la_primera_compra_del_bankomunal_es_posible() {
        let ctx = contexto();
        let lote = compra(&ctx, &socio(&ctx, 1), 100_000.0).unwrap();
        assert_eq!(lote.cantidad, 10);

        let previo = ctx
            .acciones
            .previsualizar_compra(&ctx.banco, &socio(&ctx, 1), 10_000.0)
            .unwrap();
        assert_eq!(previo.participacion_pct, 100.0);
        assert!(previo.tope_en_periodo_de_gracia);
        assert!(!previo.supera_tope_participacion);
    }

    /// RN-02 no aplica en los primeros 3 meses de operación.
    #[test]
    fn el_tope_no_aplica_en_los_primeros_tres_meses() {
        let ctx = contexto();
        // El Bankomunal arrancó el mismo mes de la compra.
        ctx.creado_el("2026-08-01T00:00:00");

        // Un solo socio con el 100% de las acciones: imposible sin la excepción.
        let lote = compra(&ctx, &socio(&ctx, 1), 100_000.0).unwrap();
        assert_eq!(lote.cantidad, 10);

        let previo = ctx
            .acciones
            .previsualizar_compra(&ctx.banco, &socio(&ctx, 1), 10_000.0)
            .unwrap();
        assert!(previo.tope_en_periodo_de_gracia);
        assert!(!previo.supera_tope_participacion);
    }

    /// Pasado el tercer mes, el mismo caso sí se bloquea.
    #[test]
    fn el_tope_empieza_a_aplicar_desde_el_tercer_mes() {
        let ctx = contexto();
        ctx.creado_el("2026-05-01T00:00:00"); // 3 meses antes de agosto
        let err = compra(&ctx, &socio(&ctx, 1), 100_000.0).unwrap_err();
        assert!(err.to_string().contains("RN-02"));
    }

    /// RF-22: la compra entra al Libro de Ingresos y Egresos como VC.
    #[test]
    fn la_compra_queda_asentada_en_el_libro_como_venta_de_acciones() {
        let ctx = contexto();
        let s1 = socio(&ctx, 1);
        compra(&ctx, &s1, 40_000.0).unwrap();

        let libro = ctx.caja.listar_libro(&ctx.banco, FiltroLibro::default()).unwrap();
        assert_eq!(libro.len(), 1);
        let mov = &libro[0];
        assert_eq!(mov.codigo.as_str(), "VC");
        assert_eq!(mov.ingreso, 40_000.0);
        assert_eq!(mov.egreso, 0.0);
        assert_eq!(mov.socio_id.as_deref(), Some(s1.as_str()));
        assert!(mov.descripcion.contains("4 acciones"));
        // Y suma al saldo de caja.
        assert_eq!(ctx.caja.saldo_caja(&ctx.banco).unwrap(), 40_000.0);
    }

    /// RN-13: cambiar el valor nominal no reescribe lo que ya se compró.
    #[test]
    fn cambiar_el_nominal_no_altera_los_lotes_anteriores() {
        let ctx = contexto();
        let antiguo = compra(&ctx, &socio(&ctx, 1), 30_000.0).unwrap();
        assert_eq!(antiguo.valor_nominal_compra, 10_000.0);
        assert_eq!(antiguo.cantidad, 3);

        let mut cfg = ctx.config.obtener_configuracion(&ctx.banco).unwrap();
        cfg.valor_nominal = 20_000.0;
        ctx.config
            .actualizar_configuracion(&ctx.banco, &cfg, "Asamblea", "Ajuste de nominal")
            .unwrap();

        let nuevo = compra(&ctx, &socio(&ctx, 1), 40_000.0).unwrap();
        assert_eq!(nuevo.valor_nominal_compra, 20_000.0);
        assert_eq!(nuevo.cantidad, 2);

        // El lote viejo conserva su nominal y su cantidad.
        let lotes = ctx.acciones.lotes.listar_de_socio(&ctx.banco, &socio(&ctx, 1)).unwrap();
        let viejo = lotes.iter().find(|l| l.id == antiguo.id).unwrap();
        assert_eq!(viejo.valor_nominal_compra, 10_000.0);
        assert_eq!(viejo.cantidad, 3);
        assert_eq!(ctx.acciones.acciones_de_socio(&ctx.banco, &socio(&ctx, 1)).unwrap(), 5);
    }

    /// RF-105: el historial agrupa por mes y acumula el saldo.
    #[test]
    fn el_historial_agrupa_por_mes_con_saldo_acumulado() {
        let ctx = contexto();
        let comprar = |fecha: &str, monto: f64| {
            ctx.acciones
                .registrar_compra(
                    &ctx.banco,
                    NuevaCompra { socio_id: socio(&ctx, 1), fecha: fecha.into(), monto },
                )
                .unwrap();
        };
        comprar("2026-05-10", 30_000.0); // 3
        comprar("2026-06-02", 10_000.0); // 1
        comprar("2026-07-15", 10_000.0); // 2 compras en julio
        comprar("2026-07-20", 10_000.0);

        let historial = ctx.acciones.historial_de_socio(&ctx.banco, &socio(&ctx, 1)).unwrap();
        assert_eq!(historial.len(), 3, "un renglón por mes, no por compra");
        assert_eq!(historial[0].mes, "2026-05-01");
        assert_eq!(historial[0].compradas, 3);
        assert_eq!(historial[0].saldo, 3);
        assert_eq!(historial[1].saldo, 4);
        assert_eq!(historial[2].compradas, 2, "las dos compras de julio se suman");
        assert_eq!(historial[2].saldo, 6);
    }

    #[test]
    fn acciones_por_socio_devuelve_el_total_vigente_de_cada_uno() {
        let ctx = contexto();
        compra(&ctx, &socio(&ctx, 1), 30_000.0).unwrap();
        compra(&ctx, &socio(&ctx, 1), 20_000.0).unwrap();
        compra(&ctx, &socio(&ctx, 2), 10_000.0).unwrap();

        // Los ids son UUID, así que se consulta por socio en vez de fijar un orden.
        let por_socio = ctx.acciones.acciones_por_socio(&ctx.banco).unwrap();
        let acciones_de = |id: &str| por_socio.iter().find(|(s, _)| s == id).map(|(_, n)| *n);

        assert_eq!(por_socio.len(), 2);
        assert_eq!(acciones_de(&socio(&ctx, 1)), Some(5), "dos compras se suman");
        assert_eq!(acciones_de(&socio(&ctx, 2)), Some(1));
    }

    /// Simula meses ya cerrados con su % de colocación sellado (lo hará el Cierre).
    fn sellar_cierres(ctx: &Contexto, colocaciones: &[(&str, f64)]) {
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        for (mes, pct) in colocaciones {
            conn.execute(
                "INSERT INTO cierre_mes (id, mes, colocacion_pct, cuadra, fecha_cierre)
                 VALUES (?1, ?2, ?3, 1, ?2)",
                rusqlite::params![Uuid::new_v4().to_string(), mes, pct],
            )
            .unwrap();
        }
    }

    /// D-02: sin 3 meses cerrados el PPCFC no existe. El cupo lo reporta como pendiente
    /// y —a propósito— NO bloquea la venta, porque esa decisión sigue abierta.
    #[test]
    fn sin_tres_cierres_el_cupo_queda_pendiente_y_no_bloquea() {
        let ctx = contexto();
        sellar_cierres(&ctx, &[("2026-06-01", 95.0), ("2026-07-01", 90.0)]);

        let cupo = ctx.acciones.cupo_del_mes(&ctx.banco, "2026-08-05").unwrap();
        assert_eq!(
            cupo.autorizacion,
            AutorizacionVenta::SinDatosSuficientes { meses_cerrados: 2 }
        );
        assert_eq!(cupo.disponible_monto, None);
        // Y la compra sigue siendo posible.
        assert!(compra(&ctx, &socio(&ctx, 1), 50_000.0).is_ok());
    }

    /// RN-09: PPCFC de 92% cae en el tramo alto → 15% del total de acciones.
    #[test]
    fn con_ppcfc_alto_el_cupo_es_el_15_por_ciento_del_total() {
        let ctx = contexto();
        // 100 acciones en circulación.
        compra(&ctx, &socio(&ctx, 1), 1_000_000.0).unwrap();
        sellar_cierres(
            &ctx,
            &[("2026-05-01", 90.0), ("2026-06-01", 92.0), ("2026-07-01", 94.0)],
        );

        let cupo = ctx.acciones.cupo_del_mes(&ctx.banco, "2026-08-05").unwrap();
        match cupo.autorizacion {
            AutorizacionVenta::Autoriza {
                ppcfc_pct, rango_desde, rango_hasta, venta_pct, cupo_acciones, cupo_monto,
            } => {
                assert_eq!(ppcfc_pct, 92.0);
                assert_eq!((rango_desde, rango_hasta), (90.0, 100.0));
                assert_eq!(venta_pct, 15.0);
                assert_eq!(cupo_acciones, 15);
                assert_eq!(cupo_monto, 150_000.0);
            }
            otro => panic!("se esperaba autorización, llegó {otro:?}"),
        }
        // RN-15: un solo socio puede tomar el 20% de ese cupo.
        assert_eq!(cupo.tope_individual_monto, Some(30_000.0));
    }

    /// RN-09: por debajo del 80% no se venden acciones.
    #[test]
    fn con_ppcfc_bajo_no_se_autoriza_venta() {
        let ctx = contexto();
        sellar_cierres(
            &ctx,
            &[("2026-05-01", 70.0), ("2026-06-01", 75.0), ("2026-07-01", 80.0)],
        );
        let cupo = ctx.acciones.cupo_del_mes(&ctx.banco, "2026-08-05").unwrap();
        assert_eq!(cupo.autorizacion, AutorizacionVenta::NoAutoriza { ppcfc_pct: 75.0 });
        assert_eq!(cupo.disponible_monto, None);
    }

    /// Lo vendido en el mes descuenta del cupo disponible.
    #[test]
    fn lo_ya_vendido_descuenta_del_cupo_disponible() {
        let ctx = contexto();
        compra(&ctx, &socio(&ctx, 1), 1_000_000.0).unwrap(); // 100 acciones en agosto
        sellar_cierres(
            &ctx,
            &[("2026-05-01", 95.0), ("2026-06-01", 95.0), ("2026-07-01", 95.0)],
        );

        let cupo = ctx.acciones.cupo_del_mes(&ctx.banco, "2026-08-20").unwrap();
        assert_eq!(cupo.vendido_acciones, 100);
        assert_eq!(cupo.vendido_monto, 1_000_000.0);
        // El cupo (15% de 100 = 15 acciones = $150.000) ya está sobrepasado.
        assert_eq!(cupo.disponible_monto, Some(0.0), "no baja de cero");
    }

    #[test]
    fn un_socio_sin_compras_no_tiene_acciones_ni_historial() {
        let ctx = contexto();
        assert_eq!(ctx.acciones.acciones_de_socio(&ctx.banco, &socio(&ctx, 9)).unwrap(), 0);
        assert!(ctx.acciones.historial_de_socio(&ctx.banco, &socio(&ctx, 9)).unwrap().is_empty());
    }
}
