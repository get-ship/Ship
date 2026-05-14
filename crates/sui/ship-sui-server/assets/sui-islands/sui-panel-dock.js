// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.sui
import { adoptSuiSheets, attachRef, bindAttr, bindFor, bindIf, bindProp, bindText, component, fade, on, prop, registerTransition, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.controller.ts
import { controller, signal, ref } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-panel-dock/resize-pill-styles.ts
var RESIZE_PILL_STYLES = `
.sui-dock-resize-pill {
  position: fixed;
  left: 50%;
  bottom: calc(var(--ship-dock-height, 240px) + 12px);
  transform: translateX(-50%);
  z-index: var(--z-tooltip, 1100);
  padding: 4px 10px;
  border-radius: var(--radius-md, 8px);
  background: var(--popover, var(--card, #fff));
  color: var(--popover-foreground, var(--card-foreground, #111));
  border: 1px solid var(--border, #e5e5e5);
  box-shadow: var(--shadow-md, 0 4px 12px rgba(0, 0, 0, 0.15));
  font-family: var(--mono-font, ui-monospace, monospace);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
  user-select: none;
}
`;
var stylesInstalled = false;
function ensureResizePillStyles() {
  if (stylesInstalled)
    return;
  if (typeof document === "undefined")
    return;
  stylesInstalled = true;
  const el = document.createElement("style");
  el.setAttribute("data-sui-panel-dock", "");
  el.textContent = RESIZE_PILL_STYLES;
  document.head.appendChild(el);
}

// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.controller.ts
var SUI_TERMINAL_BUNDLE = "/_sui/islands/sui-terminal/island.js";
function ensureTerminalLoaded() {
  if (customElements.get("sui-terminal"))
    return;
  if (document.querySelector("script[data-sui-terminal-island]"))
    return;
  const s = document.createElement("script");
  s.type = "module";
  s.src = SUI_TERMINAL_BUNDLE;
  s.dataset["suiTerminalIsland"] = "1";
  document.head.appendChild(s);
}
function ensureTerminalSlotFilled(host) {
  const existing = host.querySelector(':scope > [slot="terminal"]');
  if (existing)
    return;
  const term = document.createElement("sui-terminal");
  term.setAttribute("slot", "terminal");
  term.setAttribute("fit", "");
  host.appendChild(term);
  ensureTerminalLoaded();
}
var DEFAULT_TABS = [
  { id: "terminal", label: "Terminal", icon: "terminal", live: true },
  { id: "source", label: "Source", icon: "file-code" },
  { id: "signals", label: "Signals", icon: "radio", live: true },
  { id: "lint", label: "Lint", icon: "list" },
  { id: "schema", label: "Schema", icon: "network" },
  { id: "components", label: "Components", icon: "package" }
];
var DEFAULT_HEIGHT = 240;
var MIN_HEIGHT = 120;
var sui_panel_dock_controller_default = controller({
  scope: (state, host) => {
    const tabs = signal(DEFAULT_TABS);
    const dragHeight = signal(DEFAULT_HEIGHT);
    const resizeHandle = ref(null);
    const hostEl = host;
    const s = state;
    const writeStored = (suffix, val) => {
      try {
        localStorage.setItem(`${s.storageKey()}-${suffix}`, val);
      } catch {}
    };
    function activate(id) {
      if (!tabs().some((t) => t.id === id))
        return;
      if (s.activeTab() === id)
        return;
      s.activeTab.set(id);
      writeStored("active", id);
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:activate", {
        detail: { tab: id },
        bubbles: true,
        composed: true
      }));
    }
    function onClear() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:clear", {
        detail: { tab: s.activeTab() },
        bubbles: true,
        composed: true
      }));
    }
    function onPopout() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:popout", {
        detail: { tab: s.activeTab() },
        bubbles: true,
        composed: true
      }));
    }
    function onClose() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:close", { bubbles: true, composed: true }));
    }
    return { tabs, dragHeight, resizeHandle, activate, onClear, onPopout, onClose };
  },
  mount(state, host) {
    const scope = state;
    const tabsSig = scope.tabs;
    const activeTabSig = scope.activeTab;
    const resizingSig = scope.resizing;
    const dragHeightSig = scope.dragHeight;
    const storageKey = scope.storageKey;
    const activate = scope.activate;
    const resizeHandle = scope.resizeHandle();
    ensureResizePillStyles();
    ensureTerminalSlotFilled(host);
    const readStored = (suffix) => {
      try {
        return localStorage.getItem(`${storageKey()}-${suffix}`);
      } catch {
        return null;
      }
    };
    const writeStored = (suffix, val) => {
      try {
        localStorage.setItem(`${storageKey()}-${suffix}`, val);
      } catch {}
    };
    const applyHeight = (px) => {
      document.documentElement.style.setProperty("--ship-dock-height", `${px}px`);
      dragHeightSig.set(px);
    };
    const currentHeight = () => {
      const v = getComputedStyle(document.documentElement).getPropertyValue("--ship-dock-height");
      const n = Number.parseInt(v, 10);
      return Number.isFinite(n) ? n : DEFAULT_HEIGHT;
    };
    const lastActive = readStored("active");
    if (lastActive && tabsSig().some((t) => t.id === lastActive)) {
      activeTabSig.set(lastActive);
    }
    const h = Number.parseInt(readStored("height") ?? "", 10);
    applyHeight(Number.isFinite(h) ? h : DEFAULT_HEIGHT);
    const sr = host.shadowRoot;
    const toolbarCleanups = [];
    const trackToolbar = (tb) => {
      const slot = tb.querySelector("slot");
      if (!slot)
        return;
      const sync = () => {
        const has = slot.assignedElements({ flatten: true }).length > 0;
        tb.classList.toggle("has-content", has);
      };
      slot.addEventListener("slotchange", sync);
      queueMicrotask(sync);
      toolbarCleanups.push(() => slot.removeEventListener("slotchange", sync));
    };
    for (const tb of sr.querySelectorAll("[data-toolbar]"))
      trackToolbar(tb);
    const mo = new MutationObserver((records) => {
      for (const r of records) {
        for (const n of r.addedNodes) {
          if (n.nodeType !== 1)
            continue;
          const el = n;
          if (el.matches?.("[data-toolbar]"))
            trackToolbar(el);
        }
      }
    });
    mo.observe(sr, { childList: true, subtree: true });
    const onPointerDown = (ev) => {
      ev.preventDefault();
      const startY = ev.clientY;
      const startH = currentHeight();
      resizingSig.set(true);
      resizeHandle.setPointerCapture(ev.pointerId);
      const onMove = (e) => {
        const max = Math.round(window.innerHeight * 0.5);
        applyHeight(Math.min(max, Math.max(MIN_HEIGHT, startH + (startY - e.clientY))));
      };
      const onUp = (e) => {
        resizingSig.set(false);
        resizeHandle.releasePointerCapture(e.pointerId);
        resizeHandle.removeEventListener("pointermove", onMove);
        resizeHandle.removeEventListener("pointerup", onUp);
        resizeHandle.removeEventListener("pointercancel", onUp);
        const final = currentHeight();
        writeStored("height", String(final));
        host.dispatchEvent(new CustomEvent("sui:dock:resize", {
          detail: { height: final },
          bubbles: true,
          composed: true
        }));
      };
      resizeHandle.addEventListener("pointermove", onMove);
      resizeHandle.addEventListener("pointerup", onUp);
      resizeHandle.addEventListener("pointercancel", onUp);
    };
    const onDblClick = () => {
      applyHeight(DEFAULT_HEIGHT);
      writeStored("height", String(DEFAULT_HEIGHT));
      host.dispatchEvent(new CustomEvent("sui:dock:resize", {
        detail: { height: DEFAULT_HEIGHT },
        bubbles: true,
        composed: true
      }));
    };
    resizeHandle.addEventListener("pointerdown", onPointerDown);
    resizeHandle.addEventListener("dblclick", onDblClick);
    Object.defineProperty(host, "activate", { value: activate, configurable: true });
    Object.defineProperty(host, "show", {
      value: () => document.body.classList.add("dock-open"),
      configurable: true
    });
    Object.defineProperty(host, "hide", {
      value: () => document.body.classList.remove("dock-open"),
      configurable: true
    });
    Object.defineProperty(host, "toggle", {
      value: () => document.body.classList.toggle("dock-open"),
      configurable: true
    });
    Object.defineProperty(host, "tabs", {
      get: () => tabsSig(),
      set: (val) => tabsSig.set(val),
      configurable: true
    });
    return () => {
      for (const cleanup of toolbarCleanups)
        cleanup();
      mo.disconnect();
      resizeHandle.removeEventListener("pointerdown", onPointerDown);
      resizeHandle.removeEventListener("dblclick", onDblClick);
    };
  }
});

// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.sui
var sui_panel_dock_default = component("sui-panel-dock", {
  props: {
    "active-tab": prop.string({ default: "terminal", reflect: true }),
    "storage-key": prop.string({ default: "ship-dock" }),
    resizing: prop.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    position: relative;
    background: var(--card);
    box-shadow: var(--shadow-edge-top);
  }

  .resize {
    position: absolute;
    top: -3px;
    left: 0;
    right: 0;
    height: 6px;
    cursor: ns-resize;
    z-index: 2;
    touch-action: none;
  }
  .resize::before {
    content: '';
    position: absolute;
    left: 50%;
    top: 2px;
    transform: translateX(-50%);
    width: 28px;
    height: 2px;
    border-radius: 2px;
    background: var(--muted-foreground);
    opacity: 0.45;
    transition: opacity var(--duration-fast) var(--ease-out), background var(--duration-fast) var(--ease-out);
  }
  .resize:hover::before,
  :host([resizing]) .resize::before { opacity: 0.95; background: var(--foreground); }

  .tabstrip {
    display: flex;
    align-items: stretch;
    padding-left: var(--space-2);
    border-bottom: 1px solid var(--border-subtle, var(--border));
    background: var(--card);
    font-size: var(--text-xs);
    user-select: none;
    height: 30px;
    flex: none;
  }

  .tab {
    appearance: none;
    background: transparent;
    border: 0;
    color: var(--muted-foreground);
    font: inherit;
    font-size: 11.5px;
    padding: 0 10px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    position: relative;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color var(--duration-fast) var(--ease-out), background var(--duration-fast) var(--ease-out);
  }
  .tab:hover { color: var(--foreground); }
  .tab[aria-selected="true"] {
    color: var(--foreground);
    background: color-mix(in oklab, var(--primary) 12%, transparent);
    border-bottom-color: var(--primary);
    font-weight: var(--font-weight-medium, 500);
  }
  .tab:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 2px var(--ring);
    border-radius: var(--radius-sm);
  }
  .live-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--success, #22c55e);
    box-shadow: 0 0 0 2px color-mix(in oklab, var(--success, #22c55e) 25%, transparent);
  }
  .count {
    font-family: var(--mono-font, var(--font-mono));
    font-size: 9.5px;
    background: var(--muted);
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    padding: 1px 5px;
    min-width: 18px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .tab[aria-selected="true"] .count {
    background: color-mix(in oklab, var(--primary) 14%, transparent);
    color: var(--foreground);
  }

  .tools {
    margin-left: auto;
    display: flex;
    align-items: center;
    padding-right: 4px;
  }
  .tool {
    appearance: none;
    background: transparent;
    border: 0;
    color: var(--muted-foreground);
    cursor: pointer;
    width: 24px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    transition: background var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
  }
  .tool:hover { background: var(--hover-bg, var(--sidebar-accent)); color: var(--foreground); }
  .tool:focus-visible { outline: none; box-shadow: 0 0 0 2px var(--ring); }

  .toolbar {
    display: none;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border-subtle, var(--border));
    background: color-mix(in oklab, var(--background) 50%, var(--card));
    flex: none;
    min-height: 30px;
  }
  .toolbar.has-content { display: flex; }
  .toolbar[hidden] { display: none !important; }

  .bodies { flex: 1; min-height: 0; position: relative; overflow: hidden; background: var(--background); }
  ::slotted(*) { display: none; height: 100%; overflow: auto; }
  slot[data-active]::slotted(*) { display: block; }

  @media (prefers-reduced-motion: reduce) {
    .tab, .tool, .resize::before { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.activeTab = props.activeTab;
    $scope.storageKey = props.storageKey;
    $scope.resizing = props.resizing;
    $scope["resizeHandle"] = signal2(null);
    let __r = null;
    if (sui_panel_dock_controller_default && typeof sui_panel_dock_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_panel_dock_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor({
      anchor,
      items: () => $scope.tabs(),
      key: (t, __i) => {
        const __child = Object.create($scope);
        __child.t = t;
        __child.__i = __i;
        return (($scope2) => t.id)(__child);
      },
      render: (t, __i) => {
        const __childScope3 = Object.create($scope);
        __childScope3.t = t;
        __childScope3.__i = __i;
        __childScope3.__v = signal2(0);
        return (($scope2) => {
          const __wireBoundary_b8 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.t.live && $scope2.t.id === $scope2.activeTab())
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag12 = document.createDocumentFragment();
                const __c13 = [];
                const __e14 = document.createElement("span");
                __e14.setAttribute("class", "live-dot");
                __e14.setAttribute("aria-hidden", "true");
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
          const __wireBoundary_b9 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.t.count != null)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag16 = document.createDocumentFragment();
                const __c17 = [];
                const __e18 = document.createElement("span");
                __e18.setAttribute("class", "count");
                const __t19 = document.createTextNode("");
                __e18.appendChild(__t19);
                __c17.push(bindText(__t19, () => (void $scope2.__v(), $scope2.t.count.toLocaleString())));
                __frag16.appendChild(__e18);
                return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
                  for (const c of __c17)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElement("button");
          __e6.setAttribute("class", "tab");
          __e6.setAttribute("role", "tab");
          __e6.setAttribute("type", "button");
          __c5.push(bindAttr(__e6, "aria-selected", () => (void $scope2.__v(), $scope2.t.id === $scope2.activeTab() ? "true" : "false")));
          __c5.push(bindAttr(__e6, "tabindex", () => (void $scope2.__v(), $scope2.t.id === $scope2.activeTab() ? "0" : "-1")));
          {
            const __av7 = $scope2.t.id;
            if (__av7 == null || __av7 === false || __av7 === "")
              __e6.removeAttribute("data-tab");
            else
              __e6.setAttribute("data-tab", __av7 === true ? "" : String(__av7));
          }
          __c5.push(on(__e6, "click", function() {
            const __h = () => $scope2.activate($scope2.t.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e8 = document.createElement("sui-icon");
          __e8["name"] = $scope2.t.icon;
          __e8.setAttribute("size", "sm");
          __e6.appendChild(__e8);
          const __e9 = document.createElement("span");
          const __t10 = document.createTextNode("");
          __e9.appendChild(__t10);
          __t10.textContent = $scope2.t.label;
          __e6.appendChild(__e9);
          const __ifAnchor11 = document.createComment("if");
          __e6.appendChild(__ifAnchor11);
          __c5.push(__wireBoundary_b8(__ifAnchor11));
          const __ifAnchor15 = document.createComment("if");
          __e6.appendChild(__ifAnchor15);
          __c5.push(__wireBoundary_b9(__ifAnchor15));
          __frag4.appendChild(__e6);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          }, update: (__new20, __newI21) => {
            $scope2.t = __new20;
            $scope2.__i = __newI21;
            {
              const __av7 = $scope2.t.id;
              if (__av7 == null || __av7 === false || __av7 === "")
                __e6.removeAttribute("data-tab");
              else
                __e6.setAttribute("data-tab", __av7 === true ? "" : String(__av7));
            }
            __e8["name"] = $scope2.t.icon;
            __t10.textContent = $scope2.t.label;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope3);
      }
    });
    const __wireBoundary_b14 = (anchor) => bindFor({
      anchor,
      items: () => $scope.tabs(),
      key: (t, __i) => {
        const __child = Object.create($scope);
        __child.t = t;
        __child.__i = __i;
        return (($scope2) => t.id)(__child);
      },
      render: (t, __i) => {
        const __childScope30 = Object.create($scope);
        __childScope30.t = t;
        __childScope30.__i = __i;
        __childScope30.__v = signal2(0);
        return (($scope2) => {
          const __frag31 = document.createDocumentFragment();
          const __c32 = [];
          const __e33 = document.createElement("div");
          __e33.setAttribute("class", "toolbar");
          {
            const __av34 = $scope2.t.id;
            if (__av34 == null || __av34 === false || __av34 === "")
              __e33.removeAttribute("data-toolbar");
            else
              __e33.setAttribute("data-toolbar", __av34 === true ? "" : String(__av34));
          }
          __c32.push(bindAttr(__e33, "hidden", () => (void $scope2.__v(), $scope2.t.id !== $scope2.activeTab())));
          const __e35 = document.createElement("slot");
          {
            const __av36 = "toolbar:" + $scope2.t.id;
            if (__av36 == null || __av36 === false || __av36 === "")
              __e35.removeAttribute("name");
            else
              __e35.setAttribute("name", __av36 === true ? "" : String(__av36));
          }
          __e33.appendChild(__e35);
          __frag31.appendChild(__e33);
          return { nodes: Array.from(__frag31.childNodes), cleanup: () => {
            for (const c of __c32)
              try {
                c();
              } catch (e) {}
          }, update: (__new37, __newI38) => {
            $scope2.t = __new37;
            $scope2.__i = __newI38;
            {
              const __av34 = $scope2.t.id;
              if (__av34 == null || __av34 === false || __av34 === "")
                __e33.removeAttribute("data-toolbar");
              else
                __e33.setAttribute("data-toolbar", __av34 === true ? "" : String(__av34));
            }
            {
              const __av36 = "toolbar:" + $scope2.t.id;
              if (__av36 == null || __av36 === false || __av36 === "")
                __e35.removeAttribute("name");
              else
                __e35.setAttribute("name", __av36 === true ? "" : String(__av36));
            }
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope30);
      }
    });
    const __wireBoundary_b18 = (anchor) => bindFor({
      anchor,
      items: () => $scope.tabs(),
      key: (t, __i) => {
        const __child = Object.create($scope);
        __child.t = t;
        __child.__i = __i;
        return (($scope2) => t.id)(__child);
      },
      render: (t, __i) => {
        const __childScope41 = Object.create($scope);
        __childScope41.t = t;
        __childScope41.__i = __i;
        __childScope41.__v = signal2(0);
        return (($scope2) => {
          const __frag42 = document.createDocumentFragment();
          const __c43 = [];
          const __e44 = document.createElement("slot");
          {
            const __av45 = $scope2.t.id;
            if (__av45 == null || __av45 === false || __av45 === "")
              __e44.removeAttribute("name");
            else
              __e44.setAttribute("name", __av45 === true ? "" : String(__av45));
          }
          __c43.push(bindAttr(__e44, "data-active", () => (void $scope2.__v(), $scope2.t.id === $scope2.activeTab())));
          __frag42.appendChild(__e44);
          return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
            for (const c of __c43)
              try {
                c();
              } catch (e) {}
          }, update: (__new46, __newI47) => {
            $scope2.t = __new46;
            $scope2.__i = __newI47;
            {
              const __av45 = $scope2.t.id;
              if (__av45 == null || __av45 === false || __av45 === "")
                __e44.removeAttribute("name");
              else
                __e44.setAttribute("name", __av45 === true ? "" : String(__av45));
            }
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope41);
      }
    });
    const __wireBoundary_b21 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.resizing())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag50 = document.createDocumentFragment();
          const __c51 = [];
          const __e52 = document.createElement("div");
          __e52.setAttribute("class", "sui-dock-resize-pill");
          __e52.setAttribute("part", "resize-pill");
          registerTransition(__e52, fade, { duration: 120 }, "both");
          const __t53 = document.createTextNode("");
          __e52.appendChild(__t53);
          __c51.push(bindText(__t53, () => $scope.dragHeight()));
          const __t54 = document.createTextNode(`px
    `);
          __e52.appendChild(__t54);
          __frag50.appendChild(__e52);
          return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
            for (const c of __c51)
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
    __e0.setAttribute("class", "resize");
    __e0.setAttribute("role", "separator");
    __e0.setAttribute("aria-orientation", "horizontal");
    __e0.setAttribute("aria-label", "Resize panel");
    attachRef($scope, "resizeHandle", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "tabstrip");
    __e1.setAttribute("role", "tablist");
    const __forAnchor2 = document.createComment("for");
    __e1.appendChild(__forAnchor2);
    __cleanups.push(__wireBoundary_b1(__forAnchor2));
    const __e22 = document.createElement("span");
    __e22.setAttribute("class", "tools");
    const __e23 = document.createElement("button");
    __e23.setAttribute("class", "tool");
    __e23.setAttribute("type", "button");
    __e23.setAttribute("aria-label", "Clear");
    __e23.setAttribute("title", "Clear");
    __e23.setAttribute("data-action", "clear");
    __cleanups.push(on(__e23, "click", function() {
      const __h = $scope.onClear;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e24 = document.createElement("sui-icon");
    __e24.setAttribute("name", "refresh-cw");
    __e24.setAttribute("size", "sm");
    __e23.appendChild(__e24);
    __e22.appendChild(__e23);
    const __e25 = document.createElement("button");
    __e25.setAttribute("class", "tool");
    __e25.setAttribute("type", "button");
    __e25.setAttribute("aria-label", "Pop out");
    __e25.setAttribute("title", "Pop out");
    __e25.setAttribute("data-action", "popout");
    __cleanups.push(on(__e25, "click", function() {
      const __h = $scope.onPopout;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e26 = document.createElement("sui-icon");
    __e26.setAttribute("name", "maximize");
    __e26.setAttribute("size", "sm");
    __e25.appendChild(__e26);
    __e22.appendChild(__e25);
    const __e27 = document.createElement("button");
    __e27.setAttribute("class", "tool");
    __e27.setAttribute("type", "button");
    __e27.setAttribute("aria-label", "Close panel");
    __e27.setAttribute("title", "Close (⌘J)");
    __e27.setAttribute("data-action", "close");
    __cleanups.push(on(__e27, "click", function() {
      const __h = $scope.onClose;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e28 = document.createElement("sui-icon");
    __e28.setAttribute("name", "x");
    __e28.setAttribute("size", "sm");
    __e27.appendChild(__e28);
    __e22.appendChild(__e27);
    __e1.appendChild(__e22);
    __root.appendChild(__e1);
    const __forAnchor29 = document.createComment("for");
    __root.appendChild(__forAnchor29);
    __cleanups.push(__wireBoundary_b14(__forAnchor29));
    const __e39 = document.createElement("div");
    __e39.setAttribute("class", "bodies");
    const __forAnchor40 = document.createComment("for");
    __e39.appendChild(__forAnchor40);
    __cleanups.push(__wireBoundary_b18(__forAnchor40));
    __root.appendChild(__e39);
    const __e48 = document.createElement("sui-portal");
    __e48.setAttribute("target", "body");
    const __ifAnchor49 = document.createComment("if");
    __e48.appendChild(__ifAnchor49);
    __cleanups.push(__wireBoundary_b21(__ifAnchor49));
    __root.appendChild(__e48);
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
