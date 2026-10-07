use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::error::AppError;
use crate::modules::acciones::application::AccionesService;
use crate::modules::acciones::domain::meses_entre;
use crate::modules::caja::application::CajaService;
use crate::modules::caja::domain::CodigoOperacion;
use crate::modules::cierre::application::CierreService;
use crate::modules::configuracion::application::ConfigService;
use crate::modules::creditos::application::CreditoService;
use crate::modules::creditos::domain::EstadoCredito;
use crate::modules::socios::application::SocioService;
use crate::modules::socios::domain::EstatusSocio;

use super::domain::{ParametrosLiquidacion, SocioLiquidacion};

/// Insumos de la Liquidación Final ya prellenados con lo que el sistema conoce, para que
/// la pantalla los muestre editables (la Asamblea ajusta incobrables y fondos al cerrar).
#[derive(Debug, Clone, Serialize)]
pub struct DatosLiquidacion {
    pub parametros: ParametrosLiquidacion,
    pub socios: Vec<SocioLiquidacion>,
}

/// Insumos de la liquidación de UN socio (su retiro).
#[derive(Debug, Clone, Serialize)]
pub struct DatosLiquidacionSocio {
    pub socio_id: String,
    pub nombre: String,
    pub cedula: String,
    pub caps_activas: i64,
    /// RF-29: acciones comprometidas como garantía de un crédito vigente. No se pueden
    /// liquidar (RF-30); las libres = caps_activas − acciones_comprometidas.
    pub acciones_comprometidas: f64,
    pub acciones_comprometidas_fiador: f64,
    pub ganancia_devengada: f64,
    pub deuda: f64,
    pub valor_nominal: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatosRetiroSocio {
    pub socio_id: String,
    pub fecha: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatosPagoDeudaPendiente {
    pub socio_id: String,
    pub fecha: String,
    pub monto: f64,
}

/// Qué hace el socio con el valor de las acciones que vende en una liquidación PARCIAL:
/// lo recibe en efectivo, o lo abona a su deuda (RF-28, decisión del cliente).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestinoLiquidacionParcial {
    Efectivo,
    AbonoDeuda,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatosLiquidacionParcial {
    pub socio_id: String,
    pub cantidad: i64,
    pub fecha: String,
    pub destino: DestinoLiquidacionParcial,
}

/// Vista previa de una venta parcial (sólo cálculo, no mueve nada): el capital de las
/// acciones a liquidar y la ganancia que cobrarían (sólo la de lotes que cumplieron el
/// año, RN-10). Si `bloqueo` trae texto, la venta no se puede ejecutar así.
#[derive(Debug, Clone, Serialize)]
pub struct PreviewLiquidacionParcial {
    pub cantidad: i64,
    pub libres: i64,
    pub capital: f64,
    pub ganancia: f64,
    pub valor: f64,
    pub deuda: f64,
    pub bloqueo: Option<String>,
}

/// Resultado de ejecutar una venta parcial: cuánto se liquidó y cómo se distribuyó.
#[derive(Debug, Clone, Serialize)]
pub struct ResultadoLiquidacionParcial {
    pub capital: f64,
    pub ganancia: f64,
    pub valor: f64,
    pub abonado_deuda: f64,
    pub pagado_efectivo: f64,
    pub nueva_deuda: f64,
    pub acciones_liquidadas: i64,
    pub acciones_restantes: i64,
}

/// Capa de Aplicación de la Liquidación: orquesta el retiro de un socio (RF-28/RF-38), el
/// pago de deuda pendiente (RF-74) y la preparación de la Liquidación Final, sobre los
/// servicios de los módulos existentes. Vive aparte de la presentación para poder probar
/// la reconciliación (Activo = Pasivo) sin pasar por los comandos Tauri.
pub struct LiquidacionService {
    socios: Arc<SocioService>,
    acciones: Arc<AccionesService>,
    creditos: Arc<CreditoService>,
    cierre: Arc<CierreService>,
    caja: Arc<CajaService>,
    config: Arc<ConfigService>,
}

impl LiquidacionService {
    pub fn new(
        socios: Arc<SocioService>,
        acciones: Arc<AccionesService>,
        creditos: Arc<CreditoService>,
        cierre: Arc<CierreService>,
        caja: Arc<CajaService>,
        config: Arc<ConfigService>,
    ) -> Self {
        Self { socios, acciones, creditos, cierre, caja, config }
    }

    /// Reúne los insumos de la Liquidación Final (caja, créditos por cobrar, acciones y
    /// nominal); los ajustes (incobrables y fondos) llegan en cero / desde configuración.
    pub fn preparar_liquidacion_final(&self, banco_id: &str) -> Result<DatosLiquidacion, AppError> {
        let caja = self.caja.saldo_caja(banco_id)?;
        let cfg = self.config.obtener_configuracion(banco_id)?;
        let saldos = self.creditos.saldos_vigentes_por_socio(banco_id)?;
        let acciones = self.acciones.acciones_por_socio(banco_id)?;
        let socios = self.socios.listar(banco_id)?;

        let creditos_por_cobrar: f64 = saldos.iter().map(|(_, s)| *s).sum();

        let socios_liq: Vec<SocioLiquidacion> = socios
            .iter()
            .map(|s| {
                let caps_activas = acciones
                    .iter()
                    .find(|(id, _)| *id == s.id)
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                let credito = saldos
                    .iter()
                    .find(|(id, _)| *id == s.id)
                    .map(|(_, v)| *v)
                    .unwrap_or(0.0);
                SocioLiquidacion {
                    socio_id: s.id.clone(),
                    nombre: s.nombre_completo(),
                    cedula: s.cedula.clone(),
                    caps_ganancia: caps_activas,
                    caps_activas,
                    credito,
                }
            })
            .collect();

        Ok(DatosLiquidacion {
            parametros: ParametrosLiquidacion {
                creditos_por_cobrar,
                caja,
                incobrables: 0.0,
                pct_fondo_gastos: cfg.pct_fondo_gastos,
                pct_fondo_incobrables: cfg.pct_fondo_incobrables,
                valor_nominal: cfg.valor_nominal,
            },
            socios: socios_liq,
        })
    }

    /// Insumos de la liquidación de un socio: acciones activas, ganancia que se le pagaría
    /// hoy (RN-10, sólo lotes que cumplieron el año), deuda vigente y el nominal.
    pub fn preparar_liquidacion_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
    ) -> Result<DatosLiquidacionSocio, AppError> {
        let socio = self.socios.obtener(banco_id, socio_id)?;
        let caps_activas = self.acciones.acciones_de_socio(banco_id, socio_id)?;
        let acciones_comprometidas = self
            .creditos
            .acciones_comprometidas_de_socio(banco_id, socio_id)?;
        let acciones_comprometidas_fiador = self
            .creditos
            .acciones_comprometidas_como_fiador(banco_id, socio_id)?;

        // Lo que el retiro pagaría hoy: sólo la ganancia de lotes que ya cumplieron el año
        // (RN-10). La de lotes sin cumplir no se paga (se libera al colectivo).
        let hoy = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let ganancia_devengada = self
            .cierre
            .ganancia_cumplida_de_socio(banco_id, socio_id, &hoy)?;
        let cfg = self.config.obtener_configuracion(banco_id)?;
        let deuda = self
            .creditos
            .saldos_vigentes_por_socio(banco_id)?
            .into_iter()
            .find(|(id, _)| id == socio_id)
            .map(|(_, v)| v)
            .unwrap_or(0.0);

        Ok(DatosLiquidacionSocio {
            socio_id: socio_id.to_string(),
            nombre: socio.nombre_completo(),
            cedula: socio.cedula,
            caps_activas,
            acciones_comprometidas,
            acciones_comprometidas_fiador,
            ganancia_devengada,
            deuda,
            valor_nominal: cfg.valor_nominal,
        })
    }

    /// RF-28 / RF-38: ejecuta el retiro (liquidación total) de un socio dejando el Balance
    /// cuadrado (Activo = Pasivo). Ver el detalle de cada paso en los comentarios.
    ///
    /// LIMITACIÓN CONOCIDA (atomicidad): esta operación hace varias escrituras en distintos
    /// módulos (cierre, acciones, créditos, caja, socios), cada una en su propia transacción
    /// SQLite —no hay una transacción única que las envuelva ni compensación si una falla a
    /// mitad—. Las validaciones que pueden fallar (p. ej. fiador vigente) se hacen ANTES de
    /// tocar nada, así que en operación normal la secuencia se completa entera; el único
    /// escenario de inconsistencia es un corte de energía / caída del proceso entre dos de
    /// las escrituras. Se asume ese riesgo a propósito (app de escritorio mono-usuario, ADR-03):
    /// cerrarlo exigiría un unit-of-work que enhebre una sola transacción por los cinco
    /// módulos, cuyo costo/riesgo no compensa hoy. Si se endurece la persistencia, este es
    /// el punto a convertir en atómico. Ver memoria [[cuadre-caja-reconciliacion]].
    pub fn ejecutar_retiro_socio(
        &self,
        banco_id: &str,
        datos: DatosRetiroSocio,
    ) -> Result<(), AppError> {
        let prev = self.preparar_liquidacion_socio(banco_id, &datos.socio_id)?;

        if prev.acciones_comprometidas_fiador > 0.0 {
            return Err(AppError::OperacionNoPermitida(format!(
                "El socio no se puede retirar porque {} de sus acciones respaldan un crédito como FIADOR.",
                prev.acciones_comprometidas_fiador
            )));
        }

        // 1. Saldar los devengos por RN-10, ANTES de liquidar los lotes: los que cumplieron
        //    el año se pagan (PAGADA, salen del Pasivo); los que no, se liberan (LIBERADA,
        //    quedan colectivos). Devuelve la ganancia efectiva a pagarle.
        let ganancia = self
            .cierre
            .liquidar_devengos_de_socio(banco_id, &datos.socio_id, &datos.fecha)?;

        // El capital que se le devuelve es el que invirtió (nominal congelado por lote), que
        // es exactamente lo que su retiro baja del capital en acciones del Pasivo.
        let capital = self
            .acciones
            .capital_activo_de_socio(banco_id, &datos.socio_id)?;
        let valor_a_pagar = capital + ganancia - prev.deuda;
        let queda_debiendo = valor_a_pagar < 0.0;

        // 2. Liquidar todas sus acciones (baja su capital del Pasivo).
        let liquidadas = self
            .acciones
            .liquidar_lotes_de_socio(banco_id, &datos.socio_id, &datos.fecha)?;

        // 3. Si tenía crédito vigente, marcarlo PAGADO (sale de la cartera): su saldo se
        //    salda contra la liquidación.
        if prev.deuda > 0.0 {
            let creditos = self.creditos.listar_creditos(banco_id)?;
            if let Some(cred) = creditos
                .iter()
                .find(|c| c.socio_id == datos.socio_id && c.estatus == EstadoCredito::Vigente)
            {
                self.creditos.marcar_credito_pagado(banco_id, &cred.id)?;
            }
        }

        // 4. Cierre monetario del retiro.
        let mut saldo_incobrable = 0.0;
        if valor_a_pagar > 0.0 {
            // El Banko le paga el neto (capital + ganancia − deuda) por Liquidación (LC).
            self.caja.registrar_asiento_de_modulo(
                banco_id,
                CodigoOperacion::LiquidacionAcciones,
                datos.fecha.clone(),
                valor_a_pagar,
                format!("Liquidación por retiro de socio ({liquidadas} acciones)"),
                Some(datos.socio_id.clone()),
                None,
            )?;
        } else if queda_debiendo {
            // Queda debiendo: el faltante se absorbe del Fondo de Incobrables (RF-35/RN-08)
            // y se guarda en el socio para cobrarlo luego (PDP). No hay egreso de caja.
            saldo_incobrable = valor_a_pagar.abs();
            self.cierre
                .ajustar_fondo_incobrables(banco_id, -saldo_incobrable)?;
        }

        // 5. Marcar socio retirado.
        let estatus = if queda_debiendo {
            EstatusSocio::RetiradoConDeuda
        } else {
            EstatusSocio::RetiradoVoluntario
        };
        self.socios.marcar_retiro(
            banco_id,
            &datos.socio_id,
            estatus,
            &datos.fecha,
            saldo_incobrable,
        )?;

        Ok(())
    }

    /// RF-28 (venta PARCIAL de acciones): vista previa sin tocar datos. Calcula, sobre el
    /// plan FIFO (los lotes más antiguos primero), el capital de las `cantidad` acciones y
    /// la ganancia que se pagaría (sólo la de lotes que cumplieron el año, RN-10).
    pub fn previsualizar_liquidacion_parcial(
        &self,
        banco_id: &str,
        socio_id: &str,
        cantidad: i64,
        fecha: &str,
    ) -> Result<PreviewLiquidacionParcial, AppError> {
        let prev = self.preparar_liquidacion_socio(banco_id, socio_id)?;
        let comprometidas = prev.acciones_comprometidas.ceil() as i64;
        let libres = (prev.caps_activas - comprometidas).max(0);

        let lotes = self.acciones.lotes_activos_de_socio(banco_id, socio_id)?;
        let pendientes = self.cierre.devengo_pendiente_por_lote(banco_id)?;
        let pendiente_de = |id: &str| -> f64 {
            pendientes
                .iter()
                .find(|(lid, _)| lid == id)
                .map(|(_, p)| *p)
                .unwrap_or(0.0)
        };

        let mut restante = cantidad.max(0);
        let mut capital = 0.0;
        let mut ganancia = 0.0;
        for lote in &lotes {
            if restante <= 0 {
                break;
            }
            let take = restante.min(lote.cantidad);
            capital += take as f64 * lote.valor_nominal_compra;
            if meses_entre(&lote.mes_compra, fecha).unwrap_or(0) >= 12 {
                let p = pendiente_de(&lote.id);
                ganancia += (p * take as f64 / lote.cantidad as f64).round();
            }
            restante -= take;
        }

        let bloqueo = if cantidad < 1 {
            Some("Indique cuántas acciones desea liquidar (al menos 1).".into())
        } else if cantidad > libres {
            Some(format!(
                "Sólo puede liquidar entre 1 y {libres} acciones libres ({comprometidas} están \
                 comprometidas en garantía)."
            ))
        } else {
            None
        };

        let capital = capital.round();
        Ok(PreviewLiquidacionParcial {
            cantidad,
            libres,
            capital,
            ganancia,
            valor: capital + ganancia,
            deuda: prev.deuda,
            bloqueo,
        })
    }

    /// RF-28 (venta PARCIAL de acciones): el socio vende `cantidad` acciones libres y sigue
    /// activo. Recibe el capital de esas acciones + la ganancia de las que cumplieron el año
    /// (RN-10); la ganancia de las que no, se libera al colectivo. El valor se le paga en
    /// efectivo o se abona a su deuda, según `destino`. El Balance queda cuadrado.
    ///
    /// LIMITACIÓN CONOCIDA (atomicidad): como el retiro total, encadena escrituras en varios
    /// módulos (acciones, cierre, créditos/caja) sin una transacción única que las envuelva.
    /// Las validaciones se hacen ANTES de tocar nada; el único riesgo es un corte a mitad de
    /// la secuencia. Se asume a propósito (ADR-03). Ver [[cuadre-caja-reconciliacion]].
    pub fn ejecutar_liquidacion_parcial(
        &self,
        banco_id: &str,
        datos: DatosLiquidacionParcial,
    ) -> Result<ResultadoLiquidacionParcial, AppError> {
        let prev = self.preparar_liquidacion_socio(banco_id, &datos.socio_id)?;
        let comprometidas = prev.acciones_comprometidas.ceil() as i64;
        let libres = (prev.caps_activas - comprometidas).max(0);

        if datos.cantidad < 1 {
            return Err(AppError::OperacionNoPermitida(
                "Indique cuántas acciones desea liquidar (al menos 1).".into(),
            ));
        }
        if datos.cantidad > libres {
            return Err(AppError::OperacionNoPermitida(format!(
                "El socio sólo tiene {libres} acciones libres para liquidar ({comprometidas} están \
                 comprometidas en garantía)."
            )));
        }

        // 1. Liquida las acciones (FIFO, partiendo un lote si hace falta): baja el capital.
        let trozos =
            self.acciones
                .liquidar_acciones_parcial(banco_id, &datos.socio_id, datos.cantidad, &datos.fecha)?;
        let capital: f64 = trozos.iter().map(|t| t.capital).sum::<f64>().round();
        let acciones_liquidadas: i64 = trozos.iter().map(|t| t.cantidad).sum();

        // 2. Salda los devengos de esos trozos (RN-10): paga los que cumplieron el año,
        //    libera los demás. Devuelve la ganancia a pagarle.
        let lotes_mes: Vec<(String, String)> = trozos
            .iter()
            .map(|t| (t.lote_id.clone(), t.mes_compra.clone()))
            .collect();
        let ganancia = self
            .cierre
            .liquidar_devengos_de_lotes(banco_id, &lotes_mes, &datos.fecha)?;

        let valor = (capital + ganancia).round();

        // 3. Distribuye el valor según lo que eligió el socio.
        let (abonado_deuda, pagado_efectivo, nueva_deuda) = match datos.destino {
            DestinoLiquidacionParcial::Efectivo => {
                if valor > 0.0 {
                    self.caja.registrar_asiento_de_modulo(
                        banco_id,
                        CodigoOperacion::LiquidacionAcciones,
                        datos.fecha.clone(),
                        valor,
                        format!("Venta parcial de {acciones_liquidadas} acciones"),
                        Some(datos.socio_id.clone()),
                        None,
                    )?;
                }
                (0.0, valor, prev.deuda)
            }
            DestinoLiquidacionParcial::AbonoDeuda => {
                let (abonado, saldo_restante) =
                    self.creditos
                        .abonar_deuda_de_socio(banco_id, &datos.socio_id, valor)?;

                // Se asientan las dos caras en el Libro para que quede la traza completa: el
                // Banko paga el valor de las acciones (LC, egreso) y el socio lo abona a su
                // crédito (PC, ingreso). El neto en caja es el excedente (lo que sobró de la
                // deuda), y el Balance sigue cuadrando (ni LC ni PC son ingreso repartible).
                if valor > 0.0 {
                    self.caja.registrar_asiento_de_modulo(
                        banco_id,
                        CodigoOperacion::LiquidacionAcciones,
                        datos.fecha.clone(),
                        valor,
                        format!("Venta parcial de {acciones_liquidadas} acciones"),
                        Some(datos.socio_id.clone()),
                        None,
                    )?;
                }
                if abonado > 0.0 {
                    self.caja.registrar_asiento_de_modulo(
                        banco_id,
                        CodigoOperacion::PagoCuota,
                        datos.fecha.clone(),
                        abonado,
                        format!(
                            "Abono a crédito con venta parcial de {acciones_liquidadas} acciones"
                        ),
                        Some(datos.socio_id.clone()),
                        None,
                    )?;
                }

                // Si el valor supera la deuda, el excedente queda como neto pagado en efectivo.
                let excedente = (valor - abonado).round();
                (abonado, excedente, saldo_restante)
            }
        };

        let acciones_restantes = self.acciones.acciones_de_socio(banco_id, &datos.socio_id)?;

        Ok(ResultadoLiquidacionParcial {
            capital,
            ganancia,
            valor,
            abonado_deuda,
            pagado_efectivo,
            nueva_deuda,
            acciones_liquidadas,
            acciones_restantes,
        })
    }

    /// RF-74 a RF-76: pago de deuda pendiente de un socio retirado con deuda. El dinero
    /// entra a caja (PDP) y restituye el Fondo de Incobrables por el mismo monto (que se
    /// había absorbido al retirarlo), para que el Balance siga cuadrando.
    pub fn registrar_pago_deuda_pendiente(
        &self,
        banco_id: &str,
        datos: DatosPagoDeudaPendiente,
    ) -> Result<f64, AppError> {
        let socio = self.socios.obtener(banco_id, &datos.socio_id)?;
        if socio.estatus != EstatusSocio::RetiradoConDeuda || socio.saldo_incobrable <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El socio no se encuentra en estado RETIRADO_CON_DEUDA.".into(),
            ));
        }
        if datos.monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida(
                "El monto a abonar debe ser mayor a cero.".into(),
            ));
        }
        if datos.monto > socio.saldo_incobrable {
            return Err(AppError::OperacionNoPermitida(format!(
                "El abono de ${:.2} supera el saldo pendiente de ${:.2}.",
                datos.monto, socio.saldo_incobrable
            )));
        }

        let nuevo_saldo =
            self.socios
                .abonar_deuda_incobrable(banco_id, &datos.socio_id, datos.monto)?;

        self.caja.registrar_asiento_de_modulo(
            banco_id,
            CodigoOperacion::PagoDeudaPendiente,
            datos.fecha,
            datos.monto,
            format!("Abono a deuda de socio retirado. Restan ${nuevo_saldo:.2}"),
            Some(datos.socio_id.clone()),
            None,
        )?;

        // Restituye el Fondo de Incobrables (absorbido al retirar al socio, RF-35).
        self.cierre.ajustar_fondo_incobrables(banco_id, datos.monto)?;

        Ok(nuevo_saldo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::acciones::domain::NuevaCompra;
    use crate::modules::caja::domain::CodigoOperacion;
    use crate::modules::cierre::application::CierreService;
    use crate::modules::configuracion::application::ConfigService;
    use crate::modules::creditos::application::CreditoService;
    use crate::modules::socios::domain::DatosSocio;
    use uuid::Uuid;

    // ── Alias de los adaptadores SQLite de cada módulo (nombres que chocan entre sí) ──
    use crate::modules::auditoria::data::SqliteAuditoria;

    struct Ctx {
        liq: LiquidacionService,
        caja: Arc<CajaService>,
        acciones: Arc<AccionesService>,
        creditos: Arc<CreditoService>,
        cierre: Arc<CierreService>,
        socios: Arc<SocioService>,
        db: DbManager,
        banco: String,
    }

    /// Monta TODOS los servicios sobre una sola base, como en la app real, para probar el
    /// retiro de punta a punta (Activo = Pasivo).
    fn montar() -> Ctx {
        let dir = std::env::temp_dir().join(format!("bkn_liq_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let banco = Uuid::new_v4().to_string();
        db.open_banco_db(&banco).unwrap();

        let caja = Arc::new(CajaService::new(
            Arc::new(crate::modules::caja::data::SqliteLibro::new(db.clone())),
            Arc::new(crate::modules::caja::data::SqliteFondoGastos::new(db.clone())),
            Arc::new(crate::modules::caja::data::SqliteBienes::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        ));
        let config = Arc::new(ConfigService::new(
            Arc::new(crate::modules::configuracion::data::SqliteConfiguracion::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        ));
        let socios = Arc::new(SocioService::new(Arc::new(
            crate::modules::socios::data::SqliteSocios::new(db.clone()),
        )));
        let acciones = Arc::new(AccionesService::new(
            Arc::new(crate::modules::acciones::data::SqliteLotesAcciones::new(db.clone())),
            Arc::new(crate::modules::acciones::data::SqliteParametrosAcciones::new(db.clone())),
            Arc::new(crate::modules::acciones::data::LibroViaCaja::new(caja.clone())),
            Arc::new(crate::modules::acciones::data::SqliteCierres::new(db.clone())),
        ));
        let creditos = Arc::new(CreditoService::new(
            Arc::new(crate::modules::creditos::data::SqliteSolicitudes::new(db.clone())),
            Arc::new(crate::modules::creditos::data::SqliteCreditos::new(db.clone())),
            Arc::new(crate::modules::creditos::data::ParametrosCreditoAdapter::new(db.clone())),
            Arc::new(crate::modules::creditos::data::AccionesParaCreditoAdapter::new(db.clone())),
            Arc::new(crate::modules::creditos::data::SociosParaCreditoAdapter::new(db.clone())),
            Arc::new(crate::modules::creditos::data::LibroViaCaja::new(caja.clone())),
        ));
        let cierre = Arc::new(CierreService::new(
            Arc::new(crate::modules::cierre::data::SqliteCierres::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteReparto::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteLibroCierre::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteCartera::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteBienesCierre::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteFondosCierre::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteLotesCierre::new(db.clone())),
            Arc::new(crate::modules::cierre::data::SqliteSociosCierre::new(db.clone())),
            Arc::new(crate::modules::cierre::data::LibroViaCaja::new(caja.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        ));

        config.obtener_configuracion(&banco).unwrap();
        // El Banko se creó junto con la primera compra, para que RN-02 esté en gracia.
        db.open_banco_db(&banco)
            .unwrap()
            .execute(
                "UPDATE configuracion SET fecha_creacion = '2025-01-01' WHERE id = ?1",
                [banco.as_str()],
            )
            .unwrap();

        let liq = LiquidacionService::new(
            socios.clone(),
            acciones.clone(),
            creditos.clone(),
            cierre.clone(),
            caja.clone(),
            config.clone(),
        );
        Ctx { liq, caja, acciones, creditos, cierre, socios, db, banco }
    }

    fn socio_ana(ctx: &Ctx) -> String {
        ctx.socios
            .registrar(
                &ctx.banco,
                DatosSocio {
                    cedula: "111".into(),
                    nombres: "Ana".into(),
                    apellidos: "Paz".into(),
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

    /// Retiro total de punta a punta: un socio con acciones y ganancia devengada (de un
    /// cierre real) se retira; el sistema le paga su capital + la ganancia cumplida y el
    /// Balance queda cuadrado (Activo = Pasivo), con sólo los fondos retenidos.
    #[test]
    fn el_retiro_total_deja_el_balance_cuadrado() {
        let ctx = montar();
        let socio_id = socio_ana(&ctx);

        // Compra 10 acciones a $10.000 = $100.000 (enero 2025 → cumplen el año al retirarse).
        ctx.acciones
            .registrar_compra(
                &ctx.banco,
                NuevaCompra { socio_id: socio_id.clone(), fecha: "2025-01-05".into(), monto: 100_000.0 },
            )
            .unwrap();

        // Interés ordinario del mes (ingreso repartible), julio 2026.
        ctx.caja
            .registrar_asiento_de_modulo(
                &ctx.banco,
                CodigoOperacion::InteresOrdinario,
                "2026-07-15".into(),
                6_000.0,
                "Interés".into(),
                Some(socio_id.clone()),
                None,
            )
            .unwrap();

        // Cierra julio: retiene $600 + $600 a los fondos y devenga $4.800 al lote del socio.
        ctx.cierre.ejecutar_cierre(&ctx.banco, "2026-07-01", "Tester").unwrap();

        // El estado previo cuadra: efectivo $106.000 = capital $100.000 + fondos $1.200 +
        // ganancias $4.800.
        let (antes, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-08-01").unwrap();
        assert!(antes.cuadra, "el estado previo debe cuadrar");
        assert_eq!(antes.pasivo_total, 106_000.0);

        // Retiro total del socio en agosto.
        ctx.liq
            .ejecutar_retiro_socio(
                &ctx.banco,
                DatosRetiroSocio { socio_id: socio_id.clone(), fecha: "2026-08-10".into() },
            )
            .unwrap();

        // Se le pagó capital $100.000 + ganancia $4.800 = $104.800; queda en caja lo retenido
        // en los fondos ($1.200). El Balance sigue cuadrando (mes tranquilo, septiembre).
        let (despues, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-09-01").unwrap();
        assert!(
            despues.cuadra,
            "el retiro debe dejar el Balance cuadrado: activo {} pasivo {}",
            despues.activo_total, despues.pasivo_total
        );
        assert_eq!(despues.efectivo, 1_200.0);
        assert_eq!(despues.activo_total, 1_200.0);
        assert_eq!(despues.pasivo_total, 1_200.0);
    }

    /// Retiro con deuda: el socio debe más de lo que valen sus acciones. No se le paga; el
    /// faltante se absorbe del Fondo de Incobrables y el Balance sigue cuadrado.
    #[test]
    fn el_retiro_con_deuda_absorbe_del_fondo_y_cuadra() {
        use crate::modules::creditos::domain::{
            DestinoCredito, FiadorSolicitud, NuevoDesembolso,
        };

        let ctx = montar();
        let deudor = socio_ana(&ctx);
        // Un fiador (socio B) con acciones para respaldar el crédito.
        let fiador = ctx
            .socios
            .registrar(
                &ctx.banco,
                DatosSocio {
                    cedula: "222".into(),
                    nombres: "Beto".into(),
                    apellidos: "Ruiz".into(),
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
            .id;

        // Cada uno compra 20 acciones a $10.000 = $200.000 de capital.
        for s in [&deudor, &fiador] {
            ctx.acciones
                .registrar_compra(
                    &ctx.banco,
                    NuevaCompra { socio_id: s.clone(), fecha: "2025-01-05".into(), monto: 200_000.0 },
                )
                .unwrap();
        }

        // Se le desembolsa al deudor $800.000 (RN-03: hasta 5× sus acciones = $1.000.000),
        // más de lo que valen sus $200.000 en acciones. CON egreso; su deuda queda en $800.000.
        ctx.creditos
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: None,
                    socio_id: deudor.clone(),
                    monto: 800_000.0,
                    plazo_cuotas: 6,
                    destino: DestinoCredito::Productivo,
                    fiadores: vec![FiadorSolicitud { cedula: "222".into(), acciones_comprometidas: 16.0 }],
                    fecha: Some("2025-02-01".into()),
                },
            )
            .unwrap();

        // El estado previo cuadra: efectivo (400.000 − 800.000 = −400.000) + cartera 800.000 =
        // 400.000 = capital 400.000.
        let (antes, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-08-01").unwrap();
        assert!(
            antes.cuadra,
            "estado previo debe cuadrar: activo {} pasivo {}",
            antes.activo_total, antes.pasivo_total
        );

        // El deudor se retira debiendo: capital $200.000 − deuda $800.000 = −$600.000. No se
        // le paga; el faltante se absorbe del Fondo de Incobrables (que queda en −$600.000,
        // reserva insuficiente) y el Balance sigue cuadrando.
        ctx.liq
            .ejecutar_retiro_socio(
                &ctx.banco,
                DatosRetiroSocio { socio_id: deudor.clone(), fecha: "2026-08-10".into() },
            )
            .unwrap();

        let (despues, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-09-01").unwrap();
        assert!(
            despues.cuadra,
            "el retiro con deuda debe cuadrar: activo {} pasivo {}",
            despues.activo_total, despues.pasivo_total
        );
        // No hubo pago: la caja no se movió por el retiro (sigue en −400.000). Sólo queda el
        // capital del fiador (200.000) contra el fondo en déficit (−600.000).
        assert_eq!(despues.efectivo, -400_000.0);
        assert_eq!(despues.pasivo_total, -400_000.0);

        // El socio quedó marcado retirado con deuda de $600.000.
        let socio = ctx.socios.obtener(&ctx.banco, &deudor).unwrap();
        assert_eq!(socio.saldo_incobrable, 600_000.0);
    }

    /// RF-28 (venta parcial ANTES del año): el socio vende 5 de sus 10 acciones compradas
    /// hace un mes. Recibe sólo el capital ($50.000); la ganancia devengada de esas acciones
    /// se LIBERA al colectivo (no se paga). El socio sigue activo y el Balance cuadra.
    #[test]
    fn la_venta_parcial_antes_del_anio_paga_solo_capital_y_cuadra() {
        let ctx = montar();
        let socio_id = socio_ana(&ctx);

        // El Banko arrancó en julio 2026, así que la primera compra (100% del socio) cae en
        // el período de gracia de RN-02.
        ctx.db
            .open_banco_db(&ctx.banco)
            .unwrap()
            .execute(
                "UPDATE configuracion SET fecha_creacion = '2026-07-01' WHERE id = ?1",
                [ctx.banco.as_str()],
            )
            .unwrap();

        // Compra 10 acciones a $10.000 en julio 2026 (NO cumplen el año en agosto).
        ctx.acciones
            .registrar_compra(
                &ctx.banco,
                NuevaCompra { socio_id: socio_id.clone(), fecha: "2026-07-05".into(), monto: 100_000.0 },
            )
            .unwrap();
        // Interés del mes y cierre de julio: devenga ganancia al lote.
        ctx.caja
            .registrar_asiento_de_modulo(
                &ctx.banco,
                CodigoOperacion::InteresOrdinario,
                "2026-07-15".into(),
                6_000.0,
                "Interés".into(),
                Some(socio_id.clone()),
                None,
            )
            .unwrap();
        ctx.cierre.ejecutar_cierre(&ctx.banco, "2026-07-01", "Tester").unwrap();

        let r = ctx
            .liq
            .ejecutar_liquidacion_parcial(
                &ctx.banco,
                DatosLiquidacionParcial {
                    socio_id: socio_id.clone(),
                    cantidad: 5,
                    fecha: "2026-08-10".into(),
                    destino: DestinoLiquidacionParcial::Efectivo,
                },
            )
            .unwrap();

        assert_eq!(r.capital, 50_000.0);
        assert_eq!(r.ganancia, 0.0, "antes del año la ganancia se libera, no se paga");
        assert_eq!(r.valor, 50_000.0);
        assert_eq!(r.acciones_liquidadas, 5);
        assert_eq!(r.acciones_restantes, 5, "el socio sigue activo con 5 acciones");

        let (despues, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-09-01").unwrap();
        assert!(
            despues.cuadra,
            "la venta parcial debe dejar el Balance cuadrado: activo {} pasivo {}",
            despues.activo_total, despues.pasivo_total
        );
        // Efectivo 106.000 − 50.000 = 56.000; capital 50.000 + fondos 1.200 + ganancias
        // (4.800, todas siguen en el Pasivo: 2.400 liberadas + 2.400 pendientes) = 56.000.
        assert_eq!(despues.efectivo, 56_000.0);
        assert_eq!(despues.pasivo_total, 56_000.0);
    }

    /// RF-28 (venta parcial DESPUÉS del año): el socio vende 5 de sus 10 acciones compradas
    /// hace más de un año. Recibe el capital ($50.000) MÁS la ganancia proporcional de esas
    /// acciones (5/10 de $4.800 = $2.400), porque ya cumplieron el año (RN-10). Cuadra.
    #[test]
    fn la_venta_parcial_despues_del_anio_paga_capital_mas_ganancia_y_cuadra() {
        let ctx = montar();
        let socio_id = socio_ana(&ctx);

        ctx.acciones
            .registrar_compra(
                &ctx.banco,
                NuevaCompra { socio_id: socio_id.clone(), fecha: "2025-01-05".into(), monto: 100_000.0 },
            )
            .unwrap();
        ctx.caja
            .registrar_asiento_de_modulo(
                &ctx.banco,
                CodigoOperacion::InteresOrdinario,
                "2026-07-15".into(),
                6_000.0,
                "Interés".into(),
                Some(socio_id.clone()),
                None,
            )
            .unwrap();
        ctx.cierre.ejecutar_cierre(&ctx.banco, "2026-07-01", "Tester").unwrap();

        let r = ctx
            .liq
            .ejecutar_liquidacion_parcial(
                &ctx.banco,
                DatosLiquidacionParcial {
                    socio_id: socio_id.clone(),
                    cantidad: 5,
                    fecha: "2026-08-10".into(),
                    destino: DestinoLiquidacionParcial::Efectivo,
                },
            )
            .unwrap();

        assert_eq!(r.capital, 50_000.0);
        assert_eq!(r.ganancia, 2_400.0, "5/10 de la ganancia devengada del lote");
        assert_eq!(r.valor, 52_400.0);
        assert_eq!(r.acciones_restantes, 5);

        let (despues, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-09-01").unwrap();
        assert!(
            despues.cuadra,
            "activo {} pasivo {}",
            despues.activo_total, despues.pasivo_total
        );
        // Efectivo 106.000 − 52.400 = 53.600; capital 50.000 + fondos 1.200 + ganancia
        // pendiente restante 2.400 = 53.600.
        assert_eq!(despues.efectivo, 53_600.0);
        assert_eq!(despues.pasivo_total, 53_600.0);

        // Quedó asentado en el Libro como Liquidación de Acciones (LC, egreso).
        let lc: f64 = ctx
            .db
            .open_banco_db(&ctx.banco)
            .unwrap()
            .query_row(
                "SELECT COALESCE(SUM(egreso), 0) FROM movimiento_libro WHERE codigo = 'LC'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(lc, 52_400.0, "la venta parcial en efectivo se asienta como LC");
    }

    /// RF-28 (venta parcial → abono a deuda): el socio vende acciones y aplica su valor a la
    /// deuda en vez de recibirlo. La cartera baja por el abono, no sale dinero de caja y el
    /// Balance sigue cuadrado. El socio sigue activo.
    #[test]
    fn la_venta_parcial_con_abono_a_deuda_cuadra() {
        use crate::modules::creditos::domain::{DestinoCredito, FiadorSolicitud, NuevoDesembolso};

        let ctx = montar();
        let deudor = socio_ana(&ctx);
        let fiador = ctx
            .socios
            .registrar(
                &ctx.banco,
                DatosSocio {
                    cedula: "222".into(),
                    nombres: "Beto".into(),
                    apellidos: "Ruiz".into(),
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
            .id;

        for s in [&deudor, &fiador] {
            ctx.acciones
                .registrar_compra(
                    &ctx.banco,
                    NuevaCompra { socio_id: s.clone(), fecha: "2025-01-05".into(), monto: 200_000.0 },
                )
                .unwrap();
        }

        // Crédito de $100.000 al deudor (dentro de 5× sus acciones = $1.000.000).
        ctx.creditos
            .registrar_desembolso(
                &ctx.banco,
                NuevoDesembolso {
                    solicitud_id: None,
                    socio_id: deudor.clone(),
                    monto: 100_000.0,
                    plazo_cuotas: 6,
                    destino: DestinoCredito::Productivo,
                    fiadores: vec![FiadorSolicitud { cedula: "222".into(), acciones_comprometidas: 4.0 }],
                    fecha: Some("2025-02-01".into()),
                },
            )
            .unwrap();

        let (antes, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-08-01").unwrap();
        assert!(antes.cuadra);

        let r = ctx
            .liq
            .ejecutar_liquidacion_parcial(
                &ctx.banco,
                DatosLiquidacionParcial {
                    socio_id: deudor.clone(),
                    cantidad: 5,
                    fecha: "2026-08-10".into(),
                    destino: DestinoLiquidacionParcial::AbonoDeuda,
                },
            )
            .unwrap();

        // 5 × $10.000 = $50.000 abonados a la deuda de $100.000; no hay excedente en efectivo.
        assert_eq!(r.capital, 50_000.0);
        assert_eq!(r.abonado_deuda, 50_000.0);
        assert_eq!(r.pagado_efectivo, 0.0);
        assert_eq!(r.nueva_deuda, 50_000.0);
        assert_eq!(r.acciones_restantes, 15, "sigue activo con 15 acciones");

        let (despues, _, _) = ctx.cierre.previsualizar_cuadre(&ctx.banco, "2026-09-01").unwrap();
        assert!(
            despues.cuadra,
            "el abono a deuda debe cuadrar: activo {} pasivo {}",
            despues.activo_total, despues.pasivo_total
        );
        // La caja no se movió (efectivo 300.000); la cartera bajó a 50.000; capital 350.000.
        assert_eq!(despues.efectivo, 300_000.0);
        assert_eq!(despues.cartera, 50_000.0);
        assert_eq!(despues.pasivo_total, 350_000.0);

        // Quedan las dos caras asentadas en el Libro: LC (egreso por el valor de las
        // acciones) y PC (ingreso por el abono al crédito). Su neto en caja es cero.
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        let lc: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(egreso), 0) FROM movimiento_libro WHERE codigo = 'LC'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let pc: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(ingreso), 0) FROM movimiento_libro WHERE codigo = 'PC'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(lc, 50_000.0, "el valor de las acciones se asienta como LC");
        assert_eq!(pc, 50_000.0, "el abono al crédito se asienta como PC");
    }
}
