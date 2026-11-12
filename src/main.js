// ---------------------------------------------------------------------------
// DataDiff — lógica de la interfaz (bilingüe ES/EN).
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
    "banner.noBackend": "⚠️ Esta interfaz necesita el backend. Ejecútala con cargo run o npm run tauri dev.",
    "card.origin": "Origen",
    "card.dest": "Destino",
    "field.server": "Servidor",
    "field.port": "Puerto",
    "field.database": "Base de datos",
    "field.user": "Usuario",
    "field.password": "Contraseña",
    "field.sqliteFile": "Archivo SQLite",
    "ph.database": "ventas",
    "ph.sqlitePath": "/ruta/a/base.db",
    "hint.sqlite": "Elige el archivo .db / .sqlite con «Examinar…» o pega la ruta.",
    "btn.browse": "Examinar…",
    "btn.test": "Probar conexión",
    "btn.compare": "Comparar objetos",
    "btn.comparing": "Comparando…",
    "btn.copy": "Copiar",
    "btn.copied": "¡Copiado!",
    "btn.saveSql": "Guardar .sql",
    "status.testing": "Probando…",
    "compare.hint": "Prueba las conexiones de A y B para habilitar la comparación.",
    "step.connection": "Conexión",
    "step.comparison": "Comparación",
    "btn.back": "← Conexión",
    "tab.diff": "Diferencias",
    "tab.script": "Script de sincronización",
    "tab.diagram": "Diagrama",
    "search.ph": "Buscar objeto…",
    "empty.filter": "No hay objetos que coincidan con el filtro.",
    "script.info": "SQL para dejar <b class=\"side-b-text\">B</b> igual que <b class=\"side-a-text\">A</b> · revísalo antes de ejecutarlo.",
    "script.none": "-- (sin cambios)",
    "btn.apply": "Aplicar a B",
    "apply.confirm": "¿Aplicar el script a la base B? Esto MODIFICA el destino (crea, altera y elimina objetos). Haz un respaldo antes.",
    "apply.applying": "Aplicando…",
    "apply.done": "✓ Aplicado a B. Vuelve a comparar para confirmar.",
    "diagram.legend": "Diagrama entidad-relación · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Diferencias",
    "diagram.sideA": "Origen A",
    "diagram.sideB": "Destino B",
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
    "detail.a": "A · origen",
    "detail.b": "B · destino",
    "status.only_in_left": "Solo en A",
    "status.only_in_right": "Solo en B",
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
    "err.noBackend": "Sin backend: abre la app con cargo run o npm run tauri dev (en un navegador normal no hay conexión a las bases).",
  },
  en: {
    "app.subtitle": "Database object comparator",
    "app.tagline": "SSDT alternative",
    "theme.light": "Light",
    "theme.dark": "Dark",
    "theme.system": "System",
    "banner.noBackend": "⚠️ This UI needs the backend. Run it with cargo run or npm run tauri dev.",
    "card.origin": "Source",
    "card.dest": "Target",
    "field.server": "Server",
    "field.port": "Port",
    "field.database": "Database",
    "field.user": "User",
    "field.password": "Password",
    "field.sqliteFile": "SQLite file",
    "ph.database": "sales",
    "ph.sqlitePath": "/path/to/db.db",
    "hint.sqlite": "Pick the .db / .sqlite file with “Browse…” or paste the path.",
    "btn.browse": "Browse…",
    "btn.test": "Test connection",
    "btn.compare": "Compare objects",
    "btn.comparing": "Comparing…",
    "btn.copy": "Copy",
    "btn.copied": "Copied!",
    "btn.saveSql": "Save .sql",
    "status.testing": "Testing…",
    "compare.hint": "Test both A and B connections to enable the comparison.",
    "step.connection": "Connection",
    "step.comparison": "Comparison",
    "btn.back": "← Connection",
    "tab.diff": "Differences",
    "tab.script": "Synchronization script",
    "tab.diagram": "Diagram",
    "search.ph": "Search object…",
    "empty.filter": "No objects match the filter.",
    "script.info": "SQL to make <b class=\"side-b-text\">B</b> match <b class=\"side-a-text\">A</b> · review it before running.",
    "script.none": "-- (no changes)",
    "btn.apply": "Apply to B",
    "apply.confirm": "Apply the script to database B? This MODIFIES the target (creates, alters and drops objects). Make a backup first.",
    "apply.applying": "Applying…",
    "apply.done": "✓ Applied to B. Compare again to confirm.",
    "diagram.legend": "Entity-relationship diagram · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Differences",
    "diagram.sideA": "Source A",
    "diagram.sideB": "Target B",
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
    "detail.a": "A · source",
    "detail.b": "B · target",
    "status.only_in_left": "Only in A",
    "status.only_in_right": "Only in B",
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
    "err.noBackend": "No backend: open the app with cargo run or npm run tauri dev (a plain browser has no database connection).",
  },
};

let lang = (() => {
  try {
    return localStorage.getItem("datadiff.lang") || "es";
  } catch {
    return "es";
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

  // Re-render de lo generado por JS.
  const results = document.getElementById("results");
  if (results && !results.hidden) {
    renderResults();
    updateScriptFromSelection();
    if (!document.getElementById("tab-diagram").hidden) renderDiagram();
  }
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

// ===========================================================================
// Selector de motor por tarjeta
// ===========================================================================
document.querySelectorAll(".card").forEach((card) => {
  card.querySelectorAll(".seg").forEach((seg) => {
    seg.addEventListener("click", () => {
      const engine = seg.dataset.engine;
      card.dataset.engine = engine;
      card.querySelectorAll(".seg").forEach((s) => s.classList.toggle("active", s === seg));
      // SQL Server y PostgreSQL comparten los campos de servidor; SQLite usa el de archivo.
      card.querySelector(".f-sqlserver").hidden = engine === "sqlite";
      card.querySelector(".f-sqlite").hidden = engine !== "sqlite";
      // Puerto por defecto según el motor.
      const portInput = card.querySelector('input[name="port"]');
      if (portInput) {
        if (engine === "postgres" && (portInput.value === "1433" || portInput.value === "")) portInput.value = "5432";
        if (engine === "sqlserver" && (portInput.value === "5432" || portInput.value === "")) portInput.value = "1433";
      }
    });
  });
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
    port: Number(val("port")) || (engine === "postgres" ? 5432 : 1433),
    database: val("database"),
    user: val("user"),
    password: val("password"),
    path: "",
  };
}

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
    status.className = "conn-status";
    status.textContent = t("status.testing");
    btn.disabled = true;
    try {
      const msg = await invoke("test_connection", { config: readConfig(side) });
      status.className = "conn-status ok";
      status.textContent = "✓ " + msg;
      connOk[side] = true;
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

  // Cualquier cambio invalida la conexión.
  const invalidate = () => {
    connOk[side] = false;
    refreshCompareState();
  };
  card.querySelectorAll("input").forEach((inp) => inp.addEventListener("input", invalidate));
  card.querySelectorAll(".seg").forEach((seg) => seg.addEventListener("click", invalidate));
});
refreshCompareState();

// ===========================================================================
// Comparar
// ===========================================================================
compareBtn.addEventListener("click", async () => {
  errorBanner.hidden = true;
  comparing = true;
  compareBtn.disabled = true;
  compareBtn.textContent = t("btn.comparing");
  try {
    const result = await invoke("compare_databases", { left: readConfig("a"), right: readConfig("b") });
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

function goToStep(step) {
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
  const eng = cfg.engine === "postgres" ? "PostgreSQL" : "SQL Server";
  return eng + " · " + (cfg.database || cfg.host || "—");
}

// ===========================================================================
// Pestañas, script y diagrama
// ===========================================================================
document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((x) => x.classList.toggle("active", x === tab));
    document.getElementById("tab-diff").hidden = tab.dataset.tab !== "diff";
    document.getElementById("tab-script").hidden = tab.dataset.tab !== "script";
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
  const options = { title: "DataDiff", kind: "warning" };
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
    `<title>DataDiff — ${esc(t("report.title"))}</title><style>${REPORT_CSS}</style></head><body>` +
    `<header><h1>DataDiff</h1><p class="sub">${esc(t("report.title"))}</p>` +
    (dbs ? `<p class="meta">${esc(dbs)}</p>` : "") +
    `<p class="meta">${esc(t("report.generated"))}: ${esc(now)}</p></header>` +
    `<section class="summary">${summary}</section>` +
    `<h2>${esc(t("report.objects"))}</h2>` +
    (changed.length ? objs : `<p class="empty">${esc(t("report.noChanges"))}</p>`) +
    `<h2>${esc(t("report.script"))}</h2><pre class="script">${script}</pre>` +
    `<footer>DataDiff · github.com/marlontaro/DataDiff</footer>` +
    `</body></html>`
  );
}

async function exportReport() {
  errorBanner.hidden = true;
  if (!lastObjects || !lastObjects.length) return;
  try {
    const html = buildReportHtml();
    const path = await saveFileDialog(t("btn.report"), "informe-datadiff.html", "HTML", "html");
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
  const e = readConfig("b").engine;
  return e === "postgres" ? "PostgreSQL" : e === "sqlite" ? "SQLite" : "SQL Server";
}

function syncHeader() {
  return [
    "-- ============================================================",
    "-- Script de sincronización generado por DataDiff",
    "-- Deja el DESTINO (B) igual que el ORIGEN (A)",
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
    try { localStorage.setItem("datadiff.lang", lang); } catch {}
    applyI18n();
  });
});

// Tema: claro / oscuro / sistema (recordado en localStorage).
let theme = (() => {
  try { return localStorage.getItem("datadiff.theme") || "system"; } catch { return "system"; }
})();
function applyTheme() {
  document.documentElement.dataset.theme = theme;
  document.querySelectorAll(".theme-toggle button").forEach((b) => b.classList.toggle("active", b.dataset.themeSet === theme));
}
document.querySelectorAll(".theme-toggle button").forEach((b) => {
  b.addEventListener("click", () => {
    theme = b.dataset.themeSet;
    try { localStorage.setItem("datadiff.theme", theme); } catch {}
    applyTheme();
  });
});
applyTheme();

if (!window.__TAURI__?.core?.invoke) {
  document.getElementById("no-backend").hidden = false;
}

applyI18n();
