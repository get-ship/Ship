// packages/ui-components/src/islands/sui-nav-top/sui-nav-top.sui
import { adoptSuiSheets, bindAttr, bindFor, bindIf, bindText, component, fade, on, prop, registerTransition, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-nav-top/sui-nav-top.controller.ts
import { controller, derived, signal } from "@ship/sui-runtime";
function humanize(id) {
  return id.replace(/-/g, " ").split(" ").map((w) => w ? w[0].toUpperCase() + w.slice(1) : w).join(" ");
}
async function fetchViewLabels() {
  try {
    const resp = await fetch("/api/v1/views", { credentials: "same-origin" });
    if (!resp.ok)
      return new Map;
    const body = await resp.json();
    const map = new Map;
    for (const v of body.data?.views ?? []) {
      if (v.rail?.label)
        map.set(v.name, v.rail.label);
    }
    return map;
  } catch {
    return new Map;
  }
}
function navigateDoc(to) {
  history.pushState({}, "", to);
  document.dispatchEvent(new CustomEvent("sui:navigate", { detail: { to } }));
}
var sui_nav_top_controller_default = controller({
  scope: (state, host) => {
    const path = state.path;
    const projectProp = state.project;
    const viewProp = state.view;
    const labels = signal(new Map);
    const dockOpen = signal(false);
    const crumbs = derived(() => {
      const labelsMap = labels();
      const project = projectProp().trim();
      const view = viewProp().trim();
      const segs = view ? [view] : path().split("/").filter(Boolean);
      if (segs.length === 0)
        segs.push("dashboard");
      const out = [];
      if (project) {
        out.push({ href: "/", label: project, isLast: false });
      }
      segs.forEach((seg, i) => {
        const isLast = i === segs.length - 1;
        const href = "/" + segs.slice(0, i + 1).join("/");
        const label = i === 0 ? labelsMap.get(seg) ?? humanize(seg) : seg;
        out.push({ href, label, isLast });
      });
      return out;
    });
    const hostEl = host;
    function onCrumbClick(e, href) {
      e.preventDefault();
      navigateDoc(href);
    }
    function onBrand(e) {
      e.preventDefault();
      navigateDoc("/");
    }
    function onRailToggle() {
      hostEl?.dispatchEvent(new CustomEvent("sui:rail:toggle", { bubbles: true, composed: true }));
    }
    function onDockToggle() {
      hostEl?.dispatchEvent(new CustomEvent("sui:dock:toggle", { bubbles: true, composed: true }));
    }
    function onFullscreen() {
      if (document.fullscreenElement)
        document.exitFullscreen();
      else
        document.documentElement.requestFullscreen();
    }
    return {
      labels,
      dockOpen,
      crumbs,
      onCrumbClick,
      onBrand,
      onRailToggle,
      onDockToggle,
      onFullscreen
    };
  },
  mount(state, host) {
    const scope = state;
    const pathSig = scope.path;
    const labelsSig = scope.labels;
    const dockOpenSig = scope.dockOpen;
    if (!pathSig())
      pathSig.set(location.pathname);
    dockOpenSig.set(document.body.classList.contains("dock-open"));
    fetchViewLabels().then((map) => labelsSig.set(map));
    const onNavigate = (e) => {
      const to = e.detail?.to;
      if (typeof to === "string")
        pathSig.set(to);
    };
    document.addEventListener("sui:navigate", onNavigate);
    const dockObserver = new MutationObserver(() => {
      dockOpenSig.set(document.body.classList.contains("dock-open"));
    });
    dockObserver.observe(document.body, {
      attributes: true,
      attributeFilter: ["class"]
    });
    host.dispatchEvent(new CustomEvent("sui:nav-top:ready", { bubbles: true, composed: true }));
    return () => {
      document.removeEventListener("sui:navigate", onNavigate);
      dockObserver.disconnect();
    };
  }
});

// packages/ui-components/src/islands/sui-nav-top/sui-nav-top.sui
var sui_nav_top_default = component("sui-nav-top", {
  props: {
    path: prop.string({ default: "" }),
    project: prop.string({ default: "" }),
    view: prop.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: center;
    gap: var(--space-2, 8px);
    height: var(--ship-top-bar-h, 40px);
    padding: 0 var(--space-3, 12px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    background: var(--muted, #f6f6f6);
    font-family: var(--font-body, system-ui, sans-serif);
    font-size: var(--text-sm, 13px);
    color: var(--foreground, #111);
    user-select: none;
    -webkit-user-select: none;
    overflow: hidden;
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .brand {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm, 4px);
    color: var(--foreground, #111);
    text-decoration: none;
    flex-shrink: 0;
    transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .brand-mark {
    width: 18px;
    height: 18px;
    display: block;
  }
  .brand:hover { background: color-mix(in oklch, var(--foreground) 8%, transparent); }
  .brand:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .sep {
    width: 1px;
    height: 16px;
    background: var(--border, #e5e5e5);
    flex-shrink: 0;
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    flex: 0 0 auto;
    max-width: 300px;
    font-size: var(--text-sm, 13px);
  }
  .crumbs a {
    color: var(--muted-foreground, #888);
    text-decoration: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 120px;
    display: block;
  }
  .crumbs a:hover { color: var(--foreground, #111); text-decoration: underline; }
  .crumbs .chevron { color: var(--muted-foreground, #888); flex-shrink: 0; }
  .crumbs .current {
    color: var(--foreground, #111);
    font-weight: var(--font-weight-medium, 500);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
    display: block;
  }

  .view-actions {
    display: inline-flex;
    gap: 4px;
    align-items: center;
    flex-shrink: 0;
  }

  .spacer { flex: 1; min-width: 0; }

  .cmd-slot {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.path = props.path;
    $scope.project = props.project;
    $scope.view = props.view;
    let __r = null;
    if (sui_nav_top_controller_default && typeof sui_nav_top_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_nav_top_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor({
      anchor,
      items: () => $scope.crumbs(),
      key: (c, __i) => {
        const __child = Object.create($scope);
        __child.c = c;
        __child.__i = __i;
        return (($scope2) => c.href)(__child);
      },
      render: (c, __i) => {
        const __childScope5 = Object.create($scope);
        __childScope5.c = c;
        __childScope5.__i = __i;
        __childScope5.__v = signal2(0);
        return (($scope2) => {
          const __wireBoundary_b2 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.c.isLast)
                return 0;
              return 1;
            },
            branches: [
              () => {
                const __frag9 = document.createDocumentFragment();
                const __c10 = [];
                const __e11 = document.createElement("span");
                __e11.setAttribute("class", "current");
                __e11.setAttribute("id", "breadcrumbs-title");
                registerTransition(__e11, fade, { duration: 120 }, "both");
                const __t12 = document.createTextNode("");
                __e11.appendChild(__t12);
                __t12.textContent = $scope2.c.label;
                __frag9.appendChild(__e11);
                return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
                  for (const c2 of __c10)
                    try {
                      c2();
                    } catch (e) {}
                } };
              },
              () => {
                const __frag13 = document.createDocumentFragment();
                const __c14 = [];
                const __e15 = document.createElement("a");
                {
                  const __av16 = $scope2.c.href;
                  if (__av16 == null || __av16 === false || __av16 === "")
                    __e15.removeAttribute("href");
                  else
                    __e15.setAttribute("href", __av16 === true ? "" : String(__av16));
                }
                __c14.push(on(__e15, "click", function() {
                  const __h = (e) => $scope2.onCrumbClick(e, $scope2.c.href);
                  return typeof __h === "function" ? __h.bind(undefined) : () => {};
                }(), { modifiers: [] }));
                registerTransition(__e15, fade, { duration: 120 }, "both");
                const __t17 = document.createTextNode("");
                __e15.appendChild(__t17);
                __t17.textContent = $scope2.c.label;
                __frag13.appendChild(__e15);
                const __e18 = document.createElement("span");
                __e18.setAttribute("class", "chevron");
                __e18.setAttribute("aria-hidden", "true");
                const __t19 = document.createTextNode("›");
                __e18.appendChild(__t19);
                __frag13.appendChild(__e18);
                return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
                  for (const c2 of __c14)
                    try {
                      c2();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __ifAnchor8 = document.createComment("if");
          __frag6.appendChild(__ifAnchor8);
          __c7.push(__wireBoundary_b2(__ifAnchor8));
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c2 of __c7)
              try {
                c2();
              } catch (e) {}
          }, update: (__new20, __newI21) => {
            $scope2.c = __new20;
            $scope2.__i = __newI21;
            $scope2.__v.set((c2) => c2 + 1);
          } };
        })(__childScope5);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("a");
    __e0.setAttribute("class", "brand");
    __e0.setAttribute("part", "brand");
    __e0.setAttribute("href", "/");
    __e0.setAttribute("aria-label", "Ship home");
    __e0.setAttribute("title", "Ship home");
    __cleanups.push(on(__e0, "click", function() {
      const __h = $scope.onBrand;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElement("img");
    __e1.setAttribute("class", "brand-mark");
    __e1.setAttribute("src", "/ship-logo.svg");
    __e1.setAttribute("alt", "");
    __e1.setAttribute("aria-hidden", "true");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "sep");
    __e2.setAttribute("aria-hidden", "true");
    __root.appendChild(__e2);
    const __e3 = document.createElement("nav");
    __e3.setAttribute("class", "crumbs");
    __e3.setAttribute("part", "crumbs");
    __e3.setAttribute("aria-label", "Breadcrumbs");
    const __forAnchor4 = document.createComment("for");
    __e3.appendChild(__forAnchor4);
    __cleanups.push(__wireBoundary_b1(__forAnchor4));
    __root.appendChild(__e3);
    const __e22 = document.createElement("span");
    __e22.setAttribute("class", "view-actions");
    const __e23 = document.createElement("slot");
    __e23.setAttribute("name", "actions");
    __e22.appendChild(__e23);
    __root.appendChild(__e22);
    const __e24 = document.createElement("span");
    __e24.setAttribute("class", "spacer");
    __root.appendChild(__e24);
    const __e25 = document.createElement("span");
    __e25.setAttribute("class", "cmd-slot");
    const __e26 = document.createElement("slot");
    __e26.setAttribute("name", "command-bar");
    __e25.appendChild(__e26);
    __root.appendChild(__e25);
    const __e27 = document.createElement("span");
    __e27.setAttribute("class", "spacer");
    __root.appendChild(__e27);
    const __e28 = document.createElement("sui-button");
    __e28.setAttribute("variant", "ghost");
    __e28.setAttribute("size", "sm");
    __e28.setAttribute("part", "rail-toggle");
    __e28.setAttribute("title", "Toggle navigation (⌘B)");
    __e28.setAttribute("aria-label", "Toggle navigation");
    __cleanups.push(on(__e28, "click", function() {
      const __h = $scope.onRailToggle;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e29 = document.createElement("sui-icon");
    __e29.setAttribute("slot", "leading");
    __e29.setAttribute("name", "menu");
    __e28.appendChild(__e29);
    __root.appendChild(__e28);
    const __e30 = document.createElement("sui-theme-picker");
    __e30.setAttribute("placement", "bottom-end");
    __e30.setAttribute("part", "theme-picker");
    __root.appendChild(__e30);
    const __e31 = document.createElement("sui-button");
    __e31.setAttribute("variant", "ghost");
    __e31.setAttribute("size", "sm");
    __e31.setAttribute("part", "dock-toggle");
    __e31.setAttribute("title", "Toggle CLI / debug panel (⌘J)");
    __e31.setAttribute("aria-label", "Toggle CLI / debug panel");
    __cleanups.push(bindAttr(__e31, "aria-pressed", () => $scope.dockOpen() ? "true" : "false"));
    __cleanups.push(on(__e31, "click", function() {
      const __h = $scope.onDockToggle;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e32 = document.createElement("sui-icon");
    __e32.setAttribute("slot", "leading");
    __e32.setAttribute("name", "terminal");
    __e31.appendChild(__e32);
    __root.appendChild(__e31);
    const __e33 = document.createElement("sui-button");
    __e33.setAttribute("variant", "ghost");
    __e33.setAttribute("size", "sm");
    __e33.setAttribute("part", "fullscreen");
    __e33.setAttribute("title", "Fullscreen (F11)");
    __e33.setAttribute("aria-label", "Toggle fullscreen");
    __cleanups.push(on(__e33, "click", function() {
      const __h = $scope.onFullscreen;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e34 = document.createElement("sui-icon");
    __e34.setAttribute("slot", "leading");
    __e34.setAttribute("name", "maximize");
    __e33.appendChild(__e34);
    __root.appendChild(__e33);
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
