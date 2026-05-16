// packages/ui-components/src/islands/sui-component-picker/sui-component-picker.sui
import { adoptSuiSheets, attachRef, bindAttr, bindFor, bindIf, bindProp, bindText, bindValue, component, on, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-component-picker/sui-component-picker.controller.ts
import { controller, signal, derived, ref } from "@ship/sui-runtime";
var FALLBACK_CATALOG = "/api/assets/sui-custom-elements.json";
var CATEGORY_ICON = {
  layout: "layout",
  data: "database",
  form: "square-pen",
  feedback: "message-circle",
  navigation: "compass",
  overlay: "layers",
  display: "eye",
  chart: "bar-chart-3",
  text: "type",
  media: "image"
};
function categoryOf(tag) {
  if (/(card|stack|grid|panel|section|divider|separator|toolbar|page)/.test(tag))
    return "layout";
  if (/(input|textarea|checkbox|radio|select|slider|switch|toggle|field|label|form|file|button)/.test(tag))
    return "form";
  if (/(table|list|tile|tree|json|stat|metachip|kbd|code|markdown|diff|history)/.test(tag))
    return "data";
  if (/(alert|toast|notif|empty|loading|skeleton|progress|spinner|status)/.test(tag))
    return "feedback";
  if (/(nav|rail|breadcrumb|menu|tabs|tab|stepper|launcher|dock)/.test(tag))
    return "navigation";
  if (/(dialog|drawer|popover|tooltip|hover|context-menu|dropdown|portal)/.test(tag))
    return "overlay";
  if (/(chart|donut|gauge|sparkline|heatmap|meter|line|area|bar)/.test(tag))
    return "chart";
  if (/(icon|avatar|app-icon|theme|annotation|edit-overlay)/.test(tag))
    return "display";
  return "display";
}
function iconFor(tag) {
  return CATEGORY_ICON[categoryOf(tag)] ?? "package";
}
function flattenCem(modules) {
  const out = [];
  const seen = new Set;
  for (const m of modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName)
        continue;
      if (seen.has(d.tagName))
        continue;
      seen.add(d.tagName);
      const description = (d.description ?? "").trim();
      out.push({
        tag: d.tagName,
        description,
        firstDesc: description.split(`
`)[0] ?? "",
        icon: iconFor(d.tagName),
        category: categoryOf(d.tagName)
      });
    }
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag));
}
function fuzzyMatch(needle, hay) {
  if (!needle)
    return true;
  const n = needle.toLowerCase();
  const h = hay.toLowerCase();
  if (h.includes(n))
    return true;
  let i = 0;
  for (const c of h) {
    if (c === n[i]) {
      i += 1;
      if (i === n.length)
        return true;
    }
  }
  return false;
}
var sui_component_picker_controller_default = controller({
  scope: (state, host) => {
    const entries = signal([]);
    const filter = signal("");
    const queue = signal([]);
    const gridEl = ref(null);
    const s = state;
    const shown = derived(() => {
      const all = entries();
      const needle = filter().trim();
      const cat = s.categoryFilter().trim().toLowerCase();
      return all.filter((e) => {
        if (cat && e.category !== cat)
          return false;
        if (!needle)
          return true;
        return fuzzyMatch(needle, e.tag) || fuzzyMatch(needle, e.description);
      });
    });
    const isSelected = (tag) => queue().includes(tag);
    const h = host;
    function emitPicked(tags) {
      host.dispatchEvent(new CustomEvent("sui:component:picked", {
        detail: { tags },
        bubbles: true,
        composed: true
      }));
    }
    function onTileClick(tag) {
      if (s.selectMode() === "multi") {
        const q = queue();
        if (q.includes(tag))
          queue.set(q.filter((t) => t !== tag));
        else
          queue.set([...q, tag]);
        return;
      }
      emitPicked([tag]);
      if (s.mode() === "modal") {
        h.open = false;
      }
    }
    function commit() {
      const tags = queue();
      if (tags.length === 0)
        return;
      emitPicked([...tags]);
      queue.set([]);
      if (s.mode() === "modal")
        h.open = false;
    }
    function onDialogClose() {
      h.open = false;
    }
    return {
      entries,
      filter,
      queue,
      gridEl,
      shown,
      isSelected,
      onTileClick,
      commit,
      onDialogClose
    };
  },
  mount(state, host) {
    const s = state;
    const h = host;
    let loaded = false;
    let inflight = null;
    async function load() {
      if (loaded)
        return;
      if (inflight)
        return inflight;
      inflight = (async () => {
        try {
          const url = s.catalogUrl() || FALLBACK_CATALOG;
          const resp = await fetch(url, { credentials: "same-origin" });
          if (!resp.ok)
            return;
          const cem = await resp.json();
          s.entries.set(flattenCem(cem.modules ?? []));
          loaded = true;
        } catch {} finally {
          inflight = null;
        }
      })();
      return inflight;
    }
    Object.defineProperty(host, "open", {
      configurable: true,
      get: () => h.hasAttribute("open"),
      set: (v) => {
        if (v)
          h.setAttribute("open", "");
        else
          h.removeAttribute("open");
      }
    });
    Object.defineProperty(host, "refresh", {
      configurable: true,
      value: () => {
        loaded = false;
        s.entries.set([]);
        load();
      }
    });
    Object.defineProperty(host, "close", {
      configurable: true,
      value: () => {
        h.open = false;
      }
    });
    load();
  }
});

// packages/ui-components/src/islands/sui-component-picker/sui-component-picker.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_component_picker_default = component("sui-component-picker", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    mode: prop.union(["modal", "inline"], { default: "inline", reflect: true }),
    open: prop.bool({ default: false, reflect: true }),
    "select-mode": prop.union(["single", "multi"], { default: "single", reflect: true }),
    "catalog-url": prop.string({ default: "/api/assets/sui-custom-elements.json" }),
    placeholder: prop.string({ default: "Search components…" }),
    "category-filter": prop.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host { display: block; height: 100%; min-height: 0; }
  :host([mode="modal"]) { display: contents; }

  .root {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--card);
    color: var(--foreground);
    font-size: var(--text-sm);
    border-radius: var(--radius-md);
  }
  :host([mode="inline"]) .root {
    border: 1px solid var(--border-subtle, var(--border));
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: 8px 12px;
    border-bottom: 1px solid var(--border-subtle, var(--border));
    background: var(--sidebar, var(--card));
    flex: none;
  }
  .toolbar input {
    appearance: none;
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 8px;
    font: inherit;
    flex: 1;
    min-width: 0;
  }
  .toolbar input:focus { outline: none; border-color: var(--ring); box-shadow: 0 0 0 2px var(--ring); }
  .count {
    font-family: var(--mono-font, var(--font-mono));
    font-size: 11px;
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
  }

  .grid {
    flex: 1;
    overflow: auto;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 8px;
    padding: 10px;
    align-content: start;
  }

  .tile {
    appearance: none;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    text-align: left;
    cursor: pointer;
    font: inherit;
    color: var(--foreground);
    min-height: 78px;
    transition: border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .tile:hover {
    border-color: color-mix(in oklab, var(--primary) 60%, var(--border));
    background: color-mix(in oklab, var(--primary) 5%, var(--background));
    transform: translateY(-1px);
  }
  .tile:focus-visible {
    outline: none;
    border-color: var(--ring);
    box-shadow: 0 0 0 2px var(--ring);
  }
  .tile[data-selected="true"] {
    border-color: var(--primary);
    background: color-mix(in oklab, var(--primary) 12%, var(--background));
  }

  .tile-preview {
    min-height: 32px;
    display: grid;
    place-items: center;
    color: var(--muted-foreground);
    background: color-mix(in oklab, var(--muted) 50%, var(--background));
    border-radius: var(--radius-sm);
    padding: 6px;
    overflow: hidden;
    pointer-events: none;
  }
  .tile-tag {
    font-family: var(--mono-font, var(--font-mono));
    font-size: 11px;
    color: var(--primary);
    font-weight: var(--font-weight-medium, 500);
    word-break: break-all;
  }
  .tile-desc {
    font-size: 10.5px;
    color: var(--muted-foreground);
    line-height: 1.35;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 12px;
    border-top: 1px solid var(--border-subtle, var(--border));
    background: var(--sidebar, var(--card));
    flex: none;
    font-size: 11px;
    color: var(--muted-foreground);
  }
  .footer .queue {
    font-family: var(--mono-font, var(--font-mono));
    color: var(--foreground);
  }
  .footer button {
    appearance: none;
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    border: 0;
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    font: inherit;
    font-size: 11px;
  }
  .footer button[disabled] {
    background: var(--muted);
    color: var(--muted-foreground);
    cursor: not-allowed;
  }
  .browse-link a, ::slotted(a) {
    color: var(--primary);
    text-decoration: none;
  }
  .browse-link a:hover, ::slotted(a:hover) { text-decoration: underline; }

  .empty {
    grid-column: 1 / -1;
    padding: 24px;
    text-align: center;
    color: var(--muted-foreground);
    font-size: 12px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.mode = props.mode;
    $scope.open = props.open;
    $scope.selectMode = props.selectMode;
    $scope.catalogUrl = props.catalogUrl;
    $scope.placeholder = props.placeholder;
    $scope.categoryFilter = props.categoryFilter;
    $scope["gridEl"] = signal2(null);
    let __r = null;
    if (sui_component_picker_controller_default && typeof sui_component_picker_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_component_picker_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b9 = (anchor) => bindFor({
      anchor,
      items: () => $scope.shown(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.tag)(__child);
      },
      render: (entry, __i) => {
        const __childScope26 = Object.create($scope);
        __childScope26.entry = entry;
        __childScope26.__i = __i;
        __childScope26.__v = signal2(0);
        return (($scope2) => {
          const __frag27 = document.createDocumentFragment();
          const __c28 = [];
          const __e29 = document.createElement("button");
          __e29.setAttribute("class", "tile");
          __e29.setAttribute("type", "button");
          {
            const __av30 = $scope2.entry.tag;
            if (__av30 == null || __av30 === false || __av30 === "")
              __e29.removeAttribute("data-tag");
            else
              __e29.setAttribute("data-tag", __av30 === true ? "" : String(__av30));
          }
          __c28.push(bindAttr(__e29, "data-selected", () => (void $scope2.__v(), $scope2.queue().includes($scope2.entry.tag) ? "true" : "false")));
          __c28.push(bindAttr(__e29, "aria-pressed", () => (void $scope2.__v(), $scope2.queue().includes($scope2.entry.tag) ? "true" : "false")));
          {
            const __av31 = $scope2.entry.description;
            if (__av31 == null || __av31 === false || __av31 === "")
              __e29.removeAttribute("title");
            else
              __e29.setAttribute("title", __av31 === true ? "" : String(__av31));
          }
          __c28.push(on(__e29, "click", function() {
            const __h = () => $scope2.onTileClick($scope2.entry.tag);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e32 = document.createElement("div");
          __e32.setAttribute("class", "tile-preview");
          __e32.setAttribute("part", "preview");
          {
            const __av33 = $scope2.entry.tag;
            if (__av33 == null || __av33 === false || __av33 === "")
              __e32.removeAttribute("data-tag");
            else
              __e32.setAttribute("data-tag", __av33 === true ? "" : String(__av33));
          }
          const __e34 = document.createElement("sui-icon");
          {
            const __av35 = $scope2.entry.icon;
            if (__av35 == null || __av35 === false || __av35 === "")
              __e34.removeAttribute("name");
            else
              __e34.setAttribute("name", __av35 === true ? "" : String(__av35));
          }
          __e34.setAttribute("size", "md");
          __e32.appendChild(__e34);
          __e29.appendChild(__e32);
          const __e36 = document.createElement("span");
          __e36.setAttribute("class", "tile-tag");
          const __t37 = document.createTextNode("<");
          __e36.appendChild(__t37);
          const __t38 = document.createTextNode("");
          __e36.appendChild(__t38);
          __t38.textContent = $scope2.entry.tag;
          const __t39 = document.createTextNode(">");
          __e36.appendChild(__t39);
          __e29.appendChild(__e36);
          const __e40 = document.createElement("span");
          __e40.setAttribute("class", "tile-desc");
          const __t41 = document.createTextNode("");
          __e40.appendChild(__t41);
          __t41.textContent = $scope2.entry.firstDesc;
          __e29.appendChild(__e40);
          __frag27.appendChild(__e29);
          return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
            for (const c of __c28)
              try {
                c();
              } catch (e) {}
          }, update: (__new42, __newI43) => {
            $scope2.entry = __new42;
            $scope2.__i = __newI43;
            {
              const __av30 = $scope2.entry.tag;
              if (__av30 == null || __av30 === false || __av30 === "")
                __e29.removeAttribute("data-tag");
              else
                __e29.setAttribute("data-tag", __av30 === true ? "" : String(__av30));
            }
            {
              const __av31 = $scope2.entry.description;
              if (__av31 == null || __av31 === false || __av31 === "")
                __e29.removeAttribute("title");
              else
                __e29.setAttribute("title", __av31 === true ? "" : String(__av31));
            }
            {
              const __av33 = $scope2.entry.tag;
              if (__av33 == null || __av33 === false || __av33 === "")
                __e32.removeAttribute("data-tag");
              else
                __e32.setAttribute("data-tag", __av33 === true ? "" : String(__av33));
            }
            {
              const __av35 = $scope2.entry.icon;
              if (__av35 == null || __av35 === false || __av35 === "")
                __e34.removeAttribute("name");
              else
                __e34.setAttribute("name", __av35 === true ? "" : String(__av35));
            }
            __t38.textContent = $scope2.entry.tag;
            __t41.textContent = $scope2.entry.firstDesc;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope26);
      }
    });
    const __wireBoundary_b8 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.entries().length === 0)
          return 0;
        if ($scope.shown().length === 0)
          return 1;
        return 2;
      },
      branches: [
        () => {
          const __frag15 = document.createDocumentFragment();
          const __c16 = [];
          const __e17 = document.createElement("div");
          __e17.setAttribute("class", "empty");
          const __t18 = document.createTextNode("Loading components…");
          __e17.appendChild(__t18);
          __frag15.appendChild(__e17);
          return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
            for (const c of __c16)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag19 = document.createDocumentFragment();
          const __c20 = [];
          const __e21 = document.createElement("div");
          __e21.setAttribute("class", "empty");
          const __t22 = document.createTextNode("No matches.");
          __e21.appendChild(__t22);
          __frag19.appendChild(__e21);
          return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
            for (const c of __c20)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag23 = document.createDocumentFragment();
          const __c24 = [];
          const __forAnchor25 = document.createComment("for");
          __frag23.appendChild(__forAnchor25);
          __c24.push(__wireBoundary_b9(__forAnchor25));
          return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
            for (const c of __c24)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b19 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.selectMode() === "multi")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag50 = document.createDocumentFragment();
          const __c51 = [];
          const __e52 = document.createElement("span");
          __e52.setAttribute("class", "queue");
          const __t53 = document.createTextNode("");
          __e52.appendChild(__t53);
          __c51.push(bindText(__t53, () => $scope.queue().length));
          const __t54 = document.createTextNode(" queued");
          __e52.appendChild(__t54);
          __frag50.appendChild(__e52);
          const __e55 = document.createElement("button");
          __e55.setAttribute("type", "button");
          __c51.push(bindAttr(__e55, "disabled", () => $scope.queue().length === 0));
          __c51.push(on(__e55, "click", function() {
            const __h = $scope.commit;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t56 = document.createTextNode("Add");
          __e55.appendChild(__t56);
          __frag50.appendChild(__e55);
          return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
            for (const c of __c51)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b29 = (anchor) => bindFor({
      anchor,
      items: () => $scope.shown(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.tag)(__child);
      },
      render: (entry, __i) => {
        const __childScope79 = Object.create($scope);
        __childScope79.entry = entry;
        __childScope79.__i = __i;
        __childScope79.__v = signal2(0);
        return (($scope2) => {
          const __frag80 = document.createDocumentFragment();
          const __c81 = [];
          const __e82 = document.createElement("button");
          __e82.setAttribute("class", "tile");
          __e82.setAttribute("type", "button");
          {
            const __av83 = $scope2.entry.tag;
            if (__av83 == null || __av83 === false || __av83 === "")
              __e82.removeAttribute("data-tag");
            else
              __e82.setAttribute("data-tag", __av83 === true ? "" : String(__av83));
          }
          __c81.push(bindAttr(__e82, "data-selected", () => (void $scope2.__v(), $scope2.queue().includes($scope2.entry.tag) ? "true" : "false")));
          __c81.push(bindAttr(__e82, "aria-pressed", () => (void $scope2.__v(), $scope2.queue().includes($scope2.entry.tag) ? "true" : "false")));
          {
            const __av84 = $scope2.entry.description;
            if (__av84 == null || __av84 === false || __av84 === "")
              __e82.removeAttribute("title");
            else
              __e82.setAttribute("title", __av84 === true ? "" : String(__av84));
          }
          __c81.push(on(__e82, "click", function() {
            const __h = () => $scope2.onTileClick($scope2.entry.tag);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e85 = document.createElement("div");
          __e85.setAttribute("class", "tile-preview");
          __e85.setAttribute("part", "preview");
          {
            const __av86 = $scope2.entry.tag;
            if (__av86 == null || __av86 === false || __av86 === "")
              __e85.removeAttribute("data-tag");
            else
              __e85.setAttribute("data-tag", __av86 === true ? "" : String(__av86));
          }
          const __e87 = document.createElement("sui-icon");
          {
            const __av88 = $scope2.entry.icon;
            if (__av88 == null || __av88 === false || __av88 === "")
              __e87.removeAttribute("name");
            else
              __e87.setAttribute("name", __av88 === true ? "" : String(__av88));
          }
          __e87.setAttribute("size", "md");
          __e85.appendChild(__e87);
          __e82.appendChild(__e85);
          const __e89 = document.createElement("span");
          __e89.setAttribute("class", "tile-tag");
          const __t90 = document.createTextNode("<");
          __e89.appendChild(__t90);
          const __t91 = document.createTextNode("");
          __e89.appendChild(__t91);
          __t91.textContent = $scope2.entry.tag;
          const __t92 = document.createTextNode(">");
          __e89.appendChild(__t92);
          __e82.appendChild(__e89);
          const __e93 = document.createElement("span");
          __e93.setAttribute("class", "tile-desc");
          const __t94 = document.createTextNode("");
          __e93.appendChild(__t94);
          __t94.textContent = $scope2.entry.firstDesc;
          __e82.appendChild(__e93);
          __frag80.appendChild(__e82);
          return { nodes: Array.from(__frag80.childNodes), cleanup: () => {
            for (const c of __c81)
              try {
                c();
              } catch (e) {}
          }, update: (__new95, __newI96) => {
            $scope2.entry = __new95;
            $scope2.__i = __newI96;
            {
              const __av83 = $scope2.entry.tag;
              if (__av83 == null || __av83 === false || __av83 === "")
                __e82.removeAttribute("data-tag");
              else
                __e82.setAttribute("data-tag", __av83 === true ? "" : String(__av83));
            }
            {
              const __av84 = $scope2.entry.description;
              if (__av84 == null || __av84 === false || __av84 === "")
                __e82.removeAttribute("title");
              else
                __e82.setAttribute("title", __av84 === true ? "" : String(__av84));
            }
            {
              const __av86 = $scope2.entry.tag;
              if (__av86 == null || __av86 === false || __av86 === "")
                __e85.removeAttribute("data-tag");
              else
                __e85.setAttribute("data-tag", __av86 === true ? "" : String(__av86));
            }
            {
              const __av88 = $scope2.entry.icon;
              if (__av88 == null || __av88 === false || __av88 === "")
                __e87.removeAttribute("name");
              else
                __e87.setAttribute("name", __av88 === true ? "" : String(__av88));
            }
            __t91.textContent = $scope2.entry.tag;
            __t94.textContent = $scope2.entry.firstDesc;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope79);
      }
    });
    const __wireBoundary_b28 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.entries().length === 0)
          return 0;
        if ($scope.shown().length === 0)
          return 1;
        return 2;
      },
      branches: [
        () => {
          const __frag68 = document.createDocumentFragment();
          const __c69 = [];
          const __e70 = document.createElement("div");
          __e70.setAttribute("class", "empty");
          const __t71 = document.createTextNode("Loading components…");
          __e70.appendChild(__t71);
          __frag68.appendChild(__e70);
          return { nodes: Array.from(__frag68.childNodes), cleanup: () => {
            for (const c of __c69)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag72 = document.createDocumentFragment();
          const __c73 = [];
          const __e74 = document.createElement("div");
          __e74.setAttribute("class", "empty");
          const __t75 = document.createTextNode("No matches.");
          __e74.appendChild(__t75);
          __frag72.appendChild(__e74);
          return { nodes: Array.from(__frag72.childNodes), cleanup: () => {
            for (const c of __c73)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag76 = document.createDocumentFragment();
          const __c77 = [];
          const __forAnchor78 = document.createComment("for");
          __frag76.appendChild(__forAnchor78);
          __c77.push(__wireBoundary_b29(__forAnchor78));
          return { nodes: Array.from(__frag76.childNodes), cleanup: () => {
            for (const c of __c77)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b39 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.selectMode() === "multi")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag103 = document.createDocumentFragment();
          const __c104 = [];
          const __e105 = document.createElement("span");
          __e105.setAttribute("class", "queue");
          const __t106 = document.createTextNode("");
          __e105.appendChild(__t106);
          __c104.push(bindText(__t106, () => $scope.queue().length));
          const __t107 = document.createTextNode(" queued");
          __e105.appendChild(__t107);
          __frag103.appendChild(__e105);
          const __e108 = document.createElement("button");
          __e108.setAttribute("type", "button");
          __c104.push(bindAttr(__e108, "disabled", () => $scope.queue().length === 0));
          __c104.push(on(__e108, "click", function() {
            const __h = $scope.commit;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t109 = document.createTextNode("Add");
          __e108.appendChild(__t109);
          __frag103.appendChild(__e108);
          return { nodes: Array.from(__frag103.childNodes), cleanup: () => {
            for (const c of __c104)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.mode() === "modal")
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("sui-dialog");
          __c2.push(bindProp(__e3, "open", () => $scope.open()));
          __e3.setAttribute("size", "lg");
          __c2.push(on(__e3, "sui-close", function() {
            const __h = $scope.onDialogClose;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e4 = document.createElement("span");
          __e4.setAttribute("slot", "header");
          const __t5 = document.createTextNode("Pick a component");
          __e4.appendChild(__t5);
          __e3.appendChild(__e4);
          const __e6 = document.createElement("div");
          __e6.setAttribute("class", "root");
          __e6.setAttribute("part", "root");
          __e6.setAttribute("style", "height: 60vh");
          const __e7 = document.createElement("div");
          __e7.setAttribute("class", "toolbar");
          const __e8 = document.createElement("input");
          __e8.setAttribute("type", "text");
          __c2.push(bindValue(__e8, $scope.filter));
          __c2.push(bindAttr(__e8, "placeholder", () => $scope.placeholder()));
          __e8.setAttribute("aria-label", "Filter components");
          __e7.appendChild(__e8);
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "count");
          const __t10 = document.createTextNode("");
          __e9.appendChild(__t10);
          __c2.push(bindText(__t10, () => $scope.shown().length));
          const __t11 = document.createTextNode(" / ");
          __e9.appendChild(__t11);
          const __t12 = document.createTextNode("");
          __e9.appendChild(__t12);
          __c2.push(bindText(__t12, () => $scope.entries().length));
          __e7.appendChild(__e9);
          __e6.appendChild(__e7);
          const __e13 = document.createElement("div");
          __e13.setAttribute("class", "grid");
          attachRef($scope, "gridEl", __e13);
          const __ifAnchor14 = document.createComment("if");
          __e13.appendChild(__ifAnchor14);
          __c2.push(__wireBoundary_b8(__ifAnchor14));
          __e6.appendChild(__e13);
          const __e44 = document.createElement("div");
          __e44.setAttribute("class", "footer");
          const __e45 = document.createElement("span");
          __e45.setAttribute("class", "browse-link");
          const __e46 = document.createElement("slot");
          __e46.setAttribute("name", "browse-link");
          const __e47 = document.createElement("a");
          __e47.setAttribute("href", "/ship-ui-studio");
          const __t48 = document.createTextNode("Browse full catalog →");
          __e47.appendChild(__t48);
          __e46.appendChild(__e47);
          __e45.appendChild(__e46);
          __e44.appendChild(__e45);
          const __ifAnchor49 = document.createComment("if");
          __e44.appendChild(__ifAnchor49);
          __c2.push(__wireBoundary_b19(__ifAnchor49));
          __e6.appendChild(__e44);
          __e3.appendChild(__e6);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag57 = document.createDocumentFragment();
          const __c58 = [];
          const __e59 = document.createElement("div");
          __e59.setAttribute("class", "root");
          __e59.setAttribute("part", "root");
          const __e60 = document.createElement("div");
          __e60.setAttribute("class", "toolbar");
          const __e61 = document.createElement("input");
          __e61.setAttribute("type", "text");
          __c58.push(bindValue(__e61, $scope.filter));
          __c58.push(bindAttr(__e61, "placeholder", () => $scope.placeholder()));
          __e61.setAttribute("aria-label", "Filter components");
          __e60.appendChild(__e61);
          const __e62 = document.createElement("span");
          __e62.setAttribute("class", "count");
          const __t63 = document.createTextNode("");
          __e62.appendChild(__t63);
          __c58.push(bindText(__t63, () => $scope.shown().length));
          const __t64 = document.createTextNode(" / ");
          __e62.appendChild(__t64);
          const __t65 = document.createTextNode("");
          __e62.appendChild(__t65);
          __c58.push(bindText(__t65, () => $scope.entries().length));
          __e60.appendChild(__e62);
          __e59.appendChild(__e60);
          const __e66 = document.createElement("div");
          __e66.setAttribute("class", "grid");
          attachRef($scope, "gridEl", __e66);
          const __ifAnchor67 = document.createComment("if");
          __e66.appendChild(__ifAnchor67);
          __c58.push(__wireBoundary_b28(__ifAnchor67));
          __e59.appendChild(__e66);
          const __e97 = document.createElement("div");
          __e97.setAttribute("class", "footer");
          const __e98 = document.createElement("span");
          __e98.setAttribute("class", "browse-link");
          const __e99 = document.createElement("slot");
          __e99.setAttribute("name", "browse-link");
          const __e100 = document.createElement("a");
          __e100.setAttribute("href", "/ship-ui-studio");
          const __t101 = document.createTextNode("Browse full catalog →");
          __e100.appendChild(__t101);
          __e99.appendChild(__e100);
          __e98.appendChild(__e99);
          __e97.appendChild(__e98);
          const __ifAnchor102 = document.createComment("if");
          __e97.appendChild(__ifAnchor102);
          __c58.push(__wireBoundary_b39(__ifAnchor102));
          __e59.appendChild(__e97);
          __frag57.appendChild(__e59);
          return { nodes: Array.from(__frag57.childNodes), cleanup: () => {
            for (const c of __c58)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
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
