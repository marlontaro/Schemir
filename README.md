# DataDiff

> Comparador de **objetos** de base de datos — multiplataforma y multi-motor.
> Una alternativa open source a **SSDT Schema Compare**, hecha con **Rust + Tauri**.

![License](https://img.shields.io/badge/License-MIT-green.svg)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri_2-24C8DB?logo=tauri&logoColor=white)
![Plataformas](https://img.shields.io/badge/macOS_·_Windows_·_Linux-informational)

DataDiff conecta a dos bases de datos, compara sus **objetos** (estructura) y te
muestra qué sobra, qué falta y qué difiere entre A y B — además genera el **script
de sincronización** y un **diagrama entidad-relación**.

---

## ✨ Características

- 🔌 **Tres motores:** SQL Server, PostgreSQL y SQLite.
- 🧱 **Compara todos los objetos:** tablas y columnas (tipo, nulabilidad, *default*),
  vistas, procedimientos, funciones, **claves primarias y foráneas, índices y
  restricciones CHECK**.
- 📝 **Script de sincronización:** genera el SQL (`CREATE` / `ALTER` / `DROP`) para
  dejar B igual que A, consciente del dialecto de cada motor. Con copiar y guardar `.sql`.
- 🗺️ **Diagrama ER:** renderizador propio (dagre) con encabezados **Campo · Tipo · Clave**,
  marcas 🔑 PK / 🔗 FK y las relaciones dibujadas.
- 🗂️ **Multiesquema:** lee y compara todos los esquemas (`dbo`, `public`, los que tengas).
- 🌐 **Bilingüe ES / EN** y 🌗 **tema claro / oscuro / sistema**.
- 🔒 Habilita la comparación solo cuando **ambas conexiones** están probadas; selector
  de archivo nativo para SQLite; apertura en **solo lectura** (nunca modifica tus bases).

## 🔌 Motores soportados

| Motor | Driver | Notas |
|---|---|---|
| **SQL Server** | `tiberius` | Autenticación usuario/contraseña, TLS confiado en dev |
| **PostgreSQL** | `tokio-postgres` | Conexión sin TLS (desarrollo local) |
| **SQLite** | `rusqlite` (bundled) | Archivo local, sin servidor — ideal para probar |

---

## 🚀 Puesta en marcha (desarrollo)

Requisitos: **Rust** (reciente, para Tauri 2) y **Node** (para la CLI de Tauri).

```bash
rustup update stable
```
```bash
npm install
```
```bash
npm run tauri dev
```

> La primera compilación tarda varios minutos (baja y compila Tauri y los drivers).
> También puedes correr el binario directo: `cargo run --manifest-path src-tauri/Cargo.toml`.

### Probar rápido con SQLite (sin servidor)

Crea dos bases de ejemplo con una diferencia a propósito y compáralas (motor **SQLite**):

```bash
sqlite3 ejemplos/a.db "CREATE TABLE clientes(id INTEGER PRIMARY KEY, nombre TEXT NOT NULL, email VARCHAR(50)); CREATE TABLE pedidos(id INTEGER PRIMARY KEY, cliente_id INTEGER REFERENCES clientes(id), total REAL); CREATE INDEX ix_pedidos_cliente ON pedidos(cliente_id); CREATE VIEW v_resumen AS SELECT cliente_id, COUNT(*) n FROM pedidos GROUP BY cliente_id;"
sqlite3 ejemplos/b.db "CREATE TABLE clientes(id INTEGER PRIMARY KEY, nombre TEXT NOT NULL, email VARCHAR(100), activo INTEGER); CREATE TABLE pedidos(id INTEGER PRIMARY KEY, cliente_id INTEGER REFERENCES clientes(id), total REAL); CREATE UNIQUE INDEX ux_clientes_email ON clientes(email); CREATE TABLE auditoria(id INTEGER PRIMARY KEY, accion TEXT);"
```

---

## 🏗️ Arquitectura

La clave para soportar varios motores sin duplicar lógica: cada motor lee sus
catálogos del sistema y los traduce a un **modelo común**. El diff, el script y el
diagrama trabajan solo sobre ese modelo.

```
  Driver (tiberius / tokio-postgres / rusqlite)
        │  lee los catálogos del sistema
        ▼
  Introspección  →  Schema (modelo normalizado, en datadiff-core)
        │
        ├─▶ Motor de diff     →  Solo en A · Solo en B · Diferente · Igual
        ├─▶ Script de sync    →  CREATE / ALTER / DROP (por dialecto)
        └─▶ Modelo de diagrama →  entidades + relaciones
        ▼
  Interfaz (Tauri + HTML/CSS/JS, bilingüe y con temas)
```

### Estructura del proyecto

```
DataDiff/
├── crates/core/              # datadiff-core: lógica pura en Rust (con tests)
│   └── src/
│       ├── model.rs          #   Schema, Table, Column, Routine, ForeignKey, Index…
│       ├── diff.rs           #   compare_schemas()
│       ├── script.rs         #   generate_sync_script() por dialecto
│       └── diagram.rs        #   to_diagram_model()
├── src-tauri/                # app Tauri (backend Rust)
│   └── src/
│       ├── main.rs           #   comandos: compare_databases, test_connection, save_text_file
│       └── db/
│           ├── mod.rs        #   ConnectionConfig + despacho por motor
│           ├── sqlserver.rs  #   introspección SQL Server (tiberius)
│           ├── postgres.rs   #   introspección PostgreSQL (tokio-postgres)
│           └── sqlite.rs     #   introspección SQLite (rusqlite)
└── src/                      # frontend (HTML/CSS/JS, sin framework)
    ├── index.html · styles.css · main.js
    └── vendor/dagre.min.js   #   layout del diagrama
```

### Tests

El núcleo (modelo, diff, script, diagrama) se prueba sin necesidad de ninguna base:

```bash
cargo test
```

---

## 🗺️ Limitaciones y futuro

- Compara **estructura** de objetos, no los datos de las filas (por diseño).
- SQLite no tiene procedimientos ni funciones almacenadas (esas listas quedan vacías).
- La conexión a PostgreSQL es **sin TLS** (pensada para local); falta el modo con SSL.
- El script de sincronización cubre tablas y rutinas; índices/constraints se **comparan**
  pero aún no se incluyen en el script generado.

Ideas en el radar: TLS para Postgres, índices/constraints en el script, diagrama que
resalta las diferencias a color, aplicar cambios al destino, y guardar/cargar la comparación.

---

## 📄 Licencia

[MIT](LICENSE) — © 2026 SOEN.
