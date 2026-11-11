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
    "tab.diff": "Diferencias",
    "tab.script": "Script de sincronización",
    "tab.diagram": "Diagrama",
    "search.ph": "Buscar objeto…",
    "empty.filter": "No hay objetos que coincidan con el filtro.",
    "script.info": "SQL para dejar <b class=\"side-b-text\">B</b> igual que <b class=\"side-a-text\">A</b> · revísalo antes de ejecutarlo.",
    "script.none": "-- (sin cambios)",
    "diagram.legend": "Diagrama entidad-relación · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Diferencias",
    "diagram.sideA": "Origen A",
    "diagram.sideB": "Destino B",
    "diagram.empty": "No hay tablas para diagramar en este lado.",
    "diagram.noEngine": "No se pudo cargar el motor de diagrama.",
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
    "tab.diff": "Differences",
    "tab.script": "Synchronization script",
    "tab.diagram": "Diagram",
    "search.ph": "Search object…",
    "empty.filter": "No objects match the filter.",
    "script.info": "SQL to make <b class=\"side-b-text\">B</b> match <b class=\"side-a-text\">A</b> · review it before running.",
    "script.none": "-- (no changes)",
    "diagram.legend": "Entity-relationship diagram · 🔑 PK · 🔗 FK",
    "diagram.sideDiff": "Differences",
    "diagram.sideA": "Source A",
    "diagram.sideB": "Target B",
    "diagram.empty": "No tables to diagram on this side.",
    "diagram.noEngine": "Could not load the diagram engine.",
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
    document.getElementById("script-box").innerHTML = highlightSql(lastScript || t("script.none"));
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
    lastDiagramA = result.diagram_a || { tables: [], relations: [] };
    lastDiagramB = result.diagram_b || { tables: [], relations: [] };
    lastDiagramDiff = result.diagram_diff || { tables: [], relations: [] };
    renderResults();
    document.getElementById("script-box").innerHTML = highlightSql(lastScript || t("script.none"));
    document.getElementById("results").hidden = false;
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
  try {
    await navigator.clipboard.writeText(lastScript);
  } catch {
    const ta = document.createElement("textarea");
    ta.value = lastScript;
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
    if (path) await invoke("save_text_file", { path, content: lastScript });
  } catch (e) {
    errorBanner.textContent = errText(e);
    errorBanner.hidden = false;
  }
});

document.querySelectorAll(".diagram-toggle .seg").forEach((seg) => {
  seg.addEventListener("click", () => {
    diaSide = seg.dataset.dia;
    document.querySelectorAll(".diagram-toggle .seg").forEach((s) => s.classList.toggle("active", s === seg));
    renderDiagram();
  });
});

function renderDiagram() {
  const box = document.getElementById("diagram-box");
  const model = diaSide === "diff" ? lastDiagramDiff : diaSide === "a" ? lastDiagramA : lastDiagramB;
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

  const canvas = document.createElement("div");
  canvas.className = "er-canvas";
  const svg = document.createElementNS(SVGNS, "svg");
  svg.setAttribute("class", "er-edges");
  canvas.appendChild(svg);
  box.appendChild(canvas);

  const cards = {};
  for (const tbl of model.tables) {
    const card = buildTableCard(tbl);
    card.style.position = "absolute";
    card.style.visibility = "hidden";
    canvas.appendChild(card);
    cards[tbl.id] = card;
  }

  const g = new window.dagre.graphlib.Graph();
  g.setGraph({ rankdir: "TB", nodesep: 40, ranksep: 70, marginx: 18, marginy: 18 });
  g.setDefaultEdgeLabel(() => ({}));
  for (const tbl of model.tables) {
    const r = cards[tbl.id].getBoundingClientRect();
    g.setNode(tbl.id, { width: Math.ceil(r.width), height: Math.ceil(r.height) });
  }
  for (const rel of model.relations) {
    if (cards[rel.from] && cards[rel.to]) g.setEdge(rel.to, rel.from, { label: rel.label });
  }
  window.dagre.layout(g);

  for (const tbl of model.tables) {
    const n = g.node(tbl.id);
    const card = cards[tbl.id];
    card.style.left = Math.round(n.x - n.width / 2) + "px";
    card.style.top = Math.round(n.y - n.height / 2) + "px";
    card.style.visibility = "visible";
  }

  const gw = Math.ceil(g.graph().width) || 400;
  const gh = Math.ceil(g.graph().height) || 300;
  canvas.style.width = gw + "px";
  canvas.style.height = gh + "px";
  svg.setAttribute("width", gw);
  svg.setAttribute("height", gh);
  for (const e of g.edges()) {
    const ed = g.edge(e);
    const poly = document.createElementNS(SVGNS, "polyline");
    poly.setAttribute("points", ed.points.map((p) => `${p.x},${p.y}`).join(" "));
    poly.setAttribute("class", "er-edge");
    svg.appendChild(poly);
    if (ed.label) {
      const mid = ed.points[Math.floor(ed.points.length / 2)];
      const text = document.createElementNS(SVGNS, "text");
      text.setAttribute("x", mid.x);
      text.setAttribute("y", mid.y - 5);
      text.setAttribute("class", "er-edge-label");
      text.textContent = ed.label;
      svg.appendChild(text);
    }
  }
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

    const row = document.createElement("div");
    row.className = "row" + (hasDetails ? " expandable" : "");
    row.dataset.status = o.status;
    row.innerHTML =
      `<div class="row-main">` +
      `<span class="otype">${objIcon(o.object_type)}</span>` +
      `<span class="oname">${esc(o.name)}</span>` +
      `<span class="ometa">${esc(typeLabel(o.object_type))}</span>` +
      `<span class="spacer"></span>` +
      `<span class="badge ${m.badge}"><span class="dot"></span>${esc(t("status." + o.status))}</span>` +
      `<span class="chev">${hasDetails ? "▸" : ""}</span>` +
      `</div>`;

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
