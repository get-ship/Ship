// packages/ui-components/src/islands/sui-devtools/sui-devtools-components.sui
import { adoptSuiSheets, attachRef, bindAttr, bindFor, bindIf, bindText, bindValue, component } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-devtools/sui-devtools-components.controller.ts
import { controller, signal, derived, ref } from "@ship/sui-runtime";
function flattenCem(modules) {
  const out = [];
  for (const m of modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName)
        continue;
      const description = (d.description ?? "").trim();
      const attributes = d.attributes ?? [];
      const slots = d.slots ?? [];
      const snipAttrs = attributes.slice(0, 3).filter((a) => !a.default || a.default === "false").map((a) => (a.type?.text ?? "").includes("boolean") ? ` ${a.name}` : ` ${a.name}="…"`).join("");
      const hasDefault = slots.some((sl) => !sl.name || sl.name === "");
      const snippet = hasDefault ? `<${d.tagName}${snipAttrs}>
  
</${d.tagName}>` : `<${d.tagName}${snipAttrs}></${d.tagName}>`;
      out.push({
        tag: d.tagName,
        description,
        firstDesc: description.split(`
`)[0] ?? "",
        snippet,
        slotsLabel: slots.map((sl) => sl.name || "(default)").join(", "),
        attributes,
        slots
      });
    }
  }
  return out.sort((a, b) => a.tag.localeCompare(b.tag));
}
var sui_devtools_components_controller_default = controller({
  scope: () => {
    const entries = signal([]);
    const filter = signal("");
    const shown = derived(() => {
      const all = entries();
      const needle = filter().toLowerCase();
      return needle ? all.filter((e) => e.tag.toLowerCase().includes(needle) || e.description.toLowerCase().includes(needle)) : all;
    });
    const bodyEl = ref(null);
    return { entries, filter, shown, bodyEl };
  },
  mount(state, host) {
    const s = state;
    let loaded = false;
    async function load() {
      if (loaded)
        return;
      loaded = true;
      try {
        const resp = await fetch("/api/assets/custom-elements.json", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const cem = await resp.json();
        s["entries"] = flattenCem(cem.modules);
      } catch {}
    }
    const bodyRefFn = s["bodyEl"];
    const bodyEl = typeof bodyRefFn === "function" ? bodyRefFn() : null;
    if (bodyEl) {
      bodyEl.addEventListener("click", (e) => {
        const btn = e.target.closest(".copy[data-tag]");
        if (!btn)
          return;
        e.preventDefault();
        e.stopPropagation();
        const tag = btn.dataset.tag ?? "";
        const entry = s["entries"].find((x) => x.tag === tag);
        if (!entry)
          return;
        navigator.clipboard.writeText(entry.snippet).then(() => host.dispatchEvent(new CustomEvent("sui:notify", { detail: { tone: "success", title: "Copied" }, bubbles: true, composed: true })), () => host.dispatchEvent(new CustomEvent("sui:notify", { detail: { tone: "warning", title: "Copy failed" }, bubbles: true, composed: true })));
      });
    }
    Object.defineProperty(host, "refresh", {
      value: () => {
        loaded = false;
        s["entries"] = [];
        load();
      },
      configurable: true
    });
    load();
  }
});

// packages/ui-components/src/islands/sui-devtools/sui-devtools-components.sui
var sui_devtools_components_default = component("sui-devtools-components", {
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: var(--text-sm);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: 4px var(--space-3, 12px);
    border-bottom: 1px solid var(--border);
    background: var(--sidebar);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    flex-shrink: 0;
  }
  .toolbar input {
    appearance: none;
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    padding: 2px 8px;
    font: inherit;
    flex: 1;
    max-width: 280px;
  }
  .body { flex: 1; overflow: auto; }
  details { border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent); }
  summary {
    list-style: none;
    cursor: pointer;
    padding: 6px var(--space-3, 12px);
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-3, 12px);
    align-items: baseline;
    font-size: var(--text-xs);
  }
  summary::-webkit-details-marker { display: none; }
  .tag { font-family: var(--font-mono); color: var(--primary); }
  .desc { color: var(--muted-foreground); }
  .copy {
    appearance: none;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    padding: 1px 8px;
    font: inherit;
    font-size: 10px;
    color: var(--muted-foreground);
    cursor: pointer;
  }
  .copy:hover { color: var(--foreground); }
  .panel {
    padding: 0 var(--space-3, 12px) 10px calc(var(--space-3, 12px) + 14px);
    display: grid;
    gap: 8px;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }
  .panel pre {
    margin: 0;
    padding: 6px 10px;
    background: var(--muted);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    font-family: var(--font-mono);
    font-size: 11px;
    overflow-x: auto;
  }
  .attrs { display: grid; grid-template-columns: auto 1fr; gap: 2px 12px; }
  .attrs dt { font-family: var(--font-mono); color: var(--foreground); }
  .attrs dd { margin: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_devtools_components_controller_default && typeof sui_devtools_components_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_devtools_components_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindFor({
      anchor,
      items: () => $scope.shown(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.tag)(__child);
      },
      render: (entry, __i) => {
        const __childScope25 = Object.create($scope);
        __childScope25.entry = entry;
        __childScope25.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b11 = (anchor2) => bindFor({
            anchor: anchor2,
            items: () => $scope2.entry.attributes,
            key: (attr, __i2) => {
              const __child = Object.create($scope2);
              __child.attr = attr;
              __child.__i = __i2;
              return (($scope3) => attr.name)(__child);
            },
            render: (attr, __i2) => {
              const __childScope49 = Object.create($scope2);
              __childScope49.attr = attr;
              __childScope49.__i = __i2;
              return (($scope3) => {
                const __frag50 = document.createDocumentFragment();
                const __c51 = [];
                const __e52 = document.createElement("dt");
                const __t53 = document.createTextNode("");
                __e52.appendChild(__t53);
                __c51.push(bindText(__t53, () => $scope3.attr.name));
                const __t54 = document.createTextNode("");
                __e52.appendChild(__t54);
                __c51.push(bindText(__t54, () => $scope3.attr.default ? "=" + $scope3.attr.default : ""));
                __frag50.appendChild(__e52);
                const __e55 = document.createElement("dd");
                const __t56 = document.createTextNode("");
                __e55.appendChild(__t56);
                __c51.push(bindText(__t56, () => $scope3.attr.description ?? ""));
                __frag50.appendChild(__e55);
                return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
                  for (const c of __c51)
                    try {
                      c();
                    } catch (e) {}
                } };
              })(__childScope49);
            }
          });
          const __wireBoundary_b10 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.entry.attributes.length > 0)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag42 = document.createDocumentFragment();
                const __c43 = [];
                const __e44 = document.createElement("div");
                const __e45 = document.createElement("strong");
                const __t46 = document.createTextNode("Attributes");
                __e45.appendChild(__t46);
                __e44.appendChild(__e45);
                const __e47 = document.createElement("dl");
                __e47.setAttribute("class", "attrs");
                const __forAnchor48 = document.createComment("for");
                __e47.appendChild(__forAnchor48);
                __c43.push(__wireBoundary_b11(__forAnchor48));
                __e44.appendChild(__e47);
                __frag42.appendChild(__e44);
                return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
                  for (const c of __c43)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __wireBoundary_b15 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.entry.slots.length > 0)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag58 = document.createDocumentFragment();
                const __c59 = [];
                const __e60 = document.createElement("div");
                const __e61 = document.createElement("strong");
                const __t62 = document.createTextNode("Slots");
                __e61.appendChild(__t62);
                __e60.appendChild(__e61);
                const __t63 = document.createTextNode(": ");
                __e60.appendChild(__t63);
                const __t64 = document.createTextNode("");
                __e60.appendChild(__t64);
                __c59.push(bindText(__t64, () => $scope2.entry.slotsLabel));
                __frag58.appendChild(__e60);
                return { nodes: Array.from(__frag58.childNodes), cleanup: () => {
                  for (const c of __c59)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag26 = document.createDocumentFragment();
          const __c27 = [];
          const __e28 = document.createElement("details");
          const __e29 = document.createElement("summary");
          const __e30 = document.createElement("span");
          __e30.setAttribute("class", "tag");
          const __t31 = document.createTextNode("<");
          __e30.appendChild(__t31);
          const __t32 = document.createTextNode("");
          __e30.appendChild(__t32);
          __c27.push(bindText(__t32, () => $scope2.entry.tag));
          const __t33 = document.createTextNode(">");
          __e30.appendChild(__t33);
          __e29.appendChild(__e30);
          const __e34 = document.createElement("span");
          __e34.setAttribute("class", "desc");
          const __t35 = document.createTextNode("");
          __e34.appendChild(__t35);
          __c27.push(bindText(__t35, () => $scope2.entry.firstDesc));
          __e29.appendChild(__e34);
          const __e36 = document.createElement("button");
          __e36.setAttribute("class", "copy");
          __c27.push(bindAttr(__e36, "data-tag", () => $scope2.entry.tag));
          __e36.setAttribute("title", "Copy snippet");
          const __t37 = document.createTextNode("Copy");
          __e36.appendChild(__t37);
          __e29.appendChild(__e36);
          __e28.appendChild(__e29);
          const __e38 = document.createElement("div");
          __e38.setAttribute("class", "panel");
          const __e39 = document.createElement("pre");
          const __t40 = document.createTextNode("");
          __e39.appendChild(__t40);
          __c27.push(bindText(__t40, () => $scope2.entry.snippet));
          __e38.appendChild(__e39);
          const __ifAnchor41 = document.createComment("if");
          __e38.appendChild(__ifAnchor41);
          __c27.push(__wireBoundary_b10(__ifAnchor41));
          const __ifAnchor57 = document.createComment("if");
          __e38.appendChild(__ifAnchor57);
          __c27.push(__wireBoundary_b15(__ifAnchor57));
          __e28.appendChild(__e38);
          __frag26.appendChild(__e28);
          return { nodes: Array.from(__frag26.childNodes), cleanup: () => {
            for (const c of __c27)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope25);
      }
    });
    const __wireBoundary_b4 = (anchor) => bindIf({
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
          const __frag8 = document.createDocumentFragment();
          const __c9 = [];
          const __e10 = document.createElement("sui-empty-state");
          const __e11 = document.createElement("sui-icon");
          __e11.setAttribute("slot", "icon");
          __e11.setAttribute("name", "package");
          __e10.appendChild(__e11);
          const __e12 = document.createElement("span");
          __e12.setAttribute("slot", "title");
          const __t13 = document.createTextNode("Loading components…");
          __e12.appendChild(__t13);
          __e10.appendChild(__e12);
          const __t14 = document.createTextNode(`
      Fetching the component catalog from the daemon.
    `);
          __e10.appendChild(__t14);
          __frag8.appendChild(__e10);
          return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
            for (const c of __c9)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag15 = document.createDocumentFragment();
          const __c16 = [];
          const __e17 = document.createElement("sui-empty-state");
          const __e18 = document.createElement("sui-icon");
          __e18.setAttribute("slot", "icon");
          __e18.setAttribute("name", "search");
          __e17.appendChild(__e18);
          const __e19 = document.createElement("span");
          __e19.setAttribute("slot", "title");
          const __t20 = document.createTextNode("No matches");
          __e19.appendChild(__t20);
          __e17.appendChild(__e19);
          const __t21 = document.createTextNode(`
      No components match the current filter.
    `);
          __e17.appendChild(__t21);
          __frag15.appendChild(__e17);
          return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
            for (const c of __c16)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag22 = document.createDocumentFragment();
          const __c23 = [];
          const __forAnchor24 = document.createComment("for");
          __frag22.appendChild(__forAnchor24);
          __c23.push(__wireBoundary_b5(__forAnchor24));
          return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
            for (const c of __c23)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "toolbar");
    const __e1 = document.createElement("input");
    __e1.setAttribute("type", "text");
    __e1.setAttribute("placeholder", "Filter components…");
    __cleanups.push(bindValue(__e1, $scope.filter));
    __e0.appendChild(__e1);
    const __e2 = document.createElement("span");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText(__t3, () => $scope.shown().length));
    const __t4 = document.createTextNode(" / ");
    __e2.appendChild(__t4);
    const __t5 = document.createTextNode("");
    __e2.appendChild(__t5);
    __cleanups.push(bindText(__t5, () => $scope.entries().length));
    __e0.appendChild(__e2);
    __root.appendChild(__e0);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "body");
    attachRef($scope, "bodyEl", __e6);
    const __ifAnchor7 = document.createComment("if");
    __e6.appendChild(__ifAnchor7);
    __cleanups.push(__wireBoundary_b4(__ifAnchor7));
    __root.appendChild(__e6);
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

// packages/ui-components/src/islands/sui-devtools/sui-devtools-lint.sui
import { adoptSuiSheets as adoptSuiSheets2, bindClass, bindFor as bindFor2, bindIf as bindIf2, bindText as bindText2, component as component2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-devtools/sui-devtools-lint.controller.ts
import { controller as controller2, signal as signal2, derived as derived2 } from "@ship/sui-runtime";
var sui_devtools_lint_controller_default = controller2({
  scope: () => {
    const findings = signal2([]);
    const filter = signal2("all");
    const viewName = signal2(null);
    const loading = signal2(false);
    const errorMsg = signal2(null);
    const errCount = derived2(() => findings().filter((f) => f.severity === "error").length);
    const warnCount = derived2(() => findings().filter((f) => f.severity === "warn").length);
    const filteredFindings = derived2(() => {
      const f = filter();
      const sorted = [...findings()].sort((a, b) => {
        const sv = (s) => s === "error" ? 0 : s === "warn" ? 1 : 2;
        const d = sv(a.severity) - sv(b.severity);
        if (d !== 0)
          return d;
        if (a.file !== b.file)
          return a.file.localeCompare(b.file);
        return a.line !== b.line ? a.line - b.line : a.col - b.col;
      });
      if (f === "all")
        return sorted;
      return sorted.filter((x) => x.severity === f);
    });
    return { findings, filter, viewName, loading, errorMsg, errCount, warnCount, filteredFindings };
  },
  mount(state, host) {
    const s = state;
    function topLevelName() {
      const p = window.ship?.current().path ?? location.pathname;
      const m = p.match(/^\/api\/v\/([^/]+)/);
      if (m)
        return m[1];
      const seg = p.replace(/^\/+/, "").split("/")[0];
      return seg || null;
    }
    async function refresh() {
      const name = topLevelName();
      s["viewName"] = name;
      if (!name) {
        s["findings"] = [];
        s["errorMsg"] = null;
        return;
      }
      s["loading"] = true;
      s["errorMsg"] = null;
      try {
        const resp = await fetch("/api/v1/lint/view", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ name })
        });
        if (resp.status === 404) {
          s["findings"] = [];
          return;
        }
        if (!resp.ok) {
          s["errorMsg"] = `lint failed: ${resp.status}`;
          return;
        }
        const body = await resp.json();
        s["findings"] = body.data?.findings ?? [];
      } catch (e) {
        s["errorMsg"] = e.message;
      } finally {
        s["loading"] = false;
      }
    }
    const onNavigate = () => void refresh();
    const onDockActivate = (ev) => {
      if (ev.detail?.tab === "lint")
        refresh();
    };
    document.addEventListener("sui:navigate", onNavigate);
    host.closest("sui-panel-dock")?.addEventListener("sui:dock:activate", onDockActivate);
    Object.defineProperty(host, "refresh", { value: () => void refresh(), configurable: true });
    refresh();
    return () => {
      document.removeEventListener("sui:navigate", onNavigate);
      host.closest("sui-panel-dock")?.removeEventListener("sui:dock:activate", onDockActivate);
    };
  }
});

// packages/ui-components/src/islands/sui-devtools/sui-devtools-lint.sui
var sui_devtools_lint_default = component2("sui-devtools-lint", {
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: var(--text-sm);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
    padding: 4px var(--space-3, 12px);
    border-bottom: 1px solid var(--border);
    color: var(--muted-foreground);
    background: var(--sidebar);
    flex-shrink: 0;
    font-size: var(--text-xs);
  }
  .body {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  .row {
    display: grid;
    grid-template-columns: auto auto 1fr;
    gap: 12px;
    padding: 4px var(--space-3, 12px);
    align-items: baseline;
    border-bottom: 1px solid color-mix(in oklch, var(--border) 30%, transparent);
  }
  .sev { width: 52px; text-transform: uppercase; font-size: 10px; letter-spacing: 0.06em; }
  .sev.err { color: var(--destructive); }
  .sev.warn { color: var(--warning, var(--muted-foreground)); }
  .loc { color: var(--muted-foreground); white-space: nowrap; }
  .rule { color: var(--primary); margin-right: 8px; }
  .count {
    padding: 1px 6px;
    border-radius: 999px;
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .count.err { color: var(--destructive); border: 1px solid var(--destructive); }
  .count.warn { color: var(--warning, var(--muted-foreground)); border: 1px solid color-mix(in oklch, var(--warning, var(--muted-foreground)) 60%, transparent); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_devtools_lint_controller_default && typeof sui_devtools_lint_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_devtools_lint_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.viewName())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("span");
          __e4.setAttribute("style", "color:var(--foreground);font-weight:var(--font-weight-medium,500)");
          const __t5 = document.createTextNode("");
          __e4.appendChild(__t5);
          __c3.push(bindText2(__t5, () => $scope.viewName()));
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("span");
          const __t9 = document.createTextNode("No view");
          __e8.appendChild(__t9);
          __frag6.appendChild(__e8);
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b2 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.errCount() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("span");
          __e14.setAttribute("class", "count err");
          const __t15 = document.createTextNode("");
          __e14.appendChild(__t15);
          __c13.push(bindText2(__t15, () => $scope.errCount()));
          const __t16 = document.createTextNode(" error");
          __e14.appendChild(__t16);
          __frag12.appendChild(__e14);
          return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
            for (const c of __c13)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b4 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.warnCount() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("span");
          __e20.setAttribute("class", "count warn");
          const __t21 = document.createTextNode("");
          __e20.appendChild(__t21);
          __c19.push(bindText2(__t21, () => $scope.warnCount()));
          const __t22 = document.createTextNode(" warn");
          __e20.appendChild(__t22);
          __frag18.appendChild(__e20);
          return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
            for (const c of __c19)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b6 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag24 = document.createDocumentFragment();
          const __c25 = [];
          const __e26 = document.createElement("span");
          const __t27 = document.createTextNode("Linting…");
          __e26.appendChild(__t27);
          __frag24.appendChild(__e26);
          return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
            for (const c of __c25)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b9 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.filteredFindings(),
      key: (f, __i) => {
        const __child = Object.create($scope);
        __child.f = f;
        __child.__i = __i;
        return (($scope2) => f.rule + f.line + f.col)(__child);
      },
      render: (f, __i) => {
        const __childScope51 = Object.create($scope);
        __childScope51.f = f;
        __childScope51.__i = __i;
        return (($scope2) => {
          const __frag52 = document.createDocumentFragment();
          const __c53 = [];
          const __e54 = document.createElement("div");
          __e54.setAttribute("class", "row");
          const __e55 = document.createElement("span");
          __e55.setAttribute("class", "sev");
          __c53.push(bindClass(__e55, "err", () => $scope2.f.severity === "error"));
          __c53.push(bindClass(__e55, "warn", () => $scope2.f.severity !== "error"));
          const __t56 = document.createTextNode("");
          __e55.appendChild(__t56);
          __c53.push(bindText2(__t56, () => $scope2.f.severity));
          __e54.appendChild(__e55);
          const __e57 = document.createElement("span");
          __e57.setAttribute("class", "loc");
          const __t58 = document.createTextNode("");
          __e57.appendChild(__t58);
          __c53.push(bindText2(__t58, () => $scope2.f.file));
          const __t59 = document.createTextNode(":");
          __e57.appendChild(__t59);
          const __t60 = document.createTextNode("");
          __e57.appendChild(__t60);
          __c53.push(bindText2(__t60, () => $scope2.f.line));
          const __t61 = document.createTextNode(":");
          __e57.appendChild(__t61);
          const __t62 = document.createTextNode("");
          __e57.appendChild(__t62);
          __c53.push(bindText2(__t62, () => $scope2.f.col));
          __e54.appendChild(__e57);
          const __e63 = document.createElement("span");
          const __e64 = document.createElement("span");
          __e64.setAttribute("class", "rule");
          const __t65 = document.createTextNode("[");
          __e64.appendChild(__t65);
          const __t66 = document.createTextNode("");
          __e64.appendChild(__t66);
          __c53.push(bindText2(__t66, () => $scope2.f.rule));
          const __t67 = document.createTextNode("]");
          __e64.appendChild(__t67);
          __e63.appendChild(__e64);
          const __t68 = document.createTextNode("");
          __e63.appendChild(__t68);
          __c53.push(bindText2(__t68, () => $scope2.f.message));
          __e54.appendChild(__e63);
          __frag52.appendChild(__e54);
          return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
            for (const c of __c53)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope51);
      }
    });
    const __wireBoundary_b7 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.errorMsg())
          return 0;
        if (!$scope.viewName() && !$scope.loading())
          return 1;
        if ($scope.filteredFindings().length === 0 && !$scope.loading())
          return 2;
        return 3;
      },
      branches: [
        () => {
          const __frag30 = document.createDocumentFragment();
          const __c31 = [];
          const __e32 = document.createElement("div");
          __e32.setAttribute("style", "padding:20px;color:var(--muted-foreground)");
          const __t33 = document.createTextNode("");
          __e32.appendChild(__t33);
          __c31.push(bindText2(__t33, () => $scope.errorMsg()));
          __frag30.appendChild(__e32);
          return { nodes: Array.from(__frag30.childNodes), cleanup: () => {
            for (const c of __c31)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag34 = document.createDocumentFragment();
          const __c35 = [];
          const __e36 = document.createElement("sui-empty-state");
          const __e37 = document.createElement("sui-icon");
          __e37.setAttribute("slot", "icon");
          __e37.setAttribute("name", "check-circle");
          __e36.appendChild(__e37);
          const __e38 = document.createElement("span");
          __e38.setAttribute("slot", "title");
          const __t39 = document.createTextNode("No view");
          __e38.appendChild(__t39);
          __e36.appendChild(__e38);
          const __t40 = document.createTextNode(`
      Navigate to a view to lint it.
    `);
          __e36.appendChild(__t40);
          __frag34.appendChild(__e36);
          return { nodes: Array.from(__frag34.childNodes), cleanup: () => {
            for (const c of __c35)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag41 = document.createDocumentFragment();
          const __c42 = [];
          const __e43 = document.createElement("sui-empty-state");
          const __e44 = document.createElement("sui-icon");
          __e44.setAttribute("slot", "icon");
          __e44.setAttribute("name", "check-circle");
          __e43.appendChild(__e44);
          const __e45 = document.createElement("span");
          __e45.setAttribute("slot", "title");
          const __t46 = document.createTextNode("Clean");
          __e45.appendChild(__t46);
          __e43.appendChild(__e45);
          const __t47 = document.createTextNode(`
      No lint errors in current view.
    `);
          __e43.appendChild(__t47);
          __frag41.appendChild(__e43);
          return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
            for (const c of __c42)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag48 = document.createDocumentFragment();
          const __c49 = [];
          const __forAnchor50 = document.createComment("for");
          __frag48.appendChild(__forAnchor50);
          __c49.push(__wireBoundary_b9(__forAnchor50));
          return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
            for (const c of __c49)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "toolbar");
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b0(__ifAnchor1));
    const __e10 = document.createElement("span");
    __e10.setAttribute("style", "flex:1");
    __e0.appendChild(__e10);
    const __ifAnchor11 = document.createComment("if");
    __e0.appendChild(__ifAnchor11);
    __cleanups.push(__wireBoundary_b2(__ifAnchor11));
    const __ifAnchor17 = document.createComment("if");
    __e0.appendChild(__ifAnchor17);
    __cleanups.push(__wireBoundary_b4(__ifAnchor17));
    const __ifAnchor23 = document.createComment("if");
    __e0.appendChild(__ifAnchor23);
    __cleanups.push(__wireBoundary_b6(__ifAnchor23));
    __root.appendChild(__e0);
    const __e28 = document.createElement("div");
    __e28.setAttribute("class", "body");
    const __ifAnchor29 = document.createComment("if");
    __e28.appendChild(__ifAnchor29);
    __cleanups.push(__wireBoundary_b7(__ifAnchor29));
    __root.appendChild(__e28);
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

// packages/ui-components/src/islands/sui-devtools/sui-devtools-schema.sui
import { adoptSuiSheets as adoptSuiSheets3, bindClass as bindClass2, bindFor as bindFor3, bindIf as bindIf3, bindText as bindText3, component as component3 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-devtools/sui-devtools-schema.controller.ts
import { controller as controller3, signal as signal3, derived as derived3 } from "@ship/sui-runtime";
var SHIP_OP_RE = /ship\.(try\.)?([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)/gi;
function collectRefs(src, ops) {
  const seen = new Map;
  let m;
  SHIP_OP_RE.lastIndex = 0;
  while ((m = SHIP_OP_RE.exec(src)) !== null) {
    const soft = !!m[1];
    const name = `${m[2]}.${m[3]}`;
    const key = `${soft ? "try:" : ""}${name}`;
    if (seen.has(key))
      continue;
    const desc = ops.get(name);
    seen.set(key, {
      op: name,
      soft,
      known: ops.has(name),
      description: desc?.description ?? "(not in catalog)",
      permission: desc?.permission ?? "",
      args: desc ? desc.args : null,
      argsSummary: desc ? desc.args.map((a) => `${a.name}${a.required ? "" : "?"}:${a.kind}`).join(" ") : "",
      returns: desc?.returns ?? null,
      sensitive: desc?.sensitive ?? false
    });
  }
  return [...seen.values()].sort((a, b) => a.op.localeCompare(b.op));
}
var sui_devtools_schema_controller_default = controller3({
  scope: () => {
    const refs = signal3([]);
    const errorMsg = signal3(null);
    const knownCount = derived3(() => refs().filter((r) => r.known).length);
    return { refs, errorMsg, knownCount };
  },
  mount(state, host) {
    const s = state;
    let ops = new Map;
    let lastPath = null;
    async function loadOps() {
      if (ops.size > 0)
        return;
      try {
        const resp = await fetch("/api/v1/ship/schema", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const body = await resp.json();
        ops = new Map(body.data.ops.map((op) => [op.name, op]));
      } catch {}
    }
    function stripPrefix(path) {
      const m = path.match(/^\/api\/v\/(.+)$/);
      return m ? m[1] : path.replace(/^\/+/, "");
    }
    async function refresh() {
      const current = window.ship?.current();
      const path = current?.path ?? location.pathname;
      const stripped = stripPrefix(path);
      if (!stripped) {
        s["refs"] = [];
        s["errorMsg"] = null;
        return;
      }
      if (stripped === lastPath)
        return;
      lastPath = stripped;
      s["errorMsg"] = null;
      await loadOps();
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: "same-origin" });
        if (resp.status === 404) {
          s["refs"] = [];
          return;
        }
        if (!resp.ok) {
          s["errorMsg"] = `fetch failed: ${resp.status}`;
          return;
        }
        const src = await resp.json();
        s["refs"] = collectRefs(`${src.template}
${src.manifest ?? ""}`, ops);
      } catch (e) {
        s["errorMsg"] = e.message;
      }
    }
    const onNavigate = () => {
      lastPath = null;
      refresh();
    };
    const onDockActivate = (ev) => {
      if (ev.detail?.tab === "schema") {
        lastPath = null;
        refresh();
      }
    };
    document.addEventListener("sui:navigate", onNavigate);
    host.closest("sui-panel-dock")?.addEventListener("sui:dock:activate", onDockActivate);
    Object.defineProperty(host, "refresh", { value: () => {
      lastPath = null;
      refresh();
    }, configurable: true });
    refresh();
    return () => {
      document.removeEventListener("sui:navigate", onNavigate);
      host.closest("sui-panel-dock")?.removeEventListener("sui:dock:activate", onDockActivate);
    };
  }
});

// packages/ui-components/src/islands/sui-devtools/sui-devtools-schema.sui
var sui_devtools_schema_default = component3("sui-devtools-schema", {
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: var(--text-sm);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
    padding: 4px var(--space-3, 12px);
    border-bottom: 1px solid var(--border);
    background: var(--sidebar);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    flex-shrink: 0;
  }
  .body { flex: 1; overflow: auto; }
  details { border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent); }
  details > summary {
    list-style: none;
    cursor: pointer;
    padding: 6px var(--space-3, 12px);
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-3, 12px);
    align-items: baseline;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }
  details > summary::-webkit-details-marker { display: none; }
  .op { color: var(--primary); }
  .op.soft::before { content: 'try · '; color: var(--muted-foreground); }
  .op.unknown { color: var(--destructive); }
  .perm { color: var(--muted-foreground); }
  .desc { color: var(--muted-foreground); }
  .detail {
    padding: 4px var(--space-3, 12px) 10px calc(var(--space-3, 12px) + 14px);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    display: grid;
    gap: 4px;
    font-family: var(--font-mono);
  }
  .detail code { color: var(--foreground); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_devtools_schema_controller_default && typeof sui_devtools_schema_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_devtools_schema_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.refs().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("span");
          const __t5 = document.createTextNode("");
          __e4.appendChild(__t5);
          __c3.push(bindText3(__t5, () => $scope.refs().length));
          const __t6 = document.createTextNode(" ops referenced");
          __e4.appendChild(__t6);
          __frag2.appendChild(__e4);
          const __e7 = document.createElement("span");
          __e7.setAttribute("style", "flex:1");
          __frag2.appendChild(__e7);
          const __e8 = document.createElement("span");
          const __t9 = document.createTextNode("");
          __e8.appendChild(__t9);
          __c3.push(bindText3(__t9, () => $scope.knownCount()));
          const __t10 = document.createTextNode(" known · ");
          __e8.appendChild(__t10);
          const __t11 = document.createTextNode("");
          __e8.appendChild(__t11);
          __c3.push(bindText3(__t11, () => $scope.refs().length - $scope.knownCount()));
          const __t12 = document.createTextNode(" unknown");
          __e8.appendChild(__t12);
          __frag2.appendChild(__e8);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("span");
          __frag13.appendChild(__e15);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b6 = (anchor) => bindFor3({
      anchor,
      items: () => $scope.refs(),
      key: (r, __i) => {
        const __child = Object.create($scope);
        __child.r = r;
        __child.__i = __i;
        return (($scope2) => r.op + (r.soft ? "-soft" : ""))(__child);
      },
      render: (r, __i) => {
        const __childScope32 = Object.create($scope);
        __childScope32.r = r;
        __childScope32.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b13 = (anchor2) => bindIf3({
            anchor: anchor2,
            test: () => {
              if ($scope2.r.args.length > 0)
                return 0;
              return 1;
            },
            branches: [
              () => {
                const __frag48 = document.createDocumentFragment();
                const __c49 = [];
                const __e50 = document.createElement("div");
                const __t51 = document.createTextNode("args: ");
                __e50.appendChild(__t51);
                const __t52 = document.createTextNode("");
                __e50.appendChild(__t52);
                __c49.push(bindText3(__t52, () => $scope2.r.argsSummary));
                __frag48.appendChild(__e50);
                return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
                  for (const c of __c49)
                    try {
                      c();
                    } catch (e) {}
                } };
              },
              () => {
                const __frag53 = document.createDocumentFragment();
                const __c54 = [];
                const __e55 = document.createElement("div");
                const __t56 = document.createTextNode("args: ");
                __e55.appendChild(__t56);
                const __e57 = document.createElement("em");
                const __t58 = document.createTextNode("none");
                __e57.appendChild(__t58);
                __e55.appendChild(__e57);
                __frag53.appendChild(__e55);
                return { nodes: Array.from(__frag53.childNodes), cleanup: () => {
                  for (const c of __c54)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __wireBoundary_b16 = (anchor2) => bindIf3({
            anchor: anchor2,
            test: () => {
              if ($scope2.r.sensitive)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag64 = document.createDocumentFragment();
                const __c65 = [];
                const __e66 = document.createElement("div");
                const __e67 = document.createElement("strong");
                const __t68 = document.createTextNode("sensitive");
                __e67.appendChild(__t68);
                __e66.appendChild(__e67);
                __frag64.appendChild(__e66);
                return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
                  for (const c of __c65)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __wireBoundary_b12 = (anchor2) => bindIf3({
            anchor: anchor2,
            test: () => {
              if ($scope2.r.args !== null)
                return 0;
              return 1;
            },
            branches: [
              () => {
                const __frag45 = document.createDocumentFragment();
                const __c46 = [];
                const __ifAnchor47 = document.createComment("if");
                __frag45.appendChild(__ifAnchor47);
                __c46.push(__wireBoundary_b13(__ifAnchor47));
                const __e59 = document.createElement("div");
                const __t60 = document.createTextNode("returns: ");
                __e59.appendChild(__t60);
                const __e61 = document.createElement("code");
                const __t62 = document.createTextNode("");
                __e61.appendChild(__t62);
                __c46.push(bindText3(__t62, () => $scope2.r.returns));
                __e59.appendChild(__e61);
                __frag45.appendChild(__e59);
                const __ifAnchor63 = document.createComment("if");
                __frag45.appendChild(__ifAnchor63);
                __c46.push(__wireBoundary_b16(__ifAnchor63));
                return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
                  for (const c of __c46)
                    try {
                      c();
                    } catch (e) {}
                } };
              },
              () => {
                const __frag69 = document.createDocumentFragment();
                const __c70 = [];
                const __e71 = document.createElement("div");
                const __t72 = document.createTextNode("Not registered in ");
                __e71.appendChild(__t72);
                const __e73 = document.createElement("code");
                const __t74 = document.createTextNode("/api/ship/schema");
                __e73.appendChild(__t74);
                __e71.appendChild(__e73);
                const __t75 = document.createTextNode(".");
                __e71.appendChild(__t75);
                __frag69.appendChild(__e71);
                return { nodes: Array.from(__frag69.childNodes), cleanup: () => {
                  for (const c of __c70)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag33 = document.createDocumentFragment();
          const __c34 = [];
          const __e35 = document.createElement("details");
          const __e36 = document.createElement("summary");
          const __e37 = document.createElement("span");
          __e37.setAttribute("class", "op");
          __c34.push(bindClass2(__e37, "soft", () => $scope2.r.soft));
          __c34.push(bindClass2(__e37, "unknown", () => !$scope2.r.known));
          const __t38 = document.createTextNode("");
          __e37.appendChild(__t38);
          __c34.push(bindText3(__t38, () => $scope2.r.op));
          __e36.appendChild(__e37);
          const __e39 = document.createElement("span");
          __e39.setAttribute("class", "desc");
          const __t40 = document.createTextNode("");
          __e39.appendChild(__t40);
          __c34.push(bindText3(__t40, () => $scope2.r.description));
          __e36.appendChild(__e39);
          const __e41 = document.createElement("span");
          __e41.setAttribute("class", "perm");
          const __t42 = document.createTextNode("");
          __e41.appendChild(__t42);
          __c34.push(bindText3(__t42, () => $scope2.r.permission));
          __e36.appendChild(__e41);
          __e35.appendChild(__e36);
          const __e43 = document.createElement("div");
          __e43.setAttribute("class", "detail");
          const __ifAnchor44 = document.createComment("if");
          __e43.appendChild(__ifAnchor44);
          __c34.push(__wireBoundary_b12(__ifAnchor44));
          __e35.appendChild(__e43);
          __frag33.appendChild(__e35);
          return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
            for (const c of __c34)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope32);
      }
    });
    const __wireBoundary_b4 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.errorMsg())
          return 0;
        if ($scope.refs().length === 0)
          return 1;
        return 2;
      },
      branches: [
        () => {
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("div");
          __e20.setAttribute("style", "padding:20px;color:var(--muted-foreground)");
          const __t21 = document.createTextNode("");
          __e20.appendChild(__t21);
          __c19.push(bindText3(__t21, () => $scope.errorMsg()));
          __frag18.appendChild(__e20);
          return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
            for (const c of __c19)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag22 = document.createDocumentFragment();
          const __c23 = [];
          const __e24 = document.createElement("sui-empty-state");
          const __e25 = document.createElement("sui-icon");
          __e25.setAttribute("slot", "icon");
          __e25.setAttribute("name", "network");
          __e24.appendChild(__e25);
          const __e26 = document.createElement("span");
          __e26.setAttribute("slot", "title");
          const __t27 = document.createTextNode("No schema references");
          __e26.appendChild(__t27);
          __e24.appendChild(__e26);
          const __t28 = document.createTextNode(`
      No ship.* calls detected in this view.
    `);
          __e24.appendChild(__t28);
          __frag22.appendChild(__e24);
          return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
            for (const c of __c23)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag29 = document.createDocumentFragment();
          const __c30 = [];
          const __forAnchor31 = document.createComment("for");
          __frag29.appendChild(__forAnchor31);
          __c30.push(__wireBoundary_b6(__forAnchor31));
          return { nodes: Array.from(__frag29.childNodes), cleanup: () => {
            for (const c of __c30)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "toolbar");
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b0(__ifAnchor1));
    __root.appendChild(__e0);
    const __e16 = document.createElement("div");
    __e16.setAttribute("class", "body");
    const __ifAnchor17 = document.createComment("if");
    __e16.appendChild(__ifAnchor17);
    __cleanups.push(__wireBoundary_b4(__ifAnchor17));
    __root.appendChild(__e16);
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

// packages/ui-components/src/islands/sui-devtools/sui-devtools-signals.sui
import { adoptSuiSheets as adoptSuiSheets4, bindAttr as bindAttr2, bindFor as bindFor4, bindIf as bindIf4, bindText as bindText4, bindValue as bindValue2, component as component4 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-devtools/sui-devtools-signals.controller.ts
import { controller as controller4, signal as signal4, derived as derived4 } from "@ship/sui-runtime";
function collectSignals() {
  const ds = document._ds;
  if (ds?.signals) {
    const out2 = {};
    for (const [k, v] of Object.entries(ds.signals))
      out2[k] = v?.value ?? v;
    return out2;
  }
  const alt = window.ds ?? globalThis.__ds;
  if (alt?.signals) {
    const out2 = {};
    for (const [k, v] of Object.entries(alt.signals))
      out2[k] = v?.value ?? v;
    return out2;
  }
  const out = {};
  document.querySelectorAll("[data-signals]").forEach((el) => {
    const raw = el.getAttribute("data-signals");
    if (!raw)
      return;
    try {
      Object.assign(out, new Function(`return (${raw})`)());
    } catch {}
  });
  const view = document.getElementById("ship-view-root");
  if (view) {
    const walk = (node) => {
      for (const attr of Array.from(node.attributes)) {
        if (attr.name.startsWith("data-signals:")) {
          const name = attr.name.slice("data-signals:".length);
          try {
            out[name] = new Function(`return (${attr.value})`)();
          } catch {
            out[name] = attr.value;
          }
        }
      }
      for (const child of Array.from(node.children))
        walk(child);
    };
    walk(view);
  }
  return out;
}
function renderValue(v) {
  if (typeof v === "string")
    return v.length > 80 ? `"${v.slice(0, 77)}…"` : `"${v}"`;
  try {
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
}
var sui_devtools_signals_controller_default = controller4({
  scope: () => {
    const sigs = signal4({});
    const filter = signal4("");
    const allCount = derived4(() => Object.keys(sigs()).length);
    const shownEntries = derived4(() => {
      const s = sigs();
      const needle = filter().toLowerCase();
      const keys = Object.keys(s).sort();
      const filtered = needle ? keys.filter((k) => k.toLowerCase().includes(needle)) : keys;
      return filtered.map((k) => ({ key: k, value: renderValue(s[k]) }));
    });
    return { sigs, filter, allCount, shownEntries };
  },
  mount(state, host) {
    const s = state;
    let timer = null;
    function _sample() {
      s["sigs"] = collectSignals();
    }
    function _start() {
      if (timer)
        return;
      _sample();
      timer = setInterval(_sample, 500);
    }
    function _stop() {
      if (!timer)
        return;
      clearInterval(timer);
      timer = null;
    }
    const onDockActivate = (ev) => {
      const tab = ev.detail?.tab;
      if (tab === "signals")
        _start();
      else
        _stop();
    };
    const dock = host.closest("sui-panel-dock");
    dock?.addEventListener("sui:dock:activate", onDockActivate);
    if (dock?.getAttribute("active-tab") === "signals")
      _start();
    Object.defineProperty(host, "refresh", { value: () => _sample(), configurable: true });
    return () => {
      _stop();
      dock?.removeEventListener("sui:dock:activate", onDockActivate);
    };
  }
});

// packages/ui-components/src/islands/sui-devtools/sui-devtools-signals.sui
var sui_devtools_signals_default = component4("sui-devtools-signals", {
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: var(--text-xs);
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: 4px var(--space-3, 12px);
    border-bottom: 1px solid var(--border);
    color: var(--muted-foreground);
    background: var(--sidebar);
    flex-shrink: 0;
  }
  .toolbar input {
    appearance: none;
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    padding: 2px 8px;
    font: inherit;
    flex: 1;
    max-width: 320px;
  }
  .body {
    flex: 1;
    overflow: auto;
    font-family: var(--font-mono);
  }
  .row {
    display: flex;
    gap: 12px;
    padding: 4px var(--space-3, 12px);
    border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
    align-items: baseline;
  }
  .row:last-child { border-bottom: none; }
  .k { color: var(--primary); white-space: nowrap; flex-shrink: 0; font-weight: var(--font-weight-medium, 500); }
  .v { color: var(--muted-foreground); overflow-wrap: anywhere; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_devtools_signals_controller_default && typeof sui_devtools_signals_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_devtools_signals_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b7 = (anchor) => bindFor4({
      anchor,
      items: () => $scope.shownEntries(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.key)(__child);
      },
      render: (entry, __i) => {
        const __childScope18 = Object.create($scope);
        __childScope18.entry = entry;
        __childScope18.__i = __i;
        return (($scope2) => {
          const __frag19 = document.createDocumentFragment();
          const __c20 = [];
          const __e21 = document.createElement("div");
          __e21.setAttribute("class", "row");
          const __e22 = document.createElement("span");
          __e22.setAttribute("class", "k");
          const __t23 = document.createTextNode("");
          __e22.appendChild(__t23);
          __c20.push(bindText4(__t23, () => $scope2.entry.key));
          __e21.appendChild(__e22);
          const __e24 = document.createElement("span");
          __e24.setAttribute("class", "v");
          const __t25 = document.createTextNode("");
          __e24.appendChild(__t25);
          __c20.push(bindText4(__t25, () => $scope2.entry.value));
          __e21.appendChild(__e24);
          __frag19.appendChild(__e21);
          return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
            for (const c of __c20)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope18);
      }
    });
    const __wireBoundary_b3 = (anchor) => bindIf4({
      anchor,
      test: () => {
        if ($scope.shownEntries().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag8 = document.createDocumentFragment();
          const __c9 = [];
          const __e10 = document.createElement("sui-empty-state");
          const __e11 = document.createElement("sui-icon");
          __e11.setAttribute("slot", "icon");
          __c9.push(bindAttr2(__e11, "name", () => $scope.filter() ? "search" : "radio"));
          __e10.appendChild(__e11);
          const __e12 = document.createElement("span");
          __e12.setAttribute("slot", "title");
          const __t13 = document.createTextNode("");
          __e12.appendChild(__t13);
          __c9.push(bindText4(__t13, () => $scope.filter() ? "No matches" : "No signals"));
          __e10.appendChild(__e12);
          const __t14 = document.createTextNode("");
          __e10.appendChild(__t14);
          __c9.push(bindText4(__t14, () => $scope.filter() ? "No signals match the current filter." : "Live events will appear here when actors emit."));
          __frag8.appendChild(__e10);
          return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
            for (const c of __c9)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag15 = document.createDocumentFragment();
          const __c16 = [];
          const __forAnchor17 = document.createComment("for");
          __frag15.appendChild(__forAnchor17);
          __c16.push(__wireBoundary_b7(__forAnchor17));
          return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
            for (const c of __c16)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets4(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "toolbar");
    const __e1 = document.createElement("input");
    __e1.setAttribute("type", "text");
    __e1.setAttribute("placeholder", "Filter signals…");
    __cleanups.push(bindValue2(__e1, $scope.filter));
    __e0.appendChild(__e1);
    const __e2 = document.createElement("span");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText4(__t3, () => $scope.shownEntries().length));
    const __t4 = document.createTextNode(" / ");
    __e2.appendChild(__t4);
    const __t5 = document.createTextNode("");
    __e2.appendChild(__t5);
    __cleanups.push(bindText4(__t5, () => $scope.allCount()));
    __e0.appendChild(__e2);
    __root.appendChild(__e0);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "body");
    const __ifAnchor7 = document.createComment("if");
    __e6.appendChild(__ifAnchor7);
    __cleanups.push(__wireBoundary_b3(__ifAnchor7));
    __root.appendChild(__e6);
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

// packages/ui-components/src/islands/sui-devtools/sui-devtools-source.sui
import { adoptSuiSheets as adoptSuiSheets5, attachRef as attachRef2, bindAttr as bindAttr3, bindFor as bindFor5, bindIf as bindIf5, bindText as bindText5, component as component5, effect as effect2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-devtools/sui-devtools-source.controller.ts
import { controller as controller5, signal as signal5, derived as derived5, effect, ref as ref2 } from "@ship/sui-runtime";
function manifestLang(name) {
  if (!name)
    return "";
  if (name.endsWith(".yaml") || name.endsWith(".yml"))
    return "yaml";
  if (name.endsWith(".jsonc"))
    return "jsonc";
  return "json";
}
var sui_devtools_source_controller_default = controller5({
  scope: () => {
    const src = signal5(null);
    const draft = signal5(null);
    const saving = signal5(false);
    const diagnostics = signal5([]);
    const diagMsg = signal5(null);
    const loading = signal5(false);
    const loadError = signal5(null);
    const isDirty = derived5(() => draft() !== null && draft() !== src()?.template);
    const showDiag = derived5(() => diagnostics().length > 0 || diagMsg() !== null);
    const editorValue = derived5(() => draft() ?? src()?.template ?? "");
    const editorReadonly = derived5(() => src()?.scope !== "global");
    const manifestLangVal = derived5(() => manifestLang(src()?.manifest_filename ?? null));
    const templateRelPath = derived5(() => {
      const s = src();
      if (!s)
        return "index.html.j2";
      const marker = `/views/${s.view_name.split(".")[0]}/`;
      const idx = s.template_path.indexOf(marker);
      if (idx < 0)
        return "index.html.j2";
      return s.template_path.slice(idx + marker.length);
    });
    const headerEl = ref2(null);
    const templateEditorEl = ref2(null);
    const manifestEditorEl = ref2(null);
    return { src, draft, saving, diagnostics, diagMsg, loading, loadError, isDirty, showDiag, editorValue, editorReadonly, manifestLangVal, templateRelPath, headerEl, templateEditorEl, manifestEditorEl };
  },
  mount(state, host) {
    const s = state;
    const readRef = (key) => {
      const fn = state[key];
      return typeof fn === "function" ? fn() : null;
    };
    let editorWired = null;
    const stopSync = effect(() => {
      const el = readRef("templateEditorEl");
      const v = s["editorValue"];
      if (el && el.value !== v)
        el.value = v;
      if (el && el !== editorWired) {
        editorWired = el;
        el.addEventListener("ship-change", (e) => {
          const val = e.detail?.value;
          if (typeof val === "string")
            s["draft"] = val;
        });
        el.addEventListener("ship-save", (e) => {
          const val = e.detail?.value;
          if (typeof val === "string")
            s["draft"] = val;
          save();
        });
      }
    });
    const stopManifestSync = effect(() => {
      const el = readRef("manifestEditorEl");
      const srcObj = s["src"];
      if (el && srcObj?.manifest != null && el.value !== srcObj.manifest) {
        el.value = srcObj.manifest;
      }
    });
    const headerEl = readRef("headerEl");
    if (headerEl) {
      headerEl.addEventListener("click", (e) => {
        const btn = e.target.closest("[data-action]");
        if (!btn)
          return;
        if (btn.dataset.action === "save")
          save();
        else if (btn.dataset.action === "fork")
          fork();
      });
    }
    function stripPrefix(path) {
      const m = path.match(/^\/api\/v\/(.+)$/);
      return m ? m[1] : path.replace(/^\/+/, "");
    }
    async function save() {
      const srcObj = s["src"];
      const draftVal = s["draft"];
      if (!srcObj || srcObj.scope !== "global" || draftVal === null || draftVal === srcObj.template)
        return;
      s["saving"] = true;
      s["diagnostics"] = [];
      s["diagMsg"] = null;
      try {
        const viewName = srcObj.view_name.split(".")[0];
        const relPath = s["templateRelPath"];
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/template`, {
          method: "PUT",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ file: relPath, content: draftVal })
        });
        if (resp.status === 422) {
          const body = await resp.json();
          s["diagnostics"] = body.findings ?? [];
          return;
        }
        if (!resp.ok) {
          s["diagMsg"] = `save failed: ${resp.status}`;
          return;
        }
        s["src"] = { ...srcObj, template: draftVal };
        s["diagnostics"] = [];
        const editor = readRef("templateEditorEl");
        editor?.setDiagnostics?.([]);
        host.dispatchEvent(new CustomEvent("sui:notify", { detail: { tone: "success", title: "Saved" }, bubbles: true, composed: true }));
        const current = window.ship?.current();
        if (current?.path)
          window.ship.navigate(current.path, { replace: true });
      } catch (e) {
        s["diagMsg"] = `error: ${e.message}`;
      } finally {
        s["saving"] = false;
      }
    }
    async function fork() {
      const srcObj = s["src"];
      if (!srcObj || srcObj.scope !== "project")
        return;
      s["saving"] = true;
      s["diagMsg"] = null;
      try {
        const viewName = srcObj.view_name.split(".")[0];
        const resp = await fetch(`/api/v1/views/${encodeURIComponent(viewName)}/fork`, { method: "POST", credentials: "same-origin" });
        if (!resp.ok) {
          s["diagMsg"] = `fork failed: ${resp.status}`;
          return;
        }
        host.dispatchEvent(new CustomEvent("sui:notify", { detail: { tone: "success", title: `${viewName} forked — now editable` }, bubbles: true, composed: true }));
        lastPath = null;
        await load();
      } catch (e) {
        s["diagMsg"] = e.message;
      } finally {
        s["saving"] = false;
      }
    }
    let lastPath = null;
    async function load() {
      const current = window.ship?.current();
      const path = current?.path ?? location.pathname;
      const stripped = stripPrefix(path);
      if (!stripped) {
        s["src"] = null;
        s["draft"] = null;
        s["diagnostics"] = [];
        s["loadError"] = null;
        s["loading"] = false;
        return;
      }
      if (stripped === lastPath && s["src"])
        return;
      lastPath = stripped;
      s["draft"] = null;
      s["diagnostics"] = [];
      s["loading"] = true;
      s["loadError"] = null;
      try {
        const resp = await fetch(`/api/v1/view-source/${encodeURI(stripped)}`, { credentials: "same-origin" });
        if (resp.status === 404) {
          s["src"] = null;
          return;
        }
        if (!resp.ok) {
          s["loadError"] = `fetch failed: ${resp.status}`;
          return;
        }
        s["src"] = await resp.json();
      } catch (e) {
        s["loadError"] = e.message;
      } finally {
        s["loading"] = false;
      }
    }
    const onNavigate = () => {
      lastPath = null;
      load();
    };
    document.addEventListener("sui:navigate", onNavigate);
    load();
    Object.defineProperty(host, "refresh", { value: () => {
      lastPath = null;
      load();
    }, configurable: true });
    return () => {
      stopSync();
      stopManifestSync();
      document.removeEventListener("sui:navigate", onNavigate);
    };
  }
});

// packages/ui-components/src/islands/sui-devtools/sui-devtools-source.sui
var sui_devtools_source_default = component5("sui-devtools-source", {
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: var(--text-sm);
  }
  .header {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
    padding: var(--space-2, 8px) var(--space-3, 12px);
    border-bottom: 1px solid var(--border);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    flex-shrink: 0;
  }
  .split {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .pane { min-height: 0; display: flex; flex-direction: column; flex: 1; }
  .pane-head {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    padding: 4px var(--space-3, 12px);
    background: var(--sidebar);
    border-bottom: 1px solid var(--border);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    flex-shrink: 0;
  }
  .pane-head.manifest { border-top: 1px solid var(--border); }
  .pane-editor { flex: 1; min-height: 0; display: flex; flex-direction: column; }
  .pane-editor ship-code-editor { flex: 1; min-height: 0; }
  .diagnostics {
    padding: 4px var(--space-3, 12px);
    background: color-mix(in oklch, var(--destructive) 10%, transparent);
    color: var(--destructive);
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    border-top: 1px solid var(--destructive);
    max-height: 120px;
    overflow: auto;
    flex-shrink: 0;
  }
  .dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--primary);
    display: inline-block;
    flex-shrink: 0;
  }
  .edit-action {
    appearance: none;
    font: inherit;
    font-size: var(--text-xs);
    padding: 2px 10px;
    border-radius: var(--radius-sm, 4px);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    cursor: pointer;
  }
  .edit-action.primary { background: var(--primary); color: var(--primary-foreground, #fff); border-color: var(--primary); }
  .edit-action:disabled { opacity: 0.5; cursor: not-allowed; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_devtools_source_controller_default && typeof sui_devtools_source_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_devtools_source_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b4 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.isDirty())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("span");
          __e11.setAttribute("class", "dirty-dot");
          __e11.setAttribute("title", "Unsaved changes");
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b6 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.src().scope === "project")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag17 = document.createDocumentFragment();
          const __c18 = [];
          const __e19 = document.createElement("button");
          __e19.setAttribute("class", "edit-action primary");
          __e19.setAttribute("data-action", "fork");
          __c18.push(bindAttr3(__e19, "disabled", () => $scope.saving()));
          const __t20 = document.createTextNode("Fork to edit");
          __e19.appendChild(__t20);
          __frag17.appendChild(__e19);
          return { nodes: Array.from(__frag17.childNodes), cleanup: () => {
            for (const c of __c18)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b8 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.src().scope === "global")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag22 = document.createDocumentFragment();
          const __c23 = [];
          const __e24 = document.createElement("button");
          __e24.setAttribute("class", "edit-action primary");
          __e24.setAttribute("data-action", "save");
          __c23.push(bindAttr3(__e24, "disabled", () => $scope.saving() || !$scope.isDirty()));
          __e24.setAttribute("title", "Save (⌘/Ctrl+S)");
          const __t25 = document.createTextNode("");
          __e24.appendChild(__t25);
          __c23.push(bindText5(__t25, () => $scope.saving() ? "Savingâ¦" : "Save"));
          __frag22.appendChild(__e24);
          return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
            for (const c of __c23)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b1 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.src())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("span");
          __e4.setAttribute("style", "color:var(--foreground);font-weight:var(--font-weight-medium,500)");
          const __t5 = document.createTextNode("");
          __e4.appendChild(__t5);
          __c3.push(bindText5(__t5, () => $scope.src().view_name));
          __frag2.appendChild(__e4);
          const __e6 = document.createElement("span");
          __e6.setAttribute("style", "padding:1px 6px;border:1px solid var(--border);border-radius:999px;text-transform:uppercase;letter-spacing:.04em;font-size:10px");
          const __t7 = document.createTextNode("");
          __e6.appendChild(__t7);
          __c3.push(bindText5(__t7, () => $scope.src().scope));
          __frag2.appendChild(__e6);
          const __ifAnchor8 = document.createComment("if");
          __frag2.appendChild(__ifAnchor8);
          __c3.push(__wireBoundary_b4(__ifAnchor8));
          const __e12 = document.createElement("span");
          __e12.setAttribute("style", "flex:1");
          __frag2.appendChild(__e12);
          const __e13 = document.createElement("span");
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c3.push(bindText5(__t14, () => $scope.src().template_path));
          __frag2.appendChild(__e13);
          const __e15 = document.createElement("span");
          __e15.setAttribute("style", "display:inline-flex;align-items:center;gap:6px");
          const __ifAnchor16 = document.createComment("if");
          __e15.appendChild(__ifAnchor16);
          __c3.push(__wireBoundary_b6(__ifAnchor16));
          const __ifAnchor21 = document.createComment("if");
          __e15.appendChild(__ifAnchor21);
          __c3.push(__wireBoundary_b8(__ifAnchor21));
          __frag2.appendChild(__e15);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b16 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.src().manifest)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag50 = document.createDocumentFragment();
          const __c51 = [];
          const __e52 = document.createElement("div");
          __e52.setAttribute("class", "pane-head manifest");
          const __t53 = document.createTextNode("");
          __e52.appendChild(__t53);
          __c51.push(bindText5(__t53, () => $scope.src().manifest_filename));
          __frag50.appendChild(__e52);
          const __e54 = document.createElement("div");
          __e54.setAttribute("class", "pane");
          const __e55 = document.createElement("div");
          __e55.setAttribute("class", "pane-editor");
          const __e56 = document.createElement("ship-code-editor");
          __c51.push(bindAttr3(__e56, "language", () => $scope.manifestLangVal()));
          attachRef2($scope, "manifestEditorEl", __e56);
          __e56.setAttribute("readonly", "");
          __e55.appendChild(__e56);
          __e54.appendChild(__e55);
          __frag50.appendChild(__e54);
          return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
            for (const c of __c51)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b11 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        if ($scope.loadError())
          return 1;
        if ($scope.src())
          return 2;
        return 3;
      },
      branches: [
        () => {
          const __frag28 = document.createDocumentFragment();
          const __c29 = [];
          const __e30 = document.createElement("sui-empty-state");
          const __e31 = document.createElement("sui-icon");
          __e31.setAttribute("slot", "icon");
          __e31.setAttribute("name", "file-code");
          __e30.appendChild(__e31);
          const __e32 = document.createElement("span");
          __e32.setAttribute("slot", "title");
          const __t33 = document.createTextNode("Loading source…");
          __e32.appendChild(__t33);
          __e30.appendChild(__e32);
          const __t34 = document.createTextNode(`
      Fetching the view template from the daemon.
    `);
          __e30.appendChild(__t34);
          __frag28.appendChild(__e30);
          return { nodes: Array.from(__frag28.childNodes), cleanup: () => {
            for (const c of __c29)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag35 = document.createDocumentFragment();
          const __c36 = [];
          const __e37 = document.createElement("sui-empty-state");
          const __e38 = document.createElement("sui-icon");
          __e38.setAttribute("slot", "icon");
          __e38.setAttribute("name", "alert-triangle");
          __e37.appendChild(__e38);
          const __e39 = document.createElement("span");
          __e39.setAttribute("slot", "title");
          const __t40 = document.createTextNode("Source error");
          __e39.appendChild(__t40);
          __e37.appendChild(__e39);
          const __t41 = document.createTextNode("");
          __e37.appendChild(__t41);
          __c36.push(bindText5(__t41, () => $scope.loadError()));
          __frag35.appendChild(__e37);
          return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
            for (const c of __c36)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag42 = document.createDocumentFragment();
          const __c43 = [];
          const __e44 = document.createElement("div");
          __e44.setAttribute("class", "pane");
          const __e45 = document.createElement("div");
          __e45.setAttribute("class", "pane-head");
          const __t46 = document.createTextNode("");
          __e45.appendChild(__t46);
          __c43.push(bindText5(__t46, () => $scope.templateRelPath()));
          __e44.appendChild(__e45);
          const __e47 = document.createElement("div");
          __e47.setAttribute("class", "pane-editor");
          const __e48 = document.createElement("ship-code-editor");
          __e48.setAttribute("language", "html");
          attachRef2($scope, "templateEditorEl", __e48);
          __c43.push(bindAttr3(__e48, "readonly", () => $scope.editorReadonly()));
          __e47.appendChild(__e48);
          __e44.appendChild(__e47);
          __frag42.appendChild(__e44);
          const __ifAnchor49 = document.createComment("if");
          __frag42.appendChild(__ifAnchor49);
          __c43.push(__wireBoundary_b16(__ifAnchor49));
          return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
            for (const c of __c43)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag57 = document.createDocumentFragment();
          const __c58 = [];
          const __e59 = document.createElement("sui-empty-state");
          const __e60 = document.createElement("sui-icon");
          __e60.setAttribute("slot", "icon");
          __e60.setAttribute("name", "file-code");
          __e59.appendChild(__e60);
          const __e61 = document.createElement("span");
          __e61.setAttribute("slot", "title");
          const __t62 = document.createTextNode("No source available");
          __e61.appendChild(__t62);
          __e59.appendChild(__e61);
          const __t63 = document.createTextNode(`
      This path is not served by a file-routed view.
    `);
          __e59.appendChild(__t63);
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
    const __wireBoundary_b23 = (anchor) => bindFor5({
      anchor,
      items: () => $scope.diagnostics(),
      key: (d, __i) => {
        const __child = Object.create($scope);
        __child.d = d;
        __child.__i = __i;
        return (($scope2) => d.line + ":" + d.col + ":" + d.rule)(__child);
      },
      render: (d, __i) => {
        const __childScope72 = Object.create($scope);
        __childScope72.d = d;
        __childScope72.__i = __i;
        return (($scope2) => {
          const __frag73 = document.createDocumentFragment();
          const __c74 = [];
          const __e75 = document.createElement("div");
          const __t76 = document.createTextNode("");
          __e75.appendChild(__t76);
          __c74.push(bindText5(__t76, () => $scope2.d.line));
          const __t77 = document.createTextNode(":");
          __e75.appendChild(__t77);
          const __t78 = document.createTextNode("");
          __e75.appendChild(__t78);
          __c74.push(bindText5(__t78, () => $scope2.d.col));
          const __t79 = document.createTextNode(" [");
          __e75.appendChild(__t79);
          const __t80 = document.createTextNode("");
          __e75.appendChild(__t80);
          __c74.push(bindText5(__t80, () => $scope2.d.rule));
          const __t81 = document.createTextNode("] ");
          __e75.appendChild(__t81);
          const __t82 = document.createTextNode("");
          __e75.appendChild(__t82);
          __c74.push(bindText5(__t82, () => $scope2.d.message));
          __frag73.appendChild(__e75);
          return { nodes: Array.from(__frag73.childNodes), cleanup: () => {
            for (const c of __c74)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope72);
      }
    });
    const __wireBoundary_b21 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.diagMsg())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag66 = document.createDocumentFragment();
          const __c67 = [];
          const __t68 = document.createTextNode("");
          __frag66.appendChild(__t68);
          __c67.push(bindText5(__t68, () => $scope.diagMsg()));
          return { nodes: Array.from(__frag66.childNodes), cleanup: () => {
            for (const c of __c67)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag69 = document.createDocumentFragment();
          const __c70 = [];
          const __forAnchor71 = document.createComment("for");
          __frag69.appendChild(__forAnchor71);
          __c70.push(__wireBoundary_b23(__forAnchor71));
          return { nodes: Array.from(__frag69.childNodes), cleanup: () => {
            for (const c of __c70)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets5(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "header");
    attachRef2($scope, "headerEl", __e0);
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b1(__ifAnchor1));
    __root.appendChild(__e0);
    const __e26 = document.createElement("div");
    __e26.setAttribute("class", "split");
    const __ifAnchor27 = document.createComment("if");
    __e26.appendChild(__ifAnchor27);
    __cleanups.push(__wireBoundary_b11(__ifAnchor27));
    __root.appendChild(__e26);
    const __e64 = document.createElement("div");
    __e64.setAttribute("class", "diagnostics");
    __cleanups.push(effect2(() => {
      const __v = $scope.showDiag() ? "" : "none";
      if (__v == null)
        __e64.style.removeProperty("display");
      else
        __e64.style.setProperty("display", String(__v));
    }));
    const __ifAnchor65 = document.createComment("if");
    __e64.appendChild(__ifAnchor65);
    __cleanups.push(__wireBoundary_b21(__ifAnchor65));
    __root.appendChild(__e64);
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
