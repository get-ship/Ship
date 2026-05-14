// packages/ui-components/src/sui-rail/sui-rail.sui + controller
import { component, prop, signal, suiTonesSheet } from "@ship/sui-runtime";
import { controller, effect } from "@ship/sui-runtime";

// ── sui-rail controller ─────────────────────────────────────────
var sui_rail_controller_default = controller({
  scope: (state, host) => {
    const scope = state;
    const stopIcon = effect(() => {
      const btn = scope.toggleBtn();
      const col = scope.collapsed();
      if (btn) btn.setAttribute('aria-label', col ? 'Show rail' : 'Hide rail');
    });
    const onToggle = () => {
      const next = !scope.collapsed();
      scope.collapsed.set(next);
      host.dispatchEvent(
        new CustomEvent('sui-rail:toggle', {
          detail: { collapsed: next },
          bubbles: true,
          composed: true,
        })
      );
    };
    const stopBtn = effect(() => {
      const btn = scope.toggleBtn();
      if (!btn) return;
      btn.addEventListener('click', onToggle);
      return () => btn.removeEventListener('click', onToggle);
    });
    return () => {
      stopIcon();
      stopBtn();
    };
  },
});

// ── sui-rail-item compiled ──────────────────────────────────────
import { bindAttr, bindText, component as component2, prop as prop2 } from "@ship/sui-runtime";

var sui_rail_item_default = component2("sui-rail-item", {
  props: {
    "href": prop2.string({ default: "" }),
    "label": prop2.string({ default: "" }),
    "icon": prop2.string({ default: "" }),
    "active": prop2.bool({ default: false, reflect: true }),
    "badge": prop2.string({ default: "" }),
    "disabled": prop2.bool({ default: false, reflect: true }),
  },
  shadow: true,
  styles: "\n  :host { display: block; padding: 0 var(--space-2, 8px); }\n  :host([hidden]) { display: none; }\n  .item { display: flex; align-items: center; justify-content: var(--rail-item-justify, flex-start); gap: var(--space-2, 8px); width: 100%; padding: 6px var(--space-2, 8px); border-radius: var(--radius-sm, 4px); border: none; background: transparent; color: var(--muted-foreground); font-family: var(--font-body); font-size: var(--text-sm, 13px); font-weight: var(--font-weight-medium, 500); text-decoration: none; cursor: pointer; transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out), color var(--duration-fast, 100ms) var(--ease-out, ease-out); white-space: nowrap; overflow: hidden; position: relative; box-sizing: border-box; }\n  .item:hover { background: color-mix(in oklch, var(--foreground) 6%, transparent); color: var(--foreground); }\n  :host([active]) .item { background: color-mix(in oklch, var(--primary) 12%, transparent); color: var(--primary); }\n  :host([active]) .item:hover { background: color-mix(in oklch, var(--primary) 16%, transparent); }\n  :host([disabled]) .item { opacity: 0.45; cursor: not-allowed; pointer-events: none; }\n  .item:focus-visible { outline: none; box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent); }\n  .icon-wrap { display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; width: 18px; height: 18px; }\n  .label { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: var(--rail-label-display, flex); align-items: center; }\n  .badge { display: none; align-items: center; justify-content: center; min-width: 16px; height: 16px; padding: 0 4px; border-radius: 999px; background: var(--primary); color: var(--primary-foreground, white); font-size: 10px; font-weight: 600; line-height: 1; flex-shrink: 0; }\n  .badge:not(:empty) { display: var(--rail-label-display, inline-flex); }",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.href = props.href;
    $scope.label = props.label;
    $scope.icon = props.icon;
    $scope.active = props.active;
    $scope.badge = props.badge;
    $scope.disabled = props.disabled;
    const __root = host.shadowRoot ?? host;
    if (suiTonesSheet && __root instanceof ShadowRoot) { try { __root.adoptedStyleSheets = [suiTonesSheet, ...(__root.adoptedStyleSheets ?? [])]; } catch {} }
    const __e0 = document.createElement("a");
    __e0.setAttribute("class", "item");
    __e0.setAttribute("part", "item");
    $scope.itemEl = signal(__e0);
    __cleanups.push(bindAttr(__e0, "href", () => ($scope.href() || '#')));
    __cleanups.push(bindAttr(__e0, "aria-current", () => ($scope.active() ? 'page' : null)));
    __cleanups.push(bindAttr(__e0, "aria-disabled", () => ($scope.disabled() ? 'true' : null)));
    __cleanups.push(bindAttr(__e0, "title", () => ($scope.label())));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "icon-wrap");
    __e1.setAttribute("part", "icon-wrap");
    const __e2 = document.createElement("sui-icon");
    __cleanups.push(bindAttr(__e2, "name", () => ($scope.icon())));
    __e2.setAttribute("size", "sm");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "label");
    __e3.setAttribute("part", "label");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText(__t4, () => ($scope.label())));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "badge");
    __e5.setAttribute("part", "badge");
    $scope.badgeEl = signal(__e5);
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText(__t6, () => ($scope.badge())));
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    return $scope;
  },
});

// ── sui-rail-section compiled ───────────────────────────────────
var sui_rail_section_default = component2("sui-rail-section", {
  props: {
    "label": prop2.string({ default: "" }),
  },
  shadow: false,
  styles: "\n  :host { display: block; margin-top: var(--space-2, 8px); }\n  :host(:first-child) { margin-top: 0; }\n  :host([hidden]) { display: none; }\n  .heading { display: var(--rail-label-display, flex); padding: var(--space-1, 4px) var(--space-4, 16px); font-family: var(--font-body); font-size: 10px; font-weight: 600; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted-foreground); opacity: 0.65; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }",
  setup(ctx) {
    const { props, host } = ctx;
    const $scope = Object.create(null);
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "heading");
    __e0.setAttribute("part", "heading");
    const __t1 = document.createTextNode("");
    __e0.appendChild(__t1);
    if (bindText) __cleanups.push(bindText(__t1, () => ($scope.label())));
    __root.appendChild(__e0);
    return $scope;
  },
});

// ── sui-rail main component ─────────────────────────────────────
var sui_rail_default = component("sui-rail", {
  props: {
    "collapsed": prop.bool({ default: false, reflect: true }),
  },
  shadow: true,
  styles: "\n  :host { display: flex; flex-direction: column; height: 100%; min-height: 0; width: var(--ship-rail-w, 200px); background: var(--sidebar, var(--muted)); border-right: 1px solid var(--border); overflow: hidden; user-select: none; -webkit-user-select: none; --rail-label-display: flex; --rail-item-justify: flex-start; }\n  :host([collapsed]) { width: 32px; --rail-label-display: none; --rail-item-justify: center; }\n  :host([collapsed]) .rail-nav { display: none; }\n  :host([collapsed]) .rail-footer { padding: 4px; justify-content: center; }\n  :host([hidden]) { display: none; }\n  .rail-nav { flex: 1; display: flex; flex-direction: column; overflow-y: auto; overflow-x: hidden; padding: var(--space-2, 8px) 0; scrollbar-width: thin; scrollbar-color: transparent transparent; }\n  .rail-nav:hover { scrollbar-color: color-mix(in oklch, var(--foreground) 20%, transparent) transparent; }\n  .rail-footer { display: flex; align-items: center; justify-content: flex-end; padding: var(--space-2, 8px); border-top: 1px solid var(--border); flex-shrink: 0; }\n  .toggle-btn { appearance: none; background: transparent; border: 1px solid transparent; border-radius: var(--radius-sm, 4px); padding: 4px; color: var(--muted-foreground); cursor: pointer; display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out), border-color var(--duration-fast, 100ms) var(--ease-out, ease-out), color var(--duration-fast, 100ms) var(--ease-out, ease-out); }\n  .toggle-btn:hover { background: color-mix(in oklch, var(--foreground) 8%, transparent); border-color: var(--border); color: var(--foreground); }\n  .toggle-btn:focus-visible { outline: none; box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent); }\n  .toggle-btn::before { content: '\\25C0'; font-size: 10px; line-height: 1; }\n  :host([collapsed]) .toggle-btn::before { content: '\\25B6'; }\n  @media (prefers-reduced-motion: reduce) { :host { transition: none; } }",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.collapsed = props.collapsed;
    let __r = null;
    if (sui_rail_controller_default && typeof sui_rail_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_rail_controller_default.setup(ctx);
      if (__r && typeof __r === "object") Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    if (suiTonesSheet && __root instanceof ShadowRoot) { try { __root.adoptedStyleSheets = [suiTonesSheet, ...(__root.adoptedStyleSheets ?? [])]; } catch {} }
    const __e0 = document.createElement("nav");
    __e0.setAttribute("class", "rail-nav");
    __e0.setAttribute("part", "nav");
    const __e1 = document.createElement("slot");
    $scope.navSlot = signal(__e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "rail-footer");
    __e2.setAttribute("part", "footer");
    const __e3 = document.createElement("button");
    __e3.setAttribute("type", "button");
    __e3.setAttribute("class", "toggle-btn");
    __e3.setAttribute("part", "toggle");
    $scope.toggleBtn = signal(__e3);
    __e3.setAttribute("aria-label", "Collapse rail");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) { return t[k]; },
          set(t, k, v) { const s = t[k]; if (s != null && typeof s.set === "function") { if (typeof v === "function") { s.set(() => v); } else { s.set(v); } return true; } return false; }
        });
        if (__lc.mount) { const __mc = __lc.mount(__sp, host); if (typeof __mc === "function") __cleanups.push(__mc); }
        if (__lc.unmount) __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  },
});
