//! Formateo de dinero para los mensajes que produce el backend.
//!
//! Todo valor de caja se muestra en **pesos enteros con separador de miles** (`.`),
//! sin centavos: `30000.0` → `"30.000"`. Los porcentajes conservan sus decimales y no
//! usan este helper.

/// Formatea un monto como pesos enteros con puntos de miles: `30000.0` → `"30.000"`,
/// `-1234567.0` → `"-1.234.567"`.
pub fn pesos(valor: f64) -> String {
    let n = valor.round() as i64;
    let negativo = n < 0;
    let digitos = n.unsigned_abs().to_string();
    let len = digitos.len();

    let mut salida = String::with_capacity(len + len / 3 + 1);
    if negativo {
        salida.push('-');
    }
    for (i, c) in digitos.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            salida.push('.');
        }
        salida.push(c);
    }
    salida
}

#[cfg(test)]
mod tests {
    use super::pesos;

    #[test]
    fn formatea_con_separador_de_miles_y_sin_centavos() {
        assert_eq!(pesos(0.0), "0");
        assert_eq!(pesos(999.0), "999");
        assert_eq!(pesos(1_000.0), "1.000");
        assert_eq!(pesos(30_000.0), "30.000");
        assert_eq!(pesos(1_234_567.0), "1.234.567");
        // Redondea al entero más cercano (sin centavos).
        assert_eq!(pesos(19_999.6), "20.000");
        assert_eq!(pesos(-1_234_567.0), "-1.234.567");
    }
}
