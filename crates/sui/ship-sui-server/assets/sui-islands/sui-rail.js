// packages/ui-components/src/islands/sui-rail/sui-rail-item.sui
import { adoptSuiSheets, attachRef, bindAttr, bindText, component, prop, signal } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_rail_item_default = component("sui-rail-item", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    href: prop.string({ default: "" }),
    label: prop.string({ default: "" }),
    icon: prop.string({ default: "" }),
    active: prop.bool({ default: false, reflect: true }),
    badge: prop.string({ default: "" }),
    disabled: prop.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    padding: 0 var(--space-2, 8px);
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: var(--rail-item-justify, flex-start);
    gap: var(--space-2, 8px);
    width: 100%;
    padding: 6px var(--space-2, 8px);
    border-radius: var(--radius-sm, 4px);
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    font-family: var(--font-body);
    font-size: var(--text-sm, 13px);
    font-weight: var(--font-weight-medium, 500);
    text-decoration: none;
    cursor: pointer;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
    white-space: nowrap;
    overflow: hidden;
    position: relative;
    box-sizing: border-box;
  }
  .item:hover {
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
    color: var(--foreground);
  }
  :host([active]) .item {
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    color: var(--primary);
  }
  :host([active]) .item:hover {
    background: color-mix(in oklch, var(--primary) 16%, transparent);
  }
  :host([disabled]) .item {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
  }
  .item:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .icon-wrap {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 18px;
    height: 18px;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: var(--rail-label-display, flex);
    align-items: center;
  }

  .badge {
    display: none;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 999px;
    background: var(--primary);
    color: var(--primary-foreground, white);
    font-size: 10px;
    font-weight: 600;
    line-height: 1;
    flex-shrink: 0;
  }
  .badge:not(:empty) { display: var(--rail-label-display, inline-flex); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.href = props.href;
    $scope.label = props.label;
    $scope.icon = props.icon;
    $scope.active = props.active;
    $scope.badge = props.badge;
    $scope.disabled = props.disabled;
    $scope["badgeEl"] = signal(null);
    $scope["itemEl"] = signal(null);
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("a");
    __e0.setAttribute("class", "item");
    __e0.setAttribute("part", "item");
    attachRef($scope, "itemEl", __e0);
    __cleanups.push(bindAttr(__e0, "href", () => $scope.href() || "#"));
    __cleanups.push(bindAttr(__e0, "aria-current", () => $scope.active() ? "page" : null));
    __cleanups.push(bindAttr(__e0, "aria-disabled", () => $scope.disabled() ? "true" : null));
    __cleanups.push(bindAttr(__e0, "title", () => $scope.label()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "icon-wrap");
    __e1.setAttribute("part", "icon-wrap");
    const __e2 = document.createElement("sui-icon");
    __cleanups.push(bindAttr(__e2, "name", () => $scope.icon()));
    __e2.setAttribute("size", "sm");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "label");
    __e3.setAttribute("part", "label");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText(__t4, () => $scope.label()));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "badge");
    __e5.setAttribute("part", "badge");
    attachRef($scope, "badgeEl", __e5);
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText(__t6, () => $scope.badge()));
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-rail/sui-rail-section.sui
import { adoptSuiSheets as adoptSuiSheets2, bindText as bindText2, component as component2, prop as prop2 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_rail_section_default = component2("sui-rail-section", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    label: prop2.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    margin-top: var(--space-2, 8px);
  }
  :host(:first-child) { margin-top: 0; }

  .heading {
    display: var(--rail-label-display, flex);
    padding: var(--space-1, 4px) var(--space-4, 16px);
    font-family: var(--font-body);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    opacity: 0.65;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "heading");
    __e0.setAttribute("part", "heading");
    const __t1 = document.createTextNode("");
    __e0.appendChild(__t1);
    __cleanups.push(bindText2(__t1, () => $scope.label()));
    __root.appendChild(__e0);
    const __e2 = document.createElement("slot");
    __root.appendChild(__e2);
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-rail/sui-rail.sui
import { adoptSuiSheets as adoptSuiSheets3, attachRef as attachRef2, bindAttr as bindAttr2, bindClass, bindFor, bindIf, bindText as bindText3, component as component3, fade, prop as prop3, registerTransition, signal as signal3 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-rail/sui-rail.controller.ts
import { controller, effect as effect2, ref, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-rail/sui-rail-project-switcher.ts
import { chrome, effect } from "@ship/sui-runtime";
var POPOVER_STYLES = `
.sui-rail-project-popover {
  position: fixed;
  z-index: var(--z-popover, 1000);
  min-width: 220px;
  background: var(--popover, var(--card, #fff));
  color: var(--popover-foreground, var(--card-foreground, #111));
  border: 1px solid var(--border, #e5e5e5);
  border-radius: var(--radius-md, 8px);
  box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.15));
  overflow: hidden;
  font-family: var(--font-body, system-ui);
  font-size: var(--text-base, 14px);
}
.sui-rail-project-popover-title {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted-foreground);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.sui-rail-project-popover-row {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
}
.sui-rail-project-popover-row:hover {
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
}
.sui-rail-project-popover-row.active {
  background: color-mix(in oklch, var(--primary) 12%, transparent);
  color: var(--primary);
}
.sui-rail-project-popover-row-name {
  font-size: 13px;
  font-weight: 500;
}
.sui-rail-project-popover-row-path {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sui-rail-project-popover-sep {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.sui-rail-project-popover-row.add {
  color: var(--primary);
  font-weight: 500;
}
.sui-rail-project-popover-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  color: var(--muted-foreground);
}
`;
var stylesInstalled = false;
function ensurePopoverStyles() {
  if (stylesInstalled)
    return;
  stylesInstalled = true;
  const el = document.createElement("style");
  el.setAttribute("data-sui-rail", "");
  el.textContent = POPOVER_STYLES;
  document.head.appendChild(el);
}
function dispatchBubble(host, type, detail) {
  host.dispatchEvent(new CustomEvent(type, {
    detail,
    bubbles: true,
    composed: true
  }));
}
function setupProjectSwitcher(ctx) {
  ensurePopoverStyles();
  const { host, open, projects, activeProjectId, popoverEl, badgeEl } = ctx;
  const closePopover = () => open.set(false);
  const loadProjects = async () => {
    try {
      const resp = await fetch("/api/v1/projects", { credentials: "same-origin" });
      if (!resp.ok)
        return;
      const body = await resp.json();
      projects.set(body.projects ?? []);
      activeProjectId.set(body.active ?? null);
    } catch {}
  };
  const switchProject = async (id) => {
    const previousActive = activeProjectId();
    activeProjectId.set(id);
    closePopover();
    try {
      const resp = await fetch(`/api/v1/projects/${encodeURIComponent(id)}/activate`, {
        method: "POST",
        credentials: "same-origin"
      });
      if (!resp.ok) {
        activeProjectId.set(previousActive);
        chrome.notify({
          tone: "destructive",
          title: `Switch failed (${resp.status})`
        });
        return;
      }
      dispatchBubble(host, "ship:project:change", { id });
    } catch {
      activeProjectId.set(previousActive);
    }
    loadProjects();
  };
  const addProjectFlow = async () => {
    const path = window.prompt("Path to project directory:");
    if (!path)
      return;
    try {
      const resp = await fetch("/api/v1/projects", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path })
      });
      if (!resp.ok) {
        const err = await resp.text().catch(() => "");
        chrome.notify({
          tone: "warning",
          title: `Add failed: ${err || resp.status}`
        });
        return;
      }
      await loadProjects();
      chrome.notify({ tone: "success", title: "Project added." });
    } catch (e) {
      chrome.notify({
        tone: "destructive",
        title: String(e.message ?? e)
      });
    }
  };
  const positionPopover = () => {
    const pop = popoverEl();
    const anchor = badgeEl();
    if (!pop || !anchor)
      return;
    const r = anchor.getBoundingClientRect();
    const popWidth = pop.offsetWidth || 220;
    const gap = 8;
    const left = Math.max(8, Math.min(r.left, window.innerWidth - popWidth - 8));
    const bottom = Math.max(8, window.innerHeight - r.top + gap);
    pop.style.left = `${left}px`;
    pop.style.bottom = `${bottom}px`;
  };
  const onPopoverClick = (e) => {
    const target = e.target;
    if (!target)
      return;
    if (target.closest("[data-rail-add]")) {
      addProjectFlow();
      return;
    }
    const row = target.closest("[data-project-id]");
    if (row?.dataset.projectId)
      switchProject(row.dataset.projectId);
  };
  const onBadgeClick = () => open.set(!open());
  const onDocClick = (e) => {
    if (!open())
      return;
    const path = typeof e.composedPath === "function" ? e.composedPath() : [];
    const pop = popoverEl();
    const badge = badgeEl();
    for (const node of path) {
      if (node === pop || node === badge)
        return;
    }
    closePopover();
  };
  const onWindowChange = () => {
    if (open())
      positionPopover();
  };
  const stopBadgeBind = effect(() => {
    const btn = badgeEl();
    if (!btn)
      return;
    btn.addEventListener("click", onBadgeClick);
    return () => btn.removeEventListener("click", onBadgeClick);
  });
  const stopPopoverEffect = effect(() => {
    const pop = popoverEl();
    if (!pop)
      return;
    requestAnimationFrame(positionPopover);
    pop.addEventListener("click", onPopoverClick);
    return () => pop.removeEventListener("click", onPopoverClick);
  });
  document.addEventListener("click", onDocClick, true);
  window.addEventListener("resize", onWindowChange);
  window.addEventListener("scroll", onWindowChange, { capture: true, passive: true });
  return {
    loadProjects,
    dispose() {
      stopBadgeBind();
      stopPopoverEffect();
      document.removeEventListener("click", onDocClick, true);
      window.removeEventListener("resize", onWindowChange);
      window.removeEventListener("scroll", onWindowChange, true);
    }
  };
}

// packages/ui-components/src/islands/sui-rail/sui-rail.controller.ts
function routeToViewName(path) {
  const seg = path.split("/").filter(Boolean)[0];
  return seg ?? "dashboard";
}
function dispatchBubble2(host, type, detail) {
  host.dispatchEvent(new CustomEvent(type, {
    detail,
    bubbles: true,
    composed: true
  }));
}
var sui_rail_controller_default = controller({
  scope: () => {
    const projectOpen = signal2(false);
    const projects = signal2([]);
    const activeProjectId = signal2(null);
    const activeView = signal2(typeof location !== "undefined" ? routeToViewName(location.pathname) : "");
    const projectInitial = () => {
      const list = projects();
      const cur = list.find((p) => p.id === activeProjectId()) ?? list[0];
      if (!cur)
        return "·";
      return (cur.label ?? cur.name).charAt(0).toUpperCase();
    };
    return {
      projectOpen,
      projects,
      activeProjectId,
      activeView,
      projectInitial,
      popoverEl: ref(),
      projectBadgeEl: ref(),
      launcherBtnEl: ref(),
      createBtnEl: ref(),
      navSlot: ref()
    };
  },
  mount(state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "navigation");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "Primary navigation");
    const switcher = setupProjectSwitcher({
      host,
      open: state.projectOpen,
      projects: state.projects,
      activeProjectId: state.activeProjectId,
      popoverEl: state.popoverEl,
      badgeEl: state.projectBadgeEl
    });
    switcher.loadProjects();
    const closePopover = () => state.projectOpen.set(false);
    const onLauncherClick = (e) => {
      dispatchBubble2(host, "ship:launcher:open", { anchor: e.currentTarget });
      closePopover();
    };
    const onCreateClick = () => {
      dispatchBubble2(host, "ship:create:open");
      closePopover();
    };
    const stopLauncherBind = effect2(() => {
      const btn = state.launcherBtnEl();
      if (!btn)
        return;
      btn.addEventListener("click", onLauncherClick);
      return () => btn.removeEventListener("click", onLauncherClick);
    });
    const stopCreateBind = effect2(() => {
      const btn = state.createBtnEl();
      if (!btn)
        return;
      btn.addEventListener("click", onCreateClick);
      return () => btn.removeEventListener("click", onCreateClick);
    });
    const recomputeActive = () => {
      state.activeView.set(routeToViewName(location.pathname));
    };
    const onNavigate = () => recomputeActive();
    document.addEventListener("sui:navigate", onNavigate);
    window.addEventListener("popstate", onNavigate);
    const stopActiveSync = effect2(() => {
      const name = state.activeView();
      const items = host.querySelectorAll("sui-rail-item[data-view]");
      for (const item of items) {
        if (item.dataset.view === name)
          item.setAttribute("active", "");
        else
          item.removeAttribute("active");
      }
    });
    return () => {
      stopLauncherBind();
      stopCreateBind();
      stopActiveSync();
      switcher.dispose();
      document.removeEventListener("sui:navigate", onNavigate);
      window.removeEventListener("popstate", onNavigate);
    };
  }
});

// packages/ui-components/src/islands/sui-rail/sui-rail.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_rail_default = component3("sui-rail", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    collapsed: prop3.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    width: var(--ship-rail-w-collapsed, 48px);
    background: var(--sidebar, var(--muted));
    border-right: 1px solid var(--border);
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    --rail-label-display: none;
    --rail-item-justify: center;
  }
  :host([collapsed]),
  .rail-footer { padding: 4px; }

  .rail-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-2, 8px) 0;
    scrollbar-width: thin;
    scrollbar-color: transparent transparent;
  }
  .rail-nav:hover {
    scrollbar-color: color-mix(in oklch, var(--foreground) 20%, transparent) transparent;
  }

  .rail-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: var(--space-2, 8px);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .project-badge {
    appearance: none;
    background: var(--primary);
    color: var(--primary-foreground, white);
    border: none;
    border-radius: 6px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .project-badge:hover {
    background: color-mix(in oklch, var(--primary) 80%, black);
  }
  .project-badge:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .footer-actions {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    width: auto;
  }

  .footer-btn {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    padding: 4px;
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .footer-btn:hover {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    border-color: var(--border);
    color: var(--foreground);
  }
  .footer-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.collapsed = props.collapsed;
    $scope["createBtnEl"] = signal3(null);
    $scope["launcherBtnEl"] = signal3(null);
    $scope["navSlot"] = signal3(null);
    $scope["popoverEl"] = signal3(null);
    $scope["projectBadgeEl"] = signal3(null);
    let __r = null;
    if (sui_rail_controller_default && typeof sui_rail_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_rail_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b7 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.projects().length === 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("div");
          __e20.setAttribute("class", "sui-rail-project-popover-empty");
          const __t21 = document.createTextNode("No projects tracked.");
          __e20.appendChild(__t21);
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
    const __wireBoundary_b8 = (anchor) => bindFor({
      anchor,
      items: () => $scope.projects(),
      key: (p, __i) => {
        const __child = Object.create($scope);
        __child.p = p;
        __child.__i = __i;
        return (($scope2) => p.id)(__child);
      },
      render: (p, __i) => {
        const __childScope23 = Object.create($scope);
        __childScope23.p = p;
        __childScope23.__i = __i;
        __childScope23.__v = signal3(0);
        return (($scope2) => {
          const __frag24 = document.createDocumentFragment();
          const __c25 = [];
          const __e26 = document.createElement("div");
          __e26.setAttribute("class", "sui-rail-project-popover-row");
          __c25.push(bindClass(__e26, "active", () => (void $scope2.__v(), $scope2.p.id === $scope2.activeProjectId())));
          {
            const __av27 = $scope2.p.id;
            if (__av27 == null || __av27 === false || __av27 === "")
              __e26.removeAttribute("data-project-id");
            else
              __e26.setAttribute("data-project-id", __av27 === true ? "" : String(__av27));
          }
          const __e28 = document.createElement("span");
          __e28.setAttribute("class", "sui-rail-project-popover-row-name");
          const __t29 = document.createTextNode("");
          __e28.appendChild(__t29);
          __t29.textContent = $scope2.p.label || $scope2.p.name;
          __e26.appendChild(__e28);
          const __e30 = document.createElement("span");
          __e30.setAttribute("class", "sui-rail-project-popover-row-path");
          const __t31 = document.createTextNode("");
          __e30.appendChild(__t31);
          __t31.textContent = $scope2.p.path;
          __e26.appendChild(__e30);
          __frag24.appendChild(__e26);
          return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
            for (const c of __c25)
              try {
                c();
              } catch (e) {}
          }, update: (__new32, __newI33) => {
            $scope2.p = __new32;
            $scope2.__i = __newI33;
            {
              const __av27 = $scope2.p.id;
              if (__av27 == null || __av27 === false || __av27 === "")
                __e26.removeAttribute("data-project-id");
              else
                __e26.setAttribute("data-project-id", __av27 === true ? "" : String(__av27));
            }
            __t29.textContent = $scope2.p.label || $scope2.p.name;
            __t31.textContent = $scope2.p.path;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope23);
      }
    });
    const __wireBoundary_b5 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.projectOpen())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "sui-rail-project-popover");
          __e14.setAttribute("part", "project-popover");
          __e14.setAttribute("role", "dialog");
          __e14.setAttribute("aria-label", "Switch project");
          attachRef2($scope, "popoverEl", __e14);
          registerTransition(__e14, fade, { duration: 150 }, "both");
          const __e15 = document.createElement("div");
          __e15.setAttribute("class", "sui-rail-project-popover-title");
          const __t16 = document.createTextNode("Projects");
          __e15.appendChild(__t16);
          __e14.appendChild(__e15);
          const __ifAnchor17 = document.createComment("if");
          __e14.appendChild(__ifAnchor17);
          __c13.push(__wireBoundary_b7(__ifAnchor17));
          const __forAnchor22 = document.createComment("for");
          __e14.appendChild(__forAnchor22);
          __c13.push(__wireBoundary_b8(__forAnchor22));
          const __e34 = document.createElement("div");
          __e34.setAttribute("class", "sui-rail-project-popover-sep");
          __e14.appendChild(__e34);
          const __e35 = document.createElement("div");
          __e35.setAttribute("class", "sui-rail-project-popover-row add");
          __e35.setAttribute("data-rail-add", "");
          const __e36 = document.createElement("span");
          const __t37 = document.createTextNode("+ Add project");
          __e36.appendChild(__t37);
          __e35.appendChild(__e36);
          __e14.appendChild(__e35);
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("class", "rail-nav");
    __e0.setAttribute("part", "nav");
    const __e1 = document.createElement("slot");
    attachRef2($scope, "navSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "rail-footer");
    __e2.setAttribute("part", "footer");
    const __e3 = document.createElement("button");
    __e3.setAttribute("type", "button");
    __e3.setAttribute("class", "project-badge");
    __e3.setAttribute("part", "project-badge");
    __e3.setAttribute("aria-label", "Switch project");
    __e3.setAttribute("aria-haspopup", "dialog");
    __e3.setAttribute("title", "Switch project");
    attachRef2($scope, "projectBadgeEl", __e3);
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText3(__t4, () => $scope.projectInitial()));
    __e2.appendChild(__e3);
    const __e5 = document.createElement("div");
    __e5.setAttribute("class", "footer-actions");
    const __e6 = document.createElement("button");
    __e6.setAttribute("type", "button");
    __e6.setAttribute("class", "footer-btn");
    __e6.setAttribute("part", "launcher-btn");
    __e6.setAttribute("title", "Open launcher");
    __e6.setAttribute("aria-label", "Open launcher");
    attachRef2($scope, "launcherBtnEl", __e6);
    const __e7 = document.createElement("sui-icon");
    __e7.setAttribute("name", "folder");
    __e7.setAttribute("size", "sm");
    __e6.appendChild(__e7);
    __e5.appendChild(__e6);
    const __e8 = document.createElement("button");
    __e8.setAttribute("type", "button");
    __e8.setAttribute("class", "footer-btn");
    __e8.setAttribute("part", "create-btn");
    __e8.setAttribute("title", "Create new");
    __e8.setAttribute("aria-label", "Create new");
    attachRef2($scope, "createBtnEl", __e8);
    const __e9 = document.createElement("sui-icon");
    __e9.setAttribute("name", "plus");
    __e9.setAttribute("size", "sm");
    __e8.appendChild(__e9);
    __e5.appendChild(__e8);
    __e2.appendChild(__e5);
    __root.appendChild(__e2);
    const __e10 = document.createElement("sui-portal");
    __e10.setAttribute("target", "body");
    const __ifAnchor11 = document.createComment("if");
    __e10.appendChild(__ifAnchor11);
    __cleanups.push(__wireBoundary_b5(__ifAnchor11));
    __root.appendChild(__e10);
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
