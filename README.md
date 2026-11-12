# Schemir

> Cross-platform **database schema & data** comparator — multi-engine and open source.
> A free alternative to **SSDT Schema Compare**, built with **Rust + Tauri**.

[![CI](https://github.com/marlontaro/Schemir/actions/workflows/ci.yml/badge.svg)](https://github.com/marlontaro/Schemir/actions/workflows/ci.yml)
[![Latest version](https://img.shields.io/github/v/release/marlontaro/Schemir?label=version&color=blue)](https://github.com/marlontaro/Schemir/releases/latest)
![License](https://img.shields.io/badge/License-MIT-green.svg)
![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri_2-24C8DB?logo=tauri&logoColor=white)
![Platforms](https://img.shields.io/badge/macOS_·_Windows_·_Linux-informational)

**Schemir** connects to two databases (A = source, B = target), compares their **objects**
(structure) and their **data**, and shows you what is extra, missing or different between A
and B. It generates the **sync script** to make B match A, draws an interactive
**entity-relationship diagram**, and includes a **designer** to build models and export them
to SQL and Prisma.

The name is **Schema + Mirror** — read two schemas, make one mirror the other.

---

## 📥 Download

Grab the installer for your OS from the [**latest release**](https://github.com/marlontaro/Schemir/releases/latest):

| OS | Files |
|---|---|
| 🍎 **macOS** (Apple Silicon) | `.dmg` |
| 🪟 **Windows** | `.msi` · `.exe` |
| 🐧 **Linux** | `.AppImage` · `.deb` · `.rpm` |

> The macOS build is currently unsigned — right-click → **Open** the first time.

---

## 📸 Screenshots

**Object comparison** — summary by status and per-object detail:

![Object comparison](docs/img/comparacion.png)

**Difference diagram** — interactive (zoom, pan, drag, fullscreen):

![ER difference diagram](docs/img/diagrama.png)

**ER designer** — build a model and export it to SQL / Prisma:

![ER diagram designer](docs/img/disenador.png)

---

## ✨ Features

**Compare**
- 🔌 **Three engines:** SQL Server, PostgreSQL and SQLite.
- 🧱 **Every object:** tables and columns (type, nullability, *default*), views, procedures,
  functions, **primary & foreign keys, indexes, CHECK constraints and triggers**.
- 🎛️ **Comparison options** (SSDT-style): ignore case, whitespace, comments and column
  defaults; and choose **which object types** to compare.
- 🗂️ **Multi-schema:** reads and compares every schema (`dbo`, `public`, whatever you have).
- 📊 **Row-data comparison** (SQLite): matches by primary key and reports rows only in A /
  only in B / different, with their DML (`INSERT` / `UPDATE` / `DELETE`).

**Synchronize**
- 📝 **Sync script** (`CREATE` / `ALTER` / `DROP`), dialect-aware, with **selective sync**:
  check/uncheck which objects go in. Copy or save as `.sql`.
- ⚡ **Apply to target** from the app, with confirmation and inside a **transaction** (only
  the checked objects).

**Visualize & design**
- 🗺️ **ER diagram** with a custom renderer (dagre): **Field · Type · Key** headers, 🔑 PK /
  🔗 FK marks, relationships, and a **color-coded difference diagram**. Interactive (zoom,
  pan, drag tables, fullscreen) and **exportable to PNG / SVG**.
- 🎨 **ER designer:** build a model from scratch (tables, columns with PK/NOT NULL and
  relationships) and export it to **SQL** (SQL Server / PostgreSQL / SQLite) and **Prisma**.
- 📄 **Difference report** in HTML (printable to PDF) with summary, detail and script.

**Comfort**
- 💾 **Recent connections** and **save / open a comparison** (`.ddcmp`) — passwords are never
  stored.
- 🌐 **Bilingual ES / EN** (auto-detects your system language) and 🌗 **light / dark / system**
  theme.
- 🔒 Comparison is enabled only when **both connections** test OK; databases are opened
  **read-only** (never modified unless you press *Apply*).

## 🔌 Supported engines

| Engine | Driver | Schema | Data |
|---|---|:--:|:--:|
| **SQL Server** | `tiberius` | ✅ | — |
| **PostgreSQL** | `tokio-postgres` (TLS) | ✅ | — |
| **SQLite** | `rusqlite` (bundled) | ✅ | ✅ |

> **Data** comparison ships today for **SQLite** (covered by tests); the logic is
> engine-agnostic, so extending it to SQL Server / PostgreSQL is just adding their row reader.

---

## 🚀 Getting started (development)

Requirements: **Rust** (recent, for Tauri 2) and **Node** (for the Tauri CLI).

```bash
rustup update stable
```
```bash
npm install
```
```bash
npm run tauri dev
```

> The first build takes a few minutes (it downloads and compiles Tauri and the drivers).
> You can also run the binary directly: `cargo run --manifest-path src-tauri/Cargo.toml`.

### Quick test with SQLite (no server)

Create two sample databases with intentional differences and compare them (**SQLite** engine):

```bash
sqlite3 ejemplos/a.db "CREATE TABLE clientes(id INTEGER PRIMARY KEY, nombre TEXT NOT NULL, email VARCHAR(50)); CREATE TABLE pedidos(id INTEGER PRIMARY KEY, cliente_id INTEGER REFERENCES clientes(id), total REAL); CREATE INDEX ix_pedidos_cliente ON pedidos(cliente_id); CREATE VIEW v_resumen AS SELECT cliente_id, COUNT(*) n FROM pedidos GROUP BY cliente_id;"
sqlite3 ejemplos/b.db "CREATE TABLE clientes(id INTEGER PRIMARY KEY, nombre TEXT NOT NULL, email VARCHAR(100), activo INTEGER); CREATE TABLE pedidos(id INTEGER PRIMARY KEY, cliente_id INTEGER REFERENCES clientes(id), total REAL); CREATE UNIQUE INDEX ux_clientes_email ON clientes(email); CREATE TABLE auditoria(id INTEGER PRIMARY KEY, accion TEXT);"
```

---

## 🏗️ Architecture

The key to supporting several engines without duplicating logic: each engine reads its own
system catalogs and translates them into a **common model**. The diff, the script, the
diagram and the data comparison all work on that model alone.

```
  Driver (tiberius / tokio-postgres / rusqlite)
        │  reads the system catalogs
        ▼
  Introspection  →  Schema (normalized model, in schemir-core)
        │
        ├─▶ Diff engine        →  Only in A · Only in B · Different · Equal
        ├─▶ Sync script        →  CREATE / ALTER / DROP (per object, per dialect)
        ├─▶ Data comparison    →  INSERT / UPDATE / DELETE (by primary key)
        └─▶ Diagram model      →  entities + relationships
        ▼
  UI (Tauri + HTML/CSS/JS, bilingual and themeable)
```

### Project layout

```
Schemir/
├── crates/core/              # schemir-core: pure Rust logic (with tests)
│   └── src/
│       ├── model.rs          #   Schema, Table, Column, Routine, ForeignKey, Index, Trigger…
│       ├── diff.rs           #   compare_schemas_opts() + CompareOptions
│       ├── script.rs         #   generate_sync_units_opts() / generate_sync_script_opts()
│       ├── data.rs           #   compare_table_data() → row DML
│       └── diagram.rs        #   to_diagram_model() / to_diff_diagram_opts()
├── src-tauri/                # Tauri app (Rust backend)
│   └── src/
│       ├── main.rs           #   commands: compare_databases, compare_data, apply_script,
│       │                     #             test_connection, save/read_text_file, save_binary_file
│       └── db/
│           ├── mod.rs        #   ConnectionConfig + per-engine dispatch
│           ├── sqlserver.rs  #   SQL Server introspection (tiberius)
│           ├── postgres.rs   #   PostgreSQL introspection (tokio-postgres)
│           └── sqlite.rs     #   introspection + data reading (rusqlite)
├── src/                      # frontend (HTML/CSS/JS, no framework)
│   ├── index.html · styles.css · main.js
│   └── vendor/dagre.min.js   #   diagram layout
└── .github/workflows/        # CI (build + tests) and Release (installers)
```

### Tests

The core (model, diff, script, data, diagram) is tested without any database, and there are
integration tests that read real SQLite. **CI** runs them on every push.

```bash
cargo test                                      # core (schemir-core)
cargo test --manifest-path src-tauri/Cargo.toml # app + integration (SQLite)
```

---

## 🗺️ Roadmap & limitations

- **Data** comparison covers **SQLite** today (structure covers all three engines).
- **SQL Server and PostgreSQL** against real servers are not yet verified end-to-end: the
  code compiles and is tested with SQLite; the catalog queries may need tweaks against a real
  database.
- **Large tables:** data comparison currently loads up to **100k rows** into memory. Comparing
  tables with **millions of rows** is planned via **streaming comparison by primary key + row
  hashing** (constant memory, minimal data transfer) — schema comparison is unaffected.
- PostgreSQL uses TLS when the server supports it and **trusts its certificate**; enforcing CA
  validation is a planned option.
- In SQLite, adding or dropping a foreign key requires rebuilding the table, so the script
  marks it with a comment instead of emitting the `ALTER` (SQL Server and Postgres do emit it).

On the radar: row data for SQL Server / PostgreSQL, more objects (sequences, user-defined
types), and a signed macOS build.

---

## 🇪🇸 Español

**Schemir** es un comparador de **esquema y datos** de bases de datos, open source y
multiplataforma — una alternativa a SSDT Schema Compare hecha con **Rust + Tauri**. Compara
objetos (tablas, columnas, vistas, procedimientos, funciones, claves primarias/foráneas,
índices, restricciones CHECK y disparadores) en **SQL Server, PostgreSQL y SQLite**, con
opciones de comparación estilo SSDT. Genera un **script de sincronización** consciente del
dialecto con **aplicación selectiva**, compara **datos de filas** (SQLite) produciendo
`INSERT`/`UPDATE`/`DELETE`, dibuja un **diagrama ER interactivo** (exportable a PNG/SVG) e
incluye un **diseñador ER** que exporta a **SQL** y **Prisma**. La interfaz es bilingüe
(ES/EN, autodetecta el idioma del sistema) con tema claro/oscuro/sistema.

Descarga el instalador de tu sistema desde la [**última versión**](https://github.com/marlontaro/Schemir/releases/latest).

---

## 📄 License

[MIT](LICENSE) — © 2026 SOEN.
