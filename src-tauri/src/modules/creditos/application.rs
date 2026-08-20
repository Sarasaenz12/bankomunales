use std::sync::Arc;

use uuid::Uuid;

use crate::core::error::AppError;

use super::domain::{
    calcular_tabla, resumir_tabla, Credito, CreditoPort, DecisionSolicitud, EstadoCredito,
    EstadoSolicitud, GarantiaCredito, GarantiaSolicitud, LibroContablePort, NuevaSolicitud,
    NuevoDesembolso, ParametrosCredito, ParametrosCreditoPort, RolGarantia,
    SociosParaCreditoPort, SolicitudCredito, SolicitudPort, FRECUENCIA_PAGO_MENSUAL,
    MAX_FIADORES,
};

/// Capa de Aplicación/Servicios del módulo de Créditos, Fase 1.
///
/// Orquesta los casos de uso CU-08 (Registrar Solicitud, RF-43..RF-52) y CU-09
/// (Desembolsar Crédito, RF-53..RF-62). Todas las reglas de negocio del dominio
/// viven aquí, sobre los puertos; los adaptadores SQLite y de Caja se inyectan.
pub struct CreditoService {
    solicitudes: Arc<dyn SolicitudPort>,
    creditos: Arc<dyn CreditoPort>,
    parametros: Arc<dyn ParametrosCreditoPort>,
    acciones: Arc<dyn crate::modules::creditos::domain::AccionesParaCreditoPort>,
    socios: Arc<dyn SociosParaCreditoPort>,
    libro: Arc<dyn LibroContablePort>,
}

impl CreditoService {
    pub fn new(
        solicitudes: Arc<dyn SolicitudPort>,
        creditos: Arc<dyn CreditoPort>,
        parametros: Arc<dyn ParametrosCreditoPort>,
        acciones: Arc<dyn crate::modules::creditos::domain::AccionesParaCreditoPort>,
        socios: Arc<dyn SociosParaCreditoPort>,
        libro: Arc<dyn LibroContablePort>,
    ) -> Self {
        Self { solicitudes, creditos, parametros, acciones, socios, libro }
    }

    /// RF-46 (CU-08): vista previa de la tabla de amortización que la pantalla de
    /// Solicitud muestra antes de guardar. Usa la tasa de interés ordinario vigente
    /// (RF-59), como hará el desembolso, para que la vista nunca mienta.
    pub fn previsualizar_tabla(
        &self,
        banco_id: &str,
        monto: f64,
        plazo: i64,
    ) -> Result<super::domain::TablaCredito, AppError> {
        let p = self.parametros.obtener(banco_id)?;
        self.tabla_con(banco_id, &p, monto, plazo)
    }

    /// RF-43, RF-45, RF-47, RF-48, RF-49 (CU-08): registra una Solicitud de Crédito.
    ///
    /// Reglas aplicadas:
    /// - Monto y plazo dentro de los topes del Bankomunal (RF-11).
    /// - RN-14: al menos un fiador que sea socio.
    /// - RF-48: máximo 2 fiadores por solicitud.
    /// - RF-45: capacidad de pago = Total Ingresos − Total Egresos.
    /// - El destino debe ser uno del catálogo (RF-47, RN-11): lo garantiza el enum.
    pub fn registrar_solicitud(
        &self,
        banco_id: &str,
        nueva: NuevaSolicitud,
    ) -> Result<SolicitudCredito, AppError> {
        let p = self.parametros.obtener(banco_id)?;
        if nueva.monto_solicitado <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto solicitado debe ser mayor a cero".into(),
            ));
        }
        if nueva.monto_solicitado > p.monto_maximo_credito {
            return Err(AppError::OperacionNoPermitida(format!(
                "El monto solicitado (${:.2}) supera el máximo del Bankomunal (${:.2})",
                nueva.monto_solicitado, p.monto_maximo_credito
            )));
        }
        if nueva.plazo_cuotas < 1 {
            return Err(AppError::OperacionNoPermitida(
                "El plazo debe ser de al menos 1 cuota".into(),
            ));
        }
        if nueva.plazo_cuotas > p.plazo_maximo_cuotas {
            return Err(AppError::OperacionNoPermitida(format!(
                "El plazo ({} cuotas) supera el máximo del Bankomunal ({} cuotas)",
                nueva.plazo_cuotas, p.plazo_maximo_cuotas
            )));
        }
        if nueva.fiadores.is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Todo crédito debe tener al menos un fiador (RN-14)".into(),
            ));
        }
        if nueva.fiadores.len() > MAX_FIADORES {
            return Err(AppError::OperacionNoPermitida(format!(
                "Se registran hasta {MAX_FIADORES} fiadores por solicitud (RF-48)"
            )));
        }

        // RF-49: cada fiador debe ser un socio del Bankomunal.
        let mut fiadores_socios = Vec::new();
        for fiador in &nueva.fiadores {
            let socio_id = self
                .socios
                .buscar_por_cedula(banco_id, &fiador.cedula)?
                .ok_or_else(|| {
                    AppError::OperacionNoPermitida(format!(
                        "El fiador con cédula «{}» no es socio del Bankomunal",
                        fiador.cedula
                    ))
                })?;
            if socio_id == nueva.socio_id {
                return Err(AppError::OperacionNoPermitida(
                    "El fiador no puede ser el mismo socio titular".into(),
                ));
            }
            if fiadores_socios.contains(&socio_id) {
                return Err(AppError::OperacionNoPermitida(
                    "Un mismo socio no puede repetirse como fiador".into(),
                ));
            }
            fiadores_socios.push(socio_id);
        }

        // RN-03 (RF-56): relación 1 a 5 con las acciones propias del socio.
        let acciones_titular = self.acciones.acciones_de_socio(banco_id, &nueva.socio_id)? as f64;
        let tope_rn03 = 5.0 * acciones_titular * p.valor_nominal;
        if nueva.monto_solicitado > tope_rn03 {
            return Err(AppError::OperacionNoPermitida(format!(
                "El monto solicitado (${:.2}) supera 5 veces las acciones propias del socio \
                 (${tope_rn03:.2} = {acciones_titular:.0} acciones × ${:.2} × 5) (RN-03)",
                nueva.monto_solicitado, p.valor_nominal
            )));
        }

        // RF-49 / RN-04: la garantía mínima se exige por partes — el titular con
        // sus acciones reales cubre pct_garantia_socio % y los fiadores, con las
        // que comprometen, cubren pct_garantia_fiador %.
        let fiadores_por_socio = nueva
            .fiadores
            .iter()
            .zip(fiadores_socios.iter())
            .map(|(f, sid)| (sid.clone(), f.acciones_comprometidas))
            .collect::<Vec<_>>();
        self.verificar_garantia_minima(banco_id, &p, &nueva.socio_id, nueva.monto_solicitado, &fiadores_por_socio)?;

        let fecha = hoy();
        let solicitud = SolicitudCredito {
            id: Uuid::new_v4().to_string(),
            socio_id: nueva.socio_id.clone(),
            fecha_solicitud: fecha.clone(),
            monto_solicitado: nueva.monto_solicitado,
            plazo_cuotas: nueva.plazo_cuotas,
            destino: nueva.destino,
            total_ingresos: nueva.total_ingresos,
            total_egresos: nueva.total_egresos,
            // RF-45: la capacidad de pago la calcula el sistema.
            capacidad_pago: (nueva.total_ingresos - nueva.total_egresos).max(0.0),
            estado: EstadoSolicitud::Pendiente,
            monto_aprobado: None,
            observacion: None,
            fecha_decision: None,
            decidida_por: None,
            garantias: Vec::new(),
        };

        // Garantías de la solicitud (RF-48): titular + fiadores propuestos.
        let mut garantias = vec![GarantiaSolicitud {
            id: Uuid::new_v4().to_string(),
            solicitud_id: solicitud.id.clone(),
            socio_id: solicitud.socio_id.clone(),
            rol: RolGarantia::Titular,
            acciones_comprometidas: 0.0,
        }];
        for (fiador, socio_id) in nueva.fiadores.iter().zip(fiadores_socios) {
            garantias.push(GarantiaSolicitud {
                id: Uuid::new_v4().to_string(),
                solicitud_id: solicitud.id.clone(),
                socio_id,
                rol: RolGarantia::Fiador,
                acciones_comprometidas: fiador.acciones_comprometidas,
            });
        }

        self.solicitudes.crear(banco_id, &solicitud, &garantias)?;
        Ok(solicitud)
    }

    /// RF-50, RF-51 (CU-08): decisión de la Junta sobre una solicitud pendiente.
    ///
    /// - Sólo las solicitudes Pendientes pueden decidirse.
    /// - RF-51: Diferida exige observación; Modificada exige el monto aprobado.
    /// - Una solicitud ya desembolsada no puede volver a decidirse.
    pub fn decidir_solicitud(
        &self,
        banco_id: &str,
        decision: DecisionSolicitud,
    ) -> Result<SolicitudCredito, AppError> {
        let mut solicitud = self
            .solicitudes
            .buscar_por_id(banco_id, &decision.solicitud_id)?
            .ok_or(AppError::OperacionNoPermitida(
                "La solicitud indicada no existe".into(),
            ))?;

        if self.creditos.buscar_por_solicitud(banco_id, &solicitud.id)?.is_some() {
            return Err(AppError::OperacionNoPermitida(
                "La solicitud ya tiene un crédito desembolsado".into(),
            ));
        }
        if solicitud.estado != EstadoSolicitud::Pendiente {
            return Err(AppError::OperacionNoPermitida(format!(
                "La solicitud ya fue decidida: quedó {}",
                solicitud.estado.as_str()
            )));
        }

        let monto_maximo = self.parametros.obtener(banco_id)?.monto_maximo_credito;
        match decision.decision {
            EstadoSolicitud::Diferida => {
                if decision.observacion.as_deref().map(str::trim).unwrap_or("").is_empty() {
                    return Err(AppError::OperacionNoPermitida(
                        "Una solicitud Diferida debe registrar la observación (RF-51)".into(),
                    ));
                }
            }
            EstadoSolicitud::Modificada => {
                let monto = decision.monto_aprobado.ok_or_else(|| {
                    AppError::OperacionNoPermitida(
                        "Una solicitud Modificada debe indicar el monto aprobado (RF-51)".into(),
                    )
                })?;
                if monto <= 0.0 {
                    return Err(AppError::OperacionNoPermitida(
                        "El monto aprobado debe ser mayor a cero".into(),
                    ));
                }
                if monto > monto_maximo {
                    return Err(AppError::OperacionNoPermitida(format!(
                        "El monto aprobado (${monto:.2}) supera el máximo del Bankomunal \
                         (${monto_maximo:.2})"
                    )));
                }
            }
            EstadoSolicitud::Negada => {
                if decision.monto_aprobado.is_some() {
                    return Err(AppError::OperacionNoPermitida(
                        "Una solicitud Negada no lleva monto aprobado".into(),
                    ));
                }
            }
            EstadoSolicitud::Pendiente => {
                return Err(AppError::OperacionNoPermitida(
                    "La decisión debe ser Aprobada, Modificada, Negada o Diferida".into(),
                ));
            }
            EstadoSolicitud::Aprobada => {}
        }

        solicitud.estado = decision.decision;
        solicitud.monto_aprobado = match decision.decision {
            EstadoSolicitud::Aprobada => decision.monto_aprobado.or(Some(solicitud.monto_solicitado)),
            EstadoSolicitud::Modificada => decision.monto_aprobado,
            _ => None,
        };
        solicitud.observacion = decision.observacion;
        solicitud.fecha_decision = Some(hoy());
        solicitud.decidida_por = Some(decision.decidida_por);

        self.solicitudes.actualizar(banco_id, &solicitud, &solicitud.garantias)?;
        Ok(solicitud)
    }

    /// RF-52: listado de solicitudes, opcionalmente filtrado por estado.
    pub fn listar_solicitudes(
        &self,
        banco_id: &str,
        estado: Option<EstadoSolicitud>,
    ) -> Result<Vec<SolicitudCredito>, AppError> {
        self.solicitudes.listar_por_estado(banco_id, estado)
    }

    /// RF-52/RF-54: listado de solicitudes aprobadas aún SIN desembolso, para que
    /// la pantalla de Desembolso sólo ofrezca las que se pueden desembolsar
    /// (excluye las que ya tienen crédito asociado).
    pub fn listar_solicitudes_desembolsables(
        &self,
        banco_id: &str,
    ) -> Result<Vec<SolicitudCredito>, AppError> {
        Ok(self
            .solicitudes
            .listar_por_estado(banco_id, Some(EstadoSolicitud::Aprobada))?
            .into_iter()
            .filter(|s| {
                self.creditos
                    .buscar_por_solicitud(banco_id, &s.id)
                    .map(|c| c.is_none())
                    .unwrap_or(false)
            })
            .collect())
    }

    /// Consulta el crédito desembolsado de una solicitud, si existe (RF-52).
    pub fn buscar_credito_por_solicitud(
        &self,
        banco_id: &str,
        solicitud_id: &str,
    ) -> Result<Option<Credito>, AppError> {
        self.creditos.buscar_por_solicitud(banco_id, solicitud_id)
    }

    /// RF-54 (CU-09): vista previa del desembolso — la misma tabla que se guardará,
    /// con la tasa de interés ordinario vigente (RF-59).
    pub fn previsualizar_desembolso(
        &self,
        banco_id: &str,
        monto: f64,
        plazo: i64,
    ) -> Result<super::domain::TablaCredito, AppError> {
        let p = self.parametros.obtener(banco_id)?;
        self.tabla_con(banco_id, &p, monto, plazo)
    }

    /// RF-53 a RF-62 (CU-09): desembolsa un crédito, ya sea desde una solicitud
    /// aprobada (RF-50) o cargado directamente (RF-44).
    ///
    /// Reglas aplicadas:
    /// - RF-55: monto dentro del máximo del Bankomunal.
    /// - RN-03 (RF-56): relación 1 a 5 — el monto no supera 5 veces el valor de las
    ///   acciones propias del socio.
    /// - RN-04 (RF-57): garantía en acciones — el % del titular sobre sus acciones y
    ///   el % de los fiadores sobre las suyas.
    /// - RN-05 (RF-58): no se admiten fiadores cruzados entre créditos vigentes.
    /// - RF-53: número secuencial asignado por el sistema.
    /// - RF-59: tasa de interés precargada desde la configuración.
    /// - RF-61: la tabla de amortización se calcula y se persiste cuota por cuota.
    /// - RF-62: el desembolso entra a la caja como Desembolso de Crédito (CON).
    pub fn registrar_desembolso(
        &self,
        banco_id: &str,
        nuevo: NuevoDesembolso,
    ) -> Result<Credito, AppError> {
        let p = self.parametros.obtener(banco_id)?;
        let fecha = fecha_o_hoy(nuevo.fecha.as_deref())?;

        // Origen del desembolso: solicitud aprobada o carga directa.
        let (solicitud, garantias_origen, monto, plazo) =
            match &nuevo.solicitud_id {
                Some(sid) => {
                    let sol = self.solicitudes.buscar_por_id(banco_id, sid)?.ok_or_else(|| {
                        AppError::OperacionNoPermitida(
                            "La solicitud indicada no existe".into(),
                        )
                    })?;
                    if sol.estado != EstadoSolicitud::Aprobada {
                        return Err(AppError::OperacionNoPermitida(format!(
                            "Sólo las solicitudes Aprobadas pueden desembolsarse; esta quedó {}",
                            sol.estado.as_str()
                        )));
                    }
                    if self.creditos.buscar_por_solicitud(banco_id, sid)?.is_some() {
                        return Err(AppError::OperacionNoPermitida(
                            "La solicitud ya fue desembolsada".into(),
                        ));
                    }
                    // El monto desembolsado es el aprobado por la Junta (RF-50).
                    let monto = sol.monto_aprobado.unwrap_or(sol.monto_solicitado);
                    if (monto - nuevo.monto).abs() > 0.01 {
                        return Err(AppError::OperacionNoPermitida(
                            "El monto del desembolso debe coincidir con el aprobado por la Junta"
                                .into(),
                        ));
                    }
                    let fiadores = sol
                        .garantias
                        .iter()
                        .filter(|g| g.rol == RolGarantia::Fiador)
                        .cloned()
                        .collect::<Vec<_>>();
                    let plazo = sol.plazo_cuotas;
                    (Some(sol), fiadores, monto, plazo)
                }
                None => {
                    if nuevo.plazo_cuotas < 1 {
                        return Err(AppError::OperacionNoPermitida(
                            "El plazo debe ser de al menos 1 cuota".into(),
                        ));
                    }
                    if nuevo.fiadores.is_empty() {
                        return Err(AppError::OperacionNoPermitida(
                            "Todo crédito debe tener al menos un fiador (RN-14)".into(),
                        ));
                    }
                    if nuevo.fiadores.len() > MAX_FIADORES {
                        return Err(AppError::OperacionNoPermitida(format!(
                            "Se registran hasta {MAX_FIADORES} fiadores (RF-48)"
                        )));
                    }
                    let mut fiadores = Vec::new();
                    for f in &nuevo.fiadores {
                        let socio_id = self
                            .socios
                            .buscar_por_cedula(banco_id, &f.cedula)?
                            .ok_or_else(|| {
                                AppError::OperacionNoPermitida(format!(
                                    "El fiador con cédula «{}» no es socio del Bankomunal",
                                    f.cedula
                                ))
                            })?;
                        if socio_id == nuevo.socio_id {
                            return Err(AppError::OperacionNoPermitida(
                                "El fiador no puede ser el mismo socio titular".into(),
                            ));
                        }
                        fiadores.push(GarantiaSolicitud {
                            id: Uuid::new_v4().to_string(),
                            solicitud_id: String::new(),
                            socio_id,
                            rol: RolGarantia::Fiador,
                            acciones_comprometidas: f.acciones_comprometidas,
                        });
                    }
                    (None, fiadores, nuevo.monto, nuevo.plazo_cuotas)
                }
            };

        self.validar_desembolso(banco_id, &p, &nuevo, &solicitud, monto, plazo)?;

        // RN-04 (RF-57): garantía en acciones, % titular + % fiadores.
        let (_acciones_titular, garantias) =
            self.garantia_suficiente(banco_id, &p, &nuevo, &garantias_origen, monto)?;

        // RF-53: número secuencial.
        let numero = self.creditos.siguiente_numero(banco_id)?;

        // RF-59: la tasa viene de la configuración, no del usuario.
        let tasa = p.tasa_interes_ordinario;

        // RF-61: tabla de amortización desde el mes siguiente al desembolso.
        let primer_vencimiento = sumar_un_mes(&fecha);
        let cuotas = calcular_tabla(monto, tasa, plazo, &primer_vencimiento);
        let fecha_vencimiento = cuotas.last().map(|c| c.fecha_vencimiento.clone()).unwrap_or(fecha.clone());

        let credito = Credito {
            id: Uuid::new_v4().to_string(),
            socio_id: nuevo.socio_id.clone(),
            numero,
            monto_original: monto,
            tasa,
            plazo_cuotas: plazo,
            cuota_actual: 1,
            saldo_pendiente: monto,
            destino: nuevo.destino,
            estatus: EstadoCredito::Vigente,
            fecha_solicitud: solicitud
                .as_ref()
                .map(|s| s.fecha_solicitud.clone())
                .unwrap_or_else(|| fecha.clone()),
            fecha_desembolso: fecha.clone(),
            frecuencia_pago: FRECUENCIA_PAGO_MENSUAL.to_string(),
            fecha_vencimiento,
            solicitud_id: nuevo.solicitud_id.clone(),
            garantias: Vec::new(),
        };

        let garantias_credito = garantias
            .iter()
            .map(|g| GarantiaCredito {
                id: Uuid::new_v4().to_string(),
                credito_id: credito.id.clone(),
                socio_id: g.socio_id.clone(),
                rol: g.rol,
                acciones_comprometidas: g.acciones_comprometidas,
            })
            .collect::<Vec<_>>();

        self.creditos.crear(banco_id, &credito, &cuotas, &garantias_credito)?;

        // RF-62: el desembolso sale de caja como Desembolso de Crédito (CON).
        self.libro.registrar_desembolso(
            banco_id,
            &fecha,
            monto,
            &credito.socio_id,
            &credito.id,
            &format!("Desembolso crédito Nº {} a {} cuotas", credito.numero, plazo),
        )?;

        Ok(credito)
    }

    /// Validaciones comunes a la apertura de un crédito (RF-55, RN-03).
    fn validar_desembolso(
        &self,
        banco_id: &str,
        p: &ParametrosCredito,
        nuevo: &NuevoDesembolso,
        solicitud: &Option<SolicitudCredito>,
        monto: f64,
        plazo: i64,
    ) -> Result<(), AppError> {
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto del desembolso debe ser mayor a cero".into(),
            ));
        }
        if monto > p.monto_maximo_credito {
            return Err(AppError::OperacionNoPermitida(format!(
                "El monto (${monto:.2}) supera el máximo del Bankomunal (${:.2})",
                p.monto_maximo_credito
            )));
        }
        if plazo < 1 {
            return Err(AppError::OperacionNoPermitida(
                "El plazo debe ser de al menos 1 cuota".into(),
            ));
        }
        if plazo > p.plazo_maximo_cuotas {
            return Err(AppError::OperacionNoPermitida(format!(
                "El plazo ({} cuotas) supera el máximo del Bankomunal ({} cuotas)",
                plazo, p.plazo_maximo_cuotas
            )));
        }

        // RN-03 (RF-56): relación 1 a 5 — el monto no puede superar 5 veces el valor
        // de las acciones propias del socio (acciones × valor nominal).
        let acciones_titular = self.acciones.acciones_de_socio(banco_id, &nuevo.socio_id)? as f64;
        let tope_rn03 = 5.0 * acciones_titular * p.valor_nominal;
        if monto > tope_rn03 {
            return Err(AppError::OperacionNoPermitida(format!(
                "El monto (${monto:.2}) supera 5 veces las acciones propias del socio \
                 (${tope_rn03:.2} = {acciones_titular:.0} acciones × ${:.2} × 5) (RN-03)",
                p.valor_nominal
            )));
        }

        // RN-05 (RF-58): sin fiadores cruzados con créditos vigentes. Un socio no
        // puede ser fiador de quien es su fiador (o titular) en otro crédito vigente.
        if solicitud.is_none() {
            let pares = self.creditos.pares_titular_fiador(banco_id)?;
            for f in &nuevo.fiadores {
                if pares.contains(&(f.cedula.clone(), nuevo.socio_id.clone()))
                    || pares.contains(&(nuevo.socio_id.clone(), f.cedula.clone()))
                {
                    return Err(AppError::OperacionNoPermitida(
                        "Fiadores cruzados con un crédito vigente (RN-05)".into(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// RF-49 / RN-04 (común a solicitud y desembolso): la garantía mínima se exige
    /// por partes — el titular con sus acciones reales cubre `pct_garantia_socio` %
    /// del monto y los fiadores, con las que comprometen, cubren `pct_garantia_fiador` %.
    /// Cada fiador debe poseer las acciones que compromete.
    fn verificar_garantia_minima(
        &self,
        banco_id: &str,
        p: &ParametrosCredito,
        socio_id: &str,
        monto: f64,
        fiadores: &[(String, f64)],
    ) -> Result<(), AppError> {
        let acciones_titular = self.acciones.acciones_de_socio(banco_id, socio_id)? as f64;
        let acciones_requeridas_titular = monto * p.pct_garantia_socio / 100.0 / p.valor_nominal;
        if p.pct_garantia_socio > 0.0 && acciones_titular < acciones_requeridas_titular {
            return Err(AppError::OperacionNoPermitida(format!(
                "El titular debe garantizar el {:.0}% del crédito con sus acciones \
                 (RN-04): necesita {:.2} acciones y tiene {:.0}",
                p.pct_garantia_socio, acciones_requeridas_titular, acciones_titular
            )));
        }

        // Las acciones comprometidas por los fiadores se fijan en la solicitud o
        // el desembolso: titular no compromete, sólo respalda; los fiadores comprometen.
        let comprometidas: f64 = fiadores.iter().map(|(_, a)| a).sum();
        let acciones_requeridas_fiadores = monto * p.pct_garantia_fiador / 100.0 / p.valor_nominal;
        if p.pct_garantia_fiador > 0.0 && comprometidas < acciones_requeridas_fiadores {
            return Err(AppError::OperacionNoPermitida(format!(
                "Los fiadores deben garantizar el {:.0}% del crédito con sus acciones \
                 (RN-04): necesitan comprometer {:.2} acciones y comprometen {:.2}",
                p.pct_garantia_fiador, acciones_requeridas_fiadores, comprometidas
            )));
        }

        // Cada fiador debe poseer las acciones que compromete.
        for (fid, acciones) in fiadores {
            let disponibles = self.acciones.acciones_de_socio(banco_id, fid)? as f64;
            if *acciones > disponibles {
                return Err(AppError::OperacionNoPermitida(format!(
                    "El fiador con id «{fid}» compromete {acciones:.0} acciones pero sólo tiene {disponibles:.0}"
                )));
            }
        }

        Ok(())
    }

    /// RN-04 (RF-57): el crédito debe quedar garantizado en acciones.
    ///
    /// - El titular debe tener acciones cuyo valor nominal cubra `pct_garantia_socio`
    ///   % del monto (por defecto 20%).
    /// - La suma de las acciones comprometidas por los fiadores debe cubrir
    ///   `pct_garantia_fiador` % (por defecto 20%); y cada fiador debe poseerlas.
    fn garantia_suficiente(
        &self,
        banco_id: &str,
        p: &ParametrosCredito,
        nuevo: &NuevoDesembolso,
        fiadores: &[GarantiaSolicitud],
        monto: f64,
    ) -> Result<(f64, Vec<GarantiaSolicitud>), AppError> {
        let pares = fiadores
            .iter()
            .map(|f| (f.socio_id.clone(), f.acciones_comprometidas))
            .collect::<Vec<_>>();
        self.verificar_garantia_minima(banco_id, p, &nuevo.socio_id, monto, &pares)?;

        let acciones_titular = self.acciones.acciones_de_socio(banco_id, &nuevo.socio_id)? as f64;
        let acciones_requeridas_titular = monto * p.pct_garantia_socio / 100.0 / p.valor_nominal;

        // La garantía del titular se registra como las acciones que respaldan su %.
        let mut garantias = fiadores.to_vec();
        garantias.insert(
            0,
            GarantiaSolicitud {
                id: Uuid::new_v4().to_string(),
                solicitud_id: nuevo.solicitud_id.clone().unwrap_or_default(),
                socio_id: nuevo.socio_id.clone(),
                rol: RolGarantia::Titular,
                acciones_comprometidas: acciones_requeridas_titular,
            },
        );

        Ok((acciones_titular, garantias))
    }

    /// Tabla de amortización con validación de parámetros y la tasa vigente.
    fn tabla_con(
        &self,
        _banco_id: &str,
        p: &ParametrosCredito,
        monto: f64,
        plazo: i64,
    ) -> Result<super::domain::TablaCredito, AppError> {
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto debe ser mayor a cero".into(),
            ));
        }
        if plazo < 1 {
            return Err(AppError::OperacionNoPermitida(
                "El plazo debe ser de al menos 1 cuota".into(),
            ));
        }
        if monto > p.monto_maximo_credito {
            return Err(AppError::OperacionNoPermitida(format!(
                "El monto (${monto:.2}) supera el máximo del Bankomunal (${:.2})",
                p.monto_maximo_credito
            )));
        }
        if plazo > p.plazo_maximo_cuotas {
            return Err(AppError::OperacionNoPermitida(format!(
                "El plazo ({} cuotas) supera el máximo del Bankomunal ({} cuotas)",
                plazo, p.plazo_maximo_cuotas
            )));
        }

        let cuotas = calcular_tabla(monto, p.tasa_interes_ordinario, plazo, &hoy());
        Ok(resumir_tabla(&cuotas, plazo))
    }
}

fn hoy() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Fecha del desembolso: la dada o hoy. Exige formato ISO `YYYY-MM-DD`.
fn fecha_o_hoy(fecha: Option<&str>) -> Result<String, AppError> {
    let f = fecha.unwrap_or("").trim().to_string();
    if f.is_empty() {
        return Ok(hoy());
    }
    if f.len() == 10 && f.as_bytes()[4] == b'-' && f.as_bytes()[7] == b'-' {
        Ok(f)
    } else {
        Err(AppError::OperacionNoPermitida(format!(
            "La fecha «{f}» no es válida: use YYYY-MM-DD"
        )))
    }
}

/// Suma un mes a una fecha ISO conservando el día (recorta al máximo del mes).
fn sumar_un_mes(fecha: &str) -> String {
    let anio: i64 = fecha.get(0..4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mes: i64 = fecha.get(5..7).and_then(|s| s.parse().ok()).unwrap_or(1);
    let dia: i64 = fecha.get(8..10).and_then(|s| s.parse().ok()).unwrap_or(1);
    let max_dia = if mes == 12 { 31 } else { dia };
    let (nuevo_anio, nuevo_mes) = if mes == 12 { (anio + 1, 1) } else { (anio, mes + 1) };
    format!("{nuevo_anio:04}-{nuevo_mes:02}-{max_dia:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::caja::application::CajaService;
    use crate::modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
    use crate::modules::configuracion::application::ConfigService;
    use crate::modules::configuracion::data::SqliteConfiguracion;
    use crate::modules::creditos::data::{
        AccionesParaCreditoAdapter, LibroViaCaja, ParametrosCreditoAdapter, SociosParaCreditoAdapter,
        SqliteCreditos, SqliteSolicitudes,
    };
    use crate::modules::creditos::domain::{
        DestinoCredito, FiadorSolicitud, NuevaSolicitud,
    };
    use crate::modules::socios::application::SocioService;
    use crate::modules::socios::data::SqliteSocios;
    use crate::modules::socios::domain::DatosSocio;

    struct Contexto {
        servicio: CreditoService,
        caja: Arc<CajaService>,
        config: ConfigService,
        socios: SocioService,
        banco: String,
        db: DbManager,
    }

    fn contexto() -> Contexto {
        let dir = std::env::temp_dir().join(format!("bkn_cred_test_{}", Uuid::new_v4()));
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
        let socios = SocioService::new(Arc::new(SqliteSocios::new(db.clone())));
        let servicio = CreditoService::new(
            Arc::new(SqliteSolicitudes::new(db.clone())),
            Arc::new(SqliteCreditos::new(db.clone())),
            Arc::new(ParametrosCreditoAdapter::new(db.clone())),
            Arc::new(AccionesParaCreditoAdapter::new(db.clone())),
            Arc::new(SociosParaCreditoAdapter::new(db.clone())),
            Arc::new(LibroViaCaja::new(caja.clone())),
        );

        // La config se crea con valores por defecto al primer acceso.
        config.obtener_configuracion(&banco).unwrap();
        Contexto { servicio, caja, config, socios, banco, db }
    }

    /// Registra el socio n (o devuelve el ya creado) y da su id.
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

    /// Da `cantidad` acciones a un socio, como si las hubiera comprado (RF-22).
    fn dar_acciones(ctx: &Contexto, socio_id: &str, cantidad: i64) {
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        conn.execute(
            "INSERT INTO lote_acciones (id, socio_id, mes_compra, cantidad)
             VALUES (?1, ?2, '2026-01-01', ?3)",
            rusqlite::params![Uuid::new_v4().to_string(), socio_id, cantidad],
        )
        .unwrap();
    }

    fn solicitud(ctx: &Contexto, titular: &str, fiador_cedula: &str, monto: f64) -> NuevaSolicitud {
        NuevaSolicitud {
            socio_id: titular.into(),
            monto_solicitado: monto,
            plazo_cuotas: 12,
            destino: DestinoCredito::Vivienda,
            total_ingresos: 1_000_000.0,
            total_egresos: 400_000.0,
            fiadores: vec![FiadorSolicitud {
                cedula: fiador_cedula.into(),
                acciones_comprometidas: 100.0,
            }],
        }
    }

    /// RF-43: una solicitud se registra Pendiente y con su capacidad de pago (RF-45).
    #[test]
    fn la_solicitud_se_registra_pendiente() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);
        let cedula_fiador = "1002".to_string();
        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, &cedula_fiador, 2_000_000.0))
            .unwrap();
        assert_eq!(s.estado, EstadoSolicitud::Pendiente);
        assert_eq!(s.capacidad_pago, 600_000.0);
        assert_eq!(s.garantias.len(), 0); // el DTO no trae garantías; son internas
        let _ = fiador;
    }

    /// RN-14: sin fiador no se registra la solicitud.
    #[test]
    fn la_solicitud_requiere_fiador() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let mut s = solicitud(&ctx, &titular, "1002", 2_000_000.0);
        s.fiadores.clear();
        let err = ctx.servicio.registrar_solicitud(&ctx.banco, s).unwrap_err();
        assert!(err.to_string().contains("fiador"));
    }

    /// RF-48: máximo 2 fiadores.
    #[test]
    fn la_solicitud_no_admite_mas_de_dos_fiadores() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        socio(&ctx, 2);
        socio(&ctx, 3);
        let mut s = solicitud(&ctx, &titular, "1002", 2_000_000.0);
        s.fiadores.push(FiadorSolicitud {
            cedula: "1003".into(),
            acciones_comprometidas: 50.0,
        });
        s.fiadores.push(FiadorSolicitud {
            cedula: "1004".into(),
            acciones_comprometidas: 50.0,
        });
        let err = ctx.servicio.registrar_solicitud(&ctx.banco, s).unwrap_err();
        assert!(err.to_string().contains("2 fiadores"));
    }

    /// RF-49: el fiador debe ser socio.
    #[test]
    fn el_fiador_debe_ser_socio() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let mut s = solicitud(&ctx, &titular, "999999", 2_000_000.0);
        s.fiadores[0].cedula = "999999".into();
        let err = ctx.servicio.registrar_solicitud(&ctx.banco, s).unwrap_err();
        assert!(err.to_string().contains("no es socio"));
    }

    /// RF-50/RF-51: Diferida exige observación.
    #[test]
    fn diferida_exige_observacion() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let socio2 = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &socio2, 100);
        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap();
        let err = ctx
            .servicio
            .decidir_solicitud(
                &ctx.banco,
                DecisionSolicitud {
                    solicitud_id: s.id,
                    decision: EstadoSolicitud::Diferida,
                    monto_aprobado: None,
                    observacion: None,
                    decidida_por: "Junta".into(),
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("observación"));
    }

    /// RF-51: Modificada exige monto aprobado.
    #[test]
    fn modificada_exige_monto_aprobado() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let socio2 = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &socio2, 100);
        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap();
        let err = ctx
            .servicio
            .decidir_solicitud(
                &ctx.banco,
                DecisionSolicitud {
                    solicitud_id: s.id,
                    decision: EstadoSolicitud::Modificada,
                    monto_aprobado: None,
                    observacion: None,
                    decidida_por: "Junta".into(),
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("monto aprobado"));
    }

    /// Flujo completo RF-43..RF-62: solicitud → aprobación → desembolso con asiento CON.
    #[test]
    fn desembolso_desde_solicitud_aprobada() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);

        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap();
        let aprobada = ctx
            .servicio
            .decidir_solicitud(
                &ctx.banco,
                DecisionSolicitud {
                    solicitud_id: s.id.clone(),
                    decision: EstadoSolicitud::Aprobada,
                    monto_aprobado: None,
                    observacion: None,
                    decidida_por: "Junta".into(),
                },
            )
            .unwrap();
        assert_eq!(aprobada.estado, EstadoSolicitud::Aprobada);

        let credito = ctx
            .servicio
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: Some(s.id.clone()),
                    socio_id: titular.clone(),
                    monto: 2_000_000.0,
                    plazo_cuotas: 12,
                    destino: DestinoCredito::Vivienda,
                    fiadores: vec![],
                    fecha: Some("2026-02-01".into()),
                },
            )
            .unwrap();

        // RF-53: número secuencial.
        assert_eq!(credito.numero, "001");
        assert_eq!(credito.monto_original, 2_000_000.0);
        assert_eq!(credito.plazo_cuotas, 12);
        assert_eq!(credito.cuota_actual, 1);
        assert_eq!(credito.saldo_pendiente, 2_000_000.0);
        assert_eq!(credito.estatus, EstadoCredito::Vigente);
        assert_eq!(credito.frecuencia_pago, FRECUENCIA_PAGO_MENSUAL);

        // RF-61: 12 cuotas guardadas.
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        let cuotas: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cuota WHERE credito_id = ?1",
                [&credito.id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(cuotas, 12);

        // RF-62: el asiento CON quedó en la caja.
        let con: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM movimiento_libro WHERE codigo = 'CON'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(con, 1);

        // La garantía del titular se registró con el % de acciones exigido (RN-04).
        let titular_acciones: f64 = conn
            .query_row(
                "SELECT acciones_comprometidas FROM garantia_credito
                 WHERE credito_id = ?1 AND rol = 'TITULAR'",
                [&credito.id],
                |r| r.get(0),
            )
            .unwrap();
        // 20% de $2.000.000 = $400.000 → 40 acciones a $10.000.
        assert!((titular_acciones - 40.0).abs() < 0.001);

        // RF-52: el crédito se recupera desde su solicitud ("Ver crédito").
        let encontrado = ctx
            .servicio
            .buscar_credito_por_solicitud(&ctx.banco, &s.id)
            .unwrap()
            .expect("debe existir el crédito de la solicitud");
        assert_eq!(encontrado.id, credito.id);
        assert_eq!(encontrado.solicitud_id.as_deref(), Some(s.id.as_str()));
    }

    /// RF-52: una solicitud sin desembolso no tiene crédito asociado.
    #[test]
    fn solicitud_sin_credito_devuelve_none() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);
        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap();
        assert!(ctx
            .servicio
            .buscar_credito_por_solicitud(&ctx.banco, &s.id)
            .unwrap()
.is_none());
    }

    /// RF-52/RF-54: una solicitud aprobada ya desembolsada no aparece en el listado
    /// de desembolsables (sólo quedan las pendientes de desembolsar).
    #[test]
    fn desembolsables_excluye_solicitudes_ya_desembolsadas() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);

        let s1 = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap();
        let s2 = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 1_500_000.0))
            .unwrap();
        for s in [&s1, &s2] {
            ctx.servicio
                .decidir_solicitud(
                    &ctx.banco,
                    DecisionSolicitud {
                        solicitud_id: s.id.clone(),
                        decision: EstadoSolicitud::Aprobada,
                        monto_aprobado: None,
                        observacion: None,
                        decidida_por: "Junta".into(),
                    },
                )
                .unwrap();
        }

        // Ambas aprobadas y sin desembolso → ambas aparecen.
        let inicial = ctx.servicio.listar_solicitudes_desembolsables(&ctx.banco).unwrap();
        assert_eq!(inicial.len(), 2);

        // Se desembolsa la primera → ya no debe estar en el listado.
        ctx.servicio
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: Some(s1.id.clone()),
                    socio_id: titular,
                    monto: 2_000_000.0,
                    plazo_cuotas: 12,
                    destino: DestinoCredito::Vivienda,
                    fiadores: vec![],
                    fecha: Some("2026-02-01".into()),
                },
            )
            .unwrap();

        let restantes = ctx.servicio.listar_solicitudes_desembolsables(&ctx.banco).unwrap();
        assert_eq!(restantes.len(), 1);
        assert_eq!(restantes[0].id, s2.id);
    }

    /// Sin acciones el titular no puede respaldar el crédito: con 0 acciones la
    /// relación 1:5 (RN-03) da un tope de $0 y rechaza cualquier monto, antes aún de
    /// llegar a la garantía en acciones de RN-04. Ahora se valida al registrar la
    /// solicitud (RF-49/RF-56).
    #[test]
    fn desembolso_exige_garantia_del_titular() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &fiador, 100);

        let err = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 2_000_000.0))
            .unwrap_err();
        assert!(err.to_string().contains("RN-03"), "esperaba error RN-03, fue: {err}");
    }

    /// RN-04: los fiadores deben comprometer acciones suficientes. Se valida al
    /// registrar la solicitud (RF-49).
    #[test]
    fn desembolso_exige_garantia_de_fiadores() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);

        // El fiador compromete 1 acción: muy poco para el 20% de $2.000.000.
        let mut s = solicitud(&ctx, &titular, "1002", 2_000_000.0);
        s.fiadores[0].acciones_comprometidas = 1.0;
        let err = ctx.servicio.registrar_solicitud(&ctx.banco, s).unwrap_err();
        assert!(err.to_string().contains("RN-04"), "esperaba error RN-04, fue: {err}");
    }

    /// RF-55: el monto no puede superar el máximo configurado.
    #[test]
    fn el_monto_respeta_el_maximo() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let s = ctx
            .servicio
            .registrar_solicitud(&ctx.banco, solicitud(&ctx, &titular, "1002", 6_000_000.0))
            .unwrap_err();
        assert!(s.to_string().contains("máximo"));
    }

    /// RN-03: el monto no puede superar 5 veces el valor de las acciones del socio.
    #[test]
    fn el_monto_respeta_la_relacion_1_a_5() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        // 30 acciones a $10.000 → tope 1:5 = 5 × 30 × 10.000 = $1.500.000.
        dar_acciones(&ctx, &titular, 30);
        dar_acciones(&ctx, &fiador, 100);

        let err = ctx
            .servicio
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: None,
                    socio_id: titular,
                    monto: 2_000_000.0, // supera el tope 1:5 de $1.500.000
                    plazo_cuotas: 12,
                    destino: DestinoCredito::Productivo,
                    fiadores: vec![FiadorSolicitud {
                        cedula: "1002".into(),
                        acciones_comprometidas: 100.0,
                    }],
                    fecha: Some("2026-02-01".into()),
                },
            )
            .unwrap_err();
        assert!(err.to_string().contains("RN-03"), "esperaba error RN-03, fue: {err}");
    }

    /// RF-44: desembolso directo sin solicitud, identificando fiadores por cédula.
    #[test]
    fn desembolso_directo_sin_solicitud() {
        let ctx = contexto();
        let titular = socio(&ctx, 1);
        let fiador = socio(&ctx, 2);
        dar_acciones(&ctx, &titular, 100);
        dar_acciones(&ctx, &fiador, 100);

        let credito = ctx
            .servicio
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: None,
                    socio_id: titular,
                    monto: 1_000_000.0,
                    plazo_cuotas: 6,
                    destino: DestinoCredito::Productivo,
                    fiadores: vec![FiadorSolicitud {
                        cedula: "1002".into(),
                        acciones_comprometidas: 50.0,
                    }],
                    fecha: Some("2026-02-01".into()),
                },
            )
            .unwrap();
        assert_eq!(credito.numero, "001");
        assert!(credito.solicitud_id.is_none());
    }
}
