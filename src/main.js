// ---------------------------------------------------------------------------
// Schemir — lógica de la interfaz (bilingüe ES/EN).
// Backend (Tauri): test_connection, compare_databases -> {comparison, script, diagram_a, diagram_b}, save_text_file.
// ---------------------------------------------------------------------------

const SVGNS = "http://www.w3.org/2000/svg";

// ===========================================================================
// i18n — diccionario de recursos (equivalente ligero a los .resx de .NET)
// ===========================================================================
const I18N = {
  es: {
    "app.subtitle": "Comparador de objetos de base de datos",
    "app.tagline": "alternativa a SSDT",
    "theme.light": "Claro",
    "theme.dark": "Oscuro",
    "theme.system": "Sistema",
    "mode.compare": "Comparar",
    "mode.design": "Diseñar",
    "dz.addTable": "+ Tabla",
    "dz.dialect": "Dialecto",
    "dz.exportSql": "Exportar SQL",
    "dz.exportPrisma": "Exportar Prisma",
    "dz.clear": "Limpiar",
    "dz.clearConfirm": "¿Borrar todo el modelo del diseñador?",
    "dz.empty": "Añade una tabla para empezar tu modelo.",
    "dz.relations": "Relaciones (claves foráneas)",
    "dz.treeHead": "Modelo",
    "dz.viewLogical": "Lógico",
    "dz.viewPhysical": "Físico",
    "dz.viewLogicalHint": "Vista lógica: nombres de negocio, sin tipos de motor",
    "dz.logicalName": "Nombre lógico",
    "dz.logicalCol": "lógico",
    "dz.sameAsPhysical": "igual que el físico",
    "dz.color": "Color",
    "dz.cNone": "sin color",
    "dz.cBlue": "azul",
    "dz.cCyan": "cian",
    "dz.cGreen": "verde",
    "dz.cAmber": "ámbar",
    "dz.cRed": "rojo",
    "dz.cViolet": "violeta",
    "dz.cGray": "gris",
    "dz.toolSelect": "Seleccionar",
    "dz.toolTable": "Tabla nueva · pulsa en el lienzo",
    "dz.toolIdent": "Relación identificadora · la clave entra en la primaria de la hija",
    "dz.toolNoIdent": "Relación no identificadora · la clave queda fuera de la primaria",
    "dz.toolM2M": "Muchos a muchos · crea la tabla puente",
    "dz.toolHint": "Arrastra del padre a la hija",
    "dz.relType": "Tipo de relación",
    "dz.typeIdent": "identificadora (la clave entra en la primaria)",
    "dz.typeNoIdent": "no identificadora (la clave queda fuera)",
    "dz.bridgeMade": "{t}: tabla puente creada con las dos claves.",
    "dz.treeTables": "Tablas",
    "dz.treeRels": "Relaciones",
    "dz.treeSchemas": "Esquemas",
    "dz.treeEmpty": "El modelo está vacío.",
    "dz.stCols": "{n} columnas",
    "dz.stCol1": "1 columna",
    "dz.stRels": "{n} relaciones",
    "dz.stRel1": "1 relación",
    "dz.stTables": "{n} tablas",
    "dz.stTable1": "1 tabla",
    "dz.addRel": "Añadir relación",
    "dz.addCol": "columna",
    "dz.colName": "nombre",
    "dz.colType": "tipo",
    "dz.thAuto": "valor automático",
    "dz.delTable": "Eliminar tabla",
    "dz.save": "Guardar",
    "dz.sqlTitle": "SQL del modelo",
    "dz.prismaTitle": "Esquema Prisma",
    "dz.schema": "Esquema",
    "dz.tableTitle": "Tabla",
    "dz.tableName": "Nombre de la tabla",
    "dz.editCols": "Editar columnas",
    "dz.editHint": "Doble clic para editar las columnas",
    "dz.close": "Cerrar",
    "dz.up": "Subir",
    "dz.down": "Bajar",
    "dz.delCol": "Quitar la columna",
    "dz.noCols": "Sin columnas.",
    "dz.autoId": "contador que lleva el motor",
    "dz.autoUuid": "identificador único que genera el motor",
    "dz.autoHint": "Valor que pone el motor al insertar la fila",
    "dz.move": "Arrastra para mover la tabla",
    "dz.resize": "Tira para cambiar el tamaño · doble clic para el natural",
    "dz.openModel": "Abrir modelo",
    "dz.saveModel": "Guardar modelo",
    "dz.openConfirm": "\u00bfAbrir el modelo del archivo? Se reemplaza el que tienes en pantalla.",
    "dz.badXml": "El archivo no es XML v\u00e1lido.",
    "dz.notModel": "El archivo no es un modelo de Schemir (falta el elemento schemir-model).",
    "dz.emptyModel": "El modelo del archivo no tiene ninguna tabla.",
    "dz.addSchema": "+ Esquema",
    "dz.newSchema": "+ nuevo esquema\u2026",
    "dz.newSchemaTitle": "Nuevo esquema",
    "dz.schemaName": "Nombre del esquema",
    "dz.schemaOf": "Esquema al que pertenece la tabla",
    "dz.version": "Versi\u00f3n objetivo",
    "dz.versionNeeds": "Este modelo corre en {m} {r} o superior.",
    "dz.versionTooOld": "Elegiste {m} {v}, pero el modelo exige {r} o superior.",
    "dz.needsVersion": "{x} existe desde {v}, m\u00e1s nuevo que la versi\u00f3n elegida.",
    "dz.deprecated": "Desaconsejado: usa {x}.",
    "dz.overMax": "El motor admite hasta {n}.",
    "dz.typeOther": "otro\u2026",
    "dz.length": "long.",
    "dz.precision": "prec.",
    "dz.scale": "esc.",
    "dz.maxHint": "Sin l\u00edmite de longitud: varchar(max)",
    "dz.dragHint": "Arrastra de esta clave a la tabla que la hereda",
    "dz.relTitle": "Relaci\u00f3n (clave for\u00e1nea)",
    "dz.relName": "Nombre de la relaci\u00f3n",
    "dz.relMigrate": "{p} baja a {t} como",
    "dz.onDelete": "Al borrar el padre",
    "dz.onUpdate": "Al actualizar el padre",
    "dz.relEdit": "Pulsa para cambiarla",
    "dz.relHint": "Arrastra del punto de la clave del padre a la tabla hija: la clave entera baja sola, aunque sea compuesta.",
    "dz.noRels": "Todav\u00eda no hay ninguna relaci\u00f3n.",
    "dz.cat.number": "N\u00fameros",
    "dz.cat.text": "Texto",
    "dz.cat.date_time": "Fecha y hora",
    "dz.cat.binary": "Binario",
    "dz.cat.boolean": "Booleano",
    "dz.cat.other": "Otros",
    "btn.ok": "Aceptar",
    "btn.cancel": "Cancelar",
    "banner.noBackend": "⚠️ Esta interfaz necesita el backend. Ejecútala con cargo run o npm run tauri dev.",
    "card.origin": "Origen",
    "card.dest": "Destino",
    "field.server": "Servidor",
    "field.port": "Puerto",
    "field.database": "Base de datos",
    "field.user": "Usuario",
    "field.password": "Contraseña",
    "field.winAuth": "Autenticación de Windows",
    "hint.sqlserver": "Con el puerto por defecto, Schemir se conecta como SSMS (memoria compartida o canalizaciones si el servidor es local). Escribe otro puerto solo si quieres forzar TCP.",
    "field.sqliteFile": "Archivo SQLite",
    "ph.database": "ventas",
    "ph.sqlitePath": "/ruta/a/base.db",
    "hint.sqlite": "Elige el archivo .db / .sqlite con «Examinar…» o pega la ruta.",
    "hint.mysql": "En MySQL / MariaDB el esquema ES la base de datos: se compara la que escribas aquí.",
    "btn.browse": "Examinar…",
    "btn.test": "Probar conexión",
    "conn.recent": "Recientes…",
    "conn.recentTitle": "Conexiones recientes (sin contraseña)",
    "btn.compare": "Comparar objetos",
    "btn.comparing": "Comparando…",
    "btn.copy": "Copiar",
    "btn.copied": "¡Copiado!",
    "btn.saveSql": "Guardar .sql",
    "status.testing": "Probando…",
    "compare.hint": "Prueba las conexiones de A y B para habilitar la comparación.",
    "cmp.open": "Abrir comparación…",
    "cmp.save": "Guardar comparación",
    "cmp.loaded": "✓ Comparación cargada. Reconecta (contraseñas) y compara.",
    "cmp.invalid": "El archivo no es una comparación de Schemir válida.",
    "opt.title": "Opciones de comparación",
    "opt.ignore": "Ignorar",
    "opt.case": "Mayúsculas/minúsculas",
    "opt.whitespace": "Espacios en blanco",
    "opt.comments": "Comentarios",
    "opt.colDefault": "Valor por defecto de columnas",
    "opt.compare": "Comparar",
    "opt.tables": "Tablas",
    "opt.views": "Vistas",
    "opt.procedures": "Procedimientos",
    "opt.functions": "Funciones",
    "opt.indexes": "Índices",
    "opt.fks": "Claves foráneas",
    "opt.checks": "Restricciones CHECK",
    "opt.pks": "Claves primarias",
    "opt.triggers": "Disparadores",
    "opt.safety": "Seguridad",
    "opt.blockDataLoss": "Bloquear ante posible pérdida de datos",
    "opt.blockDataLossHelp":
      "Antepone al script una comprobación que lo detiene si la tabla tiene filas, antes de quitar una columna o una tabla o de cambiar un tipo.",
    "step.connection": "Conexión",
    "step.comparison": "Comparación",
    "btn.back": "← Conexión",
    "tab.diff": "Diferencias",
    "tab.script": "Script de sincronización",
    "tab.data": "Datos de filas",
    "tab.diagram": "Diagrama",
    "data.table": "Tabla",
    "btn.compareData": "Comparar datos",
    "data.comparing": "Comparando…",
    "data.leftRows": "Filas solo en el origen",
    "data.rightRows": "Filas solo en el destino",
    "data.diffRows": "Filas distintas",
    "data.equalRows": "Filas iguales",
    "data.opInsert": "Nueva",
    "data.opUpdate": "Cambiada",
    "data.opDelete": "Sobrante",
    "data.gridMore": "Mostrando {0} de {1} filas con diferencias (el DML completo está abajo).",
    "data.wasLabel": "antes",
    "data.info": "DML para dejar los datos de B iguales que A · revísalo antes de ejecutarlo.",
    "data.none": "-- Sin diferencias de datos.",
    "data.noTables": "No hay tablas en ambas bases.",
    "data.hint": "Elige una tabla y pulsa «Comparar datos».",
    "search.ph": "Buscar objeto…",
    "empty.filter": "No hay objetos que coincidan con el filtro.",
    "script.info": "SQL para dejar <b class=\"side-b-text\">B</b> igual que <b class=\"side-a-text\">A</b> · revísalo antes de ejecutarlo.",
    "script.none": "-- (sin cambios)",
    "btn.apply": "Aplicar a B",
    "apply.confirm": "¿Aplicar el script a la base de datos de destino? Esto la MODIFICA (crea, altera y elimina objetos). Haz un respaldo antes.",
    "apply.applying": "Aplicando…",
    "apply.done": "✓ Aplicado a B. Vuelve a comparar para confirmar.",
    "diagram.legend": "Diagrama entidad-relación · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Diferencias",
    "diagram.sideA": "Origen",
    "diagram.sideB": "Destino",
    "diagram.empty": "No hay tablas para diagramar en este lado.",
    "diagram.noEngine": "No se pudo cargar el motor de diagrama.",
    "zoom.in": "Acercar",
    "zoom.out": "Alejar",
    "zoom.fit": "Ajustar a la vista",
    "zoom.full": "Pantalla completa",
    "btn.exportPng": "Exportar PNG",
    "btn.exportSvg": "Exportar SVG",
    "export.png": "Guardar el diagrama como imagen PNG",
    "export.svg": "Guardar el diagrama como SVG (vectorial)",
    "btn.report": "Exportar informe",
    "report.hint": "Guardar un informe HTML (imprimible a PDF)",
    "report.title": "Informe de comparación",
    "report.generated": "Generado",
    "report.objects": "Objetos con diferencias",
    "report.script": "Script de sincronización",
    "report.noChanges": "No se encontraron diferencias.",
    "sel.syncAll": "Sincronizar todo",
    "sel.count": "{0} de {1} objetos",
    "sel.include": "Incluir en la sincronización",
    "script.noneSel": "(no hay objetos marcados para sincronizar)",
    "apply.nothing": "No hay objetos marcados para aplicar.",
    "diag.field": "Campo",
    "diag.type": "Tipo de dato",
    "diag.key": "Clave",
    "detail.field": "Campo",
    "detail.a": "Origen",
    "detail.b": "Destino",
    "status.only_in_left": "Solo en el origen",
    "status.only_in_right": "Solo en el destino",
    "status.different": "Diferente",
    "status.equal": "Igual",
    "type.table": "Tabla",
    "type.view": "Vista",
    "type.procedure": "Procedimiento",
    "type.function": "Función",
    "type.primary_key": "Clave primaria",
    "type.index": "Índice",
    "type.foreign_key": "Clave foránea",
    "type.check": "Restricción",
    "type.trigger": "Disparador",
    "err.noBackend": "Sin backend: abre la app con cargo run o npm run tauri dev (en un navegador normal no hay conexión a las bases).",
    "err.needDatabase": "Elige la base de datos: sin ella la conexión entra en la que el servidor tenga por defecto, que casi nunca es la que quieres comparar.",
    "status.pickDb": "Conectado al servidor · {n} bases disponibles: elige una en «Base de datos».",
    "status.noDbs": "Conectado al servidor, pero no hay ninguna base que puedas abrir.",
    "status.nDbs": "{n} bases",
    "db.connectFirst": "Conecta al servidor para ver las bases",
    "db.pick": "— elige una base —",
    "err.needPath": "Elige el archivo SQLite.",
  },
  en: {
    "app.subtitle": "Database object comparator",
    "app.tagline": "SSDT alternative",
    "theme.light": "Light",
    "theme.dark": "Dark",
    "theme.system": "System",
    "mode.compare": "Compare",
    "mode.design": "Design",
    "dz.addTable": "+ Table",
    "dz.dialect": "Dialect",
    "dz.exportSql": "Export SQL",
    "dz.exportPrisma": "Export Prisma",
    "dz.clear": "Clear",
    "dz.clearConfirm": "Clear the whole designer model?",
    "dz.empty": "Add a table to start your model.",
    "dz.relations": "Relations (foreign keys)",
    "dz.treeHead": "Model",
    "dz.viewLogical": "Logical",
    "dz.viewPhysical": "Physical",
    "dz.viewLogicalHint": "Logical view: business names, no engine types",
    "dz.logicalName": "Logical name",
    "dz.logicalCol": "logical",
    "dz.sameAsPhysical": "same as physical",
    "dz.color": "Colour",
    "dz.cNone": "no colour",
    "dz.cBlue": "blue",
    "dz.cCyan": "cyan",
    "dz.cGreen": "green",
    "dz.cAmber": "amber",
    "dz.cRed": "red",
    "dz.cViolet": "violet",
    "dz.cGray": "grey",
    "dz.toolSelect": "Select",
    "dz.toolTable": "New table · click on the canvas",
    "dz.toolIdent": "Identifying relationship · the key joins the child’s primary key",
    "dz.toolNoIdent": "Non-identifying relationship · the key stays out of the primary key",
    "dz.toolM2M": "Many to many · creates the bridge table",
    "dz.toolHint": "Drag from parent to child",
    "dz.relType": "Relationship type",
    "dz.typeIdent": "identifying (the key joins the primary key)",
    "dz.typeNoIdent": "non-identifying (the key stays out)",
    "dz.bridgeMade": "{t}: bridge table created with both keys.",
    "dz.treeTables": "Tables",
    "dz.treeRels": "Relations",
    "dz.treeSchemas": "Schemas",
    "dz.treeEmpty": "The model is empty.",
    "dz.stCols": "{n} columns",
    "dz.stCol1": "1 column",
    "dz.stRels": "{n} relations",
    "dz.stRel1": "1 relation",
    "dz.stTables": "{n} tables",
    "dz.stTable1": "1 table",
    "dz.addRel": "Add relation",
    "dz.addCol": "column",
    "dz.colName": "name",
    "dz.colType": "type",
    "dz.thAuto": "auto value",
    "dz.delTable": "Delete table",
    "dz.save": "Save",
    "dz.sqlTitle": "Model SQL",
    "dz.prismaTitle": "Prisma schema",
    "dz.schema": "Schema",
    "dz.tableTitle": "Table",
    "dz.tableName": "Table name",
    "dz.editCols": "Edit columns",
    "dz.editHint": "Double-click to edit the columns",
    "dz.close": "Close",
    "dz.up": "Move up",
    "dz.down": "Move down",
    "dz.delCol": "Remove the column",
    "dz.noCols": "No columns yet.",
    "dz.autoId": "counter kept by the engine",
    "dz.autoUuid": "unique id generated by the engine",
    "dz.autoHint": "Value the engine sets when the row is inserted",
    "dz.move": "Drag to move the table",
    "dz.resize": "Drag to resize · double-click for its natural size",
    "dz.openModel": "Open model",
    "dz.saveModel": "Save model",
    "dz.openConfirm": "Open the model from the file? It replaces the one on screen.",
    "dz.badXml": "The file is not valid XML.",
    "dz.notModel": "The file is not a Schemir model (no schemir-model element).",
    "dz.emptyModel": "The model in the file has no tables.",
    "dz.addSchema": "+ Schema",
    "dz.newSchema": "+ new schema\u2026",
    "dz.newSchemaTitle": "New schema",
    "dz.schemaName": "Schema name",
    "dz.schemaOf": "Schema the table belongs to",
    "dz.version": "Target version",
    "dz.versionNeeds": "This model runs on {m} {r} or newer.",
    "dz.versionTooOld": "You picked {m} {v}, but the model needs {r} or newer.",
    "dz.needsVersion": "{x} exists since {v}, newer than the version you picked.",
    "dz.deprecated": "Deprecated: use {x}.",
    "dz.overMax": "The engine allows up to {n}.",
    "dz.typeOther": "other\u2026",
    "dz.length": "len.",
    "dz.precision": "prec.",
    "dz.scale": "scale",
    "dz.maxHint": "No length limit: varchar(max)",
    "dz.dragHint": "Drag from this key onto the table that inherits it",
    "dz.relTitle": "Relation (foreign key)",
    "dz.relName": "Relation name",
    "dz.relMigrate": "{p} migrates to {t} as",
    "dz.onDelete": "On parent delete",
    "dz.onUpdate": "On parent update",
    "dz.relEdit": "Click to change it",
    "dz.relHint": "Drag from the parent key’s dot onto the child table: the whole key migrates, composite or not.",
    "dz.noRels": "No relations yet.",
    "dz.cat.number": "Numbers",
    "dz.cat.text": "Text",
    "dz.cat.date_time": "Date and time",
    "dz.cat.binary": "Binary",
    "dz.cat.boolean": "Boolean",
    "dz.cat.other": "Other",
    "btn.ok": "OK",
    "btn.cancel": "Cancel",
    "banner.noBackend": "⚠️ This UI needs the backend. Run it with cargo run or npm run tauri dev.",
    "card.origin": "Source",
    "card.dest": "Target",
    "field.server": "Server",
    "field.port": "Port",
    "field.database": "Database",
    "field.user": "User",
    "field.password": "Password",
    "field.winAuth": "Windows authentication",
    "hint.sqlserver": "With the default port, Schemir connects the way SSMS does (shared memory or named pipes when the server is local). Set another port only to force TCP.",
    "field.sqliteFile": "SQLite file",
    "ph.database": "sales",
    "ph.sqlitePath": "/path/to/db.db",
    "hint.sqlite": "Pick the .db / .sqlite file with “Browse…” or paste the path.",
    "hint.mysql": "In MySQL / MariaDB the schema IS the database: the one you type here is what gets compared.",
    "btn.browse": "Browse…",
    "btn.test": "Test connection",
    "conn.recent": "Recent…",
    "conn.recentTitle": "Recent connections (no password)",
    "btn.compare": "Compare objects",
    "btn.comparing": "Comparing…",
    "btn.copy": "Copy",
    "btn.copied": "Copied!",
    "btn.saveSql": "Save .sql",
    "status.testing": "Testing…",
    "compare.hint": "Test both A and B connections to enable the comparison.",
    "cmp.open": "Open comparison…",
    "cmp.save": "Save comparison",
    "cmp.loaded": "✓ Comparison loaded. Reconnect (passwords) and compare.",
    "cmp.invalid": "The file is not a valid Schemir comparison.",
    "opt.title": "Comparison options",
    "opt.ignore": "Ignore",
    "opt.case": "Case",
    "opt.whitespace": "Whitespace",
    "opt.comments": "Comments",
    "opt.colDefault": "Column default value",
    "opt.compare": "Compare",
    "opt.tables": "Tables",
    "opt.views": "Views",
    "opt.procedures": "Procedures",
    "opt.functions": "Functions",
    "opt.indexes": "Indexes",
    "opt.fks": "Foreign keys",
    "opt.checks": "CHECK constraints",
    "opt.pks": "Primary keys",
    "opt.triggers": "Triggers",
    "opt.safety": "Safety",
    "opt.blockDataLoss": "Block on possible data loss",
    "opt.blockDataLossHelp":
      "Prepends a check that stops the script if the table has rows, before dropping a column or a table or changing a type.",
    "step.connection": "Connection",
    "step.comparison": "Comparison",
    "btn.back": "← Connection",
    "tab.diff": "Differences",
    "tab.script": "Synchronization script",
    "tab.data": "Row data",
    "tab.diagram": "Diagram",
    "data.table": "Table",
    "btn.compareData": "Compare data",
    "data.comparing": "Comparing…",
    "data.leftRows": "Rows only in the source",
    "data.rightRows": "Rows only in the target",
    "data.diffRows": "Rows different",
    "data.equalRows": "Rows equal",
    "data.opInsert": "New",
    "data.opUpdate": "Changed",
    "data.opDelete": "Extra",
    "data.gridMore": "Showing {0} of {1} differing rows (full DML below).",
    "data.wasLabel": "was",
    "data.info": "DML to make B's data match A · review it before running.",
    "data.none": "-- No data differences.",
    "data.noTables": "No tables in both databases.",
    "data.hint": "Pick a table and click “Compare data”.",
    "search.ph": "Search object…",
    "empty.filter": "No objects match the filter.",
    "script.info": "SQL to make <b class=\"side-b-text\">B</b> match <b class=\"side-a-text\">A</b> · review it before running.",
    "script.none": "-- (no changes)",
    "btn.apply": "Apply to B",
    "apply.confirm": "Apply the script to the target database? This MODIFIES it (creates, alters and drops objects). Make a backup first.",
    "apply.applying": "Applying…",
    "apply.done": "✓ Applied to B. Compare again to confirm.",
    "diagram.legend": "Entity-relationship diagram · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Differences",
    "diagram.sideA": "Source",
    "diagram.sideB": "Target",
    "diagram.empty": "No tables to diagram on this side.",
    "diagram.noEngine": "Could not load the diagram engine.",
    "zoom.in": "Zoom in",
    "zoom.out": "Zoom out",
    "zoom.fit": "Reset view",
    "zoom.full": "Fullscreen",
    "btn.exportPng": "Export PNG",
    "btn.exportSvg": "Export SVG",
    "export.png": "Save the diagram as a PNG image",
    "export.svg": "Save the diagram as SVG (vector)",
    "btn.report": "Export report",
    "report.hint": "Save an HTML report (printable to PDF)",
    "report.title": "Comparison report",
    "report.generated": "Generated",
    "report.objects": "Objects with differences",
    "report.script": "Synchronization script",
    "report.noChanges": "No differences found.",
    "sel.syncAll": "Sync all",
    "sel.count": "{0} of {1} objects",
    "sel.include": "Include in synchronization",
    "script.noneSel": "(no objects selected to synchronize)",
    "apply.nothing": "No objects selected to apply.",
    "diag.field": "Field",
    "diag.type": "Data type",
    "diag.key": "Key",
    "detail.field": "Field",
    "detail.a": "Source",
    "detail.b": "Target",
    "status.only_in_left": "Only in the source",
    "status.only_in_right": "Only in the target",
    "status.different": "Different",
    "status.equal": "Equal",
    "type.table": "Table",
    "type.view": "View",
    "type.procedure": "Procedure",
    "type.function": "Function",
    "type.primary_key": "Primary key",
    "type.index": "Index",
    "type.foreign_key": "Foreign key",
    "type.check": "Constraint",
    "type.trigger": "Trigger",
    "err.noBackend": "No backend: open the app with cargo run or npm run tauri dev (a plain browser has no database connection).",
    "err.needDatabase": "Pick the database: without it the connection lands in whatever the server defaults to, which is almost never the one you meant to compare.",
    "status.pickDb": "Connected to the server · {n} databases available: pick one under “Database”.",
    "status.noDbs": "Connected to the server, but there is no database you can open.",
    "status.nDbs": "{n} databases",
    "db.connectFirst": "Connect to the server to list databases",
    "db.pick": "— pick a database —",
    "err.needPath": "Pick the SQLite file.",
  },
};

let lang = (() => {
  // 1) Preferencia guardada por el usuario (selector de idioma).
  try {
    const saved = localStorage.getItem("schemir.lang");
    if (saved) return saved;
  } catch {}
  // 2) Sin preferencia: autodetecta el idioma del sistema. Español → "es";
  //    cualquier otro → "en". Alcance internacional sin perder a los
  //    usuarios hispanohablantes.
  try {
    return (navigator.language || "en").toLowerCase().startsWith("es") ? "es" : "en";
  } catch {
    return "en";
  }
})();

/** Traduce una clave al idioma actual (con respaldo a ES y luego a la clave). */
function t(key) {
  return (I18N[lang] && I18N[lang][key]) || I18N.es[key] || key;
}

/** Aplica el idioma a todo el DOM y re-renderiza lo dinámico. */
function applyI18n() {
  document.documentElement.lang = lang;
  document.querySelectorAll("[data-i18n]").forEach((el) => (el.textContent = t(el.dataset.i18n)));
  document.querySelectorAll("[data-i18n-html]").forEach((el) => (el.innerHTML = t(el.dataset.i18nHtml)));
  document.querySelectorAll("[data-i18n-ph]").forEach((el) => (el.placeholder = t(el.dataset.i18nPh)));
  document.querySelectorAll("[data-i18n-title]").forEach((el) => (el.title = t(el.dataset.i18nTitle)));
  const chip = document.getElementById("chip-ver");
  if (chip) chip.textContent = "v0.1 · " + t("app.tagline");
  document.querySelectorAll(".lang-toggle button").forEach((b) => b.classList.toggle("active", b.dataset.lang === lang));
  if (compareBtn) compareBtn.textContent = comparing ? t("btn.comparing") : t("btn.compare");
  renderRecentSelectors();

  // Re-render de lo generado por JS.
  const results = document.getElementById("results");
  if (results && !results.hidden) {
    renderResults();
    updateScriptFromSelection();
    if (!document.getElementById("tab-diagram").hidden) renderDiagram();
  }
  const designer = document.getElementById("designer");
  if (designer && !designer.hidden) renderDesigner();
}

// ===========================================================================
// Puente con Tauri
// ===========================================================================
async function invoke(cmd, args) {
  const tauri = window.__TAURI__;
  if (!tauri?.core?.invoke) throw new Error(t("err.noBackend"));
  return tauri.core.invoke(cmd, args);
}

async function pickSqliteFile() {
  const tauri = window.__TAURI__;
  const options = {
    multiple: false,
    directory: false,
    title: t("field.sqliteFile"),
    filters: [
      { name: "SQLite", extensions: ["db", "sqlite", "sqlite3", "db3"] },
      { name: "*", extensions: ["*"] },
    ],
  };
  let sel = tauri?.dialog?.open ? await tauri.dialog.open(options) : await invoke("plugin:dialog|open", { options });
  if (Array.isArray(sel)) sel = sel[0];
  if (sel && typeof sel === "object") sel = sel.path ?? null;
  return sel || null;
}

async function saveSqlDialog() {
  const tauri = window.__TAURI__;
  const options = { title: t("btn.saveSql"), defaultPath: "sincronizacion.sql", filters: [{ name: "SQL", extensions: ["sql"] }] };
  let p = tauri?.dialog?.save ? await tauri.dialog.save(options) : await invoke("plugin:dialog|save", { options });
  if (p && typeof p === "object") p = p.path ?? null;
  return p || null;
}

/** Diálogo "Guardar como…" genérico (lo usan las exportaciones del diagrama). */
async function saveFileDialog(title, defaultPath, filterName, ext) {
  const tauri = window.__TAURI__;
  const options = { title, defaultPath, filters: [{ name: filterName, extensions: [ext] }] };
  let p = tauri?.dialog?.save ? await tauri.dialog.save(options) : await invoke("plugin:dialog|save", { options });
  if (p && typeof p === "object") p = p.path ?? null;
  return p || null;
}

// ===========================================================================
// Iconos y metadatos (independientes del idioma)
// ===========================================================================
const ICON = {
  table: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 10h18M9 4v16"/></svg>',
  view: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></svg>',
  procedure: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="m7 9 3 3-3 3M13 15h4"/></svg>',
  function: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M8 4c-2 0-3 1-3 3v2c0 1-1 1.6-2 1.6 1 0 2 .6 2 1.6v2c0 2 1 3 3 3M16 4c2 0 3 1 3 3v2c0 1 1 1.6 2 1.6-1 0-2 .6-2 1.6v2c0 2-1 3-3 3"/></svg>',
  index: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M4 6h16M4 12h16M4 18h9"/></svg>',
  primary_key: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="8" cy="15" r="4"/><path d="m10.6 12.4 7.4-7.4M15 8l2 2 2-2"/></svg>',
  foreign_key: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M10 13a5 5 0 0 0 7 0l2-2a5 5 0 0 0-7-7l-1 1"/><path d="M14 11a5 5 0 0 0-7 0l-2 2a5 5 0 0 0 7 7l1-1"/></svg>',
  check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 3 5 6v6c0 4 3 7 7 9 4-2 7-5 7-9V6l-7-3Z"/><path d="m9 12 2 2 4-4"/></svg>',
  trigger: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"><path d="M13 2 4 14h7l-1 8 9-12h-7l1-8Z"/></svg>',
};

// El backend manda el tipo en español; lo mapeamos a una clave estable.
const TYPE_KEY = {
  "Tabla": "table",
  "Vista": "view",
  "Procedimiento": "procedure",
  "Función": "function",
  "Índice": "index",
  "Clave primaria": "primary_key",
  "Clave foránea": "foreign_key",
  "Restricción": "check",
  "Disparador": "trigger",
};
const typeKey = (ot) => TYPE_KEY[ot] || "table";
const objIcon = (ot) => ICON[typeKey(ot)] || ICON.table;
const typeLabel = (ot) => t("type." + typeKey(ot));

const STATUS = {
  only_in_left: { badge: "left", stat: "left" },
  only_in_right: { badge: "right", stat: "right" },
  different: { badge: "diff", stat: "diff" },
  equal: { badge: "equal", stat: "equal" },
};

// Estado de la vista.
let lastObjects = [];
let lastScript = "";
let lastDataDml = "";            // DML de la última comparación de datos
let lastUnits = [];              // unidades del script (una por objeto)
let unitKeys = new Set();        // claves (tipo + nombre) que tienen SQL
let deselected = new Set();      // objetos desmarcados (excluidos del script)
const unitKey = (ot, name) => ot + "\u0000" + name;
let lastDiagramA = { tables: [], relations: [] };
let lastDiagramB = { tables: [], relations: [] };
let lastDiagramDiff = { tables: [], relations: [] };
let diaSide = "diff";
let comparing = false;
const activeFilters = new Set(["only_in_left", "only_in_right", "different"]);
let searchText = "";

// Opciones de comparación (persistidas); los nombres coinciden con Rust CompareOptions.
const DEFAULT_OPTIONS = {
  ignore_case: true, ignore_whitespace: true, ignore_comments: false, ignore_column_default: false,
  block_on_data_loss: true,
  tables: true, views: true, procedures: true, functions: true, indexes: true, foreign_keys: true, checks: true, primary_keys: true, triggers: true,
};
let compareOptions = (() => {
  try {
    return { ...DEFAULT_OPTIONS, ...JSON.parse(localStorage.getItem("schemir.options") || "{}") };
  } catch {
    return { ...DEFAULT_OPTIONS };
  }
})();

/** Refleja las opciones guardadas en las casillas y las persiste al cambiarlas. */
function setupOptions() {
  document.querySelectorAll("#cmp-options input[data-opt]").forEach((cb) => {
    cb.checked = !!compareOptions[cb.dataset.opt];
    cb.addEventListener("change", () => {
      compareOptions[cb.dataset.opt] = cb.checked;
      try {
        localStorage.setItem("schemir.options", JSON.stringify(compareOptions));
      } catch {}
    });
  });
}

// ===========================================================================
// Selector de motor por tarjeta
// ===========================================================================

/** Puerto que escucha cada motor cuando se instala sin tocar nada. */
const DEFAULT_PORTS = { sqlserver: 1433, postgres: 5432, mysql: 3306 };

/** Nombre legible del motor (MySQL y MariaDB comparten lector y dialecto). */
function engineLabel(engine) {
  return { postgres: "PostgreSQL", sqlite: "SQLite", mysql: "MySQL / MariaDB" }[engine] || "SQL Server";
}

/** Puerto por defecto del motor, o el de SQL Server si no se reconoce. */
function defaultPort(engine) {
  return DEFAULT_PORTS[engine] || DEFAULT_PORTS.sqlserver;
}

/**
 * Deja la tarjeta con los campos que toca: SQL Server, PostgreSQL y MySQL
 * comparten los de servidor; SQLite usa el de archivo.
 */
function applyEngineFields(card, engine) {
  card.querySelector(".f-sqlserver").hidden = engine === "sqlite";
  card.querySelector(".f-sqlite").hidden = engine !== "sqlite";
  const mysqlHint = card.querySelector(".hint-mysql");
  if (mysqlHint) mysqlHint.hidden = engine !== "mysql";
  // La autenticación integrada es cosa de SQL Server.
  const winRow = card.querySelector(".winauth-row");
  if (winRow) winRow.hidden = engine !== "sqlserver";
  const ssHint = card.querySelector(".hint-sqlserver");
  if (ssHint) ssHint.hidden = engine !== "sqlserver";
  // Las bases listadas eran de otro servidor: el selector vuelve a empezar.
  setDbPicker(card, null);
  applyWinAuth(card);
}

/** Con autenticación de Windows no hay usuario ni contraseña que escribir. */
function applyWinAuth(card) {
  const cb = card.querySelector('input[name="windows_auth"]');
  const on = card.dataset.engine === "sqlserver" && !!cb?.checked;
  ["user", "password"].forEach((name) => {
    const el = card.querySelector(`input[name="${name}"]`);
    if (!el) return;
    el.disabled = on;
    el.closest("label")?.classList.toggle("off", on);
  });
}
document.querySelectorAll(".card").forEach((card) => {
  card.querySelectorAll(".seg").forEach((seg) => {
    seg.addEventListener("click", () => {
      const engine = seg.dataset.engine;
      card.dataset.engine = engine;
      card.querySelectorAll(".seg").forEach((s) => s.classList.toggle("active", s === seg));
      applyEngineFields(card, engine);
      // Puerto por defecto según el motor: solo se pisa si el que hay es el de
      // otro motor (o está vacío), nunca uno que haya escrito el usuario.
      const portInput = card.querySelector('input[name="port"]');
      const esPorDefecto = portInput && (portInput.value === "" || Object.values(DEFAULT_PORTS).some((p) => String(p) === portInput.value));
      if (esPorDefecto) portInput.value = String(defaultPort(engine));
    });
  });
  const winCb = card.querySelector('input[name="windows_auth"]');
  if (winCb) winCb.addEventListener("change", () => applyWinAuth(card));
  applyWinAuth(card);
  attachDbPicker(card);
});

function readConfig(side) {
  const card = document.querySelector(`.card[data-side="${side}"]`);
  const engine = card.dataset.engine;
  const val = (name) => card.querySelector(`input[name="${name}"]`)?.value.trim() ?? "";
  if (engine === "sqlite") {
    return { engine: "sqlite", path: val("path"), host: "", database: "", user: "", password: "", port: 0 };
  }
  return {
    engine,
    host: val("host") || "localhost",
    port: Number(val("port")) || defaultPort(engine),
    database: val("database"),
    user: val("user"),
    password: val("password"),
    // Solo SQL Server la entiende; en los otros motores viaja siempre en false.
    windows_auth: engine === "sqlserver" && !!card.querySelector('input[name="windows_auth"]')?.checked,
    path: "",
  };
}

/** Pone un desplegable de bases de datos junto al campo de texto.
 *
 * El campo de texto sigue siendo la fuente de la verdad (de ahí lo lee
 * `readConfig`); el desplegable solo escribe en él. Se ven uno u otro, nunca los
 * dos, según el estado en que esté la tarjeta. */
function attachDbPicker(card) {
  const input = card.querySelector('input[name="database"]');
  if (!input || card.querySelector("select.db-select")) return;
  const sel = document.createElement("select");
  sel.className = "db-select";
  input.after(sel);
  sel.addEventListener("change", () => {
    input.value = sel.value;
    // La base salió de la lista del propio servidor, así que elegirla ya deja
    // la conexión lista: no hace falta volver a probarla.
    connOk[card.dataset.side] = !!sel.value;
    refreshCompareState();
  });
  setDbPicker(card, null);
}

/** Estado del selector de base de datos.
 *
 *  · `null`  → todavía no se ha conectado.
 *  · `[]`    → conectado, pero el login no puede enumerar ninguna: se escribe.
 *  · lista   → conectado: se elige de ahí.
 */
function setDbPicker(card, bases) {
  const input = card.querySelector('input[name="database"]');
  const sel = card.querySelector("select.db-select");
  if (!input || !sel) return;

  const verTexto = () => { sel.hidden = true; input.hidden = false; };
  const verLista = () => { sel.hidden = false; input.hidden = true; };

  if (bases === null) {
    // Sin conectar: si ya hay un nombre (de una comparación guardada, por
    // ejemplo) se muestra tal cual; si no, el desplegable dice qué hacer.
    if (input.value) return verTexto();
    verLista();
    sel.disabled = true;
    sel.replaceChildren(new Option(t("db.connectFirst"), ""));
    return;
  }
  if (bases.length === 0) {
    return verTexto();
  }
  verLista();
  sel.disabled = false;
  sel.replaceChildren(new Option(t("db.pick"), ""), ...bases.map((b) => new Option(b, b)));
  sel.value = bases.includes(input.value) ? input.value : "";
  if (!sel.value) input.value = "";
}

/** Qué le falta a una conexión para poder usarse, o `null` si está completa.
 *
 * La base de datos es obligatoria a propósito: si se manda vacía, el servidor
 * acepta la conexión igual y te deja en la base por defecto del login (`master`,
 * casi siempre). Antes eso se reportaba como «Conexión OK» y parecía que todo
 * estaba bien cuando no se había elegido nada. */
function connError(c) {
  if (c.engine === "sqlite") return c.path ? null : t("err.needPath");
  return c.database ? null : t("err.needDatabase");
}

// ===========================================================================
// Conexiones recientes (se recuerdan localmente, SIN la contraseña)
// ===========================================================================
let recentConns = (() => {
  try {
    return JSON.parse(localStorage.getItem("schemir.recentConns") || "[]");
  } catch {
    return [];
  }
})();

/** Firma única de una conexión (para no repetir la misma en la lista). */
function connSig(c) {
  if (c.engine === "sqlite") return "sqlite|" + c.path;
  return `${c.engine}|${c.host}:${c.port}/${c.database}@${c.windows_auth ? "(windows)" : c.user}`;
}

/** Etiqueta corta y legible de una conexión. */
function connProfileLabel(c) {
  if (c.engine === "sqlite") return "SQLite · " + (String(c.path).split(/[\\/]/).pop() || "—");
  return engineLabel(c.engine) + " · " + (c.database || c.host || "—");
}

/** Guarda una conexión válida en la lista de recientes (sin contraseña). */
function rememberConn(c) {
  if (!c) return;
  const useful = c.engine === "sqlite" ? !!c.path : !!(c.host || c.database);
  if (!useful) return;
  const profile = { engine: c.engine, host: c.host, port: c.port, database: c.database, user: c.user, path: c.path, windows_auth: !!c.windows_auth };
  const sig = connSig(c);
  recentConns = [profile, ...recentConns.filter((p) => connSig(p) !== sig)].slice(0, 8);
  try {
    localStorage.setItem("schemir.recentConns", JSON.stringify(recentConns));
  } catch {}
  renderRecentSelectors();
}

/** Rellena el selector «Recientes…» de cada tarjeta (oculto si no hay ninguna). */
function renderRecentSelectors() {
  document.querySelectorAll(".card-foot .recent-select").forEach((sel) => {
    const has = recentConns.length > 0;
    sel.hidden = !has;
    if (!has) return;
    sel.innerHTML =
      `<option value="">${esc(t("conn.recent"))}</option>` +
      recentConns.map((p, i) => `<option value="${i}">${esc(connProfileLabel(p))}</option>`).join("");
    sel.value = "";
    sel.title = t("conn.recentTitle");
  });
}

/** Aplica una conexión guardada a una tarjeta (deja la contraseña vacía). */
function applyConnProfile(card, p) {
  if (!p) return;
  const engine = p.engine || "sqlserver";
  card.dataset.engine = engine;
  card.querySelectorAll(".seg").forEach((s) => s.classList.toggle("active", s.dataset.engine === engine));
  applyEngineFields(card, engine);
  const set = (name, val) => {
    const el = card.querySelector(`input[name="${name}"]`);
    if (el) el.value = val ?? "";
  };
  if (engine === "sqlite") {
    set("path", p.path);
  } else {
    set("host", p.host);
    set("port", p.port || defaultPort(engine));
    set("database", p.database);
    set("user", p.user);
    const winCb = card.querySelector('input[name="windows_auth"]');
    if (winCb) winCb.checked = !!p.windows_auth;
    applyWinAuth(card);
    // El perfil trae un nombre de base, pero todavía no se ha conectado: hay
    // que devolver el selector a su estado inicial para que ese nombre se VEA
    // (si no, quedaría escondido detrás del desplegable).
    setDbPicker(card, null);
  }
  set("password", ""); // nunca guardamos ni rellenamos contraseñas
  // La conexión queda por validar: hay que volver a probarla.
  const side = card.dataset.side;
  connOk[side] = false;
  const status = card.querySelector(".conn-status");
  if (status) {
    status.className = "conn-status";
    status.textContent = "";
  }
  refreshCompareState();
}

// ===========================================================================
// Guardar / abrir una comparación completa (conexiones + opciones, sin claves)
// ===========================================================================
async function pickFile(title, filters) {
  const tauri = window.__TAURI__;
  const options = { multiple: false, directory: false, title, filters };
  let sel = tauri?.dialog?.open ? await tauri.dialog.open(options) : await invoke("plugin:dialog|open", { options });
  if (Array.isArray(sel)) sel = sel[0];
  if (sel && typeof sel === "object") sel = sel.path ?? null;
  return sel || null;
}

/** Documento con la comparación actual (ambas conexiones SIN contraseña + opciones). */
function currentComparisonDoc() {
  const strip = (c) => ({ engine: c.engine, host: c.host, port: c.port, database: c.database, user: c.user, path: c.path, windows_auth: !!c.windows_auth });
  return { schemir: "comparison", version: 1, options: compareOptions, a: strip(readConfig("a")), b: strip(readConfig("b")) };
}

/** Aplica un documento de comparación a las tarjetas y a las opciones. */
function applyComparisonDoc(doc) {
  if (doc.a) applyConnProfile(document.querySelector('.card[data-side="a"]'), doc.a);
  if (doc.b) applyConnProfile(document.querySelector('.card[data-side="b"]'), doc.b);
  if (doc.options && typeof doc.options === "object") {
    compareOptions = { ...DEFAULT_OPTIONS, ...doc.options };
    try { localStorage.setItem("schemir.options", JSON.stringify(compareOptions)); } catch {}
    document.querySelectorAll("#cmp-options input[data-opt]").forEach((cb) => (cb.checked = !!compareOptions[cb.dataset.opt]));
  }
}

async function saveComparison() {
  errorBanner.hidden = true;
  const st = document.getElementById("cmp-status");
  if (st) { st.className = "conn-status"; st.textContent = ""; }
  try {
    const path = await saveFileDialog(t("cmp.save"), "comparacion.ddcmp", "Schemir", "ddcmp");
    if (path) await invoke("save_text_file", { path, content: JSON.stringify(currentComparisonDoc(), null, 2) });
  } catch (e) {
    showError(e);
  }
}

async function openComparison() {
  errorBanner.hidden = true;
  const st = document.getElementById("cmp-status");
  if (st) { st.className = "conn-status"; st.textContent = ""; }
  try {
    const path = await pickFile(t("cmp.open"), [
      { name: "Schemir", extensions: ["ddcmp", "json"] },
      { name: "*", extensions: ["*"] },
    ]);
    if (!path) return;
    const text = await invoke("read_text_file", { path });
    let doc = null;
    try { doc = JSON.parse(text); } catch {}
    if (!doc || doc.schemir !== "comparison" || (!doc.a && !doc.b)) {
      if (st) { st.className = "conn-status fail"; st.textContent = "✗ " + t("cmp.invalid"); }
      return;
    }
    applyComparisonDoc(doc);
    if (st) { st.className = "conn-status ok"; st.textContent = t("cmp.loaded"); }
  } catch (e) {
    showError(e);
  }
}

document.getElementById("save-cmp").addEventListener("click", saveComparison);
document.getElementById("open-cmp").addEventListener("click", openComparison);

// ===========================================================================
// Probar conexión + habilitación de Comparar
// ===========================================================================
const compareBtn = document.getElementById("compare-btn");
const errorBanner = document.getElementById("error");
const compareHint = document.getElementById("compare-hint");
const connOk = { a: false, b: false };

function refreshCompareState() {
  const ready = connOk.a && connOk.b;
  compareBtn.disabled = !ready || comparing;
  if (compareHint) compareHint.hidden = ready;
}

document.querySelectorAll(".card").forEach((card) => {
  const side = card.dataset.side;
  const btn = card.querySelector(".test-btn");
  const status = card.querySelector(".conn-status");
  btn.addEventListener("click", async () => {
    const cfg = readConfig(side);
    // En SQLite la "base" es un archivo: no hay servidor al que conectarse ni
    // lista que traer, así que sigue haciendo falta la ruta por adelantado.
    if (cfg.engine === "sqlite" && !cfg.path) {
      status.className = "conn-status fail";
      status.textContent = "✗ " + t("err.needPath");
      connOk[side] = false;
      refreshCompareState();
      return;
    }
    status.className = "conn-status";
    status.textContent = t("status.testing");
    btn.disabled = true;
    try {
      // Primero el servidor, que es lo que se puede probar sin saber todavía
      // qué base se quiere: de ahí sale la lista para elegirla.
      let bases = [];
      if (cfg.engine !== "sqlite") {
        bases = await invoke("list_databases", { config: cfg });
        setDbPicker(card, bases);
      }
      if (cfg.engine !== "sqlite" && !cfg.database) {
        status.className = "conn-status ok";
        status.textContent =
          "✓ " + (bases.length ? t("status.pickDb").replace("{n}", bases.length) : t("status.noDbs"));
        // Conectar al servidor no basta para comparar: falta elegir la base.
        connOk[side] = false;
        return;
      }
      const msg = await invoke("test_connection", { config: cfg });
      status.className = "conn-status ok";
      const extra = bases.length ? " · " + t("status.nDbs").replace("{n}", bases.length) : "";
      status.textContent = "✓ " + msg + extra;
      connOk[side] = true;
      rememberConn(cfg);
    } catch (e) {
      status.className = "conn-status fail";
      status.textContent = "✗ " + errText(e);
      connOk[side] = false;
    } finally {
      btn.disabled = false;
      refreshCompareState();
    }
  });

  // Examinar… (selector de archivo SQLite)
  const browse = card.querySelector(".browse-btn");
  if (browse) {
    browse.addEventListener("click", async () => {
      try {
        const path = await pickSqliteFile();
        if (path) {
          card.querySelector('input[name="path"]').value = path;
          status.className = "conn-status";
          status.textContent = "";
          connOk[side] = false;
          refreshCompareState();
        }
      } catch (e) {
        status.className = "conn-status fail";
        status.textContent = "✗ " + errText(e);
      }
    });
  }

  // Selector de conexiones recientes: al elegir, rellena la tarjeta.
  const recentSel = card.querySelector(".recent-select");
  if (recentSel) {
    recentSel.addEventListener("change", () => {
      if (recentSel.value === "") return;
      applyConnProfile(card, recentConns[Number(recentSel.value)]);
      recentSel.value = "";
    });
  }

  // Cualquier cambio invalida la conexión.
  const invalidate = () => {
    connOk[side] = false;
    refreshCompareState();
  };
  card.querySelectorAll("input").forEach((inp) => inp.addEventListener("input", invalidate));
  card.querySelectorAll(".seg").forEach((seg) => seg.addEventListener("click", invalidate));
});
refreshCompareState();
renderRecentSelectors();

// ===========================================================================
// Comparar
// ===========================================================================
compareBtn.addEventListener("click", async () => {
  errorBanner.hidden = true;
  // Lo mismo que en «Probar conexión»: sin base de datos no se compara nada
  // útil, y el error es mucho más claro aquí que en forma de esquema vacío.
  for (const side of ["a", "b"]) {
    const falta = connError(readConfig(side));
    if (falta) {
      errorBanner.hidden = false;
      errorBanner.textContent = (side === "a" ? "ORIGEN: " : "DESTINO: ") + falta;
      return;
    }
  }
  comparing = true;
  compareBtn.disabled = true;
  compareBtn.textContent = t("btn.comparing");
  try {
    const result = await invoke("compare_databases", { left: readConfig("a"), right: readConfig("b"), options: compareOptions });
    rememberConn(readConfig("a"));
    rememberConn(readConfig("b"));
    lastObjects = result.comparison?.objects || [];
    lastScript = result.script || "";
    lastUnits = result.script_units || [];
    unitKeys = new Set(lastUnits.map((u) => unitKey(u.object_type, u.name)));
    deselected = new Set(); // por defecto se sincroniza todo
    lastDiagramA = result.diagram_a || { tables: [], relations: [] };
    lastDiagramB = result.diagram_b || { tables: [], relations: [] };
    lastDiagramDiff = result.diagram_diff || { tables: [], relations: [] };
    renderResults();
    updateScriptFromSelection();
    populateDataTables();
    document.getElementById("phase-dbs").textContent = dbLabel(readConfig("a")) + "  ⇄  " + dbLabel(readConfig("b"));
    hasCompared = true;
    document.querySelector('.step[data-step="comparison"]').disabled = false;
    goToStep("comparison");
  } catch (e) {
    errorBanner.textContent = errText(e);
    errorBanner.hidden = false;
  } finally {
    comparing = false;
    compareBtn.textContent = t("btn.compare");
    refreshCompareState();
  }
});

// ===========================================================================
// Navegación por fases (Paso 1 Conexión → Paso 2 Comparación)
// ===========================================================================
let hasCompared = false;
let activeStep = "connection";

function goToStep(step) {
  activeStep = step;
  document.getElementById("step-connection").hidden = step !== "connection";
  document.getElementById("step-comparison").hidden = step !== "comparison";
  document.querySelectorAll(".step").forEach((s) => s.classList.toggle("active", s.dataset.step === step));
  window.scrollTo(0, 0);
}

document.querySelectorAll(".stepper .step").forEach((s) => {
  s.addEventListener("click", () => {
    if (s.dataset.step === "comparison" && !hasCompared) return;
    goToStep(s.dataset.step);
  });
});

document.getElementById("back-btn").addEventListener("click", () => goToStep("connection"));

/** Etiqueta corta de una base para el encabezado del paso 2. */
function dbLabel(cfg) {
  if (cfg.engine === "sqlite") return "SQLite · " + (cfg.path.split(/[\\/]/).pop() || "—");
  return engineLabel(cfg.engine) + " · " + (cfg.database || cfg.host || "—");
}

// ===========================================================================
// Pestañas, script y diagrama
// ===========================================================================
document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((x) => x.classList.toggle("active", x === tab));
    document.getElementById("tab-diff").hidden = tab.dataset.tab !== "diff";
    document.getElementById("tab-script").hidden = tab.dataset.tab !== "script";
    document.getElementById("tab-data").hidden = tab.dataset.tab !== "data";
    document.getElementById("tab-diagram").hidden = tab.dataset.tab !== "diagram";
    if (tab.dataset.tab === "diagram") renderDiagram();
  });
});

document.getElementById("copy-script").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  const script = composeScript();
  try {
    await navigator.clipboard.writeText(script);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = script;
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch {}
    ta.remove();
  }
  btn.textContent = t("btn.copied");
  setTimeout(() => (btn.textContent = t("btn.copy")), 1200);
});

document.getElementById("save-script").addEventListener("click", async () => {
  errorBanner.hidden = true;
  try {
    const path = await saveSqlDialog();
    if (path) await invoke("save_text_file", { path, content: composeScript() });
  } catch (e) {
    errorBanner.textContent = errText(e);
    errorBanner.hidden = false;
  }
});

// Aplicar el script a la base B (ejecutar la migración), con confirmación.
document.getElementById("apply-script").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  const status = document.getElementById("apply-status");
  status.className = "conn-status";
  status.textContent = "";
  const composed = composeScript();
  const included = lastUnits.length - deselected.size;
  if (lastUnits.length === 0 || included === 0) {
    status.textContent = t("apply.nothing");
    return;
  }

  let ok = false;
  try {
    ok = await askConfirm(t("apply.confirm"));
  } catch {
    ok = false;
  }
  if (!ok) return;

  const prev = btn.textContent;
  btn.disabled = true;
  btn.textContent = t("apply.applying");
  try {
    await invoke("apply_script", { config: readConfig("b"), script: composed });
    status.className = "conn-status ok";
    status.textContent = t("apply.done");
  } catch (err) {
    status.className = "conn-status fail";
    status.textContent = "✗ " + errText(err);
  } finally {
    btn.disabled = false;
    btn.textContent = prev;
  }
});

/** Confirmación nativa (diálogo de Tauri) con aspecto de advertencia. */
async function askConfirm(message) {
  const td = window.__TAURI__;
  const options = { title: "Schemir", kind: "warning" };
  if (td?.dialog?.ask) return await td.dialog.ask(message, options);
  return await invoke("plugin:dialog|ask", { message, ...options });
}

document.querySelectorAll(".diagram-toggle .seg").forEach((seg) => {
  seg.addEventListener("click", () => {
    diaSide = seg.dataset.dia;
    document.querySelectorAll(".diagram-toggle .seg").forEach((s) => s.classList.toggle("active", s === seg));
    renderDiagram();
  });
});

// ── Estado del diagrama (zoom / pan / arrastre) ──
let pan = { x: 0, y: 0, scale: 1 };
let nodePos = {};
let diaModel = null;
let diagramHooked = false;

const clampScale = (v) => Math.max(0.25, Math.min(3, v));

function applyTransform() {
  const canvas = document.getElementById("er-canvas");
  if (canvas) canvas.style.transform = `translate(${pan.x}px, ${pan.y}px) scale(${pan.scale})`;
}

/** Zoom manteniendo fijo el punto (cx, cy) relativo a la caja del diagrama. */
function zoomAt(cx, cy, factor) {
  const ns = clampScale(pan.scale * factor);
  const k = ns / pan.scale;
  pan.x = cx - (cx - pan.x) * k;
  pan.y = cy - (cy - pan.y) * k;
  pan.scale = ns;
  applyTransform();
}

/** Dibuja las relaciones como líneas entre centros (se recalcula al arrastrar). */
function drawEdges() {
  const svg = document.getElementById("er-edges");
  if (!svg || !diaModel) return;
  svg.textContent = "";
  for (const rel of diaModel.relations) {
    const a = nodePos[rel.from];
    const b = nodePos[rel.to];
    if (!a || !b) continue;
    const ax = a.x + a.w / 2, ay = a.y + a.h / 2;
    const bx = b.x + b.w / 2, by = b.y + b.h / 2;
    const line = document.createElementNS(SVGNS, "line");
    line.setAttribute("x1", bx);
    line.setAttribute("y1", by);
    line.setAttribute("x2", ax);
    line.setAttribute("y2", ay);
    line.setAttribute("class", "er-edge");
    svg.appendChild(line);
    if (rel.label) {
      const text = document.createElementNS(SVGNS, "text");
      text.setAttribute("x", (ax + bx) / 2);
      text.setAttribute("y", (ay + by) / 2 - 5);
      text.setAttribute("class", "er-edge-label");
      text.textContent = rel.label;
      svg.appendChild(text);
    }
  }
}

/** Hace que una tarjeta se pueda arrastrar (ajustando por el zoom actual). */
function makeDraggable(card, id) {
  card.addEventListener("mousedown", (e) => {
    e.stopPropagation();
    const p = nodePos[id];
    const start = { x: e.clientX, y: e.clientY, left: p.x, top: p.y };
    const move = (ev) => {
      p.x = Math.round(start.left + (ev.clientX - start.x) / pan.scale);
      p.y = Math.round(start.top + (ev.clientY - start.y) / pan.scale);
      card.style.left = p.x + "px";
      card.style.top = p.y + "px";
      drawEdges();
    };
    const up = () => {
      document.removeEventListener("mousemove", move);
      document.removeEventListener("mouseup", up);
      card.classList.remove("dragging");
    };
    document.addEventListener("mousemove", move);
    document.addEventListener("mouseup", up);
    card.classList.add("dragging");
  });
}

/** Engancha rueda (zoom) y arrastre del fondo (pan) a la caja — solo una vez. */
function hookDiagramPanZoom(box) {
  if (diagramHooked) return;
  diagramHooked = true;
  box.addEventListener(
    "wheel",
    (e) => {
      e.preventDefault();
      const rect = box.getBoundingClientRect();
      zoomAt(e.clientX - rect.left, e.clientY - rect.top, e.deltaY < 0 ? 1.1 : 1 / 1.1);
    },
    { passive: false }
  );
  box.addEventListener("mousedown", (e) => {
    if (e.target.closest(".er-table") || e.target.closest(".er-controls")) return;
    const start = { x: e.clientX, y: e.clientY, px: pan.x, py: pan.y };
    const move = (ev) => {
      pan.x = start.px + (ev.clientX - start.x);
      pan.y = start.py + (ev.clientY - start.y);
      applyTransform();
    };
    const up = () => {
      document.removeEventListener("mousemove", move);
      document.removeEventListener("mouseup", up);
      box.classList.remove("panning");
    };
    document.addEventListener("mousemove", move);
    document.addEventListener("mouseup", up);
    box.classList.add("panning");
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") document.getElementById("tab-diagram")?.classList.remove("maximized");
  });
}

function toggleDiagramFullscreen() {
  document.getElementById("tab-diagram").classList.toggle("maximized");
}

function renderDiagram() {
  const box = document.getElementById("diagram-box");
  const model = diaSide === "diff" ? lastDiagramDiff : diaSide === "a" ? lastDiagramA : lastDiagramB;
  diaModel = model;

  const legend = document.getElementById("diagram-legend");
  if (legend) {
    if (diaSide === "diff") {
      legend.innerHTML =
        `<span class="lg lg-a">${esc(t("status.only_in_left"))}</span>` +
        `<span class="lg lg-b">${esc(t("status.only_in_right"))}</span>` +
        `<span class="lg lg-d">${esc(t("status.different"))}</span>` +
        `<span class="lg lg-e">${esc(t("status.equal"))}</span>`;
    } else {
      legend.textContent = t("diagram.legend");
    }
  }

  box.innerHTML = "";
  if (!model || !model.tables || model.tables.length === 0) {
    box.innerHTML = `<p class="empty">${esc(t("diagram.empty"))}</p>`;
    return;
  }
  if (!window.dagre) {
    box.innerHTML = `<p class="empty">${esc(t("diagram.noEngine"))}</p>`;
    return;
  }

  pan = { x: 0, y: 0, scale: 1 };

  const controls = document.createElement("div");
  controls.className = "er-controls";
  const mkBtn = (label, title, onClick) => {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = label;
    b.title = title;
    b.addEventListener("click", onClick);
    return b;
  };
  controls.append(
    mkBtn("+", t("zoom.in"), () => zoomAt(box.clientWidth / 2, box.clientHeight / 2, 1.2)),
    mkBtn("−", t("zoom.out"), () => zoomAt(box.clientWidth / 2, box.clientHeight / 2, 1 / 1.2)),
    mkBtn("⤢", t("zoom.fit"), () => { pan = { x: 0, y: 0, scale: 1 }; applyTransform(); }),
    mkBtn("⛶", t("zoom.full"), toggleDiagramFullscreen)
  );
  box.appendChild(controls);

  const canvas = document.createElement("div");
  canvas.className = "er-canvas";
  canvas.id = "er-canvas";
  const svg = document.createElementNS(SVGNS, "svg");
  svg.setAttribute("class", "er-edges");
  svg.id = "er-edges";
  canvas.appendChild(svg);
  box.appendChild(canvas);

  // Crear y medir las tarjetas.
  const cards = {};
  for (const tbl of model.tables) {
    const card = buildTableCard(tbl);
    card.style.position = "absolute";
    card.style.visibility = "hidden";
    canvas.appendChild(card);
    cards[tbl.id] = card;
  }

  // Layout inicial con dagre.
  const g = new window.dagre.graphlib.Graph();
  g.setGraph({ rankdir: "TB", nodesep: 44, ranksep: 72, marginx: 24, marginy: 24 });
  g.setDefaultEdgeLabel(() => ({}));
  for (const tbl of model.tables) {
    const r = cards[tbl.id].getBoundingClientRect();
    g.setNode(tbl.id, { width: Math.ceil(r.width), height: Math.ceil(r.height) });
  }
  for (const rel of model.relations) {
    if (cards[rel.from] && cards[rel.to]) g.setEdge(rel.to, rel.from);
  }
  window.dagre.layout(g);

  // Posicionar, guardar posición y hacer arrastrable cada tarjeta.
  nodePos = {};
  for (const tbl of model.tables) {
    const n = g.node(tbl.id);
    const x = Math.round(n.x - n.width / 2);
    const y = Math.round(n.y - n.height / 2);
    nodePos[tbl.id] = { x, y, w: n.width, h: n.height };
    const card = cards[tbl.id];
    card.style.left = x + "px";
    card.style.top = y + "px";
    card.style.visibility = "visible";
    makeDraggable(card, tbl.id);
  }

  const gw = Math.ceil(g.graph().width) || 400;
  const gh = Math.ceil(g.graph().height) || 300;
  canvas.style.width = gw + "px";
  canvas.style.height = gh + "px";
  svg.setAttribute("width", gw);
  svg.setAttribute("height", gh);

  drawEdges();
  applyTransform();
  hookDiagramPanZoom(box);
}

function buildTableCard(tbl) {
  const card = document.createElement("div");
  const stClass = { only_in_left: "st-left", only_in_right: "st-right", different: "st-diff", equal: "st-equal" }[tbl.status] || "";
  card.className = "er-table" + (stClass ? " " + stClass : "");
  const rows = tbl.columns
    .map((c) => `<tr><td class="er-c-name">${esc(c.name)}</td><td class="er-c-type">${esc(c.data_type)}</td><td class="er-c-key">${keyBadge(c.key)}</td></tr>`)
    .join("");
  card.innerHTML =
    `<div class="er-title">${esc(tbl.name)}</div>` +
    `<table class="er-grid"><thead><tr><th>${esc(t("diag.field"))}</th><th>${esc(t("diag.type"))}</th><th>${esc(t("diag.key"))}</th></tr></thead><tbody>${rows}</tbody></table>`;
  return card;
}

function keyBadge(key) {
  if (!key) return "";
  const pk = key.includes("PK") ? '<span class="k-pk">PK</span>' : "";
  const fk = key.includes("FK") ? '<span class="k-fk">FK</span>' : "";
  return [pk, fk].filter(Boolean).join(" ");
}

// ===========================================================================
// Exportar el diagrama (PNG / SVG)
// ===========================================================================
// Paleta clara fija para la exportación: el diagrama debe verse bien al pegarlo
// en un documento, sin importar si la app está en modo claro u oscuro.
const EXPORT = { bg: "#ffffff", card: "#ffffff", text: "#1f2933", muted: "#6b7280", edge: "#94a3b8", sep: "#eef2f6" };
const EXPORT_ST = {
  only_in_left:  { border: "#14b8a6", band: "#ccfbf1", bandText: "#0f766e" },
  only_in_right: { border: "#38bdf8", band: "#e0f2fe", bandText: "#0369a1" },
  different:     { border: "#f59e0b", band: "#fef3c7", bandText: "#92400e" },
  equal:         { border: "#cbd5e1", band: "#f1f5f9", bandText: "#64748b" },
  "":            { border: "#cbd5e1", band: "#f1f5f9", bandText: "#334155" },
};
const EXP_FONT = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif";
const TITLE_H = 34, HEAD_H = 26;

/** Ancho aproximado de un texto (para truncar sin medir en el DOM). */
const textW = (s, px) => String(s).length * px * 0.58;

/** Trunca un texto con «…» para que no exceda maxW píxeles. */
function fitText(s, maxW, px) {
  s = String(s);
  if (maxW <= 0) return "";
  if (textW(s, px) <= maxW) return s;
  let n = s.length;
  while (n > 0 && textW(s.slice(0, n) + "…", px) > maxW) n--;
  return n > 0 ? s.slice(0, n) + "…" : "…";
}

function pill(x, cy, label, fill, color) {
  const w = 22, h = 15;
  return (
    `<rect x="${x}" y="${(cy - h / 2).toFixed(1)}" width="${w}" height="${h}" rx="4" fill="${fill}"/>` +
    `<text x="${x + w / 2}" y="${cy.toFixed(1)}" font-size="9.5" font-weight="700" fill="${color}" text-anchor="middle" dominant-baseline="middle">${label}</text>`
  );
}

/** Serializa el diagrama actual (modelo + posiciones) a un SVG independiente. */
function buildDiagramSvg() {
  const model = diaModel;
  const ids = Object.keys(nodePos);
  if (!model || !model.tables || !ids.length) return null;

  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  for (const id of ids) {
    const p = nodePos[id];
    minX = Math.min(minX, p.x); minY = Math.min(minY, p.y);
    maxX = Math.max(maxX, p.x + p.w); maxY = Math.max(maxY, p.y + p.h);
  }
  const PAD = 24;
  const W = Math.ceil(maxX - minX) + PAD * 2;
  const H = Math.ceil(maxY - minY) + PAD * 2;
  const ox = PAD - minX, oy = PAD - minY;

  const parts = [];

  // Relaciones (debajo de las tarjetas).
  for (const rel of model.relations) {
    const a = nodePos[rel.from], b = nodePos[rel.to];
    if (!a || !b) continue;
    const ax = a.x + a.w / 2 + ox, ay = a.y + a.h / 2 + oy;
    const bx = b.x + b.w / 2 + ox, by = b.y + b.h / 2 + oy;
    parts.push(`<line x1="${bx.toFixed(1)}" y1="${by.toFixed(1)}" x2="${ax.toFixed(1)}" y2="${ay.toFixed(1)}" stroke="${EXPORT.edge}" stroke-width="1.5"/>`);
    if (rel.label) {
      const mx = (ax + bx) / 2, my = (ay + by) / 2 - 5;
      const lw = textW(rel.label, 11) + 8;
      parts.push(`<rect x="${(mx - lw / 2).toFixed(1)}" y="${(my - 11).toFixed(1)}" width="${lw.toFixed(1)}" height="15" rx="3" fill="${EXPORT.bg}" opacity="0.85"/>`);
      parts.push(`<text x="${mx.toFixed(1)}" y="${my.toFixed(1)}" font-size="11" fill="${EXPORT.muted}" text-anchor="middle">${esc(rel.label)}</text>`);
    }
  }

  // Tarjetas.
  for (const tbl of model.tables) {
    const p = nodePos[tbl.id];
    if (!p) continue;
    const x = p.x + ox, y = p.y + oy, w = p.w, h = p.h;
    const st = EXPORT_ST[tbl.status] || EXPORT_ST[""];
    const cols = tbl.columns || [];
    const rowsArea = Math.max(0, h - TITLE_H - HEAD_H);
    const rowH = cols.length ? rowsArea / cols.length : rowsArea;
    const nameX = x + 12, typeX = x + Math.round(w * 0.5), keyX = x + Math.round(w * 0.82);

    parts.push(`<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="9" fill="${EXPORT.card}" stroke="${st.border}" stroke-width="1.5"/>`);
    // Banda de título (esquinas superiores redondeadas).
    parts.push(
      `<path d="M${x + 1.5},${y + TITLE_H} L${x + 1.5},${y + 9} Q${x + 1.5},${y + 1.5} ${x + 9},${y + 1.5} ` +
      `L${x + w - 9},${y + 1.5} Q${x + w - 1.5},${y + 1.5} ${x + w - 1.5},${y + 9} L${x + w - 1.5},${y + TITLE_H} Z" fill="${st.band}"/>`
    );
    parts.push(`<text x="${x + 12}" y="${(y + TITLE_H / 2 + 1).toFixed(1)}" font-size="13" font-weight="700" fill="${st.bandText}" dominant-baseline="middle">${esc(fitText(tbl.name, w - 24, 13))}</text>`);

    // Encabezados de columna.
    const hy = (y + TITLE_H + HEAD_H / 2).toFixed(1);
    parts.push(`<text x="${nameX}" y="${hy}" font-size="10.5" fill="${EXPORT.muted}" dominant-baseline="middle">${esc(t("diag.field"))}</text>`);
    parts.push(`<text x="${typeX}" y="${hy}" font-size="10.5" fill="${EXPORT.muted}" dominant-baseline="middle">${esc(t("diag.type"))}</text>`);
    parts.push(`<text x="${keyX}" y="${hy}" font-size="10.5" fill="${EXPORT.muted}" dominant-baseline="middle">${esc(t("diag.key"))}</text>`);
    parts.push(`<line x1="${x + 1}" y1="${y + TITLE_H + HEAD_H}" x2="${x + w - 1}" y2="${y + TITLE_H + HEAD_H}" stroke="${EXPORT.sep}" stroke-width="1"/>`);

    // Filas.
    cols.forEach((c, i) => {
      const ry = y + TITLE_H + HEAD_H + i * rowH;
      const cy = ry + rowH / 2;
      if (i > 0) parts.push(`<line x1="${x + 1}" y1="${ry.toFixed(1)}" x2="${x + w - 1}" y2="${ry.toFixed(1)}" stroke="${EXPORT.sep}" stroke-width="1"/>`);
      parts.push(`<text x="${nameX}" y="${cy.toFixed(1)}" font-size="12" fill="${EXPORT.text}" dominant-baseline="middle">${esc(fitText(c.name, typeX - nameX - 8, 12))}</text>`);
      parts.push(`<text x="${typeX}" y="${cy.toFixed(1)}" font-size="11.5" fill="${EXPORT.muted}" dominant-baseline="middle">${esc(fitText(c.data_type, keyX - typeX - 6, 11.5))}</text>`);
      let kx = keyX;
      const key = c.key || "";
      if (key.includes("PK")) { parts.push(pill(kx, cy, "PK", "#fde68a", "#92400e")); kx += 26; }
      if (key.includes("FK")) { parts.push(pill(kx, cy, "FK", "#bae6fd", "#075985")); }
    });
  }

  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" viewBox="0 0 ${W} ${H}" font-family="${EXP_FONT}">` +
    `<rect x="0" y="0" width="${W}" height="${H}" fill="${EXPORT.bg}"/>` +
    parts.join("") +
    `</svg>`;
  return { svg, width: W, height: H };
}

/** Rasteriza un SVG a bytes PNG mediante un canvas (sin dependencias externas). */
async function svgToPngBytes(svg, width, height) {
  let scale = 2;
  const maxSide = Math.max(width, height);
  if (maxSide * scale > 4096) scale = Math.max(1, 4096 / maxSide);

  const blob = new Blob([svg], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  try {
    const img = new Image();
    await new Promise((res, rej) => {
      img.onload = () => res();
      img.onerror = () => rej(new Error("No se pudo rasterizar el diagrama."));
      img.src = url;
    });
    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(width * scale));
    canvas.height = Math.max(1, Math.round(height * scale));
    const ctx = canvas.getContext("2d");
    ctx.scale(scale, scale);
    ctx.drawImage(img, 0, 0, width, height);
    const out = await new Promise((res, rej) =>
      canvas.toBlob((b) => (b ? res(b) : rej(new Error("No se pudo generar el PNG."))), "image/png")
    );
    return new Uint8Array(await out.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}

function exportBaseName() {
  return diaSide === "a" ? "diagrama-origen-a" : diaSide === "b" ? "diagrama-destino-b" : "diagrama-diferencias";
}

function showError(e) {
  errorBanner.textContent = errText(e);
  errorBanner.hidden = false;
}

async function exportDiagramSvg() {
  errorBanner.hidden = true;
  const out = buildDiagramSvg();
  if (!out) return;
  try {
    const path = await saveFileDialog(t("btn.exportSvg"), exportBaseName() + ".svg", "SVG", "svg");
    if (path) await invoke("save_text_file", { path, content: out.svg });
  } catch (e) {
    showError(e);
  }
}

async function exportDiagramPng() {
  errorBanner.hidden = true;
  const out = buildDiagramSvg();
  if (!out) return;
  try {
    const bytes = await svgToPngBytes(out.svg, out.width, out.height);
    const path = await saveFileDialog(t("btn.exportPng"), exportBaseName() + ".png", "PNG", "png");
    if (path) await invoke("save_binary_file", { path, data: Array.from(bytes) });
  } catch (e) {
    showError(e);
  }
}

document.getElementById("export-png").addEventListener("click", exportDiagramPng);
document.getElementById("export-svg").addEventListener("click", exportDiagramSvg);

// ===========================================================================
// Exportar informe de diferencias (HTML autónomo, imprimible a PDF)
// ===========================================================================
const REPORT_CSS = `
*{box-sizing:border-box}
body{margin:0;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;color:#1f2933;background:#f1f5f9;padding:32px;line-height:1.45}
header,section.summary,h2,.obj,pre.script,.empty,footer{max-width:900px;margin-left:auto;margin-right:auto}
h1{margin:0;font-size:26px;letter-spacing:-.5px}
.sub{margin:2px 0 10px;color:#475569;font-size:15px}
.meta{margin:2px 0;color:#64748b;font-size:13px}
section.summary{margin-top:0;margin-bottom:24px;display:flex;gap:12px;flex-wrap:wrap}
.summary .s{flex:1;min-width:120px;background:#fff;border:1px solid #e2e8f0;border-radius:10px;padding:12px 14px}
.summary .n{font-size:24px;font-weight:700}
.summary .l{font-size:12px;color:#64748b}
.s-only_in_left .n{color:#0f766e}.s-only_in_right .n{color:#0369a1}.s-different .n{color:#b45309}.s-equal .n{color:#64748b}
h2{margin-top:26px;margin-bottom:12px;font-size:16px;color:#334155;border-bottom:1px solid #e2e8f0;padding-bottom:6px}
.obj{margin-bottom:10px;background:#fff;border:1px solid #e2e8f0;border-left-width:4px;border-radius:8px;padding:10px 14px}
.obj.st-only_in_left{border-left-color:#14b8a6}.obj.st-only_in_right{border-left-color:#38bdf8}.obj.st-different{border-left-color:#f59e0b}
.oh{display:flex;align-items:center;gap:10px}
.ot{font-size:11px;text-transform:uppercase;letter-spacing:.5px;color:#94a3b8}
.on{font-weight:600;font-size:15px}
.ob{margin-left:auto;font-size:11px;font-weight:600;padding:2px 9px;border-radius:999px}
.b-only_in_left{background:#ccfbf1;color:#0f766e}.b-only_in_right{background:#e0f2fe;color:#0369a1}.b-different{background:#fef3c7;color:#92400e}
table.det{width:100%;border-collapse:collapse;margin-top:8px;font-size:13px}
table.det th{text-align:left;color:#64748b;font-weight:600;padding:4px 8px;border-bottom:1px solid #e2e8f0;font-size:12px}
table.det td{padding:4px 8px;border-bottom:1px solid #f1f5f9;vertical-align:top;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:12px}
pre.script{background:#0f172a;color:#e2e8f0;padding:16px;border-radius:10px;overflow:auto;font-size:12.5px;line-height:1.5;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.empty{color:#64748b}
footer{margin-top:30px;color:#94a3b8;font-size:12px;text-align:center}
@media print{body{background:#fff;padding:0}.obj,.summary .s{break-inside:avoid}pre.script{background:#f8fafc;color:#0f172a;border:1px solid #e2e8f0}}
`;

/** Arma un informe HTML autónomo con el resumen, las diferencias y el script. */
function buildReportHtml() {
  const dbs = (document.getElementById("phase-dbs")?.textContent || "").trim();
  const now = new Date().toLocaleString(lang === "en" ? "en-US" : "es-ES");

  const counts = { only_in_left: 0, only_in_right: 0, different: 0, equal: 0 };
  for (const o of lastObjects) counts[o.status]++;
  const summary = Object.keys(STATUS)
    .map((k) => `<div class="s s-${k}"><div class="n">${counts[k]}</div><div class="l">${esc(t("status." + k))}</div></div>`)
    .join("");

  const changed = lastObjects.filter((o) => o.status !== "equal");
  const objs = changed
    .map((o) => {
      const det =
        o.details && o.details.length
          ? `<table class="det"><thead><tr><th>${esc(t("detail.field"))}</th><th>${esc(t("detail.a"))}</th><th>${esc(t("detail.b"))}</th></tr></thead><tbody>` +
            o.details.map((d) => `<tr><td>${esc(d.field)}</td><td>${esc(d.left ?? "—")}</td><td>${esc(d.right ?? "—")}</td></tr>`).join("") +
            `</tbody></table>`
          : "";
      return (
        `<div class="obj st-${o.status}"><div class="oh">` +
        `<span class="ot">${esc(typeLabel(o.object_type))}</span>` +
        `<span class="on">${esc(o.name)}</span>` +
        `<span class="ob b-${o.status}">${esc(t("status." + o.status))}</span>` +
        `</div>${det}</div>`
      );
    })
    .join("");

  const script = lastScript && lastScript.trim() ? esc(lastScript) : esc(t("script.none"));

  return (
    `<!doctype html><html lang="${lang}"><head><meta charset="utf-8">` +
    `<title>Schemir — ${esc(t("report.title"))}</title><style>${REPORT_CSS}</style></head><body>` +
    `<header><h1>Schemir</h1><p class="sub">${esc(t("report.title"))}</p>` +
    (dbs ? `<p class="meta">${esc(dbs)}</p>` : "") +
    `<p class="meta">${esc(t("report.generated"))}: ${esc(now)}</p></header>` +
    `<section class="summary">${summary}</section>` +
    `<h2>${esc(t("report.objects"))}</h2>` +
    (changed.length ? objs : `<p class="empty">${esc(t("report.noChanges"))}</p>`) +
    `<h2>${esc(t("report.script"))}</h2><pre class="script">${script}</pre>` +
    `<footer>Schemir · github.com/marlontaro/Schemir</footer>` +
    `</body></html>`
  );
}

async function exportReport() {
  errorBanner.hidden = true;
  if (!lastObjects || !lastObjects.length) return;
  try {
    const html = buildReportHtml();
    const path = await saveFileDialog(t("btn.report"), "informe-schemir.html", "HTML", "html");
    if (path) await invoke("save_text_file", { path, content: html });
  } catch (e) {
    showError(e);
  }
}

document.getElementById("export-report").addEventListener("click", exportReport);

// ===========================================================================
// Selección de sincronización: elegir qué objetos entran en el script
// ===========================================================================
function targetDialectLabel() {
  return engineLabel(readConfig("b").engine);
}

function syncHeader() {
  return [
    "-- ============================================================",
    "-- Script de sincronización generado por Schemir",
    "-- Deja el DESTINO igual que el ORIGEN",
    "-- Motor destino: " + targetDialectLabel(),
    "-- (!) Revísalo y respalda el destino antes de ejecutarlo.",
    "-- ============================================================",
  ].join("\n");
}

/** Compone el script con SOLO las unidades marcadas (en el orden del backend). */
function composeScript() {
  const units = lastUnits || [];
  if (!units.length) return lastScript || t("script.none");
  const included = units.filter((u) => !deselected.has(unitKey(u.object_type, u.name)));
  const header = syncHeader();
  if (!included.length) return header + "\n\n-- " + t("script.noneSel") + "\n";
  let out = header + "\n\n";
  let section = null;
  for (const u of included) {
    if (u.section !== section) {
      out += `-- ─── ${u.section} ───\n`;
      section = u.section;
    }
    out += u.sql.replace(/\s+$/, "") + "\n\n";
  }
  return out.replace(/\s+$/, "") + "\n";
}

/** Refresca el cuadro del script y la barra de selección. */
function updateScriptFromSelection() {
  const box = document.getElementById("script-box");
  if (box) box.innerHTML = highlightSql(composeScript());
  updateSelUI();
}

/** Actualiza el contador y la casilla maestra "Sincronizar todo". */
function updateSelUI() {
  const total = lastUnits.length;
  const inc = total - deselected.size;
  const bar = document.getElementById("sel-bar");
  if (bar) bar.hidden = total === 0;
  const cnt = document.getElementById("sel-count");
  if (cnt) cnt.textContent = total ? t("sel.count").replace("{0}", inc).replace("{1}", total) : "";
  const master = document.getElementById("sel-all-cb");
  if (master) {
    master.checked = total > 0 && inc === total;
    master.indeterminate = inc > 0 && inc < total;
    master.disabled = total === 0;
  }
}

/** Marca o desmarca todas las unidades a la vez. */
function setAllSelected(sel) {
  deselected = new Set();
  if (!sel) for (const u of lastUnits) deselected.add(unitKey(u.object_type, u.name));
  renderRows();
  updateScriptFromSelection();
}

document.getElementById("sel-all-cb").addEventListener("change", (e) => setAllSelected(e.target.checked));

// ===========================================================================
// Comparar DATOS de filas (por tabla)
// ===========================================================================
/** Llena el selector con las tablas que existen en A y B (iguales o distintas). */
function populateDataTables() {
  const sel = document.getElementById("data-table");
  if (!sel) return;
  const inBoth = lastObjects
    .filter((o) => o.object_type === "Tabla" && (o.status === "different" || o.status === "equal"))
    .map((o) => o.name);
  sel.innerHTML = inBoth.length
    ? inBoth.map((n) => `<option value="${esc(n)}">${esc(n)}</option>`).join("")
    : `<option value="">${esc(t("data.noTables"))}</option>`;
  // Estado inicial del panel.
  lastDataDml = "";
  document.getElementById("data-summary").innerHTML = "";
  document.getElementById("data-result").hidden = true;
  document.getElementById("data-grid-wrap").hidden = true;
  const empty = document.getElementById("data-empty");
  empty.hidden = false;
  empty.textContent = inBoth.length ? t("data.hint") : t("data.noTables");
}

function renderDataResult(cmp) {
  const summary = document.getElementById("data-summary");
  const result = document.getElementById("data-result");
  const empty = document.getElementById("data-empty");
  if (cmp.note) {
    summary.innerHTML = "";
    result.hidden = true;
    document.getElementById("data-grid-wrap").hidden = true;
    empty.hidden = false;
    empty.textContent = cmp.note;
    lastDataDml = "";
    return;
  }
  empty.hidden = true;
  const tiles = [
    ["left", cmp.only_in_left, "data.leftRows"],
    ["right", cmp.only_in_right, "data.rightRows"],
    ["diff", cmp.different, "data.diffRows"],
    ["equal", cmp.equal, "data.equalRows"],
  ];
  summary.innerHTML = tiles
    .map(([st, n, k]) => `<div class="stat ${st}"><div class="stat-n">${n}</div><div class="stat-l"><span class="dot"></span>${esc(t(k))}</div></div>`)
    .join("");
  renderDataGrid(cmp);
  lastDataDml = cmp.dml || "";
  document.getElementById("data-box").innerHTML = highlightSql(lastDataDml || t("data.none"));
  result.hidden = false;
}

/** Cuadrícula con una muestra de las filas que difieren (más legible que el DML). */
function renderDataGrid(cmp) {
  const gw = document.getElementById("data-grid-wrap");
  const grid = document.getElementById("data-grid");
  const note = document.getElementById("data-grid-note");
  const sample = cmp.sample || [];
  if (!sample.length) {
    gw.hidden = true;
    return;
  }
  const cols = cmp.columns || [];
  const opBadge = { insert: "left", update: "diff", delete: "right" };
  const opKey = { insert: "data.opInsert", update: "data.opUpdate", delete: "data.opDelete" };
  const head = `<thead><tr><th class="op-col"></th>${cols.map((c) => `<th>${esc(c)}</th>`).join("")}</tr></thead>`;
  const body =
    "<tbody>" +
    sample
      .map((r) => {
        const badge = opBadge[r.op] || "equal";
        const old = r.op === "update" ? r.old_cells || null : null;
        const cells = (r.cells || [])
          .map((v, i) => {
            const val = v === null || v === undefined ? '<span class="null">NULL</span>' : esc(v);
            const changed = old && (old[i] ?? null) !== (v ?? null);
            if (!changed) return `<td>${val}</td>`;
            const ov = old[i];
            const oldTxt = ov === null || ov === undefined ? "NULL" : String(ov);
            return `<td class="changed" title="${esc(t("data.wasLabel"))}: ${esc(oldTxt)}">${val}</td>`;
          })
          .join("");
        return `<tr><td class="op-col"><span class="badge ${badge}"><span class="dot"></span>${esc(t(opKey[r.op] || ""))}</span></td>${cells}</tr>`;
      })
      .join("") +
    "</tbody>";
  grid.innerHTML = head + body;
  gw.hidden = false;
  const total = cmp.only_in_left + cmp.only_in_right + cmp.different;
  if (sample.length < total) {
    note.hidden = false;
    note.textContent = t("data.gridMore").replace("{0}", sample.length).replace("{1}", total);
  } else {
    note.hidden = true;
  }
}

document.getElementById("compare-data").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  const table = document.getElementById("data-table").value;
  const status = document.getElementById("data-status");
  status.className = "conn-status";
  status.textContent = "";
  if (!table) return;
  btn.disabled = true;
  btn.textContent = t("data.comparing");
  try {
    const cmp = await invoke("compare_data", { left: readConfig("a"), right: readConfig("b"), table });
    renderDataResult(cmp);
  } catch (err) {
    status.className = "conn-status fail";
    status.textContent = "✗ " + errText(err);
  } finally {
    btn.disabled = false;
    btn.textContent = t("btn.compareData");
  }
});

document.getElementById("copy-data").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  try {
    await navigator.clipboard.writeText(lastDataDml);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = lastDataDml;
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch {}
    ta.remove();
  }
  btn.textContent = t("btn.copied");
  setTimeout(() => (btn.textContent = t("btn.copy")), 1200);
});

document.getElementById("save-data").addEventListener("click", async () => {
  errorBanner.hidden = true;
  try {
    const path = await saveSqlDialog();
    if (path) await invoke("save_text_file", { path, content: lastDataDml });
  } catch (e) {
    showError(e);
  }
});

document.getElementById("apply-data").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  const status = document.getElementById("apply-data-status");
  status.className = "conn-status";
  status.textContent = "";
  if (!lastDataDml || !lastDataDml.trim()) return;
  let ok = false;
  try { ok = await askConfirm(t("apply.confirm")); } catch { ok = false; }
  if (!ok) return;
  const prev = btn.textContent;
  btn.disabled = true;
  btn.textContent = t("apply.applying");
  try {
    await invoke("apply_script", { config: readConfig("b"), script: lastDataDml });
    status.className = "conn-status ok";
    status.textContent = t("apply.done");
  } catch (err) {
    status.className = "conn-status fail";
    status.textContent = "✗ " + errText(err);
  } finally {
    btn.disabled = false;
    btn.textContent = prev;
  }
});

// ===========================================================================
// Diseñador de modelos, al estilo de ERwin
// ===========================================================================
// Las tres cosas que lo hacen un modelador y no un dibujador de cajas:
//   · el tipo de dato se ELIGE de la lista del motor y el diseñador sabe si
//     lleva longitud, las dos cifras de precisión y escala, o nada;
//   · las tablas pertenecen a un ESQUEMA, que se puede crear y asignar;
//   · las relaciones se tiran ARRASTRANDO de una columna a otra, y al soltar
//     aparece el diálogo que las nombra.
// El catálogo de tipos no vive aquí: lo da el núcleo (`crates/core/src/types.rs`,
// comando `dialect_info`), que es quien sabe de motores y tiene las pruebas.

const DZ_DIALECT_DEFAULT = "sqlserver";

let design = loadDesign();
let lastDesignOut = "";

/** Catálogo del motor elegido, tal como lo devuelve el núcleo. Queda en `null`
 *  si el comando no respondió; entonces el tipo se escribe a mano, que es lo
 *  que hacía el diseñador antes de existir el catálogo. */
let dz = null;
const dzCache = {};
/** Lo que menos puede medir de alto una tarjeta: la cabecera, una fila de la
 *  clave y un par de columnas. Por debajo de eso la tarjeta no dice ni de qué
 *  tabla es, y lo que queda no se lee. */
const DZ_MIN_H = 140;
/** Relación que se está tirando con el ratón, o `null`. */
let dzLink = null;

function loadDesign() {
  let d = null;
  try { d = JSON.parse(localStorage.getItem("schemir.design") || "null"); } catch {}
  if (!d || !Array.isArray(d.tables)) d = {};
  return migrateDesign({ tables: [], relations: [], schemas: [], seq: 1, dialect: DZ_DIALECT_DEFAULT, version: "", ...d });
}

/** Pone al día un modelo guardado por una versión anterior del diseñador: antes
 *  el tipo era un texto libre ("varchar(50)") y ahora son tres campos, y no
 *  había ni esquemas ni nombre de relación. */
function migrateDesign(d) {
  if (!Array.isArray(d.schemas)) d.schemas = [];
  // La vista es una preferencia del modelo, como el motor y la versión: al
  // abrirlo se vuelve a la que se estaba mirando.
  d.view = d.view === "logical" ? "logical" : "physical";
  for (const tbl of d.tables) {
    if (typeof tbl.schema !== "string") tbl.schema = "";
    if (typeof tbl.color !== "string") tbl.color = "";
    if (typeof tbl.logical !== "string") tbl.logical = "";
    tbl.columns = tbl.columns || [];
    for (const col of tbl.columns) {
      if (col.a === undefined && col.b === undefined) Object.assign(col, splitType(col.type));
      col.a = col.a ?? "";
      col.b = col.b ?? "";
      col.auto = col.auto || "";
      if (typeof col.logical !== "string") col.logical = "";
    }
  }
  for (const r of d.relations) {
    r.name = r.name || "";
    r.onDelete = r.onDelete || "";
    r.onUpdate = r.onUpdate || "";
    // Antes una relación era un par suelto de columnas. Ahora es una LISTA de
    // pares, porque una clave foránea puede tener varias columnas. Se normaliza
    // aquí, de una vez, para que ningún otro sitio tenga que mirar las dos
    // formas: a partir de este punto solo existe `cols`.
    if (!Array.isArray(r.cols)) r.cols = [{ from: r.fromCol || "", to: r.toCol || "" }];
    r.cols = r.cols.filter((p) => p && p.from && p.to);
    delete r.fromCol;
    delete r.toCol;
  }
  // Una relación sin ninguna columna no dice nada y no se puede escribir en SQL.
  d.relations = d.relations.filter((r) => r.cols.length);
  return d;
}

function saveDesign() {
  try { localStorage.setItem("schemir.design", JSON.stringify(design)); } catch {}
}
function dzTableById(id) { return design.tables.find((x) => x.id === id); }

/** Parte "decimal(10,2)" en sus tres piezas. */
function splitType(s) {
  const m = /^\s*([^(]+?)\s*\(\s*([^,)]*?)\s*(?:,\s*([^)]*?)\s*)?\)\s*$/.exec(String(s || ""));
  if (!m) return { type: String(s || "").trim(), a: "", b: "" };
  return { type: m[1], a: m[2] || "", b: m[3] || "" };
}

/** Compone el tipo como va en el DDL. Espejo de `format_type` del núcleo. */
function fullType(col) {
  const a = String(col.a ?? "").trim();
  const b = String(col.b ?? "").trim();
  if (a && b) return `${col.type}(${a},${b})`;
  if (a) return `${col.type}(${a})`;
  return col.type || "";
}

// ── El catálogo del motor ────────────────────────────────────────────────────

/** Trae del núcleo los tipos, las versiones y si el motor tiene esquemas. Se
 *  guarda en memoria: no cambia mientras el programa esté abierto. */
async function loadDialectInfo(dialect) {
  if (dzCache[dialect] !== undefined) return dzCache[dialect];
  let info = null;
  try {
    info = await invoke("dialect_info", { dialect });
  } catch {
    info = null; // sin catálogo: se escribe el tipo a mano
  }
  dzCache[dialect] = info;
  return info;
}

function dzFind(name) {
  if (!dz) return null;
  const n = String(name || "").trim().toLowerCase();
  return dz.types.find((ty) => ty.name.toLowerCase() === n) || null;
}

/** Nombre del motor tal como lo dice su desplegable, para no repetir las
 *  etiquetas en dos sitios. */
function dzDialectLabel(dialect) {
  const o = document.querySelector(`#design-dialect option[value="${dialect}"]`);
  return o ? o.textContent : dialect;
}

/** La versión del motor elegida como objetivo. */
function dzTarget() {
  if (!dz) return null;
  return dz.versions.find((v) => v.label === design.version) || dz.default_version;
}
function dzNewer(a, b) {
  return a.major > b.major || (a.major === b.major && a.minor > b.minor);
}

/** La versión MÍNIMA que exige el modelo, por los tipos que usa.
 *
 *  Es el mismo cálculo que `required_version` en el núcleo (que es quien lo
 *  tiene probado); se repite aquí sobre el catálogo ya descargado para no hacer
 *  una ida y vuelta al backend con cada tecla. */
function dzRequired() {
  if (!dz) return null;
  let req = dz.versions[0];
  for (const tbl of design.tables) {
    for (const col of tbl.columns) {
      const ty = dzFind(col.type);
      if (ty && dzNewer(ty.since, req)) req = ty.since;
    }
  }
  return req;
}

// ── Esquemas ─────────────────────────────────────────────────────────────────

/** Los esquemas que el modelo conoce: el del motor, los creados a mano y los de
 *  las tablas que ya los tienen. */
function dzSchemas() {
  const out = [];
  const add = (s) => { if (s && !out.includes(s)) out.push(s); };
  if (dz?.default_schema) add(dz.default_schema);
  (design.schemas || []).forEach(add);
  design.tables.forEach((tbl) => add(tbl.schema));
  return out;
}
/** El esquema efectivo de una tabla: el suyo, o el que el motor pone por
 *  defecto (`dbo`, `public`). */
function tblSchema(tbl) {
  return tbl.schema || dz?.default_schema || "";
}
/** ¿Se está mirando el modelo lógico?
 *
 *  Lógico y físico son el MISMO modelo visto de dos maneras, no dos modelos.
 *  Lo físico sigue siendo la identidad: las relaciones apuntan a columnas por su
 *  nombre físico y el script se escribe con él. El nombre lógico es un rótulo y
 *  nada más, así que ponerlo o quitarlo no puede romper nada. */
function dzLogical() {
  return design.view === "logical";
}

/** El nombre de la tabla en la vista de ahora.
 *
 *  Sin nombre lógico se cae al físico en vez de dejar la caja vacía: un modelo
 *  al que nunca se le han puesto nombres de negocio se ve igual en las dos
 *  vistas, que es lo menos sorprendente que puede pasar. */
function tblName(tbl) {
  return (dzLogical() && tbl.logical) || tbl.name || "";
}
function colDisplay(col) {
  return (dzLogical() && col.logical) || col.name || "";
}

/** La tabla con su esquema, SIEMPRE el nombre físico.
 *
 *  Para lo que es físico de verdad y no cambia de nombre al cambiar de vista:
 *  el diálogo de una clave foránea, que configura un objeto de la base. */
function tblPhys(tbl) {
  const s = tblSchema(tbl);
  return dz?.supports_schemas && s ? `${s}.${tbl.name}` : tbl.name;
}

/** Cómo se nombra la tabla al MOSTRARLA. El SQL no pasa por aquí: lo escribe
 *  `qt` con el nombre físico, que es el único que entiende el motor.
 *
 *  En lógico no lleva esquema: un esquema es una carpeta de la base de datos y
 *  no significa nada en un modelo de negocio. */
function tblLabel(tbl) {
  if (dzLogical()) return tblName(tbl);
  return tblPhys(tbl);
}

// ── Un diálogo propio con campos ─────────────────────────────────────────────

/** El diálogo nativo de Tauri solo sabe preguntar sí/no, y aquí hace falta
 *  pedir un nombre y unos desplegables. Devuelve los valores, o `null` si se
 *  canceló. */
function askFields(title, fields) {
  return new Promise((resolve) => {
    const back = document.createElement("div");
    back.className = "dz-modal-back";
    const card = document.createElement("div");
    card.className = "dz-modal";

    const head = document.createElement("div");
    head.className = "dz-modal-head";
    head.textContent = title;
    card.appendChild(head);

    const body = document.createElement("div");
    body.className = "dz-modal-body";
    const inputs = {};
    for (const f of fields) {
      const lbl = document.createElement("label");
      const span = document.createElement("span");
      span.textContent = f.label;
      lbl.appendChild(span);
      let el;
      if (f.type === "select") {
        el = document.createElement("select");
        for (const o of f.options) {
          const value = typeof o === "object" ? o.value : o;
          const text = typeof o === "object" ? o.label : o;
          el.appendChild(new Option(text, value));
        }
        el.value = f.value ?? "";
      } else {
        el = document.createElement("input");
        el.type = "text";
        el.value = f.value ?? "";
        if (f.placeholder) el.placeholder = f.placeholder;
      }
      inputs[f.key] = el;
      lbl.appendChild(el);
      body.appendChild(lbl);
    }
    card.appendChild(body);

    const foot = document.createElement("div");
    foot.className = "dz-modal-foot";
    const cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "btn ghost";
    cancel.textContent = t("btn.cancel");
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "btn primary";
    ok.textContent = t("btn.ok");
    foot.append(cancel, ok);
    card.appendChild(foot);

    back.appendChild(card);
    document.body.appendChild(back);

    const values = () => {
      const out = {};
      for (const [k, el] of Object.entries(inputs)) out[k] = el.value;
      return out;
    };
    const close = (val) => {
      document.removeEventListener("keydown", onKey);
      back.remove();
      resolve(val);
    };
    const onKey = (e) => {
      if (e.key === "Escape") { e.preventDefault(); close(null); }
      else if (e.key === "Enter" && e.target.tagName !== "SELECT") { e.preventDefault(); close(values()); }
    };
    cancel.addEventListener("click", () => close(null));
    ok.addEventListener("click", () => close(values()));
    back.addEventListener("mousedown", (e) => { if (e.target === back) close(null); });
    document.addEventListener("keydown", onKey);

    const first = Object.values(inputs)[0];
    if (first) { first.focus(); if (first.select) first.select(); }
  });
}

// ── Pintar el diseñador ──────────────────────────────────────────────────────

async function showDesigner(on) {
  // El modo diseño se queda con la ventana entera: árbol, lienzo y barra de
  // estado, como ERwin. El modo comparar sigue siendo una página que se
  // desplaza, que es lo que le va a un informe que se lee de arriba abajo.
  document.body.classList.toggle("dz-full", on);
  document.querySelector(".stepper").hidden = on;
  document.getElementById("designer").hidden = !on;
  document.getElementById("mode-compare").classList.toggle("active", !on);
  document.getElementById("mode-design").classList.toggle("active", on);
  if (on) {
    document.getElementById("step-connection").hidden = true;
    document.getElementById("step-comparison").hidden = true;
    await applyDialect(design.dialect || DZ_DIALECT_DEFAULT);
  } else {
    goToStep(activeStep || "connection");
  }
  window.scrollTo(0, 0);
}

/** Cambia de motor: trae su catálogo, rellena los desplegables del dialecto y
 *  de la versión, y vuelve a pintar. */
async function applyDialect(dialect) {
  design.dialect = dialect;
  dz = await loadDialectInfo(dialect);
  const dsel = document.getElementById("design-dialect");
  if (dsel) dsel.value = dialect;
  fillVersionSelect();
  saveDesign();
  renderDesigner();
}

function fillVersionSelect() {
  const sel = document.getElementById("design-version");
  if (!sel) return;
  if (!dz) { sel.innerHTML = ""; sel.disabled = true; return; }
  sel.disabled = false;
  sel.replaceChildren(...dz.versions.map((v) => new Option(v.label, v.label)));
  const target = dzTarget();
  sel.value = target.label;
  design.version = target.label;
}

/** Quita el generador que el motor no podría aplicar a ese tipo.
 *
 *  Se hace ANTES de pintar: si se dejaba para cuando el editor rehacía sus
 *  filas, la tarjeta ya estaba dibujada y se quedaba anunciando un
 *  NEWSEQUENTIALID sobre un varchar —algo que el motor rechazaría— hasta el
 *  siguiente repintado. También limpia los modelos que llegan de un XML con
 *  una combinación que este motor no admite. */
function normalizeAuto() {
  if (!dz) return;
  let cambio = false;
  for (const tbl of design.tables) {
    for (const c of tbl.columns) {
      if (!c.auto) continue;
      const base = String(c.type || "").toLowerCase();
      const esDe = (lista) => (lista || []).some((n) => n.toLowerCase() === base);
      const vale =
        c.auto === "identity" ? esDe(dz.identity_types)
        : c.auto === "uuid" ? esDe(dz.uuid_types) && !!dz.uuid_default
        : false;
      if (!vale) { c.auto = ""; cambio = true; }
    }
  }
  if (cambio) saveDesign();
}

/** Mientras se escribe en el editor NO hay que rehacer sus filas: rehacerlas
 *  destruye el input donde está el cursor y se pierde lo tecleado. */
let dzKeepEditorRows = false;

/** Repinta el diagrama dejando el editor como está.
 *
 *  Al teclear el nombre de una columna hay que repintar la tarjeta —si no, el
 *  diagrama se queda con el nombre viejo, que es justo lo que pasaba— pero no
 *  las filas del diálogo, porque el cursor está en una de ellas. */
function renderCanvasKeepingEditor() {
  dzKeepEditorRows = true;
  try { renderDesigner(); } finally { dzKeepEditorRows = false; }
}

function renderDesigner() {
  const canvas = document.getElementById("design-canvas");
  if (!canvas) return;
  normalizeAuto();
  canvas.innerHTML = "";
  document.getElementById("design-empty").hidden = design.tables.length > 0;

  const svg = document.createElementNS(SVGNS, "svg");
  svg.setAttribute("class", "dz-edges");
  svg.id = "dz-edges";
  // Las líneas no deben robar el ratón: al soltar una relación hay que saber
  // sobre qué TABLA se soltó, y para eso el punto tiene que ver la tarjeta.
  svg.setAttribute("pointer-events", "none");
  canvas.appendChild(svg);

  for (const tbl of design.tables) canvas.appendChild(buildDesignCard(tbl));
  drawDesignEdges();
  renderTree();
  renderPalette();
  renderViewToggle();
  renderStatus();
  renderVersionNote();
  // Si el editor de una tabla está abierto, sus filas se rehacen también: si no,
  // cambiar ahí un tipo dejaría a la vista los controles del tipo anterior.
  if (dzEditorRefresh && !dzKeepEditorRows) dzEditorRefresh();
  // Los esquemas solo se muestran donde existen: en MySQL el esquema ES la base
  // de datos y en SQLite no hay ninguno. Y en la vista lógica tampoco: un
  // esquema es una carpeta de la base y no significa nada en un modelo de
  // negocio. Esta pasada va al final y pisa lo que pusieran las tarjetas, así
  // que la condición tiene que estar entera aquí.
  const verSchemas = !!dz?.supports_schemas && !dzLogical();
  document.querySelectorAll(".dz-schema-only").forEach((el) => { el.hidden = !verSchemas; });
}

/** La línea que dice qué versión exige el modelo, y avisa si es más nueva que
 *  la elegida. */
function renderVersionNote() {
  const el = document.getElementById("dz-req");
  if (!el) return;
  const req = dzRequired();
  const target = dzTarget();
  if (!req || !design.tables.length) { el.textContent = ""; el.className = "script-info"; return; }
  const motor = dzDialectLabel(design.dialect);
  if (target && dzNewer(req, target)) {
    el.textContent = t("dz.versionTooOld").replace("{m}", motor).replace("{r}", req.label).replace("{v}", target.label);
    el.className = "script-info dz-warn-text";
  } else {
    el.textContent = t("dz.versionNeeds").replace("{m}", motor).replace("{r}", req.label);
    el.className = "script-info";
  }
}

function flagLabel(text, checked, onChange) {
  const lbl = document.createElement("label");
  lbl.className = "dz-flag";
  lbl.title = text;
  const cb = document.createElement("input");
  cb.type = "checkbox";
  cb.checked = !!checked;
  cb.addEventListener("change", () => onChange(cb.checked));
  lbl.append(cb, document.createTextNode(text));
  return lbl;
}

/** La celda del tipo: el desplegable con los tipos del motor y, según el tipo
 *  elegido, la longitud o las dos cifras de precisión y escala. */
function buildTypeCell(col) {
  const wrap = document.createElement("span");
  wrap.className = "dz-type";

  // Sin catálogo: texto libre, para no quedarse sin diseñador.
  if (!dz) {
    const free = document.createElement("input");
    free.type = "text";
    free.className = "dz-c-type";
    free.value = fullType(col);
    free.placeholder = t("dz.colType");
    free.addEventListener("input", () => { Object.assign(col, splitType(free.value)); saveDesign(); });
    wrap.appendChild(free);
    return wrap;
  }

  const sel = document.createElement("select");
  sel.className = "dz-c-sel";
  const groups = new Map();
  for (const ty of dz.types) {
    let g = groups.get(ty.category);
    if (!g) {
      g = document.createElement("optgroup");
      g.label = t("dz.cat." + ty.category);
      groups.set(ty.category, g);
      sel.appendChild(g);
    }
    const o = new Option(ty.deprecated ? ty.name + " ⚠" : ty.name, ty.name);
    if (ty.deprecated) o.title = t("dz.deprecated").replace("{x}", ty.instead || "");
    g.appendChild(o);
  }
  const known = dzFind(col.type);
  // Un tipo escrito a mano (o de otro motor) no se pierde al abrir el modelo.
  sel.appendChild(new Option(t("dz.typeOther"), "__other__"));
  sel.value = known ? known.name : "__other__";

  const other = document.createElement("input");
  other.type = "text";
  other.className = "dz-arg dz-other";
  other.value = known ? "" : col.type || "";
  other.placeholder = t("dz.colType");

  const argA = document.createElement("input");
  argA.type = "text";
  argA.className = "dz-arg";
  argA.value = col.a ?? "";
  const comma = document.createElement("span");
  comma.className = "dz-comma";
  comma.textContent = ",";
  const argB = document.createElement("input");
  argB.type = "text";
  argB.className = "dz-arg";
  argB.value = col.b ?? "";

  // `MAX` de SQL Server: es una palabra, no un número, así que va en su propia
  // casilla en vez de pedir que se escriba dentro del campo de la longitud.
  const maxWrap = document.createElement("label");
  maxWrap.className = "dz-flag dz-max";
  maxWrap.title = t("dz.maxHint");
  const maxCb = document.createElement("input");
  maxCb.type = "checkbox";
  maxWrap.append(maxCb, document.createTextNode("MAX"));

  const refresh = () => {
    const ty = dzFind(col.type);
    const kind = ty ? ty.args.kind : "plain";
    const two = kind === "precision_scale";
    const one = kind === "length" || kind === "precision";
    const esMax = String(col.a).toLowerCase() === "max";

    other.hidden = !!ty;
    argA.hidden = !(one || two) || esMax;
    comma.hidden = !two;
    argB.hidden = !two;
    maxWrap.hidden = !(ty && kind === "length" && ty.args.supports_max);
    maxCb.checked = esMax;

    // Avisos: el tipo pide una versión más nueva que la elegida, o la cifra se
    // pasa de lo que el motor admite. Se avisa, no se impide: puede ser un
    // modelo pensado para otra versión.
    const target = dzTarget();
    const viejo = ty && target && dzNewer(ty.since, target);
    sel.classList.toggle("dz-warn", !!viejo);
    sel.title = viejo
      ? t("dz.needsVersion").replace("{x}", ty.name).replace("{v}", ty.since.label)
      : ty && ty.deprecated
        ? t("dz.deprecated").replace("{x}", ty.instead || "")
        : "";

    const limite = ty && kind === "length" ? ty.args.max : ty && kind === "precision" ? ty.args.max : ty && two ? ty.args.max_precision : 0;
    const pasado = !esMax && limite > 0 && Number(col.a) > limite;
    argA.classList.toggle("dz-warn", !!pasado);
    argA.title = pasado ? t("dz.overMax").replace("{n}", String(limite)) : "";
    if (ty && two) {
      argA.placeholder = t("dz.precision");
      argB.placeholder = t("dz.scale");
    } else if (ty && one) {
      argA.placeholder = kind === "length" ? t("dz.length") : t("dz.precision");
    }
  };

  sel.addEventListener("change", () => {
    if (sel.value === "__other__") {
      col.type = other.value.trim();
      col.a = "";
      col.b = "";
    } else {
      col.type = sel.value;
      // Al elegir el tipo se proponen sus cifras habituales (varchar(50),
      // decimal(18,0)), que es lo que hace ERwin y ahorra escribirlas.
      const a = dzFind(col.type).args;
      if (a.kind === "length") { col.a = String(a.default || ""); col.b = ""; }
      else if (a.kind === "precision") { col.a = a.optional ? "" : String(a.default); col.b = ""; }
      else if (a.kind === "precision_scale") { col.a = String(a.precision); col.b = String(a.scale); }
      else { col.a = ""; col.b = ""; }
    }
    saveDesign();
    renderDesigner();
  });
  other.addEventListener("input", () => { col.type = other.value.trim(); saveDesign(); });
  argA.addEventListener("input", () => { col.a = argA.value; saveDesign(); refresh(); });
  argB.addEventListener("input", () => { col.b = argB.value; saveDesign(); refresh(); });
  maxCb.addEventListener("change", () => {
    const ty = dzFind(col.type);
    col.a = maxCb.checked ? "max" : String(ty?.args?.default ?? "");
    saveDesign();
    renderDesigner();
  });

  wrap.append(sel, other, argA, comma, argB, maxWrap);
  refresh();
  return wrap;
}

/** El nombre que ESE motor le da a lo que rellena la columna solo.
 *
 *  Nada de rotular «uuid» en SQL Server, donde eso no existe: ahí se llama
 *  NEWSEQUENTIALID() y va sobre `uniqueidentifier`. El rótulo sale del propio
 *  catálogo del motor —de la cláusula que se va a emitir— así que no hay una
 *  lista de nombres que mantener aparte ni que se pueda quedar vieja. */
function dzAutoLabel(cual) {
  if (!dz) return "";
  if (cual === "identity") {
    // "IDENTITY(1,1)" y "GENERATED BY DEFAULT AS IDENTITY" se quedan en IDENTITY;
    // "AUTO_INCREMENT" y "AUTOINCREMENT" se quedan como están.
    const s = String(dz.identity_suffix || "").split("(")[0].trim();
    return s.split(/\s+/).pop() || "";
  }
  // "(UUID())" de MySQL viene con el paréntesis del DEFAULT puesto.
  return String(dz.uuid_default || "").replace(/^\(+/, "").split("(")[0].trim();
}

/** La cláusula entera, para el tooltip: ahí sí cabe. */
function dzAutoClause(cual) {
  if (!dz) return "";
  return cual === "identity" ? dz.identity_suffix : "DEFAULT " + dz.uuid_default;
}

/** El control del valor que pone el motor por su cuenta.
 *
 *  Solo aparece cuando el motor sabe rellenar ESE tipo: el contador va sobre
 *  enteros y el identificador único sobre el tipo uuid del motor. Si el tipo no
 *  admite ninguno no se muestra nada, que es más honesto que ofrecer algo que
 *  el motor rechazaría. */
function buildAutoCell(col) {
  const wrap = document.createElement("span");
  wrap.className = "dz-auto";
  if (!dz) return wrap;

  const base = String(col.type || "").toLowerCase();
  const esDe = (lista) => (lista || []).some((n) => n.toLowerCase() === base);
  const puedeContador = esDe(dz.identity_types);
  const puedeUuid = esDe(dz.uuid_types) && !!dz.uuid_default;

  // `normalizeAuto` ya habrá quitado el valor que este tipo no admite, así que
  // aquí solo queda no ofrecer un control que no tiene nada que ofrecer.
  if (!puedeContador && !puedeUuid) return wrap;

  const sel = document.createElement("select");
  sel.className = "dz-auto-sel";
  sel.appendChild(new Option("\u2014", ""));
  if (puedeContador) {
    const o = new Option(dzAutoLabel("identity"), "identity");
    o.title = t("dz.autoId") + " · " + dzAutoClause("identity");
    sel.appendChild(o);
  }
  if (puedeUuid) {
    const o = new Option(dzAutoLabel("uuid"), "uuid");
    o.title = t("dz.autoUuid") + " · " + dzAutoClause("uuid");
    sel.appendChild(o);
  }
  const valido = [...sel.options].some((o) => o.value === col.auto);
  if (!valido && col.auto) { col.auto = ""; saveDesign(); }
  sel.value = col.auto || "";
  sel.title = col.auto
    ? t(col.auto === "identity" ? "dz.autoId" : "dz.autoUuid") + " · " + dzAutoClause(col.auto)
    : t("dz.autoHint");
  sel.addEventListener("change", () => { col.auto = sel.value; saveDesign(); renderDesigner(); });
  wrap.appendChild(sel);
  return wrap;
}

/** Renombrar una columna se lleva consigo las relaciones que la nombran.
 *
 *  Una relación guarda el NOMBRE de la columna, así que sin esto renombrar
 *  dejaba la clave foránea apuntando a una columna que ya no existe: la marca
 *  (FK) desaparecía de la tarjeta y el SQL salía con un nombre inventado.
 *  No se toca nada si otra columna de la tabla sigue llamándose igual que
 *  antes, porque entonces no se sabría a cuál de las dos se refiere. */
function renameColInRelations(tbl, antes, ahora) {
  if (!antes || antes === ahora) return;
  if (tbl.columns.some((c) => c !== undefined && c.name === antes)) return;
  for (const r of design.relations) {
    for (const p of r.cols) {
      if (r.fromTable === tbl.id && p.from === antes) p.from = ahora;
      if (r.toTable === tbl.id && p.to === antes) p.to = ahora;
    }
  }
}

/** Los nombres de las columnas que forman la clave primaria, EN SU ORDEN, que
 *  es el que sale en el `FOREIGN KEY (a, b)` y no da igual. */
function pkNames(tbl) {
  return tbl.columns.filter((c) => c.pk).map((c) => c.name);
}

/** Quita de las relaciones una columna que se acaba de borrar.
 *
 *  Sin esto la relación seguía nombrando una columna que ya no existe: la marca
 *  (FK) se perdía y el script salía con una clave foránea sobre un nombre
 *  inventado. La relación que se queda sin ningún par se va con ellos, porque
 *  una clave foránea sin columnas no es nada. */
function dropColFromRelations(tbl, nombre) {
  for (const r of design.relations) {
    r.cols = r.cols.filter(
      (p) => !((r.fromTable === tbl.id && p.from === nombre) || (r.toTable === tbl.id && p.to === nombre))
    );
  }
  design.relations = design.relations.filter((r) => r.cols.length);
}

function mismoConjunto(a, b) {
  return a.length === b.length && a.every((x) => b.includes(x));
}

/** Pone al día las relaciones que heredaban la clave primaria de `padre`
 *  después de que esa clave haya cambiado.
 *
 *  En ERwin la clave del padre y la que migró a la hija son la misma clave
 *  vista dos veces, así que meter o sacar una columna de la primaria tiene que
 *  mover la foránea con ella. Si no, el modelo dice dos cosas distintas y el
 *  script sale apuntando a columnas que ya no son clave de nadie, que es
 *  justamente lo que el motor rechaza.
 *
 *  Solo se tocan las relaciones que heredaban ESA clave —las que apuntaban
 *  exactamente a las columnas que eran la primaria—; una que señale a propósito
 *  otra columna del padre la puso el usuario y se queda como está. Las que una
 *  tabla se hace a sí misma tampoco: ahí el nombre de la columna de la hija no
 *  puede ser el mismo que el del padre y no hay respuesta automática que valga.
 *
 *  Lo que sale de la clave no se borra de la hija: la columna sigue ahí con sus
 *  datos, solo deja de ser foránea. Y si el padre se queda sin clave ninguna no
 *  se toca nada, porque eso es casi siempre un paso intermedio mientras se
 *  edita y no la intención. */
function resyncRelations(padre, pkAntes) {
  const pkAhora = pkNames(padre);
  if (!pkAhora.length) return false;
  if (pkAntes.join("\u0000") === pkAhora.join("\u0000")) return false;
  let cambio = false;
  for (const r of design.relations) {
    if (r.toTable !== padre.id || r.fromTable === r.toTable) continue;
    if (!mismoConjunto(r.cols.map((p) => p.to), pkAntes)) continue;
    const hija = dzTableById(r.fromTable);
    if (!hija) continue;
    r.cols = pkAhora.map((nombre) => {
      // La columna que ya estaba conserva el nombre que tenga en la hija: puede
      // no ser el del padre y es decisión del usuario.
      const ya = r.cols.find((p) => p.to === nombre);
      if (ya) return ya;
      migrateKeyColumn(hija, nombre, padre.columns.find((c) => c.name === nombre));
      return { from: nombre, to: nombre };
    });
    cambio = true;
  }
  return cambio;
}

/** Hace un cambio que puede mover la clave primaria de `tbl` y deja detrás las
 *  relaciones al día. Las dos cosas que la mueven —marcar PK y reordenar las
 *  columnas— pasan por aquí para que ninguna se olvide. */
function withPkSync(tbl, hacer) {
  const antes = pkNames(tbl);
  hacer();
  resyncRelations(tbl, antes);
  saveDesign();
  renderDesigner();
}

/** Una fila del EDITOR: aquí sí se escribe. La tarjeta del lienzo no usa esto.
 *
 *  En ERwin la entidad del diagrama es una vista y se edita en su propio
 *  diálogo; meter el formulario dentro de la caja obligaba a estrujar siete
 *  controles en el ancho de una tabla. */
function buildColRow(tbl, col, idx) {
  const li = document.createElement("li");
  li.className = "dz-col";

  const name = document.createElement("input");
  name.type = "text";
  name.className = "dz-c-name";
  name.value = dzLogical() ? col.logical : col.name;
  name.placeholder = dzLogical() ? col.name || t("dz.colName") : t("dz.colName");
  name.addEventListener("input", () => {
    if (dzLogical()) {
      // El nombre lógico no lo nombra nada: no hay relaciones que arrastrar.
      col.logical = name.value;
    } else {
      const antes = col.name;
      col.name = name.value;
      renameColInRelations(tbl, antes, col.name);
    }
    saveDesign();
    renderCanvasKeepingEditor();
  });

  const mover = (delta) => {
    const destino = idx + delta;
    if (destino < 0 || destino >= tbl.columns.length) return;
    // Por `withPkSync` porque mover una columna de la clave cambia el ORDEN de
    // la primaria, y ese es el orden del `FOREIGN KEY (a, b)`.
    withPkSync(tbl, () => {
      const [c] = tbl.columns.splice(idx, 1);
      tbl.columns.splice(destino, 0, c);
    });
  };
  const subir = document.createElement("button");
  subir.type = "button";
  subir.className = "dz-icon-btn";
  subir.textContent = "↑";
  subir.title = t("dz.up");
  subir.disabled = idx === 0;
  subir.addEventListener("click", () => mover(-1));
  const bajar = document.createElement("button");
  bajar.type = "button";
  bajar.className = "dz-icon-btn";
  bajar.textContent = "↓";
  bajar.title = t("dz.down");
  bajar.disabled = idx === tbl.columns.length - 1;
  bajar.addEventListener("click", () => mover(1));

  const del = document.createElement("button");
  del.type = "button";
  del.className = "dz-icon-btn";
  del.textContent = "✕";
  del.title = t("dz.delCol");
  del.addEventListener("click", () => {
    const [ido] = tbl.columns.splice(idx, 1);
    // Aquí no hace falta resincronizar: quitando una columna de la clave no
    // aparece ninguna nueva que migrar, y los pares que la nombraban se van con
    // ella, que es exactamente la clave que queda.
    dropColFromRelations(tbl, ido.name);
    saveDesign();
    renderDesigner();
  });

  li.append(
    name,
    buildTypeCell(col),
    buildAutoCell(col),
    flagLabel("PK", col.pk, (v) => withPkSync(tbl, () => { col.pk = v; })),
    flagLabel("NN", col.nn, (v) => { col.nn = v; saveDesign(); renderCanvasKeepingEditor(); }),
    subir,
    bajar,
    del
  );
  return li;
}

/** ¿Esta columna es el lado hijo de alguna relación? Entonces lleva (FK). */
function colIsFk(tbl, col) {
  return design.relations.some(
    (r) => r.fromTable === tbl.id && r.cols.some((p) => p.from === col.name)
  );
}

/** ¿La tabla depende de otra para identificarse?
 *
 *  Lo es cuando alguna columna de su clave primaria es además foránea: la fila
 *  no existe sin su padre. ERwin las dibuja con las esquinas redondeadas. */
function tableIsDependent(tbl) {
  return tbl.columns.some((c) => c.pk && colIsFk(tbl, c));
}

/** La columna escrita como la escribe ERwin: `Nombre: varchar(500) NOT NULL`.
 *
 *  Una sola línea y no el nombre a un lado y el tipo al otro. Con quince
 *  columnas —que es lo que tiene una tabla de verdad— dos columnas de texto se
 *  pelean por el ancho de la caja y acaban las dos recortadas; en una línea, la
 *  caja mide lo que mida la más larga y se lee entera.
 *
 *  El orden es el del DDL (`int IDENTITY(1,1) NOT NULL`) para que lo que se lee
 *  en el diagrama sea lo que va a salir en el script. La nulabilidad se escribe
 *  siempre, también cuando admite nulos: en un diagrama que se mira de lejos,
 *  que falte una palabra no se distingue de que nadie la haya decidido. */
function dzColLine(tbl, col) {
  // En lógico, solo el nombre. Un tipo de motor no existe en un modelo de
  // negocio, y enseñar `varchar(500)` en una vista que se llama lógica sería
  // decir que sí existe.
  if (dzLogical()) return colDisplay(col) + (colIsFk(tbl, col) ? " (FK)" : "");
  let s = col.name + ": " + fullType(col);
  if (col.auto && dz) s += " " + dzAutoLabel(col.auto);
  s += col.pk || col.nn ? " NOT NULL" : " NULL";
  if (colIsFk(tbl, col)) s += " (FK)";
  return s;
}

/** Una fila de la TARJETA: solo se lee. */
function buildCardRow(tbl, col) {
  const li = document.createElement("li");
  li.className = "dz-ro";
  li.dataset.col = col.name;

  // La agarradera de la relación: de aquí se tira hasta la otra tabla.
  const grip = document.createElement("span");
  grip.className = "dz-grip";
  grip.title = t("dz.dragHint");
  grip.addEventListener("mousedown", (e) => startLink(e, tbl, col));

  const txt = document.createElement("span");
  txt.className = "dz-ro-txt";
  txt.textContent = dzColLine(tbl, col);
  // El título trae la línea entera por si la caja la recorta: así no hay que
  // abrir el editor para leer un tipo largo.
  txt.title = txt.textContent;

  li.append(grip, txt);
  return li;
}

/** La tarjeta del lienzo: la entidad, como la dibuja ERwin. La clave primaria
 *  arriba, una línea, y debajo lo demás. No se escribe aquí: doble clic. */
function buildDesignCard(tbl) {
  const card = document.createElement("div");
  card.className = "dz-table" + (tableIsDependent(tbl) ? " dependiente" : "");
  card.dataset.id = tbl.id;
  if (tbl.color) card.dataset.color = tbl.color;
  card.style.left = (tbl.x || 20) + "px";
  card.style.top = (tbl.y || 20) + "px";
  // El tamaño es del usuario: lo ajusta tirando de la esquina y se guarda. Sin
  // alto la tarjeta mide lo que midan sus columnas, que es lo corriente; con él
  // la lista se desplaza dentro y la tarjeta deja de comerse el lienzo.
  if (tbl.w) card.style.width = tbl.w + "px";
  if (tbl.h) card.style.height = tbl.h + "px";
  card.title = t("dz.editHint");
  if (dzSelEs("table", tbl.id)) card.classList.add("sel");
  // Pulsar la tarjeta la selecciona en el árbol: el lienzo y el árbol son dos
  // vistas de lo mismo y señalar en una tiene que verse en la otra.
  card.addEventListener("mousedown", () => dzSelect("table", tbl.id));
  // Con una herramienta de relación armada, la tarjeta ENTERA es la agarradera:
  // se sale del padre y se suelta en la hija, sin apuntar a una columna. En
  // captura para adelantarse al arrastre de mover y a la selección, que si no
  // se quedarían con el ratón antes de llegar aquí.
  card.addEventListener("mousedown", (e) => {
    if (dzTool === "select" || dzTool === "table") return;
    if (e.target.closest("button, input, select")) return;
    startLink(e, tbl, null);
  }, true);
  card.addEventListener("dblclick", (e) => {
    if (e.target.closest("button, .dz-resize, .dz-grip")) return;
    openTableEditor(tbl);
  });

  const head = document.createElement("div");
  head.className = "dz-head";

  // De aquí se agarra la tabla para moverla.
  const mover = document.createElement("span");
  mover.className = "dz-move";
  mover.title = t("dz.move");
  mover.textContent = "∷";

  const nombre = document.createElement("span");
  nombre.className = "dz-name-ro";
  nombre.textContent = tblName(tbl);

  const esquema = document.createElement("span");
  esquema.className = "dz-schema-ro dz-schema-only";
  esquema.hidden = !dz?.supports_schemas || dzLogical();
  esquema.textContent = tblSchema(tbl);
  esquema.title = t("dz.schemaOf");

  const editar = document.createElement("button");
  editar.type = "button";
  editar.className = "dz-icon-btn";
  editar.textContent = "✎";
  editar.title = t("dz.editCols");
  editar.addEventListener("click", () => openTableEditor(tbl));

  const delBtn = document.createElement("button");
  delBtn.type = "button";
  delBtn.className = "dz-icon-btn";
  delBtn.textContent = "✕";
  delBtn.title = t("dz.delTable");
  delBtn.addEventListener("click", () => {
    design.tables = design.tables.filter((x) => x.id !== tbl.id);
    design.relations = design.relations.filter((r) => r.fromTable !== tbl.id && r.toTable !== tbl.id);
    saveDesign();
    renderDesigner();
  });

  head.append(mover, nombre, esquema, editar, delBtn);
  makeDesignDraggable(card, head, tbl);
  card.appendChild(head);

  // Dos compartimentos, como en ERwin: la clave primaria arriba, separada por
  // una línea de lo demás. Es lo que distingue de un vistazo qué identifica la
  // fila de qué la describe.
  const claves = tbl.columns.filter((c) => c.pk);
  const resto = tbl.columns.filter((c) => !c.pk);

  const ulPk = document.createElement("ul");
  ulPk.className = "dz-cols dz-pk-box";
  for (const c of claves) ulPk.appendChild(buildCardRow(tbl, c));
  if (claves.length) card.appendChild(ulPk);

  const ul = document.createElement("ul");
  ul.className = "dz-cols";
  for (const c of resto) ul.appendChild(buildCardRow(tbl, c));
  card.appendChild(ul);

  if (!tbl.columns.length) {
    const vacio = document.createElement("div");
    vacio.className = "dz-ro-empty";
    vacio.textContent = t("dz.noCols");
    card.appendChild(vacio);
  }

  const tirador = document.createElement("span");
  tirador.className = "dz-resize";
  tirador.title = t("dz.resize");
  tirador.addEventListener("mousedown", (e) => startResize(e, card, tbl));
  // Doble clic en el tirador: la tarjeta vuelve a medir lo que midan sus
  // columnas. Es la salida de un alto que quedó mal y no hay que adivinarla,
  // porque la propia esquina es donde uno la busca.
  tirador.addEventListener("dblclick", (e) => {
    e.preventDefault();
    e.stopPropagation();
    delete tbl.w;
    delete tbl.h;
    saveDesign();
    renderDesigner();
  });
  card.appendChild(tirador);
  return card;
}

// ── El editor de la tabla ────────────────────────────────────────────────────

/** Lo que vuelve a pintar la lista de columnas del editor cuando está abierto.
 *  `renderDesigner` lo llama para que el diálogo y el lienzo no se separen:
 *  cambiar un tipo en el diálogo rehace sus controles y además la tarjeta. */
let dzEditorRefresh = null;

/** El editor de la tabla, que en ERwin sale al hacer doble clic en la entidad.
 *
 *  Los cambios entran en el modelo según se hacen, sin Aceptar ni Cancelar: así
 *  la tarjeta de detrás va enseñando el resultado, que es media gracia de tener
 *  el lienzo. Para deshacer está quitar lo puesto. */
function openTableEditor(tbl) {
  const back = document.createElement("div");
  back.className = "dz-modal-back";
  const caja = document.createElement("div");
  caja.className = "dz-modal dz-editor";

  const head = document.createElement("div");
  head.className = "dz-modal-head";
  head.textContent = t("dz.tableTitle");
  caja.appendChild(head);

  const body = document.createElement("div");
  body.className = "dz-modal-body";

  // Nombre y esquema de la tabla
  const arriba = document.createElement("div");
  arriba.className = "dz-ed-head";

  // En lógico se escribe el nombre lógico, en físico el físico. La vista manda
  // qué nombre se ve Y cuál se escribe: si se vieran los dos a la vez habría que
  // explicar cuál es cuál en cada caja.
  const lblNombre = document.createElement("label");
  const spNombre = document.createElement("span");
  spNombre.textContent = dzLogical() ? t("dz.logicalName") : t("dz.tableName");
  const inNombre = document.createElement("input");
  inNombre.type = "text";
  inNombre.value = dzLogical() ? tbl.logical : tbl.name;
  // Vacío quiere decir «el mismo que el físico», y el marcador de posición lo
  // enseña para que no parezca que falta algo.
  if (dzLogical()) inNombre.placeholder = tbl.name;
  inNombre.addEventListener("input", () => {
    if (dzLogical()) tbl.logical = inNombre.value;
    else tbl.name = inNombre.value;
    saveDesign();
    renderCanvasKeepingEditor();
  });
  lblNombre.append(spNombre, inNombre);
  arriba.appendChild(lblNombre);

  if (dz?.supports_schemas && !dzLogical()) {
    const lblEsq = document.createElement("label");
    const spEsq = document.createElement("span");
    spEsq.textContent = t("dz.schema");
    const selEsq = document.createElement("select");
    const rellenar = () => {
      selEsq.replaceChildren(...dzSchemas().map((s) => new Option(s, s)), new Option(t("dz.newSchema"), "__new__"));
      selEsq.value = tblSchema(tbl);
    };
    rellenar();
    selEsq.addEventListener("change", async () => {
      if (selEsq.value === "__new__") {
        const nuevo = await askNewSchema();
        tbl.schema = nuevo || tblSchema(tbl);
        rellenar();
      } else {
        tbl.schema = selEsq.value;
      }
      saveDesign();
      renderDesigner();
      rellenar();
    });
    lblEsq.append(spEsq, selEsq);
    arriba.appendChild(lblEsq);
  }
  // El color de la tabla, junto al nombre y al esquema: es una propiedad de la
  // tabla como las otras dos y aquí es donde se buscan.
  const lblColor = document.createElement("label");
  lblColor.className = "dz-color-pick";
  const spColor = document.createElement("span");
  spColor.textContent = t("dz.color");
  const muestras = document.createElement("div");
  muestras.className = "dz-colors";
  const marcar = () => {
    muestras.querySelectorAll(".dz-swatch").forEach((b) => {
      b.classList.toggle("active", (tbl.color || "") === b.dataset.color);
    });
  };
  for (const c of DZ_COLORS) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "dz-swatch";
    b.dataset.color = c.id;
    b.title = t(c.key);
    b.addEventListener("click", () => {
      tbl.color = c.id;
      saveDesign();
      marcar();
      // Sin rehacer las filas del editor: solo cambió el color de la caja.
      renderCanvasKeepingEditor();
    });
    muestras.appendChild(b);
  }
  marcar();
  lblColor.append(spColor, muestras);
  arriba.appendChild(lblColor);

  body.appendChild(arriba);

  // La cabecera de la rejilla. Va fuera de la lista para que no se desplace con
  // ella, y usa la misma plantilla de columnas, que es lo que hace que una fila
  // de `uniqueidentifier` (sin longitud) y una de `varchar(500)` se lean como
  // una tabla y no como dos filas de anchos distintos.
  const th = document.createElement("div");
  th.className = "dz-col dz-ed-th" + (dzLogical() ? " logica" : "");
  const cabeceras = dzLogical()
    ? [t("dz.logicalCol"), "PK", "NN"]
    : [t("dz.colName"), t("dz.colType"), t("dz.thAuto"), "PK", "NN"];
  for (const txt of cabeceras) {
    const sp = document.createElement("span");
    sp.textContent = txt;
    th.appendChild(sp);
  }
  body.appendChild(th);

  const ul = document.createElement("ul");
  ul.className = "dz-cols dz-ed-cols" + (dzLogical() ? " logica" : "");
  body.appendChild(ul);

  const pintar = () => {
    ul.replaceChildren();
    tbl.columns.forEach((c, i) => ul.appendChild(buildColRow(tbl, c, i)));
    if (!tbl.columns.length) {
      const p = document.createElement("p");
      p.className = "empty";
      p.textContent = t("dz.noCols");
      ul.appendChild(p);
    }
  };
  pintar();

  const addCol = document.createElement("button");
  addCol.type = "button";
  addCol.className = "dz-add-col";
  addCol.textContent = "+ " + t("dz.addCol");
  addCol.addEventListener("click", () => {
    tbl.columns.push(newColumn());
    saveDesign();
    renderDesigner();
  });
  body.appendChild(addCol);
  caja.appendChild(body);

  const foot = document.createElement("div");
  foot.className = "dz-modal-foot";
  const cerrar = document.createElement("button");
  cerrar.type = "button";
  cerrar.className = "btn primary";
  cerrar.textContent = t("dz.close");
  foot.appendChild(cerrar);
  caja.appendChild(foot);

  back.appendChild(caja);
  document.body.appendChild(back);

  // Mientras el diálogo esté abierto, cada repintado del lienzo repinta también
  // sus filas: si no, cambiar un tipo dejaría aquí los controles del anterior.
  dzEditorRefresh = pintar;
  const cerrarlo = () => {
    dzEditorRefresh = null;
    document.removeEventListener("keydown", onKey);
    back.remove();
  };
  const onKey = (e) => { if (e.key === "Escape") { e.preventDefault(); cerrarlo(); } };
  cerrar.addEventListener("click", cerrarlo);
  back.addEventListener("mousedown", (e) => { if (e.target === back) cerrarlo(); });
  document.addEventListener("keydown", onKey);
  inNombre.focus();
  inNombre.select();
}

/** Ajusta el ancho de la tabla tirando de su esquina.
 *
 *  Las dos medidas. El alto no deja un hueco debajo: la lista de columnas se
 *  queda dentro y se desplaza, así que una tabla de treinta columnas cabe en la
 *  tarjeta que uno le dé en vez de estirarse y tapar el resto del diagrama. */
function startResize(e, card, tbl) {
  e.preventDefault();
  e.stopPropagation();
  const x0 = e.clientX;
  const y0 = e.clientY;
  const w0 = card.offsetWidth;
  const h0 = card.offsetHeight;
  const move = (ev) => {
    tbl.w = Math.max(240, Math.round(w0 + (ev.clientX - x0)));
    card.style.width = tbl.w + "px";
    // El suelo deja ver la cabecera y un par de filas: por debajo de eso la
    // tarjeta no dice ni de qué tabla es.
    tbl.h = Math.max(DZ_MIN_H, Math.round(h0 + (ev.clientY - y0)));
    card.style.height = tbl.h + "px";
    drawDesignEdges();
  };
  const up = () => {
    document.removeEventListener("mousemove", move);
    document.removeEventListener("mouseup", up);
    card.classList.remove("resizing");
    saveDesign();
  };
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", up);
  card.classList.add("resizing");
}

/** Una columna nueva, con el tipo de texto más corriente del motor. */
function newColumn(name) {
  const texto = dz ? dzFind("varchar") || dzFind("TEXT") || dz.types[0] : null;
  const col = { name: name || "columna", type: texto ? texto.name : "TEXT", a: "", b: "", pk: false, nn: false, auto: "" };
  if (texto && texto.args.kind === "length") col.a = String(texto.args.default || "");
  return col;
}

/** La columna clave de una tabla nueva.
 *
 *  Donde el motor tiene un tipo uuid PROPIO se usa ese, con su generador por
 *  defecto: `uniqueidentifier` en SQL Server y `uuid` en PostgreSQL. Donde no
 *  lo tiene se queda el contador, porque el uuid ahí sale prestado —MySQL lo
 *  guardaría en un `char(36)` y SQLite no trae con qué generarlo— y un contador
 *  de verdad es mejor que un uuid de mentira. */
function newPkColumn() {
  if (dz && dz.uuid_default) {
    const ty = (dz.uuid_types || []).map((n) => dzFind(n)).find(Boolean);
    if (ty && /^(uuid|uniqueidentifier)$/i.test(ty.name)) {
      return { name: "id", type: ty.name, a: "", b: "", pk: true, nn: true, auto: "uuid" };
    }
  }
  return { name: "id", type: dzIntType(), a: "", b: "", pk: true, nn: true, auto: dz ? "identity" : "" };
}

/** El entero del motor: `int` en SQL Server y MySQL, `integer` en PostgreSQL,
 *  `INTEGER` en SQLite. Sin catálogo, el nombre que los cuatro entienden. */
function dzIntType() {
  const ty = dz ? dzFind("int") || dzFind("integer") || dzFind("INTEGER") : null;
  return ty ? ty.name : "INTEGER";
}

function makeDesignDraggable(card, handle, tbl) {
  handle.addEventListener("mousedown", (e) => {
    if (e.target.closest("input, button, select")) return; // no arrastres al editar
    e.preventDefault();
    const start = { x: e.clientX, y: e.clientY, left: tbl.x || 20, top: tbl.y || 20 };
    const move = (ev) => {
      tbl.x = Math.max(0, Math.round(start.left + (ev.clientX - start.x)));
      tbl.y = Math.max(0, Math.round(start.top + (ev.clientY - start.y)));
      card.style.left = tbl.x + "px";
      card.style.top = tbl.y + "px";
      drawDesignEdges();
    };
    const up = () => {
      document.removeEventListener("mousemove", move);
      document.removeEventListener("mouseup", up);
      handle.classList.remove("dragging");
      saveDesign();
    };
    document.addEventListener("mousemove", move);
    document.addEventListener("mouseup", up);
    handle.classList.add("dragging");
  });
}

// ── Relacionar arrastrando ───────────────────────────────────────────────────

/** Empieza a tirar una relación desde una columna. Mientras se arrastra se
 *  dibuja una línea de goma; al soltar sobre otra tabla sale el diálogo. */
function startLink(e, tbl, col) {
  e.preventDefault();
  e.stopPropagation();
  const canvas = document.getElementById("design-canvas");
  const caja = canvas.getBoundingClientRect();
  const punto = (ev) => ({
    x: ev.clientX - caja.left + canvas.scrollLeft,
    y: ev.clientY - caja.top + canvas.scrollTop,
  });
  const inicio = punto(e);
  // La herramienta se guarda al empezar: si cambia a mitad del arrastre, manda
  // la que estaba cuando se pulsó. Con la flecha, lo de siempre.
  const tipo = dzTool === "select" || dzTool === "table" ? "noident" : dzTool;
  dzLink = { tbl, col, tipo, from: inicio, to: inicio, movido: false };
  canvas.classList.add("dz-linking");

  const move = (ev) => {
    dzLink.to = punto(ev);
    if (Math.abs(dzLink.to.x - inicio.x) + Math.abs(dzLink.to.y - inicio.y) > 6) dzLink.movido = true;
    drawDesignEdges();
  };
  const up = async (ev) => {
    document.removeEventListener("mousemove", move);
    document.removeEventListener("mouseup", up);
    canvas.classList.remove("dz-linking");
    const tirada = dzLink;
    dzLink = null;
    drawDesignEdges();
    if (!tirada.movido) return; // un clic suelto no es una relación

    const el = document.elementFromPoint(ev.clientX, ev.clientY);
    const card = el && el.closest ? el.closest(".dz-table") : null;
    if (!card) return;
    const destino = dzTableById(card.dataset.id);
    if (!destino) return;
    // Soltar en la misma columna de la que se salió no es nada; en otra columna
    // de la misma tabla sí (una jerarquía que se referencia a sí misma).
    // La fila se busca por su NOMBRE y no por su posición: la tarjeta tiene dos
    // compartimentos (la clave y lo demás), así que el índice no dice cuál es.
    const fila = el.closest(".dz-ro");
    const colDestino = fila ? destino.columns.find((c) => c.name === fila.dataset.col) : null;
    if (destino.id === tirada.tbl.id && colDestino === tirada.col) return;
    // Se arrastra del PADRE a la HIJA, como en ERwin: se sale de la clave que se
    // va a heredar y se suelta sobre la tabla que la recibe.
    if (tirada.tipo === "m2m") makeBridge(tirada.tbl, destino);
    else await askRelation(destino, colDestino, tirada.tbl, tirada.col, tirada.tipo);
    // La herramienta se usa una vez y vuelve la flecha: si se quedara armada no
    // se podría mover una tabla sin acordarse de desarmarla.
    dzSetTool("select");
  };
  document.addEventListener("mousemove", move);
  document.addEventListener("mouseup", up);
}

/** Las acciones que el motor admite al borrar o actualizar el padre. */
function dzActions() {
  const base = [
    { value: "", label: "NO ACTION" },
    { value: "CASCADE", label: "CASCADE" },
    { value: "SET NULL", label: "SET NULL" },
    { value: "SET DEFAULT", label: "SET DEFAULT" },
  ];
  // RESTRICT no existe en SQL Server; en los otros tres sí.
  if (design.dialect !== "sqlserver") base.push({ value: "RESTRICT", label: "RESTRICT" });
  return base;
}

/** El nombre que se propone para una relación nueva: la costumbre
 *  `FK_hija_padre`, con un número detrás si ya está usado. */
/** El nombre por defecto de la relación: fk + Hija + Has + Padre.
 *
 *  `fkUserroleHasUser` se lee como una frase y dice de un vistazo quién depende
 *  de quién, que es lo que uno busca al mirar una lista de claves foráneas. Si
 *  ya existe se le pone un número detrás. */
function dzRelName(hija, padre) {
  const base = "fk" + pascal(hija.name) + "Has" + pascal(padre.name);
  const usados = new Set(design.relations.map((r) => (r.name || "").toLowerCase()));
  if (!usados.has(base.toLowerCase())) return base;
  for (let i = 2; i < 100; i++) {
    const n = base + i;
    if (!usados.has(n.toLowerCase())) return n;
  }
  return base;
}

/** La columna de la hija que recibe la clave foránea, creándola si no está.
 *
 *  Es la «migración de atributos» de ERwin: la clave del padre baja a la hija
 *  con su mismo nombre y su mismo tipo —si no coincidieran, el motor no dejaría
 *  crear la clave foránea— y nace NOT NULL y fuera de la clave primaria, que es
 *  una relación NO identificadora: la fila de la hija existe por sí misma.
 *  Si la hija ya tiene esa columna se usa la suya y no se le toca el tipo.
 *
 *  `identificadora` dice si nace DENTRO de la clave primaria de la hija, que es
 *  toda la diferencia entre los dos tipos de relación. */
function migrateKeyColumn(hija, nombre, colPadre, identificadora) {
  const ya = hija.columns.find((c) => c.name === nombre);
  if (ya) return ya;
  const col = {
    name: nombre,
    type: colPadre ? colPadre.type : "",
    a: colPadre ? colPadre.a : "",
    b: colPadre ? colPadre.b : "",
    pk: !!identificadora,
    nn: true,
    // Nunca un generador: ese valor lo pone el padre, no el motor.
    auto: "",
  };
  hija.columns.push(col);
  return col;
}


/** Las columnas del padre que migran a la hija.
 *
 *  En ERwin la relación hereda la clave primaria ENTERA del padre, así que una
 *  primaria compuesta da una foránea compuesta sin que haya que pedir nada: son
 *  la misma clave vista desde la hija, y partirla la dejaría sin sentido. Solo
 *  cuando se arrastra de una columna que NO es parte de la primaria se hereda
 *  esa sola, porque ahí la intención es señalar esa columna y no la clave. */
function parentKeyCols(padre, colPadre) {
  if (colPadre && !colPadre.pk) return [colPadre];
  const pk = padre.columns.filter((c) => c.pk);
  if (pk.length) return pk;
  return colPadre ? [colPadre] : padre.columns.slice(0, 1);
}

/** Los campos del diálogo que eligen en qué columna de la hija cae cada parte
 *  de la clave del padre: una fila por columna, que es lo que vuelve compuesta
 *  a una clave foránea. `proponer(col, i)` da el nombre preseleccionado. */
function relColFields(hija, padre, clave, proponer) {
  return clave.map((c, i) => {
    const propuesto = proponer(c, i);
    const opciones = hija.columns.map((x) => x.name);
    // La columna propuesta puede no existir todavía en la hija: se ofrece igual
    // y se crea al aceptar.
    if (propuesto && !opciones.includes(propuesto)) opciones.unshift(propuesto);
    return {
      key: "c" + i,
      label: t("dz.relMigrate")
        .replace("{p}", tblPhys(padre) + "." + c.name)
        .replace("{t}", tblPhys(hija)),
      type: "select",
      options: opciones,
      value: propuesto,
    };
  });
}

/** Lee del diálogo los pares columna-de-la-hija → columna-del-padre.
 *
 *  Dos partes de la clave no pueden caer en la misma columna de la hija —el
 *  motor no admite una foránea con la columna repetida—, así que la segunda se
 *  descarta. Con los nombres que se proponen eso no pasa nunca; solo si se
 *  elige la misma a mano. */
function relColsFromVals(vals, clave) {
  const cols = [];
  clave.forEach((c, i) => {
    const nombre = vals["c" + i];
    if (nombre && !cols.some((p) => p.from === nombre)) cols.push({ from: nombre, to: c.name });
  });
  return cols;
}

/** Mete (o saca) de la clave primaria de la hija las columnas de la relación.
 *
 *  Eso ES el tipo de relación, no una etiqueta que lo acompañe: identificadora
 *  quiere decir que la clave del padre forma parte de la clave de la hija, y por
 *  eso su fila no existe sin él. No se guarda en ninguna parte —`relIsIdentifying`
 *  lo lee de las columnas— así que el rótulo no puede decir una cosa mientras el
 *  modelo dice otra, ni el script salir de una tercera. */
function applyRelKind(hija, cols, identificadora) {
  for (const p of cols) {
    const c = hija.columns.find((x) => x.name === p.from);
    if (c) c.pk = !!identificadora;
  }
}

/** Los dos campos del diálogo que eligen el tipo. */
function relKindField(identificadora) {
  return {
    key: "kind",
    label: t("dz.relType"),
    type: "select",
    options: [
      { value: "noident", label: t("dz.typeNoIdent") },
      { value: "ident", label: t("dz.typeIdent") },
    ],
    value: identificadora ? "ident" : "noident",
  };
}

/** Resuelve un muchos a muchos: crea la tabla puente con las dos claves.
 *
 *  Un muchos a muchos no existe en una base de datos; se escribe con una tabla
 *  en medio. ERwin lo dibuja como una línea en el modelo lógico y lo resuelve al
 *  pasar a físico; aquí, que es físico desde el principio, se crea ya.
 *
 *  La puente lleva su propia clave sustituta y las dos foráneas FUERA de la
 *  primaria. Hacerla con una primaria compuesta de las dos claves también sería
 *  correcto, pero esta es la forma que tienen las demás tablas que crea el
 *  diseñador, y marcar PK en las dos columnas la convierte en la otra. */
function makeBridge(a, b) {
  // Una tabla consigo misma sí admite puente (una jerarquía de muchos a muchos),
  // pero entonces las dos claves se llaman igual y hay que distinguirlas: eso lo
  // resuelve `nombreLibre` poniendo el nombre de la tabla detrás.
  const base = pascal(a.name) + pascal(b.name);
  const usados = new Set(design.tables.map((x) => String(x.name || "").toLowerCase()));
  let nombre = base;
  for (let i = 2; usados.has(nombre.toLowerCase()); i++) nombre = base + i;

  const puente = {
    id: "t" + design.seq++,
    name: nombre,
    schema: a.schema || dz?.default_schema || "",
    x: Math.round(((a.x || 20) + (b.x || 20)) / 2),
    y: Math.round(Math.max(a.y || 20, b.y || 20)) + 220,
    columns: [newPkColumn()],
  };
  design.tables.push(puente);

  const nombreLibre = (padre, col) =>
    puente.columns.some((x) => x.name === col.name) ? col.name + pascal(padre.name) : col.name;

  for (const padre of [a, b]) {
    const clave = parentKeyCols(padre, null);
    const cols = [];
    for (const c of clave) {
      const nom = nombreLibre(padre, c);
      migrateKeyColumn(puente, nom, c, false);
      cols.push({ from: nom, to: c.name });
    }
    if (!cols.length) continue;
    design.relations.push({
      id: "r" + design.seq++,
      name: dzRelName(puente, padre),
      fromTable: puente.id,
      toTable: padre.id,
      cols,
      onDelete: "",
      onUpdate: "",
    });
  }
  saveDesign();
  renderDesigner();
  dzSelect("table", puente.id);
  const el = document.getElementById("dz-status");
  if (el) el.textContent = t("dz.bridgeMade").replace("{t}", tblPhys(puente));
}

/** El diálogo de la relación, como el de ERwin: el nombre, las columnas que la
 *  hija hereda y qué hacer cuando el padre cambia o desaparece. */
async function askRelation(hija, colHija, padre, colPadre, tipo) {
  const acciones = dzActions();
  const clave = parentKeyCols(padre, colPadre);
  if (!clave.length) return;
  // El nombre que se propone en la hija es el mismo de la columna del padre,
  // que es como se lee un modelo de ERwin. Solo si se soltó justo encima de una
  // columna de la hija, y la clave es de una sola, manda esa.
  //
  // Si la hija YA tiene una columna con ese nombre, se propone con el nombre de
  // la tabla padre detrás: con la clave sustituta que pone el diseñador, todas
  // las tablas tienen una columna `id`, y heredar `id` sobre `id` convertiría la
  // propia clave de la hija en foránea sin que nadie lo haya pedido. Es el
  // «rolename» de ERwin.
  const proponer = (c) => {
    if (clave.length === 1 && colHija) return colHija.name;
    return hija.columns.some((x) => x.name === c.name) ? c.name + pascal(padre.name) : c.name;
  };
  const campos = [
    { key: "name", label: t("dz.relName"), value: dzRelName(hija, padre) },
    relKindField(tipo === "ident"),
    ...relColFields(hija, padre, clave, proponer),
    { key: "onDelete", label: t("dz.onDelete"), type: "select", options: acciones, value: "" },
    { key: "onUpdate", label: t("dz.onUpdate"), type: "select", options: acciones, value: "" },
  ];
  const vals = await askFields(t("dz.relTitle"), campos);
  if (!vals) return;
  const cols = relColsFromVals(vals, clave);
  if (!cols.length) return;
  const ident = vals.kind === "ident";
  // Ahora sí: las columnas que la hija no tenía se crean heredando las del padre.
  for (const p of cols) migrateKeyColumn(hija, p.from, padre.columns.find((c) => c.name === p.to), ident);
  applyRelKind(hija, cols, ident);
  design.relations.push({
    id: "r" + design.seq++,
    name: vals.name.trim(),
    fromTable: hija.id,
    toTable: padre.id,
    cols,
    onDelete: vals.onDelete,
    onUpdate: vals.onUpdate,
  });
  saveDesign();
  renderDesigner();
}

/** El mismo diálogo, para cambiar una relación que ya existe. */
async function editRelation(r) {
  const hija = dzTableById(r.fromTable);
  const padre = dzTableById(r.toTable);
  if (!hija || !padre) return;
  const acciones = dzActions();
  // El lado del padre no se toca aquí: lo decide su clave, y cambiarlo a mano
  // haría una foránea que apunta a columnas que no son clave de nadie. Para
  // enganchar otra clave se borra la relación y se vuelve a arrastrar.
  const clave = r.cols.map((p) => padre.columns.find((c) => c.name === p.to) || { name: p.to });
  const vals = await askFields(t("dz.relTitle"), [
    { key: "name", label: t("dz.relName"), value: r.name || dzRelName(hija, padre) },
    relKindField(relIsIdentifying(r)),
    ...relColFields(hija, padre, clave, (c, i) => r.cols[i].from),
    { key: "onDelete", label: t("dz.onDelete"), type: "select", options: acciones, value: r.onDelete || "" },
    { key: "onUpdate", label: t("dz.onUpdate"), type: "select", options: acciones, value: r.onUpdate || "" },
  ]);
  if (!vals) return;
  const cols = relColsFromVals(vals, clave);
  if (!cols.length) return;
  const ident = vals.kind === "ident";
  for (const p of cols) migrateKeyColumn(hija, p.from, padre.columns.find((c) => c.name === p.to), ident);
  applyRelKind(hija, cols, ident);
  Object.assign(r, { name: vals.name.trim(), cols, onDelete: vals.onDelete, onUpdate: vals.onUpdate });
  saveDesign();
  renderDesigner();
}

/** Pide el nombre de un esquema nuevo y lo agrega al modelo. */
async function askNewSchema() {
  const vals = await askFields(t("dz.newSchemaTitle"), [
    { key: "name", label: t("dz.schemaName"), value: "", placeholder: "ventas" },
  ]);
  const nombre = vals && vals.name.trim();
  if (!nombre) return null;
  if (!design.schemas.includes(nombre)) design.schemas.push(nombre);
  saveDesign();
  return nombre;
}

// ── Las líneas entre tablas ──────────────────────────────────────────────────


/** El camino entre dos tarjetas, en ángulo recto como el de ERwin.
 *
 *  Una diagonal entre dos cajas cruza por encima de todo lo que haya en medio,
 *  y en un modelo de quince tablas en medio hay siempre algo. Con tramos rectos
 *  la línea va por los pasillos y se puede seguir con la vista, que es para lo
 *  único que sirve una línea en un diagrama.
 *
 *  Sale por el lado que mira al otro —el eje en el que las cajas están más
 *  separadas— y gira a medio camino. Así los dos extremos llegan de frente a su
 *  caja y la pata de gallo y la barra apuntan a donde tienen que apuntar. */
function orthoPath(ba, bb) {
  const ca = { x: ba.x + ba.w / 2, y: ba.y + ba.h / 2 };
  const cb = { x: bb.x + bb.w / 2, y: bb.y + bb.h / 2 };
  const dx = cb.x - ca.x;
  const dy = cb.y - ca.y;
  if (Math.abs(dx) >= Math.abs(dy)) {
    const p1 = { x: dx >= 0 ? ba.x + ba.w : ba.x, y: ca.y };
    const p2 = { x: dx >= 0 ? bb.x : bb.x + bb.w, y: cb.y };
    const xm = (p1.x + p2.x) / 2;
    return [p1, { x: xm, y: p1.y }, { x: xm, y: p2.y }, p2];
  }
  const p1 = { x: ca.x, y: dy >= 0 ? ba.y + ba.h : ba.y };
  const p2 = { x: cb.x, y: dy >= 0 ? bb.y : bb.y + bb.h };
  const ym = (p1.y + p2.y) / 2;
  return [p1, { x: p1.x, y: ym }, { x: p2.x, y: ym }, p2];
}

function cardBox(card) {
  return { x: card.offsetLeft, y: card.offsetTop, w: card.offsetWidth, h: card.offsetHeight };
}

/** ¿La relación IDENTIFICA a la hija?
 *
 *  Lo hace cuando la columna foránea forma parte de la clave primaria de la
 *  hija: entonces la fila no puede existir sin su padre. ERwin dibuja esas con
 *  línea continua y las demás con línea discontinua, y la diferencia se deduce
 *  del modelo sin preguntar nada. */
function relIsIdentifying(r) {
  const hija = dzTableById(r.fromTable);
  if (!hija || !r.cols.length) return false;
  // La clave entera, no una parte: si solo alguna de sus columnas está en la
  // primaria, la fila de la hija todavía necesita algo más para identificarse,
  // y eso no es una relación identificadora.
  return r.cols.every((p) => {
    const c = hija.columns.find((x) => x.name === p.from);
    return !!(c && c.pk);
  });
}

function drawDesignEdges() {
  const svg = document.getElementById("dz-edges");
  const canvas = document.getElementById("design-canvas");
  if (!svg || !canvas) return;
  svg.textContent = "";
  svg.setAttribute("width", canvas.scrollWidth);
  svg.setAttribute("height", canvas.scrollHeight);

  // Pata de gallo en el lado "muchos" y barra en el lado "uno": la notación de
  // ERwin. `markerUnits="userSpaceOnUse"` para que el grosor de la línea no
  // escale el dibujo de la punta.
  const defs = document.createElementNS(SVGNS, "defs");
  defs.innerHTML =
    '<marker id="dz-crow" viewBox="0 0 12 12" refX="0" refY="6" markerWidth="12" markerHeight="12"' +
    ' markerUnits="userSpaceOnUse" orient="auto"><path d="M0 1 L10 6 M0 6 L10 6 M0 11 L10 6" /></marker>' +
    '<marker id="dz-one" viewBox="0 0 12 12" refX="11" refY="6" markerWidth="12" markerHeight="12"' +
    ' markerUnits="userSpaceOnUse" orient="auto"><path d="M8 1 L8 11" /></marker>';
  svg.appendChild(defs);

  for (const r of design.relations) {
    const a = canvas.querySelector(`.dz-table[data-id="${r.fromTable}"]`);
    const b = canvas.querySelector(`.dz-table[data-id="${r.toTable}"]`);
    if (!a || !b) continue;
    const puntos = orthoPath(cardBox(a), cardBox(b));
    const line = document.createElementNS(SVGNS, "polyline");
    line.setAttribute("points", puntos.map((p) => `${p.x},${p.y}`).join(" "));
    line.setAttribute("fill", "none");
    // La pata de gallo va en la HIJA (de donde sale la línea) y la barra en el
    // padre. Continua si la relación identifica a la hija, discontinua si no.
    line.setAttribute("marker-start", "url(#dz-crow)");
    line.setAttribute("marker-end", "url(#dz-one)");
    line.dataset.id = r.id;
    if (!relIsIdentifying(r)) line.setAttribute("class", "dz-no-id");
    if (dzSelEs("relation", r.id)) line.classList.add("sel");
    svg.appendChild(line);
  }

  // La línea de goma de la relación que se está tirando.
  if (dzLink) {
    const goma = document.createElementNS(SVGNS, "line");
    goma.setAttribute("class", "dz-rubber");
    goma.setAttribute("x1", dzLink.from.x);
    goma.setAttribute("y1", dzLink.from.y);
    goma.setAttribute("x2", dzLink.to.x);
    goma.setAttribute("y2", dzLink.to.y);
    svg.appendChild(goma);
  }
}

/** Los colores que puede llevar una tabla.
 *
 *  Una lista cerrada y no un selector de color libre, y lo que se guarda es el
 *  NOMBRE y no el píxel. Con dos temas, un `#1b5e20` elegido sobre fondo blanco
 *  es texto negro sobre negro al pasar a oscuro; guardando «verde», cada tema
 *  lo tinñe sobre su propia superficie y se lee en los dos. Que la lista sea
 *  corta también es a propósito: el color de un diagrama sirve para agrupar, y
 *  dieciocho verdes distintos no agrupan nada. */
const DZ_COLORS = [
  { id: "", key: "dz.cNone" },
  { id: "blue", key: "dz.cBlue" },
  { id: "cyan", key: "dz.cCyan" },
  { id: "green", key: "dz.cGreen" },
  { id: "amber", key: "dz.cAmber" },
  { id: "red", key: "dz.cRed" },
  { id: "violet", key: "dz.cViolet" },
  { id: "gray", key: "dz.cGray" },
];

/** La herramienta elegida en la paleta. Con `select` el lienzo se comporta
 *  como siempre: se arrastran las tablas y se tira de las agarraderas. */
let dzTool = "select";

/** Los dibujos de la paleta. En `svg`, el cuerpo de un `<svg viewBox="0 0 20 20">`.
 *
 *  La diferencia entre identificadora y no identificadora se ve en el icono
 *  igual que se ve en el diagrama: la línea continua y la discontinua. Es la
 *  misma gramática en los dos sitios, así que no hay nada nuevo que aprender. */
const DZ_TOOLS = [
  { id: "select", key: "dz.toolSelect", svg: '<path d="M5 3l10 7-4.2 1.2L13 16l-2 .8-2.1-4.7L5 15z"/>' },
  { id: "table", key: "dz.toolTable", svg: '<rect x="3" y="4" width="14" height="12" rx="1.5"/><path d="M3 8h14"/>' },
  { id: "ident", key: "dz.toolIdent", svg: '<path d="M2 10h16"/><path d="M14 6l4 4-4 4"/>' },
  { id: "noident", key: "dz.toolNoIdent", svg: '<path d="M2 10h3M7.5 10h3M13 10h2" /><path d="M14 6l4 4-4 4"/>' },
  { id: "m2m", key: "dz.toolM2M", svg: '<path d="M3 5l14 10M3 15L17 5"/><circle cx="3" cy="5" r="1.6"/><circle cx="3" cy="15" r="1.6"/><circle cx="17" cy="5" r="1.6"/><circle cx="17" cy="15" r="1.6"/>' },
];

/** Cambia de vista. Es solo cómo se mira: no toca ni un dato del modelo. */
function setView(v) {
  design.view = v === "logical" ? "logical" : "physical";
  saveDesign();
  renderDesigner();
}

function renderViewToggle() {
  const l = document.getElementById("view-logical");
  const f = document.getElementById("view-physical");
  if (!l || !f) return;
  l.classList.toggle("active", dzLogical());
  f.classList.toggle("active", !dzLogical());
}

function dzSetTool(id) {
  dzTool = id;
  renderPalette();
  const canvas = document.getElementById("design-canvas");
  if (canvas) canvas.classList.toggle("dz-armed", id !== "select");
  renderStatus();
}

function renderPalette() {
  const caja = document.getElementById("dz-palette");
  if (!caja) return;
  caja.replaceChildren();
  for (const h of DZ_TOOLS) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "dz-tool" + (dzTool === h.id ? " active" : "");
    b.title = t(h.key);
    b.innerHTML =
      '<svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.6" ' +
      'stroke-linecap="round" stroke-linejoin="round">' + h.svg + "</svg>";
    b.addEventListener("click", () => dzSetTool(h.id));
    caja.appendChild(b);
  }
}

/** Lo que hay seleccionado en el árbol, o `null`. */
let dzSel = null;

function dzSelEs(kind, id) {
  return !!dzSel && dzSel.kind === kind && dzSel.id === id;
}

/** Selecciona un objeto, lo deja a la vista en el lienzo y lo cuenta abajo. */
function dzSelect(kind, id) {
  dzSel = { kind, id };
  renderTree();
  renderStatus();
  marcarSeleccion();
  if (kind === "table") {
    const card = document.querySelector(`.dz-table[data-id="${id}"]`);
    // `nearest` y no `center`: si la tabla ya se ve, no se mueve nada. Que el
    // lienzo salte cada vez que se pulsa en el árbol marea más que ayuda.
    if (card) card.scrollIntoView({ block: "nearest", inline: "nearest" });
  }
}

/** Pinta el resalte de lo seleccionado sin rehacer el lienzo entero. */
function marcarSeleccion() {
  document.querySelectorAll(".dz-table").forEach((c) => {
    c.classList.toggle("sel", dzSelEs("table", c.dataset.id));
  });
  document.querySelectorAll("#dz-edges polyline").forEach((p) => {
    p.classList.toggle("sel", dzSelEs("relation", p.dataset.id));
  });
}

/** Una rama del árbol: el título con su cuenta y los hijos debajo. */
function treeGroup(ul, clave, label, n, hijos) {
  const li = document.createElement("li");
  li.className = "dz-tr-group";
  const head = document.createElement("div");
  head.className = "dz-tr-head";
  head.textContent = label;
  const cuenta = document.createElement("span");
  cuenta.className = "dz-tr-n";
  cuenta.textContent = n;
  head.appendChild(cuenta);
  const sub = document.createElement("ul");
  sub.className = "dz-tr-sub";
  // Abierta o cerrada se recuerda mientras dure la sesión: rehacer el árbol en
  // cada tecla no debe volver a abrir lo que se acaba de cerrar. La clave es
  // fija y no el título, que cambia al cambiar de idioma.
  const abierto = dzTreeOpen[clave] !== false;
  li.classList.toggle("cerrado", !abierto);
  head.addEventListener("click", () => {
    dzTreeOpen[clave] = !abierto;
    renderTree();
  });
  li.append(head, sub);
  ul.appendChild(li);
  hijos(sub);
}

const dzTreeOpen = {};

/** Una hoja del árbol: un objeto del modelo. */
function treeItem(sub, kind, id, texto, onAbrir, onBorrar, color) {
  const li = document.createElement("li");
  li.className = "dz-tr-item" + (dzSelEs(kind, id) ? " sel" : "");
  if (color) {
    // El mismo punto que la tarjeta: el árbol y el lienzo tienen que agrupar
    // igual, o el color sirve en un sitio y estorba en el otro.
    const pt = document.createElement("span");
    pt.className = "dz-tr-dot";
    pt.dataset.color = color;
    li.appendChild(pt);
  }
  const txt = document.createElement("span");
  txt.className = "dz-tr-txt";
  txt.textContent = texto;
  txt.title = texto;
  li.appendChild(txt);
  if (onBorrar) {
    const del = document.createElement("button");
    del.type = "button";
    del.className = "dz-icon-btn dz-tr-del";
    del.textContent = "\u2715";
    del.addEventListener("click", (e) => { e.stopPropagation(); onBorrar(); });
    li.appendChild(del);
  }
  // Un clic selecciona y dos abren, igual que en el lienzo: así se puede mirar
  // qué es cada cosa sin que se abra un diálogo a cada paso.
  li.addEventListener("click", () => dzSelect(kind, id));
  if (onAbrir) li.addEventListener("dblclick", onAbrir);
  sub.appendChild(li);
}

/** El árbol de objetos del modelo, el Model Explorer de ERwin.
 *
 *  Con quince tablas el lienzo deja de servir para saber qué hay: hay que
 *  buscar la caja con la vista. El árbol dice de un vistazo qué contiene el
 *  modelo y lleva a cada cosa. Las relaciones viven aquí, donde ERwin las
 *  pone, y no en una lista aparte debajo del lienzo. */
function renderTree() {
  const ul = document.getElementById("dz-tree-list");
  if (!ul) return;
  ul.innerHTML = "";

  treeGroup(ul, "tables", t("dz.treeTables"), design.tables.length, (sub) => {
    for (const tbl of design.tables) {
      treeItem(sub, "table", tbl.id, tblLabel(tbl), () => openTableEditor(tbl), () => {
        if (!confirm(t("dz.delTable") + ": " + tblLabel(tbl) + "?")) return;
        design.tables = design.tables.filter((x) => x.id !== tbl.id);
        design.relations = design.relations.filter((r) => r.fromTable !== tbl.id && r.toTable !== tbl.id);
        if (dzSelEs("table", tbl.id)) dzSel = null;
        saveDesign();
        renderDesigner();
      }, tbl.color);
    }
  });

  treeGroup(ul, "rels", t("dz.treeRels"), design.relations.length, (sub) => {
    for (const r of design.relations) {
      const ft = dzTableById(r.fromTable);
      const tt = dzTableById(r.toTable);
      if (!ft || !tt) continue;
      treeItem(sub, "relation", r.id, r.name || dzRelName(ft, tt), () => editRelation(r), () => {
        design.relations = design.relations.filter((x) => x.id !== r.id);
        if (dzSelEs("relation", r.id)) dzSel = null;
        saveDesign();
        renderDesigner();
      });
    }
  });

  if (dz?.supports_schemas) {
    const esquemas = dzSchemas();
    treeGroup(ul, "schemas", t("dz.treeSchemas"), esquemas.length, (sub) => {
      for (const e of esquemas) treeItem(sub, "schema", e, e, null, null);
    });
  }
}

/** La barra de abajo: qué hay seleccionado, o de qué tamaño es el modelo. */
function renderStatus() {
  const el = document.getElementById("dz-status");
  if (!el) return;
  const plural = (n, uno, varios) => (n === 1 ? t(uno) : t(varios).replace("{n}", n));

  if (dzSel && dzSel.kind === "table") {
    const tbl = dzTableById(dzSel.id);
    if (tbl) {
      const rels = design.relations.filter((r) => r.fromTable === tbl.id || r.toTable === tbl.id).length;
      el.textContent = `${tblLabel(tbl)} \u00b7 ${plural(tbl.columns.length, "dz.stCol1", "dz.stCols")} \u00b7 ${plural(rels, "dz.stRel1", "dz.stRels")}`;
      return;
    }
  }
  if (dzSel && dzSel.kind === "relation") {
    const r = design.relations.find((x) => x.id === dzSel.id);
    const ft = r && dzTableById(r.fromTable);
    const tt = r && dzTableById(r.toTable);
    if (ft && tt) {
      const lado = (l) => (l.length > 1 ? `(${l.join(", ")})` : l[0]);
      const acciones = [r.onDelete ? "ON DELETE " + r.onDelete : "", r.onUpdate ? "ON UPDATE " + r.onUpdate : ""].filter(Boolean).join(" \u00b7 ");
      el.textContent =
        `${r.name ? r.name + ": " : ""}${tblPhys(ft)}.${lado(r.cols.map((p) => p.from))} \u2192 ` +
        `${tblPhys(tt)}.${lado(r.cols.map((p) => p.to))}${acciones ? "  " + acciones : ""}`;
      return;
    }
  }
  if (dzTool !== "select") {
    const h = DZ_TOOLS.find((x) => x.id === dzTool);
    el.textContent = t(h.key) + (dzTool === "table" ? "" : " \u00b7 " + t("dz.toolHint"));
    return;
  }
  if (!design.tables.length) { el.textContent = t("dz.treeEmpty"); return; }
  if (dzLogical()) {
    el.textContent =
      t("dz.viewLogicalHint") + " \u00b7 " +
      `${plural(design.tables.length, "dz.stTable1", "dz.stTables")} \u00b7 ` +
      plural(design.relations.length, "dz.stRel1", "dz.stRels");
    return;
  }
  el.textContent =
    `${plural(design.tables.length, "dz.stTable1", "dz.stTables")} \u00b7 ` +
    `${plural(design.relations.length, "dz.stRel1", "dz.stRels")} \u00b7 ${t("dz.relHint")}`;
}


// ── Exportadores ─────────────────────────────────────────────────────────────
/** Las columnas de un lado de la clave foránea, entrecomilladas y separadas por
 *  comas: `[a], [b]`. Sirve igual para una clave de una columna que para una
 *  compuesta, que es toda la diferencia entre las dos en el SQL. */
function fkCols(r, lado, q) {
  return r.cols.map((p) => q(p[lado])).join(", ");
}

function dzQuote(dialect, s) {
  if (dialect === "sqlserver") return `[${String(s).replace(/]/g, "]]")}]`;
  // MySQL / MariaDB: acento grave, duplicado para escaparlo.
  if (dialect === "mysql") return `\`${String(s).replace(/`/g, "``")}\``;
  return `"${String(s).replace(/"/g, '""')}"`;
}

function designToSql(dialect) {
  const q = (s) => dzQuote(dialect, s);
  const conEsquemas = !!dz?.supports_schemas;
  const qt = (tbl) => {
    const s = tblSchema(tbl);
    return conEsquemas && s ? `${q(s)}.${q(tbl.name)}` : q(tbl.name);
  };

  let out = "-- Modelo generado por el diseñador de Schemir\n";
  const req = dzRequired();
  const target = dzTarget();
  if (req) {
    out += `-- Motor: ${dzDialectLabel(dialect)} ${req.label} o superior\n`;
    if (target && dzNewer(req, target)) {
      out += `-- AVISO: el objetivo elegido es ${target.label}, pero algún tipo exige ${req.label}.\n`;
    }
  }
  out += "\n";

  // Los esquemas, antes que las tablas que van dentro. El de por defecto del
  // motor (dbo, public) ya existe: no se crea.
  const esquemas = conEsquemas
    ? [...new Set(design.tables.map(tblSchema))].filter((s) => s && s !== dz.default_schema).sort()
    : [];
  for (const s of esquemas) {
    // En SQL Server CREATE SCHEMA tiene que ser la primera sentencia de su
    // lote, así que va dentro de un EXEC para poder condicionarla.
    out += dialect === "sqlserver"
      ? `IF SCHEMA_ID(N'${s.replace(/'/g, "''")}') IS NULL EXEC(N'CREATE SCHEMA ${q(s)}');\n`
      : `CREATE SCHEMA IF NOT EXISTS ${q(s)};\n`;
  }
  if (esquemas.length) out += "\n";

  for (const tbl of design.tables) {
    // SQLite solo admite AUTOINCREMENT pegado a `INTEGER PRIMARY KEY`: dentro
    // de la columna y sin una cláusula PRIMARY KEY aparte.
    const pkCols = tbl.columns.filter((c) => c.pk);
    const sqliteInline = dialect === "sqlite" && pkCols.length === 1 && pkCols[0].auto === "identity";
    const lines = tbl.columns.map((c) => colDdl(dialect, c, q, sqliteInline && c === pkCols[0]));
    if (pkCols.length && !sqliteInline) lines.push(`    PRIMARY KEY (${pkCols.map((c) => q(c.name)).join(", ")})`);
    // SQLite no sabe agregar una foránea con ALTER TABLE: va dentro.
    if (dialect === "sqlite") {
      for (const r of design.relations.filter((x) => x.fromTable === tbl.id)) {
        const tt = dzTableById(r.toTable);
        if (tt) lines.push(`    CONSTRAINT ${q(r.name || dzRelName(tbl, tt))} FOREIGN KEY (${fkCols(r, "from", q)}) REFERENCES ${qt(tt)} (${fkCols(r, "to", q)})${fkActions(r)}`);
      }
    }
    out += `CREATE TABLE ${qt(tbl)} (\n${lines.join(",\n")}\n);\n\n`;
  }

  if (dialect !== "sqlite") {
    for (const r of design.relations) {
      const ft = dzTableById(r.fromTable);
      const tt = dzTableById(r.toTable);
      if (!ft || !tt) continue;
      out += `ALTER TABLE ${qt(ft)} ADD CONSTRAINT ${q(r.name || dzRelName(ft, tt))} FOREIGN KEY (${fkCols(r, "from", q)}) REFERENCES ${qt(tt)} (${fkCols(r, "to", q)})${fkActions(r)};\n`;
    }
  }
  return out;
}

/** Una columna tal como va dentro del CREATE TABLE.
 *
 *  El orden de las piezas no es el mismo en todos los motores: MySQL documenta
 *  el AUTO_INCREMENT DETRÁS del NOT NULL, mientras que SQL Server y PostgreSQL
 *  lo escriben pegado al tipo. */
function colDdl(dialect, c, q, pkInline) {
  const tipo = fullType(c) || "TEXT";
  const contador = c.auto === "identity" && dz ? dz.identity_suffix : "";
  const porDefecto = c.auto === "uuid" && dz && dz.uuid_default ? "DEFAULT " + dz.uuid_default : "";
  // Ni una columna de la clave primaria ni un contador pueden admitir nulos:
  // el motor rechazaría el CREATE TABLE.
  const nulo = c.pk || c.nn || c.auto === "identity" ? "NOT NULL" : "NULL";
  const partes = pkInline
    ? [q(c.name), tipo, "PRIMARY KEY", contador, nulo]
    : dialect === "mysql"
      ? [q(c.name), tipo, nulo, porDefecto, contador]
      : [q(c.name), tipo, contador, nulo, porDefecto];
  return "    " + partes.filter(Boolean).join(" ");
}

/** Las cláusulas ON DELETE / ON UPDATE, solo cuando dicen algo: NO ACTION es
 *  lo que el motor hace de todos modos. */
function fkActions(r) {
  let s = "";
  if (r.onDelete) s += ` ON DELETE ${r.onDelete}`;
  if (r.onUpdate) s += ` ON UPDATE ${r.onUpdate}`;
  return s;
}

function prismaType(sqlType) {
  const s = (sqlType || "").toLowerCase();
  if (/bigint/.test(s)) return "BigInt";
  if (/int|serial/.test(s)) return "Int";
  if (/real|floa|doub|decimal|numeric|money/.test(s)) return "Float";
  if (/bool|^bit$/.test(s)) return "Boolean";
  if (/date|time/.test(s)) return "DateTime";
  if (/bytea|binary|blob|image/.test(s)) return "Bytes";
  return "String";
}
function pascal(s) {
  return String(s).replace(/(^|[_\s]+)([a-zA-Z])/g, (_, __, c) => c.toUpperCase()).replace(/[^A-Za-z0-9]/g, "") || "Modelo";
}

function designToPrisma() {
  const relFrom = {};
  const relTo = {};
  for (const r of design.relations) {
    (relFrom[r.fromTable] = relFrom[r.fromTable] || []).push(r);
    (relTo[r.toTable] = relTo[r.toTable] || []).push(r);
  }
  let out = "// Modelo generado por el diseñador de Schemir\n\n";
  for (const tbl of design.tables) {
    out += `model ${pascal(tbl.name)} {\n`;
    const pkCols = tbl.columns.filter((c) => c.pk).map((c) => c.name);
    for (const c of tbl.columns) {
      const optional = c.nn || c.pk ? "" : "?";
      let line = `  ${c.name} ${prismaType(c.type)}${optional}`;
      if (c.pk && pkCols.length === 1) line += " @id";
      if (c.auto === "identity") line += " @default(autoincrement())";
      else if (c.auto === "uuid") line += " @default(uuid())";
      out += line + "\n";
    }
    for (const r of relFrom[tbl.id] || []) {
      const tt = dzTableById(r.toTable);
      if (tt) out += `  ${(tt.name || "rel").toLowerCase()} ${pascal(tt.name)} @relation(fields: [${r.cols.map((p) => p.from).join(", ")}], references: [${r.cols.map((p) => p.to).join(", ")}])\n`;
    }
    for (const r of relTo[tbl.id] || []) {
      const ft = dzTableById(r.fromTable);
      if (ft) out += `  ${(ft.name || "rel").toLowerCase()}s ${pascal(ft.name)}[]\n`;
    }
    if (pkCols.length > 1) out += `  @@id([${pkCols.join(", ")}])\n`;
    // El esquema, cuando el motor los tiene y la tabla no está en el de siempre.
    const s = tblSchema(tbl);
    if (dz?.supports_schemas && s && s !== dz.default_schema) out += `  @@schema("${s}")\n`;
    out += `}\n\n`;
  }
  return out;
}

function showDesignOutput(title, text) {
  lastDesignOut = text;
  document.getElementById("dz-output-title").textContent = title;
  document.getElementById("design-box").innerHTML = highlightSql(text);
  const box = document.getElementById("design-output");
  box.hidden = false;
  box.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

// ── Enganches de la interfaz del diseñador ──
document.getElementById("view-logical").addEventListener("click", () => setView("logical"));
document.getElementById("view-physical").addEventListener("click", () => setView("physical"));

document.getElementById("mode-design").addEventListener("click", () => showDesigner(true));
document.getElementById("mode-compare").addEventListener("click", () => showDesigner(false));

document.getElementById("design-dialect").addEventListener("change", (e) => applyDialect(e.target.value));
document.getElementById("design-version").addEventListener("change", (e) => {
  design.version = e.target.value;
  saveDesign();
  renderDesigner();
});

document.getElementById("add-table").addEventListener("click", () => {
  const n = design.tables.length;
  design.tables.push({
    id: "t" + design.seq++,
    name: "tabla_" + (n + 1),
    schema: dz?.default_schema || "",
    x: 20 + (n % 4) * 250,
    y: 20 + Math.floor(n / 4) * 230,
    columns: [newPkColumn()],
  });
  saveDesign();
  renderDesigner();
});

document.getElementById("add-schema").addEventListener("click", () => askNewSchema().then(renderDesigner));

// Con la herramienta de tabla armada, se pulsa en el lienzo y la tabla nace
// ahí. Es la diferencia entre colocar y que te la coloquen: el botón «+ Tabla»
// la deja donde le toca por orden, y en un modelo grande eso casi nunca es
// donde la quieres.
document.getElementById("design-canvas").addEventListener("mousedown", (e) => {
  if (dzTool !== "table" || e.target.closest(".dz-table")) return;
  const canvas = e.currentTarget;
  const caja = canvas.getBoundingClientRect();
  design.tables.push({
    id: "t" + design.seq++,
    name: "tabla_" + (design.tables.length + 1),
    schema: dz?.default_schema || "",
    x: Math.round(e.clientX - caja.left + canvas.scrollLeft),
    y: Math.round(e.clientY - caja.top + canvas.scrollTop),
    columns: [newPkColumn()],
  });
  saveDesign();
  renderDesigner();
  dzSetTool("select");
});

// Escape desarma la herramienta. Un modal se queda con su propio Escape, y
// mientras hay uno abierto no hay ninguna armada, así que no se pisan.
document.addEventListener("keydown", (e) => {
  if (e.key !== "Escape" || dzTool === "select") return;
  if (document.querySelector(".dz-modal-back")) return;
  dzSetTool("select");
});

document.getElementById("rel-add").addEventListener("click", async () => {
  if (design.tables.length < 1) return;
  const hija = design.tables[0];
  const padre = design.tables[1] || design.tables[0];
  await askRelation(hija, hija.columns[0], padre, padre.columns.find((c) => c.pk));
});

document.getElementById("clear-design").addEventListener("click", async () => {
  let ok = false;
  try { ok = await askConfirm(t("dz.clearConfirm")); } catch { ok = false; }
  if (!ok) return;
  design = migrateDesign({ tables: [], relations: [], schemas: [], seq: 1, dialect: design.dialect, version: design.version });
  saveDesign();
  renderDesigner();
  document.getElementById("design-output").hidden = true;
});

document.getElementById("export-sql").addEventListener("click", () => {
  if (!design.tables.length) return;
  showDesignOutput(t("dz.sqlTitle"), designToSql(design.dialect));
});
document.getElementById("export-prisma").addEventListener("click", () => {
  if (!design.tables.length) return;
  showDesignOutput(t("dz.prismaTitle"), designToPrisma());
});

document.getElementById("copy-design").addEventListener("click", async (e) => {
  const btn = e.currentTarget;
  try {
    await navigator.clipboard.writeText(lastDesignOut);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = lastDesignOut;
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch {}
    ta.remove();
  }
  btn.textContent = t("btn.copied");
  setTimeout(() => (btn.textContent = t("btn.copy")), 1200);
});

document.getElementById("save-design").addEventListener("click", async () => {
  errorBanner.hidden = true;
  const isPrisma = document.getElementById("dz-output-title").textContent === t("dz.prismaTitle");
  try {
    const path = isPrisma
      ? await saveFileDialog(t("dz.save"), "schema.prisma", "Prisma", "prisma")
      : await saveFileDialog(t("dz.save"), "modelo.sql", "SQL", "sql");
    if (path) await invoke("save_text_file", { path, content: lastDesignOut });
  } catch (e) {
    showError(e);
  }
});


// ── Guardar y abrir el modelo en un archivo ──────────────────────────────────
// El modelo vivía solo en localStorage: se va con la caché del navegador
// incrustado, no se comparte y no entra en git. Con un archivo, sí.
//
// El formato es XML y no JSON a propósito: es lo que exportan las herramientas
// de modelado (ERwin entre ellas), se lee a ojo y se diferencia bien en git.
// Las relaciones apuntan a las tablas por NOMBRE —no por el identificador
// interno, que no significa nada fuera del programa— para que el archivo se
// pueda leer y hasta editar a mano.

const MODEL_FORMAT_VERSION = "1";

function xesc(s) {
  return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}

/** Cómo se nombra una tabla dentro del archivo, para que las relaciones la
 *  encuentren aunque haya dos con el mismo nombre en esquemas distintos. */
function tblKey(tbl) {
  return tbl.schema ? `${tbl.schema}.${tbl.name}` : tbl.name;
}

function designToXml() {
  // Un atributo que no dice nada no se escribe: el archivo queda más corto y
  // se ve de un golpe qué tiene valor puesto a mano.
  const at = (k, v) => (v === "" || v === undefined || v === null ? "" : ` ${k}="${xesc(v)}"`);
  let out = '<?xml version="1.0" encoding="UTF-8"?>\n';
  out += `<schemir-model version="${MODEL_FORMAT_VERSION}"${at("dialect", design.dialect)}${at("engine-version", design.version)}>\n`;

  const esquemas = (design.schemas || []).filter(Boolean);
  if (esquemas.length) {
    out += "  <schemas>\n";
    for (const e of esquemas) out += `    <schema${at("name", e)} />\n`;
    out += "  </schemas>\n";
  }

  out += "  <tables>\n";
  for (const tbl of design.tables) {
    out += `    <table${at("name", tbl.name)}${at("logical", tbl.logical)}${at("schema", tbl.schema)}${at("color", tbl.color)}${at("x", tbl.x)}${at("y", tbl.y)}${at("w", tbl.w)}${at("h", tbl.h)}>\n`;
    for (const c of tbl.columns) {
      out += `      <column${at("name", c.name)}${at("logical", c.logical)}${at("type", fullType(c))}${at("auto", c.auto)}${c.pk ? ' pk="true"' : ""}${c.nn ? ' nn="true"' : ""} />\n`;
    }
    out += "    </table>\n";
  }
  out += "  </tables>\n";

  const rels = design.relations.filter((r) => dzTableById(r.fromTable) && dzTableById(r.toTable));
  if (rels.length) {
    out += "  <relations>\n";
    for (const r of rels) {
      const ft = dzTableById(r.fromTable);
      const tt = dzTableById(r.toTable);
      // Una relación de una sola columna se graba como siempre, en atributos,
      // para que los modelos ya guardados y los que se guarden ahora sean el
      // mismo archivo. Las compuestas llevan dentro un <pair> por columna.
      const uno = r.cols.length === 1 ? r.cols[0] : null;
      out +=
        `    <relation${at("name", r.name)}` +
        `${at("child-table", tblKey(ft))}${uno ? at("child-column", uno.from) : ""}` +
        `${at("parent-table", tblKey(tt))}${uno ? at("parent-column", uno.to) : ""}` +
        `${at("on-delete", r.onDelete)}${at("on-update", r.onUpdate)}`;
      if (uno) out += " />\n";
      else {
        out += ">\n";
        for (const p of r.cols) {
          out += `      <pair${at("child-column", p.from)}${at("parent-column", p.to)} />\n`;
        }
        out += "    </relation>\n";
      }
    }
    out += "  </relations>\n";
  }

  out += "</schemir-model>\n";
  return out;
}

/** Lee un modelo de XML. Lanza con un mensaje entendible si el archivo no es
 *  uno de estos. */
function xmlToDesign(texto) {
  const doc = new DOMParser().parseFromString(String(texto || ""), "application/xml");
  if (doc.querySelector("parsererror")) throw new Error(t("dz.badXml"));
  const root = doc.documentElement;
  if (!root || root.nodeName !== "schemir-model") throw new Error(t("dz.notModel"));

  const d = {
    tables: [],
    relations: [],
    schemas: [],
    seq: 1,
    dialect: root.getAttribute("dialect") || DZ_DIALECT_DEFAULT,
    version: root.getAttribute("engine-version") || "",
  };

  for (const e of root.querySelectorAll("schemas > schema")) {
    const n = e.getAttribute("name");
    if (n && !d.schemas.includes(n)) d.schemas.push(n);
  }

  const porNombre = new Map();
  for (const te of root.querySelectorAll("tables > table")) {
    const tbl = {
      id: "t" + d.seq++,
      name: te.getAttribute("name") || "tabla",
      logical: te.getAttribute("logical") || "",
      schema: te.getAttribute("schema") || "",
      color: te.getAttribute("color") || "",
      x: Number(te.getAttribute("x")) || 20,
      y: Number(te.getAttribute("y")) || 20,
      w: Number(te.getAttribute("w")) || 0,
      h: Number(te.getAttribute("h")) || 0,
      columns: [],
    };
    for (const ce of te.querySelectorAll("column")) {
      const p = splitType(ce.getAttribute("type") || "");
      tbl.columns.push({
        name: ce.getAttribute("name") || "columna",
        logical: ce.getAttribute("logical") || "",
        type: p.type,
        a: p.a,
        b: p.b,
        auto: ce.getAttribute("auto") || "",
        pk: ce.getAttribute("pk") === "true",
        nn: ce.getAttribute("nn") === "true",
      });
    }
    d.tables.push(tbl);
    porNombre.set(tblKey(tbl), tbl.id);
  }
  if (!d.tables.length) throw new Error(t("dz.emptyModel"));

  for (const re of root.querySelectorAll("relations > relation")) {
    const from = porNombre.get(re.getAttribute("child-table") || "");
    const to = porNombre.get(re.getAttribute("parent-table") || "");
    // Una relación que apunta a una tabla que no está en el archivo se
    // descarta: mejor abrir el modelo sin ella que no abrirlo.
    if (!from || !to) continue;
    const cols = [...re.querySelectorAll("pair")]
      .map((pe) => ({
        from: pe.getAttribute("child-column") || "",
        to: pe.getAttribute("parent-column") || "",
      }))
      .filter((p) => p.from && p.to);
    // Sin <pair> es una relación de una columna, en atributos: así se graban y
    // así venían los archivos de antes de las claves compuestas.
    if (!cols.length) {
      cols.push({
        from: re.getAttribute("child-column") || "",
        to: re.getAttribute("parent-column") || "",
      });
    }
    d.relations.push({
      id: "r" + d.seq++,
      name: re.getAttribute("name") || "",
      fromTable: from,
      toTable: to,
      cols,
      onDelete: re.getAttribute("on-delete") || "",
      onUpdate: re.getAttribute("on-update") || "",
    });
  }

  return migrateDesign(d);
}

document.getElementById("save-model").addEventListener("click", async () => {
  errorBanner.hidden = true;
  if (!design.tables.length) return;
  try {
    const path = await saveFileDialog(t("dz.saveModel"), "modelo.schemir.xml", "Schemir", "xml");
    if (path) await invoke("save_text_file", { path, content: designToXml() });
  } catch (e) {
    showError(e);
  }
});

document.getElementById("open-model").addEventListener("click", async () => {
  errorBanner.hidden = true;
  try {
    const path = await pickFile(t("dz.openModel"), [
      { name: "Schemir", extensions: ["xml"] },
      { name: "*", extensions: ["*"] },
    ]);
    if (!path) return;
    const texto = await invoke("read_text_file", { path });
    // Se lee ANTES de preguntar: si el archivo no sirve, no tiene sentido
    // avisar de que se va a perder el modelo que hay abierto.
    const cargado = xmlToDesign(texto);
    if (design.tables.length) {
      let ok = false;
      try { ok = await askConfirm(t("dz.openConfirm")); } catch { ok = false; }
      if (!ok) return;
    }
    design = cargado;
    saveDesign();
    await applyDialect(design.dialect || DZ_DIALECT_DEFAULT);
  } catch (e) {
    showError(e);
  }
});

// ===========================================================================
// Render de resultados
// ===========================================================================
function renderResults() {
  renderSummary();
  renderFilters();
  renderRows();
}

function renderSummary() {
  const counts = { only_in_left: 0, only_in_right: 0, different: 0, equal: 0 };
  for (const o of lastObjects) counts[o.status]++;
  document.getElementById("summary").innerHTML = Object.entries(STATUS)
    .map(([k, m]) => `<div class="stat ${m.stat}"><div class="stat-n">${counts[k]}</div><div class="stat-l"><span class="dot"></span>${esc(t("status." + k))}</div></div>`)
    .join("");
}

function renderFilters() {
  const el = document.getElementById("filters");
  el.innerHTML = Object.entries(STATUS)
    .map(([k]) => `<button type="button" class="chip ${activeFilters.has(k) ? "" : "off"}" data-k="${k}"><span class="dot"></span>${esc(t("status." + k))}</button>`)
    .join("");
  el.querySelectorAll(".chip").forEach((chip) => {
    chip.addEventListener("click", () => {
      const k = chip.dataset.k;
      activeFilters.has(k) ? activeFilters.delete(k) : activeFilters.add(k);
      chip.classList.toggle("off");
      renderRows();
    });
  });
}

document.getElementById("search").addEventListener("input", (e) => {
  searchText = e.target.value.toLowerCase();
  renderRows();
});

function renderRows() {
  const container = document.getElementById("rows");
  const needle = searchText;
  const visible = lastObjects.filter(
    (o) => activeFilters.has(o.status) && (needle === "" || o.name.toLowerCase().includes(needle) || typeLabel(o.object_type).toLowerCase().includes(needle))
  );

  container.innerHTML = "";
  for (const o of visible) {
    const m = STATUS[o.status];
    const hasDetails = o.details && o.details.length > 0;
    const uKey = unitKey(o.object_type, o.name);
    const hasUnit = unitKeys.has(uKey);

    const row = document.createElement("div");
    row.className = "row" + (hasDetails ? " expandable" : "");
    row.dataset.status = o.status;
    const checkHtml = hasUnit
      ? `<input type="checkbox" class="row-check" ${deselected.has(uKey) ? "" : "checked"} title="${esc(t("sel.include"))}" />`
      : `<span class="row-check-spacer" aria-hidden="true"></span>`;
    row.innerHTML =
      `<div class="row-main">` +
      checkHtml +
      `<span class="otype">${objIcon(o.object_type)}</span>` +
      `<span class="oname">${esc(o.name)}</span>` +
      `<span class="ometa">${esc(typeLabel(o.object_type))}</span>` +
      `<span class="spacer"></span>` +
      `<span class="badge ${m.badge}"><span class="dot"></span>${esc(t("status." + o.status))}</span>` +
      `<span class="chev">${hasDetails ? "▸" : ""}</span>` +
      `</div>`;
    if (hasUnit) {
      const cb = row.querySelector(".row-check");
      cb.addEventListener("click", (ev) => ev.stopPropagation());
      cb.addEventListener("change", () => {
        if (cb.checked) deselected.delete(uKey);
        else deselected.add(uKey);
        updateScriptFromSelection();
      });
    }

    if (hasDetails) {
      const det = document.createElement("div");
      det.className = "row-detail";
      det.hidden = true;
      det.innerHTML = detailHtml(o.details);
      row.appendChild(det);
      row.querySelector(".row-main").addEventListener("click", () => {
        det.hidden = !det.hidden;
        row.classList.toggle("open", !det.hidden);
        row.querySelector(".chev").textContent = det.hidden ? "▸" : "▾";
      });
    }
    container.appendChild(row);
  }
  document.getElementById("empty").hidden = visible.length > 0;
}

function detailHtml(details) {
  const body = details
    .map((d) => `<div class="d-field">${esc(d.field)}</div><div class="d-a">${esc(d.left ?? "—")}</div><div class="d-b">${esc(d.right ?? "—")}</div>`)
    .join("");
  return `<div class="detail-grid"><div class="d-head d-field">${esc(t("detail.field"))}</div><div class="d-head d-a">${esc(t("detail.a"))}</div><div class="d-head d-b">${esc(t("detail.b"))}</div>${body}</div>`;
}

// ===========================================================================
// Resaltado SQL
// ===========================================================================
const SQL_KW = /\b(CREATE|OR|ALTER|DROP|ADD|COLUMN|TABLE|VIEW|PROCEDURE|FUNCTION|TRIGGER|INDEX|CONSTRAINT|PRIMARY|FOREIGN|KEY|REFERENCES|NOT|NULL|DEFAULT|IF|EXISTS|AS|GO|BEGIN|END|SELECT|FROM|WHERE|JOIN|ON|INTO|VALUES|SET|UNIQUE|CHECK)\b/gi;

function highlightSql(sql) {
  return sql
    .split("\n")
    .map((line) => {
      if (line.trimStart().startsWith("--")) return `<span class="sql-c">${esc(line)}</span>`;
      let h = esc(line);
      h = h.replace(/'[^']*'/g, (m) => `<span class="sql-s">${m}</span>`);
      h = h.replace(SQL_KW, (m) => `<span class="sql-k">${m}</span>`);
      return h;
    })
    .join("\n");
}

// ===========================================================================
// Utilidades
// ===========================================================================
function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}
function errText(e) {
  if (typeof e === "string") return e;
  return e?.message || JSON.stringify(e);
}

// ===========================================================================
// Toggle de idioma + arranque
// ===========================================================================
document.querySelectorAll(".lang-toggle button").forEach((b) => {
  b.addEventListener("click", () => {
    lang = b.dataset.lang;
    try { localStorage.setItem("schemir.lang", lang); } catch {}
    applyI18n();
  });
});

// Tema: claro / oscuro / sistema (recordado en localStorage).
let theme = (() => {
  try { return localStorage.getItem("schemir.theme") || "system"; } catch { return "system"; }
})();
function applyTheme() {
  document.documentElement.dataset.theme = theme;
  document.querySelectorAll(".theme-toggle button").forEach((b) => b.classList.toggle("active", b.dataset.themeSet === theme));
}
document.querySelectorAll(".theme-toggle button").forEach((b) => {
  b.addEventListener("click", () => {
    theme = b.dataset.themeSet;
    try { localStorage.setItem("schemir.theme", theme); } catch {}
    applyTheme();
  });
});
applyTheme();

if (!window.__TAURI__?.core?.invoke) {
  document.getElementById("no-backend").hidden = false;
}

setupOptions();
applyI18n();
