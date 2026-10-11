use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Respaldo {
    pub id: String,
    pub fecha: String,
    pub usuario: String,
    pub ruta_archivo: String,
    pub tipo: String,
}

/// Lo que el sistema reconoce de un archivo de respaldo antes de importarlo: de qué
/// Bankomunal es y qué pasaría en este computador.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VistaRespaldo {
    pub id: String,
    pub nombre: String,
    pub ubicacion: String,
    pub moneda: String,
    pub fecha_creacion: String,
    /// El Bankomunal ya está en este computador: importar REEMPLAZA sus datos actuales.
    pub ya_existe: bool,
}

pub trait RespaldoPort: Send + Sync {
    /// Lee de qué Bankomunal es el archivo (sin modificar nada).
    fn inspeccionar_respaldo(
        &self,
        origen_ruta: &str,
    ) -> Result<VistaRespaldo, crate::core::error::AppError>;
    /// Deja el Bankomunal del archivo disponible en este computador: lo registra en el
    /// catálogo si no estaba, o reemplaza sus datos si ya estaba.
    fn importar_respaldo(
        &self,
        origen_ruta: &str,
    ) -> Result<VistaRespaldo, crate::core::error::AppError>;
    fn registrar_respaldo(
        &self,
        banco_id: &str,
        respaldo: Respaldo,
    ) -> Result<(), crate::core::error::AppError>;
    fn crear_copia_backup(
        &self,
        banco_id: &str,
        destino_ruta: &str,
    ) -> Result<(), crate::core::error::AppError>;
    fn validar_y_restaurar_backup(
        &self,
        banco_id: &str,
        origen_ruta: &str,
    ) -> Result<(), crate::core::error::AppError>;
    fn listar_respaldos(
        &self,
        banco_id: &str,
    ) -> Result<Vec<Respaldo>, crate::core::error::AppError>;
}

/// Extensión de los archivos de respaldo que se pueden abrir con doble clic.
pub const EXTENSION_RESPALDO: &str = "bankomunal";

/// Respaldo con el que se lanzó la app: el primer argumento (después del programa) que sea
/// un archivo `.bankomunal`. Windows lo pasa así cuando se hace doble clic en el archivo.
pub fn ruta_de_respaldo(argumentos: &[String]) -> Option<String> {
    argumentos
        .iter()
        .skip(1)
        .find(|a| {
            std::path::Path::new(a.as_str())
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION_RESPALDO))
        })
        .cloned()
}

#[cfg(test)]
mod tests_apertura {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn reconoce_el_archivo_de_respaldo_pasado_al_abrir() {
        let a = args(&["C:/Programas/Bankomunales/bankomunales.exe", "C:/Usuarios/Ana/Banco_2026-10-11.bankomunal"]);
        assert_eq!(ruta_de_respaldo(&a).as_deref(), Some("C:/Usuarios/Ana/Banco_2026-10-11.bankomunal"));
    }

    #[test]
    fn sin_archivo_o_con_otra_extension_no_hay_respaldo() {
        assert_eq!(ruta_de_respaldo(&args(&["bankomunales.exe"])), None);
        assert_eq!(ruta_de_respaldo(&args(&["bankomunales.exe", "datos.db"])), None);
        // El programa mismo (primer argumento) nunca cuenta.
        assert_eq!(ruta_de_respaldo(&args(&["raro.bankomunal"])), None);
    }

    #[test]
    fn la_extension_no_distingue_mayusculas() {
        assert!(ruta_de_respaldo(&args(&["x.exe", "A.BANKOMUNAL"])).is_some());
    }
}
