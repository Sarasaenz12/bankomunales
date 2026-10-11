use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::core::atomicidad::{atomico, AtomicidadPort};
use crate::core::error::AppError;
use crate::core::format::pesos;
use crate::modules::acciones::domain::{calcular_ppcfc, meses_entre, MESES_PPCFC};
use crate::modules::auditoria::domain::{tipo_accion, AuditoriaPort, EntradaAuditoria};

use super::domain::{
    calcular_reparto, colocacion_pct, fin_de_mes, ingresos_repartibles, mes_anterior,
    otros_egresos, sumar_meses, valor_por_accion, BienesCierrePort, CarteraPort, CierreMes,
    DesgloseRepartoMes,
    CierresPort, CohorteSocio, CuadreMes, DetalleCierre, Devengo, FondosPort, GuardarCierre,
    InformeGestionMes, LibroCierrePort, LibroContablePort, LotesPort, PagoGananciaRegistrado,
    RepartoAnual, RepartoGuardado, RepartoMes, RepartoPort, ResultadoCierre, SocioPagoGanancia,
    SociosCierrePort,
};

/// Meses que devenga una cohorte antes de cobrar (RN-10): el mes de compra y los 11
/// siguientes. Sumar sus ganancias por acción da el valor total del año del Excel.
const MESES_VENTANA_ANUAL: u32 = 11;

/// Capa de Aplicación/Servicios del módulo de Cierre de Mes.
/// Orquesta los casos de uso de Cuadre y Cierre (RF-91 a RF-97) y, como efecto del
/// cierre, el devengo mensual de las ganancias (RF-39 a RF-42, D-12/D-13).
pub struct CierreService {
    cierres: Arc<dyn CierresPort>,
    reparto: Arc<dyn RepartoPort>,
    libro: Arc<dyn LibroCierrePort>,
    cartera: Arc<dyn CarteraPort>,
    bienes: Arc<dyn BienesCierrePort>,
    fondos: Arc<dyn FondosPort>,
    lotes: Arc<dyn LotesPort>,
    socios: Arc<dyn SociosCierrePort>,
    libro_contable: Arc<dyn LibroContablePort>,
    auditoria: Arc<dyn AuditoriaPort>,
    atomicidad: Arc<dyn AtomicidadPort>,
}

impl CierreService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cierres: Arc<dyn CierresPort>,
        reparto: Arc<dyn RepartoPort>,
        libro: Arc<dyn LibroCierrePort>,
        cartera: Arc<dyn CarteraPort>,
        bienes: Arc<dyn BienesCierrePort>,
        fondos: Arc<dyn FondosPort>,
        lotes: Arc<dyn LotesPort>,
        socios: Arc<dyn SociosCierrePort>,
        libro_contable: Arc<dyn LibroContablePort>,
        auditoria: Arc<dyn AuditoriaPort>,
        atomicidad: Arc<dyn AtomicidadPort>,
    ) -> Self {
        Self {
            cierres,
            reparto,
            libro,
            cartera,
            bienes,
            fondos,
            lotes,
            socios,
            libro_contable,
            auditoria,
            atomicidad,
        }
    }

    /// RF-91 a RF-93 + RF-40 a RF-42: calcula el cuadre, el informe de gestión y el
    /// reparto del mes **sin tocar datos definitivos**. Es el proceso de cuadre que
    /// el Verificador puede repetir cuantas veces quiera antes de cerrar.
    pub fn previsualizar_cuadre(
        &self,
        banco_id: &str,
        mes: &str,
    ) -> Result<(CuadreMes, InformeGestionMes, RepartoMes), AppError> {
        let mes = validar_mes(mes)?;
        let fin = fin_de_mes(&mes)
            .ok_or_else(|| AppError::OperacionNoPermitida("Mes inválido".into()))?;

        let efectivo = self.libro.saldo_caja_hasta(banco_id, &fin)?;
        let cartera = self.cartera.saldo_cartera_hasta(banco_id, &fin)?;
        let bienes = self.bienes.valor_total_hasta(banco_id, &fin)?;
        let acciones_activas = self.lotes.acciones_activas(banco_id)?;
        let capital = self.lotes.capital_en_acciones(banco_id)?;
        let ganancias_pasivo = self.reparto.ganancias_no_repartidas(banco_id)?;
        let fondos = self.fondos.parametros(banco_id)?;

        // El resultado repartible del mes (RF-40..42): sus retenciones a los fondos y el
        // balance neto. Se calcula antes del Pasivo porque forma parte de él.
        let movimientos = self.libro.movimientos_del_mes(banco_id, &mes)?;
        let repartibles = ingresos_repartibles(&movimientos);
        let egresos = otros_egresos(&movimientos);
        let (fondo_gastos, fondo_incobrables, balance_neto) =
            calcular_reparto(repartibles, egresos, &fondos, capital);

        let activo_total = efectivo + cartera + bienes;
        // D-20 (c): Activo = Pasivo + Patrimonio. Los bienes NO van al Pasivo: comprar un
        // bien propio es una permuta dentro del Activo (baja la caja con un egreso CAF,
        // sube el activo fijo). El Pasivo = capital en acciones + los dos fondos +
        // ganancias no repartidas + el RESULTADO del mes aún sin distribuir (retenciones a
        // los fondos + balance neto). Sin ese resultado, el ingreso repartible del mes
        // (OR/MO/OI, ya sumado al efectivo) dejaría el Balance descuadrado justo por esa
        // ganancia sin repartir.
        let pasivo_total = capital
            + fondos.saldo_fondo_gastos
            + fondos.saldo_fondo_incobrables
            + ganancias_pasivo
            + fondo_gastos
            + fondo_incobrables
            + balance_neto;

        let cuadre = CuadreMes {
            mes: mes.clone(),
            efectivo,
            cartera,
            bienes,
            activo_total,
            pasivo_total,
            cuadra: (activo_total - pasivo_total).abs() < 0.01,
        };

        let informe = InformeGestionMes {
            mes: mes.clone(),
            socios_activos: self.socios.socios_activos(banco_id)?,
            creditos_vigentes: self.cartera.creditos_vigentes_hasta(banco_id, &fin)?,
            monto_cartera: cartera,
            efectivo_disponible: efectivo,
            rendimiento_estimado: valor_por_accion(balance_neto, acciones_activas),
        };

        let reparto = RepartoMes {
            ingresos_repartibles: repartibles,
            otros_egresos: egresos,
            fondo_gastos,
            fondo_incobrables,
            balance_neto,
            acciones_activas,
            valor_por_accion: valor_por_accion(balance_neto, acciones_activas),
        };

        Ok((cuadre, informe, reparto))
    }

    /// RF-40/RF-41: desglose del Reparto de Ganancias del mes para la pantalla — abre los
    /// ingresos repartibles en OR/MO/OI, el balance bruto y neto, el rendimiento por acción
    /// del mes y el acumulado de los últimos 12 meses (Σ de la ganancia por acción de los
    /// cierres de la ventana). Es sólo lectura: se recalcula de los movimientos del mes.
    pub fn desglose_reparto_mes(
        &self,
        banco_id: &str,
        mes: &str,
    ) -> Result<DesgloseRepartoMes, AppError> {
        let mes = validar_mes(mes)?;
        let movimientos = self.libro.movimientos_del_mes(banco_id, &mes)?;
        let suma_codigo = |cod: &str| -> f64 {
            movimientos
                .iter()
                .filter(|f| f.codigo == cod)
                .map(|f| f.ingreso)
                .sum()
        };
        let intereses_ordinarios = suma_codigo("OR");
        let intereses_mora = suma_codigo("MO");
        let otros_ingresos = suma_codigo("OI");
        let repartibles = ingresos_repartibles(&movimientos);
        let egresos = otros_egresos(&movimientos);

        let capital = self.lotes.capital_en_acciones(banco_id)?;
        let fondos = self.fondos.parametros(banco_id)?;
        let (fondo_gastos, fondo_incobrables, balance_neto) =
            calcular_reparto(repartibles, egresos, &fondos, capital);
        let acciones = self.lotes.acciones_activas(banco_id)?;

        // Ventana de 12 meses hacia atrás (incluido éste): Σ de la ganancia por acción de
        // cada cierre sellado en el rango.
        let mut desde = mes.clone();
        for _ in 0..11 {
            match mes_anterior(&desde) {
                Some(m) => desde = m,
                None => break,
            }
        }
        let filas = self.reparto.filas_reparto_entre(banco_id, &desde, &mes)?;
        let rendimiento_acumulado_12m = filas.iter().map(|f| f.valor_por_accion).sum();

        let cerrado_el = self
            .cierres
            .listar(banco_id)?
            .into_iter()
            .find(|c| c.mes == mes)
            .map(|c| c.fecha_cierre);

        Ok(DesgloseRepartoMes {
            mes,
            intereses_ordinarios,
            intereses_mora,
            otros_ingresos,
            total_ingresos_repartibles: repartibles,
            otros_egresos: egresos,
            balance_bruto: repartibles - egresos,
            fondo_incobrables,
            fondo_gastos,
            pct_fondo_incobrables: fondos.pct_fondo_incobrables,
            pct_fondo_gastos: fondos.pct_fondo_gastos,
            balance_neto,
            acciones_activas: acciones,
            rendimiento_mes: valor_por_accion(balance_neto, acciones),
            rendimiento_acumulado_12m,
            cerrado_el,
        })
    }

    /// RF-94 a RF-97 + RF-39: ejecuta el Cierre de Mes.
    ///
    /// Todo persiste en una sola transacción (ver `CierresPort::guardar_cierre`):
    /// si algo falla a la mitad, la base queda en el mes abierto, no a medio cerrar.
    pub fn ejecutar_cierre(
        &self,
        banco_id: &str,
        mes: &str,
        nombre_quien_realiza: &str,
    ) -> Result<ResultadoCierre, AppError> {
        let mes = validar_mes(mes)?;
        if nombre_quien_realiza.trim().is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar quién ejecuta el cierre".into(),
            ));
        }

        // RF-94: no se puede cerrar un mes si el anterior sigue abierto. La única
        // excepción es el primer cierre del Bankomunal: antes no hay meses que cerrar.
        if !self.cierres.listar(banco_id)?.is_empty() {
            let anterior = mes_anterior(&mes)
                .ok_or_else(|| AppError::OperacionNoPermitida(format!("Mes inválido: {mes}")))?;
            if !self.cierres.existe_mes(banco_id, &anterior)? {
                return Err(AppError::OperacionNoPermitida(format!(
                    "No se puede cerrar {mes} porque el mes anterior ({anterior}) sigue \
                     abierto. Cierre primero el mes anterior."
                )));
            }
        }
        if self.cierres.existe_mes(banco_id, &mes)? {
            return Err(AppError::OperacionNoPermitida(format!(
                "El mes {mes} ya está cerrado"
            )));
        }

        // Los mismos cálculos de la previsualización, para que lo que se muestra sea
        // exactamente lo que se guarda (DRY).
        let fin = fin_de_mes(&mes)
            .ok_or_else(|| AppError::OperacionNoPermitida("Mes inválido".into()))?;
        let efectivo = self.libro.saldo_caja_hasta(banco_id, &fin)?;
        let cartera = self.cartera.saldo_cartera_hasta(banco_id, &fin)?;
        let bienes = self.bienes.valor_total_hasta(banco_id, &fin)?;
        let acciones_activas = self.lotes.acciones_activas(banco_id)?;
        let capital = self.lotes.capital_en_acciones(banco_id)?;
        let ganancias_pasivo = self.reparto.ganancias_no_repartidas(banco_id)?;
        let fondos = self.fondos.parametros(banco_id)?;

        let movimientos = self.libro.movimientos_del_mes(banco_id, &mes)?;
        let repartibles = ingresos_repartibles(&movimientos);
        let egresos = otros_egresos(&movimientos);
        let (fondo_gastos, fondo_incobrables, balance_neto) =
            calcular_reparto(repartibles, egresos, &fondos, capital);
        let valor = valor_por_accion(balance_neto, acciones_activas);

        let activo_total = efectivo + cartera + bienes;
        // D-20 (c): el Pasivo incluye el resultado del mes aún sin distribuir (retenciones
        // a los fondos + balance neto); ver `previsualizar_cuadre`.
        let pasivo_total = capital
            + fondos.saldo_fondo_gastos
            + fondos.saldo_fondo_incobrables
            + ganancias_pasivo
            + fondo_gastos
            + fondo_incobrables
            + balance_neto;
        let cuadra = (activo_total - pasivo_total).abs() < 0.01;
        if !cuadra {
            return Err(AppError::OperacionNoPermitida(format!(
                "No se puede cerrar {mes}: el balance no cuadra. Activo: ${}, pasivo: ${}, diferencia: ${}. Corrija los datos y vuelva a intentar.",
                pesos(activo_total),
                pesos(pasivo_total),
                pesos((activo_total - pasivo_total).abs()),
            )));
        }

        // RF-96: colocación del mes y PPCFC (promedio de los 3 últimos cierres,
        // incluido éste). Sin 3 cierres el PPCFC no existe aún (D-02): se guarda 0 y
        // el módulo de Acciones sigue reportándolo como pendiente.
        let colocacion = colocacion_pct(cartera, efectivo);
        let anteriores = self.cierres.colocaciones_recientes(banco_id, MESES_PPCFC - 1)?;
        let mut serie = vec![colocacion];
        serie.extend_from_slice(&anteriores);
        let ppcfc = calcular_ppcfc(&serie).unwrap_or(0.0);

        // RF-39 (D-12): el devengo mensual de cada lote activo, con la fotografía
        // del mes (acciones y valor) para que el histórico no cambie después.
        let lotes = self.lotes.lotes_activos(banco_id)?;
        let mut devengos: Vec<Devengo> = lotes
            .iter()
            .map(|l| Devengo {
                lote_acciones_id: l.id.clone(),
                socio_id: l.socio_id.clone(),
                acciones: l.cantidad,
                valor_por_accion: valor,
                monto: l.cantidad as f64 * valor,
            })
            .collect();

        // El valor por acción se redondea a peso entero, así que Σ(acciones × valor) puede
        // quedar por debajo del balance neto por unos pesos. Ese residuo se asigna al
        // último devengo para que Σ devengos == balance_neto: si no, las "ganancias no
        // repartidas" del Pasivo quedarían cortas y el Balance descuadraría un poco más en
        // cada cierre. Es el mismo criterio de "la última cuota absorbe el redondeo".
        let suma_devengos: f64 = devengos.iter().map(|d| d.monto).sum();
        if let Some(ultimo) = devengos.last_mut() {
            ultimo.monto += balance_neto - suma_devengos;
        }

        let fecha_cierre = ahora();
        let cierre_id = Uuid::new_v4().to_string();
        let guardar = GuardarCierre {
            cierre: CierreMes {
                id: cierre_id.clone(),
                mes: mes.clone(),
                efectivo,
                cartera,
                bienes,
                activo_total,
                pasivo_total,
                cuadra,
                colocacion_pct: colocacion,
                ppcfc_pct: ppcfc,
                fecha_cierre: fecha_cierre.clone(),
            },
            reparto: RepartoGuardado {
                id: Uuid::new_v4().to_string(),
                cierre_mes_id: cierre_id.clone(),
                mes: mes.clone(),
                total_ingresos_repartibles: repartibles,
                otros_egresos: egresos,
                fondo_incobrables,
                fondo_gastos,
                balance_neto,
                acciones_activas,
                valor_por_accion: valor,
            },
            devengos: devengos.clone(),
            delta_fondo_gastos: fondo_gastos,
            delta_fondo_incobrables: fondo_incobrables,
        };
        self.cierres.guardar_cierre(banco_id, &guardar)?;

        self.auditoria.registrar(
            banco_id,
            &EntradaAuditoria {
                id: Uuid::new_v4().to_string(),
                fecha: fecha_cierre.clone(),
                nombre_quien_realiza: nombre_quien_realiza.trim().to_string(),
                entidad_afectada: "cierre_mes".into(),
                campo_modificado: Some(mes.clone()),
                valor_anterior: Some("abierto".into()),
                valor_nuevo: Some(format!("balance neto {balance_neto:.2}")),
                motivo: Some("Cierre de mes".into()),
                tipo_accion: tipo_accion::CIERRE_MES.into(),
            },
        )?;

        let reparto = RepartoMes {
            ingresos_repartibles: repartibles,
            otros_egresos: egresos,
            fondo_gastos,
            fondo_incobrables,
            balance_neto,
            acciones_activas,
            valor_por_accion: valor,
        };

        Ok(ResultadoCierre { cierre: guardar.cierre, reparto, devengos })
    }

    /// Historial de cierres, del más reciente al más antiguo.
    pub fn listar_cierres(&self, banco_id: &str) -> Result<Vec<CierreMes>, AppError> {
        self.cierres.listar(banco_id)
    }

    /// Detalle de un cierre: su balance, su reparto y el devengo mes a mes.
    pub fn detalle_cierre(&self, banco_id: &str, id: &str) -> Result<DetalleCierre, AppError> {
        let cierre = self
            .cierres
            .obtener(banco_id, id)?
            .ok_or(AppError::OperacionNoPermitida("Cierre no encontrado".into()))?;
        let reparto = self
            .reparto
            .obtener_de_cierre(banco_id, &cierre.id)?
            .ok_or(AppError::OperacionNoPermitida(
                "El cierre no tiene reparto de ganancias".into(),
            ))?;
        let reparto_id = self
            .reparto
            .id_de_cierre(banco_id, &cierre.id)?
            .ok_or(AppError::OperacionNoPermitida(
                "El cierre no tiene reparto de ganancias".into(),
            ))?;
        let devengos = self.reparto.devengos_de_reparto(banco_id, &reparto_id)?;
        Ok(DetalleCierre { cierre, reparto, devengos })
    }

    /// RF-39 a RF-42 (D-12): la "hoja" de reparto anual de una cohorte (mes de compra),
    /// tal como el cliente la lleva en su Excel: la tabla mes a mes de la ganancia por
    /// acción durante los 12 meses de la ventana, su acumulado, y el listado de pago por
    /// socio (acciones de ese mes × ganancia devengada). Es sólo lectura.
    pub fn reparto_anual(&self, banco_id: &str, mes: &str) -> Result<RepartoAnual, AppError> {
        let mes = validar_mes(mes)?;
        let fin = sumar_meses(&mes, MESES_VENTANA_ANUAL)
            .ok_or_else(|| AppError::OperacionNoPermitida("Mes inválido".into()))?;

        let meses = self.reparto.filas_reparto_entre(banco_id, &mes, &fin)?;
        let pagos = self.reparto.pagos_de_cohorte(banco_id, &mes)?;

        let acumulado_por_accion = meses.iter().map(|f| f.valor_por_accion).sum();
        let acciones_cohorte = pagos.iter().map(|p| p.acciones).sum();
        let total_a_pagar = pagos.iter().map(|p| p.monto_acumulado).sum();

        Ok(RepartoAnual {
            mes_cohorte: mes,
            mes_fin_ventana: fin,
            acciones_cohorte,
            acumulado_por_accion,
            total_a_pagar,
            meses,
            pagos,
        })
    }

    /// RF-42 (RN-10): resumen de pagos de ganancias a la `fecha` dada. Por cada socio,
    /// cuántas de sus acciones **cumplen el año**, el valor de ganancia por acción, la
    /// ganancia a pagar, el estado (PAGADA/PENDIENTE/PROXIMO) y el detalle de sus
    /// cohortes para el historial. Es sólo lectura.
    pub fn resumen_pagos_ganancias(
        &self,
        banco_id: &str,
        fecha: &str,
    ) -> Result<Vec<SocioPagoGanancia>, AppError> {
        let fecha = fecha.trim();
        if fecha.is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar la fecha del pago".into(),
            ));
        }
        let lotes = self.reparto.resumen_por_lote(banco_id)?;

        // Acumulador por socio: además de lo visible, guarda el pendiente de los lotes
        // que cumplen para decidir el estado (PAGADA vs PENDIENTE).
        struct Acc {
            fila: SocioPagoGanancia,
            pendiente_cumplen: f64,
        }
        let mut accs: Vec<Acc> = Vec::new();

        for lote in &lotes {
            let meses = meses_entre(&lote.mes_compra, fecha).unwrap_or(0);
            let cumple = meses >= 12;
            let cohorte = CohorteSocio {
                mes_compra: lote.mes_compra.clone(),
                cantidad: lote.cantidad,
                meses_transcurridos: meses,
                cumple_anio: cumple,
                ganancia_acumulada: lote.accrued_total,
            };
            let acc = match accs.iter_mut().find(|a| a.fila.socio_id == lote.socio_id) {
                Some(a) => a,
                None => {
                    accs.push(Acc {
                        fila: SocioPagoGanancia {
                            socio_id: lote.socio_id.clone(),
                            acciones_cumplen: 0,
                            valor_ganancia_accion: 0.0,
                            ganancia_a_pagar: 0.0,
                            estado: String::new(),
                            cohortes: Vec::new(),
                        },
                        pendiente_cumplen: 0.0,
                    });
                    accs.last_mut().unwrap()
                }
            };
            acc.fila.cohortes.push(cohorte);
            if cumple {
                acc.fila.acciones_cumplen += lote.cantidad;
                acc.fila.ganancia_a_pagar += lote.accrued_total;
                acc.pendiente_cumplen += lote.accrued_pendiente;
            }
        }

        let mut filas: Vec<SocioPagoGanancia> = accs
            .into_iter()
            .map(|a| {
                let mut f = a.fila;
                f.estado = if f.acciones_cumplen == 0 {
                    "PROXIMO".into()
                } else if a.pendiente_cumplen.abs() < 0.01 {
                    "PAGADA".into()
                } else {
                    "PENDIENTE".into()
                };
                f.valor_ganancia_accion = if f.acciones_cumplen > 0 {
                    f.ganancia_a_pagar / f.acciones_cumplen as f64
                } else {
                    0.0
                };
                f
            })
            .collect();
        // Primero los que hay que pagar, luego los pendientes de cumplir.
        filas.sort_by(|a, b| b.ganancia_a_pagar.total_cmp(&a.ganancia_a_pagar));
        Ok(filas)
    }

    /// RF-42 (D-12): paga la ganancia acumulada a cada lote que ya cumplió un año
    /// desde su compra (RN-10). Idempotente: un lote pagado no vuelve a pagarse.
    ///
    /// El pago se asienta en el Libro como UR (egreso), uno por socio con el total
    /// de sus lotes que cumplieron.
    pub fn pagar_ganancias_cumplidas(
        &self,
        banco_id: &str,
        fecha: &str,
    ) -> Result<Vec<PagoGananciaRegistrado>, AppError> {
        atomico(&*self.atomicidad, banco_id, || {
            self.pagar_lotes_que_cumplen(banco_id, fecha, None)
        })
    }

    /// RF-42 (RN-10): paga la ganancia acumulada sólo de los lotes de **un** socio que
    /// cumplieron el año. Mismo asiento UR y misma idempotencia que el pago general.
    pub fn pagar_ganancias_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
        fecha: &str,
    ) -> Result<Vec<PagoGananciaRegistrado>, AppError> {
        atomico(&*self.atomicidad, banco_id, || {
            self.pagar_lotes_que_cumplen(banco_id, fecha, Some(socio_id))
        })
    }

    /// RF-28 (RN-10, D-13): al retirar un socio, salda sus devengos según el aniversario de
    /// cada lote. Los lotes que cumplieron el año se PAGAN (PAGADA: su ganancia se le
    /// entrega y sale del Pasivo); los que no, se LIBERAN (LIBERADA: quedan como ganancia
    /// colectiva, siguen en el Pasivo). Devuelve la ganancia a pagarle (Σ PAGADA). Debe
    /// llamarse ANTES de liquidar los lotes (usa los lotes todavía activos).
    pub fn liquidar_devengos_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
        fecha: &str,
    ) -> Result<f64, AppError> {
        let lotes = self.lotes.lotes_con_mes_compra(banco_id)?;
        let mut ganancia_pagada = 0.0;
        for lote in lotes.iter().filter(|l| l.socio_id == socio_id) {
            let cumple = meses_entre(&lote.mes_compra, fecha).unwrap_or(0) >= 12;
            if cumple {
                ganancia_pagada += self.reparto.pagar_lote(banco_id, &lote.id, fecha)?;
            } else {
                self.reparto.liberar_lote(banco_id, &lote.id, fecha)?;
            }
        }
        Ok(ganancia_pagada)
    }

    /// RF-28 (RN-10, D-13) — venta PARCIAL de acciones: salda los devengos de los trozos de
    /// lote recién liquidados. Cada trozo cuyo lote cumplió el año se PAGA (su ganancia se le
    /// entrega y sale del Pasivo); el que no, se LIBERA (queda como ganancia colectiva). Es la
    /// misma regla del retiro total, pero sobre los lotes puntuales que se liquidaron. Recibe
    /// `(lote_id, mes_compra)` de cada trozo y devuelve la ganancia total a pagarle.
    pub fn liquidar_devengos_de_lotes(
        &self,
        banco_id: &str,
        lotes: &[(String, String)],
        fecha: &str,
    ) -> Result<f64, AppError> {
        let mut ganancia_pagada = 0.0;
        for (lote_id, mes_compra) in lotes {
            let cumple = meses_entre(mes_compra, fecha).unwrap_or(0) >= 12;
            if cumple {
                ganancia_pagada += self.reparto.pagar_lote(banco_id, lote_id, fecha)?;
            } else {
                self.reparto.liberar_lote(banco_id, lote_id, fecha)?;
            }
        }
        Ok(ganancia_pagada)
    }

    /// Devengo PENDIENTE (aún pagable) acumulado por lote activo, para prever cuánta
    /// ganancia se pagaría en una venta parcial (RN-10) sin tocar datos. `(lote_id, pendiente)`.
    pub fn devengo_pendiente_por_lote(
        &self,
        banco_id: &str,
    ) -> Result<Vec<(String, f64)>, AppError> {
        Ok(self
            .reparto
            .resumen_por_lote(banco_id)?
            .into_iter()
            .map(|r| (r.lote_id, r.accrued_pendiente))
            .collect())
    }

    /// Ajusta el Fondo de Reserva para Incobrables (RF-35/RN-08): negativo al absorber la
    /// deuda de un socio que se retira debiendo, positivo al recuperarla (RF-74).
    pub fn ajustar_fondo_incobrables(&self, banco_id: &str, delta: f64) -> Result<(), AppError> {
        self.fondos.ajustar_incobrables(banco_id, delta)
    }

    /// Ganancia que se le pagaría a un socio si se retirara en `fecha`: sólo la devengada
    /// por los lotes que ya cumplieron el año (RN-10). Es sólo lectura (no salda nada), para
    /// que la pantalla muestre exactamente lo que el retiro pagará.
    pub fn ganancia_cumplida_de_socio(
        &self,
        banco_id: &str,
        socio_id: &str,
        fecha: &str,
    ) -> Result<f64, AppError> {
        let resumen = self.reparto.resumen_por_lote(banco_id)?;
        let total = resumen
            .iter()
            .filter(|r| {
                r.socio_id == socio_id && meses_entre(&r.mes_compra, fecha).unwrap_or(0) >= 12
            })
            .map(|r| r.accrued_pendiente)
            .sum();
        Ok(total)
    }

    /// Paga los lotes que cumplen el año (opcionalmente sólo los de un socio), asienta
    /// el UR por socio y deja registro en Auditoría (RF-42, RNF-08).
    fn pagar_lotes_que_cumplen(
        &self,
        banco_id: &str,
        fecha: &str,
        solo_socio: Option<&str>,
    ) -> Result<Vec<PagoGananciaRegistrado>, AppError> {
        let fecha = fecha.trim().to_string();
        if fecha.is_empty() {
            return Err(AppError::OperacionNoPermitida(
                "Debe indicar la fecha del pago".into(),
            ));
        }
        let lotes = self.lotes.lotes_con_mes_compra(banco_id)?;

        // Se agrupa por socio lo que se pagará, para asentar un solo UR por socio.
        let mut por_socio: Vec<(String, f64)> = Vec::new();
        for lote in &lotes {
            if let Some(s) = solo_socio {
                if lote.socio_id != s {
                    continue;
                }
            }
            let cumple = meses_entre(&lote.mes_compra, &fecha).unwrap_or(0) >= 12;
            if !cumple {
                continue;
            }
            let monto = self.reparto.pagar_lote(banco_id, &lote.id, &fecha)?;
            if monto <= 0.0 {
                continue;
            }
            match por_socio.iter_mut().find(|(id, _)| *id == lote.socio_id) {
                Some((_, total)) => *total += monto,
                None => por_socio.push((lote.socio_id.clone(), monto)),
            }
        }

        for (socio_id, monto) in &por_socio {
            self.libro_contable
                .registrar_ganancia_repartida(banco_id, &fecha, *monto, socio_id)?;
        }

        if !por_socio.is_empty() {
            let motivo = match solo_socio {
                Some(_) => format!("Pago de ganancias de un socio al {fecha}"),
                None => format!("Pago anual de ganancias al {fecha}"),
            };
            self.auditoria.registrar(
                banco_id,
                &EntradaAuditoria {
                    id: Uuid::new_v4().to_string(),
                    fecha: ahora(),
                    nombre_quien_realiza: String::new(),
                    entidad_afectada: "pago_ganancia".into(),
                    campo_modificado: Some("estado".into()),
                    valor_anterior: Some("PENDIENTE".into()),
                    valor_nuevo: Some("PAGADA".into()),
                    motivo: Some(motivo),
                    tipo_accion: tipo_accion::PAGO_GANANCIAS.into(),
                },
            )?;
        }

        Ok(por_socio
            .into_iter()
            .map(|(socio_id, monto)| PagoGananciaRegistrado { socio_id, monto })
            .collect())
    }
}

fn validar_mes(mes: &str) -> Result<String, AppError> {
    let mes = mes.trim();
    let valido = mes.len() == 10
        && mes.get(4..5) == Some("-")
        && mes.get(7..8) == Some("-")
        && fin_de_mes(mes).is_some();
    if !valido {
        return Err(AppError::OperacionNoPermitida(format!(
            "El mes «{mes}» no es válido. Use el formato YYYY-MM-01."
        )));
    }
    Ok(mes.to_string())
}

fn ahora() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::caja::application::CajaService;
    use crate::modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
    use crate::modules::caja::puente::LibroViaCaja;
    use crate::modules::cierre::data::{
        SqliteBienesCierre, SqliteCartera, SqliteCierres, SqliteFondosCierre,
        SqliteLibroCierre, SqliteLotesCierre, SqliteReparto, SqliteSociosCierre,
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn servicio() -> (CierreService, DbManager, String) {
        let dir = std::env::temp_dir().join(format!("bkn_cierre_test_{}", Uuid::new_v4()));
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
        let svc = CierreService::new(
            Arc::new(SqliteCierres::new(db.clone())),
            Arc::new(SqliteReparto::new(db.clone())),
            Arc::new(SqliteLibroCierre::new(db.clone())),
            Arc::new(SqliteCartera::new(db.clone())),
            Arc::new(SqliteBienesCierre::new(db.clone())),
            Arc::new(SqliteFondosCierre::new(db.clone())),
            Arc::new(SqliteLotesCierre::new(db.clone())),
            Arc::new(SqliteSociosCierre::new(db.clone())),
            Arc::new(LibroViaCaja::new(caja.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
            Arc::new(crate::core::atomicidad::SqliteAtomicidad::new(db.clone())),
        );
        (svc, db, banco)
    }

    /// Inserta el socio 'socio-x' (dueño de los lotes de prueba): `lote_acciones.socio_id`
    /// es clave foránea a `socio`, que la base sí exige.
    fn sembrar_socio(conn: &rusqlite::Connection) {
        conn.execute(
            "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                 telefono, celular, correo, estatus, fecha_ingreso)
             VALUES ('socio-x', '111', 'Test', 'Socio', '', '', '', '', '', 'ACTIVO', '2026-01-01')",
            [],
        )
        .unwrap();
    }

    #[test]
    fn no_cierra_ni_guarda_el_mes_si_el_balance_no_cuadra() {
        let (svc, db, banco) = servicio();
        let conn = db.open_banco_db(&banco).unwrap();
        conn.execute(
            "INSERT INTO movimiento_libro
                 (id, numero, fecha, codigo, descripcion, ingreso, egreso, saldo)
             VALUES ('movimiento-inconsistente', 1, '2026-08-15', 'CAF',
                     'Compra de bien sin registrar el activo', 0, 1000, -1000)",
            [],
        )
        .unwrap();

        let error = svc.ejecutar_cierre(&banco, "2026-08-01", "Tester").unwrap_err();
        assert!(matches!(
            error,
            AppError::OperacionNoPermitida(mensaje)
                if mensaje.contains("el balance no cuadra")
                    && mensaje.contains("diferencia: $1.000")
        ));

        let conn = db.open_banco_db(&banco).unwrap();
        let cierres: i64 = conn
            .query_row("SELECT COUNT(*) FROM cierre_mes", [], |row| row.get(0))
            .unwrap();
        let repartos: i64 = conn
            .query_row("SELECT COUNT(*) FROM reparto_ganancias", [], |row| row.get(0))
            .unwrap();
        assert_eq!(cierres, 0, "no debe quedar un cierre parcialmente guardado");
        assert_eq!(repartos, 0, "no debe devengarse ganancia si se rechaza el cierre");
    }

    /// El Balance del Mes debe cuadrar (Activo = Pasivo) cuando el mes tuvo ingreso
    /// repartible: ese ingreso ya está en el efectivo (Activo) y su reparto —retenciones a
    /// los fondos + balance neto— tiene que estar en el Pasivo. Sin sumarlo, todo mes con
    /// intereses reportaría "no cuadra".
    #[test]
    fn el_balance_del_mes_cuadra_con_ingreso_repartible() {
        let (svc, db, banco) = servicio();
        let conn = db.open_banco_db(&banco).unwrap();
        sembrar_socio(&conn);

        // Capital en acciones: un lote de 10 acciones a $10.000 = $100.000.
        conn.execute(
            "INSERT INTO lote_acciones (id, socio_id, mes_compra, fecha_compra, cantidad,
                 valor_nominal_compra, monto_pagado, liquidada)
             VALUES (?1, 'socio-x', '2026-08-01', '2026-08-05', 10, 10000, 100000, 0)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )
        .unwrap();

        // Efectivo: la venta de acciones (VC $100.000) y un interés ordinario (OR $5.000).
        let asiento = |numero: i64, codigo: &str, ingreso: f64, fecha: &str| {
            conn.execute(
                "INSERT INTO movimiento_libro
                     (id, numero, fecha, codigo, descripcion, ingreso, egreso, saldo)
                 VALUES (?1, ?2, ?3, ?4, '', ?5, 0, 0)",
                rusqlite::params![Uuid::new_v4().to_string(), numero, fecha, codigo, ingreso],
            )
            .unwrap();
        };
        asiento(1, "VC", 100_000.0, "2026-08-05");
        asiento(2, "OR", 5_000.0, "2026-08-15");

        let (cuadre, _informe, reparto) = svc.previsualizar_cuadre(&banco, "2026-08-01").unwrap();

        // Activo = $105.000 (efectivo). Pasivo = $100.000 de capital + $5.000 del reparto
        // del mes (fondos $500 + $500 + balance neto $4.000).
        assert_eq!(cuadre.efectivo, 105_000.0);
        assert_eq!(cuadre.activo_total, 105_000.0);
        assert_eq!(cuadre.pasivo_total, 105_000.0);
        assert!(cuadre.cuadra, "el mes con ingreso repartible debe cuadrar");
        assert_eq!(
            reparto.fondo_gastos + reparto.fondo_incobrables + reparto.balance_neto,
            5_000.0
        );
    }

    /// Un mes sin ingreso repartible (sólo aportes de capital) también cuadra: no hay
    /// resultado que repartir, así que el Pasivo es sólo el capital.
    #[test]
    fn el_balance_cuadra_sin_ingreso_repartible() {
        let (svc, db, banco) = servicio();
        let conn = db.open_banco_db(&banco).unwrap();
        sembrar_socio(&conn);
        conn.execute(
            "INSERT INTO lote_acciones (id, socio_id, mes_compra, fecha_compra, cantidad,
                 valor_nominal_compra, monto_pagado, liquidada)
             VALUES (?1, 'socio-x', '2026-08-01', '2026-08-05', 10, 10000, 100000, 0)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO movimiento_libro
                 (id, numero, fecha, codigo, descripcion, ingreso, egreso, saldo)
             VALUES (?1, 1, '2026-08-05', 'VC', '', 100000, 0, 0)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )
        .unwrap();

        let (cuadre, _, reparto) = svc.previsualizar_cuadre(&banco, "2026-08-01").unwrap();
        assert_eq!(cuadre.activo_total, 100_000.0);
        assert_eq!(cuadre.pasivo_total, 100_000.0);
        assert!(cuadre.cuadra);
        assert_eq!(reparto.balance_neto, 0.0);
    }

    /// Tras EJECUTAR un cierre, el mes siguiente debe seguir cuadrando aunque el balance
    /// neto no sea divisible entre las acciones: el residuo del redondeo por acción se
    /// absorbe en el devengo, de modo que las ganancias no repartidas del Pasivo igualan
    /// exactamente el resultado del mes. Con 3 acciones y balance neto $4.000 → $1.333/acc
    /// deja $1 de residuo que, sin absorber, descuadraría septiembre.
    #[test]
    fn tras_ejecutar_el_cierre_el_mes_siguiente_sigue_cuadrando() {
        let (svc, db, banco) = servicio();
        let conn = db.open_banco_db(&banco).unwrap();
        sembrar_socio(&conn);
        // 3 acciones a $10.000 = $30.000 de capital.
        conn.execute(
            "INSERT INTO lote_acciones (id, socio_id, mes_compra, fecha_compra, cantidad,
                 valor_nominal_compra, monto_pagado, liquidada)
             VALUES (?1, 'socio-x', '2026-08-01', '2026-08-05', 3, 10000, 30000, 0)",
            rusqlite::params![Uuid::new_v4().to_string()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO movimiento_libro
                 (id, numero, fecha, codigo, descripcion, ingreso, egreso, saldo)
             VALUES (?1, 1, '2026-08-05', 'VC', '', 30000, 0, 0),
                    (?2, 2, '2026-08-15', 'OR', '', 5000, 0, 0)",
            rusqlite::params![Uuid::new_v4().to_string(), Uuid::new_v4().to_string()],
        )
        .unwrap();

        // Cierra agosto: balance neto = 5.000 − 500 − 500 = 4.000; 4.000 / 3 = 1.333/acc.
        let r = svc.ejecutar_cierre(&banco, "2026-08-01", "Tester").unwrap();
        assert_eq!(r.reparto.balance_neto, 4_000.0);

        // Septiembre sin movimientos: el Activo (efectivo $35.000) no cambió, y el Pasivo
        // —capital $30.000 + fondos $1.000 + ganancias devengadas— debe igualarlo. Si el
        // residuo de $1 no se hubiera absorbido, las ganancias serían $3.999 y descuadraría.
        let (sept, _, _) = svc.previsualizar_cuadre(&banco, "2026-09-01").unwrap();
        assert_eq!(sept.activo_total, 35_000.0);
        assert_eq!(sept.pasivo_total, 35_000.0);
        assert!(sept.cuadra, "el mes siguiente al cierre debe seguir cuadrando");
    }

    /// RF-28 (RN-10, D-13): al retirar un socio, el devengo de los lotes que cumplieron el
    /// año se PAGA (sale del Pasivo) y el de los que no se LIBERA (queda colectivo, sigue en
    /// el Pasivo). Así lo pagado no queda contado dos veces en "ganancias no repartidas".
    #[test]
    fn el_retiro_paga_los_devengos_cumplidos_y_libera_los_demas() {
        let (svc, db, banco) = servicio();
        let conn = db.open_banco_db(&banco).unwrap();
        sembrar_socio(&conn);
        conn.execute(
            "INSERT INTO reparto_ganancias (id, cierre_mes_id, mes)
             VALUES ('rep-1', 'c-1', '2026-07-01')",
            [],
        )
        .unwrap();
        // Un lote que cumplió el año (compra 2025-01) y uno que no (compra 2026-06).
        conn.execute(
            "INSERT INTO lote_acciones (id, socio_id, mes_compra, fecha_compra, cantidad,
                 valor_nominal_compra, monto_pagado, liquidada) VALUES
             ('lote-viejo','socio-x','2025-01-01','2025-01-01',10,10000,100000,0),
             ('lote-nuevo','socio-x','2026-06-01','2026-06-01',5,10000,50000,0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO pago_ganancia (id, lote_acciones_id, reparto_ganancias_id, acciones,
                 valor_por_accion, monto, estado, fecha_pago) VALUES
             ('pg-viejo','lote-viejo','rep-1',10,100,1000,'PENDIENTE',NULL),
             ('pg-nuevo','lote-nuevo','rep-1',5,100,500,'PENDIENTE',NULL)",
            [],
        )
        .unwrap();

        let ganancia = svc
            .liquidar_devengos_de_socio(&banco, "socio-x", "2026-08-01")
            .unwrap();
        assert_eq!(ganancia, 1_000.0, "sólo se paga el devengo del lote que cumplió el año");

        let estado = |id: &str| -> String {
            conn.query_row("SELECT estado FROM pago_ganancia WHERE id = ?1", [id], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(estado("pg-viejo"), "PAGADA");
        assert_eq!(estado("pg-nuevo"), "LIBERADA");

        // Ganancias no repartidas del Pasivo (PENDIENTE + LIBERADA) = sólo el liberado.
        let no_repartidas: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(monto), 0) FROM pago_ganancia
                 WHERE estado IN ('PENDIENTE', 'LIBERADA')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(no_repartidas, 500.0);
    }

    /// RF-35/RN-08 y RF-74: el faltante de un retiro con deuda se absorbe del Fondo de
    /// Incobrables (delta negativo) y, al recuperarse luego, se restituye (delta positivo).
    #[test]
    fn el_fondo_de_incobrables_se_absorbe_y_se_restituye() {
        let (svc, db, banco) = servicio();
        svc.ajustar_fondo_incobrables(&banco, -5_000.0).unwrap(); // retiro con deuda
        svc.ajustar_fondo_incobrables(&banco, 2_000.0).unwrap(); // recupera parte (PDP)

        let conn = db.open_banco_db(&banco).unwrap();
        let saldo: f64 = conn
            .query_row(
                "SELECT saldo_fondo_incobrables FROM configuracion WHERE id = ?1",
                [banco.as_str()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(saldo, -3_000.0);
    }
}