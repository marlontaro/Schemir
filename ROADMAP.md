# Roadmap de DataDiff

Estado honesto del proyecto: qué está hecho, qué falta y qué está sin probar.
_Última actualización: noviembre 2026._

## ✅ Hecho

- **Tres motores:** SQL Server (`tiberius`), PostgreSQL (`tokio-postgres`, con TLS) y SQLite (`rusqlite`).
- **Comparación de objetos:** tablas, columnas, vistas, procedimientos, funciones,
  claves primarias, claves foráneas, índices y restricciones CHECK.
- **Multiesquema:** lee y compara todos los esquemas de la base.
- **Script de sincronización** (`CREATE` / `ALTER` / `DROP`) por dialecto, incluyendo
  índices, FK y CHECK. Con copiar y guardar `.sql`.
- **Diagrama ER** con renderizador propio (dagre): vista por esquema y **diagrama de
  diferencias** a color (solo en A / solo en B / diferente / igual).
- **Interfaz por fases** (Conexión → Comparación), **bilingüe ES/EN** y con
  **tema claro / oscuro / sistema**.
- **Núcleo probado:** 18 pruebas unitarias en `datadiff-core`.

## ⚠️ Pendiente de verificar

- **SQL Server y PostgreSQL contra servidores reales.** El código compila, pero el flujo
  completo solo se probó con SQLite (no había servidores disponibles al desarrollarlo).
  Las consultas a los catálogos pueden necesitar algún ajuste al conectar a una base real.

## 🔜 Próximo

- **Aplicar el script al destino:** ejecutar la migración desde la app, con confirmación
  (hoy solo se genera y se guarda el SQL).
- **Opciones de comparación:** ignorar comentarios, mayúsculas/minúsculas, orden de columnas…
- **Más objetos:** triggers, secuencias, tipos definidos por usuario, permisos.

## 🚀 Funcionalidad grande

- **Comparar datos de filas** (no solo estructura), al estilo SQL Data Compare.
- **Guardar / cargar una comparación** (como los `.scmp` de SSDT) y recordar conexiones recientes.
- **Validación de CA en TLS** (hoy se confía en el certificado del servidor).
- **Exportar** el diagrama (PNG/SVG) y un reporte de diferencias.

## 📦 Pulido y distribución

- **Instaladores** con `tauri build` → `.dmg` / `.msi` / `.AppImage`.
- **Capturas** en el README y una versión en inglés.
- **CI** (GitHub Actions) para compilar y correr los tests en cada push.

---

¿Ideas o prioridades distintas? Las sugerencias y contribuciones son bienvenidas. 🙌
