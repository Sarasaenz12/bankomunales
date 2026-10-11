use std::sync::Arc;

use uuid::Uuid;

use crate::core::atomicidad::{atomico, AtomicidadPort};
use crate::core::error::AppError;
use crate::modules::auditoria::domain::{tipo_accion, AuditoriaPort, EntradaAuditoria};

use crate::modules::creditos::domain::calcular_tabla;

use super::domain::{
    aplicar_pago, cuota_saldada, dias_mora, mora_de_cuota, plazo_reamortizado, total_debido,
    redondear, AplicacionPago, CreditoDeuda, CreditoConsultaPort, Cuota, CuotaPort,
    CuotaReamortizada, LibroDeudaPort, ParametrosDeudaPort, ResultadoAbono, ResultadoPago,
};

/// Capa de Aplicación del Servicio de la Deuda (RF-63 a RF-72, Fase 1).
///
/// Orquesta el pago de una cuota: calcula la mora sobre la cuota, imputa el monto en
/// cascada (mora → interés → capital), persiste cuota + crédito en una sola transacción
/// y asienta cada concepto en el Libro por Caja (OR/PC/MO).
pub struct DeudaService {
    creditos: Arc<dyn CreditoConsultaPort>,
    cuotas: Arc<dyn CuotaPort>,
    libro: Arc<dyn LibroDeudaPort>,
    parametros: Arc<dyn ParametrosDeudaPort>,
    auditoria: Arc<dyn AuditoriaPort>,
    atomicidad: Arc<dyn AtomicidadPort>,
}

impl DeudaService {
    pub fn new(
        creditos: Arc<dyn CreditoConsultaPort>,
        cuotas: Arc<dyn CuotaPort>,
        libro: Arc<dyn LibroDeudaPort>,
        parametros: Arc<dyn ParametrosDeudaPort>,
        auditoria: Arc<dyn AuditoriaPort>,
        atomicidad: Arc<dyn AtomicidadPort>,
    ) -> Self {
        Self { creditos, cuotas, libro, parametros, auditoria, atomicidad }
    }

    /// RF-63: estado de un crédito por número, con sus cuotas, para la pantalla de pago.
    pub fn estado_credito(
        &self,
        banco_id: &str,
        numero: &str,
    ) -> Result<(CreditoDeuda, Vec<Cuota>), AppError> {
        let credito = self
            .creditos
            .buscar_por_numero(banco_id, numero.trim())?
            .ok_or_else(|| {
                AppError::OperacionNoPermitida(format!("No existe el crédito Nº {numero}"))
            })?;
        let cuotas = self.cuotas.cuotas_de_credito(banco_id, &credito.id)?;
        Ok((credito, cuotas))
    }

    /// RF-64 a RF-72: registra el pago de una cuota. `monto` se imputa en cascada
    /// mora → interés → capital y no puede exceder lo que la cuota debe hoy (RF-68).
    pub fn registrar_pago_cuota(
        &self,
        banco_id: &str,
        numero_credito: &str,
        numero_cuota: i64,
        fecha_pago: &str,
        monto: f64,
    ) -> Result<ResultadoPago, AppError> {
        atomico(&*self.atomicidad, banco_id, || {
            self.aplicar_pago_cuota(banco_id, numero_credito, numero_cuota, fecha_pago, monto)
        })
    }

    fn aplicar_pago_cuota(
        &self,
        banco_id: &str,
        numero_credito: &str,
        numero_cuota: i64,
        fecha_pago: &str,
        monto: f64,
    ) -> Result<ResultadoPago, AppError> {
        let fecha_pago = fecha_pago.trim();
        if fecha_pago.is_empty() {
            return Err(AppError::OperacionNoPermitida("Debe indicar la fecha del pago".into()));
        }
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida("El monto del pago debe ser mayor a cero".into()));
        }

        let (credito, cuotas) = self.estado_credito(banco_id, numero_credito)?;
        if credito.estatus != "VIGENTE" {
            return Err(AppError::OperacionNoPermitida(format!(
                "El crédito Nº {} no está vigente (estatus {})",
                credito.numero, credito.estatus
            )));
        }
        let cuota = cuotas
            .into_iter()
            .find(|c| c.numero == numero_cuota)
            .ok_or_else(|| {
                AppError::OperacionNoPermitida(format!(
                    "El crédito Nº {} no tiene la cuota {numero_cuota}",
                    credito.numero
                ))
            })?;
        if cuota.pagada {
            return Err(AppError::OperacionNoPermitida(format!(
                "La cuota {numero_cuota} ya está pagada"
            )));
        }

        let dias = dias_mora(&cuota.fecha_vencimiento, fecha_pago);
        let mora_debida = mora_de_cuota(cuota.valor_total, dias, self.parametros.tasa_mora(banco_id)?);
        let debido = total_debido(&cuota, mora_debida);
        if monto > debido + 0.01 {
            return Err(AppError::OperacionNoPermitida(format!(
                "El pago de {monto:.2} excede lo que debe esta cuota ({debido:.2}). Registre el \
                 excedente en la siguiente cuota."
            )));
        }

        let aplic = aplicar_pago(&cuota, mora_debida, monto);
        let saldada = cuota_saldada(&cuota, &aplic);

        // Nueva foto de la cuota tras el abono.
        let cuota_actualizada = Cuota {
            capital_pagado: redondear(cuota.capital_pagado + aplic.capital),
            interes_pagado: redondear(cuota.interes_pagado + aplic.interes),
            mora_pagada: redondear(cuota.mora_pagada + aplic.mora),
            pagada: saldada,
            fecha_pago: Some(fecha_pago.to_string()),
            dias_atraso: dias,
            ..cuota.clone()
        };

        // RF-69: el saldo del crédito baja por el capital pagado; se cancela al llegar a 0.
        let nuevo_saldo = redondear((credito.saldo_pendiente - aplic.capital).max(0.0));
        let estatus = if nuevo_saldo <= 0.01 { "PAGADO" } else { "VIGENTE" };
        let nueva_cuota_actual = if saldada { cuota.numero + 1 } else { cuota.numero };

        self.cuotas.registrar_pago(
            banco_id,
            &cuota_actualizada,
            &credito.id,
            nuevo_saldo,
            nueva_cuota_actual,
            estatus,
        )?;

        // Cada concepto entra al Libro con su propio código (ingresos a la caja).
        if aplic.mora > 0.0 {
            self.libro.registrar_mora(banco_id, fecha_pago, aplic.mora, &credito.socio_id, &credito.id)?;
        }
        if aplic.interes > 0.0 {
            self.libro.registrar_interes_ordinario(banco_id, fecha_pago, aplic.interes, &credito.socio_id, &credito.id)?;
        }
        if aplic.capital > 0.0 {
            self.libro.registrar_pago_capital(banco_id, fecha_pago, aplic.capital, &credito.socio_id, &credito.id)?;
        }

        self.auditoria.registrar(
            banco_id,
            &EntradaAuditoria {
                id: Uuid::new_v4().to_string(),
                fecha: ahora(),
                nombre_quien_realiza: String::new(),
                entidad_afectada: "cuota".into(),
                campo_modificado: Some(format!("crédito {} cuota {}", credito.numero, cuota.numero)),
                valor_anterior: Some(format!("saldo {:.2}", credito.saldo_pendiente)),
                valor_nuevo: Some(format!("saldo {nuevo_saldo:.2} (pago {:.2})", aplic.total())),
                motivo: Some(format!("Pago de cuota al {fecha_pago}")),
                tipo_accion: tipo_accion::PAGO_CUOTA.into(),
            },
        )?;

        Ok(ResultadoPago {
            aplicacion: aplic,
            mora_debida,
            dias_atraso: dias,
            cuota_saldada: saldada,
            nuevo_saldo,
            estatus_credito: estatus.to_string(),
        })
    }

    /// Abono a capital: el monto se imputa **todo a capital**, baja el saldo del crédito y
    /// las cuotas pendientes se **re-amortizan** (mismo capital de referencia por cuota, así
    /// quedan menos cuotas; el interés de cada una se recalcula sobre el saldo decreciente).
    pub fn abonar_capital(
        &self,
        banco_id: &str,
        numero_credito: &str,
        fecha: &str,
        monto: f64,
    ) -> Result<ResultadoAbono, AppError> {
        atomico(&*self.atomicidad, banco_id, || {
            self.aplicar_abono_capital(banco_id, numero_credito, fecha, monto)
        })
    }

    fn aplicar_abono_capital(
        &self,
        banco_id: &str,
        numero_credito: &str,
        fecha: &str,
        monto: f64,
    ) -> Result<ResultadoAbono, AppError> {
        let fecha = fecha.trim();
        if fecha.is_empty() {
            return Err(AppError::OperacionNoPermitida("Debe indicar la fecha del abono".into()));
        }
        if monto <= 0.0 {
            return Err(AppError::OperacionNoPermitida("El monto del abono debe ser mayor a cero".into()));
        }

        let (credito, cuotas) = self.estado_credito(banco_id, numero_credito)?;
        if credito.estatus != "VIGENTE" {
            return Err(AppError::OperacionNoPermitida(format!(
                "El crédito Nº {} no está vigente (estatus {})",
                credito.numero, credito.estatus
            )));
        }
        if monto > credito.saldo_pendiente + 0.01 {
            return Err(AppError::OperacionNoPermitida(format!(
                "El abono de {monto:.2} excede el saldo del crédito ({:.2})",
                credito.saldo_pendiente
            )));
        }

        // El abono regenera las cuotas pendientes; si la cuota en curso ya tiene un pago
        // parcial, se pide terminarla primero para no perder lo ya imputado.
        let pendientes: Vec<&Cuota> = cuotas.iter().filter(|c| !c.pagada).collect();
        if pendientes
            .iter()
            .any(|c| c.capital_pagado > 0.0 || c.interes_pagado > 0.0 || c.mora_pagada > 0.0)
        {
            return Err(AppError::OperacionNoPermitida(
                "Termine de pagar la cuota en curso antes de abonar a capital".into(),
            ));
        }
        let primera = pendientes.first().ok_or_else(|| {
            AppError::OperacionNoPermitida("El crédito no tiene cuotas pendientes".into())
        })?;
        let desde_numero = primera.numero;
        let ancla = primera.fecha_vencimiento.clone();

        let nuevo_saldo = redondear((credito.saldo_pendiente - monto).max(0.0));
        let nuevas = plan_reamortizado(&credito, nuevo_saldo, &ancla, desde_numero);
        let estatus = if nuevo_saldo <= 0.01 { "PAGADO" } else { "VIGENTE" };
        let cuotas_restantes = nuevas.len() as i64;

        self.cuotas.reamortizar(
            banco_id,
            &credito.id,
            None,
            desde_numero,
            &nuevas,
            nuevo_saldo,
            desde_numero,
            estatus,
        )?;

        // El abono entra al Libro como capital (PC): sube la caja y baja el saldo del crédito.
        self.libro
            .registrar_pago_capital(banco_id, fecha, monto, &credito.socio_id, &credito.id)?;

        self.auditoria.registrar(
            banco_id,
            &EntradaAuditoria {
                id: Uuid::new_v4().to_string(),
                fecha: ahora(),
                nombre_quien_realiza: String::new(),
                entidad_afectada: "credito".into(),
                campo_modificado: Some(format!("crédito {}", credito.numero)),
                valor_anterior: Some(format!("saldo {:.2}", credito.saldo_pendiente)),
                valor_nuevo: Some(format!("saldo {nuevo_saldo:.2} ({cuotas_restantes} cuotas)")),
                motivo: Some(format!("Abono a capital de {:.2} al {fecha}", redondear(monto))),
                tipo_accion: tipo_accion::PAGO_CUOTA.into(),
            },
        )?;

        Ok(ResultadoAbono {
            monto_capital: redondear(monto),
            nuevo_saldo,
            cuotas_restantes,
            estatus_credito: estatus.to_string(),
        })
    }

    /// Pago de una o varias cuotas. Con una cuota se mantiene el flujo clásico. Con varias,
    /// se cobra **un solo interés** (el de la cuota en curso), la mora de esa cuota si hay
    /// atraso, y el **capital de las N cuotas**; ese capital re-amortiza el saldo restante.
    pub fn pagar_cuotas(
        &self,
        banco_id: &str,
        numero_credito: &str,
        fecha: &str,
        cantidad: i64,
    ) -> Result<ResultadoPago, AppError> {
        atomico(&*self.atomicidad, banco_id, || {
            self.aplicar_pago_de_cuotas(banco_id, numero_credito, fecha, cantidad)
        })
    }

    fn aplicar_pago_de_cuotas(
        &self,
        banco_id: &str,
        numero_credito: &str,
        fecha: &str,
        cantidad: i64,
    ) -> Result<ResultadoPago, AppError> {
        let fecha = fecha.trim();
        if fecha.is_empty() {
            return Err(AppError::OperacionNoPermitida("Debe indicar la fecha del pago".into()));
        }

        let (credito, cuotas) = self.estado_credito(banco_id, numero_credito)?;
        if credito.estatus != "VIGENTE" {
            return Err(AppError::OperacionNoPermitida(format!(
                "El crédito Nº {} no está vigente (estatus {})",
                credito.numero, credito.estatus
            )));
        }
        let pendientes: Vec<Cuota> = cuotas.into_iter().filter(|c| !c.pagada).collect();
        let actual = pendientes.first().cloned().ok_or_else(|| {
            AppError::OperacionNoPermitida("El crédito no tiene cuotas pendientes".into())
        })?;

        // Una sola cuota: flujo clásico (imputa mora → interés → capital, salda la cuota y no
        // re-amortiza; el resto de la tabla ya es correcto por saldo decreciente).
        if cantidad <= 1 {
            let dias = dias_mora(&actual.fecha_vencimiento, fecha);
            let mora = mora_de_cuota(actual.valor_total, dias, self.parametros.tasa_mora(banco_id)?);
            let monto = total_debido(&actual, mora);
            return self.registrar_pago_cuota(banco_id, numero_credito, actual.numero, fecha, monto);
        }

        let n = (cantidad.min(pendientes.len() as i64)).max(1);
        let dias = dias_mora(&actual.fecha_vencimiento, fecha);
        let mora = mora_de_cuota(actual.valor_total, dias, self.parametros.tasa_mora(banco_id)?);
        let interes = actual.interes_pendiente();
        let capital_total = redondear(
            pendientes.iter().take(n as usize).map(|c| c.capital_pendiente()).sum(),
        );
        let nuevo_saldo = redondear((credito.saldo_pendiente - capital_total).max(0.0));

        // La cuota en curso queda saldada con el interés del mes y su mora.
        let actual_saldada = Cuota {
            capital_pagado: redondear(actual.capital_pagado + actual.capital_pendiente()),
            interes_pagado: redondear(actual.interes_pagado + interes),
            mora_pagada: redondear(actual.mora_pagada + mora),
            pagada: true,
            fecha_pago: Some(fecha.to_string()),
            dias_atraso: dias,
            ..actual.clone()
        };

        // Las cuotas siguientes se reemplazan por el plan re-amortizado del saldo restante;
        // la primera conserva la fecha de vencimiento de la (N+1)-ésima cuota original.
        let desde_numero = actual.numero + 1;
        let ancla = pendientes
            .get(n as usize)
            .map(|c| c.fecha_vencimiento.clone())
            .unwrap_or_else(|| actual.fecha_vencimiento.clone());
        let nuevas = plan_reamortizado(&credito, nuevo_saldo, &ancla, desde_numero);
        let estatus = if nuevo_saldo <= 0.01 { "PAGADO" } else { "VIGENTE" };

        self.cuotas.reamortizar(
            banco_id,
            &credito.id,
            Some(&actual_saldada),
            desde_numero,
            &nuevas,
            nuevo_saldo,
            desde_numero,
            estatus,
        )?;

        if mora > 0.0 {
            self.libro.registrar_mora(banco_id, fecha, mora, &credito.socio_id, &credito.id)?;
        }
        if interes > 0.0 {
            self.libro.registrar_interes_ordinario(banco_id, fecha, interes, &credito.socio_id, &credito.id)?;
        }
        if capital_total > 0.0 {
            self.libro.registrar_pago_capital(banco_id, fecha, capital_total, &credito.socio_id, &credito.id)?;
        }

        self.auditoria.registrar(
            banco_id,
            &EntradaAuditoria {
                id: Uuid::new_v4().to_string(),
                fecha: ahora(),
                nombre_quien_realiza: String::new(),
                entidad_afectada: "cuota".into(),
                campo_modificado: Some(format!("crédito {} · {n} cuotas", credito.numero)),
                valor_anterior: Some(format!("saldo {:.2}", credito.saldo_pendiente)),
                valor_nuevo: Some(format!("saldo {nuevo_saldo:.2}")),
                motivo: Some(format!("Pago de {n} cuotas al {fecha}")),
                tipo_accion: tipo_accion::PAGO_CUOTA.into(),
            },
        )?;

        Ok(ResultadoPago {
            aplicacion: AplicacionPago { mora, interes, capital: capital_total },
            mora_debida: mora,
            dias_atraso: dias,
            cuota_saldada: true,
            nuevo_saldo,
            estatus_credito: estatus.to_string(),
        })
    }
}

/// Construye el plan re-amortizado para `nuevo_saldo` con las mismas reglas que la tabla
/// original (capital igualado, interés sobre saldo decreciente), anclado en `fecha_ancla` y
/// numerado consecutivamente desde `desde_numero`.
fn plan_reamortizado(
    credito: &CreditoDeuda,
    nuevo_saldo: f64,
    fecha_ancla: &str,
    desde_numero: i64,
) -> Vec<CuotaReamortizada> {
    let count = plazo_reamortizado(nuevo_saldo, credito.capital_referencia());
    calcular_tabla(nuevo_saldo, credito.tasa, count, fecha_ancla)
        .into_iter()
        .enumerate()
        .map(|(i, c)| CuotaReamortizada {
            numero: desde_numero + i as i64,
            fecha_vencimiento: c.fecha_vencimiento,
            capital: c.capital,
            interes: c.interes,
            valor_total: c.valor_total,
        })
        .collect()
}

fn ahora() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::db::DbManager;
    use crate::modules::auditoria::data::SqliteAuditoria;
    use crate::modules::caja::application::CajaService;
    use crate::modules::caja::data::{SqliteBienes, SqliteFondoGastos, SqliteLibro};
    use crate::modules::configuracion::data::SqliteConfiguracion;
    use crate::modules::caja::puente::LibroViaCaja;
    use crate::modules::configuracion::domain::ConfiguracionPort;
    use crate::modules::deuda::data::{
        SqliteCreditoConsulta, SqliteCuotas, SqliteParametrosDeuda,
    };
    use uuid::Uuid;

    struct Ctx {
        deuda: DeudaService,
        caja: Arc<CajaService>,
        db: DbManager,
        banco: String,
    }

    /// Monta el Servicio de la Deuda con sus adaptadores reales sobre una BD temporal,
    /// con la configuración sembrada (tasa de mora por defecto = 5%).
    fn contexto() -> Ctx {
        let dir = std::env::temp_dir().join(format!("bkn_deuda_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = DbManager::new(dir);
        let banco = Uuid::new_v4().to_string();
        db.open_banco_db(&banco).unwrap();

        // Siembra la configuración por defecto (tasa_interes_mora = 5%).
        let cfg = SqliteConfiguracion::new(db.clone()).obtener(&banco).unwrap();
        assert_eq!(cfg.tasa_interes_mora, 5.0);

        // El crédito referencia a un socio (FK): se siembra 'socio-1'.
        db.open_banco_db(&banco).unwrap().execute(
            "INSERT INTO socio (id, cedula, nombres, apellidos, profesion, direccion,
                                telefono, celular, correo, estatus, fecha_ingreso)
             VALUES ('socio-1','123','Ana','Ríos','','','','','','ACTIVO','2026-01-01')",
            [],
        ).unwrap();

        let caja = Arc::new(CajaService::new(
            Arc::new(SqliteLibro::new(db.clone())),
            Arc::new(SqliteFondoGastos::new(db.clone())),
            Arc::new(SqliteBienes::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
        ));
        let deuda = DeudaService::new(
            Arc::new(SqliteCreditoConsulta::new(db.clone())),
            Arc::new(SqliteCuotas::new(db.clone())),
            Arc::new(LibroViaCaja::new(caja.clone())),
            Arc::new(SqliteParametrosDeuda::new(db.clone())),
            Arc::new(SqliteAuditoria::new(db.clone())),
            Arc::new(crate::core::atomicidad::SqliteAtomicidad::new(db.clone())),
        );
        Ctx { deuda, caja, db, banco }
    }

    /// Inserta un crédito VIGENTE con una cuota. Devuelve el número del crédito.
    fn credito_con_cuota(
        ctx: &Ctx,
        numero: &str,
        saldo: f64,
        capital: f64,
        interes: f64,
        venc: &str,
    ) -> String {
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        let credito_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO credito
             (id, socio_id, numero, monto_original, tasa, plazo_cuotas, cuota_actual,
              saldo_pendiente, destino, estatus, fecha_solicitud, fecha_desembolso,
              frecuencia_pago, fecha_vencimiento, solicitud_id)
             VALUES (?1, 'socio-1', ?2, ?3, 3, 12, 1, ?4, 'GP', 'VIGENTE',
                     '2026-01-01', '2026-01-01', 'MENSUAL', ?5, NULL)",
            rusqlite::params![credito_id, numero, saldo, saldo, venc],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cuota
             (id, credito_id, numero, fecha_vencimiento, capital, interes, valor_total)
             VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                Uuid::new_v4().to_string(),
                credito_id,
                venc,
                capital,
                interes,
                capital + interes
            ],
        )
        .unwrap();
        numero.to_string()
    }

    /// Inserta un crédito VIGENTE con su tabla de amortización completa (saldo decreciente).
    fn credito_amortizado(
        ctx: &Ctx,
        numero: &str,
        monto: f64,
        tasa: f64,
        plazo: i64,
        primer_venc: &str,
    ) -> String {
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        let credito_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO credito
             (id, socio_id, numero, monto_original, tasa, plazo_cuotas, cuota_actual,
              saldo_pendiente, destino, estatus, fecha_solicitud, fecha_desembolso,
              frecuencia_pago, fecha_vencimiento, solicitud_id)
             VALUES (?1, 'socio-1', ?2, ?3, ?4, ?5, 1, ?3, 'GP', 'VIGENTE',
                     '2026-01-01', '2026-01-01', 'MENSUAL', ?6, NULL)",
            rusqlite::params![credito_id, numero, monto, tasa, plazo, primer_venc],
        )
        .unwrap();
        let tabla = crate::modules::creditos::domain::calcular_tabla(monto, tasa, plazo, primer_venc);
        for c in &tabla {
            conn.execute(
                "INSERT INTO cuota
                 (id, credito_id, numero, fecha_vencimiento, capital, interes, valor_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    Uuid::new_v4().to_string(),
                    credito_id,
                    c.numero,
                    c.fecha_vencimiento,
                    c.capital,
                    c.interes,
                    c.valor_total
                ],
            )
            .unwrap();
        }
        numero.to_string()
    }

    /// Cuotas pendientes (numero, capital, interés) de un crédito, en orden.
    fn cuotas_pendientes(ctx: &Ctx, numero: &str) -> Vec<(i64, f64, f64)> {
        let conn = ctx.db.open_banco_db(&ctx.banco).unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT cu.numero, cu.capital, cu.interes FROM cuota cu
                 JOIN credito c ON c.id = cu.credito_id
                 WHERE c.numero = ?1 AND cu.pagada = 0 ORDER BY cu.numero",
            )
            .unwrap();
        let rows = stmt
            .query_map([numero], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap();
        rows.map(|r| r.unwrap()).collect()
    }

    /// Suma de una columna del Libro filtrando por código de operación.
    fn asiento(ctx: &Ctx, codigo: &str) -> f64 {
        ctx.db
            .open_banco_db(&ctx.banco)
            .unwrap()
            .query_row(
                "SELECT COALESCE(SUM(ingreso), 0) FROM movimiento_libro WHERE codigo = ?1",
                [codigo],
                |r| r.get(0),
            )
            .unwrap()
    }

    fn cuota_estado(ctx: &Ctx, numero: &str) -> (f64, f64, f64, bool) {
        ctx.db
            .open_banco_db(&ctx.banco)
            .unwrap()
            .query_row(
                "SELECT cu.capital_pagado, cu.interes_pagado, cu.mora_pagada, cu.pagada
                 FROM cuota cu JOIN credito c ON c.id = cu.credito_id
                 WHERE c.numero = ?1 AND cu.numero = 1",
                [numero],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0)),
            )
            .unwrap()
    }

    fn credito_estado(ctx: &Ctx, numero: &str) -> (f64, String) {
        ctx.db
            .open_banco_db(&ctx.banco)
            .unwrap()
            .query_row(
                "SELECT saldo_pendiente, estatus FROM credito WHERE numero = ?1",
                [numero],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap()
    }

    /// RF-69: pago total de una cuota a tiempo (sin mora). El capital baja el saldo, el
    /// interés y el capital se asientan (OR + PC), y la caja sube por el total cobrado.
    #[test]
    fn pago_total_a_tiempo_imputa_y_asienta_bien() {
        let ctx = contexto();
        credito_con_cuota(&ctx, "001", 120_000.0, 10_000.0, 3_600.0, "2026-01-31");

        let r = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "001", 1, "2026-01-31", 13_600.0)
            .unwrap();

        assert_eq!(r.aplicacion.mora, 0.0);
        assert_eq!(r.aplicacion.interes, 3_600.0);
        assert_eq!(r.aplicacion.capital, 10_000.0);
        assert!(r.cuota_saldada);

        let (cap, int, mora, pagada) = cuota_estado(&ctx, "001");
        assert_eq!((cap, int, mora), (10_000.0, 3_600.0, 0.0));
        assert!(pagada);

        let (saldo, estatus) = credito_estado(&ctx, "001");
        assert_eq!(saldo, 110_000.0);
        assert_eq!(estatus, "VIGENTE");

        // Libro: interés ordinario (OR) y capital (PC); nada de mora (MO).
        assert_eq!(asiento(&ctx, "OR"), 3_600.0);
        assert_eq!(asiento(&ctx, "PC"), 10_000.0);
        assert_eq!(asiento(&ctx, "MO"), 0.0);
        // Cuadre: la caja sube exactamente por lo cobrado (todo son ingresos).
        assert_eq!(ctx.caja.saldo_caja(&ctx.banco).unwrap(), 13_600.0);
    }

    /// RF-71/RF-65: abono parcial con atraso. La cascada paga primero la mora, luego el
    /// interés; no alcanza para capital, así que la cuota no se salda ni baja el saldo.
    #[test]
    fn abono_parcial_con_mora_respeta_la_cascada() {
        let ctx = contexto();
        credito_con_cuota(&ctx, "002", 120_000.0, 10_000.0, 3_600.0, "2026-01-31");

        // 10 días de atraso, mora 5%: 13.600 × 5%/30 × 10 = 227 (peso entero).
        let r = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "002", 1, "2026-02-10", 1_000.0)
            .unwrap();

        assert_eq!(r.mora_debida, 227.0);
        assert_eq!(r.dias_atraso, 10);
        assert_eq!(r.aplicacion.mora, 227.0);
        assert_eq!(r.aplicacion.interes, 773.0);
        assert_eq!(r.aplicacion.capital, 0.0);
        assert!(!r.cuota_saldada);

        // El saldo del crédito no baja (no se abonó capital).
        let (saldo, estatus) = credito_estado(&ctx, "002");
        assert_eq!(saldo, 120_000.0);
        assert_eq!(estatus, "VIGENTE");

        assert_eq!(asiento(&ctx, "MO"), 227.0);
        assert_eq!(asiento(&ctx, "OR"), 773.0);
        assert_eq!(asiento(&ctx, "PC"), 0.0);
        assert_eq!(ctx.caja.saldo_caja(&ctx.banco).unwrap(), 1_000.0);
    }

    /// RF-69: al pagar el capital pendiente completo, el saldo llega a 0 y el crédito
    /// queda PAGADO.
    #[test]
    fn pagar_el_ultimo_capital_cancela_el_credito() {
        let ctx = contexto();
        // Crédito de una sola cuota cuyo capital es todo el saldo.
        credito_con_cuota(&ctx, "003", 10_000.0, 10_000.0, 300.0, "2026-01-31");

        let r = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "003", 1, "2026-01-31", 10_300.0)
            .unwrap();
        assert_eq!(r.nuevo_saldo, 0.0);
        assert_eq!(r.estatus_credito, "PAGADO");

        let (saldo, estatus) = credito_estado(&ctx, "003");
        assert_eq!(saldo, 0.0);
        assert_eq!(estatus, "PAGADO");
    }

    /// RF-68: un pago no puede exceder lo que la cuota debe hoy.
    #[test]
    fn pago_mayor_al_debido_se_rechaza() {
        let ctx = contexto();
        credito_con_cuota(&ctx, "004", 120_000.0, 10_000.0, 3_600.0, "2026-01-31");
        let err = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "004", 1, "2026-01-31", 99_999.0)
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
        // Nada se movió: la caja sigue en cero.
        assert_eq!(ctx.caja.saldo_caja(&ctx.banco).unwrap(), 0.0);
    }

    /// Una cuota ya pagada no se puede volver a pagar.
    #[test]
    fn cuota_ya_pagada_se_rechaza() {
        let ctx = contexto();
        credito_con_cuota(&ctx, "005", 120_000.0, 10_000.0, 3_600.0, "2026-01-31");
        ctx.deuda
            .registrar_pago_cuota(&ctx.banco, "005", 1, "2026-01-31", 13_600.0)
            .unwrap();
        let err = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "005", 1, "2026-01-31", 100.0)
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
    }

    /// Abono a capital: tras pagar la cuota 1 (saldo 900.000), un abono de 400.000 baja el
    /// saldo a 500.000 y re-amortiza a 5 cuotas de 100.000; el interés de la próxima cuota
    /// se recalcula sobre el saldo nuevo (3% de 500.000 = 15.000).
    #[test]
    fn abono_a_capital_reduce_las_cuotas() {
        let ctx = contexto();
        credito_amortizado(&ctx, "010", 1_000_000.0, 3.0, 10, "2026-01-31");

        ctx.deuda.pagar_cuotas(&ctx.banco, "010", "2026-01-31", 1).unwrap();
        assert_eq!(credito_estado(&ctx, "010").0, 900_000.0);

        let r = ctx.deuda.abonar_capital(&ctx.banco, "010", "2026-02-15", 400_000.0).unwrap();
        assert_eq!(r.nuevo_saldo, 500_000.0);
        assert_eq!(r.cuotas_restantes, 5);
        assert_eq!(r.estatus_credito, "VIGENTE");

        let (saldo, estatus) = credito_estado(&ctx, "010");
        assert_eq!(saldo, 500_000.0);
        assert_eq!(estatus, "VIGENTE");

        let pend = cuotas_pendientes(&ctx, "010");
        assert_eq!(pend.len(), 5);
        assert!(pend.iter().all(|(_, cap, _)| *cap == 100_000.0));
        assert_eq!(pend[0].2, 15_000.0, "el interés se recalcula sobre el saldo nuevo");

        // Libro: capital de la cuota 1 (100.000) + el abono (400.000) = 500.000; interés 30.000.
        assert_eq!(asiento(&ctx, "PC"), 500_000.0);
        assert_eq!(asiento(&ctx, "OR"), 30_000.0);
    }

    /// Pago de dos cuotas en el primer mes: un solo interés (30.000) + el capital de las dos
    /// (200.000) = 230.000; el saldo baja a 800.000 y quedan 8 cuotas.
    #[test]
    fn pago_de_dos_cuotas_cobra_un_solo_interes() {
        let ctx = contexto();
        credito_amortizado(&ctx, "020", 1_000_000.0, 3.0, 10, "2026-01-31");

        let r = ctx.deuda.pagar_cuotas(&ctx.banco, "020", "2026-01-31", 2).unwrap();
        assert_eq!(r.aplicacion.mora, 0.0);
        assert_eq!(r.aplicacion.interes, 30_000.0);
        assert_eq!(r.aplicacion.capital, 200_000.0);
        assert_eq!(r.nuevo_saldo, 800_000.0);

        let (saldo, estatus) = credito_estado(&ctx, "020");
        assert_eq!(saldo, 800_000.0);
        assert_eq!(estatus, "VIGENTE");

        let pend = cuotas_pendientes(&ctx, "020");
        assert_eq!(pend.len(), 8);
        assert_eq!(pend[0].1, 100_000.0);
        assert_eq!(pend[0].2, 24_000.0, "3% de 800.000");

        assert_eq!(asiento(&ctx, "OR"), 30_000.0);
        assert_eq!(asiento(&ctx, "PC"), 200_000.0);
        assert_eq!(asiento(&ctx, "MO"), 0.0);
        assert_eq!(ctx.caja.saldo_caja(&ctx.banco).unwrap(), 230_000.0);
    }

    /// Pago de dos cuotas en el segundo mes: el interés es el del mes (3% de 900.000 =
    /// 27.000) + capital de dos cuotas (200.000) = 227.000.
    #[test]
    fn pago_de_dos_cuotas_en_segundo_mes_usa_el_interes_del_mes() {
        let ctx = contexto();
        credito_amortizado(&ctx, "030", 1_000_000.0, 3.0, 10, "2026-01-31");

        ctx.deuda.pagar_cuotas(&ctx.banco, "030", "2026-01-31", 1).unwrap();
        let r = ctx.deuda.pagar_cuotas(&ctx.banco, "030", "2026-02-28", 2).unwrap();

        assert_eq!(r.aplicacion.interes, 27_000.0);
        assert_eq!(r.aplicacion.capital, 200_000.0);
        assert_eq!(r.nuevo_saldo, 700_000.0);
    }

    /// Un abono que cubre todo el saldo cancela el crédito (sin cuotas pendientes).
    #[test]
    fn abono_que_cubre_el_saldo_cancela_el_credito() {
        let ctx = contexto();
        credito_amortizado(&ctx, "040", 1_000_000.0, 3.0, 10, "2026-01-31");

        let r = ctx.deuda.abonar_capital(&ctx.banco, "040", "2026-01-15", 1_000_000.0).unwrap();
        assert_eq!(r.nuevo_saldo, 0.0);
        assert_eq!(r.cuotas_restantes, 0);
        assert_eq!(r.estatus_credito, "PAGADO");
        assert_eq!(credito_estado(&ctx, "040").1, "PAGADO");
        assert!(cuotas_pendientes(&ctx, "040").is_empty());
    }

    /// La mora tiene gracia hasta fin del mes de vencimiento: pagar dentro de ese mes no
    /// genera mora aunque sea después del día nominal; sí a partir del mes siguiente.
    #[test]
    fn no_hay_mora_si_se_paga_dentro_del_mes_de_vencimiento() {
        let ctx = contexto();
        credito_amortizado(&ctx, "050", 1_000_000.0, 3.0, 10, "2026-09-20");
        // La reunión se corrió al 25-sep: la cuota venció el 20 pero no hay mora.
        let r = ctx.deuda.pagar_cuotas(&ctx.banco, "050", "2026-09-25", 1).unwrap();
        assert_eq!(r.aplicacion.mora, 0.0);
        assert_eq!(r.dias_atraso, 0);
        assert_eq!(asiento(&ctx, "MO"), 0.0);

        // Otro crédito pagado ya en octubre: la mora corre desde el 1-oct (5 días al 5-oct).
        credito_amortizado(&ctx, "051", 1_000_000.0, 3.0, 10, "2026-09-20");
        let r2 = ctx.deuda.pagar_cuotas(&ctx.banco, "051", "2026-10-05", 1).unwrap();
        assert_eq!(r2.dias_atraso, 5);
        assert!(r2.aplicacion.mora > 0.0, "en octubre sí hay mora");
    }

    /// No se paga la cuota de un crédito inexistente.
    #[test]
    fn credito_inexistente_se_rechaza() {
        let ctx = contexto();
        let err = ctx
            .deuda
            .registrar_pago_cuota(&ctx.banco, "999", 1, "2026-01-31", 100.0)
            .unwrap_err();
        assert!(matches!(err, AppError::OperacionNoPermitida(_)));
    }
}
