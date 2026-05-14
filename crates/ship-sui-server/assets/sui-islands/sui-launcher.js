// packages/ui-components/src/islands/sui-launcher/sui-launcher.sui
import { adoptSuiSheets, attachRef, component, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-launcher/sui-launcher.controller.ts
import { controller, effect, signal, bindNativeEvent } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-launcher/render.ts
var KNOWN_APP_NAMES = new Set([
  "dashboard",
  "pipelines",
  "agents",
  "skills",
  "git",
  "collections",
  "canvas",
  "settings",
  "chat",
  "events",
  "specs",
  "theme",
  "theme-studio",
  "setup",
  "overview",
  "mesh"
]);
function humanize(id) {
  return id.replace(/-/g, " ").split(" ").map((w) => w ? w[0].toUpperCase() + w.slice(1) : w).join(" ");
}
function renderHead(el, project) {
  el.innerHTML = "";
  const icon = document.createElement("sui-icon");
  icon.setAttribute("name", "settings");
  icon.setAttribute("size", "md");
  icon.setAttribute("aria-hidden", "true");
  const title = document.createElement("span");
  title.className = "launcher-title";
  title.style.cssText = "font-weight:600;font-size:14px;flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;";
  title.textContent = project?.label ?? project?.name ?? "Ship";
  const ws = document.createElement("span");
  ws.className = "launcher-ws";
  ws.style.cssText = "font-size:11px;color:var(--muted-foreground,#888);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:160px;";
  ws.textContent = project?.path ?? "";
  el.append(icon, title, ws);
}
function renderFoot(el) {
  el.innerHTML = "";
  const add = document.createElement("a");
  add.className = "launcher-add";
  add.href = "#";
  add.style.cssText = "display:inline-flex;align-items:center;gap:4px;color:var(--muted-foreground,#888);text-decoration:none;font-size:12px;";
  add.addEventListener("click", (e) => e.preventDefault());
  const addIcon = document.createElement("sui-icon");
  addIcon.setAttribute("name", "plus");
  addIcon.setAttribute("size", "sm");
  add.appendChild(addIcon);
  add.appendChild(Object.assign(document.createElement("span"), { textContent: "Install view" }));
  const spacer = Object.assign(document.createElement("span"), { style: "flex:1" });
  el.append(add, spacer);
  for (const k of ["⌘", "⇧", "P"]) {
    const kbd = document.createElement("sui-kbd");
    kbd.textContent = k;
    el.appendChild(kbd);
  }
}
function renderBody(el, pinned, others, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName) {
  el.innerHTML = "";
  const total = pinned.length + others.length;
  if (total === 0) {
    const empty = document.createElement("div");
    empty.style.cssText = "padding:24px;text-align:center;color:var(--muted-foreground,#888);font-size:13px;";
    empty.textContent = "No views match.";
    el.appendChild(empty);
    return;
  }
  if (pinned.length > 0) {
    el.appendChild(buildSection("Pinned to rail", pinned, 0, true, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName));
  }
  if (others.length > 0) {
    el.appendChild(buildSection("All views", others, pinned.length, false, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName));
  }
}
function buildSection(label, rows, indexBase, isPinned, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName) {
  const section = document.createElement("div");
  section.style.cssText = "padding-bottom:4px;";
  const header = document.createElement("div");
  header.style.cssText = "display:flex;align-items:center;gap:4px;padding:8px 12px 4px;font-size:11px;font-weight:600;color:var(--muted-foreground,#888);text-transform:uppercase;letter-spacing:.04em;";
  if (isPinned) {
    const pinIcon = document.createElement("sui-icon");
    pinIcon.setAttribute("name", "pin");
    pinIcon.setAttribute("size", "sm");
    header.appendChild(pinIcon);
  }
  header.appendChild(Object.assign(document.createElement("span"), { textContent: label }));
  const spacer = Object.assign(document.createElement("span"), { style: "flex:1" });
  header.appendChild(spacer);
  if (isPinned) {
    header.appendChild(Object.assign(document.createElement("span"), {
      textContent: "drag to reorder",
      style: "font-size:10px;font-weight:400;color:var(--muted-foreground,#888);"
    }));
  }
  section.appendChild(header);
  rows.forEach((v, i) => {
    section.appendChild(buildRow(v, indexBase + i, isPinned, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName));
  });
  return section;
}
function buildRow(v, idx, isPinned, cursor, pins, onNavigate, onTogglePin, onDragStart, onDragOver, onDrop, onDragEnd, dragOverName, draggingName) {
  const label = v.rail?.label ?? humanize(v.name);
  const fallbackIcon = v.rail?.icon ?? "app-window";
  const known = KNOWN_APP_NAMES.has(v.name);
  const pinned = pins.includes(v.name);
  const isCursor = idx === cursor;
  const isDragOver = dragOverName === v.name;
  const isDragging = draggingName === v.name;
  const row = document.createElement("div");
  row.className = ["launcher-row", isCursor ? "is-cursor" : "", isDragOver ? "is-drag-over" : "", isDragging ? "is-dragging" : ""].filter(Boolean).join(" ");
  row.setAttribute("role", "button");
  row.setAttribute("tabindex", "-1");
  if (isPinned)
    row.setAttribute("draggable", "true");
  row.style.cssText = [
    "display:flex;align-items:center;gap:var(--space-2,8px);padding:6px 12px;cursor:pointer;",
    isCursor ? "background:var(--accent,color-mix(in oklch,var(--foreground)8%,transparent));" : "",
    isDragOver ? "outline:2px solid var(--primary,#3b82f6);outline-offset:-2px;" : "",
    isDragging ? "opacity:0.5;" : ""
  ].join("");
  const iconEl = document.createElement(known ? "sui-icon" : "sui-icon");
  iconEl.setAttribute("name", known ? fallbackIcon : fallbackIcon);
  iconEl.setAttribute("size", "md");
  const nameWrap = document.createElement("span");
  nameWrap.style.cssText = "flex:1;min-width:0;display:flex;flex-direction:column;gap:1px;";
  const labelEl = document.createElement("span");
  labelEl.style.cssText = "font-size:13px;color:var(--foreground,#111);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;";
  labelEl.textContent = label;
  const pathEl = document.createElement("span");
  pathEl.style.cssText = "font-size:11px;color:var(--muted-foreground,#888);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;";
  pathEl.textContent = v.source;
  nameWrap.append(labelEl, pathEl);
  const pinBtn = document.createElement("button");
  pinBtn.className = "launcher-pin" + (pinned ? " is-pinned" : "");
  pinBtn.type = "button";
  pinBtn.setAttribute("aria-label", pinned ? "Unpin from rail" : "Pin to rail");
  pinBtn.title = pinned ? "Unpin from rail" : "Pin to rail";
  pinBtn.style.cssText = "appearance:none;background:transparent;border:none;padding:2px;color:var(--muted-foreground,#888);cursor:pointer;display:inline-flex;align-items:center;opacity:" + (pinned ? "1" : "0") + ";flex-shrink:0;";
  const pinIcon = document.createElement("sui-icon");
  pinIcon.setAttribute("name", "pin");
  pinIcon.setAttribute("size", "sm");
  pinBtn.appendChild(pinIcon);
  row.append(iconEl, nameWrap, pinBtn);
  row.addEventListener("click", () => onNavigate(v.name));
  row.addEventListener("mouseenter", () => {});
  pinBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    onTogglePin(e, v.name);
  });
  if (isPinned) {
    row.addEventListener("dragstart", (e) => onDragStart(e, v.name));
    row.addEventListener("dragover", (e) => onDragOver(e, v.name));
    row.addEventListener("drop", (e) => onDrop(e, v.name));
    row.addEventListener("dragend", () => onDragEnd());
  }
  row.addEventListener("mouseenter", () => {
    pinBtn.style.opacity = "1";
  });
  row.addEventListener("mouseleave", () => {
    if (!pinned)
      pinBtn.style.opacity = "0";
  });
  return row;
}

// packages/ui-components/src/islands/sui-launcher/sui-launcher.controller.ts
var SETTING_KEY = "shell.rail.pinned";
var DEFAULT_PINS = ["dashboard", "pipelines", "agents", "skills", "git"];
async function fetchPins() {
  try {
    const resp = await fetch("/api/v1/settings", { credentials: "same-origin" });
    if (!resp.ok)
      return DEFAULT_PINS;
    const body = await resp.json();
    const raw = body.data?.[SETTING_KEY];
    if (typeof raw === "string") {
      try {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.every((v) => typeof v === "string"))
          return parsed;
      } catch {}
    } else if (Array.isArray(raw) && raw.every((v) => typeof v === "string")) {
      return raw;
    }
    return DEFAULT_PINS;
  } catch {
    return DEFAULT_PINS;
  }
}
async function savePins(pins) {
  try {
    await fetch("/api/v1/settings", {
      method: "PUT",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key: SETTING_KEY, value: JSON.stringify(pins) })
    });
  } catch {}
}
async function fetchViews() {
  try {
    const resp = await fetch("/api/v1/views", { credentials: "same-origin" });
    if (!resp.ok)
      return [];
    const body = await resp.json();
    return body.data?.views ?? [];
  } catch {
    return [];
  }
}
async function fetchProject() {
  try {
    const resp = await fetch("/api/v1/projects", { credentials: "same-origin" });
    if (!resp.ok)
      return null;
    const body = await resp.json();
    return body.projects.find((p) => p.id === body.active) ?? body.projects[0] ?? null;
  } catch {
    return null;
  }
}
function matchView(v, filterVal) {
  const f = filterVal.trim().toLowerCase();
  if (!f)
    return true;
  const label = (v.rail?.label ?? humanize(v.name)).toLowerCase();
  return label.includes(f) || v.source.toLowerCase().includes(f);
}
var sui_launcher_controller_default = controller({
  mount(state, host) {
    const scope = state;
    const views = signal([]);
    const pins = signal([]);
    const project = signal(null);
    const filter = signal("");
    const cursor = signal(0);
    let dragName = null;
    let dragOver = null;
    let filterTimer = null;
    const headEl = scope.headEl();
    const bodyEl = scope.bodyEl();
    const footEl = scope.footEl();
    const countEl = scope.countEl();
    const filterInput = scope.filterInput();
    const maskEl = scope.maskEl();
    if (footEl)
      renderFoot(footEl);
    const stopHead = effect(() => {
      if (headEl)
        renderHead(headEl, project());
    });
    const stopBody = effect(() => {
      const allViews = views();
      const allPins = pins();
      const f = filter();
      const cur = cursor();
      const pinned = allPins.map((n) => allViews.find((v) => v.name === n)).filter((v) => !!v && matchView(v, f));
      const pinSet = new Set(allPins);
      const others = allViews.filter((v) => !pinSet.has(v.name) && matchView(v, f));
      const total = pinned.length + others.length;
      if (countEl)
        countEl.textContent = String(total);
      if (bodyEl) {
        renderBody(bodyEl, pinned, others, cur, allPins, (name) => navigate(name), (e, name) => void togglePin(e, name), (e, name) => onDragStart(e, name), (e, name) => onDragOver(e, name), (e, name) => void onDrop(e, name), () => onDragEnd(), dragOver, dragName);
      }
    });
    let wasOpen = false;
    const stopOpen = effect(() => {
      const isOpen = scope.open();
      if (isOpen && !wasOpen) {
        cursor.set(0);
        filter.set("");
        load();
      } else if (!isOpen && wasOpen) {
        filter.set("");
        if (filterInput)
          filterInput.value = "";
      }
      wasOpen = isOpen;
    });
    if (maskEl)
      bindNativeEvent(maskEl, "click", () => doClose());
    if (filterInput)
      bindNativeEvent(filterInput, "input", () => {
        if (filterTimer)
          clearTimeout(filterTimer);
        filterTimer = setTimeout(() => {
          filter.set(filterInput.value);
          cursor.set(0);
        }, 80);
      });
    if (filterInput)
      bindNativeEvent(filterInput, "keydown", (e) => {
        if (["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(e.key))
          onGlobalKey(e);
      });
    function onGlobalKey(e) {
      if (!scope.open())
        return;
      if (e.key === "Escape") {
        e.preventDefault();
        doClose();
        return;
      }
      if (e.key === "/" && document.activeElement !== filterInput) {
        e.preventDefault();
        filterInput?.focus();
        return;
      }
      const allViews = views();
      const allPins = pins();
      const f = filter();
      const pinned = allPins.map((n) => allViews.find((v) => v.name === n)).filter((v) => !!v && matchView(v, f));
      const pinSet = new Set(allPins);
      const others = allViews.filter((v) => !pinSet.has(v.name) && matchView(v, f));
      const visible = [...pinned, ...others];
      if (e.key === "ArrowDown") {
        e.preventDefault();
        cursor.set(Math.min(visible.length - 1, cursor() + 1));
        scrollCursor();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        cursor.set(Math.max(0, cursor() - 1));
        scrollCursor();
      } else if (e.key === "Enter") {
        const row = visible[cursor()];
        if (!row)
          return;
        e.preventDefault();
        if (e.metaKey || e.ctrlKey) {
          host.emit("sui:launcher:open-pane", { name: row.name });
        }
        navigate(row.name);
      }
    }
    bindNativeEvent(document, "keydown", onGlobalKey);
    function scrollCursor() {
      requestAnimationFrame(() => {
        bodyEl?.querySelector(".launcher-row.is-cursor")?.scrollIntoView({ block: "nearest" });
      });
    }
    async function load() {
      const [loadedPins, loadedProject] = await Promise.all([
        fetchPins(),
        fetchProject(),
        fetchViews().then((v) => views.set(v))
      ]);
      pins.set(loadedPins);
      project.set(loadedProject);
      requestAnimationFrame(() => filterInput?.focus());
    }
    function navigate(name) {
      const path = "/" + (name === "dashboard" ? "" : name);
      history.pushState({}, "", path);
      document.dispatchEvent(new CustomEvent("sui:navigate", { detail: { to: path } }));
      doClose();
    }
    async function togglePin(e, name) {
      e.stopPropagation();
      const cur = pins();
      const next = cur.includes(name) ? cur.filter((n) => n !== name) : [...cur, name];
      pins.set(next);
      await savePins(next);
      host.emit("sui:rail:pins-changed", { pins: next });
    }
    function doClose() {
      scope.open.set(false);
    }
    function onDragStart(e, name) {
      dragName = name;
      e.dataTransfer?.setData("text/plain", name);
      if (e.dataTransfer)
        e.dataTransfer.effectAllowed = "move";
    }
    function onDragOver(e, name) {
      if (!dragName)
        return;
      e.preventDefault();
      if (e.dataTransfer)
        e.dataTransfer.dropEffect = "move";
      dragOver = name;
    }
    async function onDrop(e, target) {
      e.preventDefault();
      const src = dragName;
      dragName = null;
      dragOver = null;
      if (!src || src === target)
        return;
      const cur = pins();
      if (!cur.includes(src) || !cur.includes(target))
        return;
      const without = cur.filter((n) => n !== src);
      const idx = without.indexOf(target);
      const next = [...without.slice(0, idx), src, ...without.slice(idx)];
      pins.set(next);
      await savePins(next);
      host.emit("sui:rail:pins-changed", { pins: next });
    }
    function onDragEnd() {
      dragName = null;
      dragOver = null;
    }
    const onExtOpen = () => scope.open.set(true);
    const onExtClose = () => doClose();
    bindNativeEvent(document, "sui:launcher:open", onExtOpen);
    bindNativeEvent(document, "sui:launcher:close", onExtClose);
    return () => {
      stopHead();
      stopBody();
      stopOpen();
      if (filterTimer)
        clearTimeout(filterTimer);
    };
  }
});

// packages/ui-components/src/islands/sui-launcher/sui-launcher.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_launcher_default = component("sui-launcher", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    open: prop.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  :host(:not([open])) .launcher-mask,
  :host(:not([open])) .launcher-panel { display: none; }

  .launcher-mask {
    position: fixed;
    inset: 0;
    z-index: var(--z-overlay, 900);
    background: hsl(0 0% 0% / 0.35);
    animation: launcher-fade-in var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }

  .launcher-panel {
    position: fixed;
    top: 64px;
    left: 50%;
    transform: translateX(-50%);
    z-index: calc(var(--z-overlay, 900) + 1);
    width: min(560px, calc(100vw - 32px));
    max-height: min(560px, calc(100vh - 96px));
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    box-shadow: var(--shadow-xl, 0 20px 60px rgba(0,0,0,0.25));
    font-family: var(--font-body, system-ui, sans-serif);
    font-size: var(--text-sm, 13px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: launcher-slide-in var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }

  @keyframes launcher-fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }
  @keyframes launcher-slide-in {
    from { opacity: 0; transform: translateX(-50%) translateY(-8px) scale(0.97); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0) scale(1); }
  }

  /* head */
  .launcher-head {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: var(--space-3, 12px) var(--space-4, 16px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    flex-shrink: 0;
  }

  /* filter row */
  .launcher-filter {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: var(--space-2, 8px) var(--space-3, 12px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    flex-shrink: 0;
  }
  .launcher-filter-input {
    flex: 1;
    border: none;
    background: transparent;
    font: inherit;
    font-size: var(--text-sm, 13px);
    color: var(--foreground, #111);
    outline: none;
    min-width: 0;
  }
  .launcher-filter-input::placeholder { color: var(--muted-foreground, #888); }
  .launcher-count {
    font-size: 11px;
    color: var(--muted-foreground, #888);
    flex-shrink: 0;
  }

  /* body */
  .launcher-body {
    flex: 1;
    overflow-y: auto;
    overscroll-behavior: contain;
    min-height: 0;
    scrollbar-width: thin;
    scrollbar-color: color-mix(in oklch, var(--foreground) 15%, transparent) transparent;
  }

  /* foot */
  .launcher-foot {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: var(--space-2, 8px) var(--space-3, 12px);
    border-top: 1px solid var(--border, #e5e5e5);
    flex-shrink: 0;
  }

  @media (prefers-reduced-motion: reduce) {
    .launcher-mask, .launcher-panel { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope["bodyEl"] = signal2(null);
    $scope["countEl"] = signal2(null);
    $scope["filterInput"] = signal2(null);
    $scope["footEl"] = signal2(null);
    $scope["headEl"] = signal2(null);
    $scope["maskEl"] = signal2(null);
    $scope["panelEl"] = signal2(null);
    let __r = null;
    if (sui_launcher_controller_default && typeof sui_launcher_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_launcher_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "launcher-mask");
    __e0.setAttribute("part", "mask");
    attachRef($scope, "maskEl", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "launcher-panel");
    __e1.setAttribute("role", "dialog");
    __e1.setAttribute("aria-label", "App launcher");
    __e1.setAttribute("aria-modal", "true");
    __e1.setAttribute("part", "panel");
    attachRef($scope, "panelEl", __e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "launcher-head");
    __e2.setAttribute("part", "head");
    attachRef($scope, "headEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "launcher-filter");
    __e3.setAttribute("part", "filter");
    const __e4 = document.createElement("sui-icon");
    __e4.setAttribute("name", "search");
    __e4.setAttribute("size", "sm");
    __e4.setAttribute("aria-hidden", "true");
    __e3.appendChild(__e4);
    const __e5 = document.createElement("input");
    __e5.setAttribute("type", "text");
    __e5.setAttribute("class", "launcher-filter-input");
    __e5.setAttribute("placeholder", "Filter views…");
    __e5.setAttribute("aria-label", "Filter views");
    attachRef($scope, "filterInput", __e5);
    __e3.appendChild(__e5);
    const __e6 = document.createElement("span");
    __e6.setAttribute("class", "launcher-count");
    attachRef($scope, "countEl", __e6);
    const __t7 = document.createTextNode("0");
    __e6.appendChild(__t7);
    __e3.appendChild(__e6);
    __e1.appendChild(__e3);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "launcher-body");
    __e8.setAttribute("part", "body");
    attachRef($scope, "bodyEl", __e8);
    __e1.appendChild(__e8);
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "launcher-foot");
    __e9.setAttribute("part", "foot");
    attachRef($scope, "footEl", __e9);
    __e1.appendChild(__e9);
    __root.appendChild(__e1);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});
