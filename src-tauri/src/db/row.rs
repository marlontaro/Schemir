//! Una fila de resultado independiente del driver.
//!
//! SQL Server se puede leer por dos caminos —tiberius sobre TCP, u ODBC, que es
//! como se conecta SSMS— y las consultas y el parseo son los mismos en los dos.
//! Para no escribirlos dos veces, cada transporte traduce lo que devuelve su
//! driver a estas filas, y a partir de ahí el código es uno solo.
//!
//! Los accesores no fallan: si la columna no está o viene NULL, devuelven el
//! valor por defecto que se les pase. Las consultas son constantes de este mismo
//! módulo y los nombres salen de ahí, así que una columna ausente sería un error
//! de programación, no algo que deba abortar la lectura de un esquema entero.

use std::collections::BTreeMap;

/// El valor de una celda, reducido a lo que las consultas de Schemir piden.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Text(String),
    Int(i64),
    Bool(bool),
}

/// Una fila: columnas por nombre.
#[derive(Debug, Clone, Default)]
pub struct Row {
    cells: BTreeMap<String, Value>,
}

impl Row {
    pub fn new() -> Self {
        Self::default()
    }

    /// Agrega una celda. Los nombres de columna de SQL Server no distinguen
    /// mayúsculas, así que se guardan en minúsculas y se buscan igual.
    pub fn put(&mut self, name: &str, value: Value) {
        self.cells.insert(name.to_ascii_lowercase(), value);
    }

    fn get(&self, name: &str) -> Option<&Value> {
        self.cells.get(&name.to_ascii_lowercase())
    }

    /// Texto de la columna, o cadena vacía si falta o es NULL.
    pub fn text(&self, name: &str) -> &str {
        match self.get(name) {
            Some(Value::Text(s)) => s,
            _ => "",
        }
    }

    /// Texto de la columna, distinguiendo NULL de cadena vacía. Hace falta para
    /// los valores por defecto: una columna sin DEFAULT no es lo mismo que una
    /// con DEFAULT ''.
    pub fn opt_text(&self, name: &str) -> Option<&str> {
        match self.get(name) {
            Some(Value::Text(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Entero de la columna. ODBC devuelve los números como texto, así que
    /// también se acepta un texto que se pueda convertir.
    pub fn int(&self, name: &str, default: i64) -> i64 {
        match self.get(name) {
            Some(Value::Int(n)) => *n,
            Some(Value::Bool(b)) => i64::from(*b),
            Some(Value::Text(s)) => s.trim().parse().unwrap_or(default),
            _ => default,
        }
    }

    /// Booleano de la columna. En SQL Server un `bit` llega como 1/0, y por ODBC
    /// como el texto "1"/"0".
    pub fn bool(&self, name: &str, default: bool) -> bool {
        match self.get(name) {
            Some(Value::Bool(b)) => *b,
            Some(Value::Int(n)) => *n != 0,
            Some(Value::Text(s)) => match s.trim() {
                "1" => true,
                "0" => false,
                otro => otro.eq_ignore_ascii_case("true"),
            },
            _ => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fila(pares: &[(&str, Value)]) -> Row {
        let mut r = Row::new();
        for (n, v) in pares {
            r.put(n, v.clone());
        }
        r
    }

    #[test]
    fn el_nombre_de_columna_no_distingue_mayusculas() {
        let r = fila(&[("Schema_Name", Value::Text("dbo".into()))]);
        assert_eq!(r.text("schema_name"), "dbo");
        assert_eq!(r.text("SCHEMA_NAME"), "dbo");
    }

    #[test]
    fn una_columna_que_falta_devuelve_el_valor_por_defecto() {
        let r = Row::new();
        assert_eq!(r.text("nada"), "");
        assert_eq!(r.opt_text("nada"), None);
        assert_eq!(r.int("nada", 7), 7);
        assert!(r.bool("nada", true));
    }

    #[test]
    fn null_no_es_lo_mismo_que_cadena_vacia() {
        let r = fila(&[("def", Value::Null), ("vacio", Value::Text(String::new()))]);
        assert_eq!(r.opt_text("def"), None);
        assert_eq!(r.opt_text("vacio"), Some(""));
    }

    #[test]
    fn los_numeros_se_leen_vengan_como_vengan() {
        // tiberius los entrega tipados; ODBC, como texto.
        let tipado = fila(&[("n", Value::Int(42))]);
        let texto = fila(&[("n", Value::Text(" 42 ".into()))]);
        assert_eq!(tipado.int("n", 0), 42);
        assert_eq!(texto.int("n", 0), 42);
    }

    #[test]
    fn los_bit_se_leen_vengan_como_vengan() {
        assert!(fila(&[("b", Value::Bool(true))]).bool("b", false));
        assert!(fila(&[("b", Value::Int(1))]).bool("b", false));
        assert!(fila(&[("b", Value::Text("1".into()))]).bool("b", false));
        assert!(!fila(&[("b", Value::Text("0".into()))]).bool("b", true));
    }

    #[test]
    fn un_texto_que_no_es_numero_cae_en_el_valor_por_defecto() {
        let r = fila(&[("n", Value::Text("(nada)".into()))]);
        assert_eq!(r.int("n", -1), -1);
    }
}
