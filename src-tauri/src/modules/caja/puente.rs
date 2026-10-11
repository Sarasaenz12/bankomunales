//! Puente único hacia el Libro: Caja es la dueña del consecutivo y del saldo, así que
//! los demás módulos le piden los asientos a través de este adaptador. Cada módulo sigue
//! declarando su propio puerto (lo mínimo que necesita); aquí se implementan todos.

use std::sync::Arc;

use crate::core::error::AppError;
use crate::modules::acciones::domain::LibroContablePort as LibroAcciones;
use crate::modules::cierre::domain::LibroContablePort as LibroCierre;
use crate::modules::creditos::domain::LibroContablePort as LibroCreditos;
use crate::modules::deuda::domain::LibroDeudaPort as LibroDeuda;

use super::application::CajaService;
use super::domain::CodigoOperacion;

pub struct LibroViaCaja {
    caja: Arc<CajaService>,
}

impl LibroViaCaja {
    pub fn new(caja: Arc<CajaService>) -> Self {
        Self { caja }
    }

    #[allow(clippy::too_many_arguments)]
    fn asentar(
        &self,
        banco_id: &str,
        codigo: CodigoOperacion,
        fecha: &str,
        monto: f64,
        descripcion: &str,
        socio_id: &str,
        credito_id: Option<&str>,
    ) -> Result<(), AppError> {
        self.caja
            .registrar_asiento_de_modulo(
                banco_id,
                codigo,
                fecha.to_string(),
                monto,
                descripcion.to_string(),
                Some(socio_id.to_string()),
                credito_id.map(str::to_string),
            )
            .map(|_| ())
    }
}

impl LibroAcciones for LibroViaCaja {
    fn registrar_venta_acciones(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::VentaAcciones, fecha, monto, descripcion, socio_id, None)
    }
}

impl LibroCierre for LibroViaCaja {
    fn registrar_ganancia_repartida(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(
            banco_id,
            CodigoOperacion::GananciaRepartida,
            fecha,
            monto,
            "Pago de ganancias (UR)",
            socio_id,
            None,
        )
    }
}

impl LibroCreditos for LibroViaCaja {
    fn registrar_desembolso(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::DesembolsoCredito, fecha, monto, descripcion, socio_id, Some(credito_id))
    }

    fn registrar_refinanciamiento(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
        descripcion: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::Refinanciamiento, fecha, monto, descripcion, socio_id, Some(credito_id))
    }

    fn registrar_interes_ordinario(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(
            banco_id,
            CodigoOperacion::InteresOrdinario,
            fecha,
            monto,
            "Interés ordinario saldado al refinanciar",
            socio_id,
            Some(credito_id),
        )
    }

    fn registrar_mora(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(
            banco_id,
            CodigoOperacion::InteresMora,
            fecha,
            monto,
            "Mora saldada al refinanciar",
            socio_id,
            Some(credito_id),
        )
    }
}

impl LibroDeuda for LibroViaCaja {
    fn registrar_interes_ordinario(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::InteresOrdinario, fecha, monto, "Interés ordinario de cuota", socio_id, Some(credito_id))
    }

    fn registrar_pago_capital(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::PagoCuota, fecha, monto, "Pago de capital de cuota", socio_id, Some(credito_id))
    }

    fn registrar_mora(
        &self,
        banco_id: &str,
        fecha: &str,
        monto: f64,
        socio_id: &str,
        credito_id: &str,
    ) -> Result<(), AppError> {
        self.asentar(banco_id, CodigoOperacion::InteresMora, fecha, monto, "Interés de mora de cuota", socio_id, Some(credito_id))
    }
}
