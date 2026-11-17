# Roadmap de Schemir

Estado honesto del proyecto: qué está hecho, qué falta y qué está sin probar.
_Última actualización: noviembre 2026._

## ✅ Hecho

- **Cuatro motores:** SQL Server (`tiberius`), PostgreSQL (`tokio-postgres`, con TLS),
  MySQL / MariaDB (`mysql_async`, con TLS si el servidor lo ofrece) y SQLite (`rusqlite`).
  MySQL y MariaDB hablan el mismo protocolo y publican los mismos catálogos, así que un solo
  lector cubre los dos; en ellos el esquema ES la base de datos.
- **Comparación de objetos:** tablas, columnas, vistas, procedimientos, funciones,
  claves primarias, claves foráneas, índices, restricciones CHECK y **disparadores (triggers)**.
- **Multiesquema:** lee y compara todos los esquemas de la base.
- **Opciones de comparación** (estilo SSDT): ignorar mayúsculas/minúsculas, espacios,
  comentarios y el valor por defecto de columnas, y elegir qué tipos de objeto comparar.
- **Script de sincronización** (`CREATE` / `ALTER` / `DROP`) por dialecto, con
  **sincronización selectiva** (marcar/desmarcar qué objetos entran). Copiar y guardar `.sql`.
- **Aplicar al destino:** ejecutar el script (solo lo marcado) contra la base B desde la
  app, con confirmación y en una transacción (verificado en SQLite). En MySQL / MariaDB va
  sentencia a sentencia y SIN transacción: el motor hace *commit* implícito en cada DDL.
- **Comparar datos de filas** (SQLite): por tabla, empareja por clave primaria y reporta
  filas solo en A / solo en B / distintas, con su DML (`INSERT` / `UPDATE` / `DELETE`).
- **Diagrama ER** con renderizador propio (dagre): vista por esquema y **diagrama de
  diferencias** a color. Interactivo (zoom, desplazamiento, arrastrar, pantalla completa)
  y **exportable a PNG/SVG**.
- **Informe de diferencias** en HTML (imprimible a PDF) con resumen, detalle y script.
- **Diseñador de diagramas ER:** crea un modelo visualmente (tablas, columnas con
  PK/NOT NULL y relaciones) y expórtalo a **SQL** (SQL Server / PostgreSQL / MySQL / SQLite)
  y **Prisma**.
- **Conexiones recientes** y **guardar / abrir una comparación** (`.ddcmp`), ambas sin
  guardar contraseñas.
- **Interfaz por fases** (Conexión → Comparación), **bilingüe ES/EN** y con
  **tema claro / oscuro / sistema**.
- **Instalador para macOS** (`.dmg`) generado con `tauri build`.
- **Pruebas:** 51 unitarias en `schemir-core` + 9 del lector MySQL (entrecomillado, valores
  por defecto, troceo del script) + 2 de integración del lector SQLite.

## ⚠️ Pendiente de verificar

- **SQL Server, PostgreSQL y MySQL / MariaDB contra servidores reales.** El código compila y
  la parte que no necesita servidor está cubierta por tests, pero el flujo completo solo se
  probó con SQLite (no había servidores disponibles al desarrollarlo). Las consultas a los
  catálogos pueden necesitar algún ajuste al conectar a una base real.
- En MySQL / MariaDB, tres puntos concretos conviene mirarlos con un servidor delante:
  el `CREATE` de procedimientos y funciones se rearma a partir de `ROUTINES` + `PARAMETERS`
  (no de `SHOW CREATE`); los índices por expresión se leen con una marca en lugar de la
  expresión, porque `information_schema.STATISTICS` deja la columna en NULL; y el
  `AUTO_INCREMENT` no se lee, igual que no se lee el `IDENTITY` de SQL Server (el modelo
  común todavía no tiene dónde guardarlo).

## 🔜 Próximo

- **Más objetos:** secuencias, tipos definidos por usuario, permisos.

## 🚀 Funcionalidad grande

- **Comparar datos de filas en SQL Server, PostgreSQL y MySQL / MariaDB** (hoy solo SQLite).
- **Validación de CA en TLS** (hoy se confía en el certificado del servidor).

## 📦 Pulido y distribución

- **Instaladores** para Windows (`.msi`) y Linux (`.AppImage`) — el `.dmg` de macOS ya está.
- **Capturas** en el README y una versión en inglés.
- **CI** (GitHub Actions) para compilar y correr los tests en cada push.

---

¿Ideas o prioridades distintas? Las sugerencias y contribuciones son bienvenidas. 🙌
