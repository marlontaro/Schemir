//! Catálogo de tipos de datos de cada motor, con la versión en que apareció
//! cada uno.
//!
//! Para qué: el diseñador ofrece los tipos en una lista en vez de pedirlos
//! escritos a mano, sabe cuáles llevan longitud (`varchar(50)`), cuáles llevan
//! precisión y escala (`decimal(10,2)`) y cuáles no llevan nada (`int`), y puede
//! avisar cuando un tipo exige una versión del motor más nueva que la elegida.
//!
//! Sobre `since`: es la versión a partir de la cual el motor admite el tipo. Los
//! tipos que existen desde siempre llevan la versión MÁS ANTIGUA que cubre este
//! catálogo, que para cada motor es la más vieja que todavía recibe soporte del
//! fabricante. O sea: `since` igual al piso significa «desde esta y cualquiera
//! anterior», no «apareció justo aquí».
//!
//! El catálogo es curado, no exhaustivo: están los tipos con los que se modela
//! de verdad. Lo exótico (los rangos de PostgreSQL, el `cursor` de SQL Server,
//! los `enum` de MySQL que necesitan su lista de valores) se sigue pudiendo
//! escribir a mano en el diseñador.

use serde::{Deserialize, Serialize};

use crate::script::Dialect;

/// Una versión de un motor. Se ordena por número; `label` es cómo la escribe su
/// fabricante, que no se puede deducir del número: SQL Server usa el año
/// («2019»), MySQL escribe «8.0» y PostgreSQL, desde la 10, solo el mayor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub label: &'static str,
}

impl Version {
    const fn new(major: u32, minor: u32, label: &'static str) -> Self {
        Self { major, minor, label }
    }
}

// ─────────────────────────── SQL Server ──────────────────────────────────────
// El piso es 2008: es la más vieja cuyos tipos modernos (date, time, datetime2,
// datetimeoffset) ya existen, y todo lo anterior está fuera de soporte.
const SS_2008: Version = Version::new(2008, 0, "2008");
const SS_2012: Version = Version::new(2012, 0, "2012");
const SS_2014: Version = Version::new(2014, 0, "2014");
const SS_2016: Version = Version::new(2016, 0, "2016");
const SS_2017: Version = Version::new(2017, 0, "2017");
const SS_2019: Version = Version::new(2019, 0, "2019");
const SS_2022: Version = Version::new(2022, 0, "2022");
const SS_2025: Version = Version::new(2025, 0, "2025");

// ─────────────────────────── PostgreSQL ──────────────────────────────────────
const PG_96: Version = Version::new(9, 6, "9.6");
const PG_10: Version = Version::new(10, 0, "10");
const PG_12: Version = Version::new(12, 0, "12");
const PG_13: Version = Version::new(13, 0, "13");
const PG_14: Version = Version::new(14, 0, "14");
const PG_15: Version = Version::new(15, 0, "15");
const PG_16: Version = Version::new(16, 0, "16");
const PG_17: Version = Version::new(17, 0, "17");
const PG_18: Version = Version::new(18, 0, "18");

// ────────────────────────── MySQL / MariaDB ──────────────────────────────────
// Las versiones son las de MySQL. MariaDB numera aparte (10.x, 11.x) y para lo
// que hay aquí la equivalencia práctica es: MariaDB 10.2 ≈ MySQL 5.7.
const MY_56: Version = Version::new(5, 6, "5.6");
const MY_57: Version = Version::new(5, 7, "5.7");
const MY_80: Version = Version::new(8, 0, "8.0");
const MY_84: Version = Version::new(8, 4, "8.4");

// ──────────────────────────── SQLite ─────────────────────────────────────────
const LT_3: Version = Version::new(3, 0, "3.0");
const LT_331: Version = Version::new(3, 31, "3.31");
const LT_337: Version = Version::new(3, 37, "3.37");
const LT_345: Version = Version::new(3, 45, "3.45");

/// Qué lleva el tipo entre paréntesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeArgs {
    /// Nada: `int`, `date`, `uniqueidentifier`.
    Plain,
    /// Una longitud: `varchar(50)`, `char(2)`.
    Length {
        /// Lo que se propone al elegir el tipo.
        default: i32,
        /// Lo máximo que acepta el motor.
        max: i32,
        /// Admite la palabra `MAX` en lugar de un número (solo SQL Server).
        supports_max: bool,
        /// Se puede dejar sin paréntesis y el motor pone su propio límite.
        optional: bool,
    },
    /// Un solo número que no es longitud, sino precisión de fracción de segundo
    /// o de coma flotante: `datetime2(3)`, `float(53)`.
    Precision { default: i32, max: i32, optional: bool },
    /// Precisión y escala, las dos cifras separadas por coma: `decimal(10,2)`.
    PrecisionScale { precision: i32, scale: i32, max_precision: i32 },
}

/// Para agrupar la lista en el desplegable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeCategory {
    Number,
    Text,
    DateTime,
    Binary,
    Boolean,
    Other,
}

/// Un tipo de datos del catálogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataType {
    /// Como se escribe en el DDL.
    pub name: &'static str,
    pub category: TypeCategory,
    pub args: TypeArgs,
    /// Versión del motor desde la que está disponible (ver la nota del módulo).
    pub since: Version,
    /// El motor lo mantiene solo por compatibilidad: no conviene usarlo en algo
    /// nuevo.
    pub deprecated: bool,
    /// Qué usar en su lugar cuando está desaconsejado.
    pub instead: Option<&'static str>,
}

impl DataType {
    fn new(name: &'static str, category: TypeCategory, args: TypeArgs, since: Version) -> Self {
        Self { name, category, args, since, deprecated: false, instead: None }
    }
    /// Lo marca como desaconsejado y dice con qué reemplazarlo.
    fn old(mut self, instead: &'static str) -> Self {
        self.deprecated = true;
        self.instead = Some(instead);
        self
    }
}

// Atajos para que las listas de abajo se lean como una tabla.
fn plain(name: &'static str, c: TypeCategory, since: Version) -> DataType {
    DataType::new(name, c, TypeArgs::Plain, since)
}
fn length(name: &'static str, default: i32, max: i32, since: Version) -> DataType {
    DataType::new(name, TypeCategory::Text, TypeArgs::Length { default, max, supports_max: false, optional: false }, since)
}
/// Longitud que se puede omitir (el motor pone su límite).
fn length_opt(name: &'static str, default: i32, max: i32, since: Version) -> DataType {
    DataType::new(name, TypeCategory::Text, TypeArgs::Length { default, max, supports_max: false, optional: true }, since)
}
/// Longitud que además admite `MAX`.
fn length_max(name: &'static str, default: i32, max: i32, since: Version) -> DataType {
    DataType::new(name, TypeCategory::Text, TypeArgs::Length { default, max, supports_max: true, optional: false }, since)
}
fn bin(name: &'static str, args: TypeArgs, since: Version) -> DataType {
    DataType::new(name, TypeCategory::Binary, args, since)
}
fn prec(name: &'static str, c: TypeCategory, default: i32, max: i32, since: Version) -> DataType {
    DataType::new(name, c, TypeArgs::Precision { default, max, optional: true }, since)
}
fn dec(name: &'static str, precision: i32, scale: i32, max_precision: i32, since: Version) -> DataType {
    DataType::new(name, TypeCategory::Number, TypeArgs::PrecisionScale { precision, scale, max_precision }, since)
}
fn len_args(default: i32, max: i32, supports_max: bool, optional: bool) -> TypeArgs {
    TypeArgs::Length { default, max, supports_max, optional }
}

/// Todos los tipos que el diseñador ofrece para un motor, en el orden en que se
/// muestran.
pub fn data_types(d: Dialect) -> Vec<DataType> {
    use TypeCategory::{Boolean, DateTime, Number, Other, Text};
    match d {
        Dialect::SqlServer => vec![
            // Números
            plain("int", Number, SS_2008),
            plain("bigint", Number, SS_2008),
            plain("smallint", Number, SS_2008),
            plain("tinyint", Number, SS_2008),
            dec("decimal", 18, 0, 38, SS_2008),
            dec("numeric", 18, 0, 38, SS_2008),
            plain("money", Number, SS_2008),
            plain("smallmoney", Number, SS_2008),
            prec("float", Number, 53, 53, SS_2008),
            plain("real", Number, SS_2008),
            // Booleano
            plain("bit", Boolean, SS_2008),
            // Texto
            length_max("varchar", 50, 8000, SS_2008),
            length_max("nvarchar", 50, 4000, SS_2008),
            length("char", 10, 8000, SS_2008),
            length("nchar", 10, 4000, SS_2008),
            plain("text", Text, SS_2008).old("varchar(max)"),
            plain("ntext", Text, SS_2008).old("nvarchar(max)"),
            // Fecha y hora
            plain("date", DateTime, SS_2008),
            prec("time", DateTime, 7, 7, SS_2008),
            prec("datetime2", DateTime, 7, 7, SS_2008),
            prec("datetimeoffset", DateTime, 7, 7, SS_2008),
            plain("datetime", DateTime, SS_2008),
            plain("smalldatetime", DateTime, SS_2008),
            // Binario
            bin("varbinary", len_args(50, 8000, true, false), SS_2008),
            bin("binary", len_args(50, 8000, false, false), SS_2008),
            bin("image", TypeArgs::Plain, SS_2008).old("varbinary(max)"),
            // Otros
            plain("uniqueidentifier", Other, SS_2008),
            plain("xml", Other, SS_2008),
            plain("rowversion", Other, SS_2008),
            plain("hierarchyid", Other, SS_2008),
            plain("geography", Other, SS_2008),
            plain("geometry", Other, SS_2008),
            plain("sql_variant", Other, SS_2008),
            plain("json", Other, SS_2025),
            DataType::new("vector", Other, TypeArgs::Precision { default: 1536, max: 1998, optional: false }, SS_2025),
        ],
        Dialect::Postgres => vec![
            // Números
            plain("integer", Number, PG_96),
            plain("bigint", Number, PG_96),
            plain("smallint", Number, PG_96),
            dec("numeric", 18, 0, 1000, PG_96),
            dec("decimal", 18, 0, 1000, PG_96),
            plain("real", Number, PG_96),
            plain("double precision", Number, PG_96),
            plain("money", Number, PG_96),
            plain("serial", Number, PG_96).old("integer GENERATED BY DEFAULT AS IDENTITY"),
            plain("bigserial", Number, PG_96).old("bigint GENERATED BY DEFAULT AS IDENTITY"),
            plain("smallserial", Number, PG_96).old("smallint GENERATED BY DEFAULT AS IDENTITY"),
            // Booleano
            plain("boolean", Boolean, PG_96),
            // Texto
            plain("text", Text, PG_96),
            length_opt("varchar", 50, 10_485_760, PG_96),
            length_opt("char", 10, 10_485_760, PG_96),
            // Fecha y hora
            plain("date", DateTime, PG_96),
            prec("timestamptz", DateTime, 6, 6, PG_96),
            prec("timestamp", DateTime, 6, 6, PG_96),
            prec("time", DateTime, 6, 6, PG_96),
            prec("timetz", DateTime, 6, 6, PG_96),
            plain("interval", DateTime, PG_96),
            // Binario
            bin("bytea", TypeArgs::Plain, PG_96),
            // Otros
            plain("uuid", Other, PG_96),
            plain("jsonb", Other, PG_96),
            plain("json", Other, PG_96),
            plain("xml", Other, PG_96),
            plain("inet", Other, PG_96),
            plain("cidr", Other, PG_96),
            plain("macaddr", Other, PG_96),
            plain("macaddr8", Other, PG_10),
            DataType::new("bit", Other, len_args(1, 83_886_080, false, true), PG_96),
            DataType::new("varbit", Other, len_args(1, 83_886_080, false, true), PG_96),
            plain("tsvector", Other, PG_96),
        ],
        Dialect::MySql => vec![
            // Números
            plain("int", Number, MY_56),
            plain("bigint", Number, MY_56),
            plain("smallint", Number, MY_56),
            plain("mediumint", Number, MY_56),
            plain("tinyint", Number, MY_56),
            dec("decimal", 10, 0, 65, MY_56),
            dec("numeric", 10, 0, 65, MY_56),
            plain("double", Number, MY_56),
            prec("float", Number, 53, 53, MY_56),
            // Booleano
            plain("boolean", Boolean, MY_56),
            DataType::new("bit", Boolean, len_args(1, 64, false, true), MY_56),
            // Texto. En MySQL varchar EXIGE longitud, a diferencia de PostgreSQL.
            length("varchar", 50, 65_535, MY_56),
            length("char", 10, 255, MY_56),
            plain("text", Text, MY_56),
            plain("tinytext", Text, MY_56),
            plain("mediumtext", Text, MY_56),
            plain("longtext", Text, MY_56),
            // Fecha y hora
            plain("date", DateTime, MY_56),
            prec("datetime", DateTime, 6, 6, MY_56),
            prec("timestamp", DateTime, 6, 6, MY_56),
            prec("time", DateTime, 6, 6, MY_56),
            plain("year", DateTime, MY_56),
            // Binario
            bin("varbinary", len_args(50, 65_535, false, false), MY_56),
            bin("binary", len_args(1, 255, false, false), MY_56),
            bin("blob", TypeArgs::Plain, MY_56),
            bin("tinyblob", TypeArgs::Plain, MY_56),
            bin("mediumblob", TypeArgs::Plain, MY_56),
            bin("longblob", TypeArgs::Plain, MY_56),
            // Otros
            plain("json", Other, MY_57),
            plain("geometry", Other, MY_56),
            plain("point", Other, MY_56),
            plain("linestring", Other, MY_56),
            plain("polygon", Other, MY_56),
        ],
        // SQLite no tiene tipos propiamente: tiene AFINIDADES, y acepta
        // cualquier nombre que se le escriba. Se ofrecen las cinco afinidades
        // primero y después los alias que se usan por costumbre (y que SQLite
        // reduce a una de las cinco).
        Dialect::Sqlite => vec![
            plain("INTEGER", Number, LT_3),
            plain("REAL", Number, LT_3),
            plain("TEXT", Text, LT_3),
            bin("BLOB", TypeArgs::Plain, LT_3),
            plain("NUMERIC", Number, LT_3),
            length_opt("VARCHAR", 50, 1_000_000_000, LT_3),
            dec("DECIMAL", 10, 2, 1000, LT_3),
            plain("BOOLEAN", Boolean, LT_3),
            plain("DATE", DateTime, LT_3),
            plain("DATETIME", DateTime, LT_3),
        ],
    }
}

/// Las versiones del motor entre las que se puede elegir, de la más vieja a la
/// más nueva. La primera es el piso del catálogo.
pub fn engine_versions(d: Dialect) -> Vec<Version> {
    match d {
        Dialect::SqlServer => vec![SS_2008, SS_2012, SS_2014, SS_2016, SS_2017, SS_2019, SS_2022, SS_2025],
        Dialect::Postgres => vec![PG_96, PG_10, PG_12, PG_13, PG_14, PG_15, PG_16, PG_17, PG_18],
        Dialect::MySql => vec![MY_56, MY_57, MY_80, MY_84],
        Dialect::Sqlite => vec![LT_3, LT_331, LT_337, LT_345],
    }
}

/// La versión que el diseñador propone: una que esté en soporte y sea lo bastante
/// común para no obligar a nadie a cambiarla.
pub fn default_version(d: Dialect) -> Version {
    match d {
        Dialect::SqlServer => SS_2019,
        Dialect::Postgres => PG_14,
        Dialect::MySql => MY_80,
        Dialect::Sqlite => LT_337,
    }
}

/// ¿El motor agrupa las tablas en esquemas?
///
/// En MySQL/MariaDB «esquema» y «base de datos» son la misma cosa, así que no hay
/// un nivel intermedio que elegir; en SQLite tampoco (la base es un archivo).
pub fn supports_schemas(d: Dialect) -> bool {
    matches!(d, Dialect::SqlServer | Dialect::Postgres)
}

/// El esquema al que van las tablas si no se dice otra cosa.
pub fn default_schema(d: Dialect) -> &'static str {
    match d {
        Dialect::SqlServer => "dbo",
        Dialect::Postgres => "public",
        Dialect::MySql | Dialect::Sqlite => "",
    }
}

// ───────────────── Valores que pone el motor por su cuenta ──────────────────
//
// Las dos maneras corrientes de dar valor a una clave primaria sin escribirlo:
// un contador que el motor incrementa, o un identificador único que el motor
// genera. Cuál conviene no lo decide este módulo —depende de si el id se genera
// dentro o fuera de la base, de si se fusionan datos de varias— y por eso aquí
// están las dos, y cada motor dice cuál admite y sobre qué tipos.

/// Un valor que el motor pone solo al insertar la fila.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoValue {
    /// Un contador que el motor incrementa: `IDENTITY`, `AUTO_INCREMENT`.
    Identity,
    /// Un identificador único que el motor genera.
    Uuid,
}

/// Los tipos del motor a los que se les puede poner un contador.
pub fn identity_types(d: Dialect) -> Vec<&'static str> {
    match d {
        // En SQL Server `decimal` y `numeric` también valen, con escala 0.
        Dialect::SqlServer => vec!["tinyint", "smallint", "int", "bigint", "decimal", "numeric"],
        Dialect::Postgres => vec!["smallint", "integer", "bigint"],
        Dialect::MySql => vec!["tinyint", "smallint", "mediumint", "int", "bigint"],
        // SQLite solo admite AUTOINCREMENT sobre `INTEGER PRIMARY KEY`: ni otro
        // tipo, ni una columna que no sea la clave primaria.
        Dialect::Sqlite => vec!["INTEGER"],
    }
}

/// Los tipos a los que el motor sabe generarles un identificador único.
pub fn uuid_types(d: Dialect) -> Vec<&'static str> {
    match d {
        Dialect::SqlServer => vec!["uniqueidentifier"],
        Dialect::Postgres => vec!["uuid"],
        // MySQL no tiene un tipo uuid: lo corriente es guardarlo en char(36).
        Dialect::MySql => vec!["char"],
        // SQLite no trae generador de uuid, así que no se ofrece: mejor no
        // ofrecerlo que emitir algo que no es un uuid de verdad.
        Dialect::Sqlite => vec![],
    }
}

/// Lo que va pegado al tipo para declarar el contador.
pub fn identity_suffix(d: Dialect) -> &'static str {
    match d {
        Dialect::SqlServer => "IDENTITY(1,1)",
        // `serial` está desaconsejado desde PostgreSQL 10 a favor de esto.
        Dialect::Postgres => "GENERATED BY DEFAULT AS IDENTITY",
        Dialect::MySql => "AUTO_INCREMENT",
        Dialect::Sqlite => "AUTOINCREMENT",
    }
}

/// La expresión del `DEFAULT` que genera el identificador único, o `None` si el
/// motor no trae ninguna.
pub fn uuid_default(d: Dialect) -> Option<&'static str> {
    match d {
        // NEWSEQUENTIALID y no NEWID a propósito: un GUID al azar como clave
        // agrupada parte las páginas del índice en cada inserción, y esa clave
        // se arrastra además a todos los índices no agrupados.
        Dialect::SqlServer => Some("NEWSEQUENTIALID()"),
        // Incorporado en PostgreSQL 13; antes hacía falta la extensión pgcrypto.
        Dialect::Postgres => Some("gen_random_uuid()"),
        // MySQL exige los paréntesis cuando el DEFAULT es una expresión, y solo
        // los admite desde 8.0.13.
        Dialect::MySql => Some("(UUID())"),
        Dialect::Sqlite => None,
    }
}

/// Busca un tipo por nombre, sin distinguir mayúsculas.
pub fn find_type(d: Dialect, name: &str) -> Option<DataType> {
    let n = name.trim().to_lowercase();
    data_types(d).into_iter().find(|t| t.name.to_lowercase() == n)
}

/// Compone el tipo tal como va en el DDL. Los argumentos van como texto para que
/// el `MAX` de SQL Server pase igual que un número.
pub fn format_type(name: &str, a: Option<&str>, b: Option<&str>) -> String {
    let a = a.map(str::trim).filter(|s| !s.is_empty());
    let b = b.map(str::trim).filter(|s| !s.is_empty());
    match (a, b) {
        (Some(x), Some(y)) => format!("{name}({x},{y})"),
        (Some(x), None) => format!("{name}({x})"),
        _ => name.to_string(),
    }
}

/// La versión MÍNIMA del motor que exige un modelo, mirando los tipos que usa.
///
/// Es la más nueva de las versiones en que apareció cada tipo, con el piso del
/// catálogo como respuesta cuando ninguno exige nada en particular. Los nombres
/// que no están en el catálogo (escritos a mano) no cuentan: de esos no se sabe.
pub fn required_version(d: Dialect, type_names: &[String]) -> Version {
    let catalog = data_types(d);
    let floor = engine_versions(d)[0];
    type_names
        .iter()
        .filter_map(|n| {
            let base = n.split('(').next().unwrap_or(n).trim().to_lowercase();
            catalog.iter().find(|t| t.name.to_lowercase() == base).map(|t| t.since)
        })
        .max()
        .unwrap_or(floor)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOTORES: [Dialect; 4] = [Dialect::SqlServer, Dialect::Postgres, Dialect::MySql, Dialect::Sqlite];

    #[test]
    fn cada_motor_trae_sus_tipos_y_ninguno_repetido() {
        for d in MOTORES {
            let ts = data_types(d);
            assert!(ts.len() >= 10, "{d:?} trae muy pocos tipos");
            let mut nombres: Vec<String> = ts.iter().map(|t| t.name.to_lowercase()).collect();
            nombres.sort();
            let antes = nombres.len();
            nombres.dedup();
            assert_eq!(antes, nombres.len(), "{d:?} tiene tipos repetidos");
        }
    }

    #[test]
    fn los_tipos_llevan_los_argumentos_que_les_toca() {
        // varchar pide longitud; int no pide nada; decimal pide las dos cifras.
        let v = find_type(Dialect::SqlServer, "varchar").unwrap();
        assert!(matches!(v.args, TypeArgs::Length { supports_max: true, .. }));
        let i = find_type(Dialect::SqlServer, "int").unwrap();
        assert_eq!(i.args, TypeArgs::Plain);
        let d = find_type(Dialect::SqlServer, "decimal").unwrap();
        assert!(matches!(d.args, TypeArgs::PrecisionScale { precision: 18, scale: 0, .. }));
    }

    #[test]
    fn solo_sql_server_admite_max() {
        // El `varchar(max)` es de SQL Server. En PostgreSQL el varchar sin
        // longitud es ilimitado, y en MySQL la longitud es obligatoria.
        let ss = find_type(Dialect::SqlServer, "varchar").unwrap();
        assert!(matches!(ss.args, TypeArgs::Length { supports_max: true, optional: false, .. }));
        let pg = find_type(Dialect::Postgres, "varchar").unwrap();
        assert!(matches!(pg.args, TypeArgs::Length { supports_max: false, optional: true, .. }));
        let my = find_type(Dialect::MySql, "varchar").unwrap();
        assert!(matches!(my.args, TypeArgs::Length { supports_max: false, optional: false, .. }));
    }

    #[test]
    fn cada_motor_tiene_lo_suyo_y_no_lo_ajeno() {
        assert!(find_type(Dialect::SqlServer, "nvarchar").is_some());
        assert!(find_type(Dialect::Postgres, "nvarchar").is_none());
        assert!(find_type(Dialect::Postgres, "jsonb").is_some());
        assert!(find_type(Dialect::SqlServer, "jsonb").is_none());
        assert!(find_type(Dialect::MySql, "longtext").is_some());
        assert!(find_type(Dialect::SqlServer, "longtext").is_none());
    }

    #[test]
    fn version_minima_sale_del_tipo_mas_nuevo() {
        let d = Dialect::SqlServer;
        // Tipos de siempre: el piso del catálogo.
        let viejos = vec!["int".to_string(), "varchar(50)".to_string()];
        assert_eq!(required_version(d, &viejos).label, "2008");
        // El json de SQL Server es de 2025 y manda sobre el resto.
        let con_json = vec!["int".to_string(), "json".to_string()];
        assert_eq!(required_version(d, &con_json).label, "2025");
        // En MySQL el json es de 5.7.
        let my = vec!["int".to_string(), "json".to_string()];
        assert_eq!(required_version(Dialect::MySql, &my).label, "5.7");
    }

    #[test]
    fn version_minima_ignora_lo_que_no_conoce() {
        // Un tipo escrito a mano no sube ni baja la exigencia: no se sabe de él.
        let names = vec!["int".to_string(), "mi_tipo_raro".to_string()];
        assert_eq!(required_version(Dialect::SqlServer, &names).label, "2008");
    }

    #[test]
    fn el_tipo_se_compone_con_sus_argumentos() {
        assert_eq!(format_type("int", None, None), "int");
        assert_eq!(format_type("varchar", Some("50"), None), "varchar(50)");
        assert_eq!(format_type("varchar", Some("max"), None), "varchar(max)");
        assert_eq!(format_type("decimal", Some("10"), Some("2")), "decimal(10,2)");
        // Vacíos: como si no estuvieran.
        assert_eq!(format_type("int", Some(""), Some("")), "int");
        assert_eq!(format_type("varchar", Some(" 50 "), Some("")), "varchar(50)");
    }

    #[test]
    fn los_esquemas_son_de_sql_server_y_postgres() {
        assert!(supports_schemas(Dialect::SqlServer));
        assert!(supports_schemas(Dialect::Postgres));
        // En MySQL el esquema ES la base; en SQLite no hay.
        assert!(!supports_schemas(Dialect::MySql));
        assert!(!supports_schemas(Dialect::Sqlite));
        assert_eq!(default_schema(Dialect::SqlServer), "dbo");
        assert_eq!(default_schema(Dialect::Postgres), "public");
        assert_eq!(default_schema(Dialect::MySql), "");
    }

    #[test]
    fn la_version_por_defecto_es_una_de_las_ofrecidas() {
        for d in MOTORES {
            let vs = engine_versions(d);
            assert!(vs.contains(&default_version(d)), "{d:?}");
            // Y la lista va de la más vieja a la más nueva.
            let mut ordenadas = vs.clone();
            ordenadas.sort();
            assert_eq!(vs, ordenadas, "{d:?} no está ordenada");
        }
    }

    #[test]
    fn ningun_tipo_exige_una_version_que_no_se_puede_elegir() {
        // Si un tipo dijera «desde la 2030», el aviso de versión no tendría
        // forma de dejar de saltar: no habría versión que lo satisfaga.
        for d in MOTORES {
            let ultima = *engine_versions(d).last().unwrap();
            for t in data_types(d) {
                assert!(t.since <= ultima, "{d:?}: {} exige {}", t.name, t.since.label);
            }
        }
    }

    /// El diseñador lee este JSON campo por campo (`args.kind`, `since.label`,
    /// `category`), así que renombrar algo aquí rompe la interfaz sin que el
    /// compilador diga nada. Esta prueba es ese contrato.
    #[test]
    fn el_json_del_catalogo_trae_los_nombres_que_espera_la_interfaz() {
        let j = |d, n| serde_json::to_value(find_type(d, n).unwrap()).unwrap();

        let v = j(Dialect::SqlServer, "varchar");
        assert_eq!(v["name"], "varchar");
        assert_eq!(v["category"], "text");
        assert_eq!(v["args"]["kind"], "length");
        assert_eq!(v["args"]["default"], 50);
        assert_eq!(v["args"]["max"], 8000);
        assert_eq!(v["args"]["supports_max"], true);
        assert_eq!(v["args"]["optional"], false);
        assert_eq!(v["since"]["label"], "2008");
        assert_eq!(v["since"]["major"], 2008);
        assert_eq!(v["deprecated"], false);

        let d = j(Dialect::SqlServer, "decimal");
        assert_eq!(d["args"]["kind"], "precision_scale");
        assert_eq!(d["args"]["precision"], 18);
        assert_eq!(d["args"]["scale"], 0);
        assert_eq!(d["args"]["max_precision"], 38);

        assert_eq!(j(Dialect::SqlServer, "int")["args"]["kind"], "plain");

        let dt = j(Dialect::SqlServer, "datetime2");
        assert_eq!(dt["args"]["kind"], "precision");
        assert_eq!(dt["args"]["optional"], true);
        assert_eq!(dt["category"], "date_time");

        let txt = j(Dialect::SqlServer, "text");
        assert_eq!(txt["deprecated"], true);
        assert_eq!(txt["instead"], "varchar(max)");

        // Y el dialecto viaja con el nombre que usan los desplegables.
        assert_eq!(serde_json::to_value(Dialect::SqlServer).unwrap(), "sqlserver");
        assert_eq!(serde_json::to_value(Dialect::MySql).unwrap(), "mysql");
        assert_eq!(serde_json::to_value(Dialect::Postgres).unwrap(), "postgres");
        assert_eq!(serde_json::to_value(Dialect::Sqlite).unwrap(), "sqlite");
    }

    /// Si un nombre de aquí estuviera mal escrito, el diseñador nunca ofrecería
    /// el contador para ese tipo y nadie se enteraría.
    #[test]
    fn los_tipos_que_se_autogeneran_existen_en_su_motor() {
        for d in MOTORES {
            for n in identity_types(d) {
                assert!(find_type(d, n).is_some(), "{d:?}: contador sobre {n}, que no está en el catálogo");
            }
            for n in uuid_types(d) {
                assert!(find_type(d, n).is_some(), "{d:?}: uuid sobre {n}, que no está en el catálogo");
            }
        }
    }

    #[test]
    fn cada_motor_autogenera_a_su_manera() {
        assert_eq!(identity_suffix(Dialect::SqlServer), "IDENTITY(1,1)");
        assert_eq!(identity_suffix(Dialect::MySql), "AUTO_INCREMENT");
        assert_eq!(identity_suffix(Dialect::Postgres), "GENERATED BY DEFAULT AS IDENTITY");
        assert_eq!(identity_suffix(Dialect::Sqlite), "AUTOINCREMENT");

        assert_eq!(uuid_default(Dialect::SqlServer), Some("NEWSEQUENTIALID()"));
        assert_eq!(uuid_default(Dialect::Postgres), Some("gen_random_uuid()"));
        // SQLite no trae generador de uuid y no se inventa ninguno.
        assert_eq!(uuid_default(Dialect::Sqlite), None);
        assert!(uuid_types(Dialect::Sqlite).is_empty());
    }

    #[test]
    fn el_contador_va_sobre_enteros_y_no_sobre_texto() {
        for d in MOTORES {
            for n in identity_types(d) {
                let ty = find_type(d, n).unwrap();
                assert!(
                    matches!(ty.category, TypeCategory::Number),
                    "{d:?}: {n} no es un número y no debería admitir contador"
                );
            }
        }
        // Y el de SQLite es solo el INTEGER, porque AUTOINCREMENT no vale sobre
        // ningún otro tipo ni fuera de la clave primaria.
        assert_eq!(identity_types(Dialect::Sqlite), vec!["INTEGER"]);
    }

    #[test]
    fn lo_desaconsejado_dice_con_que_reemplazarlo() {
        for d in MOTORES {
            for t in data_types(d) {
                assert_eq!(t.deprecated, t.instead.is_some(), "{d:?}: {}", t.name);
            }
        }
        assert_eq!(find_type(Dialect::SqlServer, "text").unwrap().instead, Some("varchar(max)"));
    }
}
