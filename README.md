# DataDiff

Comparador de **objetos** de base de datos multiplataforma (macOS, Windows, Linux).
Construido con **Rust + Tauri**. Compara esquemas y te muestra qué objetos sobran,
faltan o difieren entre dos bases: **tablas, columnas, vistas, procedimientos y funciones**.

> Estado: MVP inicial. Motores soportados: **SQL Server** y **SQLite**. PostgreSQL es el próximo.
> Por ahora compara solo **objetos** (estructura), no los datos de las filas.

---

## Arquitectura

La clave para soportar varios motores sin duplicar lógica: cada motor lee sus
catálogos del sistema y los traduce a un **modelo común**. El diff y la interfaz
trabajan solo sobre ese modelo.

```
  Driver del motor (tiberius / rusqlite / …)
        │  lee catálogos del sistema
        ▼
  Introspección  →  Schema (modelo normalizado, en datadiff-core)
        │
        ▼
  Motor de diff  →  Solo en A · Solo en B · Diferente · Igual
        │
        ▼
  Interfaz (Tauri + HTML/CSS/JS)
```

### Estructura del proyecto

```
ComparadorBaseDatos/
├── crates/core/          # datadiff-core: modelo + motor de diff (puro Rust, con tests)
│   └── src/
│       ├── model.rs      #   Schema, Table, Column, Routine…
│       └── diff.rs       #   compare_schemas() y sus tests
├── src-tauri/            # app Tauri (backend Rust)
│   ├── src/
│   │   ├── main.rs       #   comandos: compare_databases, test_connection
│   │   └── db/
│   │       ├── mod.rs    #   ConnectionConfig + despacho por motor
│   │       ├── sqlite.rs #   introspección SQLite (rusqlite)
│   │       └── sqlserver.rs # introspección SQL Server (tiberius)
│   └── tauri.conf.json
└── src/                  # frontend (HTML/CSS/JS, sin framework)
    ├── index.html
    ├── styles.css
    └── main.js
```

---

## Puesta en marcha

### 1. Actualizar Rust (necesario para Tauri 2)

Este equipo tiene Rust 1.72 (2023); Tauri 2 y sus dependencias necesitan una versión más nueva:

```bash
rustup update stable
```

> Si tu Rust vino de Homebrew en vez de rustup: `brew upgrade rust`.

### 2. Instalar la CLI de Tauri (vía npm, sin compilar)

```bash
npm install
```

### 3. Ejecutar la app en modo desarrollo

```bash
npm run tauri dev
```

La primera compilación tarda varios minutos (baja y compila Tauri, tiberius y SQLite).

---

## Probar el núcleo (sin base de datos)

El motor de comparación tiene tests y no necesita ninguna base:

```bash
cargo test
```

## Probar rápido con SQLite (sin servidor)

Crea dos bases de ejemplo y compáralas desde la app (motor **SQLite**, pegando las rutas):

```bash
sqlite3 /tmp/a.db "CREATE TABLE clientes(id INTEGER, email TEXT); CREATE VIEW v AS SELECT 1;"
sqlite3 /tmp/b.db "CREATE TABLE clientes(id INTEGER, email VARCHAR(100), activo INTEGER);"
```

---

## Cómo agregar PostgreSQL (próximo paso)

1. Añadir `Postgres` a `EngineKind` en `src-tauri/src/db/mod.rs`.
2. Crear `src-tauri/src/db/postgres.rs` con `read_schema()` y `test_connection()`
   (usando `sqlx` o `tokio-postgres`), leyendo de `information_schema` / `pg_catalog`.
3. Despacharlo en `read_schema` / `test_connection`.
4. Agregar la opción "PostgreSQL" al selector de motor en `src/index.html`.

El modelo y el motor de diff **no cambian**.

---

## Limitaciones actuales

- Compara estructura de objetos, no datos de filas (por diseño, por ahora).
- SQLite no tiene procedimientos ni funciones almacenadas (esas listas quedan vacías).
- Comparar entre motores distintos (p.ej. SQL Server vs SQLite) funciona, pero los
  nombres de tipo difieren (`int` vs `INTEGER`), así que habrá diferencias esperables.
  El caso principal es comparar dos bases del **mismo** motor.
