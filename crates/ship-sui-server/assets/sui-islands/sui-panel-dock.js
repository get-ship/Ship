// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.sui
import { adoptSuiSheets, attachRef, bindAttr, bindFor, bindIf, bindProp, bindText, component, on, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.controller.ts
import { controller, signal, ref } from "@ship/sui-runtime";
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
    const hostEl = host;
    const activeTabSig = state.activeTab;
    function activate(id) {
      if (!tabs().some((t) => t.id === id))
        return;
      if (activeTabSig() === id)
        return;
      activeTabSig.set(id);
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:activate", {
        detail: { tab: id },
        bubbles: true,
        composed: true
      }));
    }
    function onClear() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:clear", {
        detail: { tab: activeTabSig() },
        bubbles: true,
        composed: true
      }));
    }
    function onPopout() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:popout", {
        detail: { tab: activeTabSig() },
        bubbles: true,
        composed: true
      }));
    }
    function onClose() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:close", { bubbles: true, composed: true }));
    }
    const resizeHandle = ref(null);
    return { tabs, resizeHandle, activate, onClear, onPopout, onClose };
  },
  mount(state, host) {
    const scope = state;
    const tabsSig = scope.tabs;
    const activeTabSig = scope.activeTab;
    const resizingSig = scope.resizing;
    const storageKeySig = scope.storageKey;
    const activate = scope.activate;
    const resizeHandle = scope.resizeHandle();
    function readStored(suffix) {
      try {
        return localStorage.getItem(`${storageKeySig()}-${suffix}`);
      } catch {
        return null;
      }
    }
    function writeStored(suffix, val) {
      try {
        localStorage.setItem(`${storageKeySig()}-${suffix}`, val);
      } catch {}
    }
    function applyHeight(px) {
      document.documentElement.style.setProperty("--ship-dock-height", `${px}px`);
    }
    function currentHeight() {
      const v = getComputedStyle(document.documentElement).getPropertyValue("--ship-dock-height");
      const n = Number.parseInt(v, 10);
      return Number.isFinite(n) ? n : DEFAULT_HEIGHT;
    }
    const lastActive = readStored("active");
    if (lastActive && DEFAULT_TABS.some((t) => t.id === lastActive)) {
      activeTabSig.set(lastActive);
    }
    const h = Number.parseInt(readStored("height") ?? "", 10);
    applyHeight(Number.isFinite(h) ? h : DEFAULT_HEIGHT);
    const wrappedActivate = (id) => {
      const prev = activeTabSig();
      activate(id);
      if (activeTabSig() !== prev)
        writeStored("active", id);
    };
    const sr = host.shadowRoot;
    const toolbarCleanups = [];
    for (const tb of sr.querySelectorAll("[data-toolbar]")) {
      const slot = tb.querySelector("slot");
      if (!slot)
        continue;
      const sync = () => {
        const has = slot.assignedElements({ flatten: true }).length > 0;
        tb.classList.toggle("has-content", has);
      };
      slot.addEventListener("slotchange", sync);
      queueMicrotask(sync);
      toolbarCleanups.push(() => slot.removeEventListener("slotchange", sync));
    }
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
    Object.defineProperty(host, "activate", { value: wrappedActivate, configurable: true });
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
      resizeHandle.removeEventListener("pointerdown", onPointerDown);
      resizeHandle.removeEventListener("dblclick", onDblClick);
    };
  }
});

// packages/ui-components/src/islands/sui-panel-dock/sui-panel-dock.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_panel_dock_default = component("sui-panel-dock", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
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
    __e24.setAttribute("name", "refresh-ccw");
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
    __e26.setAttribute("name", "maximize-2");
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
    const __e29 = document.createElement("div");
    __e29.setAttribute("class", "toolbar");
    __e29.setAttribute("data-toolbar", "terminal");
    __cleanups.push(bindAttr(__e29, "hidden", () => $scope.activeTab() !== "terminal"));
    const __e30 = document.createElement("slot");
    __e30.setAttribute("name", "toolbar:terminal");
    __e29.appendChild(__e30);
    __root.appendChild(__e29);
    const __e31 = document.createElement("div");
    __e31.setAttribute("class", "toolbar");
    __e31.setAttribute("data-toolbar", "source");
    __cleanups.push(bindAttr(__e31, "hidden", () => $scope.activeTab() !== "source"));
    const __e32 = document.createElement("slot");
    __e32.setAttribute("name", "toolbar:source");
    __e31.appendChild(__e32);
    __root.appendChild(__e31);
    const __e33 = document.createElement("div");
    __e33.setAttribute("class", "toolbar");
    __e33.setAttribute("data-toolbar", "signals");
    __cleanups.push(bindAttr(__e33, "hidden", () => $scope.activeTab() !== "signals"));
    const __e34 = document.createElement("slot");
    __e34.setAttribute("name", "toolbar:signals");
    __e33.appendChild(__e34);
    __root.appendChild(__e33);
    const __e35 = document.createElement("div");
    __e35.setAttribute("class", "toolbar");
    __e35.setAttribute("data-toolbar", "lint");
    __cleanups.push(bindAttr(__e35, "hidden", () => $scope.activeTab() !== "lint"));
    const __e36 = document.createElement("slot");
    __e36.setAttribute("name", "toolbar:lint");
    __e35.appendChild(__e36);
    __root.appendChild(__e35);
    const __e37 = document.createElement("div");
    __e37.setAttribute("class", "toolbar");
    __e37.setAttribute("data-toolbar", "schema");
    __cleanups.push(bindAttr(__e37, "hidden", () => $scope.activeTab() !== "schema"));
    const __e38 = document.createElement("slot");
    __e38.setAttribute("name", "toolbar:schema");
    __e37.appendChild(__e38);
    __root.appendChild(__e37);
    const __e39 = document.createElement("div");
    __e39.setAttribute("class", "toolbar");
    __e39.setAttribute("data-toolbar", "components");
    __cleanups.push(bindAttr(__e39, "hidden", () => $scope.activeTab() !== "components"));
    const __e40 = document.createElement("slot");
    __e40.setAttribute("name", "toolbar:components");
    __e39.appendChild(__e40);
    __root.appendChild(__e39);
    const __e41 = document.createElement("div");
    __e41.setAttribute("class", "bodies");
    const __e42 = document.createElement("slot");
    __e42.setAttribute("name", "terminal");
    __cleanups.push(bindAttr(__e42, "data-active", () => $scope.activeTab() === "terminal"));
    __e41.appendChild(__e42);
    const __e43 = document.createElement("slot");
    __e43.setAttribute("name", "source");
    __cleanups.push(bindAttr(__e43, "data-active", () => $scope.activeTab() === "source"));
    __e41.appendChild(__e43);
    const __e44 = document.createElement("slot");
    __e44.setAttribute("name", "signals");
    __cleanups.push(bindAttr(__e44, "data-active", () => $scope.activeTab() === "signals"));
    __e41.appendChild(__e44);
    const __e45 = document.createElement("slot");
    __e45.setAttribute("name", "lint");
    __cleanups.push(bindAttr(__e45, "data-active", () => $scope.activeTab() === "lint"));
    __e41.appendChild(__e45);
    const __e46 = document.createElement("slot");
    __e46.setAttribute("name", "schema");
    __cleanups.push(bindAttr(__e46, "data-active", () => $scope.activeTab() === "schema"));
    __e41.appendChild(__e46);
    const __e47 = document.createElement("slot");
    __e47.setAttribute("name", "components");
    __cleanups.push(bindAttr(__e47, "data-active", () => $scope.activeTab() === "components"));
    __e41.appendChild(__e47);
    __root.appendChild(__e41);
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
