//! Modelo estructurado para dibujar el diagrama entidad-relación con un
//! renderizador propio (la interfaz lo posiciona con dagre y lo dibuja en HTML).
//!
//! A diferencia de Mermaid, esto nos da control total del formato: encabezados
//! «Campo · Tipo de dato · Clave», orden de columnas y colores.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::model::Schema;

/// Una columna tal como se muestra en el diagrama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramColumn {
    pub name: String,
    pub data_type: String,
    /// "PK", "FK", "PK·FK" o "" (vacío).
    pub key: String,
}

/// Una tabla (entidad) del diagrama.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramTable {
    /// Id saneado para usar como clave de nodo.
    pub id: String,
    /// Nombre calificado para mostrar, p.ej. "dbo.Clientes".
    pub name: String,
    pub columns: Vec<DiagramColumn>,
}

/// Una relación de clave foránea (child -> parent).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramRelation {
    pub from: String, // tabla con la FID (lado "muchos")
    pub to: String,   // tabla referenciada (lado "uno")
    pub label: String,
}

/// El diagrama completo de un esquema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagramModel {
    pub tables: Vec<DiagramTable>,
    pub relations: Vec<DiagramRelation>,
}

fn did(qualified: &str) -> String {
    qualified
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Construye el modelo de diagrama a partir de un esquema.
pub fn to_diagram_model(schema: &Schema) -> DiagramModel {
    let mut tables = Vec::new();
    for t in &schema.tables {
        let qn = t.qualified_name();
        let fk_cols: BTreeSet<&str> = schema
            .foreign_keys
            .iter()
            .filter(|f| f.from_table == qn)
            .flat_map(|f| f.from_columns.iter().map(|s| s.as_str()))
            .collect();

        let mut cols: Vec<_> = t.columns.iter().collect();
        cols.sort_by_key(|c| c.ordinal);
        let columns = cols
            .iter()
            .map(|c| {
                let mut key = String::new();
                if c.is_primary_key {
                    key.push_str("PK");
                }
                if fk_cols.contains(c.name.as_str()) {
                    if !key.is_empty() {
                        key.push('·');
                    }
                    key.push_str("FK");
                }
                DiagramColumn { name: c.name.clone(), data_type: c.data_type.clone(), key }
            })
            .collect();

        tables.push(DiagramTable { id: did(&qn), name: qn, columns });
    }

    let ids: BTreeSet<String> = schema.tables.iter().map(|t| did(&t.qualified_name())).collect();
    let mut relations = Vec::new();
    for fk in &schema.foreign_keys {
        let (from, to) = (did(&fk.from_table), did(&fk.to_table));
        if ids.contains(&from) && ids.contains(&to) {
            let label = if fk.from_columns.is_empty() {
                fk.name.clone()
            } else {
                fk.from_columns.join(", ")
            };
            relations.push(DiagramRelation { from, to, label });
        }
    }

    DiagramModel { tables, relations }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Column, ForeignKey, Schema, Table};

    fn col(name: &str, ty: &str, pk: bool, ord: i32) -> Column {
        Column { name: name.into(), data_type: ty.into(), is_nullable: true, default_value: None, ordinal: ord, is_primary_key: pk }
    }

    #[test]
    fn modelo_con_pk_fk_y_relacion() {
        let clientes = Table { schema: "dbo".into(), name: "Clientes".into(), columns: vec![col("id", "int", true, 1)] };
        let pedidos = Table {
            schema: "dbo".into(),
            name: "Pedidos".into(),
            columns: vec![col("id", "int", true, 1), col("cliente_id", "int", false, 2)],
        };
        let fk = ForeignKey {
            name: "fk".into(),
            from_table: "dbo.Pedidos".into(),
            from_columns: vec!["cliente_id".into()],
            to_table: "dbo.Clientes".into(),
            to_columns: vec!["id".into()],
        };
        let s = Schema { tables: vec![clientes, pedidos], foreign_keys: vec![fk], ..Default::default() };
        let d = to_diagram_model(&s);

        assert_eq!(d.tables.len(), 2);
        let ped = d.tables.iter().find(|t| t.name == "dbo.Pedidos").unwrap();
        assert_eq!(ped.columns.iter().find(|c| c.name == "id").unwrap().key, "PK");
        assert_eq!(ped.columns.iter().find(|c| c.name == "cliente_id").unwrap().key, "FK");
        assert_eq!(d.relations.len(), 1);
        assert_eq!(d.relations[0].from, "dbo_Pedidos");
        assert_eq!(d.relations[0].to, "dbo_Clientes");
    }
}
