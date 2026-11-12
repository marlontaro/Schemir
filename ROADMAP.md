# Roadmap de DataDiff

Estado honesto del proyecto: qué está hecho, qué falta y qué está sin probar.
_Última actualización: noviembre 2026._

## ✅ Hecho

- **Tres motores:** SQL Server (`tiberius`), PostgreSQL (`tokio-postgres`, con TLS) y SQLite (`rusqlite`).
- **Comparación de objetos:** tablas, columnas, vistas, procedimientos, funciones,
  claves primarias, claves foráneas, índices, restricciones CHECK y **disparadores (triggers)**.
- **Multiesquema:** lee y compara todos los esquemas de la base.
- **Opciones de comparación** (estilo SSDT): ignorar mayúsculas/minúsculas, espacios,
  comentarios y el valor por defecto de columnas, y elegir qué tipos de objeto comparar.
- **Script de sincronización** (`CREATE` / `ALTER` / `DROP`) por dialecto, con
  **sincronización selectiva** (marcar/desmarcar qué objetos entran). Copiar y guardar `.sql`.
- **Aplicar al destino:** ejecutar el script (solo lo marcado) contra la base B desde la
  app, con confirmación y en una transacción (verificado en SQLite).
- **Diagrama ER** con renderizador propio (dagre): vista por esquema y **diagrama de
  diferencias** a color. Interactivo (zoom, desplazamiento, arrastrar, pantalla completa)
  y **exportable a PNG/SVG**.
- **Informe de diferencias** en HTML (imprimible a PDF) con resumen, detalle y script.
- **Conexiones recientes:** recuerda las conexiones probadas (sin la contraseña).
- **Interfaz por fases** (Conexión → Comparación), **bilingüe ES/EN** y con
  **tema claro / oscuro / sistema**.
- **Instalador para macOS** (`.dmg`) generado con `tauri build`.
- **Pruebas:** 25 unitarias en `datadiff-core` + 1 de integración del lector SQLite.

## ⚠️ Pendiente de verificar

- **SQL Server y PostgreSQL contra servidores reales.** El código compila, pero el flujo
  completo solo se probó con SQLite (no había servidores disponibles al desarrollarlo).
  Las consultas a los catálogos pueden necesitar algún ajuste al conectar a una base real.

## 🔜 Próximo

- **Más objetos:** secuencias, tipos definidos por usuario, permisos.

## 🚀 Funcionalidad grande

- **Comparar datos de filas** (no solo estructura), al estilo SQL Data Compare.
- **Guardar / cargar una comparación** completa (como los `.scmp` de SSDT).
- **Validación de CA en TLS** (hoy se confía en el certificado del servidor).
- **Diseñador de diagramas ER:** crear y editar el modelo visualmente y exportarlo a
  SQL, Prisma, etc. (hoy el diagrama es de solo lectura).

## 📦 Pulido y distribución

- **Instaladores** para Windows (`.msi`) y Linux (`.AppImage`) — el `.dmg` de macOS ya está.
- **Capturas** en el README y una versión en inglés.
- **CI** (GitHub Actions) para compilar y correr los tests en cada push.

---

¿Ideas o prioridades distintas? Las sugerencias y contribuciones son bienvenidas. 🙌
