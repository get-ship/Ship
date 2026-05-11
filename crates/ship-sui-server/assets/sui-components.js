// packages/ui-components/src/sui-status-dot/sui-status-dot.sui
import { adoptSuiSheets, bindAttr, component, prop } from "@ship/sui-runtime";
var sui_status_dot_default = component("sui-status-dot", {
  props: {
    tone: prop.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "success", reflect: true }),
    pulse: prop.bool({ default: false, reflect: true }),
    size: prop.union(["sm", "md"], { default: "md", reflect: true }),
    label: prop.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--success);
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--tone-fg);
    box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }
  :host([size='sm']) .dot {
    width: 6px;
    height: 6px;
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }

  :host([pulse]) .dot { animation: sui-status-dot-pulse 2.5s infinite; }
  @keyframes sui-status-dot-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%      { opacity: 0.75; transform: scale(0.9); }
  }

  @media (prefers-reduced-motion: reduce) {
    :host([pulse]) .dot { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.pulse = props.pulse;
    $scope.size = props.size;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "dot");
    __e0.setAttribute("part", "dot");
    __cleanups.push(bindAttr(__e0, "role", () => $scope.label() ? "img" : null));
    __cleanups.push(bindAttr(__e0, "aria-label", () => $scope.label() || null));
    __cleanups.push(bindAttr(__e0, "aria-hidden", () => $scope.label() ? null : "true"));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-spinner/sui-spinner.sui
import { adoptSuiSheets as adoptSuiSheets2, bindAttr as bindAttr2, component as component2, prop as prop2 } from "@ship/sui-runtime";
var sui_spinner_default = component2("sui-spinner", {
  props: {
    size: prop2.union(["xs", "sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop2.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    label: prop2.string({ default: "Loading" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--muted-foreground);
    --sz: 16px;
    --bw: 2px;
  }
  :host([size='xs']) { --sz: 10px; --bw: 1.5px; }
  :host([size='sm']) { --sz: 12px; --bw: 1.5px; }
  :host([size='md']) { --sz: 16px; --bw: 2px; }
  :host([size='lg']) { --sz: 24px; --bw: 2.5px; }

  .ring {
    display: inline-block;
    width: var(--sz);
    height: var(--sz);
    border: var(--bw) solid var(--tone-fg);
    border-right-color: transparent;
    border-radius: 50%;
    animation: sui-spinner-spin 0.7s linear infinite;
  }
  @keyframes sui-spinner-spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "ring");
    __e0.setAttribute("part", "ring");
    __cleanups.push(bindAttr2(__e0, "role", () => $scope.label() ? "status" : null));
    __cleanups.push(bindAttr2(__e0, "aria-label", () => $scope.label() || null));
    __cleanups.push(bindAttr2(__e0, "aria-hidden", () => $scope.label() ? null : "true"));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-badge/sui-badge.sui
import { adoptSuiSheets as adoptSuiSheets3, component as component3, prop as prop3 } from "@ship/sui-runtime";
var sui_badge_default = component3("sui-badge", {
  props: {
    tone: prop3.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop3.union(["solid", "soft", "outline"], { default: "soft", reflect: true }),
    size: prop3.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    pill: prop3.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
    --tone-contrast: var(--background);
    /* Size channel defaults */
    --bdg-h: 22px;
    --bdg-px: 8px;
    --bdg-fs: 10.5px;
    --bdg-gap: 4px;
    --bdg-icon: 10px;
    /* Shape channel */
    --bdg-radius: var(--radius-sm);
  }

  /* Size matrix */
  :host([size='sm']) { --bdg-h: 18px; --bdg-px: 6px;  --bdg-fs: 9.5px;  --bdg-gap: 3px; --bdg-icon: 8px; }
  :host([size='md']) { --bdg-h: 22px; --bdg-px: 8px;  --bdg-fs: 10.5px; --bdg-gap: 4px; --bdg-icon: 10px; }
  :host([size='lg']) { --bdg-h: 26px; --bdg-px: 10px; --bdg-fs: 11.5px; --bdg-gap: 5px; --bdg-icon: 12px; }

  /* Shape — square default, pill when opted-in */
  :host([pill]) { --bdg-radius: var(--radius-full); }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--bdg-gap);
    min-height: var(--bdg-h);
    padding: 0 var(--bdg-px);
    font-family: var(--font-body);
    font-size: var(--bdg-fs);
    font-weight: var(--font-weight-semibold);
    line-height: 1;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    border-radius: var(--bdg-radius);
    /* soft is the default — base .badge renders as soft */
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
    white-space: nowrap;
    -webkit-font-smoothing: antialiased;
    font-variant-numeric: tabular-nums;
  }

  /* Variant matrix */
  :host([variant='soft']) .badge {
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border-color: color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
  }
  :host([variant='solid']) .badge {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='outline']) .badge {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 50%, transparent);
    color: var(--tone-fg);
  }

  /* Leading slot wrapper — hidden when empty */
  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--bdg-icon);
    height: var(--bdg-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

  @media (forced-colors: active) {
    .badge { border-color: CanvasText; color: CanvasText; }
    :host([variant='solid']) .badge { background: Highlight; color: HighlightText; border-color: Highlight; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.pill = props.pill;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "badge");
    __e0.setAttribute("part", "badge");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "leading");
    __e1.setAttribute("part", "leading");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-chip/sui-chip.sui
import { adoptSuiSheets as adoptSuiSheets4, attachRef, bindIf, component as component4, prop as prop4 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-chip/sui-chip.controller.ts
import { controller, effect } from "@ship/sui-runtime";
var sui_chip_controller_default = controller({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "button");
    if (!host.hasAttribute("tabindex") && !scope.disabled()) {
      host.setAttribute("tabindex", "0");
    }
    const stopSync = effect(() => {
      const sel = scope.selected();
      const dis = scope.disabled();
      host.setAttribute("aria-pressed", sel ? "true" : "false");
      if (dis) {
        host.setAttribute("aria-disabled", "true");
        if (host.getAttribute("tabindex") !== "-1")
          host.setAttribute("tabindex", "-1");
      } else {
        host.removeAttribute("aria-disabled");
        if (host.getAttribute("tabindex") === "-1")
          host.setAttribute("tabindex", "0");
      }
    });
    const onKeydown = (e) => {
      if (scope.disabled())
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      } else if (scope.removable() && (e.key === "Backspace" || e.key === "Delete")) {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("ship-remove", { detail: {}, bubbles: true, composed: true }));
      }
    };
    host.addEventListener("keydown", onKeydown);
    const stopRemoveBtn = effect(() => {
      const btn = scope.removeBtn?.();
      if (!btn)
        return;
      const onClick = (e) => {
        e.stopPropagation();
        if (scope.disabled())
          return;
        host.dispatchEvent(new CustomEvent("ship-remove", { detail: {}, bubbles: true, composed: true }));
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    return () => {
      stopSync();
      host.removeEventListener("keydown", onKeydown);
      stopRemoveBtn();
    };
  }
});

// packages/ui-components/src/sui-chip/sui-chip.sui
var sui_chip_default = component4("sui-chip", {
  props: {
    tone: prop4.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true }),
    variant: prop4.union(["solid", "soft", "outline", "ghost"], { default: "soft", reflect: true }),
    size: prop4.union(["sm", "md", "lg"], { default: "sm", reflect: true }),
    selected: prop4.bool({ default: false, reflect: true }),
    removable: prop4.bool({ default: false, reflect: true }),
    disabled: prop4.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    -webkit-tap-highlight-color: transparent;
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-contrast: var(--background);
    --chip-radius: var(--radius-full);
  }

  :host(:not([size])),
  :host([size='sm']) { --chip-h: 22px; --chip-px: 8px;  --chip-fs: var(--text-xs);   --chip-gap: 4px; --chip-icon: 10px; --chip-remove: 14px; }
  :host([size='md']) { --chip-h: 26px; --chip-px: 10px; --chip-fs: var(--text-sm);   --chip-gap: 6px; --chip-icon: 12px; --chip-remove: 16px; }
  :host([size='lg']) { --chip-h: 30px; --chip-px: 12px; --chip-fs: var(--text-base); --chip-gap: 6px; --chip-icon: 14px; --chip-remove: 18px; }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--chip-gap);
    min-height: var(--chip-h);
    padding: 0 var(--chip-px);
    font-family: var(--font-body);
    font-size: var(--chip-fs);
    font-weight: var(--font-weight-medium);
    line-height: 1;
    color: var(--tone-fg);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--chip-radius);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .chip,
  :host([variant='soft']) .chip {
    background: color-mix(in oklch, var(--tone) 14%, transparent);
    border-color: color-mix(in oklch, var(--tone) 22%, transparent);
  }
  :host([variant='soft']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 22%, transparent);
    border-color: color-mix(in oklch, var(--tone) 32%, transparent);
  }
  :host([variant='outline']) .chip {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 45%, transparent);
  }
  :host([variant='outline']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }
  :host([variant='solid']) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='ghost']) .chip {
    background: transparent;
    border-color: transparent;
  }
  :host([variant='ghost']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }

  :host([selected]) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 10%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent);
    font-weight: var(--font-weight-semibold);
  }
  :host([selected]:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
  }
  :host([selected]:not([tone])) .chip,
  :host([selected][tone='muted']) .chip {
    background: var(--foreground);
    border-color: var(--foreground);
    color: var(--background);
  }

  :host(:focus-visible) { outline: none; }
  :host(:focus-visible) .chip {
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }

  :host([disabled]) .chip {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
    filter: saturate(0.65);
    box-shadow: none;
  }

  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--chip-icon);
    height: var(--chip-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

  .remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--chip-remove);
    height: var(--chip-remove);
    margin-inline: 2px -4px;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .remove svg {
    width: 70%;
    height: 70%;
    display: block;
    stroke: currentColor;
    fill: none;
    stroke-width: 1.75;
    stroke-linecap: round;
  }
  .remove:hover:not(:disabled) {
    background: color-mix(in oklch, currentColor 18%, transparent);
  }
  :host([disabled]) .remove { cursor: not-allowed; }

  @media (forced-colors: active) {
    .chip { border-color: CanvasText; color: CanvasText; }
    :host([selected]) .chip { background: Highlight; color: HighlightText; border-color: Highlight; }
    :host(:focus-visible) .chip { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.selected = props.selected;
    $scope.removable = props.removable;
    $scope.disabled = props.disabled;
    let __r = null;
    if (sui_chip_controller_default && typeof sui_chip_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_chip_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.removable())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("button");
          __e7.setAttribute("class", "remove");
          __e7.setAttribute("part", "remove");
          __e7.setAttribute("type", "button");
          __e7.setAttribute("aria-label", "Remove");
          __e7.setAttribute("tabindex", "-1");
          attachRef($scope, "removeBtn", __e7);
          const __e8 = document.createElement("svg");
          __e8.setAttribute("viewbox", "0 0 10 10");
          __e8.setAttribute("aria-hidden", "true");
          const __e9 = document.createElement("path");
          __e9.setAttribute("d", "M2 2 L8 8 M8 2 L2 8");
          __e8.appendChild(__e9);
          __e7.appendChild(__e8);
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets4(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "chip");
    __e0.setAttribute("part", "chip");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "leading");
    __e1.setAttribute("part", "leading");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    const __ifAnchor4 = document.createComment("if");
    __e0.appendChild(__ifAnchor4);
    __cleanups.push(__wireBoundary_b0(__ifAnchor4));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-card/sui-card.sui
import { adoptSuiSheets as adoptSuiSheets5, bindSlotEmpty, component as component5, prop as prop5 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-card/sui-card.controller.ts
import { controller as controller2, effect as effect2 } from "@ship/sui-runtime";
var sui_card_controller_default = controller2({
  mount(state, host) {
    const scope = state;
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      }
    };
    const stopSync = effect2(() => {
      if (scope.interactive()) {
        if (!host.hasAttribute("role"))
          host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
        host.addEventListener("keydown", onKeydown);
        return () => host.removeEventListener("keydown", onKeydown);
      } else {
        if (host.getAttribute("role") === "button")
          host.removeAttribute("role");
        if (host.getAttribute("tabindex") === "0")
          host.removeAttribute("tabindex");
      }
    });
    return () => {
      stopSync();
    };
  }
});

// packages/ui-components/src/sui-card/sui-card.sui
var sui_card_default = component5("sui-card", {
  props: {
    tone: prop5.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop5.union(["default", "outline"], { default: "default", reflect: true }),
    padding: prop5.union(["none", "sm", "md", "lg"], { default: "md", reflect: true }),
    interactive: prop5.bool({ default: false, reflect: true }),
    "accent-bar": prop5.bool({ default: false, reflect: true }),
    "status-rail": prop5.bool({ default: false, reflect: true }),
    fill: prop5.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    position: relative;
    background: var(--card);
    color: var(--card-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    --card-px: 20px;
    --card-py: 16px;
    --card-gap: var(--space-3, 12px);
    --tone-fg: var(--foreground);
    --tone-border: var(--border);
    --tone-bg: transparent;
    padding: var(--card-py) var(--card-px);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  :host([tone='primary'])     { --tone-border: color-mix(in oklch, var(--primary) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--primary) 8%, var(--card)); }
  :host([tone='info'])        { --tone-border: color-mix(in oklch, var(--info) 40%, var(--border));        --tone-bg: color-mix(in oklch, var(--info) 8%, var(--card)); }
  :host([tone='success'])     { --tone-border: color-mix(in oklch, var(--success) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--success) 8%, var(--card)); }
  :host([tone='warning'])     { --tone-border: color-mix(in oklch, var(--warning) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--warning) 8%, var(--card)); }
  :host([tone='destructive']) { --tone-border: color-mix(in oklch, var(--destructive) 40%, var(--border)); --tone-bg: color-mix(in oklch, var(--destructive) 8%, var(--card)); }
  :host([tone='accent'])      { --tone-border: color-mix(in oklch, var(--accent) 40%, var(--border));      --tone-bg: color-mix(in oklch, var(--accent) 8%, var(--card)); }
  :host([tone='muted'])       { --tone-border: var(--border); --tone-bg: var(--muted); }

  :host([tone]:not([tone='default'])) { border-color: var(--tone-border); }
  :host([fill]) { background: var(--tone-bg); border-color: var(--tone-border); }

  :host([padding='none']) { --card-px: 0px; --card-py: 0px; --card-gap: 0px; padding: 0; }
  :host([padding='sm'])   { --card-px: 12px; --card-py: 10px; --card-gap: 8px; padding: 10px 12px; }
  :host([padding='md'])   { --card-px: 20px; --card-py: 16px; padding: 16px 20px; }
  :host([padding='lg'])   { --card-px: 28px; --card-py: 24px; --card-gap: 16px; padding: 24px 28px; }

  :host([variant='outline']) { background: transparent; box-shadow: none; }

  :host([status-rail]) {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--card-px) - 3px);
  }

  :host([accent-bar])::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    opacity: 0;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    pointer-events: none;
  }
  :host([accent-bar]:hover)::before { opacity: 1; }

  :host([interactive]) { cursor: pointer; -webkit-tap-highlight-color: transparent; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-hover, var(--shadow-sm));
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }
  :host([interactive]:active) {
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    transform: translateY(0);
    transition-duration: 60ms;
  }
  :host([interactive]:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  .header, .footer { display: block; }

  /* Separator — show only when slot has projected content */
  :host(:not([data-no-header])) .header {
    margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
    padding: var(--card-py) var(--card-px);
    border-bottom: 1px solid var(--border);
    font-family: var(--heading-font);
    font-weight: var(--heading-weight, 600);
  }
  :host(:not([data-no-footer])) .footer {
    margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
    padding: var(--card-py) var(--card-px);
    border-top: 1px solid var(--border);
  }
  :host([padding='none']:not([data-no-header])) .header {
    margin: 0 0 var(--space-3, 12px);
    padding: 0 0 var(--space-3, 12px);
  }
  :host([padding='none']:not([data-no-footer])) .footer {
    margin: var(--space-3, 12px) 0 0;
    padding: var(--space-3, 12px) 0 0;
  }

  @media (forced-colors: active) {
    :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.padding = props.padding;
    $scope.interactive = props.interactive;
    $scope.accentBar = props.accentBar;
    $scope.statusRail = props.statusRail;
    $scope.fill = props.fill;
    let __r = null;
    if (sui_card_controller_default && typeof sui_card_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_card_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets5(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "header");
    __e0.setAttribute("part", "header");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "header");
    __cleanups.push(bindSlotEmpty(__e1, host, "data-no-header"));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "content");
    __e2.setAttribute("part", "content");
    const __e3 = document.createElement("slot");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "footer");
    __e4.setAttribute("part", "footer");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "footer");
    __cleanups.push(bindSlotEmpty(__e5, host, "data-no-footer"));
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
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

// packages/ui-components/src/sui-link/sui-link.sui
import { adoptSuiSheets as adoptSuiSheets6, bindAttr as bindAttr3, bindIf as bindIf2, component as component6, prop as prop6 } from "@ship/sui-runtime";
var sui_link_default = component6("sui-link", {
  props: {
    href: prop6.string({ default: "" }),
    external: prop6.bool({ default: false, reflect: true }),
    tone: prop6.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop6.union(["default", "underline", "subtle"], { default: "default", reflect: true }),
    target: prop6.string({ default: "" }),
    rel: prop6.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-body);
    --link-fg: var(--primary);
  }

  :host([tone='default'])     { --link-fg: var(--primary); }
  :host([tone='primary'])     { --link-fg: var(--primary); }
  :host([tone='info'])        { --link-fg: var(--info); }
  :host([tone='success'])     { --link-fg: var(--success); }
  :host([tone='warning'])     { --link-fg: var(--warning); }
  :host([tone='destructive']) { --link-fg: var(--destructive); }
  :host([tone='accent'])      { --link-fg: var(--accent); }
  :host([tone='muted'])       { --link-fg: var(--muted-foreground); }

  a {
    color: var(--link-fg);
    text-decoration: none;
    display: inline;
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast),
      text-decoration-color var(--duration-fast);
  }
  a:hover {
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :host([variant='underline']) a {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
    text-decoration-color: color-mix(in oklch, var(--link-fg) 40%, transparent);
  }
  :host([variant='subtle']) a:hover {
    color: color-mix(in oklch, var(--link-fg) 80%, var(--foreground));
  }
  a:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
  .ext {
    display: inline-block;
    margin-left: 2px;
    font-size: 0.9em;
    opacity: 0.8;
    vertical-align: -0.1em;
  }
  .ext svg {
    width: 0.9em;
    height: 0.9em;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.href = props.href;
    $scope.external = props.external;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.target = props.target;
    $scope.rel = props.rel;
    const __wireBoundary_b3 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.external())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("span");
          __e5.setAttribute("class", "sr-only");
          const __t6 = document.createTextNode("(opens in new tab)");
          __e5.appendChild(__t6);
          __frag3.appendChild(__e5);
          const __e7 = document.createElement("span");
          __e7.setAttribute("class", "ext");
          __e7.setAttribute("aria-hidden", "true");
          const __e8 = document.createElement("svg");
          __e8.setAttribute("viewbox", "0 0 24 24");
          __e8.setAttribute("fill", "none");
          __e8.setAttribute("stroke", "currentColor");
          __e8.setAttribute("stroke-width", "2");
          __e8.setAttribute("stroke-linecap", "round");
          __e8.setAttribute("stroke-linejoin", "round");
          const __e9 = document.createElement("path");
          __e9.setAttribute("d", "M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3");
          __e8.appendChild(__e9);
          __e7.appendChild(__e8);
          __frag3.appendChild(__e7);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets6(__root);
    const __e0 = document.createElement("a");
    __e0.setAttribute("part", "link");
    __cleanups.push(bindAttr3(__e0, "href", () => $scope.href()));
    __cleanups.push(bindAttr3(__e0, "target", () => $scope.external() ? "_blank" : $scope.target()));
    __cleanups.push(bindAttr3(__e0, "rel", () => $scope.external() ? "noopener noreferrer" : $scope.rel()));
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    const __ifAnchor2 = document.createComment("if");
    __e0.appendChild(__ifAnchor2);
    __cleanups.push(__wireBoundary_b3(__ifAnchor2));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-divider/sui-divider.sui
import { adoptSuiSheets as adoptSuiSheets7, bindAttr as bindAttr4, component as component7, prop as prop7 } from "@ship/sui-runtime";
var sui_divider_default = component7("sui-divider", {
  props: {
    orientation: prop7.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    variant: prop7.union(["solid", "dashed"], { default: "solid", reflect: true }),
    tone: prop7.union(["default", "muted", "primary", "destructive"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone-bg: var(--border);
  }
  :host([tone='muted'])       { --tone-bg: color-mix(in oklch, var(--border) 50%, transparent); }
  :host([tone='primary'])     { --tone-bg: color-mix(in oklch, var(--primary) 50%, var(--border)); }
  :host([tone='destructive']) { --tone-bg: color-mix(in oklch, var(--destructive) 50%, var(--border)); }

  .line {
    border: 0;
  }

  /* Horizontal — full-width 1px line. */
  :host(:not([orientation='vertical'])) .line {
    width: 100%;
    height: 1px;
    margin: var(--space-2) 0;
    background: var(--tone-bg);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .line {
    background: transparent;
    border-top: 1px dashed var(--tone-bg);
    height: 0;
  }

  /* Vertical — inline 1px line that stretches to parent height. */
  :host([orientation='vertical']) {
    display: inline-block;
    vertical-align: middle;
    align-self: stretch;
  }
  :host([orientation='vertical']) .line {
    width: 1px;
    height: 1em;
    margin: 0 var(--space-2);
    display: inline-block;
    background: var(--tone-bg);
  }
  :host([orientation='vertical'][variant='dashed']) .line {
    background: transparent;
    border-left: 1px dashed var(--tone-bg);
    width: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.orientation = props.orientation;
    $scope.variant = props.variant;
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets7(__root);
    const __e0 = document.createElement("hr");
    __e0.setAttribute("class", "line");
    __e0.setAttribute("part", "line");
    __e0.setAttribute("role", "separator");
    __cleanups.push(bindAttr4(__e0, "aria-orientation", () => $scope.orientation()));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-skeleton/sui-skeleton.sui
import { adoptSuiSheets as adoptSuiSheets8, component as component8, prop as prop8 } from "@ship/sui-runtime";
var sui_skeleton_default = component8("sui-skeleton", {
  props: {
    shape: prop8.union(["text", "line", "block", "circle"], { default: "text", reflect: true }),
    size: prop8.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-block; vertical-align: middle; }
  :host([shape='block']) { display: block; }

  .sk {
    display: block;
    background: linear-gradient(
      90deg,
      var(--muted) 0%,
      color-mix(in oklch, var(--muted) 75%, var(--background)) 50%,
      var(--muted) 100%
    );
    background-size: 200% 100%;
    animation: sui-skeleton-shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }
  :host([shape='circle']) .sk { border-radius: 50%; }

  /* Text / line — narrow heights, full-width default. */
  :host([shape='text']) .sk,
  :host([shape='line']) .sk { width: 100%; }
  :host([size='sm'][shape='text']) .sk,
  :host([size='sm'][shape='line']) .sk { height: 8px; }
  :host([size='md'][shape='text']) .sk,
  :host([size='md'][shape='line']) .sk { height: 12px; }
  :host([size='lg'][shape='text']) .sk,
  :host([size='lg'][shape='line']) .sk { height: 16px; }

  /* Block — fills parent width with a fixed min-height. */
  :host([shape='block']) .sk { width: 100%; min-height: 80px; }

  /* Circle — fixed diameters per size. */
  :host([shape='circle'][size='sm']) .sk { width: 24px; height: 24px; }
  :host([shape='circle'][size='md']) .sk { width: 32px; height: 32px; }
  :host([shape='circle'][size='lg']) .sk { width: 48px; height: 48px; }

  @keyframes sui-skeleton-shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .sk { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.shape = props.shape;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets8(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "sk");
    __e0.setAttribute("part", "sk");
    __e0.setAttribute("role", "status");
    __e0.setAttribute("aria-label", "Loading");
    __e0.setAttribute("aria-live", "polite");
    __e0.setAttribute("aria-busy", "true");
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-stat/sui-stat.sui
import { adoptSuiSheets as adoptSuiSheets9, component as component9, prop as prop9 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-stat/sui-stat.controller.ts
import { controller as controller3, effect as effect3 } from "@ship/sui-runtime";
var sui_stat_controller_default = controller3({
  mount(state, host) {
    const scope = state;
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      }
    };
    const stopSync = effect3(() => {
      if (scope.interactive()) {
        if (!host.hasAttribute("role"))
          host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
        host.addEventListener("keydown", onKeydown);
        return () => host.removeEventListener("keydown", onKeydown);
      } else {
        if (host.getAttribute("role") === "button")
          host.removeAttribute("role");
        if (host.getAttribute("tabindex") === "0")
          host.removeAttribute("tabindex");
      }
    });
    return () => {
      stopSync();
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/ui-components/src/sui-stat/sui-stat.sui
var sui_stat_default = component9("sui-stat", {
  props: {
    tone: prop9.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "default", reflect: true }),
    interactive: prop9.bool({ default: false, reflect: true }),
    size: prop9.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 18px;
    font-family: var(--font-body);
    transition:
      border-color var(--duration-medium) var(--ease-out),
      transform var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
    --tone-fg: var(--foreground);
  }

  :host([interactive]) { cursor: pointer; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }

  .label {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .value-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    flex-wrap: wrap;
  }
  .value {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--font-weight-bold);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    color: var(--tone-fg);
  }
  :host([size='sm']) .value { font-size: var(--text-xl, 20px); }
  :host([size='md']) .value { font-size: var(--text-3xl, 30px); }
  :host([size='lg']) .value { font-size: var(--text-4xl, 36px); }

  .trend {
    display: inline-flex;
    align-items: center;
  }
  .hint {
    font-size: 11px;
    color: var(--muted-foreground);
  }
  ::slotted([slot='trend']) { font-size: var(--text-sm); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.interactive = props.interactive;
    $scope.size = props.size;
    let __r = null;
    if (sui_stat_controller_default && typeof sui_stat_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_stat_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets9(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "label");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "label");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "value-row");
    __e2.setAttribute("part", "value-row");
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "value");
    __e3.setAttribute("part", "value");
    const __e4 = document.createElement("slot");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "trend");
    __e5.setAttribute("part", "trend");
    const __e6 = document.createElement("slot");
    __e6.setAttribute("name", "trend");
    __e5.appendChild(__e6);
    __e2.appendChild(__e5);
    __root.appendChild(__e2);
    const __e7 = document.createElement("span");
    __e7.setAttribute("class", "hint");
    __e7.setAttribute("part", "hint");
    const __e8 = document.createElement("slot");
    __e8.setAttribute("name", "hint");
    __e7.appendChild(__e8);
    __root.appendChild(__e7);
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

// packages/ui-components/src/sui-mark/sui-mark.sui
import { adoptSuiSheets as adoptSuiSheets10, component as component10, prop as prop10 } from "@ship/sui-runtime";
var sui_mark_default = component10("sui-mark", {
  props: {
    tone: prop10.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "warning", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    --tone: var(--warning);
    --tone-fg: var(--warning);
  }
  mark {
    background: color-mix(in oklch, var(--tone) 20%, transparent);
    color: var(--tone-fg);
    padding: 0 2px;
    border-radius: 2px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets10(__root);
    const __e0 = document.createElement("mark");
    __e0.setAttribute("part", "mark");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-kbd/sui-kbd.sui
import { adoptSuiSheets as adoptSuiSheets11, component as component11, prop as prop11 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-kbd/sui-kbd.controller.ts
import { controller as controller4, effect as effect4 } from "@ship/sui-runtime";
var sui_kbd_controller_default = controller4({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "group");
    const stopSync = effect4(() => {
      const k = scope.keys();
      if (k)
        host.setAttribute("aria-label", `Keyboard shortcut: ${k}`);
      else
        host.removeAttribute("aria-label");
    });
    return () => stopSync();
  }
});

// packages/ui-components/src/sui-kbd/sui-kbd.sui
var sui_kbd_default = component11("sui-kbd", {
  props: {
    keys: prop11.string({ default: "" }),
    size: prop11.union(["sm", "md"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    vertical-align: baseline;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    padding: 2px 5px;
    font-family: var(--font-mono);
    font-weight: var(--font-weight-medium, 500);
    font-size: 11px;
    line-height: 1.2;
    color: var(--foreground);
    background: var(--muted);
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-xs);
    white-space: nowrap;
  }

  :host([size='sm']) kbd { font-size: 10px; padding: 1px 4px; min-width: 16px; }
  :host([size='md']) kbd { font-size: 11px; padding: 2px 5px; min-width: 18px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.keys = props.keys;
    $scope.size = props.size;
    let __r = null;
    if (sui_kbd_controller_default && typeof sui_kbd_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_kbd_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets11(__root);
    const __e0 = document.createElement("kbd");
    __e0.setAttribute("part", "kbd");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-avatar/sui-avatar.sui
import { adoptSuiSheets as adoptSuiSheets12, bindAttr as bindAttr5, bindIf as bindIf3, bindText, component as component12, prop as prop12 } from "@ship/sui-runtime";
var sui_avatar_default = component12("sui-avatar", {
  props: {
    src: prop12.string({ default: "" }),
    name: prop12.string({ default: "" }),
    initials: prop12.string({ default: "" }),
    "icon-slug": prop12.string({ default: "" }),
    size: prop12.union(["xs", "sm", "md", "lg", "xl"], { default: "md", reflect: true }),
    shape: prop12.union(["round", "square"], { default: "round", reflect: true }),
    tone: prop12.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --sz: 32px;
    --fs: 13px;
    --av-bg: linear-gradient(
      135deg,
      color-mix(in oklch, var(--primary) 70%, transparent),
      color-mix(in oklch, var(--accent) 70%, transparent)
    );
    --av-fg: var(--foreground);
  }

  :host([size='xs']) { --sz: 20px; --fs: 10px; }
  :host([size='sm']) { --sz: 28px; --fs: 11px; }
  :host([size='md']) { --sz: 32px; --fs: 13px; }
  :host([size='lg']) { --sz: 40px; --fs: 15px; }
  :host([size='xl']) { --sz: 56px; --fs: 22px; }

  /* Tone channel for initials background */
  :host([tone='primary'])     { --av-bg: var(--primary);     --av-fg: var(--primary-foreground); }
  :host([tone='accent'])      { --av-bg: var(--accent);      --av-fg: var(--accent-foreground); }
  :host([tone='info'])        { --av-bg: color-mix(in oklch, var(--info) 20%, var(--muted));        --av-fg: var(--info); }
  :host([tone='success'])     { --av-bg: color-mix(in oklch, var(--success) 20%, var(--muted));     --av-fg: var(--success); }
  :host([tone='warning'])     { --av-bg: color-mix(in oklch, var(--warning) 20%, var(--muted));     --av-fg: var(--warning); }
  :host([tone='destructive']) { --av-bg: color-mix(in oklch, var(--destructive) 20%, var(--muted)); --av-fg: var(--destructive); }
  :host([tone='muted'])       { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }
  :host([tone='default'])     { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }

  .root {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--sz);
    height: var(--sz);
    flex: none;
    background: var(--av-bg);
    color: var(--av-fg);
    font-family: var(--font-body);
    font-weight: var(--font-weight-bold);
    font-size: var(--fs);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: 1;
    overflow: hidden;
    user-select: none;
  }
  :host([shape='round'])  .root { border-radius: 50%; }
  :host([shape='square']) .root { border-radius: var(--radius-sm); }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    padding: 10%;
    box-sizing: border-box;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.src = props.src;
    $scope.name = props.name;
    $scope.initials = props.initials;
    $scope.iconSlug = props.iconSlug;
    $scope.size = props.size;
    $scope.shape = props.shape;
    $scope.tone = props.tone;
    const __wireBoundary_b1 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.iconSlug() || $scope.src())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("img");
          __e4.setAttribute("part", "img");
          __c3.push(bindAttr5(__e4, "src", () => $scope.iconSlug() ? "https://cdn.simpleicons.org/" + $scope.iconSlug() : $scope.src()));
          __c3.push(bindAttr5(__e4, "alt", () => $scope.name()));
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("span");
          __e7.setAttribute("part", "initials");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c6.push(bindText(__t8, () => $scope.initials() ? $scope.initials().slice(0, 2).toUpperCase() : $scope.name().trim().slice(0, 1).toUpperCase() || "?"));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets12(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "root");
    __e0.setAttribute("part", "root");
    __e0.setAttribute("role", "img");
    __cleanups.push(bindAttr5(__e0, "aria-label", () => $scope.name() || $scope.initials() || "avatar"));
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b1(__ifAnchor1));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-progress/sui-progress.sui
import { adoptSuiSheets as adoptSuiSheets13, bindAttr as bindAttr6, component as component13, prop as prop13 } from "@ship/sui-runtime";
var sui_progress_default = component13("sui-progress", {
  props: {
    value: prop13.number({ default: 0 }),
    max: prop13.number({ default: 100 }),
    tone: prop13.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "primary", reflect: true }),
    size: prop13.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    indeterminate: prop13.bool({ default: false, reflect: true }),
    label: prop13.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--primary);
    --h: 6px;
  }

  :host([size='sm']) { --h: 4px; }
  :host([size='md']) { --h: 6px; }
  :host([size='lg']) { --h: 10px; }

  .track {
    width: 100%;
    height: var(--h);
    background: var(--muted);
    border-radius: var(--radius-full);
    overflow: hidden;
    position: relative;
  }

  .fill {
    height: 100%;
    background: var(--tone);
    border-radius: var(--radius-full);
    transition: width var(--duration-fast) var(--ease-out);
  }

  :host([indeterminate]) .fill {
    width: 30%;
    animation: sui-progress-indet 1.4s ease-in-out infinite;
  }
  @keyframes sui-progress-indet {
    0%   { transform: translateX(-100%); }
    50%  { transform: translateX(150%); }
    100% { transform: translateX(350%); }
  }
  @media (prefers-reduced-motion: reduce) {
    :host([indeterminate]) .fill { animation-duration: 3s; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.max = props.max;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.indeterminate = props.indeterminate;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets13(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "track");
    __e0.setAttribute("part", "track");
    __e0.setAttribute("role", "progressbar");
    __cleanups.push(bindAttr6(__e0, "aria-valuenow", () => $scope.indeterminate() ? null : $scope.value()));
    __e0.setAttribute("aria-valuemin", "0");
    __cleanups.push(bindAttr6(__e0, "aria-valuemax", () => $scope.max()));
    __cleanups.push(bindAttr6(__e0, "aria-busy", () => $scope.indeterminate() ? "true" : "false"));
    __cleanups.push(bindAttr6(__e0, "aria-label", () => $scope.label()));
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "fill");
    __e1.setAttribute("part", "fill");
    __cleanups.push(bindAttr6(__e1, "style", () => $scope.indeterminate() ? "" : "width: clamp(0%, " + $scope.value() / ($scope.max() || 1) * 100 + "%, 100%)"));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-button/sui-button.sui
import { adoptSuiSheets as adoptSuiSheets14, bindAttr as bindAttr7, bindIf as bindIf4, bindSlotEmpty as bindSlotEmpty2, component as component14, prop as prop14 } from "@ship/sui-runtime";
var sui_button_default = component14("sui-button", {
  props: {
    tone: prop14.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop14.union(["solid", "soft", "outline", "ghost", "default"], { default: "default", reflect: true }),
    size: prop14.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    type: prop14.union(["button", "submit", "reset"], { default: "button", reflect: true }),
    disabled: prop14.bool({ default: false, reflect: true }),
    loading: prop14.bool({ default: false, reflect: true }),
    label: prop14.string({ default: "" }),
    pressed: prop14.string({ default: "", reflect: true }),
    expanded: prop14.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
    :host {
      display: inline-flex;
      vertical-align: middle;
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
      --btn-h: 34px;
      --btn-px: var(--space-3);
      --btn-fs: 13px;
      --btn-fw: var(--font-weight-medium, 500);
      --btn-gap: 8px;
    }

    /* Size matrix */
    :host([size='sm']) { --btn-h: 28px; --btn-fs: 12px; --btn-px: 10px; --btn-gap: 6px; }
    :host([size='md']) { --btn-h: 34px; --btn-fs: 13px; --btn-px: 12px; --btn-gap: 8px; }
    :host([size='lg']) { --btn-h: 40px; --btn-fs: 14px; --btn-px: 16px; --btn-gap: 8px; }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--btn-gap);
      min-height: var(--btn-h);
      padding: 0 var(--btn-px);
      font-family: var(--font-body);
      font-weight: var(--btn-fw);
      font-size: var(--btn-fs);
      line-height: 1;
      letter-spacing: -0.005em;
      border-radius: var(--radius-md);
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      cursor: pointer;
      user-select: none;
      white-space: nowrap;
      isolation: isolate;
      position: relative;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }

    /* Default (no variant attr) — neutral bordered button. */
    :host(:not([variant])) button,
    :host([variant='default']) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }
    :host(:not([variant])) button:hover:not(:disabled),
    :host([variant='default']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--muted) 70%, var(--background));
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      transform: translateY(-1px);
    }

    /* Solid */
    :host([variant='solid']) button {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      transform: translateY(-1px);
      box-shadow: var(--shadow-sm);
    }

    /* Soft */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='soft']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
    }

    /* Outline */
    :host([variant='outline']) button {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 55%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
    }

    /* Ghost */
    :host([variant='ghost']) button {
      background: transparent;
      border-color: transparent;
      color: var(--tone-fg);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 12%, transparent);
    }

    /* Active press — feels clicked across all variants. */
    :host(:not([disabled]):not([loading])) button:active:not(:disabled) {
      transform: translateY(1px) scale(0.98);
    }
    @media (prefers-reduced-motion: reduce) {
      button { transition: none; }
      button:active { transform: none !important; }
    }

    /* Focus ring */
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 2px var(--background),
        0 0 0 4px color-mix(in oklch, var(--tone) 55%, transparent);
    }

    /* Disabled */
    button:disabled {
      cursor: not-allowed;
      color: color-mix(in oklch, var(--muted-foreground) 65%, transparent);
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 80%, transparent);
      box-shadow: none;
      transform: none;
      filter: saturate(0.6);
    }
    :host([variant='ghost']) button:disabled,
    :host([variant='outline']) button:disabled {
      background: transparent;
    }

    /* Loading state — fade the label, overlay a spinner. */
    :host([loading]) button { cursor: progress; }
    :host([loading]) .content { opacity: 0; }
    .content {
      display: inline-flex;
      align-items: center;
      gap: var(--btn-gap);
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .spinner-wrap {
      position: absolute;
      inset: 0;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      pointer-events: none;
    }
    .spinner {
      width: 14px;
      height: 14px;
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: sui-button-spin 0.7s linear infinite;
    }
    @keyframes sui-button-spin { to { transform: rotate(360deg); } }

    /* Icon-only — collapse padding to a square when default slot is empty. */
    :host([data-icon-only]) button {
      padding-inline: 0;
      width: var(--btn-h);
      min-width: var(--btn-h);
    }

    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
  `,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.type = props.type;
    $scope.disabled = props.disabled;
    $scope.loading = props.loading;
    $scope.label = props.label;
    $scope.pressed = props.pressed;
    $scope.expanded = props.expanded;
    const __wireBoundary_b7 = (anchor) => bindIf4({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("span");
          __e8.setAttribute("class", "spinner-wrap");
          __e8.setAttribute("aria-hidden", "true");
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "spinner");
          __e8.appendChild(__e9);
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets14(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __cleanups.push(bindAttr7(__e0, "type", () => $scope.type()));
    __cleanups.push(bindAttr7(__e0, "disabled", () => $scope.disabled() || $scope.loading()));
    __cleanups.push(bindAttr7(__e0, "aria-busy", () => $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr7(__e0, "aria-disabled", () => $scope.disabled() || $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr7(__e0, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr7(__e0, "aria-pressed", () => $scope.pressed()));
    __cleanups.push(bindAttr7(__e0, "aria-expanded", () => $scope.expanded()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "content");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    const __e3 = document.createElement("slot");
    __cleanups.push(bindSlotEmpty2(__e3, host, "data-icon-only"));
    __e1.appendChild(__e3);
    const __e4 = document.createElement("slot");
    __e4.setAttribute("name", "trailing");
    __e1.appendChild(__e4);
    __e0.appendChild(__e1);
    const __ifAnchor5 = document.createComment("if");
    __e0.appendChild(__ifAnchor5);
    __cleanups.push(__wireBoundary_b7(__ifAnchor5));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-tabs/sui-tabs.sui
import { adoptSuiSheets as adoptSuiSheets15, attachRef as attachRef2, bindAttr as bindAttr8, component as component15, prop as prop15 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-tabs/sui-tabs.controller.ts
import { controller as controller5, effect as effect5, signal } from "@ship/sui-runtime";
var sui_tabs_controller_default = controller5({
  mount(state, host) {
    const scope = state;
    const slotVersion = signal(0);
    const getTabs = () => {
      const slot = scope.tabSlot();
      if (!slot)
        return Array.from(host.querySelectorAll(":scope > sui-tab"));
      return slot.assignedElements().filter((el) => el.tagName.toLowerCase() === "sui-tab");
    };
    const activate = (name) => {
      if (scope.active() === name)
        return;
      scope.active.set(name);
      host.dispatchEvent(new CustomEvent("sui-tab-change", {
        detail: { name },
        bubbles: true,
        composed: true
      }));
    };
    const stopSync = effect5(() => {
      slotVersion();
      const activeName = scope.active();
      const tabs = getTabs();
      let resolved = activeName;
      if (!resolved && tabs.length > 0) {
        resolved = tabs[0].getAttribute("name") ?? "";
        if (resolved)
          scope.active.set(resolved);
      }
      for (const tab of tabs) {
        const name = tab.getAttribute("name") ?? "";
        if (name === resolved) {
          tab.setAttribute("active", "");
        } else {
          tab.removeAttribute("active");
        }
        tab.id = `sui-tab-${name}`;
        tab.setAttribute("aria-controls", `sui-tabpanel-${name}`);
      }
      for (const panel of Array.from(host.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]'))) {
        const name = panel.getAttribute("name") ?? "";
        if (name === resolved) {
          panel.removeAttribute("hidden");
          panel.setAttribute("active", "");
        } else {
          panel.setAttribute("hidden", "");
          panel.removeAttribute("active");
        }
      }
    });
    const onKeydown = (e) => {
      const vertical = scope.orientation() === "vertical";
      const nextKey = vertical ? "ArrowDown" : "ArrowRight";
      const prevKey = vertical ? "ArrowUp" : "ArrowLeft";
      const navKeys = [nextKey, prevKey, "Home", "End"];
      const activateKeys = ["Enter", " "];
      if (!navKeys.includes(e.key) && !activateKeys.includes(e.key))
        return;
      e.preventDefault();
      const tabs = getTabs().filter((t) => !t.hasAttribute("disabled"));
      if (!tabs.length)
        return;
      const names = tabs.map((t) => t.getAttribute("name") ?? "");
      const current = scope.active();
      const i = Math.max(0, names.indexOf(current));
      let next = i;
      if (e.key === nextKey)
        next = (i + 1) % names.length;
      else if (e.key === prevKey)
        next = (i - 1 + names.length) % names.length;
      else if (e.key === "Home")
        next = 0;
      else if (e.key === "End")
        next = names.length - 1;
      const targetName = names[next];
      activate(targetName);
      queueMicrotask(() => {
        const targetTab = tabs[next];
        const btn = targetTab.shadowRoot?.querySelector("button");
        btn ? btn.focus() : targetTab.focus();
      });
    };
    const onClick = (e) => {
      const target = e.target.closest("sui-tab");
      if (!target || target.hasAttribute("disabled"))
        return;
      const name = target.getAttribute("name") ?? "";
      if (name)
        activate(name);
    };
    const onSlotChange = () => slotVersion.set(slotVersion() + 1);
    const strip = scope.strip();
    const tabSlot = scope.tabSlot();
    if (strip) {
      strip.addEventListener("keydown", onKeydown);
      strip.addEventListener("click", onClick);
    }
    if (tabSlot) {
      tabSlot.addEventListener("slotchange", onSlotChange);
    }
    return () => {
      stopSync();
      if (strip) {
        strip.removeEventListener("keydown", onKeydown);
        strip.removeEventListener("click", onClick);
      }
      if (tabSlot) {
        tabSlot.removeEventListener("slotchange", onSlotChange);
      }
    };
  }
});

// packages/ui-components/src/sui-tabs/sui-tabs.sui
var sui_tabs_default = component15("sui-tabs", {
  props: {
    active: prop15.string({ default: "", reflect: true }),
    variant: prop15.union(["underline", "pill", "enclosed"], { default: "underline", reflect: true }),
    size: prop15.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop15.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    sticky: prop15.bool({ default: false, reflect: true }),
    orientation: prop15.union(["horizontal", "vertical"], { default: "horizontal", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--primary);
  }
  :host([sticky]) { position: sticky; top: 0; z-index: 10; background: var(--background); }

  .strip {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border);
  }

  :host([orientation='vertical']) .strip {
    flex-direction: column;
    align-items: flex-start;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }

  :host([variant='pill']) .strip {
    gap: 4px;
    border-bottom: 0;
    padding: 4px;
    background: var(--muted);
    border-radius: var(--radius-md);
  }

  :host([variant='enclosed']) .strip {
    gap: 0;
    border-bottom: 1px solid var(--border);
  }

  ::slotted(sui-tab),
  ::slotted([slot='tab']) {
    flex: 0 0 auto;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.active = props.active;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.sticky = props.sticky;
    $scope.orientation = props.orientation;
    let __r = null;
    if (sui_tabs_controller_default && typeof sui_tabs_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tabs_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets15(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "strip");
    __e0.setAttribute("part", "strip");
    __e0.setAttribute("role", "tablist");
    __cleanups.push(bindAttr8(__e0, "aria-orientation", () => $scope.orientation()));
    attachRef2($scope, "strip", __e0);
    const __e1 = document.createElement("slot");
    attachRef2($scope, "tabSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-tab/sui-tab.sui
import { adoptSuiSheets as adoptSuiSheets16, attachRef as attachRef3, bindAttr as bindAttr9, bindIf as bindIf5, bindText as bindText2, component as component16, prop as prop16 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-tab/sui-tab.controller.ts
import { controller as controller6, effect as effect6 } from "@ship/sui-runtime";
var sui_tab_controller_default = controller6({
  mount(state, host) {
    const scope = state;
    const stopTabIndex = effect6(() => {
      const btn = scope.btn();
      if (!btn)
        return;
      btn.tabIndex = scope.active() ? 0 : -1;
    });
    return () => {
      stopTabIndex();
    };
  }
});

// packages/ui-components/src/sui-tab/sui-tab.sui
var sui_tab_default = component16("sui-tab", {
  props: {
    name: prop16.string({ default: "", reflect: true }),
    label: prop16.string({ default: "", reflect: true }),
    count: prop16.string({ default: "", reflect: true }),
    disabled: prop16.bool({ default: false, reflect: true }),
    active: prop16.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; }

  button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    font: inherit;
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: var(--font-weight-medium, 500);
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    border-radius: 0;
    white-space: nowrap;
    transition:
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  button:hover:not(:disabled) { color: var(--foreground); }

  :host([active]) button {
    color: var(--foreground);
    border-bottom-color: var(--tone, var(--primary));
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .leading {
    display: none;
    align-items: center;
    flex: 0 0 auto;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }

  .count {
    font-size: 11px;
    font-weight: var(--font-weight-medium, 500);
    color: var(--muted-foreground);
    padding: 0 6px;
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--muted) 80%, transparent);
  }
  :host([active]) .count {
    color: var(--tone, var(--primary));
    background: color-mix(in oklch, var(--tone, var(--primary)) 12%, transparent);
  }

  button:focus-visible {
    outline: 2px solid color-mix(in oklch, var(--tone, var(--primary)) 55%, transparent);
    outline-offset: -2px;
  }

  :host([variant='pill']) button {
    border-bottom: 0;
    border-radius: var(--radius-sm);
    padding: 6px 12px;
  }
  :host([variant='pill'][active]) button {
    background: var(--background);
    box-shadow: var(--shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
    color: var(--foreground);
  }

  :host([variant='enclosed']) button {
    border-bottom: 0;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    border: 1px solid transparent;
  }
  :host([variant='enclosed'][active]) button {
    background: var(--background);
    border-color: var(--border) var(--border) transparent;
    color: var(--foreground);
    position: relative;
    bottom: -1px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.label = props.label;
    $scope.count = props.count;
    $scope.disabled = props.disabled;
    $scope.active = props.active;
    let __r = null;
    if (sui_tab_controller_default && typeof sui_tab_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tab_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.count() !== "")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("span");
          __e7.setAttribute("class", "count");
          __e7.setAttribute("aria-hidden", "true");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c6.push(bindText2(__t8, () => $scope.count()));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets16(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __e0.setAttribute("type", "button");
    __e0.setAttribute("role", "tab");
    __cleanups.push(bindAttr9(__e0, "data-name", () => $scope.name()));
    __cleanups.push(bindAttr9(__e0, "aria-selected", () => $scope.active() ? "true" : "false"));
    __cleanups.push(bindAttr9(__e0, "aria-disabled", () => $scope.disabled() ? "true" : "false"));
    __cleanups.push(bindAttr9(__e0, "disabled", () => $scope.disabled()));
    attachRef3($scope, "btn", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "leading");
    __e1.setAttribute("part", "leading");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    const __ifAnchor4 = document.createComment("if");
    __e0.appendChild(__ifAnchor4);
    __cleanups.push(__wireBoundary_b5(__ifAnchor4));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-empty-state/sui-empty-state.sui
import { adoptSuiSheets as adoptSuiSheets17, bindAttr as bindAttr10, bindSlotEmpty as bindSlotEmpty3, component as component17, prop as prop17 } from "@ship/sui-runtime";
var sui_empty_state_default = component17("sui-empty-state", {
  props: {
    variant: prop17.union(["default", "inline", "placeholder", "connecting"], { default: "default", reflect: true }),
    size: prop17.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop17.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
    --es-radius: var(--radius-lg);
    --es-border: 1px dashed var(--border);
    --es-bg: var(--card);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  :host([variant='placeholder']),
  :host([variant='connecting']) {
    --es-border: 0;
    --es-bg: transparent;
    --es-radius: 0;
    --es-pad-y: var(--space-6, 24px);
  }
  :host([variant='inline']) {
    --es-border: 1px dashed var(--border);
    --es-bg: color-mix(in oklch, var(--muted) 55%, transparent);
    --es-radius: var(--radius-md);
    --es-pad-y: var(--space-3, 12px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-3, 12px);
  }

  .box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    background: var(--es-bg);
    border: var(--es-border);
    border-radius: var(--es-radius);
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--tone-fg) 18%, var(--muted));
    color: var(--tone-fg);
    line-height: 1;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon,
  :host([tone='muted']) .icon {
    background: var(--muted);
    color: var(--muted-foreground);
  }
  :host([data-no-icon]) .icon { display: none; }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }
  ::slotted([slot='icon']) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: currentColor;
  }
  ::slotted(svg[slot='icon']) {
    width: var(--es-icon);
    height: var(--es-icon);
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-family: var(--heading-font, var(--font-body));
    font-size: var(--es-title-fs);
    font-weight: var(--heading-weight, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }
  :host([variant='connecting']) .title,
  :host([variant='placeholder']) .title {
    font-weight: var(--font-weight-medium);
    color: var(--muted-foreground);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 40ch;
    color: var(--muted-foreground);
  }
  :host([variant='inline']) .desc { max-width: 60ch; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets17(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __cleanups.push(bindAttr10(__e0, "role", () => $scope.variant() === "connecting" ? "status" : "note"));
    __cleanups.push(bindAttr10(__e0, "aria-live", () => $scope.variant() === "connecting" ? "polite" : "off"));
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "icon");
    __e1.setAttribute("part", "icon");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    __cleanups.push(bindSlotEmpty3(__e2, host, "data-no-icon"));
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "copy");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    __e4.setAttribute("part", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "desc");
    __e6.setAttribute("part", "desc");
    const __e7 = document.createElement("slot");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    __e0.appendChild(__e3);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "actions");
    __e8.setAttribute("part", "actions");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "action");
    __e8.appendChild(__e9);
    __e0.appendChild(__e8);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-input/sui-input.sui
import { adoptSuiSheets as adoptSuiSheets18, attachRef as attachRef4, bindAttr as bindAttr12, bindIf as bindIf6, bindSlotEmpty as bindSlotEmpty4, bindText as bindText3, bindValue, component as component18, prop as prop18 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-input/sui-input.controller.ts
import { bindAttr as bindAttr11, bindNativeEvent, controller as controller7, onCleanup } from "@ship/sui-runtime";
var _idSeq = 0;
var sui_input_controller_default = controller7({
  mount(state, host) {
    const scope = state;
    const inputEl = scope.inputEl();
    if (!inputEl)
      return;
    host.expose({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      clear: () => {
        inputEl.value = "";
        scope.value.set("");
      },
      reportValidity: () => inputEl.reportValidity()
    });
    bindNativeEvent(inputEl, "input", () => host.emit("sui-input", { value: inputEl.value }));
    bindNativeEvent(inputEl, "change", () => host.emit("sui-input-change", { value: inputEl.value }));
    const errId = `sui-input-err-${++_idSeq}`;
    onCleanup(bindAttr11(inputEl, "aria-describedby", () => {
      const parts = [];
      if (scope.invalid() && scope.errorMessage()) {
        const errRef = scope.errorEl;
        const errorEl = typeof errRef === "function" ? errRef() : null;
        if (errorEl) {
          errorEl.id = errId;
          parts.push(errId);
        }
      }
      const extra = scope.describedBy();
      if (extra)
        parts.push(extra);
      return parts.length > 0 ? parts.join(" ") : null;
    }));
  }
});

// packages/ui-components/src/sui-input/sui-input.sui
var sui_input_default = component18("sui-input", {
  props: {
    type: prop18.union(["text", "search", "email", "url", "tel", "password", "number"], { default: "text", reflect: true }),
    value: prop18.string({ default: "" }),
    placeholder: prop18.string({ default: "" }),
    name: prop18.string({ default: "" }),
    autocomplete: prop18.string({ default: "" }),
    required: prop18.bool({ default: false, reflect: true }),
    disabled: prop18.bool({ default: false, reflect: true }),
    readonly: prop18.bool({ default: false, reflect: true }),
    invalid: prop18.bool({ default: false, reflect: true }),
    loading: prop18.bool({ default: false, reflect: true }),
    tone: prop18.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop18.union(["default", "ghost"], { default: "default", reflect: true }),
    size: prop18.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop18.string({ default: "" }),
    "described-by": prop18.string({ default: "" }),
    "error-message": prop18.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    flex-direction: column;
    gap: 4px;
    vertical-align: middle;
    width: 100%;
    --tone-border: var(--border);
    --tone-ring: var(--primary);
    --h: 34px;
    --fs: 13px;
    --px: 10px;
  }

  :host([tone='primary'])     { --tone-border: var(--primary);     --tone-ring: var(--primary); }
  :host([tone='info'])        { --tone-border: var(--info);         --tone-ring: var(--info); }
  :host([tone='success'])     { --tone-border: var(--success);      --tone-ring: var(--success); }
  :host([tone='warning'])     { --tone-border: var(--warning);      --tone-ring: var(--warning); }
  :host([tone='destructive']),
  :host([invalid])            { --tone-border: var(--destructive);  --tone-ring: var(--destructive); }
  :host([tone='accent'])      { --tone-border: var(--accent);       --tone-ring: var(--accent); }
  :host([tone='muted'])       { --tone-border: var(--muted-foreground); --tone-ring: var(--muted-foreground); }

  :host([size='sm']) { --h: 28px; --fs: 12px; --px: 8px; }
  :host([size='md']) { --h: 34px; --fs: 13px; --px: 10px; }
  :host([size='lg']) { --h: 40px; --fs: 14px; --px: 12px; }

  .wrap {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    min-height: var(--h);
    padding: 0 var(--px);
    background: var(--background);
    border: 1px solid var(--tone-border);
    border-radius: var(--radius-md);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  :host([variant='ghost']) .wrap {
    border-color: transparent;
    background: transparent;
  }

  .wrap:focus-within {
    border-color: var(--tone-ring);
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-ring) 25%, transparent);
  }

  :host([disabled]) .wrap {
    background: var(--muted);
    cursor: not-allowed;
    opacity: 0.65;
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    padding: 0;
    font: inherit;
    font-family: var(--font-body);
    font-size: var(--fs);
    color: var(--foreground);
  }
  input::placeholder { color: var(--muted-foreground); }
  input:disabled { cursor: not-allowed; }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    color: var(--muted-foreground);
  }

  .spinner {
    display: inline-block;
    flex: 0 0 auto;
    width: 14px;
    height: 14px;
    border: 2px solid color-mix(in oklch, var(--tone-ring) 30%, transparent);
    border-top-color: var(--tone-ring);
    border-radius: 50%;
    animation: sui-spin 0.6s linear infinite;
  }

  @keyframes sui-spin {
    to { transform: rotate(360deg); }
  }

  .error {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--destructive);
    line-height: 1.4;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    $scope.placeholder = props.placeholder;
    $scope.name = props.name;
    $scope.autocomplete = props.autocomplete;
    $scope.required = props.required;
    $scope.disabled = props.disabled;
    $scope.readonly = props.readonly;
    $scope.invalid = props.invalid;
    $scope.loading = props.loading;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.label = props.label;
    $scope.describedBy = props.describedBy;
    $scope.errorMessage = props.errorMessage;
    let __r = null;
    if (sui_input_controller_default && typeof sui_input_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_input_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b13 = (anchor) => bindIf6({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElement("span");
          __e6.setAttribute("class", "spinner");
          __e6.setAttribute("aria-hidden", "true");
          __frag4.appendChild(__e6);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("slot");
          __e9.setAttribute("name", "trailing");
          __c8.push(bindSlotEmpty4(__e9, host, "data-no-trailing"));
          __frag7.appendChild(__e9);
          return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
            for (const c of __c8)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b14 = (anchor) => bindIf6({
      anchor,
      test: () => {
        if ($scope.invalid() && $scope.errorMessage())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("span");
          __e13.setAttribute("class", "error");
          __e13.setAttribute("role", "alert");
          attachRef4($scope, "errorEl", __e13);
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c12.push(bindText3(__t14, () => $scope.errorMessage()));
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets18(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "wrap");
    __e0.setAttribute("part", "wrap");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "leading");
    __cleanups.push(bindSlotEmpty4(__e1, host, "data-no-leading"));
    __e0.appendChild(__e1);
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __cleanups.push(bindAttr12(__e2, "type", () => $scope.type()));
    __cleanups.push(bindAttr12(__e2, "placeholder", () => $scope.placeholder()));
    __cleanups.push(bindAttr12(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr12(__e2, "autocomplete", () => $scope.autocomplete() || "off"));
    __cleanups.push(bindAttr12(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr12(__e2, "readonly", () => $scope.readonly()));
    __cleanups.push(bindAttr12(__e2, "required", () => $scope.required()));
    __cleanups.push(bindAttr12(__e2, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr12(__e2, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr12(__e2, "aria-required", () => $scope.required() ? "true" : "false"));
    __cleanups.push(bindAttr12(__e2, "aria-busy", () => $scope.loading() ? "true" : "false"));
    __cleanups.push(bindValue(__e2, $scope.value));
    attachRef4($scope, "inputEl", __e2);
    __e0.appendChild(__e2);
    const __ifAnchor3 = document.createComment("if");
    __e0.appendChild(__ifAnchor3);
    __cleanups.push(__wireBoundary_b13(__ifAnchor3));
    __root.appendChild(__e0);
    const __ifAnchor10 = document.createComment("if");
    __root.appendChild(__ifAnchor10);
    __cleanups.push(__wireBoundary_b14(__ifAnchor10));
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

// packages/ui-components/src/sui-icon/sui-icon.sui
import { adoptSuiSheets as adoptSuiSheets19, component as component19, prop as prop19 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-icon/sui-icon.controller.ts
import { controller as controller8, effect as effect7, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-icon/registry.ts
var ALIASES = {
  git: "git-branch",
  refresh: "refresh-cw",
  stop: "stop-circle",
  warning: "triangle-alert"
};
var ICON_REGISTRY_URL = "/_sui/lucide-icons.json";
var registry = null;
var pending = null;
function getIconRegistry() {
  return registry;
}
function loadIconRegistry() {
  if (registry)
    return Promise.resolve(registry);
  if (pending)
    return pending;
  pending = fetch(ICON_REGISTRY_URL).then((r) => {
    if (!r.ok)
      throw new Error(`lucide registry fetch ${r.status}`);
    return r.json();
  }).then((map) => {
    registry = map;
    return map;
  }).catch((err) => {
    pending = null;
    throw err;
  });
  return pending;
}
function resolveIconName(name) {
  if (!name || !registry)
    return;
  if (registry[name])
    return name;
  const aliased = ALIASES[name];
  if (aliased && registry[aliased])
    return aliased;
  return;
}
function getIconSvg(resolved) {
  return registry ? registry[resolved] : undefined;
}

// packages/ui-components/src/sui-icon/sui-icon.controller.ts
var SVG_NS = "http://www.w3.org/2000/svg";
var registryRev = signal2(getIconRegistry() ? 1 : 0);
var kicked = false;
function kickFetch() {
  if (kicked || getIconRegistry())
    return;
  kicked = true;
  loadIconRegistry().then(() => registryRev.set(registryRev() + 1)).catch(() => {
    kicked = false;
  });
}
function makeSvg(doc) {
  const svg = doc.createElementNS(SVG_NS, "svg");
  svg.setAttribute("part", "svg");
  svg.setAttribute("class", "sui-icon-svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "2");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  return svg;
}
function populatePaths(svg, fragment) {
  while (svg.firstChild)
    svg.removeChild(svg.firstChild);
  const parsed = new DOMParser().parseFromString(`<svg xmlns="${SVG_NS}">${fragment}</svg>`, "image/svg+xml").documentElement;
  while (parsed.firstChild) {
    svg.appendChild(svg.ownerDocument.adoptNode(parsed.firstChild));
  }
}
var sui_icon_controller_default = controller8({
  mount(state, host) {
    const scope = state;
    const root = host.shadowRoot;
    let svgEl = null;
    let lastResolved = null;
    const stopAria = effect7(() => {
      const label = scope.label();
      if (label) {
        host.setAttribute("role", "img");
        host.setAttribute("aria-label", label);
        host.removeAttribute("aria-hidden");
      } else {
        host.setAttribute("aria-hidden", "true");
        host.removeAttribute("role");
        host.removeAttribute("aria-label");
      }
    });
    const stopRender = effect7(() => {
      registryRev();
      const requested = scope.name();
      const resolved = resolveIconName(requested) ?? null;
      const label = scope.label();
      if (!resolved) {
        if (svgEl) {
          svgEl.remove();
          svgEl = null;
        }
        lastResolved = null;
        if (requested && !getIconRegistry())
          kickFetch();
        return;
      }
      if (!svgEl) {
        svgEl = makeSvg(host.ownerDocument);
        root.insertBefore(svgEl, root.firstChild);
      }
      if (resolved !== lastResolved) {
        populatePaths(svgEl, getIconSvg(resolved));
        lastResolved = resolved;
      }
      if (label)
        svgEl.removeAttribute("aria-hidden");
      else
        svgEl.setAttribute("aria-hidden", "true");
    });
    return () => {
      stopAria();
      stopRender();
    };
  }
});

// packages/ui-components/src/sui-icon/sui-icon.sui
var sui_icon_default = component19("sui-icon", {
  props: {
    name: prop19.string({ default: "", reflect: true }),
    size: prop19.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop19.union(["default", "info", "success", "warning", "destructive", "accent", "muted", "primary"], { default: "default", reflect: true }),
    label: prop19.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    /* default tone; suiTonesSheet (adopted by compiler) overrides per [tone] */
    --tone-fg: var(--foreground);
    color: var(--tone-fg);
  }

  /* Size matrix — applies to the controller-injected SVG */
  svg.sui-icon-svg {
    display: block;
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
  }
  :host([size='sm']) svg.sui-icon-svg { width: 12px; height: 12px; }
  :host([size='lg']) svg.sui-icon-svg { width: 20px; height: 20px; }

  /* Slot passthrough: user-provided SVG fills the icon box */
  ::slotted(svg) { width: 100%; height: 100%; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.label = props.label;
    let __r = null;
    if (sui_icon_controller_default && typeof sui_icon_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_icon_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets19(__root);
    const __e0 = document.createElement("slot");
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-separator/sui-separator.sui
import { adoptSuiSheets as adoptSuiSheets20, bindAttr as bindAttr13, component as component20, prop as prop20 } from "@ship/sui-runtime";
var sui_separator_default = component20("sui-separator", {
  props: {
    orientation: prop20.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    variant: prop20.union(["solid", "dashed"], { default: "solid", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
  }

  .sep {
    border: 0;
    margin: 0;
    padding: 0;
    flex: 0 0 auto;
  }

  /* Horizontal (default) */
  :host(:not([orientation='vertical'])) .sep {
    display: block;
    width: 100%;
    height: 1px;
    margin: 8px 0;
    background: var(--border);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .sep {
    background: transparent;
    border-top: 1px dashed var(--border);
    height: 0;
  }

  /* Vertical */
  :host([orientation='vertical']) {
    display: inline-block;
    align-self: stretch;
  }
  :host([orientation='vertical']) .sep {
    display: inline-block;
    width: 1px;
    min-height: 100%;
    margin: 0 8px;
    background: var(--border);
  }
  :host([orientation='vertical'][variant='dashed']) .sep {
    background: transparent;
    border-left: 1px dashed var(--border);
    width: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.orientation = props.orientation;
    $scope.variant = props.variant;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets20(__root);
    const __e0 = document.createElement("hr");
    __e0.setAttribute("class", "sep");
    __e0.setAttribute("part", "sep");
    __e0.setAttribute("role", "separator");
    __cleanups.push(bindAttr13(__e0, "aria-orientation", () => $scope.orientation()));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-label/sui-label.sui
import { adoptSuiSheets as adoptSuiSheets21, attachRef as attachRef5, bindIf as bindIf7, component as component21, prop as prop21 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-label/sui-label.controller.ts
import { controller as controller9, effect as effect8 } from "@ship/sui-runtime";
var sui_label_controller_default = controller9({
  mount(state, _host) {
    const scope = state;
    const stopFor = effect8(() => {
      const labelEl = scope.labelEl();
      if (!labelEl)
        return;
      labelEl.htmlFor = scope["for"]?.() ?? "";
    });
    return () => {
      stopFor();
    };
  }
});

// packages/ui-components/src/sui-label/sui-label.sui
var sui_label_default = component21("sui-label", {
  props: {
    for: prop21.string({ default: "" }),
    size: prop21.union(["sm", "md"], { default: "sm", reflect: true }),
    required: prop21.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-block;
  }

  label {
    display: inline;
    font-family: var(--font-body);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    cursor: pointer;
  }

  :host(:not([size])) label,
  :host([size='sm']) label { font-size: 10px; }
  :host([size='md']) label { font-size: 11px; }

  .req {
    color: var(--destructive);
    margin-left: 2px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.for = props.for;
    $scope.size = props.size;
    $scope.required = props.required;
    let __r = null;
    if (sui_label_controller_default && typeof sui_label_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_label_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindIf7({
      anchor,
      test: () => {
        if ($scope.required())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("span");
          __e5.setAttribute("class", "req");
          __e5.setAttribute("aria-hidden", "true");
          const __t6 = document.createTextNode("*");
          __e5.appendChild(__t6);
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets21(__root);
    const __e0 = document.createElement("label");
    __e0.setAttribute("part", "label");
    attachRef5($scope, "labelEl", __e0);
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    const __ifAnchor2 = document.createComment("if");
    __e0.appendChild(__ifAnchor2);
    __cleanups.push(__wireBoundary_b1(__ifAnchor2));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-alert/sui-alert.sui
import { adoptSuiSheets as adoptSuiSheets22, attachRef as attachRef6, bindAttr as bindAttr14, component as component22, prop as prop22 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-alert/sui-alert.controller.ts
import { controller as controller10, effect as effect9 } from "@ship/sui-runtime";
var sui_alert_controller_default = controller10({
  mount(state, host) {
    const scope = state;
    const stopDismiss = effect9(() => {
      const btn = scope.closeBtn();
      if (!btn)
        return;
      const onClick = () => {
        if (!scope.dismissible())
          return;
        host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
        host.remove();
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    return () => {
      stopDismiss();
    };
  }
});

// packages/ui-components/src/sui-alert/sui-alert.sui
var sui_alert_default = component22("sui-alert", {
  props: {
    tone: prop22.union(["info", "success", "warning", "destructive", "muted"], { default: "info", reflect: true }),
    dismissible: prop22.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--info);
    --tone-fg: var(--info);
    --alert-bg: color-mix(in oklch, var(--tone) 12%, var(--background));
    --alert-border: color-mix(in oklch, var(--tone) 30%, transparent);
  }

  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  .box {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius);
    border: 1px solid var(--alert-border);
    background: var(--alert-bg);
    color: var(--tone-fg);
    font-family: var(--font-body);
    font-size: 13px;
    line-height: 1.5;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .title { font-weight: 600; }

  .body { color: var(--foreground); }

  .actions {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }

  .close {
    display: none;
    background: transparent;
    border: 0;
    color: inherit;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }
  :host([dismissible]) .close {
    display: inline-flex;
  }
  .close:hover {
    background: color-mix(in oklch, var(--tone-fg) 12%, transparent);
  }

  ::slotted([slot='icon']) { display: inline-flex; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.dismissible = props.dismissible;
    let __r = null;
    if (sui_alert_controller_default && typeof sui_alert_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_alert_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets22(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __cleanups.push(bindAttr14(__e0, "role", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "alert" : "status"));
    __cleanups.push(bindAttr14(__e0, "aria-live", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "assertive" : "polite"));
    __e0.setAttribute("aria-atomic", "true");
    const __e1 = document.createElement("span");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "content");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "body");
    const __e7 = document.createElement("slot");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "actions");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "actions");
    __e8.appendChild(__e9);
    __e3.appendChild(__e8);
    __e0.appendChild(__e3);
    const __e10 = document.createElement("button");
    __e10.setAttribute("class", "close");
    __e10.setAttribute("part", "close");
    __e10.setAttribute("aria-label", "Dismiss");
    attachRef6($scope, "closeBtn", __e10);
    const __t11 = document.createTextNode("×");
    __e10.appendChild(__t11);
    __e0.appendChild(__e10);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-tooltip/sui-tooltip.sui
import { adoptSuiSheets as adoptSuiSheets23, attachRef as attachRef7, component as component23, prop as prop23 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-tooltip/sui-tooltip.controller.ts
import { controller as controller11, effect as effect10 } from "@ship/sui-runtime";
var _seq = 0;
var sui_tooltip_controller_default = controller11({
  mount(state, host) {
    const scope = state;
    const bubbleId = `sui-tooltip-${++_seq}`;
    const stopBubbleId = effect10(() => {
      const bubble = scope.bubbleEl();
      if (bubble)
        bubble.id = bubbleId;
    });
    const wireTrigger = () => {
      const slot = scope.defaultSlot();
      if (!slot)
        return;
      const first = slot.assignedElements({ flatten: true })[0];
      if (!first)
        return;
      const existing = first.getAttribute("aria-describedby") ?? "";
      if (!existing.split(/\s+/).includes(bubbleId)) {
        first.setAttribute("aria-describedby", existing ? `${existing} ${bubbleId}` : bubbleId);
      }
    };
    const stopSlot = effect10(() => {
      const slot = scope.defaultSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", wireTrigger);
      wireTrigger();
      return () => slot.removeEventListener("slotchange", wireTrigger);
    });
    const onKeydown = (e) => {
      if (e.key === "Escape")
        host.blur();
    };
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopBubbleId();
      stopSlot();
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/ui-components/src/sui-tooltip/sui-tooltip.sui
var sui_tooltip_default = component23("sui-tooltip", {
  props: {
    placement: prop23.union(["top", "bottom", "left", "right"], { default: "top", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: inline-flex;
  }

  .bubble {
    position: absolute;
    z-index: var(--z-popover, 50);
    padding: 4px 8px;
    background: var(--foreground);
    color: var(--background);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: var(--font-weight-medium, 500);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity var(--duration-fast, 120ms) var(--ease-out, ease-out);
    box-shadow: var(--shadow-sm);
  }

  :host(:hover) .bubble,
  :host(:focus-within) .bubble {
    opacity: 1;
  }

  /* Placement: top (default) */
  :host(:not([placement])) .bubble,
  :host([placement='top']) .bubble {
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  :host([placement='bottom']) .bubble {
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  :host([placement='left']) .bubble {
    right: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }

  :host([placement='right']) .bubble {
    left: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.placement = props.placement;
    let __r = null;
    if (sui_tooltip_controller_default && typeof sui_tooltip_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tooltip_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets23(__root);
    const __e0 = document.createElement("slot");
    attachRef7($scope, "defaultSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "bubble");
    __e1.setAttribute("role", "tooltip");
    __e1.setAttribute("part", "bubble");
    attachRef7($scope, "bubbleEl", __e1);
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "content");
    __e1.appendChild(__e2);
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

// packages/ui-components/src/sui-collapsible/sui-collapsible.sui
import { adoptSuiSheets as adoptSuiSheets24, attachRef as attachRef8, component as component24, prop as prop24 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-collapsible/sui-collapsible.controller.ts
import { controller as controller12, effect as effect11 } from "@ship/sui-runtime";
var _seq2 = 0;
var sui_collapsible_controller_default = controller12({
  mount(state, host) {
    const scope = state;
    const contentId = `sui-collapsible-${++_seq2}`;
    const stopContentId = effect11(() => {
      const wrap = scope.contentWrap();
      if (wrap)
        wrap.id = contentId;
    });
    const syncAriaExpanded = () => {
      const slot = scope.triggerSlot();
      if (!slot)
        return;
      const isOpen = scope.open();
      for (const el of slot.assignedElements()) {
        const h = el;
        h.setAttribute("aria-expanded", String(isOpen));
        h.setAttribute("aria-controls", contentId);
        if (!h.hasAttribute("tabindex") && !h.matches("button, a, input, select, textarea")) {
          h.setAttribute("tabindex", "0");
        }
      }
    };
    const stopSlot = effect11(() => {
      const slot = scope.triggerSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", syncAriaExpanded);
      syncAriaExpanded();
      return () => slot.removeEventListener("slotchange", syncAriaExpanded);
    });
    const stopOpenSync = effect11(() => {
      scope.open();
      scope.triggerSlot();
      syncAriaExpanded();
    });
    const toggle = () => {
      if (scope.disabled())
        return;
      const next = !scope.open();
      scope.open.set(next);
      host.dispatchEvent(new CustomEvent("sui-toggle", {
        detail: { open: next },
        bubbles: true,
        composed: true
      }));
    };
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        toggle();
      }
    };
    const stopWrap = effect11(() => {
      const wrap = scope.triggerWrap();
      if (!wrap)
        return;
      wrap.addEventListener("click", toggle);
      wrap.addEventListener("keydown", onKeydown);
      return () => {
        wrap.removeEventListener("click", toggle);
        wrap.removeEventListener("keydown", onKeydown);
      };
    });
    return () => {
      stopContentId();
      stopSlot();
      stopOpenSync();
      stopWrap();
    };
  }
});

// packages/ui-components/src/sui-collapsible/sui-collapsible.sui
var sui_collapsible_default = component24("sui-collapsible", {
  props: {
    open: prop24.bool({ default: false, reflect: true }),
    disabled: prop24.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .content-wrap {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([open]) .content-wrap {
    grid-template-rows: 1fr;
  }

  .content-inner {
    overflow: hidden;
  }

  ::slotted([slot='trigger']) {
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) ::slotted([slot='trigger']) {
    cursor: not-allowed;
    opacity: 0.5;
  }

  @media (prefers-reduced-motion: reduce) {
    .content-wrap { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.disabled = props.disabled;
    let __r = null;
    if (sui_collapsible_controller_default && typeof sui_collapsible_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_collapsible_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets24(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "trigger-wrap");
    attachRef8($scope, "triggerWrap", __e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "trigger");
    attachRef8($scope, "triggerSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "content-wrap");
    __e2.setAttribute("part", "content");
    attachRef8($scope, "contentWrap", __e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "content-inner");
    const __e4 = document.createElement("slot");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-checkbox/sui-checkbox.sui
import { adoptSuiSheets as adoptSuiSheets25, attachRef as attachRef9, bindAttr as bindAttr15, bindChecked, component as component25, prop as prop25 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-checkbox/sui-checkbox.controller.ts
import { controller as controller13, effect as effect12 } from "@ship/sui-runtime";
var sui_checkbox_controller_default = controller13({
  mount(state, host) {
    const scope = state;
    const stopIndet = effect12(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      el.indeterminate = scope.indeterminate();
    });
    const stopChange = effect12(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      const onNativeChange = () => {
        scope.indeterminate.set(false);
        host.dispatchEvent(new CustomEvent("sui-change", {
          detail: { checked: el.checked, value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      el.addEventListener("change", onNativeChange);
      return () => el.removeEventListener("change", onNativeChange);
    });
    return () => {
      stopIndet();
      stopChange();
    };
  }
});

// packages/ui-components/src/sui-checkbox/sui-checkbox.sui
var sui_checkbox_default = component25("sui-checkbox", {
  props: {
    checked: prop25.bool({ default: false, reflect: true }),
    indeterminate: prop25.bool({ default: false, reflect: true }),
    disabled: prop25.bool({ default: false, reflect: true }),
    required: prop25.bool({ default: false, reflect: true }),
    invalid: prop25.bool({ default: false, reflect: true }),
    tone: prop25.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop25.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop25.string({ default: "" }),
    value: prop25.string({ default: "on" }),
    label: prop25.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --cb-size: 16px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2px; }
  :host([size='sm']) { --cb-size: 14px; --cb-radius: 3px;                   --cb-stroke: 1.75px; }
  :host([size='lg']) { --cb-size: 18px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2.25px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .box {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--cb-size);
    height: var(--cb-size);
    background: var(--background);
    border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
    border-radius: var(--cb-radius);
    box-sizing: border-box;
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked]):not([indeterminate])) label:hover .box {
    border-color: color-mix(in oklch, var(--border) 220%, transparent);
    box-shadow: var(--shadow-sm);
  }

  :host([invalid]) .box {
    border-color: color-mix(in oklch, var(--destructive) 70%, var(--border));
  }
  :host([invalid]:not([disabled])) label:hover .box {
    border-color: color-mix(in oklch, var(--destructive) 85%, var(--border));
  }

  :host([checked]) .box,
  :host([indeterminate]) .box {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent),
      var(--shadow-xs);
  }
  :host([checked]:not([disabled])) label:hover .box,
  :host([indeterminate]:not([disabled])) label:hover .box {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
      var(--shadow-sm);
  }

  /* Glyphs — CSS-drawn check and indeterminate indicator. */
  .glyph {
    position: absolute;
    inset: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--tone-on-fg);
    opacity: 0;
    transform: scale(0.6);
    transition:
      opacity var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]:not([indeterminate])) .glyph.check,
  :host([indeterminate]) .glyph.mixed {
    opacity: 1;
    transform: scale(1);
  }

  /* Check mark: rotated L-shape via borders */
  .glyph.check::before {
    content: '';
    display: block;
    width: 45%;
    height: 28%;
    border-left: var(--cb-stroke) solid currentColor;
    border-bottom: var(--cb-stroke) solid currentColor;
    transform: rotate(-45deg) translate(0, -15%);
  }

  /* Indeterminate bar */
  .glyph.mixed::before {
    content: '';
    display: block;
    width: 60%;
    height: var(--cb-stroke);
    background: currentColor;
    border-radius: 1px;
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    opacity: 0;
    cursor: inherit;
    appearance: none;
    -webkit-appearance: none;
  }

  :host(:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([invalid]:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    :host([checked]) .box,
    :host([indeterminate]) .box { background: Highlight; }
    .glyph { color: HighlightText; }
    :host(:not([disabled])) label:has(input:focus-visible) .box {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }


  :host([disabled]) { color: var(--muted-foreground); }
  :host([disabled]) .box {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    filter: saturate(0.6);
    box-shadow: none;
  }
  :host([disabled][checked]) .box,
  :host([disabled][indeterminate]) .box {
    background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
  }

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.checked = props.checked;
    $scope.indeterminate = props.indeterminate;
    $scope.disabled = props.disabled;
    $scope.required = props.required;
    $scope.invalid = props.invalid;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.value = props.value;
    $scope.label = props.label;
    let __r = null;
    if (sui_checkbox_controller_default && typeof sui_checkbox_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_checkbox_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets25(__root);
    const __e0 = document.createElement("label");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "box");
    __e1.setAttribute("part", "box");
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __e2.setAttribute("type", "checkbox");
    __cleanups.push(bindChecked(__e2, $scope.checked));
    __cleanups.push(bindAttr15(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr15(__e2, "required", () => $scope.required()));
    __cleanups.push(bindAttr15(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr15(__e2, "value", () => $scope.value()));
    __cleanups.push(bindAttr15(__e2, "aria-checked", () => $scope.indeterminate() ? "mixed" : $scope.checked() ? "true" : "false"));
    __cleanups.push(bindAttr15(__e2, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr15(__e2, "aria-label", () => $scope.label() || $scope.name() || "checkbox"));
    attachRef9($scope, "inputEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "glyph check");
    __e3.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e3);
    const __e4 = document.createElement("span");
    __e4.setAttribute("class", "glyph mixed");
    __e4.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e4);
    __e0.appendChild(__e1);
    const __e5 = document.createElement("slot");
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-switch/sui-switch.sui
import { adoptSuiSheets as adoptSuiSheets26, attachRef as attachRef10, bindAttr as bindAttr16, bindChecked as bindChecked2, component as component26, prop as prop26 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-switch/sui-switch.controller.ts
import { controller as controller14, effect as effect13 } from "@ship/sui-runtime";
var sui_switch_controller_default = controller14({
  mount(state, host) {
    const scope = state;
    const stopChange = effect13(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      const onNativeChange = () => {
        host.dispatchEvent(new CustomEvent("sui-change", {
          detail: { checked: el.checked },
          bubbles: true,
          composed: true
        }));
      };
      el.addEventListener("change", onNativeChange);
      return () => el.removeEventListener("change", onNativeChange);
    });
    return () => stopChange();
  }
});

// packages/ui-components/src/sui-switch/sui-switch.sui
var sui_switch_default = component26("sui-switch", {
  props: {
    checked: prop26.bool({ default: false, reflect: true }),
    disabled: prop26.bool({ default: false, reflect: true }),
    tone: prop26.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop26.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop26.string({ default: "" }),
    value: prop26.string({ default: "on" }),
    label: prop26.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sw-w: 28px; --sw-h: 16px; --sw-thumb: 12px; }
  :host([size='sm']) { --sw-w: 24px; --sw-h: 14px; --sw-thumb: 10px; }
  :host([size='lg']) { --sw-w: 32px; --sw-h: 18px; --sw-thumb: 14px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .track {
    position: relative;
    display: inline-block;
    flex-shrink: 0;
    width: var(--sw-w);
    height: var(--sw-h);
    background: var(--muted);
    border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
    border-radius: var(--radius-full, 9999px);
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked])) label:hover .track {
    background: color-mix(in oklch, var(--muted) 70%, var(--foreground) 6%);
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
  }

  :host([checked]) .track {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent);
  }
  :host([checked]:not([disabled])) label:hover .track {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 26%, transparent),
      var(--shadow-xs);
  }

  .thumb {
    position: absolute;
    top: 50%;
    left: 1px;
    width: var(--sw-thumb);
    height: var(--sw-thumb);
    background: var(--background);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translateY(-50%);
    transition:
      left var(--duration-fast, 150ms) var(--ease-out, ease-out),
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]) .thumb {
    left: calc(100% - var(--sw-thumb) - 3px);
    background: var(--tone-on-fg);
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    opacity: 0;
    cursor: inherit;
    appearance: none;
    -webkit-appearance: none;
  }

  :host(:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([checked]:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    :host(:not([disabled])) label:has(input:focus-visible) .track {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
    .track { border-color: CanvasText; }
    :host([checked]) .track { background: Highlight; }
  }


  :host([disabled]) { color: var(--muted-foreground); }
  :host([disabled]) .track {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    filter: saturate(0.6);
    box-shadow: none;
  }
  :host([disabled][checked]) .track {
    background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
  }
  :host([disabled]) .thumb { box-shadow: var(--shadow-xs); }

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.checked = props.checked;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.value = props.value;
    $scope.label = props.label;
    let __r = null;
    if (sui_switch_controller_default && typeof sui_switch_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_switch_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets26(__root);
    const __e0 = document.createElement("label");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "track");
    __e1.setAttribute("part", "track");
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __e2.setAttribute("type", "checkbox");
    __e2.setAttribute("role", "switch");
    __cleanups.push(bindChecked2(__e2, $scope.checked));
    __cleanups.push(bindAttr16(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr16(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr16(__e2, "value", () => $scope.value()));
    __cleanups.push(bindAttr16(__e2, "aria-checked", () => $scope.checked() ? "true" : "false"));
    __cleanups.push(bindAttr16(__e2, "aria-label", () => $scope.label() || $scope.name() || "toggle"));
    attachRef10($scope, "inputEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "thumb");
    __e3.setAttribute("part", "thumb");
    __e1.appendChild(__e3);
    __e0.appendChild(__e1);
    const __e4 = document.createElement("slot");
    __e0.appendChild(__e4);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-textarea/sui-textarea.sui
import { adoptSuiSheets as adoptSuiSheets27, attachRef as attachRef11, bindAttr as bindAttr17, bindValue as bindValue2, component as component27, effect as effect15, prop as prop27 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-textarea/sui-textarea.controller.ts
import { controller as controller15, effect as effect14 } from "@ship/sui-runtime";
var sui_textarea_controller_default = controller15({
  mount(state, host) {
    const scope = state;
    let blurValue = scope.value();
    const stopListeners = effect14(() => {
      const el = scope.textareaEl();
      if (!el)
        return;
      const onNativeInput = () => {
        host.dispatchEvent(new CustomEvent("sui-input", {
          detail: { value: el.value },
          bubbles: true,
          composed: true
        }));
      };
      const onNativeBlur = () => {
        const current = scope.value();
        if (current !== blurValue) {
          blurValue = current;
          host.dispatchEvent(new CustomEvent("sui-change", {
            detail: { value: current },
            bubbles: true,
            composed: true
          }));
        }
      };
      el.addEventListener("input", onNativeInput);
      el.addEventListener("blur", onNativeBlur);
      return () => {
        el.removeEventListener("input", onNativeInput);
        el.removeEventListener("blur", onNativeBlur);
      };
    });
    return () => stopListeners();
  }
});

// packages/ui-components/src/sui-textarea/sui-textarea.sui
var sui_textarea_default = component27("sui-textarea", {
  props: {
    value: prop27.string({ default: "" }),
    placeholder: prop27.string({ default: "" }),
    name: prop27.string({ default: "" }),
    rows: prop27.number({ default: 4 }),
    required: prop27.bool({ default: false, reflect: true }),
    disabled: prop27.bool({ default: false, reflect: true }),
    readonly: prop27.bool({ default: false, reflect: true }),
    invalid: prop27.bool({ default: false, reflect: true }),
    tone: prop27.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop27.union(["default", "code"], { default: "default", reflect: true }),
    size: prop27.union(["sm", "md"], { default: "md", reflect: true }),
    resize: prop27.union(["none", "vertical", "horizontal", "both"], { default: "vertical" }),
    label: prop27.string({ default: "" }),
    "described-by": prop27.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }
  :host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --ta-px: 12px;
    --ta-py: 10px;
    --ta-fs: var(--text-base, 14px);
    --ta-lh: 1.5;
    --ta-radius: var(--radius-md, 6px);
  }
  :host([size='sm']) {
    --ta-px: 10px;
    --ta-py: 8px;
    --ta-fs: var(--text-sm, 13px);
    --ta-lh: 1.45;
    --ta-radius: var(--radius-sm, 4px);
  }

  textarea {
    display: block;
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--border);
    border-radius: var(--ta-radius);
    padding: var(--ta-py) var(--ta-px);
    background: var(--background);
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--ta-fs);
    line-height: var(--ta-lh);
    letter-spacing: -0.005em;
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled])) textarea:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  textarea:focus-visible,
  textarea:focus {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  :host([invalid]) textarea {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) textarea:hover {
    border-color: color-mix(in oklch, var(--destructive) 80%, var(--border));
  }
  :host([invalid]) textarea:focus-visible,
  :host([invalid]) textarea:focus {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
  }

  textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 1;
  }
  textarea:disabled {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    color: var(--muted-foreground);
    box-shadow: none;
    opacity: 0.6;
  }
  textarea:read-only { cursor: default; }

  :host([variant='code']) textarea {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-base, 14px);
    line-height: 1.55;
    letter-spacing: 0;
    tab-size: 2;
  }

  @media (forced-colors: active) {
    textarea:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.placeholder = props.placeholder;
    $scope.name = props.name;
    $scope.rows = props.rows;
    $scope.required = props.required;
    $scope.disabled = props.disabled;
    $scope.readonly = props.readonly;
    $scope.invalid = props.invalid;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.resize = props.resize;
    $scope.label = props.label;
    $scope.describedBy = props.describedBy;
    let __r = null;
    if (sui_textarea_controller_default && typeof sui_textarea_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_textarea_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets27(__root);
    const __e0 = document.createElement("textarea");
    __e0.setAttribute("part", "textarea");
    __cleanups.push(bindAttr17(__e0, "placeholder", () => $scope.placeholder()));
    __cleanups.push(bindAttr17(__e0, "name", () => $scope.name()));
    __cleanups.push(bindAttr17(__e0, "rows", () => $scope.rows()));
    __cleanups.push(bindAttr17(__e0, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr17(__e0, "required", () => $scope.required()));
    __cleanups.push(bindAttr17(__e0, "readonly", () => $scope.readonly()));
    __cleanups.push(bindAttr17(__e0, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr17(__e0, "aria-required", () => $scope.required() ? "true" : "false"));
    __cleanups.push(bindAttr17(__e0, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr17(__e0, "aria-describedby", () => $scope.describedBy()));
    __cleanups.push(effect15(() => {
      const __v = $scope.resize();
      if (__v == null)
        __e0.style.removeProperty("resize");
      else
        __e0.style.setProperty("resize", String(__v));
    }));
    __cleanups.push(bindValue2(__e0, $scope.value));
    attachRef11($scope, "textareaEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-field/sui-field.sui
import { adoptSuiSheets as adoptSuiSheets28, attachRef as attachRef12, component as component28, prop as prop28 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-field/sui-field.controller.ts
import { controller as controller16, effect as effect16 } from "@ship/sui-runtime";
var _idSeq2 = 0;
var sui_field_controller_default = controller16({
  mount(state, host) {
    const scope = state;
    const hintId = `sui-field-hint-${++_idSeq2}`;
    const errorId = `sui-field-error-${++_idSeq2}`;
    const stopIds = effect16(() => {
      const hint = scope.hintEl();
      const error = scope.errorEl();
      if (hint)
        hint.id = hintId;
      if (error)
        error.id = errorId;
    });
    const wireControl = () => {
      const slot = scope.controlSlot();
      const first = slot?.assignedElements({ flatten: true })[0];
      if (!first)
        return;
      const inv = scope.invalid();
      const targetId = inv ? errorId : hintId;
      const prior = (first.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== hintId && id !== errorId);
      const final = [...prior, targetId].join(" ").trim();
      if (final)
        first.setAttribute("aria-describedby", final);
      if ("invalid" in first)
        first.invalid = inv;
      else
        first.setAttribute("aria-invalid", inv ? "true" : "false");
    };
    const stopInvalid = effect16(() => {
      scope.invalid();
      wireControl();
    });
    const stopSlot = effect16(() => {
      const slot = scope.controlSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", wireControl);
      return () => slot.removeEventListener("slotchange", wireControl);
    });
    return () => {
      stopIds();
      stopInvalid();
      stopSlot();
    };
  }
});

// packages/ui-components/src/sui-field/sui-field.sui
var sui_field_default = component28("sui-field", {
  props: {
    invalid: prop28.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hint,
  .error {
    font-family: var(--font-body);
    font-size: 11px;
    line-height: 1.4;
  }
  .hint { color: var(--muted-foreground); }
  .error { color: var(--destructive); }

  :host(:not([invalid])) .error { display: none; }
  :host([invalid]) .hint { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.invalid = props.invalid;
    let __r = null;
    if (sui_field_controller_default && typeof sui_field_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_field_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets28(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "label");
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    attachRef12($scope, "controlSlot", __e1);
    __root.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "hint");
    attachRef12($scope, "hintEl", __e2);
    const __e3 = document.createElement("slot");
    __e3.setAttribute("name", "hint");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "error");
    __e4.setAttribute("role", "alert");
    __e4.setAttribute("aria-live", "polite");
    attachRef12($scope, "errorEl", __e4);
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "error");
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
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

// packages/ui-components/src/sui-toast/sui-toast.sui
import { adoptSuiSheets as adoptSuiSheets29, attachRef as attachRef13, bindAttr as bindAttr18, bindIf as bindIf8, component as component29, prop as prop29 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-toast/sui-toast.controller.ts
import { controller as controller17, effect as effect17, ref, signal as signal3 } from "@ship/sui-runtime";
var TONES_WITH_AUTO_ICON = new Set(["info", "success", "warning", "destructive", "accent"]);
var sui_toast_controller_default = controller17({
  scope: () => {
    const hasSlottedIcon = signal3(false);
    const iconSlotEl = ref(null);
    const titleSlotEl = ref(null);
    const actionsSlotEl = ref(null);
    const closeBtn = ref(null);
    return { hasSlottedIcon, iconSlotEl, titleSlotEl, actionsSlotEl, closeBtn };
  },
  mount(state, host) {
    const scope = state;
    let timer = null;
    let remaining = 0;
    let startedAt = 0;
    let removed = false;
    if (!host.hasAttribute("tabindex"))
      host.tabIndex = 0;
    function clearTimer() {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    }
    function scheduleDismiss() {
      clearTimer();
      const dur = scope.duration();
      if (dur > 0) {
        remaining = dur;
        startedAt = Date.now();
        timer = setTimeout(() => dismiss("timeout"), dur);
        host.style.setProperty("--toast-duration", `${dur}ms`);
      }
    }
    function dismiss(reason = "manual") {
      if (removed)
        return;
      removed = true;
      clearTimer();
      host.dispatchEvent(new CustomEvent("sui-dismiss", {
        detail: { reason },
        bubbles: true,
        composed: true
      }));
      host.remove();
    }
    async function close(reason = "manual") {
      if (removed)
        return;
      const prefersReduce = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
      if (prefersReduce) {
        dismiss(reason);
        return;
      }
      clearTimer();
      host.setAttribute("closing", "");
      await new Promise((resolve) => {
        const onEnd = () => {
          host.removeEventListener("animationend", onEnd);
          resolve();
        };
        host.addEventListener("animationend", onEnd);
        setTimeout(() => resolve(), 260);
      });
      dismiss(reason);
    }
    const onPause = () => {
      if (!timer)
        return;
      const elapsed = Date.now() - startedAt;
      remaining = Math.max(0, remaining - elapsed);
      clearTimer();
      host.toggleAttribute("data-paused", true);
    };
    const onResume = () => {
      if (removed || scope.duration() <= 0)
        return;
      if (remaining <= 0)
        return;
      host.toggleAttribute("data-paused", false);
      startedAt = Date.now();
      timer = setTimeout(() => dismiss("timeout"), remaining);
    };
    const onKeydown = (e) => {
      if (e.key === "Escape" && scope.dismissible()) {
        e.stopPropagation();
        dismiss("manual");
      }
    };
    host.addEventListener("mouseenter", onPause);
    host.addEventListener("mouseleave", onResume);
    host.addEventListener("focusin", onPause);
    host.addEventListener("focusout", onResume);
    host.addEventListener("keydown", onKeydown);
    scheduleDismiss();
    const hasSlottedIconSig = scope.hasSlottedIcon;
    const slotIsFilled = (slot) => slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.ELEMENT_NODE)
        return true;
      return (n.textContent?.trim().length ?? 0) > 0;
    });
    const iconSlotEl = scope.iconSlotEl();
    if (iconSlotEl) {
      const onIconSlotChange = () => hasSlottedIconSig.set(slotIsFilled(iconSlotEl));
      iconSlotEl.addEventListener("slotchange", onIconSlotChange);
      hasSlottedIconSig.set(slotIsFilled(iconSlotEl));
    }
    const titleSlotEl = scope.titleSlotEl();
    const actionsSlotEl = scope.actionsSlotEl();
    if (titleSlotEl) {
      const onTitleChange = () => host.toggleAttribute("data-no-title", !slotIsFilled(titleSlotEl));
      titleSlotEl.addEventListener("slotchange", onTitleChange);
      host.toggleAttribute("data-no-title", !slotIsFilled(titleSlotEl));
    }
    if (actionsSlotEl) {
      const onActionsChange = () => host.toggleAttribute("data-no-actions", !slotIsFilled(actionsSlotEl));
      actionsSlotEl.addEventListener("slotchange", onActionsChange);
      host.toggleAttribute("data-no-actions", !slotIsFilled(actionsSlotEl));
    }
    const stopIconAttrEffect = effect17(() => {
      const tone = scope.tone();
      const slotted = scope.hasSlottedIcon();
      const hasAuto = TONES_WITH_AUTO_ICON.has(tone);
      host.toggleAttribute("data-no-icon", !hasAuto && !slotted);
    });
    const stopCloseEffect = effect17(() => {
      const btn = scope.closeBtn();
      if (!btn)
        return;
      const onClick = () => dismiss("manual");
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    const h = host;
    h.dismiss = dismiss;
    h.close = close;
    return () => {
      removed = true;
      clearTimer();
      host.removeEventListener("mouseenter", onPause);
      host.removeEventListener("mouseleave", onResume);
      host.removeEventListener("focusin", onPause);
      host.removeEventListener("focusout", onResume);
      host.removeEventListener("keydown", onKeydown);
      stopIconAttrEffect();
      stopCloseEffect();
    };
  }
});

// packages/ui-components/src/sui-toast/sui-toast.sui
var sui_toast_default = component29("sui-toast", {
  props: {
    tone: prop29.union(["default", "info", "success", "warning", "destructive", "accent", "muted", "primary"], { default: "default", reflect: true }),
    size: prop29.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    duration: prop29.number({ default: 5000 }),
    dismissible: prop29.bool({ default: true, reflect: true }),
    "show-progress": prop29.bool({ default: false, reflect: true }),
    closing: prop29.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --toast-px: 14px;
    --toast-py: 12px;
    --toast-fs: var(--text-base, 14px);
    --toast-title-fs: var(--text-base, 14px);
    --toast-gap: 10px;
    --toast-min-w: 280px;
    --toast-max-w: 420px;
    --toast-slide-x: 16px;
    --toast-slide-y: 0;
    /* tone channel defaults */
    --tone-bg: transparent;
    --tone-border: var(--border, #e5e5e5);
    pointer-events: auto;
    will-change: transform, opacity;
  }

  :host([size='sm']) {
    --toast-px: 12px;
    --toast-py: 10px;
    --toast-fs: var(--text-sm, 12px);
    --toast-title-fs: var(--text-sm, 12px);
    --toast-gap: 8px;
    --toast-min-w: 240px;
    --toast-max-w: 360px;
  }
  :host([size='lg']) {
    --toast-px: 16px;
    --toast-py: 14px;
    --toast-fs: var(--text-md, 16px);
    --toast-title-fs: var(--text-md, 16px);
    --toast-gap: 12px;
    --toast-min-w: 320px;
    --toast-max-w: 460px;
  }

  /* Tone channel — bg + border per semantic tone */
  :host([tone='info'])        { --tone-bg: var(--tone-info-bg, color-mix(in oklch, var(--info, #3b82f6) 14%, transparent));        --tone-border: var(--tone-info-border, color-mix(in oklch, var(--info, #3b82f6) 30%, transparent)); }
  :host([tone='success'])     { --tone-bg: var(--tone-success-bg, color-mix(in oklch, var(--success, #22c55e) 14%, transparent));   --tone-border: var(--tone-success-border, color-mix(in oklch, var(--success, #22c55e) 30%, transparent)); }
  :host([tone='warning'])     { --tone-bg: var(--tone-warning-bg, color-mix(in oklch, var(--warning, #f59e0b) 14%, transparent));   --tone-border: var(--tone-warning-border, color-mix(in oklch, var(--warning, #f59e0b) 30%, transparent)); }
  :host([tone='destructive']) { --tone-bg: var(--tone-destructive-bg, color-mix(in oklch, var(--destructive, #ef4444) 14%, transparent)); --tone-border: var(--tone-destructive-border, color-mix(in oklch, var(--destructive, #ef4444) 30%, transparent)); }
  :host([tone='accent'])      { --tone-bg: var(--tone-accent-bg, color-mix(in oklch, var(--accent, #a855f7) 14%, transparent));     --tone-border: var(--tone-accent-border, color-mix(in oklch, var(--accent, #a855f7) 30%, transparent)); }

  :host([data-slide-from='right'])  { --toast-slide-x: 16px;  --toast-slide-y: 0; }
  :host([data-slide-from='left'])   { --toast-slide-x: -16px; --toast-slide-y: 0; }
  :host([data-slide-from='top'])    { --toast-slide-x: 0;     --toast-slide-y: -12px; }
  :host([data-slide-from='bottom']) { --toast-slide-x: 0;     --toast-slide-y: 12px; }

  @keyframes sui-toast-in {
    from {
      opacity: 0;
      transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translate3d(0, 0, 0) scale(1);
    }
  }

  .toast {
    display: grid;
    grid-template-columns: auto 1fr auto;
    column-gap: var(--toast-gap);
    row-gap: 4px;
    align-items: start;
    padding: var(--toast-py) var(--toast-px);
    padding-right: calc(var(--toast-px) + 24px);
    min-width: var(--toast-min-w);
    max-width: var(--toast-max-w);
    border-radius: var(--radius-md, 6px);
    border: 1px solid var(--tone-border);
    background: color-mix(in oklch, var(--tone-bg) 1, var(--card, #fff));
    color: var(--card-foreground, #111);
    box-shadow:
      0 0 0 1px color-mix(in oklch, black 3%, transparent),
      var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    font-family: var(--font-body, system-ui);
    font-size: var(--toast-fs);
    line-height: var(--leading-normal, 1.5);
    position: relative;
    overflow: hidden;
    isolation: isolate;
    animation: sui-toast-in var(--duration-medium, 200ms) var(--ease-out, ease-out) both;
  }

  :host(:not([tone])) .toast,
  :host([tone='default']) .toast {
    border-color: color-mix(in oklch, var(--border, #e5e5e5) 160%, transparent);
    background: var(--card, #fff);
  }

  :host([tone='info']) .toast,
  :host([tone='success']) .toast,
  :host([tone='warning']) .toast,
  :host([tone='destructive']) .toast,
  :host([tone='accent']) .toast {
    background: color-mix(in oklch, var(--tone-bg) 70%, var(--card, #fff));
    border-color: var(--tone-border);
  }

  .toast::before {
    content: '';
    position: absolute;
    inset-inline-start: 0;
    top: 0; bottom: 0;
    width: 3px;
    background: var(--tone-fg, var(--foreground, #111));
    opacity: 0;
    transition: opacity var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  :host([tone='info']) .toast::before,
  :host([tone='success']) .toast::before,
  :host([tone='warning']) .toast::before,
  :host([tone='destructive']) .toast::before,
  :host([tone='accent']) .toast::before { opacity: 1; }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex: 0 0 auto;
    color: var(--tone-fg, var(--muted-foreground, #888));
    margin-top: 1px;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon { color: var(--muted-foreground, #888); }
  .icon svg { width: 18px; height: 18px; display: block; }
  :host([size='sm']) .icon,
  :host([size='sm']) .icon svg { width: 16px; height: 16px; }
  :host([size='lg']) .icon,
  :host([size='lg']) .icon svg { width: 20px; height: 20px; }

  :host([data-no-icon]) .toast { grid-template-columns: 1fr auto; }
  :host([data-no-icon]) .icon { display: none; }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    color: var(--card-foreground, #111);
  }
  .title {
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--toast-title-fs);
    letter-spacing: -0.005em;
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .body {
    font-size: var(--toast-fs);
    color: var(--muted-foreground, #888);
    line-height: var(--leading-normal, 1.5);
    word-break: break-word;
  }
  :host([data-no-title]) .body { color: var(--card-foreground, #111); }
  :host([data-no-title]) .title { display: none; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    margin-top: 6px;
    grid-column: 2;
  }
  :host([data-no-actions]) .actions { display: none; margin-top: 0; }

  .close {
    position: absolute;
    top: 8px;
    inset-inline-end: 8px;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    color: var(--muted-foreground, #888);
    cursor: pointer;
    padding: 0;
    font: inherit;
    z-index: 1;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .close:hover {
    background: var(--active-bg, color-mix(in oklch, currentColor 8%, transparent));
    color: var(--foreground, #111);
  }
  .close:active { transform: scale(0.94); }
  .close:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--card, #fff),
      0 0 0 4px color-mix(in oklch, var(--tone-fg, var(--primary, #3b82f6)) 55%, transparent);
  }
  .close svg { width: 12px; height: 12px; display: block; }

  .progress {
    position: absolute;
    left: 0; right: 0; bottom: 0;
    height: 2px;
    background: var(--tone-fg, var(--muted-foreground, #888));
    transform: scaleX(1);
    transform-origin: left center;
    animation: sui-toast-progress var(--toast-duration, 5000ms) linear forwards;
  }
  :host(:not([tone])) .progress,
  :host([tone='default']) .progress { background: var(--muted-foreground, #888); }
  :host(:hover) .progress,
  :host([data-paused]) .progress { animation-play-state: paused; }
  @keyframes sui-toast-progress {
    from { transform: scaleX(1); }
    to   { transform: scaleX(0); }
  }

  :host([closing]) {
    animation: sui-toast-out var(--duration-medium, 200ms) var(--ease-in, ease-in) forwards;
  }
  @keyframes sui-toast-out {
    to {
      opacity: 0;
      transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0) scale(0.96);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast { animation: none; }
    :host([closing]) { animation: none; }
    .progress { animation: none; }
  }

  @media (forced-colors: active) {
    .toast { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.duration = props.duration;
    $scope.dismissible = props.dismissible;
    $scope.showProgress = props.showProgress;
    $scope.closing = props.closing;
    let __r = null;
    if (sui_toast_controller_default && typeof sui_toast_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toast_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b4 = (anchor) => bindIf8({
      anchor,
      test: () => {
        const __d = $scope.tone();
        if (__d === "info")
          return 0;
        if (__d === "success")
          return 1;
        if (__d === "warning")
          return 2;
        if (__d === "destructive")
          return 3;
        if (__d === "accent")
          return 4;
        return -1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("svg");
          __e9.setAttribute("viewbox", "0 0 24 24");
          __e9.setAttribute("fill", "none");
          __e9.setAttribute("stroke", "currentColor");
          __e9.setAttribute("stroke-width", "2");
          __e9.setAttribute("stroke-linecap", "round");
          __e9.setAttribute("stroke-linejoin", "round");
          __e9.setAttribute("aria-hidden", "true");
          const __e10 = document.createElement("circle");
          __e10.setAttribute("cx", "12");
          __e10.setAttribute("cy", "12");
          __e10.setAttribute("r", "9");
          __e9.appendChild(__e10);
          const __e11 = document.createElement("path");
          __e11.setAttribute("d", "M12 8h.01");
          __e9.appendChild(__e11);
          const __e12 = document.createElement("path");
          __e12.setAttribute("d", "M11 12h1v4h1");
          __e9.appendChild(__e12);
          __frag7.appendChild(__e9);
          return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
            for (const c of __c8)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("svg");
          __e15.setAttribute("viewbox", "0 0 24 24");
          __e15.setAttribute("fill", "none");
          __e15.setAttribute("stroke", "currentColor");
          __e15.setAttribute("stroke-width", "2");
          __e15.setAttribute("stroke-linecap", "round");
          __e15.setAttribute("stroke-linejoin", "round");
          __e15.setAttribute("aria-hidden", "true");
          const __e16 = document.createElement("circle");
          __e16.setAttribute("cx", "12");
          __e16.setAttribute("cy", "12");
          __e16.setAttribute("r", "9");
          __e15.appendChild(__e16);
          const __e17 = document.createElement("path");
          __e17.setAttribute("d", "m8.5 12.5 2.5 2.5 4.5-5");
          __e15.appendChild(__e17);
          __frag13.appendChild(__e15);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("svg");
          __e20.setAttribute("viewbox", "0 0 24 24");
          __e20.setAttribute("fill", "none");
          __e20.setAttribute("stroke", "currentColor");
          __e20.setAttribute("stroke-width", "2");
          __e20.setAttribute("stroke-linecap", "round");
          __e20.setAttribute("stroke-linejoin", "round");
          __e20.setAttribute("aria-hidden", "true");
          const __e21 = document.createElement("path");
          __e21.setAttribute("d", "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z");
          __e20.appendChild(__e21);
          const __e22 = document.createElement("line");
          __e22.setAttribute("x1", "12");
          __e22.setAttribute("x2", "12");
          __e22.setAttribute("y1", "9");
          __e22.setAttribute("y2", "13");
          __e20.appendChild(__e22);
          const __e23 = document.createElement("line");
          __e23.setAttribute("x1", "12");
          __e23.setAttribute("x2", "12.01");
          __e23.setAttribute("y1", "17");
          __e23.setAttribute("y2", "17");
          __e20.appendChild(__e23);
          __frag18.appendChild(__e20);
          return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
            for (const c of __c19)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag24 = document.createDocumentFragment();
          const __c25 = [];
          const __e26 = document.createElement("svg");
          __e26.setAttribute("viewbox", "0 0 24 24");
          __e26.setAttribute("fill", "none");
          __e26.setAttribute("stroke", "currentColor");
          __e26.setAttribute("stroke-width", "2");
          __e26.setAttribute("stroke-linecap", "round");
          __e26.setAttribute("stroke-linejoin", "round");
          __e26.setAttribute("aria-hidden", "true");
          const __e27 = document.createElement("circle");
          __e27.setAttribute("cx", "12");
          __e27.setAttribute("cy", "12");
          __e27.setAttribute("r", "9");
          __e26.appendChild(__e27);
          const __e28 = document.createElement("line");
          __e28.setAttribute("x1", "15");
          __e28.setAttribute("x2", "9");
          __e28.setAttribute("y1", "9");
          __e28.setAttribute("y2", "15");
          __e26.appendChild(__e28);
          const __e29 = document.createElement("line");
          __e29.setAttribute("x1", "9");
          __e29.setAttribute("x2", "15");
          __e29.setAttribute("y1", "9");
          __e29.setAttribute("y2", "15");
          __e26.appendChild(__e29);
          __frag24.appendChild(__e26);
          return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
            for (const c of __c25)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag30 = document.createDocumentFragment();
          const __c31 = [];
          const __e32 = document.createElement("svg");
          __e32.setAttribute("viewbox", "0 0 24 24");
          __e32.setAttribute("fill", "none");
          __e32.setAttribute("stroke", "currentColor");
          __e32.setAttribute("stroke-width", "2");
          __e32.setAttribute("stroke-linecap", "round");
          __e32.setAttribute("stroke-linejoin", "round");
          __e32.setAttribute("aria-hidden", "true");
          const __e33 = document.createElement("path");
          __e33.setAttribute("d", "M12 2 3 14h7l-1 8 10-12h-7l1-8Z");
          __e32.appendChild(__e33);
          __frag30.appendChild(__e32);
          return { nodes: Array.from(__frag30.childNodes), cleanup: () => {
            for (const c of __c31)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b3 = (anchor) => bindIf8({
      anchor,
      test: () => {
        if (!$scope.hasSlottedIcon())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __matchAnchor6 = document.createComment("match");
          __frag4.appendChild(__matchAnchor6);
          __c5.push(__wireBoundary_b4(__matchAnchor6));
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b7 = (anchor) => bindIf8({
      anchor,
      test: () => {
        if ($scope.dismissible())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag42 = document.createDocumentFragment();
          const __c43 = [];
          const __e44 = document.createElement("button");
          __e44.setAttribute("class", "close");
          __e44.setAttribute("part", "close");
          __e44.setAttribute("type", "button");
          __e44.setAttribute("aria-label", "Dismiss notification");
          attachRef13($scope, "closeBtn", __e44);
          const __e45 = document.createElement("svg");
          __e45.setAttribute("viewbox", "0 0 24 24");
          __e45.setAttribute("fill", "none");
          __e45.setAttribute("stroke", "currentColor");
          __e45.setAttribute("stroke-width", "2");
          __e45.setAttribute("stroke-linecap", "round");
          __e45.setAttribute("stroke-linejoin", "round");
          __e45.setAttribute("aria-hidden", "true");
          const __e46 = document.createElement("line");
          __e46.setAttribute("x1", "18");
          __e46.setAttribute("y1", "6");
          __e46.setAttribute("x2", "6");
          __e46.setAttribute("y2", "18");
          __e45.appendChild(__e46);
          const __e47 = document.createElement("line");
          __e47.setAttribute("x1", "6");
          __e47.setAttribute("y1", "6");
          __e47.setAttribute("x2", "18");
          __e47.setAttribute("y2", "18");
          __e45.appendChild(__e47);
          __e44.appendChild(__e45);
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
    const __wireBoundary_b9 = (anchor) => bindIf8({
      anchor,
      test: () => {
        if ($scope.showProgress() && $scope.duration() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag49 = document.createDocumentFragment();
          const __c50 = [];
          const __e51 = document.createElement("span");
          __e51.setAttribute("class", "progress");
          __e51.setAttribute("aria-hidden", "true");
          __frag49.appendChild(__e51);
          return { nodes: Array.from(__frag49.childNodes), cleanup: () => {
            for (const c of __c50)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets29(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "toast");
    __cleanups.push(bindAttr18(__e0, "role", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "alert" : "status"));
    __cleanups.push(bindAttr18(__e0, "aria-live", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "assertive" : "polite"));
    __e0.setAttribute("aria-atomic", "true");
    __e0.setAttribute("part", "toast");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "icon");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    attachRef13($scope, "iconSlotEl", __e2);
    __e1.appendChild(__e2);
    const __ifAnchor3 = document.createComment("if");
    __e1.appendChild(__ifAnchor3);
    __cleanups.push(__wireBoundary_b3(__ifAnchor3));
    __e0.appendChild(__e1);
    const __e34 = document.createElement("div");
    __e34.setAttribute("class", "content");
    const __e35 = document.createElement("div");
    __e35.setAttribute("class", "title");
    const __e36 = document.createElement("slot");
    __e36.setAttribute("name", "title");
    attachRef13($scope, "titleSlotEl", __e36);
    __e35.appendChild(__e36);
    __e34.appendChild(__e35);
    const __e37 = document.createElement("div");
    __e37.setAttribute("class", "body");
    const __e38 = document.createElement("slot");
    __e37.appendChild(__e38);
    __e34.appendChild(__e37);
    const __e39 = document.createElement("div");
    __e39.setAttribute("class", "actions");
    const __e40 = document.createElement("slot");
    __e40.setAttribute("name", "actions");
    attachRef13($scope, "actionsSlotEl", __e40);
    __e39.appendChild(__e40);
    __e34.appendChild(__e39);
    __e0.appendChild(__e34);
    const __ifAnchor41 = document.createComment("if");
    __e0.appendChild(__ifAnchor41);
    __cleanups.push(__wireBoundary_b7(__ifAnchor41));
    const __ifAnchor48 = document.createComment("if");
    __e0.appendChild(__ifAnchor48);
    __cleanups.push(__wireBoundary_b9(__ifAnchor48));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-toast/sui-toast-container.sui
import { adoptSuiSheets as adoptSuiSheets30, attachRef as attachRef14, component as component30, prop as prop30 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-toast/sui-toast-container.controller.ts
import { bindHtml, controller as controller18, effect as effect18, signal as signal4 } from "@ship/sui-runtime";
var SLIDE_FROM = {
  "top-left": "top",
  "top-center": "top",
  "top-right": "top",
  "bottom-left": "bottom",
  "bottom-center": "bottom",
  "bottom-right": "bottom"
};
var sui_toast_container_controller_default = controller18({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "region");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "Notifications");
    if (!host.hasAttribute("aria-live"))
      host.setAttribute("aria-live", "polite");
    const slotVersion = signal4(0);
    const defaultSlotEl = scope.defaultSlotEl();
    const getToasts = () => {
      if (defaultSlotEl) {
        return defaultSlotEl.assignedElements().filter((e) => e.tagName.toLowerCase() === "sui-toast");
      }
      return Array.from(host.querySelectorAll(":scope > sui-toast"));
    };
    const stopSlotEffect = effect18(() => {
      slotVersion();
      const pos = scope.position();
      const maxVis = scope.maxVisible();
      const dir = SLIDE_FROM[pos] ?? "bottom";
      const toasts = getToasts();
      for (const t of toasts) {
        t.setAttribute("data-slide-from", dir);
      }
      if (maxVis > 0 && toasts.length > maxVis) {
        const overflow = toasts.length - maxVis;
        for (let i = 0;i < overflow; i++) {
          const victim = toasts[i];
          if (typeof victim.close === "function")
            victim.close("programmatic");
          else
            victim.dismiss?.("programmatic");
        }
      }
    });
    const onSlotChange = () => slotVersion.set(slotVersion() + 1);
    if (defaultSlotEl) {
      defaultSlotEl.addEventListener("slotchange", onSlotChange);
    }
    function show(opts = {}) {
      const doc = host.ownerDocument ?? document;
      const t = doc.createElement("sui-toast");
      if (opts.tone)
        t.setAttribute("tone", opts.tone);
      if (opts.size)
        t.setAttribute("size", opts.size);
      if (typeof opts.duration === "number")
        t.setAttribute("duration", String(opts.duration));
      if (opts.dismissible === false)
        t.removeAttribute("dismissible");
      else
        t.setAttribute("dismissible", "");
      if (opts.showProgress)
        t.setAttribute("show-progress", "");
      if (opts.title) {
        const span = doc.createElement("span");
        span.setAttribute("slot", "title");
        span.textContent = opts.title;
        t.appendChild(span);
      }
      if (opts.description) {
        t.appendChild(doc.createTextNode(opts.description));
      }
      if (opts.actionHtml) {
        const wrap = doc.createElement("span");
        wrap.setAttribute("slot", "actions");
        const html = opts.actionHtml;
        bindHtml(wrap, () => html);
        t.appendChild(wrap);
      }
      host.appendChild(t);
      return t;
    }
    host.show = show;
    return () => {
      stopSlotEffect();
      if (defaultSlotEl) {
        defaultSlotEl.removeEventListener("slotchange", onSlotChange);
      }
    };
  }
});

// packages/ui-components/src/sui-toast/sui-toast-container.sui
var sui_toast_container_default = component30("sui-toast-container", {
  props: {
    position: prop30.union(["top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"], { default: "bottom-right", reflect: true }),
    "max-visible": prop30.number({ default: 5, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: fixed;
    z-index: var(--z-toast, 9999);
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    pointer-events: none;
    max-width: min(440px, calc(100vw - 32px));
    --toast-edge: 16px;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }

  :host(:not([position])),
  :host([position='bottom-right']) {
    bottom: var(--toast-edge);
    right: var(--toast-edge);
    top: auto;
    left: auto;
    flex-direction: column-reverse;
    align-items: flex-end;
  }
  :host([position='bottom-left']) {
    bottom: var(--toast-edge);
    left: var(--toast-edge);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: flex-start;
  }
  :host([position='bottom-center']) {
    bottom: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: center;
  }
  :host([position='top-right']) {
    top: var(--toast-edge);
    right: var(--toast-edge);
    bottom: auto;
    left: auto;
    flex-direction: column;
    align-items: flex-end;
  }
  :host([position='top-left']) {
    top: var(--toast-edge);
    left: var(--toast-edge);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: flex-start;
  }
  :host([position='top-center']) {
    top: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: center;
  }

  ::slotted(sui-toast) {
    pointer-events: auto;
    width: auto;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.position = props.position;
    $scope.maxVisible = props.maxVisible;
    let __r = null;
    if (sui_toast_container_controller_default && typeof sui_toast_container_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toast_container_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets30(__root);
    const __e0 = document.createElement("slot");
    attachRef14($scope, "defaultSlotEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-breadcrumb/sui-breadcrumb.sui
import { adoptSuiSheets as adoptSuiSheets31, bindAttr as bindAttr19, bindFor, bindIf as bindIf9, bindText as bindText4, component as component31, prop as prop31 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-breadcrumb/sui-breadcrumb.controller.ts
import { controller as controller19, signal as signal5 } from "@ship/sui-runtime";
var sui_breadcrumb_controller_default = controller19({
  scope: () => {
    const items = signal5([]);
    const lastSlot = signal5("");
    return { items, lastSlot };
  },
  mount(state, host) {
    const scope = state;
    const items = scope.items;
    const lastSlot = scope.lastSlot;
    const sync = () => {
      const children = Array.from(host.querySelectorAll(":scope > sui-breadcrumb-item"));
      const next = children.map((el, i) => {
        const slot = `i${i}`;
        el.setAttribute("slot", slot);
        return { slot };
      });
      lastSlot.set(next.length > 0 ? next[next.length - 1].slot : "");
      items.set(next);
    };
    sync();
    const observer = new MutationObserver(sync);
    observer.observe(host, { childList: true });
    return () => observer.disconnect();
  }
});

// packages/ui-components/src/sui-breadcrumb/sui-breadcrumb.sui
var sui_breadcrumb_default = component31("sui-breadcrumb", {
  props: {
    separator: prop31.string({ default: "/", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.separator = props.separator;
    let __r = null;
    if (sui_breadcrumb_controller_default && typeof sui_breadcrumb_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_breadcrumb_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindFor({
      anchor,
      items: () => $scope.items(),
      key: (item, __i) => {
        const __child = Object.create($scope);
        __child.item = item;
        __child.__i = __i;
        return (($scope2) => item.slot)(__child);
      },
      render: (item, __i) => {
        const __childScope2 = Object.create($scope);
        __childScope2.item = item;
        __childScope2.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b3 = (anchor2) => bindIf9({
            anchor: anchor2,
            test: () => {
              if ($scope2.separator() === "/")
                return 0;
              return 1;
            },
            branches: [
              () => {
                const __frag11 = document.createDocumentFragment();
                const __c12 = [];
                const __e13 = document.createElement("svg");
                __e13.setAttribute("viewbox", "0 0 24 24");
                __e13.setAttribute("fill", "none");
                __e13.setAttribute("stroke", "currentColor");
                __e13.setAttribute("stroke-width", "2");
                __e13.setAttribute("stroke-linecap", "round");
                __e13.setAttribute("stroke-linejoin", "round");
                const __e14 = document.createElement("path");
                __e14.setAttribute("d", "M9 6l6 6-6 6");
                __e13.appendChild(__e14);
                __frag11.appendChild(__e13);
                return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
                  for (const c of __c12)
                    try {
                      c();
                    } catch (e) {}
                } };
              },
              () => {
                const __frag15 = document.createDocumentFragment();
                const __c16 = [];
                const __t17 = document.createTextNode("");
                __frag15.appendChild(__t17);
                __c16.push(bindText4(__t17, () => $scope2.separator()));
                return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
                  for (const c of __c16)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __wireBoundary_b2 = (anchor2) => bindIf9({
            anchor: anchor2,
            test: () => {
              if ($scope2.item.slot !== $scope2.lastSlot())
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag7 = document.createDocumentFragment();
                const __c8 = [];
                const __e9 = document.createElement("span");
                __e9.setAttribute("class", "sep");
                __e9.setAttribute("aria-hidden", "true");
                const __ifAnchor10 = document.createComment("if");
                __e9.appendChild(__ifAnchor10);
                __c8.push(__wireBoundary_b3(__ifAnchor10));
                __frag7.appendChild(__e9);
                return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
                  for (const c of __c8)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("slot");
          __c4.push(bindAttr19(__e5, "name", () => $scope2.item.slot));
          __frag3.appendChild(__e5);
          const __ifAnchor6 = document.createComment("if");
          __frag3.appendChild(__ifAnchor6);
          __c4.push(__wireBoundary_b2(__ifAnchor6));
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope2);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets31(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("aria-label", "Breadcrumb");
    __e0.setAttribute("part", "nav");
    const __forAnchor1 = document.createComment("for");
    __e0.appendChild(__forAnchor1);
    __cleanups.push(__wireBoundary_b0(__forAnchor1));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-breadcrumb/sui-breadcrumb-item.sui
import { adoptSuiSheets as adoptSuiSheets32, bindAttr as bindAttr20, bindIf as bindIf10, component as component32, prop as prop32 } from "@ship/sui-runtime";
var sui_breadcrumb_item_default = component32("sui-breadcrumb-item", {
  props: {
    href: prop32.string({ default: "", reflect: true }),
    active: prop32.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--muted-foreground);
  }
  :host([active]) {
    color: var(--foreground);
    font-weight: var(--font-weight-medium);
  }
  a, .label {
    display: inline-flex;
    align-items: center;
    color: inherit;
    text-decoration: none;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    transition: color var(--duration-fast);
  }
  a:hover { color: var(--foreground); }
  a:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.href = props.href;
    $scope.active = props.active;
    const __wireBoundary_b0 = (anchor) => bindIf10({
      anchor,
      test: () => {
        if ($scope.href() && !$scope.active())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("a");
          __c2.push(bindAttr20(__e3, "href", () => $scope.href()));
          __e3.setAttribute("part", "link");
          const __e4 = document.createElement("slot");
          __e3.appendChild(__e4);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("span");
          __e7.setAttribute("class", "label");
          __e7.setAttribute("part", "label");
          __c6.push(bindAttr20(__e7, "aria-current", () => $scope.active() ? "page" : "false"));
          const __e8 = document.createElement("slot");
          __e7.appendChild(__e8);
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets32(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
    return $scope;
  }
});

// packages/ui-components/src/sui-pagination/sui-pagination.sui
import { adoptSuiSheets as adoptSuiSheets33, attachRef as attachRef15, bindAttr as bindAttr21, bindText as bindText5, component as component33, prop as prop33 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pagination/sui-pagination.controller.ts
import { controller as controller20, signal as signal6, ref as ref2, effect as effect19 } from "@ship/sui-runtime";
var sui_pagination_controller_default = controller20({
  scope: () => ({
    totalPages: signal6(1),
    info: signal6(""),
    prevBtn: ref2(null),
    nextBtn: ref2(null)
  }),
  mount(state, host) {
    const scope = state;
    const stopEffect = effect19(() => {
      const p = scope.page();
      const t = scope.total();
      const pp = scope.perPage();
      const tp = t > 0 ? Math.ceil(t / pp) : 1;
      scope.totalPages.set(tp);
      if (t > 0) {
        const start = (p - 1) * pp + 1;
        const end = Math.min(p * pp, t);
        scope.info.set(`Page ${p} of ${tp} (${start}–${end} of ${t})`);
      } else {
        scope.info.set(`Page ${p} of ${tp}`);
      }
    });
    const prevBtn = scope.prevBtn();
    const nextBtn = scope.nextBtn();
    const onPrev = () => {
      const p = scope.page();
      if (p > 1) {
        const np = p - 1;
        scope.page.set(np);
        host.dispatchEvent(new CustomEvent("sui-page", {
          detail: { page: np },
          bubbles: true,
          composed: true
        }));
      }
    };
    const onNext = () => {
      const p = scope.page();
      const tp = scope.totalPages();
      if (p < tp) {
        const np = p + 1;
        scope.page.set(np);
        host.dispatchEvent(new CustomEvent("sui-page", {
          detail: { page: np },
          bubbles: true,
          composed: true
        }));
      }
    };
    prevBtn?.addEventListener("click", onPrev);
    nextBtn?.addEventListener("click", onNext);
    return () => {
      stopEffect();
      prevBtn?.removeEventListener("click", onPrev);
      nextBtn?.removeEventListener("click", onNext);
    };
  }
});

// packages/ui-components/src/sui-pagination/sui-pagination.sui
var sui_pagination_default = component33("sui-pagination", {
  props: {
    page: prop33.number({ default: 1, reflect: true }),
    total: prop33.number({ default: 0, reflect: true }),
    "per-page": prop33.number({ default: 25, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
  nav.bar {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 0.8125rem;
    font-family: var(--font-body);
  }
  .info {
    color: var(--muted-foreground);
    user-select: none;
    white-space: nowrap;
  }
  .spacer { flex: 1; }
  .pg-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: 0.8125rem;
    cursor: pointer;
    transition: background-color var(--duration-fast);
  }
  .pg-btn:hover:not(:disabled) {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
  }
  .pg-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .pg-btn:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.page = props.page;
    $scope.total = props.total;
    $scope.perPage = props.perPage;
    let __r = null;
    if (sui_pagination_controller_default && typeof sui_pagination_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pagination_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets33(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("class", "bar");
    __e0.setAttribute("aria-label", "Pagination");
    const __e1 = document.createElement("button");
    attachRef15($scope, "prevBtn", __e1);
    __e1.setAttribute("class", "pg-btn");
    __e1.setAttribute("part", "prev");
    __cleanups.push(bindAttr21(__e1, "disabled", () => $scope.page() <= 1));
    __e1.setAttribute("aria-label", "Previous page");
    __cleanups.push(bindAttr21(__e1, "aria-disabled", () => $scope.page() <= 1 ? "true" : "false"));
    const __t2 = document.createTextNode("Prev");
    __e1.appendChild(__t2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "info");
    __e3.setAttribute("part", "info");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText5(__t4, () => $scope.info()));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "spacer");
    __e0.appendChild(__e5);
    const __e6 = document.createElement("button");
    attachRef15($scope, "nextBtn", __e6);
    __e6.setAttribute("class", "pg-btn");
    __e6.setAttribute("part", "next");
    __cleanups.push(bindAttr21(__e6, "disabled", () => $scope.page() >= $scope.totalPages()));
    __e6.setAttribute("aria-label", "Next page");
    __cleanups.push(bindAttr21(__e6, "aria-disabled", () => $scope.page() >= $scope.totalPages() ? "true" : "false"));
    const __t7 = document.createTextNode("Next");
    __e6.appendChild(__t7);
    __e0.appendChild(__e6);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-timestamp/sui-timestamp.sui
import { adoptSuiSheets as adoptSuiSheets34, bindAttr as bindAttr22, bindIf as bindIf11, bindText as bindText6, component as component34, prop as prop34 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-timestamp/sui-timestamp.controller.ts
import { controller as controller21, signal as signal7, derived, effect as effect20 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-timestamp/sui-timestamp.ts
var MINUTE = 60000;
var HOUR = 3600000;
var DAY = 86400000;
function formatRelative(iso, now = Date.now()) {
  const t = Date.parse(iso);
  if (Number.isNaN(t))
    return iso;
  const diff = now - t;
  const abs = Math.abs(diff);
  const suffix = diff >= 0 ? "ago" : "from now";
  if (abs < MINUTE)
    return "just now";
  if (abs < HOUR)
    return `${Math.floor(abs / MINUTE)}m ${suffix}`;
  if (abs < DAY)
    return `${Math.floor(abs / HOUR)}h ${suffix}`;
  const days = Math.floor(abs / DAY);
  if (days < 30)
    return `${days}d ${suffix}`;
  if (days < 365)
    return `${Math.floor(days / 30)}mo ${suffix}`;
  return `${Math.floor(days / 365)}y ${suffix}`;
}
function absoluteIso(value) {
  const parsed = Date.parse(value);
  return Number.isNaN(parsed) ? value : new Date(parsed).toISOString();
}

// packages/ui-components/src/sui-timestamp/sui-timestamp.controller.ts
var MINUTE2 = 60000;
var sui_timestamp_controller_default = controller21({
  scope: ({ value }) => {
    const tick = signal7(0);
    const absStr = derived(() => {
      const v = value();
      return v ? absoluteIso(v) : "";
    });
    const relStr = derived(() => {
      tick();
      const v = value();
      return v ? formatRelative(v) : "";
    });
    return { tick, absStr, relStr };
  },
  mount(state, host) {
    const scope = state;
    const tick = scope.tick;
    const stopTitle = effect20(() => {
      const v = scope.value();
      if (v)
        host.title = scope.absStr();
    });
    const timer = setInterval(() => tick.set(tick() + 1), MINUTE2);
    return () => {
      stopTitle();
      clearInterval(timer);
    };
  }
});

// packages/ui-components/src/sui-timestamp/sui-timestamp.sui
var sui_timestamp_default = component34("sui-timestamp", {
  props: {
    value: prop34.string({ default: "", reflect: true }),
    format: prop34.union(["relative", "absolute", "both"], { default: "relative", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.format = props.format;
    let __r = null;
    if (sui_timestamp_controller_default && typeof sui_timestamp_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_timestamp_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.format() === "absolute")
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __t7 = document.createTextNode("");
          __frag5.appendChild(__t7);
          __c6.push(bindText6(__t7, () => $scope.absStr()));
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag8 = document.createDocumentFragment();
          const __c9 = [];
          const __t10 = document.createTextNode("");
          __frag8.appendChild(__t10);
          __c9.push(bindText6(__t10, () => $scope.relStr()));
          return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
            for (const c of __c9)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b5 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.format() === "both")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("span");
          __e14.setAttribute("class", "abs");
          __e14.setAttribute("part", "abs");
          const __t15 = document.createTextNode("(");
          __e14.appendChild(__t15);
          const __t16 = document.createTextNode("");
          __e14.appendChild(__t16);
          __c13.push(bindText6(__t16, () => $scope.absStr()));
          const __t17 = document.createTextNode(")");
          __e14.appendChild(__t17);
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
    const __wireBoundary_b0 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.value())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("time");
          __c2.push(bindAttr22(__e3, "datetime", () => $scope.value()));
          __e3.setAttribute("part", "time");
          const __ifAnchor4 = document.createComment("if");
          __e3.appendChild(__ifAnchor4);
          __c2.push(__wireBoundary_b2(__ifAnchor4));
          __frag1.appendChild(__e3);
          const __ifAnchor11 = document.createComment("if");
          __frag1.appendChild(__ifAnchor11);
          __c2.push(__wireBoundary_b5(__ifAnchor11));
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets34(__root);
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

// packages/ui-components/src/sui-metachip/sui-metachip.sui
import { adoptSuiSheets as adoptSuiSheets35, bindText as bindText7, component as component35, prop as prop35 } from "@ship/sui-runtime";
var sui_metachip_default = component35("sui-metachip", {
  props: {
    label: prop35.string({ default: "", reflect: true }),
    value: prop35.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    padding: 4px 14px;
    border-right: 1px solid color-mix(in oklch, var(--border) 70%, transparent);
    font-size: var(--text-xs);
    line-height: var(--leading-tight);
  }
  :host(:last-of-type) { border-right: 0; padding-right: 0; }
  :host(:first-of-type) { padding-left: 0; }
  .k {
    font-size: 10px;
    font-weight: var(--font-weight-semibold);
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .v {
    font-family: var(--font-mono);
    color: var(--foreground);
    font-size: var(--text-xs);
  }
  ::slotted(*) { color: inherit; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.label = props.label;
    $scope.value = props.value;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets35(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "k");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "label");
    const __t2 = document.createTextNode("");
    __e1.appendChild(__t2);
    __cleanups.push(bindText7(__t2, () => $scope.label()));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "v");
    __e3.setAttribute("part", "value");
    const __e4 = document.createElement("slot");
    const __t5 = document.createTextNode("");
    __e4.appendChild(__t5);
    __cleanups.push(bindText7(__t5, () => $scope.value()));
    __e3.appendChild(__e4);
    __root.appendChild(__e3);
    return $scope;
  }
});

// packages/ui-components/src/sui-popover/sui-popover.sui
import { adoptSuiSheets as adoptSuiSheets36, attachRef as attachRef16, component as component36, prop as prop36 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-popover/sui-popover.controller.ts
import { controller as controller22, effect as effect21, bindNativeEvent as bindNativeEvent2, slotted } from "@ship/sui-runtime";
var anchorSeq = 0;
var anchorSupported = typeof CSS !== "undefined" && typeof CSS.supports === "function" && CSS.supports("anchor-name", "--x");
var sui_popover_controller_default = controller22({
  mount(state, host) {
    const scope = state;
    const id = ++anchorSeq;
    const anchorName = `--sui-popover-${id}`;
    const panelId = `sui-popover-panel-${id}`;
    host.style.setProperty("--sui-popover-anchor", anchorName);
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    slotted(scope.triggerSlot, (els) => {
      for (const el of els) {
        el.setAttribute("popovertarget", panelId);
        el.setAttribute("aria-haspopup", "dialog");
        el.setAttribute("aria-controls", panelId);
        el.setAttribute("aria-expanded", scope.open() ? "true" : "false");
      }
    });
    function onToggle(ev) {
      const newState = ev.newState;
      const nowOpen = newState === "open";
      const hasAttr = host.hasAttribute("open");
      if (nowOpen && !hasAttr)
        host.setAttribute("open", "");
      else if (!nowOpen && hasAttr)
        host.removeAttribute("open");
      const slot = triggerSlot;
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-expanded", nowOpen ? "true" : "false");
      }
      host.emit("sui-toggle", { open: nowOpen });
      if (nowOpen && !anchorSupported)
        positionFallback();
    }
    bindNativeEvent2(panelEl, "toggle", onToggle);
    host.expose({
      show: () => panelEl.showPopover?.(),
      hide: () => panelEl.hidePopover?.(),
      toggle: () => typeof panelEl.togglePopover === "function" ? void panelEl.togglePopover() : panelEl.matches?.(":popover-open") ? panelEl.hidePopover?.() : panelEl.showPopover?.()
    });
    const stopOpenSync = effect21(() => {
      const wantOpen = scope.open();
      const isOpen = panelEl.matches?.(":popover-open") ?? false;
      if (wantOpen && !isOpen)
        panelEl.showPopover?.();
      else if (!wantOpen && isOpen)
        panelEl.hidePopover?.();
    });
    function positionFallback() {
      const slot = triggerSlot;
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      const trigger = els[0];
      if (!trigger || !panelEl)
        return;
      const r = trigger.getBoundingClientRect();
      const placement = scope.placement();
      panelEl.style.top = "";
      panelEl.style.left = "";
      panelEl.style.right = "";
      panelEl.style.bottom = "";
      const gap = 6;
      switch (placement) {
        case "top":
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`;
          panelEl.style.left = `${r.left}px`;
          break;
        case "top-end":
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`;
          panelEl.style.right = `${window.innerWidth - r.right}px`;
          break;
        case "bottom-end":
          panelEl.style.top = `${r.bottom + gap}px`;
          panelEl.style.right = `${window.innerWidth - r.right}px`;
          break;
        case "left":
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`;
          panelEl.style.top = `${r.top}px`;
          break;
        case "left-end":
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`;
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`;
          break;
        case "right":
          panelEl.style.left = `${r.right + gap}px`;
          panelEl.style.top = `${r.top}px`;
          break;
        case "right-end":
          panelEl.style.left = `${r.right + gap}px`;
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`;
          break;
        case "bottom":
        default:
          panelEl.style.top = `${r.bottom + gap}px`;
          panelEl.style.left = `${r.left}px`;
      }
    }
    function onWindowChange() {
      if (!scope.open())
        return;
      positionFallback();
    }
    if (!anchorSupported) {
      bindNativeEvent2(window, "resize", onWindowChange);
      bindNativeEvent2(window, "scroll", onWindowChange, { capture: true, passive: true });
    }
    return () => {
      stopOpenSync();
    };
  }
});

// packages/ui-components/src/sui-popover/sui-popover.sui
var sui_popover_default = component36("sui-popover", {
  props: {
    open: prop36.bool({ default: false, reflect: true }),
    placement: prop36.union(["top", "top-end", "bottom", "bottom-end", "left", "left-end", "right", "right-end"], { default: "bottom", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: inline-flex;
  }

  /* Native popover defaults are aggressive (position:fixed, inset:0,
     huge default margins). Reset to a layout-neutral container so our
     surface div carries the chrome and anchor positioning drives the
     placement. */
  .panel {
    position: absolute;
    margin: 0;
    border: 0;
    padding: 0;
    background: transparent;
    overflow: visible;
    inset: auto;
  }
  .panel::backdrop { background: transparent; }

  .panel > .surface {
    min-width: 180px;
    padding: 6px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    backdrop-filter: var(--card-backdrop-filter, none);
    -webkit-backdrop-filter: var(--card-backdrop-filter, none);
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open > .surface {
    opacity: 1;
    transform: translateY(0);
  }

  /* CSS Anchor Positioning. Controller sets a unique anchor-name on the
     SLOTTED trigger element via \`--sui-popover-anchor\`, so each instance
     anchors to its own trigger without any inline-style juggling. */
  @supports (anchor-name: --x) {
    ::slotted([slot='trigger']) {
      anchor-name: var(--sui-popover-anchor, --sui-popover-default);
    }
    .panel {
      position-anchor: var(--sui-popover-anchor, --sui-popover-default);
    }
    :host(:not([placement])) .panel,
    :host([placement='bottom']) .panel {
      top: calc(anchor(bottom) + 6px);
      left: anchor(left);
    }
    :host([placement='bottom-end']) .panel {
      top: calc(anchor(bottom) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='top']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      left: anchor(left);
    }
    :host([placement='top-end']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='right']) .panel {
      left: calc(anchor(right) + 6px);
      top: anchor(top);
    }
    :host([placement='right-end']) .panel {
      left: calc(anchor(right) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
    :host([placement='left']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      top: anchor(top);
    }
    :host([placement='left-end']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .panel > .surface { transition: none; transform: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.placement = props.placement;
    let __r = null;
    if (sui_popover_controller_default && typeof sui_popover_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_popover_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets36(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef16($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    attachRef16($scope, "panelEl", __e1);
    __e1.setAttribute("role", "dialog");
    __e1.setAttribute("aria-modal", "false");
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "surface");
    const __e3 = document.createElement("slot");
    __e2.appendChild(__e3);
    __e1.appendChild(__e2);
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

// packages/ui-components/src/sui-theme-picker/sui-theme-picker.sui
import { adoptSuiSheets as adoptSuiSheets37, attachRef as attachRef17, bindAttr as bindAttr23, bindFor as bindFor2, bindText as bindText8, component as component37, effect as effect23, on, prop as prop37 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-theme-picker/sui-theme-picker.controller.ts
import { controller as controller23, signal as signal8, derived as derived2, effect as effect22, ref as ref3 } from "@ship/sui-runtime";

// packages/ship-set/src/themes.ts
var THEMES = [
  {
    id: "ship-light",
    label: "SHIP Light",
    dark: false,
    swatches: [
      "oklch(0.99 0.001 106.423)",
      "oklch(1 0 0)",
      "oklch(0.67 0.16 58)",
      "oklch(0.61 0.23 303)"
    ]
  },
  {
    id: "ship-dark",
    label: "SHIP Dark",
    dark: true,
    swatches: [
      "oklch(0.18 0.01 250)",
      "oklch(0.18 0.01 50)",
      "oklch(0.77 0.16 70)",
      "oklch(0.3 0.05 300)"
    ]
  },
  {
    id: "paper",
    label: "Paper",
    dark: false,
    swatches: [
      "oklch(0.97 0.014 80)",
      "oklch(0.99 0.010 80)",
      "oklch(0.55 0.14 35)",
      "oklch(0.48 0.09 55)"
    ]
  },
  {
    id: "halo",
    label: "Halo (Soft)",
    dark: false,
    swatches: [
      "oklch(0.92 0.008 230)",
      "oklch(0.92 0.008 230)",
      "oklch(0.62 0.14 50)",
      "oklch(0.55 0.05 230)"
    ]
  },
  {
    id: "console",
    label: "Console",
    dark: true,
    swatches: [
      "oklch(0.13 0.008 220)",
      "oklch(0.16 0.010 220)",
      "oklch(0.78 0.20 145)",
      "oklch(0.78 0.16 195)"
    ]
  },
  {
    id: "contrast",
    label: "High Contrast",
    dark: true,
    swatches: [
      "oklch(0.09 0 0)",
      "oklch(0.15 0 0)",
      "oklch(0.87 0.18 85)",
      "oklch(0.99 0 0)"
    ]
  },
  {
    id: "glass",
    label: "Glass",
    dark: false,
    swatches: [
      "oklch(0.95 0.04 230)",
      "oklch(0.99 0.005 240)",
      "oklch(0.62 0.16 50)",
      "oklch(0.62 0.14 280)"
    ]
  }
];
var THEME_BY_ID = Object.fromEntries(THEMES.map((t) => [t.id, t]));
var DEFAULT_THEME = "ship-light";
var THEME_STORAGE_KEY = "ship-site-theme";
var isValidTheme = (value) => typeof value === "string" && (value in THEME_BY_ID);
var isDarkTheme = (id) => THEME_BY_ID[id]?.dark ?? false;
function applyTheme(id, root = document.documentElement) {
  const next = isValidTheme(id) ? id : DEFAULT_THEME;
  const dark = isDarkTheme(next);
  root.dataset.theme = next;
  root.classList.toggle("dark", dark);
  if (typeof document !== "undefined" && typeof CustomEvent !== "undefined") {
    document.dispatchEvent(new CustomEvent("ship:theme:change", { detail: { id: next, dark } }));
  }
  return next;
}
function readStoredTheme() {
  if (typeof localStorage === "undefined")
    return DEFAULT_THEME;
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    return isValidTheme(raw) ? raw : DEFAULT_THEME;
  } catch {
    return DEFAULT_THEME;
  }
}
function persistTheme(id) {
  if (typeof localStorage === "undefined")
    return;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id);
  } catch {}
  try {
    fetch("/api/settings", {
      method: "PUT",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key: "shell.theme", value: id })
    }).catch(() => {});
  } catch {}
}

// packages/ui-components/src/sui-theme-picker/sui-theme-picker.controller.ts
var GROUPS = [
  { label: "Default", ids: ["ship-light", "ship-dark"] },
  { label: "Texture", ids: ["paper", "halo"] },
  { label: "Effects", ids: ["glass"] },
  { label: "Special", ids: ["console", "contrast"] }
];
var THEMES_BY_ID = new Map(THEMES.map((t) => [t.id, t]));
function ensureThemesCssInShadow(host) {
  const root = host.shadowRoot;
  if (!root)
    return;
  if (root.querySelector("link[data-sui-theme-preview-styles]"))
    return;
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = "/api/assets/view-tokens.css";
  link.setAttribute("data-sui-theme-preview-styles", "");
  root.appendChild(link);
}
var sui_theme_picker_controller_default = controller23({
  scope: (init) => {
    const initAny = init;
    const activeTheme = signal8(readStoredTheme());
    const panelOpen = signal8(false);
    const groups = signal8(GROUPS.map((g) => ({
      label: g.label,
      themes: g.ids.map((id) => THEMES_BY_ID.get(id)).filter((t) => !!t)
    })));
    const dotColor = derived2(() => THEMES_BY_ID.get(activeTheme())?.swatches[2] ?? "var(--primary, #3b82f6)");
    const triggerLabel = derived2(() => THEMES_BY_ID.get(activeTheme())?.label ?? "Theme");
    const internals = {
      host: null,
      panel: null
    };
    function togglePanel() {
      const panel = internals.panel;
      if (!panel)
        return;
      if (panelOpen())
        panel.hidePopover?.();
      else
        panel.showPopover?.();
    }
    function selectTheme(id) {
      const next = applyTheme(id);
      if (initAny.persist())
        persistTheme(next);
      activeTheme.set(next);
      const host = internals.host;
      if (host) {
        host.dispatchEvent(new CustomEvent("sui-theme-change", {
          detail: { id: next, dark: THEMES_BY_ID.get(next)?.dark ?? false },
          bubbles: true,
          composed: true
        }));
      }
      internals.panel?.hidePopover?.();
    }
    const triggerBtn = ref3(null);
    const panelEl = ref3(null);
    return {
      activeTheme,
      panelOpen,
      groups,
      dotColor,
      triggerLabel,
      togglePanel,
      selectTheme,
      triggerBtn,
      panelEl,
      _internals: internals
    };
  },
  mount(state, host) {
    const scope = state;
    scope._internals.host = host;
    ensureThemesCssInShadow(host);
    applyTheme(readStoredTheme());
    const stopPanelSync = effect22(() => {
      const panel = scope.panelEl();
      scope._internals.panel = panel;
      if (!panel)
        return;
      const onToggle = (e) => {
        const newState = e.newState;
        const open = newState === "open";
        if (scope.panelOpen() !== open)
          scope.panelOpen.set(open);
        if (open)
          anchorPanel();
      };
      panel.addEventListener("toggle", onToggle);
      return () => panel.removeEventListener("toggle", onToggle);
    });
    function anchorPanel() {
      const panel = scope.panelEl();
      const trigger = scope.triggerBtn();
      if (!panel || !trigger)
        return;
      const r = trigger.getBoundingClientRect();
      panel.style.top = `${r.bottom + 4}px`;
      panel.style.left = `${Math.max(8, r.right - panel.offsetWidth)}px`;
    }
    function onExternalChange(e) {
      const id = e.detail?.id;
      if (!isValidTheme(id))
        return;
      if (scope.activeTheme() !== id)
        scope.activeTheme.set(id);
    }
    document.addEventListener("ship:theme:change", onExternalChange);
    function onPanelKeydown(e) {
      const panel = scope.panelEl();
      if (!panel)
        return;
      const cards = Array.from(panel.querySelectorAll(".card"));
      if (!cards.length)
        return;
      const root = panel.getRootNode();
      const idx = cards.findIndex((c) => c === root.activeElement);
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          cards[(idx + 1 + cards.length) % cards.length]?.focus();
          break;
        case "ArrowUp":
          e.preventDefault();
          cards[(idx - 1 + cards.length) % cards.length]?.focus();
          break;
        case "Home":
          e.preventDefault();
          cards[0]?.focus();
          break;
        case "End":
          e.preventDefault();
          cards[cards.length - 1]?.focus();
          break;
        case "Escape":
          e.preventDefault();
          panel.hidePopover?.();
          break;
      }
    }
    const stopKeyBind = effect22(() => {
      const panel = scope.panelEl();
      if (!panel)
        return;
      panel.addEventListener("keydown", onPanelKeydown);
      return () => panel.removeEventListener("keydown", onPanelKeydown);
    });
    const stopFocusOnOpen = effect22(() => {
      if (!scope.panelOpen())
        return;
      requestAnimationFrame(() => {
        const panel = scope.panelEl();
        if (!panel)
          return;
        const cards = Array.from(panel.querySelectorAll(".card"));
        const current = cards.find((c) => c.getAttribute("aria-selected") === "true");
        (current ?? cards[0])?.focus();
      });
    });
    return () => {
      document.removeEventListener("ship:theme:change", onExternalChange);
      stopPanelSync();
      stopKeyBind();
      stopFocusOnOpen();
    };
  }
});

// packages/ui-components/src/sui-theme-picker/sui-theme-picker.sui
var sui_theme_picker_default = component37("sui-theme-picker", {
  props: {
    placement: prop37.union(["top", "top-end", "bottom", "bottom-end", "left", "left-end", "right", "right-end"], { default: "bottom-end", reflect: true }),
    "label-style": prop37.union(["text", "compact"], { default: "text", reflect: true }),
    persist: prop37.bool({ default: true, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: inline-flex;
    --tp-trigger-gap: 6px;
  }

  /* Trigger button — uses ghost-button visual: transparent until hover.
     Mirrors the look of the shell's <ship-button variant=ghost size=sm>
     because we don't (yet) want sui-theme-picker to depend on sui-button. */
  .trigger {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    padding: 4px 8px;
    font: inherit;
    font-size: var(--text-base, 14px);
    color: var(--foreground, #111);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--tp-trigger-gap);
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .trigger:hover {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
  }
  .trigger:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .dot {
    width: 10px; height: 10px;
    border-radius: 999px;
    background: var(--primary, #3b82f6);
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--foreground, #111) 15%, transparent);
    flex: 0 0 auto;
  }

  .label {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 14ch;
  }
  :host([label-style='compact']) .label { display: none; }

  .caret {
    width: 10px; height: 6px;
    flex: 0 0 auto;
    color: var(--muted-foreground, #888);
  }

  /* Native popover panel. Reset the UA's default centering and let
     the controller place us via getBoundingClientRect on the trigger. */
  .panel {
    position: fixed;
    inset: auto;
    margin: 0;
    border: 0;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    opacity: 0;
    transform: translateY(-4px) scale(0.97);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  .panel::backdrop { background: transparent; }

  .grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 332px;
    max-height: min(80vh, 720px);
    overflow-y: auto;
    scrollbar-width: thin;
  }

  .group { display: flex; flex-direction: column; gap: 2px; }
  .group-h {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted-foreground, #888);
    padding: 0 8px 2px;
    opacity: 0.7;
  }
  .group-rows { display: flex; flex-direction: column; gap: 4px; }

  /* Card — full-width entry per theme. Aria-selected drives a tinted
     background + radio fill. */
  .card {
    appearance: none;
    background: transparent;
    border: 0;
    border-radius: 10px;
    padding: 6px;
    cursor: pointer;
    text-align: left;
    font: inherit;
    display: flex;
    flex-direction: column;
    gap: 6px;
    outline: none;
    transition:
      background var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .card:hover {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
  }
  .card:focus-visible {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }
  .card[aria-selected='true'] {
    background: color-mix(in oklch, var(--primary, #3b82f6) 10%, transparent);
  }

  .row {
    display: flex; align-items: center; gap: 8px;
    padding: 0 4px;
  }
  .name {
    flex: 1;
    font-family: var(--font-body, system-ui);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground, #111);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .radio {
    width: 12px; height: 12px;
    border-radius: 999px;
    border: 1.5px solid color-mix(in oklch, var(--foreground, #111) 40%, transparent);
    background: transparent;
    position: relative;
    flex-shrink: 0;
  }
  .card[aria-selected='true'] .radio { border-color: var(--primary, #3b82f6); }
  .card[aria-selected='true'] .radio::after {
    content: '';
    position: absolute;
    inset: 2px;
    border-radius: 999px;
    background: var(--primary, #3b82f6);
  }

  /* Mini card preview tile. Each .tc-bg carries data-theme-preview="<id>",
     which the per-theme \`[data-theme-preview]\` rules in themes.css scope
     for. The controller links view-tokens.css (which bundles themes.css)
     into the shadow root so those rules reach these elements. */
  .tc-bg {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 132px;
    background: var(--background);
    color: var(--foreground);
    border-radius: 8px;
    overflow: hidden;
    isolation: isolate;
    box-shadow: inset 0 0 0 1px color-mix(in oklch, currentColor 14%, transparent);
  }
  .tc-overlay {
    position: absolute; inset: 0;
    pointer-events: none;
    background: var(--surface-overlay, none);
    mix-blend-mode: var(--surface-overlay-blend, normal);
    opacity: var(--surface-overlay-opacity, 1);
    z-index: 1;
  }
  @media (prefers-reduced-motion: reduce) {
    .tc-overlay { display: none; }
    .panel { transition: none; transform: none !important; }
  }
  .tc-card {
    position: relative;
    z-index: 2;
    display: flex; flex-direction: column; gap: 7px;
    width: calc(100% - 24px);
    padding: 10px 12px 11px;
    background: var(--card);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-card-rest, var(--shadow-sm, 0 1px 2px oklch(0 0 0 / 8%)));
    backdrop-filter: var(--card-backdrop-filter, none);
    -webkit-backdrop-filter: var(--card-backdrop-filter, none);
  }
  .tc-card-head { display: flex; flex-direction: column; gap: 2px; }
  .tc-eyebrow {
    font-family: var(--label-font, var(--font-mono));
    font-weight: var(--label-weight, 600);
    letter-spacing: var(--label-tracking, 0.08em);
    font-size: 8px; line-height: 1;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .tc-heading {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--heading-weight, 600);
    font-size: 13px; line-height: 1.05;
    color: var(--card-foreground);
  }
  .tc-typo {
    display: flex; align-items: baseline; gap: 6px;
    font-family: var(--body-font, var(--font-body));
    font-size: 9.5px; line-height: 1;
    color: var(--muted-foreground);
  }
  .tc-body { color: var(--card-foreground); opacity: 0.85; }
  .tc-mono {
    font-family: var(--font-mono);
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 3px;
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
    color: var(--muted-foreground);
  }
  .tc-controls { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
  .tc-cta {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: 4px 8px;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground);
  }
  .tc-ghost {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: 4px 8px;
    border-radius: var(--radius-sm, 4px);
    background: transparent;
    color: var(--muted-foreground);
    border: 1px solid var(--border);
  }
  .tc-chip {
    display: inline-flex; align-items: center;
    font-weight: 600; font-size: 8.5px; line-height: 1;
    padding: 3px 6px;
    border-radius: 999px;
  }
  .tc-chip-accent {
    background: color-mix(in oklch, var(--accent) 18%, transparent);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--accent) 30%, transparent);
  }
  .tc-chip-muted {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    color: var(--muted-foreground);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.placement = props.placement;
    $scope.labelStyle = props.labelStyle;
    $scope.persist = props.persist;
    let __r = null;
    if (sui_theme_picker_controller_default && typeof sui_theme_picker_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_theme_picker_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b6 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.groups(),
      key: (g, __i) => {
        const __child = Object.create($scope);
        __child.g = g;
        __child.__i = __i;
        return (($scope2) => g.label)(__child);
      },
      render: (g, __i) => {
        const __childScope9 = Object.create($scope);
        __childScope9.g = g;
        __childScope9.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b9 = (anchor2) => bindFor2({
            anchor: anchor2,
            items: () => $scope2.g.themes,
            key: (t, __i2) => {
              const __child = Object.create($scope2);
              __child.t = t;
              __child.__i = __i2;
              return (($scope3) => t.id)(__child);
            },
            render: (t, __i2) => {
              const __childScope17 = Object.create($scope2);
              __childScope17.t = t;
              __childScope17.__i = __i2;
              return (($scope3) => {
                const __frag18 = document.createDocumentFragment();
                const __c19 = [];
                const __e20 = document.createElement("button");
                __e20.setAttribute("class", "card");
                __e20.setAttribute("role", "option");
                __e20.setAttribute("type", "button");
                __c19.push(bindAttr23(__e20, "data-theme-card", () => $scope3.t.id));
                __c19.push(bindAttr23(__e20, "aria-label", () => $scope3.t.label));
                __c19.push(bindAttr23(__e20, "aria-selected", () => $scope3.activeTheme() === $scope3.t.id ? "true" : "false"));
                __c19.push(bindAttr23(__e20, "tabindex", () => $scope3.activeTheme() === $scope3.t.id ? "0" : "-1"));
                __c19.push(on(__e20, "click", function() {
                  const __h = () => $scope3.selectTheme($scope3.t.id);
                  return typeof __h === "function" ? __h.bind(undefined) : () => {};
                }(), { modifiers: [] }));
                const __e21 = document.createElement("span");
                __e21.setAttribute("class", "tc-bg");
                __c19.push(bindAttr23(__e21, "data-theme-preview", () => $scope3.t.id));
                __e21.setAttribute("aria-hidden", "true");
                const __e22 = document.createElement("span");
                __e22.setAttribute("class", "tc-overlay");
                __e21.appendChild(__e22);
                const __e23 = document.createElement("span");
                __e23.setAttribute("class", "tc-card");
                const __e24 = document.createElement("span");
                __e24.setAttribute("class", "tc-card-head");
                const __e25 = document.createElement("span");
                __e25.setAttribute("class", "tc-eyebrow");
                const __t26 = document.createTextNode("THEME");
                __e25.appendChild(__t26);
                __e24.appendChild(__e25);
                const __e27 = document.createElement("span");
                __e27.setAttribute("class", "tc-heading");
                const __t28 = document.createTextNode("");
                __e27.appendChild(__t28);
                __c19.push(bindText8(__t28, () => $scope3.t.label));
                __e24.appendChild(__e27);
                __e23.appendChild(__e24);
                const __e29 = document.createElement("span");
                __e29.setAttribute("class", "tc-typo");
                const __e30 = document.createElement("span");
                __e30.setAttribute("class", "tc-body");
                const __t31 = document.createTextNode("The quick brown fox");
                __e30.appendChild(__t31);
                __e29.appendChild(__e30);
                const __e32 = document.createElement("span");
                __e32.setAttribute("class", "tc-mono");
                const __t33 = document.createTextNode("");
                __e32.appendChild(__t33);
                __c19.push(bindText8(__t33, () => "{ build: ok }"));
                __e29.appendChild(__e32);
                __e23.appendChild(__e29);
                const __e34 = document.createElement("span");
                __e34.setAttribute("class", "tc-controls");
                const __e35 = document.createElement("span");
                __e35.setAttribute("class", "tc-cta");
                const __t36 = document.createTextNode("Action");
                __e35.appendChild(__t36);
                __e34.appendChild(__e35);
                const __e37 = document.createElement("span");
                __e37.setAttribute("class", "tc-ghost");
                const __t38 = document.createTextNode("Cancel");
                __e37.appendChild(__t38);
                __e34.appendChild(__e37);
                const __e39 = document.createElement("span");
                __e39.setAttribute("class", "tc-chip tc-chip-accent");
                const __t40 = document.createTextNode("New");
                __e39.appendChild(__t40);
                __e34.appendChild(__e39);
                const __e41 = document.createElement("span");
                __e41.setAttribute("class", "tc-chip tc-chip-muted");
                const __t42 = document.createTextNode("Idle");
                __e41.appendChild(__t42);
                __e34.appendChild(__e41);
                __e23.appendChild(__e34);
                __e21.appendChild(__e23);
                __e20.appendChild(__e21);
                const __e43 = document.createElement("span");
                __e43.setAttribute("class", "row");
                const __e44 = document.createElement("span");
                __e44.setAttribute("class", "name");
                const __t45 = document.createTextNode("");
                __e44.appendChild(__t45);
                __c19.push(bindText8(__t45, () => $scope3.t.label));
                __e43.appendChild(__e44);
                const __e46 = document.createElement("span");
                __e46.setAttribute("class", "radio");
                __e46.setAttribute("aria-hidden", "true");
                __e43.appendChild(__e46);
                __e20.appendChild(__e43);
                __frag18.appendChild(__e20);
                return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
                  for (const c of __c19)
                    try {
                      c();
                    } catch (e) {}
                } };
              })(__childScope17);
            }
          });
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("section");
          __e12.setAttribute("class", "group");
          __c11.push(bindAttr23(__e12, "aria-label", () => $scope2.g.label));
          const __e13 = document.createElement("div");
          __e13.setAttribute("class", "group-h");
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c11.push(bindText8(__t14, () => $scope2.g.label));
          __e12.appendChild(__e13);
          const __e15 = document.createElement("div");
          __e15.setAttribute("class", "group-rows");
          const __forAnchor16 = document.createComment("for");
          __e15.appendChild(__forAnchor16);
          __c11.push(__wireBoundary_b9(__forAnchor16));
          __e12.appendChild(__e15);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope9);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets37(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("class", "trigger");
    __e0.setAttribute("part", "trigger");
    __e0.setAttribute("type", "button");
    attachRef17($scope, "triggerBtn", __e0);
    __e0.setAttribute("aria-label", "Choose theme");
    __e0.setAttribute("aria-haspopup", "dialog");
    __cleanups.push(bindAttr23(__e0, "aria-expanded", () => $scope.panelOpen() ? "true" : "false"));
    __cleanups.push(on(__e0, "click", function() {
      const __h = () => $scope.togglePanel();
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "dot");
    __e1.setAttribute("part", "dot");
    __cleanups.push(effect23(() => {
      const __v = $scope.dotColor();
      if (__v == null)
        __e1.style.removeProperty("background");
      else
        __e1.style.setProperty("background", String(__v));
    }));
    __e1.setAttribute("aria-hidden", "true");
    __e0.appendChild(__e1);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "label");
    __e2.setAttribute("part", "label");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText8(__t3, () => $scope.triggerLabel()));
    __e0.appendChild(__e2);
    const __e4 = document.createElement("svg");
    __e4.setAttribute("class", "caret");
    __e4.setAttribute("viewbox", "0 0 10 6");
    __e4.setAttribute("aria-hidden", "true");
    const __e5 = document.createElement("path");
    __e5.setAttribute("d", "M1 1l4 4 4-4");
    __e5.setAttribute("fill", "none");
    __e5.setAttribute("stroke", "currentColor");
    __e5.setAttribute("stroke-width", "1.5");
    __e5.setAttribute("stroke-linecap", "round");
    __e5.setAttribute("stroke-linejoin", "round");
    __e4.appendChild(__e5);
    __e0.appendChild(__e4);
    __root.appendChild(__e0);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "panel");
    __e6.setAttribute("part", "panel");
    __e6.setAttribute("popover", "auto");
    attachRef17($scope, "panelEl", __e6);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "grid");
    __e7.setAttribute("part", "grid");
    __e7.setAttribute("role", "listbox");
    __e7.setAttribute("aria-label", "Theme");
    const __forAnchor8 = document.createComment("for");
    __e7.appendChild(__forAnchor8);
    __cleanups.push(__wireBoundary_b6(__forAnchor8));
    __e6.appendChild(__e7);
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

// packages/ui-components/src/sui-rail/sui-rail.sui
import { adoptSuiSheets as adoptSuiSheets38, attachRef as attachRef18, component as component38, prop as prop38 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-rail/sui-rail.controller.ts
import { controller as controller24, effect as effect24 } from "@ship/sui-runtime";
var sui_rail_controller_default = controller24({
  mount(state, host) {
    const scope = state;
    const stopIcon = effect24(() => {
      const btn = scope.toggleBtn();
      const col = scope.collapsed();
      if (btn)
        btn.setAttribute("aria-label", col ? "Show rail" : "Hide rail");
    });
    const onToggle = () => {
      const next = !scope.collapsed();
      scope.collapsed.set(next);
      host.dispatchEvent(new CustomEvent("sui-rail:toggle", {
        detail: { collapsed: next },
        bubbles: true,
        composed: true
      }));
    };
    const stopBtn = effect24(() => {
      const btn = scope.toggleBtn();
      if (!btn)
        return;
      btn.addEventListener("click", onToggle);
      return () => btn.removeEventListener("click", onToggle);
    });
    return () => {
      stopIcon();
      stopBtn();
    };
  }
});

// packages/ui-components/src/sui-rail/sui-rail.sui
var sui_rail_default = component38("sui-rail", {
  props: {
    collapsed: prop38.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    width: var(--ship-rail-w, 200px);
    background: var(--sidebar, var(--muted));
    border-right: 1px solid var(--border);
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    /* Inherited custom props consumed by slotted children */
    --rail-label-display: flex;
    --rail-item-justify: flex-start;
  }
  :host([collapsed]) {
    width: 32px;
    --rail-label-display: none;
    --rail-item-justify: center;
  }
  :host([collapsed]) .rail-nav { display: none; }
  :host([collapsed]) .rail-footer { padding: 4px; justify-content: center; }

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
    align-items: center;
    justify-content: flex-end;
    padding: var(--space-2, 8px);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }

  .toggle-btn {
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
  .toggle-btn:hover {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    border-color: var(--border);
    color: var(--foreground);
  }
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }
  .toggle-btn::before { content: '◀'; font-size: 10px; line-height: 1; }
  :host([collapsed]) .toggle-btn::before { content: '▶'; }

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
    let __r = null;
    if (sui_rail_controller_default && typeof sui_rail_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_rail_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets38(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("class", "rail-nav");
    __e0.setAttribute("part", "nav");
    const __e1 = document.createElement("slot");
    attachRef18($scope, "navSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "rail-footer");
    __e2.setAttribute("part", "footer");
    const __e3 = document.createElement("button");
    __e3.setAttribute("type", "button");
    __e3.setAttribute("class", "toggle-btn");
    __e3.setAttribute("part", "toggle");
    attachRef18($scope, "toggleBtn", __e3);
    __e3.setAttribute("aria-label", "Collapse rail");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-rail/sui-rail-item.sui
import { adoptSuiSheets as adoptSuiSheets39, attachRef as attachRef19, bindAttr as bindAttr24, bindText as bindText9, component as component39, prop as prop39 } from "@ship/sui-runtime";
var sui_rail_item_default = component39("sui-rail-item", {
  props: {
    href: prop39.string({ default: "" }),
    label: prop39.string({ default: "" }),
    icon: prop39.string({ default: "" }),
    active: prop39.bool({ default: false, reflect: true }),
    badge: prop39.string({ default: "" }),
    disabled: prop39.bool({ default: false, reflect: true })
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets39(__root);
    const __e0 = document.createElement("a");
    __e0.setAttribute("class", "item");
    __e0.setAttribute("part", "item");
    attachRef19($scope, "itemEl", __e0);
    __cleanups.push(bindAttr24(__e0, "href", () => $scope.href() || "#"));
    __cleanups.push(bindAttr24(__e0, "aria-current", () => $scope.active() ? "page" : null));
    __cleanups.push(bindAttr24(__e0, "aria-disabled", () => $scope.disabled() ? "true" : null));
    __cleanups.push(bindAttr24(__e0, "title", () => $scope.label()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "icon-wrap");
    __e1.setAttribute("part", "icon-wrap");
    const __e2 = document.createElement("sui-icon");
    __cleanups.push(bindAttr24(__e2, "name", () => $scope.icon()));
    __e2.setAttribute("size", "sm");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "label");
    __e3.setAttribute("part", "label");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText9(__t4, () => $scope.label()));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "badge");
    __e5.setAttribute("part", "badge");
    attachRef19($scope, "badgeEl", __e5);
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText9(__t6, () => $scope.badge()));
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/ui-components/src/sui-rail/sui-rail-section.sui
import { adoptSuiSheets as adoptSuiSheets40, bindText as bindText10, component as component40, prop as prop40 } from "@ship/sui-runtime";
var sui_rail_section_default = component40("sui-rail-section", {
  props: {
    label: prop40.string({ default: "" })
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
    adoptSuiSheets40(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "heading");
    __e0.setAttribute("part", "heading");
    const __t1 = document.createTextNode("");
    __e0.appendChild(__t1);
    __cleanups.push(bindText10(__t1, () => $scope.label()));
    __root.appendChild(__e0);
    const __e2 = document.createElement("slot");
    __root.appendChild(__e2);
    return $scope;
  }
});

// packages/ui-components/src/sui-status-bar/sui-status-bar.sui
import { adoptSuiSheets as adoptSuiSheets41, bindText as bindText11, component as component41, prop as prop41 } from "@ship/sui-runtime";
var sui_status_bar_default = component41("sui-status-bar", {
  props: {
    branch: prop41.string({ default: "" }),
    theme: prop41.string({ default: "" }),
    online: prop41.bool({ default: false, reflect: true }),
    agents: prop41.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
    height: 24px;
    padding: 0 var(--space-3, 12px);
    border-top: 1px solid var(--border);
    background: var(--sidebar, var(--muted));
    font-family: var(--font-body);
    font-size: 11px;
    color: var(--muted-foreground);
    user-select: none;
    -webkit-user-select: none;
    overflow: hidden;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full, 9999px);
    background: var(--destructive);
    flex-shrink: 0;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  :host([online]) .dot {
    background: var(--success);
  }

  .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .text:empty { display: none; }

  .slot-wrap {
    display: inline-flex;
    gap: 8px;
    align-items: center;
  }

  .spacer { flex: 1; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.branch = props.branch;
    $scope.theme = props.theme;
    $scope.online = props.online;
    $scope.agents = props.agents;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets41(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "dot");
    __e0.setAttribute("part", "dot");
    __e0.setAttribute("aria-hidden", "true");
    __root.appendChild(__e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "text");
    __e1.setAttribute("part", "branch");
    const __t2 = document.createTextNode("");
    __e1.appendChild(__t2);
    __cleanups.push(bindText11(__t2, () => $scope.branch()));
    __root.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "text");
    __e3.setAttribute("part", "agents");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText11(__t4, () => $scope.agents()));
    __root.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "slot-wrap");
    const __e6 = document.createElement("slot");
    __e6.setAttribute("name", "start");
    __e5.appendChild(__e6);
    __root.appendChild(__e5);
    const __e7 = document.createElement("span");
    __e7.setAttribute("class", "spacer");
    __root.appendChild(__e7);
    const __e8 = document.createElement("span");
    __e8.setAttribute("class", "slot-wrap");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "end");
    __e8.appendChild(__e9);
    __root.appendChild(__e8);
    const __e10 = document.createElement("span");
    __e10.setAttribute("class", "text");
    __e10.setAttribute("part", "theme");
    const __t11 = document.createTextNode("");
    __e10.appendChild(__t11);
    __cleanups.push(bindText11(__t11, () => $scope.theme()));
    __root.appendChild(__e10);
    return $scope;
  }
});

// packages/ui-components/src/sui-dialog/sui-dialog.sui
import { adoptSuiSheets as adoptSuiSheets42, attachRef as attachRef20, component as component42, prop as prop42 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-dialog/sui-dialog.controller.ts
import { controller as controller25, effect as effect25 } from "@ship/sui-runtime";

// packages/ui-components/src/overlay-focus.ts
var FOCUSABLE = [
  "a[href]",
  "area[href]",
  "button:not([disabled])",
  'input:not([disabled]):not([type="hidden"])',
  "select:not([disabled])",
  "textarea:not([disabled])",
  "iframe",
  "object",
  "embed",
  "summary",
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex^="-"])'
].join(",");
function focusableWithin(root) {
  return Array.from(root.querySelectorAll(FOCUSABLE)).filter((el) => {
    if (el.hasAttribute("disabled"))
      return false;
    if (el.getAttribute("aria-hidden") === "true")
      return false;
    return true;
  });
}

class FocusReturn {
  saved = null;
  save() {
    this.saved = document.activeElement ?? null;
  }
  restore() {
    const t = this.saved;
    this.saved = null;
    if (t && typeof t.focus === "function")
      t.focus();
  }
}
var _scrollLockCount = 0;
var _savedOverflow = null;
var _savedPadding = null;
function acquireScrollLock() {
  if (typeof document === "undefined")
    return;
  _scrollLockCount += 1;
  if (_scrollLockCount > 1)
    return;
  const body = document.body;
  _savedOverflow = body.style.overflow;
  _savedPadding = body.style.paddingRight;
  const scrollW = typeof window !== "undefined" ? window.innerWidth - document.documentElement.clientWidth : 0;
  if (scrollW > 0) {
    const cur = parseFloat(getComputedStyle(body).paddingRight || "0") || 0;
    body.style.paddingRight = `${cur + scrollW}px`;
  }
  body.style.overflow = "hidden";
}
function releaseScrollLock() {
  if (typeof document === "undefined")
    return;
  if (_scrollLockCount === 0)
    return;
  _scrollLockCount -= 1;
  if (_scrollLockCount > 0)
    return;
  const body = document.body;
  body.style.overflow = _savedOverflow ?? "";
  body.style.paddingRight = _savedPadding ?? "";
  _savedOverflow = null;
  _savedPadding = null;
}

// packages/ui-components/src/sui-dialog/sui-dialog.controller.ts
var _seq3 = 0;
var sui_dialog_controller_default = controller25({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq3;
    const headerId = `sui-dialog-header-${uid}`;
    const bodyId = `sui-dialog-body-${uid}`;
    const dialogEl = scope.dialogEl();
    const closeBtn = scope.closeBtn();
    const sentinelStart = scope.sentinelStart();
    const sentinelEnd = scope.sentinelEnd();
    const headerEl = scope.headerEl();
    const bodyEl = scope.bodyEl();
    if (!dialogEl)
      return;
    if (headerEl)
      headerEl.id = headerId;
    if (bodyEl)
      bodyEl.id = bodyId;
    dialogEl.setAttribute("aria-labelledby", headerId);
    dialogEl.setAttribute("aria-describedby", bodyId);
    let hasScrollLock = false;
    const focusReturn = new FocusReturn;
    function updateRole() {
      dialogEl.setAttribute("role", scope.roleOverride() || "dialog");
      dialogEl.setAttribute("aria-modal", "true");
    }
    updateRole();
    function collectFocusables() {
      const shadowFocusables = focusableWithin(dialogEl).filter((el) => !el.classList.contains("sentinel"));
      const lightFocusables = focusableWithin(host);
      return [...lightFocusables, ...shadowFocusables];
    }
    function focusInitial() {
      const autofocus = host.querySelector("[autofocus]") ?? dialogEl.querySelector("[autofocus]");
      if (autofocus) {
        autofocus.focus();
        return;
      }
      const focusables = collectFocusables();
      const primary = focusables.find((el) => el !== closeBtn) ?? focusables[0];
      if (primary)
        primary.focus();
      else
        dialogEl.focus();
    }
    function doOpen() {
      focusReturn.save();
      try {
        dialogEl.showModal();
      } catch {}
      if (!hasScrollLock) {
        acquireScrollLock();
        hasScrollLock = true;
      }
      queueMicrotask(() => focusInitial());
      host.dispatchEvent(new CustomEvent("sui-open", { bubbles: true, composed: true }));
    }
    function doClose() {
      if (dialogEl.open)
        dialogEl.close();
    }
    const onNativeClose = () => {
      if (scope.open())
        scope.open.set(false);
      if (hasScrollLock) {
        releaseScrollLock();
        hasScrollLock = false;
      }
      focusReturn.restore();
      host.dispatchEvent(new CustomEvent("sui-close", { bubbles: true, composed: true }));
    };
    const onBackdropClick = (e) => {
      if (!scope.dismissible())
        return;
      if (e.target === dialogEl)
        scope.open.set(false);
    };
    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
      scope.open.set(false);
    };
    const onSentinelStartFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[items.length - 1].focus();
    };
    const onSentinelEndFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[0].focus();
    };
    dialogEl.addEventListener("close", onNativeClose);
    dialogEl.addEventListener("click", onBackdropClick);
    closeBtn?.addEventListener("click", onDismiss);
    sentinelStart?.addEventListener("focus", onSentinelStartFocus);
    sentinelEnd?.addEventListener("focus", onSentinelEndFocus);
    const stopOpenSync = effect25(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      if (wantOpen && !dialogEl.open)
        doOpen();
      else if (!wantOpen && dialogEl.open)
        doClose();
    });
    const stopRoleSync = effect25(() => {
      updateRole();
    });
    host.show = () => scope.open.set(true);
    host.showModal = () => scope.open.set(true);
    host.close = () => scope.open.set(false);
    return () => {
      if (hasScrollLock)
        releaseScrollLock();
      dialogEl.removeEventListener("close", onNativeClose);
      dialogEl.removeEventListener("click", onBackdropClick);
      closeBtn?.removeEventListener("click", onDismiss);
      sentinelStart?.removeEventListener("focus", onSentinelStartFocus);
      sentinelEnd?.removeEventListener("focus", onSentinelEndFocus);
      stopOpenSync();
      stopRoleSync();
    };
  }
});

// packages/ui-components/src/sui-dialog/sui-dialog.sui
var sui_dialog_default = component42("sui-dialog", {
  props: {
    open: prop42.bool({ default: false, reflect: true }),
    size: prop42.union(["sm", "md", "lg", "full"], { default: "md", reflect: true }),
    backdrop: prop42.union(["default", "blur", "solid"], { default: "default", reflect: true }),
    dismissible: prop42.bool({ default: true }),
    "no-close-button": prop42.bool({ default: false, reflect: true }),
    "no-header": prop42.bool({ default: false, reflect: true }),
    "no-footer": prop42.bool({ default: false, reflect: true }),
    "role-override": prop42.union(["dialog", "alertdialog"], { default: "dialog", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dlg-max-w: 500px; --dlg-px: 24px; --dlg-hpy: 18px; --dlg-bpy: 16px; --dlg-fpy: 16px; }
  :host([size='sm']) { --dlg-max-w: 400px; --dlg-px: 20px; --dlg-hpy: 16px; --dlg-bpy: 14px; --dlg-fpy: 14px; }
  :host([size='lg']) { --dlg-max-w: 700px; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }
  :host([size='full']) { --dlg-max-w: 90vw; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }

  dialog {
    position: fixed;
    inset: 0;
    margin: auto;
    padding: 0;
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: calc(100vw - 32px);
    max-width: var(--dlg-max-w, 500px);
    max-height: min(calc(100vh - 32px), 85vh);
    box-shadow: var(--shadow-xl, 0 20px 60px rgba(0,0,0,0.3));
    font-family: var(--font-body, system-ui);
    overflow: visible;
    display: flex;
    flex-direction: column;
    opacity: 0;
    /* Closed state: don't intercept clicks. The dialog stays in the
       layout tree so the open-state CSS transition has something to
       animate from, but \`position: fixed; inset: 0\` would otherwise
       cover the entire viewport with an invisible (opacity: 0) but
       click-eating overlay — clicks would land on the dialog instead
       of the underlying app surface. \`pointer-events: none\` lifts that
       restriction whenever the dialog isn't open. */
    pointer-events: none;
    transform: translateY(8px) scale(0.96);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([size='full']) dialog { max-height: 90vh; }
  dialog[open] {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
  }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }
  :host([backdrop='blur']) dialog::backdrop { backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px); }
  :host([backdrop='solid']) dialog::backdrop { background: var(--background, #fff); }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    padding: var(--dlg-hpy, 18px) var(--dlg-px, 24px);
    padding-right: calc(var(--dlg-px, 24px) + 36px);
    flex: 0 0 auto;
    font-size: var(--text-lg, 16px);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground, #111);
  }
  :host([no-header]) .header { display: none; }

  .body {
    padding: var(--dlg-bpy, 16px) var(--dlg-px, 24px);
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    padding: var(--dlg-fpy, 16px) var(--dlg-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: flex-end;
    align-items: center;
    flex: 0 0 auto;
  }
  :host([no-footer]) .footer { display: none; }

  .close {
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }
  :host([no-close-button]) .close { display: none; }

  @media (prefers-reduced-motion: reduce) {
    dialog { transition: none; transform: none !important; opacity: 1; }
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.size = props.size;
    $scope.backdrop = props.backdrop;
    $scope.dismissible = props.dismissible;
    $scope.noCloseButton = props.noCloseButton;
    $scope.noHeader = props.noHeader;
    $scope.noFooter = props.noFooter;
    $scope.roleOverride = props.roleOverride;
    let __r = null;
    if (sui_dialog_controller_default && typeof sui_dialog_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_dialog_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets42(__root);
    const __e0 = document.createElement("dialog");
    __e0.setAttribute("part", "dialog");
    attachRef20($scope, "dialogEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("tabindex", "0");
    __e1.setAttribute("class", "sentinel");
    __e1.setAttribute("aria-hidden", "true");
    attachRef20($scope, "sentinelStart", __e1);
    __e0.appendChild(__e1);
    const __e2 = document.createElement("button");
    __e2.setAttribute("class", "close");
    __e2.setAttribute("part", "close");
    __e2.setAttribute("type", "button");
    __e2.setAttribute("aria-label", "Close dialog");
    attachRef20($scope, "closeBtn", __e2);
    const __e3 = document.createElement("svg");
    __e3.setAttribute("viewbox", "0 0 16 16");
    __e3.setAttribute("fill", "none");
    __e3.setAttribute("stroke", "currentColor");
    __e3.setAttribute("stroke-width", "1.75");
    __e3.setAttribute("stroke-linecap", "round");
    __e3.setAttribute("stroke-linejoin", "round");
    __e3.setAttribute("aria-hidden", "true");
    __e3.setAttribute("focusable", "false");
    const __e4 = document.createElement("path");
    __e4.setAttribute("d", "M3.5 3.5l9 9M12.5 3.5l-9 9");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    __e0.appendChild(__e2);
    const __e5 = document.createElement("div");
    __e5.setAttribute("class", "header");
    __e5.setAttribute("part", "header");
    attachRef20($scope, "headerEl", __e5);
    const __e6 = document.createElement("slot");
    __e6.setAttribute("name", "header");
    __e5.appendChild(__e6);
    __e0.appendChild(__e5);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "body");
    __e7.setAttribute("part", "body");
    attachRef20($scope, "bodyEl", __e7);
    const __e8 = document.createElement("slot");
    __e7.appendChild(__e8);
    __e0.appendChild(__e7);
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "footer");
    __e9.setAttribute("part", "footer");
    const __e10 = document.createElement("slot");
    __e10.setAttribute("name", "footer");
    __e9.appendChild(__e10);
    __e0.appendChild(__e9);
    const __e11 = document.createElement("span");
    __e11.setAttribute("tabindex", "0");
    __e11.setAttribute("class", "sentinel");
    __e11.setAttribute("aria-hidden", "true");
    attachRef20($scope, "sentinelEnd", __e11);
    __e0.appendChild(__e11);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-drawer/sui-drawer.sui
import { adoptSuiSheets as adoptSuiSheets43, attachRef as attachRef21, component as component43, prop as prop43 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-drawer/sui-drawer.controller.ts
import { controller as controller26, effect as effect26 } from "@ship/sui-runtime";
var _seq4 = 0;
var sui_drawer_controller_default = controller26({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq4;
    const headerId = `sui-drawer-header-${uid}`;
    const bodyId = `sui-drawer-body-${uid}`;
    const dialogEl = scope.dialogEl();
    const closeBtn = scope.closeBtn();
    const sentinelStart = scope.sentinelStart();
    const sentinelEnd = scope.sentinelEnd();
    const headerEl = scope.headerEl();
    const bodyEl = scope.bodyEl();
    if (!dialogEl)
      return;
    if (headerEl)
      headerEl.id = headerId;
    if (bodyEl)
      bodyEl.id = bodyId;
    dialogEl.setAttribute("aria-labelledby", headerId);
    dialogEl.setAttribute("aria-describedby", bodyId);
    let hasScrollLock = false;
    const focusReturn = new FocusReturn;
    function collectFocusables() {
      const shadowFocusables = focusableWithin(dialogEl).filter((el) => !el.classList.contains("sentinel"));
      const lightFocusables = focusableWithin(host);
      return [...lightFocusables, ...shadowFocusables];
    }
    function focusInitial() {
      const autofocus = host.querySelector("[autofocus]") ?? dialogEl.querySelector("[autofocus]");
      if (autofocus) {
        autofocus.focus();
        return;
      }
      const focusables = collectFocusables();
      const primary = focusables.find((el) => el !== closeBtn) ?? focusables[0];
      if (primary)
        primary.focus();
      else
        dialogEl.focus();
    }
    function doOpen() {
      focusReturn.save();
      try {
        dialogEl.showModal();
      } catch {}
      if (!hasScrollLock) {
        acquireScrollLock();
        hasScrollLock = true;
      }
      queueMicrotask(() => focusInitial());
      host.dispatchEvent(new CustomEvent("sui-open", { bubbles: true, composed: true }));
    }
    function doClose() {
      if (dialogEl.open)
        dialogEl.close();
    }
    const onNativeClose = () => {
      if (scope.open())
        scope.open.set(false);
      if (hasScrollLock) {
        releaseScrollLock();
        hasScrollLock = false;
      }
      focusReturn.restore();
      host.dispatchEvent(new CustomEvent("sui-close", { bubbles: true, composed: true }));
    };
    const onBackdropClick = (e) => {
      if (!scope.dismissible())
        return;
      if (e.target === dialogEl)
        scope.open.set(false);
    };
    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
      scope.open.set(false);
    };
    const onSentinelStartFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[items.length - 1].focus();
    };
    const onSentinelEndFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[0].focus();
    };
    dialogEl.addEventListener("close", onNativeClose);
    dialogEl.addEventListener("click", onBackdropClick);
    closeBtn?.addEventListener("click", onDismiss);
    sentinelStart?.addEventListener("focus", onSentinelStartFocus);
    sentinelEnd?.addEventListener("focus", onSentinelEndFocus);
    const stopOpenSync = effect26(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      if (wantOpen && !dialogEl.open)
        doOpen();
      else if (!wantOpen && dialogEl.open)
        doClose();
    });
    host.show = () => scope.open.set(true);
    host.close = () => scope.open.set(false);
    return () => {
      if (hasScrollLock)
        releaseScrollLock();
      dialogEl.removeEventListener("close", onNativeClose);
      dialogEl.removeEventListener("click", onBackdropClick);
      closeBtn?.removeEventListener("click", onDismiss);
      sentinelStart?.removeEventListener("focus", onSentinelStartFocus);
      sentinelEnd?.removeEventListener("focus", onSentinelEndFocus);
      stopOpenSync();
    };
  }
});

// packages/ui-components/src/sui-drawer/sui-drawer.sui
var sui_drawer_default = component43("sui-drawer", {
  props: {
    open: prop43.bool({ default: false, reflect: true }),
    side: prop43.union(["right", "left"], { default: "right", reflect: true }),
    size: prop43.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    dismissible: prop43.bool({ default: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dr-w: 420px; --dr-px: 24px; --dr-hpy: 16px; --dr-bpy: 16px; --dr-fpy: 14px; }
  :host([size='sm']) { --dr-w: 320px; --dr-px: 20px; --dr-hpy: 14px; --dr-bpy: 14px; --dr-fpy: 12px; }
  :host([size='lg']) { --dr-w: 560px; --dr-px: 28px; --dr-hpy: 20px; --dr-bpy: 20px; --dr-fpy: 16px; }

  dialog {
    position: fixed;
    top: 0; bottom: 0;
    right: 0; left: auto;
    margin: 0; padding: 0;
    border: none;
    border-left: 1px solid var(--border, #e5e5e5);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: min(var(--dr-w, 420px), calc(100vw - 32px));
    max-width: 100vw;
    height: 100%;
    max-height: 100%;
    box-shadow: var(--shadow-xl, -4px 0 30px rgba(0,0,0,0.15));
    font-family: var(--font-body, system-ui);
    display: flex;
    flex-direction: column;
    opacity: 0;
    transform: translateX(24px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([side='left']) dialog {
    right: auto; left: 0;
    border-left: none;
    border-right: 1px solid var(--border, #e5e5e5);
    box-shadow: var(--shadow-xl, 4px 0 30px rgba(0,0,0,0.15));
    transform: translateX(-24px);
  }
  dialog[open] { opacity: 1; transform: translateX(0); }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    display: flex; align-items: flex-start; gap: var(--space-2, 8px);
    padding: var(--dr-hpy, 16px) var(--dr-px, 24px);
    padding-right: calc(var(--dr-px, 24px) + 36px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    flex: 0 0 auto; position: relative;
  }
  .header-content { flex: 1 1 0; min-width: 0; }
  .title {
    font-size: var(--text-base, 14px);
    font-weight: var(--font-weight-semibold, 600);
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .description {
    font-size: var(--text-sm, 12px);
    color: var(--muted-foreground, #888);
    margin-top: 2px;
    line-height: var(--leading-normal, 1.5);
  }

  .body {
    flex: 1 1 auto; min-height: 0;
    overflow-y: auto; overscroll-behavior: contain;
    padding: var(--dr-bpy, 16px) var(--dr-px, 24px);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    flex: 0 0 auto;
    padding: var(--dr-fpy, 14px) var(--dr-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex; gap: var(--space-2, 8px);
    justify-content: flex-end; align-items: center;
  }

  .close {
    position: absolute;
    top: 12px; inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }

  @media (prefers-reduced-motion: reduce) {
    dialog { transition: none; transform: none !important; opacity: 1; }
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.side = props.side;
    $scope.size = props.size;
    $scope.dismissible = props.dismissible;
    let __r = null;
    if (sui_drawer_controller_default && typeof sui_drawer_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_drawer_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets43(__root);
    const __e0 = document.createElement("dialog");
    __e0.setAttribute("part", "dialog");
    __e0.setAttribute("role", "dialog");
    __e0.setAttribute("aria-modal", "true");
    attachRef21($scope, "dialogEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("tabindex", "0");
    __e1.setAttribute("class", "sentinel");
    __e1.setAttribute("aria-hidden", "true");
    attachRef21($scope, "sentinelStart", __e1);
    __e0.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "header");
    __e2.setAttribute("part", "header");
    attachRef21($scope, "headerEl", __e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "header-content");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "description");
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "description");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    __e2.appendChild(__e3);
    const __e8 = document.createElement("button");
    __e8.setAttribute("class", "close");
    __e8.setAttribute("part", "close");
    __e8.setAttribute("type", "button");
    __e8.setAttribute("aria-label", "Close drawer");
    attachRef21($scope, "closeBtn", __e8);
    const __e9 = document.createElement("svg");
    __e9.setAttribute("viewbox", "0 0 16 16");
    __e9.setAttribute("fill", "none");
    __e9.setAttribute("stroke", "currentColor");
    __e9.setAttribute("stroke-width", "1.75");
    __e9.setAttribute("stroke-linecap", "round");
    __e9.setAttribute("stroke-linejoin", "round");
    __e9.setAttribute("aria-hidden", "true");
    __e9.setAttribute("focusable", "false");
    const __e10 = document.createElement("path");
    __e10.setAttribute("d", "M3.5 3.5l9 9M12.5 3.5l-9 9");
    __e9.appendChild(__e10);
    __e8.appendChild(__e9);
    __e2.appendChild(__e8);
    __e0.appendChild(__e2);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "body");
    __e11.setAttribute("part", "body");
    attachRef21($scope, "bodyEl", __e11);
    const __e12 = document.createElement("slot");
    __e11.appendChild(__e12);
    __e0.appendChild(__e11);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "footer");
    __e13.setAttribute("part", "footer");
    const __e14 = document.createElement("slot");
    __e14.setAttribute("name", "footer");
    __e13.appendChild(__e14);
    __e0.appendChild(__e13);
    const __e15 = document.createElement("span");
    __e15.setAttribute("tabindex", "0");
    __e15.setAttribute("class", "sentinel");
    __e15.setAttribute("aria-hidden", "true");
    attachRef21($scope, "sentinelEnd", __e15);
    __e0.appendChild(__e15);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-dropdown-menu/sui-dropdown-menu.sui
import { adoptSuiSheets as adoptSuiSheets44, attachRef as attachRef22, component as component44, prop as prop44 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-dropdown-menu/sui-dropdown-menu.controller.ts
import { controller as controller27, effect as effect27, bindNativeEvent as bindNativeEvent3 } from "@ship/sui-runtime";
var _seq5 = 0;
var sui_dropdown_menu_controller_default = controller27({
  mount(state, host) {
    const scope = state;
    const panelId = `sui-dropdown-menu-panel-${++_seq5}`;
    const triggerWrap = scope.triggerWrap();
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    let typeaheadBuf = "";
    let typeaheadTimer = null;
    function pushTypeahead(key) {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      typeaheadBuf += key.toLowerCase();
      typeaheadTimer = setTimeout(() => {
        typeaheadBuf = "";
      }, 350);
      const items = getItems();
      const i = items.indexOf(document.activeElement);
      for (let j = 0;j < items.length; j++) {
        const idx = (i + 1 + j) % items.length;
        const label = (items[idx].textContent ?? "").trim().toLowerCase();
        if (label.startsWith(typeaheadBuf))
          return idx;
      }
      return -1;
    }
    function getItems() {
      return Array.from(host.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((el) => !el.hasAttribute("disabled"));
    }
    function focusItem(index) {
      const items = getItems();
      if (!items.length)
        return;
      const i = (index % items.length + items.length) % items.length;
      for (const it of items)
        it.setAttribute("tabindex", "-1");
      items[i].setAttribute("tabindex", "0");
      items[i].focus();
    }
    function currentIndex() {
      return getItems().indexOf(document.activeElement);
    }
    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute("role"))
          el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex"))
          el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
      });
    }
    function setupTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-haspopup", "menu");
        el.setAttribute("aria-controls", panelId);
        el.setAttribute("aria-expanded", scope.open() ? "true" : "false");
      }
    }
    function openPanel() {
      const trigger = triggerSlot?.assignedElements({ flatten: true })?.[0];
      if (trigger && panelEl) {
        const r = trigger.getBoundingClientRect();
        panelEl.style.top = `${r.bottom + 4}px`;
        panelEl.style.left = `${r.left}px`;
      }
      panelEl?.showPopover?.();
    }
    function closePanel() {
      panelEl?.hidePopover?.();
    }
    const onTriggerClick = () => {
      if (scope.open())
        closePanel();
      else
        openPanel();
    };
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      setupTrigger();
      if (nowOpen)
        queueMicrotask(() => {
          setupItems();
          focusItem(0);
        });
    };
    const onItemClick = (e) => {
      const target = e.target.closest("[data-value]");
      if (!target)
        return;
      host.emit("sui-select", { value: target.dataset.value ?? "" });
      closePanel();
    };
    const onKeydown = (e) => {
      if (!scope.open())
        return;
      const items = getItems();
      if (!items.length)
        return;
      const i = currentIndex();
      if (e.key === "ArrowDown") {
        e.preventDefault();
        focusItem(i + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        focusItem(i - 1);
      } else if (e.key === "Home") {
        e.preventDefault();
        focusItem(0);
      } else if (e.key === "End") {
        e.preventDefault();
        focusItem(items.length - 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        closePanel();
      } else if (e.key === "Tab") {
        closePanel();
      } else if ((e.key === "Enter" || e.key === " ") && i >= 0) {
        e.preventDefault();
        items[i].click();
      } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key);
        if (next >= 0) {
          e.preventDefault();
          focusItem(next);
        }
      }
    };
    if (triggerWrap)
      bindNativeEvent3(triggerWrap, "click", onTriggerClick);
    bindNativeEvent3(panelEl, "toggle", onToggle);
    bindNativeEvent3(panelEl, "click", onItemClick);
    bindNativeEvent3(host, "keydown", onKeydown);
    if (triggerSlot)
      bindNativeEvent3(triggerSlot, "slotchange", setupTrigger);
    bindNativeEvent3(panelEl, "slotchange", setupItems);
    const stopSync = effect27(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      setupTrigger();
    });
    return () => {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      stopSync();
    };
  }
});

// packages/ui-components/src/sui-dropdown-menu/sui-dropdown-menu.sui
var sui_dropdown_menu_default = component44("sui-dropdown-menu", {
  props: {
    open: prop44.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { position: relative; display: inline-flex; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: translateY(-4px) scale(0.97);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    let __r = null;
    if (sui_dropdown_menu_controller_default && typeof sui_dropdown_menu_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_dropdown_menu_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets44(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "trigger-wrap");
    attachRef22($scope, "triggerWrap", __e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "trigger");
    attachRef22($scope, "triggerSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "panel");
    __e2.setAttribute("part", "panel");
    __e2.setAttribute("popover", "auto");
    __e2.setAttribute("role", "menu");
    attachRef22($scope, "panelEl", __e2);
    const __e3 = document.createElement("slot");
    attachRef22($scope, "itemSlot", __e3);
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-context-menu/sui-context-menu.sui
import { adoptSuiSheets as adoptSuiSheets45, attachRef as attachRef23, component as component45, prop as prop45 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-context-menu/sui-context-menu.controller.ts
import { controller as controller28, effect as effect28, bindNativeEvent as bindNativeEvent4 } from "@ship/sui-runtime";
var _seq6 = 0;
var sui_context_menu_controller_default = controller28({
  mount(state, host) {
    const scope = state;
    const panelId = `sui-context-menu-panel-${++_seq6}`;
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    let typeaheadBuf = "";
    let typeaheadTimer = null;
    function pushTypeahead(key) {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      typeaheadBuf += key.toLowerCase();
      typeaheadTimer = setTimeout(() => {
        typeaheadBuf = "";
      }, 350);
      const items = getItems();
      const i = items.indexOf(document.activeElement);
      for (let j = 0;j < items.length; j++) {
        const idx = (i + 1 + j) % items.length;
        const label = (items[idx].textContent ?? "").trim().toLowerCase();
        if (label.startsWith(typeaheadBuf))
          return idx;
      }
      return -1;
    }
    function getItems() {
      return Array.from(host.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((el) => !el.hasAttribute("disabled"));
    }
    function focusItem(index) {
      const items = getItems();
      if (!items.length)
        return;
      const i = (index % items.length + items.length) % items.length;
      for (const it of items)
        it.setAttribute("tabindex", "-1");
      items[i].setAttribute("tabindex", "0");
      items[i].focus();
    }
    function currentIndex() {
      return getItems().indexOf(document.activeElement);
    }
    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute("role"))
          el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex"))
          el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
      });
    }
    function doShow(x, y) {
      if (panelEl) {
        panelEl.style.top = `${y}px`;
        panelEl.style.left = `${x}px`;
      }
      panelEl?.showPopover?.();
    }
    function doClose() {
      panelEl?.hidePopover?.();
    }
    host.expose({ show: doShow, close: doClose });
    const onContextMenu = (e) => {
      e.preventDefault();
      doShow(e.clientX, e.clientY);
    };
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      if (nowOpen) {
        host.emit("sui-open");
        queueMicrotask(() => {
          setupItems();
          focusItem(0);
        });
      } else {
        host.emit("sui-close");
      }
    };
    const onItemClick = (e) => {
      const target = e.target.closest("[data-value]");
      if (!target)
        return;
      host.emit("sui-select", { value: target.dataset.value ?? "" });
      doClose();
    };
    const onKeydown = (e) => {
      if (!scope.open())
        return;
      const items = getItems();
      if (!items.length)
        return;
      const i = currentIndex();
      if (e.key === "ArrowDown") {
        e.preventDefault();
        focusItem(i + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        focusItem(i - 1);
      } else if (e.key === "Home") {
        e.preventDefault();
        focusItem(0);
      } else if (e.key === "End") {
        e.preventDefault();
        focusItem(items.length - 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        doClose();
      } else if (e.key === "Tab") {
        doClose();
      } else if ((e.key === "Enter" || e.key === " ") && i >= 0) {
        e.preventDefault();
        items[i].click();
      } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key);
        if (next >= 0) {
          e.preventDefault();
          focusItem(next);
        }
      }
    };
    const wireTrigger = () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.addEventListener("contextmenu", onContextMenu);
      }
    };
    if (triggerSlot)
      bindNativeEvent4(triggerSlot, "slotchange", wireTrigger);
    wireTrigger();
    bindNativeEvent4(panelEl, "toggle", onToggle);
    bindNativeEvent4(panelEl, "click", onItemClick);
    bindNativeEvent4(host, "keydown", onKeydown);
    const stopSync = effect28(() => {
      host.setAttribute("data-state", scope.open() ? "open" : "closed");
    });
    return () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els)
        el.removeEventListener("contextmenu", onContextMenu);
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      stopSync();
    };
  }
});

// packages/ui-components/src/sui-context-menu/sui-context-menu.sui
var sui_context_menu_default = component45("sui-context-menu", {
  props: {
    open: prop45.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: scale(0.96);
    transition:
      opacity var(--duration-fast, 120ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    let __r = null;
    if (sui_context_menu_controller_default && typeof sui_context_menu_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_context_menu_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets45(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef23($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    __e1.setAttribute("role", "menu");
    __e1.setAttribute("aria-label", "Context menu");
    attachRef23($scope, "panelEl", __e1);
    const __e2 = document.createElement("slot");
    attachRef23($scope, "itemSlot", __e2);
    __e1.appendChild(__e2);
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

// packages/ui-components/src/sui-hover-card/sui-hover-card.sui
import { adoptSuiSheets as adoptSuiSheets46, attachRef as attachRef24, component as component46, prop as prop46 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-hover-card/sui-hover-card.controller.ts
import { controller as controller29, effect as effect29 } from "@ship/sui-runtime";
var _seq7 = 0;
var GAP = 8;
function positionPanel(trigger, panel, placement) {
  const r = trigger.getBoundingClientRect();
  panel.style.top = "";
  panel.style.left = "";
  panel.style.right = "";
  panel.style.bottom = "";
  switch (placement) {
    case "top":
      panel.style.bottom = `${window.innerHeight - r.top + GAP}px`;
      panel.style.left = `${r.left + r.width / 2}px`;
      panel.style.transform = "translateX(-50%)";
      break;
    case "left":
      panel.style.right = `${window.innerWidth - r.left + GAP}px`;
      panel.style.top = `${r.top}px`;
      break;
    case "right":
      panel.style.left = `${r.right + GAP}px`;
      panel.style.top = `${r.top}px`;
      break;
    case "bottom":
    default:
      panel.style.top = `${r.bottom + GAP}px`;
      panel.style.left = `${r.left + r.width / 2}px`;
      panel.style.transform = "translateX(-50%)";
  }
}
var sui_hover_card_controller_default = controller29({
  mount(state, host) {
    const scope = state;
    const cardId = `sui-hover-card-${++_seq7}`;
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = cardId;
    let showTimer = null;
    let hideTimer = null;
    function clearTimers() {
      if (showTimer) {
        clearTimeout(showTimer);
        showTimer = null;
      }
      if (hideTimer) {
        clearTimeout(hideTimer);
        hideTimer = null;
      }
    }
    function wireTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-describedby", cardId);
      }
    }
    function doShow() {
      clearTimers();
      const trigger = triggerSlot?.assignedElements({ flatten: true })?.[0];
      if (trigger && panelEl) {
        positionPanel(trigger, panelEl, scope.placement() || "bottom");
      }
      panelEl?.showPopover?.();
    }
    function doHide() {
      clearTimers();
      panelEl?.hidePopover?.();
    }
    host.show = doShow;
    host.hide = doHide;
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      host.dispatchEvent(new CustomEvent(nowOpen ? "sui-open" : "sui-close", {
        bubbles: true,
        composed: true
      }));
    };
    const onPointerEnter = () => {
      clearTimers();
      showTimer = setTimeout(() => doShow(), scope.showDelay() ?? 300);
    };
    const onPointerLeave = () => {
      clearTimers();
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100);
    };
    const onFocusIn = () => {
      clearTimers();
      doShow();
    };
    const onFocusOut = () => {
      clearTimers();
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100);
    };
    panelEl.addEventListener("toggle", onToggle);
    triggerSlot?.addEventListener("slotchange", wireTrigger);
    wireTrigger();
    host.addEventListener("pointerenter", onPointerEnter);
    host.addEventListener("pointerleave", onPointerLeave);
    host.addEventListener("focusin", onFocusIn);
    host.addEventListener("focusout", onFocusOut);
    const stopSync = effect29(() => {
      host.setAttribute("data-state", scope.open() ? "open" : "closed");
    });
    return () => {
      clearTimers();
      panelEl.removeEventListener("toggle", onToggle);
      triggerSlot?.removeEventListener("slotchange", wireTrigger);
      host.removeEventListener("pointerenter", onPointerEnter);
      host.removeEventListener("pointerleave", onPointerLeave);
      host.removeEventListener("focusin", onFocusIn);
      host.removeEventListener("focusout", onFocusOut);
      stopSync();
    };
  }
});

// packages/ui-components/src/sui-hover-card/sui-hover-card.sui
var sui_hover_card_default = component46("sui-hover-card", {
  props: {
    open: prop46.bool({ default: false, reflect: true }),
    placement: prop46.union(["top", "bottom", "left", "right"], { default: "bottom", reflect: true }),
    "show-delay": prop46.number({ default: 300 }),
    "hide-delay": prop46.number({ default: 100 })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; position: relative; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 240px;
    max-width: 340px;
    padding: 16px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0);
  }
  .panel::backdrop { background: transparent; }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.placement = props.placement;
    $scope.showDelay = props.showDelay;
    $scope.hideDelay = props.hideDelay;
    let __r = null;
    if (sui_hover_card_controller_default && typeof sui_hover_card_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_hover_card_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets46(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef24($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    attachRef24($scope, "panelEl", __e1);
    const __e2 = document.createElement("slot");
    __e1.appendChild(__e2);
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

// packages/ui-components/src/sui-accordion/sui-accordion.sui
import { adoptSuiSheets as adoptSuiSheets47, component as component47, prop as prop47 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-accordion/sui-accordion.controller.ts
import { controller as controller30, effect as effect30 } from "@ship/sui-runtime";
var sui_accordion_controller_default = controller30({
  mount(state, host) {
    const scope = state;
    const getItems = () => Array.from(host.querySelectorAll(":scope > sui-accordion-item"));
    const getEnabledItems = () => getItems().filter((i) => !i.hasAttribute("disabled"));
    const getOpenValues = () => {
      const v = scope.value();
      return new Set(v ? v.split(",").filter(Boolean) : []);
    };
    const syncItems = () => {
      const open = getOpenValues();
      for (const item of getItems()) {
        const val = item.getAttribute("value") || "";
        if (open.has(val)) {
          item.setAttribute("open", "");
        } else {
          item.removeAttribute("open");
        }
      }
    };
    const stopEffect = effect30(() => {
      scope.value();
      syncItems();
    });
    const onToggle = (e) => {
      const ev = e;
      ev.stopPropagation();
      const itemVal = ev.detail?.value;
      const open = getOpenValues();
      if (scope.type() === "single") {
        scope.value.set(open.has(itemVal) ? "" : itemVal);
      } else {
        if (open.has(itemVal)) {
          open.delete(itemVal);
        } else {
          open.add(itemVal);
        }
        scope.value.set(Array.from(open).join(","));
      }
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
    };
    const onNav = (e) => {
      const ev = e;
      ev.stopPropagation();
      const enabled = getEnabledItems();
      if (!enabled.length)
        return;
      const current = ev.target?.closest("sui-accordion-item");
      const idx = current ? enabled.indexOf(current) : -1;
      let target;
      switch (ev.detail?.direction) {
        case "next":
          target = enabled[(idx + 1) % enabled.length];
          break;
        case "prev":
          target = enabled[(idx - 1 + enabled.length) % enabled.length];
          break;
        case "first":
          target = enabled[0];
          break;
        case "last":
          target = enabled[enabled.length - 1];
          break;
      }
      if (target) {
        const btn = target.shadowRoot?.querySelector("button");
        btn?.focus();
      }
    };
    host.addEventListener("sui-accordion-item-toggle", onToggle);
    host.addEventListener("sui-accordion-nav", onNav);
    return () => {
      stopEffect();
      host.removeEventListener("sui-accordion-item-toggle", onToggle);
      host.removeEventListener("sui-accordion-nav", onNav);
    };
  }
});

// packages/ui-components/src/sui-accordion/sui-accordion.sui
var sui_accordion_default = component47("sui-accordion", {
  props: {
    type: prop47.string({ default: "single", reflect: true }),
    value: prop47.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    let __r = null;
    if (sui_accordion_controller_default && typeof sui_accordion_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_accordion_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets47(__root);
    const __e0 = document.createElement("slot");
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-accordion/sui-accordion-item.sui
import { adoptSuiSheets as adoptSuiSheets48, attachRef as attachRef25, bindAttr as bindAttr25, component as component48, prop as prop48 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-accordion/sui-accordion-item.controller.ts
import { controller as controller31, effect as effect31 } from "@ship/sui-runtime";
var _seq8 = 0;
var sui_accordion_item_controller_default = controller31({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq8;
    const triggerId = `sui-acc-trigger-${uid}`;
    const contentId = `sui-acc-content-${uid}`;
    const stopIds = effect31(() => {
      const btn = scope.triggerBtn();
      const wrap = scope.contentWrap();
      if (btn) {
        btn.id = triggerId;
        btn.setAttribute("aria-controls", contentId);
      }
      if (wrap) {
        wrap.id = contentId;
        wrap.setAttribute("aria-labelledby", triggerId);
      }
    });
    const stopAriaExpanded = effect31(() => {
      const btn = scope.triggerBtn();
      if (btn)
        btn.setAttribute("aria-expanded", scope.open() ? "true" : "false");
    });
    const onClick = () => {
      if (scope.disabled())
        return;
      host.dispatchEvent(new CustomEvent("sui-accordion-item-toggle", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
    };
    const onKeydown = (e) => {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("sui-accordion-nav", {
          detail: { direction: e.key === "ArrowDown" ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      } else if (e.key === "Home" || e.key === "End") {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("sui-accordion-nav", {
          detail: { direction: e.key === "Home" ? "first" : "last" },
          bubbles: true,
          composed: true
        }));
      }
    };
    const stopBtn = effect31(() => {
      const btn = scope.triggerBtn();
      if (!btn)
        return;
      btn.addEventListener("click", onClick);
      btn.addEventListener("keydown", onKeydown);
      return () => {
        btn.removeEventListener("click", onClick);
        btn.removeEventListener("keydown", onKeydown);
      };
    });
    return () => {
      stopIds();
      stopAriaExpanded();
      stopBtn();
    };
  }
});

// packages/ui-components/src/sui-accordion/sui-accordion-item.sui
var sui_accordion_item_default = component48("sui-accordion-item", {
  props: {
    value: prop48.string({ default: "", reflect: true }),
    disabled: prop48.bool({ default: false, reflect: true }),
    open: prop48.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .trigger {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 0;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border);
    text-align: left;
    font: inherit;
    font-size: var(--text-base);
    font-weight: var(--font-weight-medium);
    color: var(--foreground);
    cursor: pointer;
    gap: 8px;
  }
  .trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .trigger:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
  @media (forced-colors: active) {
    .trigger:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  .trigger-label { flex: 1; }

  .chevron {
    flex-shrink: 0;
    width: 16px;
    height: 16px;
    color: var(--muted-foreground);
    transition: transform var(--duration-medium) var(--ease-out);
  }
  :host([open]) .chevron { transform: rotate(180deg); }
  @media (prefers-reduced-motion: reduce) { .chevron { transition: none; } }

  .content-wrap {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--duration-medium) var(--ease-out);
  }
  :host([open]) .content-wrap { grid-template-rows: 1fr; }
  @media (prefers-reduced-motion: reduce) { .content-wrap { transition: none; } }

  .content-inner { overflow: hidden; }
  .content-body { padding: 12px 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.disabled = props.disabled;
    $scope.open = props.open;
    let __r = null;
    if (sui_accordion_item_controller_default && typeof sui_accordion_item_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_accordion_item_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets48(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("class", "trigger");
    __e0.setAttribute("part", "trigger");
    attachRef25($scope, "triggerBtn", __e0);
    __cleanups.push(bindAttr25(__e0, "disabled", () => $scope.disabled()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "trigger-label");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "trigger");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("svg");
    __e3.setAttribute("class", "chevron");
    __e3.setAttribute("part", "chevron");
    __e3.setAttribute("viewbox", "0 0 16 16");
    __e3.setAttribute("fill", "none");
    __e3.setAttribute("stroke", "currentColor");
    __e3.setAttribute("stroke-width", "1.75");
    __e3.setAttribute("stroke-linecap", "round");
    __e3.setAttribute("stroke-linejoin", "round");
    __e3.setAttribute("aria-hidden", "true");
    const __e4 = document.createElement("path");
    __e4.setAttribute("d", "M4 6l4 4 4-4");
    __e3.appendChild(__e4);
    __e0.appendChild(__e3);
    __root.appendChild(__e0);
    const __e5 = document.createElement("div");
    __e5.setAttribute("class", "content-wrap");
    __e5.setAttribute("part", "content");
    attachRef25($scope, "contentWrap", __e5);
    __e5.setAttribute("role", "region");
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "content-inner");
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "content-body");
    const __e8 = document.createElement("slot");
    __e7.appendChild(__e8);
    __e6.appendChild(__e7);
    __e5.appendChild(__e6);
    __root.appendChild(__e5);
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

// packages/ui-components/src/sui-list-tile/sui-list-tile.sui
import { adoptSuiSheets as adoptSuiSheets49, attachRef as attachRef26, bindAttr as bindAttr26, bindIf as bindIf12, component as component49, prop as prop49 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-list-tile/sui-list-tile.controller.ts
import { controller as controller32, effect as effect32 } from "@ship/sui-runtime";
var DIVIDER_CSS = `
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`;
var _dividerInstalled = new WeakSet;
function installDividerStyles(doc) {
  if (_dividerInstalled.has(doc))
    return;
  _dividerInstalled.add(doc);
  try {
    if (typeof CSSStyleSheet !== "undefined" && "replaceSync" in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet;
      sheet.replaceSync(DIVIDER_CSS);
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      return;
    }
  } catch {}
  const style = doc.createElement("style");
  style.setAttribute("data-sui-list-tile-dividers", "");
  style.textContent = DIVIDER_CSS;
  doc.head.appendChild(style);
}
var sui_list_tile_controller_default = controller32({
  mount(state, host) {
    const scope = state;
    installDividerStyles(host.ownerDocument);
    const resolveRole = () => {
      if (scope.navigable())
        return "link";
      if (scope.interactive())
        return "button";
      return "listitem";
    };
    const stopRole = effect32(() => {
      scope.navigable();
      scope.interactive();
      host.setAttribute("role", resolveRole());
    });
    const stopSelected = effect32(() => {
      host.setAttribute("aria-selected", scope.selected() ? "true" : "false");
    });
    const stopDisabled = effect32(() => {
      host.setAttribute("aria-disabled", scope.disabled() ? "true" : "false");
    });
    const stopTabindex = effect32(() => {
      const el = scope.tileEl();
      if (!el)
        return;
      const interactive = scope.interactive();
      const disabled = scope.disabled();
      if (interactive && !disabled) {
        el.setAttribute("tabindex", "0");
      } else {
        el.removeAttribute("tabindex");
      }
    });
    const dispatchSelect = () => {
      if (scope.disabled() || !scope.interactive())
        return false;
      const ev = new CustomEvent("sui-select", {
        detail: {},
        bubbles: true,
        composed: true,
        cancelable: true
      });
      host.dispatchEvent(ev);
      return ev.defaultPrevented;
    };
    const onClick = (e) => {
      if (!scope.interactive())
        return;
      const prevented = dispatchSelect();
      if (prevented && scope.navigable()) {
        e.preventDefault();
      }
    };
    const onKeyDown = (e) => {
      if (!scope.interactive() || scope.disabled())
        return;
      if (scope.navigable() && e.key === "Enter")
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        dispatchSelect();
      }
    };
    const stopHandlers = effect32(() => {
      const el = scope.tileEl();
      if (!el)
        return;
      el.addEventListener("click", onClick);
      el.addEventListener("keydown", onKeyDown);
      return () => {
        el.removeEventListener("click", onClick);
        el.removeEventListener("keydown", onKeyDown);
      };
    });
    return () => {
      stopRole();
      stopSelected();
      stopDisabled();
      stopTabindex();
      stopHandlers();
    };
  }
});

// packages/ui-components/src/sui-list-tile/sui-list-tile.sui
var sui_list_tile_default = component49("sui-list-tile", {
  props: {
    variant: prop49.string({ default: "row", reflect: true }),
    interactive: prop49.bool({ default: false, reflect: true }),
    navigable: prop49.string({ default: "", reflect: true }),
    selected: prop49.bool({ default: false, reflect: true }),
    disabled: prop49.bool({ default: false, reflect: true }),
    density: prop49.string({ default: "normal", reflect: true }),
    tone: prop49.string({ default: "default", reflect: true }),
    target: prop49.string({ default: "", reflect: true }),
    rel: prop49.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --pad-y: 10px;
    --pad-x: 10px;
  }
  :host([density='compact']) { --pad-y: 6px;  --pad-x: 8px; }
  :host([density='relaxed']) { --pad-y: 16px; --pad-x: 14px; }

  .tile {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: var(--pad-y) var(--pad-x);
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: inherit;
    font: inherit;
    text-decoration: none;
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  a.tile { display: flex; }

  :host([variant='card']) { height: 100%; box-sizing: border-box; }
  :host([variant='card']) .tile {
    min-height: 100%;
    box-sizing: border-box;
  }
  :host([variant='card']) .row { flex: 1; align-items: stretch; }
  :host([variant='card']) ::slotted([slot='leading']) { align-self: flex-start; }
  :host([variant='card']) .primary {
    display: flex;
    flex-direction: column;
    align-self: stretch;
  }

  :host([variant='row']) .tile:hover,
  :host(:not([variant])) .tile:hover {
    background: var(--muted);
    border-color: var(--border);
  }

  :host([variant='card']) .tile {
    background: var(--card);
    border-color: var(--border-subtle);
    box-shadow: var(--shadow-card-rest);
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    position: relative;
  }
  :host([variant='card']) .tile::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: var(--card-texture);
    mix-blend-mode: var(--card-texture-blend);
    opacity: var(--card-texture-opacity);
    border-radius: inherit;
    z-index: 0;
  }
  :host([variant='card']) .tile > * { position: relative; z-index: 1; }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile::after { background: none; }
  }
  :host([variant='card']) .tile:hover {
    border-color: var(--primary);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile:hover { transform: none; }
  }
  :host([variant='card'][interactive]) .tile:active {
    transform: translateY(0);
    box-shadow: var(--shadow-card-rest);
    transition-duration: 60ms;
  }

  .tile:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  @media (forced-colors: active) {
    .tile:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  :host([selected]) .tile {
    background: var(--muted);
    border-color: var(--primary);
  }

  :host([disabled]) .tile {
    opacity: 0.6;
    cursor: not-allowed;
    pointer-events: none;
  }

  :host([interactive]) .tile { cursor: pointer; }

  :host([tone]:not([tone='default'])) .tile {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--pad-x) - 2px);
  }

  /* tone-fg mapping */
  :host { --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .row { display: flex; align-items: center; gap: 8px; }
  .primary {
    flex: 1;
    min-width: 0;
    font-weight: var(--font-weight-medium);
    font-size: var(--text-base);
  }
  .secondary {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 10px;
    color: var(--muted-foreground);
  }
  .secondary:empty { display: none; }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.interactive = props.interactive;
    $scope.navigable = props.navigable;
    $scope.selected = props.selected;
    $scope.disabled = props.disabled;
    $scope.density = props.density;
    $scope.tone = props.tone;
    $scope.target = props.target;
    $scope.rel = props.rel;
    let __r = null;
    if (sui_list_tile_controller_default && typeof sui_list_tile_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_list_tile_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf12({
      anchor,
      test: () => {
        if ($scope.navigable())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("a");
          __e3.setAttribute("class", "tile");
          __e3.setAttribute("part", "button");
          attachRef26($scope, "tileEl", __e3);
          __c2.push(bindAttr26(__e3, "href", () => $scope.navigable()));
          __c2.push(bindAttr26(__e3, "target", () => $scope.target() || null));
          __c2.push(bindAttr26(__e3, "rel", () => $scope.rel() || null));
          const __e4 = document.createElement("div");
          __e4.setAttribute("class", "row");
          __e4.setAttribute("part", "row");
          const __e5 = document.createElement("slot");
          __e5.setAttribute("name", "leading");
          __e4.appendChild(__e5);
          const __e6 = document.createElement("span");
          __e6.setAttribute("class", "primary");
          __e6.setAttribute("part", "primary");
          const __e7 = document.createElement("slot");
          __e6.appendChild(__e7);
          __e4.appendChild(__e6);
          const __e8 = document.createElement("span");
          __e8.setAttribute("part", "trailing");
          const __e9 = document.createElement("slot");
          __e9.setAttribute("name", "trailing");
          __e8.appendChild(__e9);
          __e4.appendChild(__e8);
          __e3.appendChild(__e4);
          const __e10 = document.createElement("div");
          __e10.setAttribute("class", "secondary");
          __e10.setAttribute("part", "secondary");
          const __e11 = document.createElement("slot");
          __e11.setAttribute("name", "meta");
          __e10.appendChild(__e11);
          __e3.appendChild(__e10);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "tile");
          __e14.setAttribute("part", "button");
          attachRef26($scope, "tileEl", __e14);
          const __e15 = document.createElement("div");
          __e15.setAttribute("class", "row");
          __e15.setAttribute("part", "row");
          const __e16 = document.createElement("slot");
          __e16.setAttribute("name", "leading");
          __e15.appendChild(__e16);
          const __e17 = document.createElement("span");
          __e17.setAttribute("class", "primary");
          __e17.setAttribute("part", "primary");
          const __e18 = document.createElement("slot");
          __e17.appendChild(__e18);
          __e15.appendChild(__e17);
          const __e19 = document.createElement("span");
          __e19.setAttribute("part", "trailing");
          const __e20 = document.createElement("slot");
          __e20.setAttribute("name", "trailing");
          __e19.appendChild(__e20);
          __e15.appendChild(__e19);
          __e14.appendChild(__e15);
          const __e21 = document.createElement("div");
          __e21.setAttribute("class", "secondary");
          __e21.setAttribute("part", "secondary");
          const __e22 = document.createElement("slot");
          __e22.setAttribute("name", "meta");
          __e21.appendChild(__e22);
          __e14.appendChild(__e21);
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
    adoptSuiSheets49(__root);
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

// packages/ui-components/src/sui-error-state/sui-error-state.sui
import { adoptSuiSheets as adoptSuiSheets50, attachRef as attachRef27, bindAttr as bindAttr27, bindText as bindText12, component as component50, prop as prop50 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-error-state/sui-error-state.controller.ts
import { controller as controller33, effect as effect33 } from "@ship/sui-runtime";
var sui_error_state_controller_default = controller33({
  mount(state, host) {
    const scope = state;
    let detailsOpen = false;
    const updateToggleLabel = (btn) => {
      btn.textContent = detailsOpen ? "Hide details" : "Show details";
      btn.setAttribute("aria-expanded", detailsOpen ? "true" : "false");
    };
    const stopDetailsSlot = effect33(() => {
      const slot = scope.detailsSlot();
      if (!slot)
        return;
      const check = () => {
        const hasContent = slot.assignedNodes({ flatten: true }).some((n) => n.nodeType === Node.ELEMENT_NODE || n.nodeType === Node.TEXT_NODE && (n.textContent ?? "").trim().length > 0);
        if (hasContent) {
          host.setAttribute("data-has-details", "");
        } else {
          host.removeAttribute("data-has-details");
          host.removeAttribute("data-details-open");
          detailsOpen = false;
          const btn = scope.detailsToggleBtn();
          if (btn)
            updateToggleLabel(btn);
        }
      };
      slot.addEventListener("slotchange", check);
      check();
      return () => slot.removeEventListener("slotchange", check);
    });
    const stopToggleInit = effect33(() => {
      const btn = scope.detailsToggleBtn();
      if (!btn)
        return;
      updateToggleLabel(btn);
      const onClick = () => {
        detailsOpen = !detailsOpen;
        if (detailsOpen) {
          host.setAttribute("data-details-open", "");
        } else {
          host.removeAttribute("data-details-open");
        }
        updateToggleLabel(btn);
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    const stopActionSlot = effect33(() => {
      const slot = scope.actionSlot();
      if (!slot)
        return;
      const onSlotChange = () => {
        const retry = slot.assignedElements().find((el) => el.getAttribute("slot") === "retry");
        if (retry)
          requestAnimationFrame(() => retry.focus?.());
      };
      slot.addEventListener("slotchange", onSlotChange);
      return () => slot.removeEventListener("slotchange", onSlotChange);
    });
    return () => {
      stopDetailsSlot();
      stopToggleInit();
      stopActionSlot();
    };
  }
});

// packages/ui-components/src/sui-error-state/sui-error-state.sui
var sui_error_state_default = component50("sui-error-state", {
  props: {
    title: prop50.string({ default: "" }),
    body: prop50.string({ default: "" }),
    size: prop50.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop50.union(["default", "inline"], { default: "default", reflect: true }),
    icon: prop50.string({ default: "alert-circle" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    --tone-fg: var(--destructive);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  .box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    background: var(--card);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
    background: color-mix(in oklch, var(--destructive) 4%, var(--card));
    border: 1px solid color-mix(in oklch, var(--destructive) 24%, var(--border));
    border-radius: var(--radius-md);
    padding: var(--space-3, 12px) var(--space-4, 16px);
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--destructive) 18%, var(--muted));
    color: var(--destructive);
    line-height: 1;
  }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-size: var(--es-title-fs);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 60ch;
    color: var(--muted-foreground);
  }

  /* details-toggle: hidden until controller detects slot content */
  .details-toggle {
    display: none;
    appearance: none;
    background: transparent;
    border: 0;
    font: inherit;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    margin-top: var(--space-1);
  }
  :host([data-has-details]) .details-toggle { display: inline-block; }
  .details-toggle:hover { color: var(--foreground); background: var(--muted); }

  /* details panel: always in DOM for slotchange detection, shown when open */
  .details {
    display: none;
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: color-mix(in oklch, var(--destructive) 6%, var(--muted));
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    white-space: pre-wrap;
    word-break: break-word;
    max-width: 60ch;
    text-align: left;
  }
  :host([data-has-details][data-details-open]) .details { display: block; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.title = props.title;
    $scope.body = props.body;
    $scope.size = props.size;
    $scope.variant = props.variant;
    $scope.icon = props.icon;
    let __r = null;
    if (sui_error_state_controller_default && typeof sui_error_state_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_error_state_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets50(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __e0.setAttribute("role", "alert");
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "icon");
    __e1.setAttribute("part", "icon");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElement("sui-icon");
    __cleanups.push(bindAttr27(__e2, "name", () => $scope.icon()));
    __e2.setAttribute("size", "md");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "copy");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    __e4.setAttribute("part", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText12(__t6, () => $scope.title() || "Something went wrong"));
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "desc");
    __e7.setAttribute("part", "desc");
    const __e8 = document.createElement("slot");
    const __t9 = document.createTextNode("");
    __e8.appendChild(__t9);
    __cleanups.push(bindText12(__t9, () => $scope.body()));
    __e7.appendChild(__e8);
    __e3.appendChild(__e7);
    const __e10 = document.createElement("button");
    __e10.setAttribute("class", "details-toggle");
    attachRef27($scope, "detailsToggleBtn", __e10);
    __e10.setAttribute("type", "button");
    __e3.appendChild(__e10);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "details");
    attachRef27($scope, "detailsPanel", __e11);
    const __e12 = document.createElement("slot");
    __e12.setAttribute("name", "details");
    attachRef27($scope, "detailsSlot", __e12);
    __e11.appendChild(__e12);
    __e3.appendChild(__e11);
    __e0.appendChild(__e3);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "actions");
    __e13.setAttribute("part", "actions");
    const __e14 = document.createElement("slot");
    __e14.setAttribute("name", "action");
    attachRef27($scope, "actionSlot", __e14);
    __e13.appendChild(__e14);
    __e0.appendChild(__e13);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-stepper/sui-stepper.sui
import { adoptSuiSheets as adoptSuiSheets51, attachRef as attachRef28, component as component51, prop as prop51 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-stepper/sui-stepper.controller.ts
import { controller as controller34, effect as effect34 } from "@ship/sui-runtime";
var sui_stepper_controller_default = controller34({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "progressbar");
    const stopEffect = effect34(() => {
      const t = Math.max(1, Math.floor(scope.total() || 0));
      const c = Math.max(1, Math.min(t, Math.floor(scope.current() || 1)));
      const label = scope.label();
      host.setAttribute("aria-valuemin", "1");
      host.setAttribute("aria-valuemax", String(t));
      host.setAttribute("aria-valuenow", String(c));
      if (label) {
        host.setAttribute("aria-label", label);
      } else {
        host.setAttribute("aria-label", `Step ${c} of ${t}`);
      }
      const container = scope.dotsEl();
      if (!container)
        return;
      container.textContent = "";
      for (let i = 1;i <= t; i++) {
        const dot = document.createElement("span");
        dot.className = "dot";
        dot.setAttribute("part", "dot");
        dot.setAttribute("data-state", i === c ? "active" : i < c ? "done" : "pending");
        dot.setAttribute("aria-hidden", "true");
        container.appendChild(dot);
      }
    });
    return () => {
      stopEffect();
    };
  }
});

// packages/ui-components/src/sui-stepper/sui-stepper.sui
var sui_stepper_default = component51("sui-stepper", {
  props: {
    total: prop51.number({ default: 4, reflect: true }),
    current: prop51.number({ default: 1, reflect: true }),
    tone: prop51.string({ default: "", reflect: true }),
    size: prop51.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop51.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2, 8px);
    --dot-w: 7px;
    --dot-h: 7px;
    --dot-active-w: 22px;
    --tone-fg: var(--primary);
  }

  :host([size='sm']) { --dot-w: 6px; --dot-h: 6px; --dot-active-w: 18px; }
  :host([size='lg']) { --dot-w: 9px; --dot-h: 9px; --dot-active-w: 28px; }

  /* Tone overrides */
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .dot {
    width: var(--dot-w);
    height: var(--dot-h);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--muted-foreground) 35%, transparent);
    transition: all var(--duration-medium) var(--ease-out);
    flex-shrink: 0;
  }

  .dot[data-state='active'] {
    width: var(--dot-active-w);
    background: var(--primary);
  }
  :host([tone]) .dot[data-state='active'] {
    background: var(--tone-fg);
  }

  .dot[data-state='done'] {
    background: color-mix(in oklch, var(--primary) 55%, transparent);
  }
  :host([tone]) .dot[data-state='done'] {
    background: color-mix(in oklch, var(--tone-fg) 55%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    .dot { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.total = props.total;
    $scope.current = props.current;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.label = props.label;
    let __r = null;
    if (sui_stepper_controller_default && typeof sui_stepper_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_stepper_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets51(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "dots");
    attachRef28($scope, "dotsEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-timeline/sui-timeline.sui
import { adoptSuiSheets as adoptSuiSheets52, component as component52 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-timeline/sui-timeline.controller.ts
import { controller as controller35 } from "@ship/sui-runtime";
var sui_timeline_controller_default = controller35({
  mount(_state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "list");
  }
});

// packages/ui-components/src/sui-timeline/sui-timeline.sui
var sui_timeline_default = component52("sui-timeline", {
  shadow: true,
  styles: `
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_timeline_controller_default && typeof sui_timeline_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_timeline_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets52(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "list");
    __e0.setAttribute("part", "list");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-timeline/sui-timeline-item.sui
import { adoptSuiSheets as adoptSuiSheets53, bindIf as bindIf13, bindText as bindText13, component as component53, prop as prop52 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-timeline/sui-timeline-item.controller.ts
import { controller as controller36, effect as effect35 } from "@ship/sui-runtime";
var sui_timeline_item_controller_default = controller36({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "listitem");
    const stopRole = effect35(() => {
      if (scope.interactive()) {
        host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
      } else {
        host.setAttribute("role", "listitem");
        host.removeAttribute("tabindex");
      }
    });
    const emit = () => {
      host.dispatchEvent(new CustomEvent("sui-select", { bubbles: true, composed: true, detail: {} }));
    };
    const onClick = () => {
      if (scope.interactive())
        emit();
    };
    const onKeydown = (e) => {
      if (!scope.interactive())
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        emit();
      }
    };
    host.addEventListener("click", onClick);
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopRole();
      host.removeEventListener("click", onClick);
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/ui-components/src/sui-timeline/sui-timeline-item.sui
var sui_timeline_item_default = component53("sui-timeline-item", {
  props: {
    tone: prop52.string({ default: "default", reflect: true }),
    at: prop52.string({ default: "", reflect: true }),
    interactive: prop52.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: grid;
    grid-template-columns: 20px 1fr;
    column-gap: 10px;
    padding: 6px 0;
    position: relative;
    --tone-fg: var(--muted-foreground);
  }

  /* Tone color mapping */
  :host([tone='default'])     { --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .gutter {
    position: relative;
    display: flex;
    justify-content: center;
  }
  .gutter::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--border);
  }
  :host(:first-of-type) .gutter::before { top: 10px; }
  :host(:last-of-type) .gutter::before { bottom: calc(100% - 10px); }

  .dot {
    position: relative;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    background: var(--tone-fg);
    border: 2px solid var(--background);
    margin-top: 4px;
    box-shadow: var(--shadow-xs);
    flex-shrink: 0;
  }

  .body { min-width: 0; }

  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: var(--font-weight-medium);
    color: var(--foreground);
  }

  .at {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    margin-left: auto;
  }

  .content {
    font-size: var(--text-sm);
    color: var(--muted-foreground);
    line-height: var(--leading-normal);
    margin-top: 2px;
  }

  :host([interactive]) {
    cursor: pointer;
    border-radius: var(--radius-sm);
    outline: none;
  }
  :host([interactive]:hover) { background: var(--muted); }
  :host([interactive]:focus-visible) {
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  @media (forced-colors: active) {
    :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.at = props.at;
    $scope.interactive = props.interactive;
    let __r = null;
    if (sui_timeline_item_controller_default && typeof sui_timeline_item_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_timeline_item_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf13({
      anchor,
      test: () => {
        if ($scope.at())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("span");
          __e8.setAttribute("class", "at");
          __e8.setAttribute("part", "at");
          const __t9 = document.createTextNode("");
          __e8.appendChild(__t9);
          __c7.push(bindText13(__t9, () => $scope.at()));
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets53(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "gutter");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "dot");
    __e1.setAttribute("part", "dot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "body");
    __e2.setAttribute("part", "body");
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "head");
    const __e4 = document.createElement("slot");
    __e4.setAttribute("name", "title");
    __e3.appendChild(__e4);
    const __ifAnchor5 = document.createComment("if");
    __e3.appendChild(__ifAnchor5);
    __cleanups.push(__wireBoundary_b0(__ifAnchor5));
    __e2.appendChild(__e3);
    const __e10 = document.createElement("div");
    __e10.setAttribute("class", "content");
    const __e11 = document.createElement("slot");
    __e10.appendChild(__e11);
    __e2.appendChild(__e10);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-loading/sui-loading.sui
import { adoptSuiSheets as adoptSuiSheets54, attachRef as attachRef29, component as component54, prop as prop53 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-loading/sui-loading.controller.ts
import { controller as controller37 } from "@ship/sui-runtime";
var DEFAULT_MESSAGES = ["Waking daemon", "Warming routes", "Fetching state", "Hydrating shell"];
function parseMessages(raw) {
  const trimmed = raw.trim();
  if (!trimmed)
    return DEFAULT_MESSAGES;
  return trimmed.split("|").map((s) => s.trim()).filter(Boolean);
}
var sui_loading_controller_default = controller37({
  mount(state, _host) {
    const scope = state;
    const msgEl = scope.msgEl();
    if (!msgEl)
      return;
    let index = 0;
    const showMsg = () => {
      const msgs = parseMessages(scope.messages());
      msgEl.textContent = msgs[index] ?? "";
      msgEl.style.opacity = "1";
    };
    showMsg();
    const timer = setInterval(() => {
      msgEl.style.opacity = "0";
      requestAnimationFrame(() => {
        const msgs = parseMessages(scope.messages());
        index = (index + 1) % msgs.length;
        msgEl.textContent = msgs[index] ?? "";
        msgEl.style.opacity = "1";
      });
    }, 1400);
    return () => clearInterval(timer);
  }
});

// packages/ui-components/src/sui-loading/sui-loading.sui
var sui_loading_default = component54("sui-loading", {
  props: {
    messages: prop53.string({ default: "" }),
    compact: prop53.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3, 12px);
    min-height: 240px;
    width: 100%;
    color: var(--muted-foreground);
  }
  :host([compact]) {
    min-height: 80px;
    gap: 6px;
  }

  /* Stage: fixed footprint so bob/flame animations don't shift the message. */
  .stage {
    position: relative;
    width: 56px;
    height: 64px;
    display: grid;
    place-items: center;
  }
  :host([compact]) .stage {
    width: 28px;
    height: 32px;
  }

  .rocket {
    position: relative;
    z-index: 2;
    width: 32px;
    height: 32px;
    color: var(--primary);
    filter: drop-shadow(0 4px 6px color-mix(in oklch, var(--primary) 35%, transparent));
    animation: sui-loading-rocket 2.2s ease-in-out infinite;
    transform-origin: 50% 100%;
  }
  :host([compact]) .rocket {
    width: 18px;
    height: 18px;
    filter: none;
  }
  .rocket svg {
    width: 100%;
    height: 100%;
  }

  .flame {
    position: absolute;
    left: 50%;
    bottom: 6px;
    transform: translateX(-50%);
    width: 14px;
    height: 18px;
    z-index: 1;
    background: radial-gradient(
      ellipse at 50% 30%,
      color-mix(in oklch, var(--primary) 95%, white) 0%,
      color-mix(in oklch, var(--primary) 70%, transparent) 55%,
      transparent 80%
    );
    border-radius: 50% 50% 35% 35% / 60% 60% 35% 35%;
    filter: blur(0.5px);
    transform-origin: 50% 0%;
    animation:
      sui-loading-flame 0.18s ease-in-out infinite alternate,
      sui-loading-flame-pulse 2.2s ease-in-out infinite;
  }
  :host([compact]) .flame {
    width: 8px;
    height: 10px;
    bottom: 4px;
  }

  .particle {
    position: absolute;
    left: 50%;
    top: 75%;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: color-mix(in oklch, var(--primary) 85%, transparent);
    opacity: 0;
    z-index: 0;
  }
  .p1 { animation: sui-loading-particle 1.4s ease-out 0s    infinite; }
  .p2 { animation: sui-loading-particle 1.7s ease-out 0.4s  infinite; width: 2px; height: 2px; transform: translate(-50%, 0); }
  .p3 { animation: sui-loading-particle 1.5s ease-out 0.8s  infinite; width: 4px; height: 4px; }
  :host([compact]) .particle { display: none; }

  .message {
    font-size: var(--text-sm);
    color: var(--muted-foreground);
    transition: opacity var(--duration-medium, 220ms) var(--ease-out, ease);
    letter-spacing: 0.01em;
  }
  :host([compact]) .message { font-size: var(--text-xs); }

  @keyframes sui-loading-rocket {
    0%   { transform: translateY(0)    rotate(0deg);    }
    40%  { transform: translateY(-6px) rotate(-1.5deg); }
    60%  { transform: translateY(-7px) rotate(1.5deg);  }
    100% { transform: translateY(0)    rotate(0deg);    }
  }
  @keyframes sui-loading-flame {
    0%   { transform: translateX(-50%) scaleY(1)    scaleX(1);    opacity: 0.9; }
    100% { transform: translateX(-50%) scaleY(1.15) scaleX(0.92); opacity: 1;   }
  }
  @keyframes sui-loading-flame-pulse {
    0%, 100% { height: 16px; }
    50%      { height: 22px; }
  }
  @keyframes sui-loading-particle {
    0%   { transform: translate(-50%, 0)    scale(1);   opacity: 0.7; }
    80%  { transform: translate(-50%, 22px) scale(0.5); opacity: 0;   }
    100% { transform: translate(-50%, 26px) scale(0.3); opacity: 0;   }
  }

  @media (prefers-reduced-motion: reduce) {
    .rocket, .flame, .particle { animation: none; }
    .flame { opacity: 0.7; }
    .particle { display: none; }
    .message { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.messages = props.messages;
    $scope.compact = props.compact;
    let __r = null;
    if (sui_loading_controller_default && typeof sui_loading_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_loading_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets54(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "stage");
    __e0.setAttribute("part", "stage");
    __e0.setAttribute("aria-hidden", "true");
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "rocket");
    const __e2 = document.createElement("svg");
    __e2.setAttribute("viewbox", "0 0 32 40");
    __e2.setAttribute("fill", "none");
    __e2.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    const __e3 = document.createElement("path");
    __e3.setAttribute("d", "M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z");
    __e3.setAttribute("fill", "currentColor");
    __e2.appendChild(__e3);
    const __e4 = document.createElement("rect");
    __e4.setAttribute("x", "10");
    __e4.setAttribute("y", "16");
    __e4.setAttribute("width", "12");
    __e4.setAttribute("height", "14");
    __e4.setAttribute("rx", "2");
    __e4.setAttribute("fill", "currentColor");
    __e4.setAttribute("opacity", "0.9");
    __e2.appendChild(__e4);
    const __e5 = document.createElement("circle");
    __e5.setAttribute("cx", "16");
    __e5.setAttribute("cy", "20");
    __e5.setAttribute("r", "2.5");
    __e5.setAttribute("fill", "var(--background, #fff)");
    __e5.setAttribute("opacity", "0.85");
    __e2.appendChild(__e5);
    const __e6 = document.createElement("path");
    __e6.setAttribute("d", "M10 24 L4 32 L10 30 Z");
    __e6.setAttribute("fill", "currentColor");
    __e6.setAttribute("opacity", "0.75");
    __e2.appendChild(__e6);
    const __e7 = document.createElement("path");
    __e7.setAttribute("d", "M22 24 L28 32 L22 30 Z");
    __e7.setAttribute("fill", "currentColor");
    __e7.setAttribute("opacity", "0.75");
    __e2.appendChild(__e7);
    const __e8 = document.createElement("rect");
    __e8.setAttribute("x", "13");
    __e8.setAttribute("y", "29");
    __e8.setAttribute("width", "6");
    __e8.setAttribute("height", "2");
    __e8.setAttribute("rx", "0.5");
    __e8.setAttribute("fill", "var(--background, #000)");
    __e8.setAttribute("opacity", "0.5");
    __e2.appendChild(__e8);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "flame");
    __e0.appendChild(__e9);
    const __e10 = document.createElement("div");
    __e10.setAttribute("class", "particle p1");
    __e0.appendChild(__e10);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "particle p2");
    __e0.appendChild(__e11);
    const __e12 = document.createElement("div");
    __e12.setAttribute("class", "particle p3");
    __e0.appendChild(__e12);
    __root.appendChild(__e0);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "message");
    __e13.setAttribute("part", "message");
    attachRef29($scope, "msgEl", __e13);
    __root.appendChild(__e13);
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

// packages/ui-components/src/sui-edit-overlay/sui-edit-overlay.sui
import { adoptSuiSheets as adoptSuiSheets55, attachRef as attachRef30, component as component55 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-edit-overlay/sui-edit-overlay.controller.ts
import { controller as controller38 } from "@ship/sui-runtime";
function targetFromEvent(e) {
  const root = document.getElementById("ship-view-root");
  if (!root)
    return null;
  const target = e.target;
  if (!target || !root.contains(target))
    return null;
  if (target.closest("sui-edit-overlay"))
    return null;
  return target;
}
function describe(el) {
  const tag = el.tagName.toLowerCase();
  const id = el.id || null;
  let selector = tag;
  if (id)
    selector = `#${id}`;
  else if (el.classList.length > 0) {
    selector = `${tag}.${Array.from(el.classList).slice(0, 2).join(".")}`;
  }
  return { tag, selector, id };
}
var sui_edit_overlay_controller_default = controller38({
  mount(state, host) {
    const scope = state;
    const boxEl = scope.boxEl();
    const labelEl = scope.labelEl();
    let _raf = null;
    function applyHighlight(hl) {
      if (!boxEl || !labelEl)
        return;
      if (!hl) {
        boxEl.classList.remove("visible");
        labelEl.classList.remove("visible");
        return;
      }
      boxEl.style.cssText = `top:${hl.top}px;left:${hl.left}px;width:${hl.width}px;height:${hl.height}px`;
      boxEl.classList.add("visible");
      labelEl.style.cssText = `top:${hl.top}px;left:${hl.left}px`;
      labelEl.textContent = `<${hl.tag}${hl.id ? `#${hl.id}` : ""}>`;
      labelEl.classList.add("visible");
    }
    const onPointerMove = (e) => {
      if (_raf)
        return;
      _raf = requestAnimationFrame(() => {
        _raf = null;
        const target = targetFromEvent(e);
        if (!target) {
          applyHighlight(null);
          return;
        }
        const rect = target.getBoundingClientRect();
        const { tag, id } = describe(target);
        applyHighlight({ top: rect.top, left: rect.left, width: rect.width, height: rect.height, tag, id });
      });
    };
    const onPointerLeave = () => {
      applyHighlight(null);
    };
    const onClick = (e) => {
      const target = targetFromEvent(e);
      if (!target)
        return;
      e.preventDefault();
      e.stopPropagation();
      const { tag, selector, id } = describe(target);
      const shellEl = host.closest("sui-shell") ?? host.closest("ship-shell");
      shellEl?.dispatchEvent(new CustomEvent("sui:chat:annotate", {
        detail: {
          tag,
          selector,
          id,
          preview: (target.textContent ?? "").trim().slice(0, 60)
        },
        bubbles: true
      }));
      window.ship?.chat?.open?.();
    };
    function setupListeners() {
      const root = document.getElementById("ship-view-root");
      if (!root)
        return;
      root.addEventListener("pointermove", onPointerMove, true);
      root.addEventListener("pointerleave", onPointerLeave, true);
      root.addEventListener("click", onClick, true);
    }
    function teardownListeners() {
      const root = document.getElementById("ship-view-root");
      if (!root)
        return;
      root.removeEventListener("pointermove", onPointerMove, true);
      root.removeEventListener("pointerleave", onPointerLeave, true);
      root.removeEventListener("click", onClick, true);
    }
    function applyActive() {
      const on2 = document.body.classList.contains("edit-mode");
      host.classList.toggle("active", on2);
      if (on2)
        setupListeners();
      else {
        teardownListeners();
        applyHighlight(null);
        if (_raf) {
          cancelAnimationFrame(_raf);
          _raf = null;
        }
      }
    }
    applyActive();
    const observer = new MutationObserver(() => applyActive());
    observer.observe(document.body, { attributes: true, attributeFilter: ["class"] });
    return () => {
      observer.disconnect();
      teardownListeners();
      if (_raf) {
        cancelAnimationFrame(_raf);
        _raf = null;
      }
    };
  }
});

// packages/ui-components/src/sui-edit-overlay/sui-edit-overlay.sui
var sui_edit_overlay_default = component55("sui-edit-overlay", {
  shadow: true,
  styles: `
  :host {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 9500;
    display: none;
  }
  :host(.active) {
    display: block;
  }

  .box {
    position: absolute;
    border: 2px solid var(--primary);
    background: color-mix(in oklch, var(--primary) 8%, transparent);
    border-radius: 2px;
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--primary) 40%, transparent);
    transition: all 60ms linear;
    display: none;
  }
  .box.visible { display: block; }

  .label {
    position: absolute;
    padding: 2px 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--primary-foreground, #fff);
    background: var(--primary);
    border-radius: 2px;
    white-space: nowrap;
    transform: translateY(-100%);
    display: none;
  }
  .label.visible { display: block; }

  .hint {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    padding: 6px 12px;
    background: var(--card, var(--background));
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  }
  .hint kbd {
    font-family: var(--font-mono);
    background: var(--muted);
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid var(--border);
    margin: 0 2px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_edit_overlay_controller_default && typeof sui_edit_overlay_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_edit_overlay_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets55(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    attachRef30($scope, "boxEl", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "label");
    __e1.setAttribute("part", "label");
    attachRef30($scope, "labelEl", __e1);
    __root.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "hint");
    __e2.setAttribute("part", "hint");
    const __t3 = document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);
    __e2.appendChild(__t3);
    const __e4 = document.createElement("kbd");
    const __t5 = document.createTextNode("⌘⇧E");
    __e4.appendChild(__t5);
    __e2.appendChild(__e4);
    const __t6 = document.createTextNode(` to exit.
`);
    __e2.appendChild(__t6);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-annotation/sui-annotation-layer.sui
import { adoptSuiSheets as adoptSuiSheets56, attachRef as attachRef31, bindAttr as bindAttr28, bindFor as bindFor3, bindIf as bindIf14, bindProp, bindValue as bindValue3, component as component56, on as on2, prop as prop54 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-annotation/sui-annotation-layer.controller.ts
import { controller as controller39, signal as signal9, derived as derived3, ref as ref4 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-annotation/sui-annotation.internals.ts
var CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
function newUlid(now = Date.now()) {
  let ts = now;
  const time = new Array(10);
  for (let i = 9;i >= 0; i--) {
    time[i] = CROCKFORD[ts % 32];
    ts = Math.floor(ts / 32);
  }
  const rand = new Array(16);
  const cryptoObj = typeof globalThis !== "undefined" ? globalThis.crypto : undefined;
  if (cryptoObj?.getRandomValues) {
    const buf = new Uint8Array(16);
    cryptoObj.getRandomValues(buf);
    for (let i = 0;i < 16; i++)
      rand[i] = CROCKFORD[buf[i] % 32];
  } else {
    for (let i = 0;i < 16; i++)
      rand[i] = CROCKFORD[Math.floor(Math.random() * 32)];
  }
  return time.join("") + rand.join("");
}
function buildAnnotationEvent(input) {
  const target = {};
  if (input.viewId)
    target.view_id = input.viewId;
  if (input.selector)
    target.selector = input.selector;
  if (input.rect)
    target.rect = input.rect;
  return {
    id: newUlid(),
    kind: input.kind,
    target,
    note: input.note,
    created_at: new Date().toISOString(),
    author_kind: "human"
  };
}

// packages/ui-components/src/sui-annotation/sui-annotation-layer.controller.ts
var DEFAULT_ENDPOINT = "/api/feedback";
var sui_annotation_layer_controller_default = controller39({
  scope: (props, host) => {
    const viewIdProp = props.viewId;
    const modeProp = props.mode;
    const endpointProp = props.endpoint;
    const stagingProp = props.staging;
    const staged = signal9([]);
    const pending2 = signal9(null);
    const drag = signal9(null);
    const noteText = signal9("");
    const _positions = {};
    const dragVisible = derived3(() => {
      const d = drag();
      if (!d)
        return false;
      return Math.abs(d.curX - d.startX) > 4 || Math.abs(d.curY - d.startY) > 4;
    });
    const dragRectStyle = derived3(() => {
      const d = drag();
      if (!d)
        return "";
      const x = Math.min(d.startX, d.curX);
      const y = Math.min(d.startY, d.curY);
      const w = Math.abs(d.curX - d.startX);
      const h = Math.abs(d.curY - d.startY);
      return `left:${x}px;top:${y}px;width:${w}px;height:${h}px`;
    });
    const popoverStyle = derived3(() => {
      const p = pending2();
      if (!p)
        return "";
      const left = Math.min(p.x + 12, 1e4);
      const top = Math.min(p.y + 12, 1e4);
      return `left:${left}px;top:${top}px`;
    });
    const markers = derived3(() => staged().map((a, i) => ({
      id: a.id,
      kind: a.kind,
      index: i,
      rect: a.kind === "region" && a.target.rect ? a.target.rect.join(",") : "",
      pos: _positions[a.id] ?? { x: 0, y: 0 }
    })));
    async function sendOne(ev) {
      const ep = endpointProp() || DEFAULT_ENDPOINT;
      try {
        const resp = await fetch(ep, {
          method: "POST",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(ev)
        });
        if (!resp.ok) {
          host.dispatchEvent(new CustomEvent("sui-annotation-error", {
            detail: { code: `http_${resp.status}`, id: ev.id },
            bubbles: true,
            composed: true
          }));
          return;
        }
        staged.set(staged().filter((s) => s.id !== ev.id));
        host.dispatchEvent(new CustomEvent("sui-annotation-sent", {
          detail: { id: ev.id, status: resp.status },
          bubbles: true,
          composed: true
        }));
      } catch (e) {
        host.dispatchEvent(new CustomEvent("sui-annotation-error", {
          detail: { code: "network_error", message: String(e), id: ev.id },
          bubbles: true,
          composed: true
        }));
      }
    }
    function confirmNote(note) {
      const p = pending2();
      if (!p)
        return;
      const trimmed = note.trim();
      if (!trimmed) {
        pending2.set(null);
        noteText.set("");
        return;
      }
      const ev = buildAnnotationEvent({
        viewId: viewIdProp() || undefined,
        kind: p.kind,
        selector: p.kind === "point" ? p.selector : undefined,
        rect: p.kind === "region" ? p.rect : undefined,
        note: trimmed
      });
      _positions[ev.id] = { x: p.x, y: p.y };
      staged.set([...staged(), ev]);
      host.dispatchEvent(new CustomEvent("sui-annotation-staged", {
        detail: ev,
        bubbles: true,
        composed: true
      }));
      pending2.set(null);
      noteText.set("");
      if (!stagingProp())
        sendOne(ev);
    }
    const sendNote = () => {
      confirmNote(noteText());
    };
    const cancelPending = () => {
      pending2.set(null);
      noteText.set("");
    };
    const onNoteKeydown = (e) => {
      if (e.key === "Escape") {
        cancelPending();
      } else if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
        const ta = e.currentTarget;
        confirmNote(ta.value);
      }
    };
    const overlayEl = ref4(null);
    return {
      staged,
      pending: pending2,
      drag,
      noteText,
      dragVisible,
      dragRectStyle,
      popoverStyle,
      markers,
      overlayEl,
      confirmNote,
      sendNote,
      cancelPending,
      onNoteKeydown,
      sendOne
    };
  },
  mount(state, host) {
    const scope = state;
    let _drag = null;
    let _raf = null;
    const overlayPos = (e) => {
      const el = scope.overlayEl();
      if (!el)
        return { x: 0, y: 0 };
      const r = el.getBoundingClientRect();
      return { x: e.clientX - r.left, y: e.clientY - r.top };
    };
    const onPointerDown = (e) => {
      const m = scope.mode();
      if (m === "off")
        return;
      const t = e.target;
      if (t.closest("[data-annotation-popover]") || t.closest("sui-annotation-marker"))
        return;
      if (scope.pending()) {
        scope.pending.set(null);
        scope.noteText.set("");
        return;
      }
      e.preventDefault();
      const p = overlayPos(e);
      _drag = { startX: p.x, startY: p.y, curX: p.x, curY: p.y };
      scope.drag.set(_drag);
      e.target.setPointerCapture?.(e.pointerId);
    };
    const onPointerMove = (e) => {
      if (!_drag)
        return;
      const p = overlayPos(e);
      _drag = { ..._drag, curX: p.x, curY: p.y };
      if (_raf !== null)
        return;
      _raf = requestAnimationFrame(() => {
        _raf = null;
        if (_drag)
          scope.drag.set({ ..._drag });
      });
    };
    const onPointerUp = (e) => {
      if (!_drag)
        return;
      const p = overlayPos(e);
      const dx = Math.abs(p.x - _drag.startX);
      const dy = Math.abs(p.y - _drag.startY);
      const m = scope.mode();
      let next = null;
      if (m === "point" || m === "region" && dx < 8 && dy < 8) {
        next = { kind: "point", x: p.x, y: p.y, selector: "body" };
      } else if (m === "region") {
        const x = Math.min(_drag.startX, p.x);
        const y = Math.min(_drag.startY, p.y);
        const rect = [
          Math.round(x),
          Math.round(y),
          Math.round(dx),
          Math.round(dy)
        ];
        next = { kind: "region", x: x + dx / 2, y: y + dy / 2, rect };
      }
      _drag = null;
      scope.drag.set(null);
      scope.noteText.set("");
      if (next)
        scope.pending.set(next);
    };
    const overlayEl = scope.overlayEl();
    if (overlayEl) {
      overlayEl.addEventListener("pointerdown", onPointerDown);
      overlayEl.addEventListener("pointermove", onPointerMove);
      overlayEl.addEventListener("pointerup", onPointerUp);
    }
    Object.defineProperty(host, "staged", {
      configurable: true,
      get: () => scope.staged()
    });
    host.send = async () => {
      for (const ev of [...scope.staged()])
        await scope.sendOne(ev);
    };
    host.clear = () => {
      scope.staged.set([]);
    };
    return () => {
      if (_raf !== null) {
        cancelAnimationFrame(_raf);
        _raf = null;
      }
      if (overlayEl) {
        overlayEl.removeEventListener("pointerdown", onPointerDown);
        overlayEl.removeEventListener("pointermove", onPointerMove);
        overlayEl.removeEventListener("pointerup", onPointerUp);
      }
    };
  }
});

// packages/ui-components/src/sui-annotation/sui-annotation-layer.sui
var sui_annotation_layer_default = component56("sui-annotation-layer", {
  props: {
    "view-id": prop54.string({ default: "" }),
    mode: prop54.union(["off", "point", "region"], { default: "off", reflect: true }),
    endpoint: prop54.string({ default: "/api/feedback" }),
    staging: prop54.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: block;
    font-family: var(--font-body);
  }

  .surface {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    z-index: 10;
    pointer-events: none;
  }
  :host([mode='point']) .overlay,
  :host([mode='region']) .overlay {
    pointer-events: auto;
    cursor: crosshair;
  }

  /* Drag selection rectangle */
  .drag-rect {
    position: absolute;
    border: 2px dashed var(--primary);
    background: color-mix(in oklch, var(--primary) 8%, transparent);
    pointer-events: none;
    border-radius: var(--radius-sm);
  }

  /* Inline note popover */
  .popover {
    position: absolute;
    z-index: 20;
    min-width: 220px;
    padding: 8px;
    background: var(--popover, var(--background));
    color: var(--popover-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    gap: 6px;
    pointer-events: auto;
  }
  .popover textarea {
    font: inherit;
    width: 100%;
    min-height: 60px;
    resize: vertical;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
    color: var(--foreground);
    box-sizing: border-box;
  }
  .popover-actions { display: flex; justify-content: flex-end; gap: 6px; }
  .popover button {
    font: inherit;
    font-size: var(--text-sm);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    cursor: pointer;
  }
  .popover button[data-primary] {
    background: var(--primary);
    color: var(--primary-foreground);
    border-color: var(--primary);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.viewId = props.viewId;
    $scope.mode = props.mode;
    $scope.endpoint = props.endpoint;
    $scope.staging = props.staging;
    let __r = null;
    if (sui_annotation_layer_controller_default && typeof sui_annotation_layer_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_annotation_layer_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindIf14({
      anchor,
      test: () => {
        if ($scope.dragVisible())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElement("div");
          __e6.setAttribute("class", "drag-rect");
          __c5.push(bindAttr28(__e6, "style", () => $scope.dragRectStyle()));
          __frag4.appendChild(__e6);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b3 = (anchor) => bindFor3({
      anchor,
      items: () => $scope.markers(),
      key: (m, __i) => {
        const __child = Object.create($scope);
        __child.m = m;
        __child.__i = __i;
        return (($scope2) => m.id)(__child);
      },
      render: (m, __i) => {
        const __childScope8 = Object.create($scope);
        __childScope8.m = m;
        __childScope8.__i = __i;
        return (($scope2) => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("sui-annotation-marker");
          __c10.push(bindProp(__e11, "kind", () => $scope2.m.kind));
          __c10.push(bindProp(__e11, "index", () => $scope2.m.index));
          __c10.push(bindProp(__e11, "rect", () => $scope2.m.rect));
          __c10.push(bindProp(__e11, "x", () => $scope2.m.pos.x));
          __c10.push(bindProp(__e11, "y", () => $scope2.m.pos.y));
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope8);
      }
    });
    const __wireBoundary_b9 = (anchor) => bindIf14({
      anchor,
      test: () => {
        if ($scope.pending())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("div");
          __e15.setAttribute("class", "popover");
          __e15.setAttribute("data-annotation-popover", "");
          __c14.push(bindAttr28(__e15, "style", () => $scope.popoverStyle()));
          __c14.push(on2(__e15, "pointerdown", function() {
            const __h = (e) => e.stopPropagation();
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e16 = document.createElement("textarea");
          __e16.setAttribute("part", "note-input");
          __e16.setAttribute("placeholder", "Add a note…");
          __c14.push(bindValue3(__e16, $scope.noteText));
          __c14.push(on2(__e16, "keydown", function() {
            const __h = $scope.onNoteKeydown;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          __e15.appendChild(__e16);
          const __e17 = document.createElement("div");
          __e17.setAttribute("class", "popover-actions");
          const __e18 = document.createElement("button");
          __e18.setAttribute("type", "button");
          __c14.push(on2(__e18, "click", function() {
            const __h = $scope.cancelPending;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t19 = document.createTextNode("Cancel");
          __e18.appendChild(__t19);
          __e17.appendChild(__e18);
          const __e20 = document.createElement("button");
          __e20.setAttribute("type", "button");
          __e20.setAttribute("data-primary", "");
          __c14.push(on2(__e20, "click", function() {
            const __h = $scope.sendNote;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t21 = document.createTextNode("Send");
          __e20.appendChild(__t21);
          __e17.appendChild(__e20);
          __e15.appendChild(__e17);
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets56(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "surface");
    __e0.setAttribute("part", "surface");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "overlay");
    __e2.setAttribute("part", "overlay");
    attachRef31($scope, "overlayEl", __e2);
    const __ifAnchor3 = document.createComment("if");
    __e2.appendChild(__ifAnchor3);
    __cleanups.push(__wireBoundary_b1(__ifAnchor3));
    const __forAnchor7 = document.createComment("for");
    __e2.appendChild(__forAnchor7);
    __cleanups.push(__wireBoundary_b3(__forAnchor7));
    const __ifAnchor12 = document.createComment("if");
    __e2.appendChild(__ifAnchor12);
    __cleanups.push(__wireBoundary_b9(__ifAnchor12));
    __e0.appendChild(__e2);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-annotation/sui-annotation-marker.sui
import { adoptSuiSheets as adoptSuiSheets57, attachRef as attachRef32, component as component57, prop as prop55 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-annotation/sui-annotation-marker.controller.ts
import { controller as controller40, effect as effect36 } from "@ship/sui-runtime";
var sui_annotation_marker_controller_default = controller40({
  mount(state, _host) {
    const scope = state;
    const stop = effect36(() => {
      const pinEl = scope.pinEl();
      const boxEl = scope.boxEl();
      const x = scope.x();
      const y = scope.y();
      const idx = scope.index();
      const rect = scope.rect();
      if (pinEl) {
        pinEl.style.left = `${x}px`;
        pinEl.style.top = `${y}px`;
        pinEl.textContent = String(idx + 1);
      }
      if (boxEl && rect) {
        const parts = rect.split(",").map(Number);
        if (parts.length === 4 && parts.every((n) => Number.isFinite(n))) {
          const [rx, ry, rw, rh] = parts;
          boxEl.style.cssText = `left:${rx}px;top:${ry}px;width:${rw}px;height:${rh}px`;
        }
      }
    });
    return stop;
  }
});

// packages/ui-components/src/sui-annotation/sui-annotation-marker.sui
var sui_annotation_marker_default = component57("sui-annotation-marker", {
  props: {
    kind: prop55.union(["point", "region"], { default: "point", reflect: true }),
    x: prop55.number({ default: 0 }),
    y: prop55.number({ default: 0 }),
    rect: prop55.string({ default: "" }),
    active: prop55.bool({ default: false, reflect: true }),
    index: prop55.number({ default: 0 })
  },
  shadow: true,
  styles: `
  :host {
    position: absolute;
    pointer-events: auto;
    font-family: var(--font-body);
    font-size: var(--text-xs);
  }

  [part='pin'] {
    position: absolute;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--primary);
    color: var(--primary-foreground);
    display: none;
    align-items: center;
    justify-content: center;
    font-weight: var(--font-weight-medium);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    transform: translate(-50%, -50%);
    border: 2px solid var(--background);
    transition: filter var(--duration-fast) var(--ease-out);
  }
  :host([kind='point']) [part='pin'] { display: inline-flex; }
  :host([active]) [part='pin'] {
    filter: brightness(1.2);
    outline: 2px solid var(--ring-color);
    outline-offset: 2px;
  }

  [part='box'] {
    position: absolute;
    border: 2px dashed var(--primary);
    background: var(--tone-primary-bg);
    border-radius: var(--radius-sm);
    cursor: pointer;
    box-sizing: border-box;
    display: none;
  }
  :host([kind='region']) [part='box'] { display: block; }
  :host([active]) [part='box'] {
    background: color-mix(in oklch, var(--primary) 14%, transparent);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.kind = props.kind;
    $scope.x = props.x;
    $scope.y = props.y;
    $scope.rect = props.rect;
    $scope.active = props.active;
    $scope.index = props.index;
    let __r = null;
    if (sui_annotation_marker_controller_default && typeof sui_annotation_marker_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_annotation_marker_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets57(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("part", "pin");
    __e0.setAttribute("role", "button");
    __e0.setAttribute("tabindex", "0");
    attachRef32($scope, "pinEl", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("part", "box");
    __e1.setAttribute("role", "button");
    __e1.setAttribute("tabindex", "0");
    attachRef32($scope, "boxEl", __e1);
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

// packages/ui-components/src/sui-annotation/sui-annotation-list.sui
import { adoptSuiSheets as adoptSuiSheets58, bindAttr as bindAttr29, bindFor as bindFor4, bindIf as bindIf15, bindText as bindText14, component as component58, on as on3 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-annotation/sui-annotation-list.controller.ts
import { controller as controller41, signal as signal10 } from "@ship/sui-runtime";
var sui_annotation_list_controller_default = controller41({
  scope: (_, host) => {
    const staged = signal10([]);
    const emitRemove = (id) => {
      host.dispatchEvent(new CustomEvent("sui-annotation-remove", {
        detail: { id },
        bubbles: true,
        composed: true
      }));
    };
    return { staged, emitRemove };
  },
  mount(state, host) {
    const scope = state;
    const stagedSignal = scope.staged;
    const initial = host.staged;
    if (initial != null)
      stagedSignal.set(Array.isArray(initial) ? initial : []);
    Object.defineProperty(host, "staged", {
      configurable: true,
      get: () => stagedSignal(),
      set: (v) => stagedSignal.set(Array.isArray(v) ? v : [])
    });
  }
});

// packages/ui-components/src/sui-annotation/sui-annotation-list.sui
var sui_annotation_list_default = component58("sui-annotation-list", {
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
  }
  .kind {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--muted-foreground);
    min-width: 48px;
  }
  .note {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty {
    color: var(--muted-foreground);
    padding: 4px 8px;
  }
  button {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--muted-foreground);
    cursor: pointer;
    font-size: var(--text-xs);
    padding: 2px 6px;
  }
  button:hover {
    color: var(--destructive);
    border-color: var(--destructive);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_annotation_list_controller_default && typeof sui_annotation_list_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_annotation_list_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor4({
      anchor,
      items: () => $scope.staged(),
      key: (a, __i) => {
        const __child = Object.create($scope);
        __child.a = a;
        __child.__i = __i;
        return (($scope2) => a.id)(__child);
      },
      render: (a, __i) => {
        const __childScope9 = Object.create($scope);
        __childScope9.a = a;
        __childScope9.__i = __i;
        return (($scope2) => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("li");
          __e12.setAttribute("part", "item");
          __c11.push(bindAttr29(__e12, "data-id", () => $scope2.a.id));
          const __e13 = document.createElement("span");
          __e13.setAttribute("class", "kind");
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c11.push(bindText14(__t14, () => $scope2.a.kind));
          __e12.appendChild(__e13);
          const __e15 = document.createElement("span");
          __e15.setAttribute("class", "note");
          __c11.push(bindAttr29(__e15, "title", () => $scope2.a.note ?? ""));
          const __t16 = document.createTextNode("");
          __e15.appendChild(__t16);
          __c11.push(bindText14(__t16, () => $scope2.a.note ?? "(no note)"));
          __e12.appendChild(__e15);
          const __e17 = document.createElement("button");
          __e17.setAttribute("type", "button");
          __e17.setAttribute("aria-label", "Remove annotation");
          __c11.push(on3(__e17, "click", function() {
            const __h = () => $scope2.emitRemove($scope2.a.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t18 = document.createTextNode("Remove");
          __e17.appendChild(__t18);
          __e12.appendChild(__e17);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope9);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf15({
      anchor,
      test: () => {
        if ($scope.staged().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "empty");
          __e3.setAttribute("part", "empty");
          const __t4 = document.createTextNode("No staged annotations.");
          __e3.appendChild(__t4);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("ol");
          __e7.setAttribute("part", "list");
          const __forAnchor8 = document.createComment("for");
          __e7.appendChild(__forAnchor8);
          __c6.push(__wireBoundary_b1(__forAnchor8));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets58(__root);
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

// packages/ui-components/src/sui-radio-group/sui-radio.sui
import { adoptSuiSheets as adoptSuiSheets59, attachRef as attachRef33, bindAttr as bindAttr30, bindChecked as bindChecked3, component as component59, prop as prop56 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-radio-group/sui-radio.controller.ts
import { controller as controller42, effect as effect37 } from "@ship/sui-runtime";
var sui_radio_controller_default = controller42({
  mount(state, host) {
    const scope = state;
    const stop = effect37(() => {
      const input = scope.inputEl();
      if (!input)
        return;
      const onClick = () => {
        if (scope.disabled())
          return;
        host.dispatchEvent(new CustomEvent("sui-radio-select", {
          detail: { value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      const onKeydown = (e) => {
        const navKeys = ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"];
        if (!navKeys.includes(e.key))
          return;
        e.preventDefault();
        const forward = e.key === "ArrowDown" || e.key === "ArrowRight";
        host.dispatchEvent(new CustomEvent("sui-radio-nav", {
          detail: { direction: forward ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      };
      input.addEventListener("click", onClick);
      input.addEventListener("keydown", onKeydown);
      return () => {
        input.removeEventListener("click", onClick);
        input.removeEventListener("keydown", onKeydown);
      };
    });
    host.focusInput = () => {
      const input = host.shadowRoot?.querySelector("input");
      input?.focus();
    };
    return () => stop();
  }
});

// packages/ui-components/src/sui-radio-group/sui-radio.sui
var sui_radio_default = component59("sui-radio", {
  props: {
    value: prop56.string({ default: "", reflect: true }),
    disabled: prop56.bool({ default: false, reflect: true }),
    checked: prop56.bool({ default: false, reflect: true }),
    tone: prop56.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop56.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
  }

  :host(:not([tone])),
  :host([tone='default'])     { --tone: var(--primary); }
  :host([tone='primary'])     { --tone: var(--primary); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --rd-size: 16px; --rd-dot: 8px; }
  :host([size='sm']) { --rd-size: 14px; --rd-dot: 6px; }
  :host([size='lg']) { --rd-size: 20px; --rd-dot: 10px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .circle {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--rd-size);
    height: var(--rd-size);
    background: var(--background);
    border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
    border-radius: 50%;
    box-sizing: border-box;
    transition:
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked])) label:hover .circle {
    border-color: color-mix(in oklch, var(--border) 220%, transparent);
  }
  :host([checked]) .circle { border-color: var(--tone); }

  .dot {
    width: var(--rd-dot);
    height: var(--rd-dot);
    border-radius: 50%;
    background: var(--tone);
    opacity: 0;
    transform: scale(0);
    transition:
      opacity var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]) .dot { opacity: 1; transform: scale(1); }

  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    opacity: 0;
    cursor: inherit;
    appearance: none;
    -webkit-appearance: none;
  }

  :host(:not([disabled])) label:has(input:focus-visible) .circle {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .circle { border-color: CanvasText; }
    :host([checked]) .dot { background: Highlight; }
    :host(:not([disabled])) label:has(input:focus-visible) .circle {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }

  :host([disabled]) { color: var(--muted-foreground); }
  :host([disabled]) .circle {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    filter: saturate(0.6);
  }
  :host([disabled][checked]) .circle {
    border-color: color-mix(in oklch, var(--tone) 50%, var(--border));
  }
  :host([disabled][checked]) .dot {
    background: color-mix(in oklch, var(--tone) 55%, var(--muted));
  }


  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.disabled = props.disabled;
    $scope.checked = props.checked;
    $scope.tone = props.tone;
    $scope.size = props.size;
    let __r = null;
    if (sui_radio_controller_default && typeof sui_radio_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_radio_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets59(__root);
    const __e0 = document.createElement("label");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "circle");
    __e1.setAttribute("part", "circle");
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __e2.setAttribute("type", "radio");
    __cleanups.push(bindChecked3(__e2, $scope.checked));
    __cleanups.push(bindAttr30(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr30(__e2, "aria-checked", () => $scope.checked() ? "true" : "false"));
    __cleanups.push(bindAttr30(__e2, "tabindex", () => $scope.checked() ? "0" : "-1"));
    attachRef33($scope, "inputEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "dot");
    __e3.setAttribute("part", "dot");
    __e3.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e3);
    __e0.appendChild(__e1);
    const __e4 = document.createElement("slot");
    __e0.appendChild(__e4);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-radio-group/sui-radio-group.sui
import { adoptSuiSheets as adoptSuiSheets60, component as component60, prop as prop57 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-radio-group/sui-radio-group.controller.ts
import { controller as controller43, effect as effect38 } from "@ship/sui-runtime";
var sui_radio_group_controller_default = controller43({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "radiogroup");
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const getRadios = () => Array.from(host.querySelectorAll(":scope > sui-radio"));
    const enabledRadios = () => getRadios().filter((r) => !r.hasAttribute("disabled") && !scope.disabled());
    const syncRadios = () => {
      const currentValue = scope.value();
      const isGroupDisabled = scope.disabled();
      const groupTone = scope.tone();
      const groupSize = scope.size();
      for (const radio of getRadios()) {
        radio.toggleAttribute("checked", radio.getAttribute("value") === currentValue);
        if (isGroupDisabled)
          radio.setAttribute("disabled", "");
        if (groupTone && !radio.hasAttribute("tone"))
          radio.setAttribute("tone", groupTone);
        if (groupSize && !radio.hasAttribute("size"))
          radio.setAttribute("size", groupSize);
      }
      hiddenInput.name = scope.name();
      hiddenInput.value = currentValue;
    };
    const stopSync = effect38(() => {
      scope.value();
      scope.disabled();
      scope.tone();
      scope.size();
      syncRadios();
    });
    const stopLabel = effect38(() => {
      const label = scope.label();
      if (label)
        host.setAttribute("aria-label", label);
      else
        host.removeAttribute("aria-label");
    });
    const onSelect = (e) => {
      const ce = e;
      ce.stopPropagation();
      const newVal = ce.detail.value;
      if (newVal === scope.value())
        return;
      scope.value.set(newVal);
      syncRadios();
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: newVal },
        bubbles: true,
        composed: true
      }));
    };
    const onNav = (e) => {
      const ce = e;
      ce.stopPropagation();
      const enabled = enabledRadios();
      if (!enabled.length)
        return;
      const current = ce.composedPath().find((el) => el.tagName?.toLowerCase() === "sui-radio");
      const idx = current ? enabled.indexOf(current) : -1;
      const next = ce.detail.direction === "next" ? enabled[(idx + 1) % enabled.length] : enabled[(idx - 1 + enabled.length) % enabled.length];
      const nextVal = next.getAttribute("value") ?? "";
      scope.value.set(nextVal);
      syncRadios();
      next.focusInput?.();
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: nextVal },
        bubbles: true,
        composed: true
      }));
    };
    host.addEventListener("sui-radio-select", onSelect);
    host.addEventListener("sui-radio-nav", onNav);
    return () => {
      stopSync();
      stopLabel();
      host.removeEventListener("sui-radio-select", onSelect);
      host.removeEventListener("sui-radio-nav", onNav);
      hiddenInput.remove();
    };
  }
});

// packages/ui-components/src/sui-radio-group/sui-radio-group.sui
var sui_radio_group_default = component60("sui-radio-group", {
  props: {
    value: prop57.string({ default: "", reflect: true }),
    disabled: prop57.bool({ default: false, reflect: true }),
    tone: prop57.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop57.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop57.string({ default: "" }),
    label: prop57.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.label = props.label;
    let __r = null;
    if (sui_radio_group_controller_default && typeof sui_radio_group_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_radio_group_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets60(__root);
    const __e0 = document.createElement("slot");
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-slider/sui-slider.sui
import { adoptSuiSheets as adoptSuiSheets61, attachRef as attachRef34, bindAttr as bindAttr31, component as component61, prop as prop58 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-slider/sui-slider.controller.ts
import { controller as controller44, effect as effect39 } from "@ship/sui-runtime";
var sui_slider_controller_default = controller44({
  mount(state, host) {
    const scope = state;
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const clamp = (v) => {
      const step = scope.step();
      const min = scope.min();
      const max = scope.max();
      const stepped = Math.round((v - min) / step) * step + min;
      return Math.max(min, Math.min(max, stepped));
    };
    const setValue = (v) => {
      const clamped = clamp(v);
      if (clamped === scope.value())
        return;
      scope.value.set(clamped);
      hiddenInput.name = scope.name();
      hiddenInput.value = String(clamped);
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: clamped },
        bubbles: true,
        composed: true
      }));
    };
    const valueFromPointer = (clientX) => {
      const track = scope.trackEl();
      if (!track)
        return scope.value();
      const rect = track.getBoundingClientRect();
      const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
      return scope.min() + pct * (scope.max() - scope.min());
    };
    const stopPositionSync = effect39(() => {
      const val = scope.value();
      const min = scope.min();
      const max = scope.max();
      const range = max - min || 1;
      const pct = (val - min) / range * 100;
      const fill = scope.fillEl();
      const thumb = scope.thumbEl();
      if (fill)
        fill.style.width = `${pct}%`;
      if (thumb)
        thumb.style.left = `${pct}%`;
      hiddenInput.name = scope.name();
      hiddenInput.value = String(val);
    });
    let dragging = false;
    const onPointerDown = (e) => {
      if (scope.disabled())
        return;
      e.preventDefault();
      dragging = true;
      setValue(valueFromPointer(e.clientX));
      const thumb = scope.thumbEl();
      thumb?.focus();
      const onMove = (ev) => {
        if (dragging)
          setValue(valueFromPointer(ev.clientX));
      };
      const onUp = () => {
        dragging = false;
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
      };
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
    };
    const onKeydown = (e) => {
      if (scope.disabled())
        return;
      const step = scope.step();
      let handled = true;
      switch (e.key) {
        case "ArrowRight":
        case "ArrowUp":
          setValue(scope.value() + step);
          break;
        case "ArrowLeft":
        case "ArrowDown":
          setValue(scope.value() - step);
          break;
        case "Home":
          setValue(scope.min());
          break;
        case "End":
          setValue(scope.max());
          break;
        case "PageUp":
          setValue(scope.value() + step * 10);
          break;
        case "PageDown":
          setValue(scope.value() - step * 10);
          break;
        default:
          handled = false;
      }
      if (handled)
        e.preventDefault();
    };
    const stopPointer = effect39(() => {
      const container = scope.containerEl();
      if (!container)
        return;
      container.addEventListener("pointerdown", onPointerDown);
      return () => container.removeEventListener("pointerdown", onPointerDown);
    });
    const stopKeydown = effect39(() => {
      const thumb = scope.thumbEl();
      if (!thumb)
        return;
      thumb.addEventListener("keydown", onKeydown);
      return () => thumb.removeEventListener("keydown", onKeydown);
    });
    return () => {
      stopPositionSync();
      stopPointer();
      stopKeydown();
      hiddenInput.remove();
    };
  }
});

// packages/ui-components/src/sui-slider/sui-slider.sui
var sui_slider_default = component61("sui-slider", {
  props: {
    value: prop58.number({ default: 0, reflect: true }),
    min: prop58.number({ default: 0 }),
    max: prop58.number({ default: 100 }),
    step: prop58.number({ default: 1 }),
    disabled: prop58.bool({ default: false, reflect: true }),
    tone: prop58.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop58.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop58.string({ default: "" }),
    label: prop58.string({ default: "slider" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
  }

  :host(:not([tone])),
  :host([tone='default'])     { --tone: var(--primary); }
  :host([tone='primary'])     { --tone: var(--primary); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sl-h: 6px; --sl-thumb: 16px; }
  :host([size='sm']) { --sl-h: 4px; --sl-thumb: 12px; }
  :host([size='lg']) { --sl-h: 8px; --sl-thumb: 20px; }

  .container {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    height: var(--sl-thumb);
    cursor: pointer;
    touch-action: none;
  }
  :host([disabled]) .container { cursor: not-allowed; opacity: 0.5; }

  .track {
    position: relative;
    width: 100%;
    height: var(--sl-h);
    background: var(--muted);
    border-radius: var(--radius-full, 9999px);
    overflow: hidden;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--tone);
    border-radius: inherit;
  }

  .thumb {
    position: absolute;
    top: 50%;
    width: var(--sl-thumb);
    height: var(--sl-thumb);
    background: var(--background);
    border: 2px solid var(--tone);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translate(-50%, -50%);
    transition:
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    box-sizing: border-box;
    outline: none;
  }
  .thumb:hover { box-shadow: var(--shadow-md); transform: translate(-50%, -50%) scale(1.05); }
  .thumb:active { transform: translate(-50%, -50%) scale(0.95); }

  :host(:not([disabled])) .thumb:focus-visible {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .thumb { border-color: CanvasText; }
    .fill { background: Highlight; }
    .thumb:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.min = props.min;
    $scope.max = props.max;
    $scope.step = props.step;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.label = props.label;
    let __r = null;
    if (sui_slider_controller_default && typeof sui_slider_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_slider_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets61(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "container");
    attachRef34($scope, "containerEl", __e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "track");
    __e1.setAttribute("part", "track");
    attachRef34($scope, "trackEl", __e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "fill");
    __e2.setAttribute("part", "fill");
    attachRef34($scope, "fillEl", __e2);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "thumb");
    __e3.setAttribute("part", "thumb");
    __e3.setAttribute("role", "slider");
    __cleanups.push(bindAttr31(__e3, "tabindex", () => $scope.disabled() ? "-1" : "0"));
    __cleanups.push(bindAttr31(__e3, "aria-valuemin", () => $scope.min()));
    __cleanups.push(bindAttr31(__e3, "aria-valuemax", () => $scope.max()));
    __cleanups.push(bindAttr31(__e3, "aria-valuenow", () => $scope.value()));
    __cleanups.push(bindAttr31(__e3, "aria-label", () => $scope.label() || "slider"));
    __cleanups.push(bindAttr31(__e3, "aria-disabled", () => $scope.disabled() ? "true" : "false"));
    attachRef34($scope, "thumbEl", __e3);
    __e0.appendChild(__e3);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-segmented/sui-segmented-option.sui
import { adoptSuiSheets as adoptSuiSheets62, component as component62, prop as prop59 } from "@ship/sui-runtime";
var sui_segmented_option_default = component62("sui-segmented-option", {
  props: {
    value: prop59.string({ default: "", reflect: true }),
    icon: prop59.string({ default: "", reflect: true }),
    disabled: prop59.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.icon = props.icon;
    $scope.disabled = props.disabled;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets62(__root);
    const __t0 = document.createTextNode(`

`);
    __root.appendChild(__t0);
    const __t1 = document.createTextNode(`
`);
    __root.appendChild(__t1);
    const __t2 = document.createTextNode(`
`);
    __root.appendChild(__t2);
    const __t3 = document.createTextNode(`

`);
    __root.appendChild(__t3);
    const __t4 = document.createTextNode(`
`);
    __root.appendChild(__t4);
    return $scope;
  }
});

// packages/ui-components/src/sui-segmented/sui-segmented.sui
import { adoptSuiSheets as adoptSuiSheets63, attachRef as attachRef35, bindAttr as bindAttr32, bindFor as bindFor5, bindIf as bindIf16, bindProp as bindProp2, bindText as bindText15, component as component63, prop as prop60 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-segmented/sui-segmented.controller.ts
import { controller as controller45, signal as signal11, effect as effect40, ref as ref5 } from "@ship/sui-runtime";
var cssEscape = (s) => typeof CSS !== "undefined" && CSS.escape ? CSS.escape(s) : s.replace(/[^\w-]/g, "\\$&");
var sui_segmented_controller_default = controller45({
  scope: () => {
    const options = signal11([]);
    const containerEl = ref5(null);
    return { options, containerEl };
  },
  mount(state, host) {
    const scope = state;
    const optionsSig = scope.options;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "radiogroup");
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const readOptions = () => {
      const els = Array.from(host.querySelectorAll(":scope > sui-segmented-option"));
      return els.map((el) => ({
        value: el.getAttribute("value") ?? "",
        label: el.textContent?.trim() ?? "",
        icon: el.getAttribute("icon") || null,
        disabled: el.hasAttribute("disabled")
      }));
    };
    const select = (val) => {
      if (scope.disabled())
        return;
      const opts = optionsSig();
      const opt = opts.find((o) => o.value === val);
      if (!opt || opt.disabled)
        return;
      if (scope.value() === val)
        return;
      scope.value.set(val);
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: val },
        bubbles: true,
        composed: true
      }));
    };
    const onClick = (e) => {
      const btn = e.target?.closest(".btn");
      if (!btn)
        return;
      const val = btn.getAttribute("data-value");
      if (val !== null)
        select(val);
    };
    const onKeyDown = (e) => {
      const opts = optionsSig();
      const enabled = opts.filter((o) => !o.disabled && !scope.disabled());
      if (!enabled.length)
        return;
      const values = enabled.map((o) => o.value);
      const cur = scope.value();
      const i = Math.max(0, values.indexOf(cur));
      let next = i;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        e.preventDefault();
        next = (i + 1) % values.length;
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        e.preventDefault();
        next = (i - 1 + values.length) % values.length;
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const focused = host.shadowRoot.activeElement;
        const focusedValue = focused?.getAttribute("data-value");
        if (focusedValue)
          select(focusedValue);
        return;
      } else {
        return;
      }
      const targetValue = values[next] ?? "";
      select(targetValue);
      queueMicrotask(() => {
        const btn = host.shadowRoot?.querySelector(`.btn[data-value="${cssEscape(targetValue)}"]`);
        btn?.focus();
      });
    };
    optionsSig.set(readOptions());
    const observer = new MutationObserver(() => optionsSig.set(readOptions()));
    observer.observe(host, { childList: true, subtree: false });
    const stopHidden = effect40(() => {
      hiddenInput.name = scope.name();
      hiddenInput.value = scope.value();
    });
    const stopContainer = effect40(() => {
      const container = scope.containerEl();
      if (!container)
        return;
      container.addEventListener("click", onClick);
      container.addEventListener("keydown", onKeyDown);
      return () => {
        container.removeEventListener("click", onClick);
        container.removeEventListener("keydown", onKeyDown);
      };
    });
    return () => {
      observer.disconnect();
      stopHidden();
      stopContainer();
      hiddenInput.remove();
    };
  }
});

// packages/ui-components/src/sui-segmented/sui-segmented.sui
var sui_segmented_default = component63("sui-segmented", {
  props: {
    value: prop60.string({ default: "", reflect: true }),
    size: prop60.union(["sm", "md"], { default: "md", reflect: true }),
    disabled: prop60.bool({ default: false, reflect: true }),
    name: prop60.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
  }

  .container {
    display: inline-flex;
    padding: 2px;
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    gap: 1px;
  }

  .btn {
    appearance: none;
    border: 0;
    background: transparent;
    border-radius: var(--radius-sm, 4px);
    font-family: var(--font-body);
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-1, 4px);
    transition:
      background var(--duration-fast, 150ms) var(--ease-out, ease-out),
      color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    white-space: nowrap;
    line-height: 1;
  }

  :host(:not([size])) .btn,
  :host([size='md']) .btn {
    padding: 5px var(--space-3, 12px);
    font-size: var(--text-sm, 13px);
  }
  :host([size='sm']) .btn {
    padding: 4px var(--space-2, 8px);
    font-size: var(--text-xs, 11px);
  }

  .btn[aria-checked='true'] {
    background: var(--card);
    color: var(--foreground);
    box-shadow: var(--shadow-xs);
  }

  .btn:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--background),
      0 0 0 4px color-mix(in oklch, var(--primary) 60%, transparent);
  }

  .btn[disabled],
  :host([disabled]) .btn {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    width: 14px;
    height: 14px;
  }
  :host(:not([size='sm'])) .btn-icon { width: 16px; height: 16px; }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.size = props.size;
    $scope.disabled = props.disabled;
    $scope.name = props.name;
    let __r = null;
    if (sui_segmented_controller_default && typeof sui_segmented_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_segmented_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor5({
      anchor,
      items: () => $scope.options(),
      key: (opt, __i) => {
        const __child = Object.create($scope);
        __child.opt = opt;
        __child.__i = __i;
        return (($scope2) => opt.value)(__child);
      },
      render: (opt, __i) => {
        const __childScope2 = Object.create($scope);
        __childScope2.opt = opt;
        __childScope2.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b7 = (anchor2) => bindIf16({
            anchor: anchor2,
            test: () => {
              if ($scope2.opt.icon)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag7 = document.createDocumentFragment();
                const __c8 = [];
                const __e9 = document.createElement("span");
                __e9.setAttribute("class", "btn-icon");
                __e9.setAttribute("aria-hidden", "true");
                const __e10 = document.createElement("sui-icon");
                __c8.push(bindProp2(__e10, "name", () => $scope2.opt.icon));
                __e10.setAttribute("size", "sm");
                __e9.appendChild(__e10);
                __frag7.appendChild(__e9);
                return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
                  for (const c of __c8)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("button");
          __e5.setAttribute("class", "btn");
          __e5.setAttribute("role", "radio");
          __e5.setAttribute("type", "button");
          __c4.push(bindAttr32(__e5, "data-value", () => $scope2.opt.value));
          __c4.push(bindAttr32(__e5, "aria-checked", () => $scope2.opt.value === $scope2.value() ? "true" : "false"));
          __c4.push(bindAttr32(__e5, "aria-disabled", () => $scope2.disabled() || $scope2.opt.disabled ? "true" : "false"));
          __c4.push(bindProp2(__e5, "disabled", () => $scope2.disabled() || $scope2.opt.disabled));
          __c4.push(bindAttr32(__e5, "tabindex", () => $scope2.opt.value === $scope2.value() ? "0" : "-1"));
          const __ifAnchor6 = document.createComment("if");
          __e5.appendChild(__ifAnchor6);
          __c4.push(__wireBoundary_b7(__ifAnchor6));
          const __t11 = document.createTextNode("");
          __e5.appendChild(__t11);
          __c4.push(bindText15(__t11, () => $scope2.opt.label));
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope2);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets63(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "container");
    __e0.setAttribute("part", "container");
    attachRef35($scope, "containerEl", __e0);
    const __forAnchor1 = document.createComment("for");
    __e0.appendChild(__forAnchor1);
    __cleanups.push(__wireBoundary_b1(__forAnchor1));
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-toggle-group/sui-toggle.sui
import { adoptSuiSheets as adoptSuiSheets64, attachRef as attachRef36, bindAttr as bindAttr33, component as component64, prop as prop61 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-toggle-group/sui-toggle.controller.ts
import { controller as controller46, effect as effect41 } from "@ship/sui-runtime";
var sui_toggle_controller_default = controller46({
  mount(state, host) {
    const scope = state;
    const stop = effect41(() => {
      const btn = scope.buttonEl();
      if (!btn)
        return;
      const onClick = () => {
        if (scope.disabled())
          return;
        scope.pressed.set(!scope.pressed());
        host.dispatchEvent(new CustomEvent("sui-toggle", {
          detail: { pressed: scope.pressed(), value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    host.focusButton = () => {
      const btn = host.shadowRoot?.querySelector("button");
      btn?.focus();
    };
    return () => stop();
  }
});

// packages/ui-components/src/sui-toggle-group/sui-toggle.sui
var sui_toggle_default = component64("sui-toggle", {
  props: {
    pressed: prop61.bool({ default: false, reflect: true }),
    disabled: prop61.bool({ default: false, reflect: true }),
    value: prop61.string({ default: "", reflect: true }),
    size: prop61.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop61.union(["outline", "ghost"], { default: "outline", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; }

  :host(:not([size])), :host([size='md']) {
    --tgl-h: 36px; --tgl-px: 12px; --tgl-fs: var(--text-base, 14px);
  }
  :host([size='sm']) {
    --tgl-h: 28px; --tgl-px: 8px; --tgl-fs: var(--text-sm, 13px);
  }
  :host([size='lg']) {
    --tgl-h: 42px; --tgl-px: 16px; --tgl-fs: var(--text-base, 14px);
  }

  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: var(--tgl-h);
    padding: 0 var(--tgl-px);
    font-family: var(--font-body);
    font-size: var(--tgl-fs);
    font-weight: var(--font-weight-medium, 500);
    line-height: 1;
    color: var(--muted-foreground);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius, 6px);
    cursor: pointer;
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }
  :host([variant='ghost']) button { border-color: transparent; }
  button:hover { background: var(--muted); color: var(--foreground); }
  :host([pressed]) button {
    background: var(--muted);
    color: var(--foreground);
    border-color: var(--border);
  }
  :host([disabled]) button { cursor: not-allowed; opacity: 0.5; pointer-events: none; }

  button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--primary) 50%, transparent);
  }

  @media (forced-colors: active) {
    button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    :host([pressed]) button { outline: 2px solid CanvasText; }
  }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.pressed = props.pressed;
    $scope.disabled = props.disabled;
    $scope.value = props.value;
    $scope.size = props.size;
    $scope.variant = props.variant;
    let __r = null;
    if (sui_toggle_controller_default && typeof sui_toggle_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toggle_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets64(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __e0.setAttribute("type", "button");
    __cleanups.push(bindAttr33(__e0, "aria-pressed", () => $scope.pressed() ? "true" : "false"));
    __cleanups.push(bindAttr33(__e0, "disabled", () => $scope.disabled()));
    attachRef36($scope, "buttonEl", __e0);
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-toggle-group/sui-toggle-group.sui
import { adoptSuiSheets as adoptSuiSheets65, attachRef as attachRef37, component as component65, prop as prop62 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-toggle-group/sui-toggle-group.controller.ts
import { controller as controller47, effect as effect42 } from "@ship/sui-runtime";
var sui_toggle_group_controller_default = controller47({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "group");
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const getToggles = () => {
      const slot = scope.defaultSlotEl();
      if (!slot)
        return Array.from(host.querySelectorAll(":scope > sui-toggle"));
      return slot.assignedElements().filter((el) => el.tagName.toLowerCase() === "sui-toggle");
    };
    const enabledToggles = () => getToggles().filter((t) => !t.hasAttribute("disabled"));
    const getSelectedValues = () => new Set(scope.value().split(",").filter(Boolean));
    const setSelectedValues = (vals) => {
      const joined = Array.from(vals).join(",");
      scope.value.set(joined);
      syncToggles();
      hiddenInput.name = scope.name();
      hiddenInput.value = joined;
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: joined },
        bubbles: true,
        composed: true
      }));
    };
    const syncToggles = () => {
      const selected = getSelectedValues();
      const size = scope.size();
      const variant = scope.variant();
      for (const toggle of getToggles()) {
        const val = toggle.getAttribute("value") ?? "";
        toggle.toggleAttribute("pressed", selected.has(val));
        if (size)
          toggle.setAttribute("size", size);
        if (variant)
          toggle.setAttribute("variant", variant);
      }
    };
    const onToggle = (e) => {
      const ce = e;
      ce.stopPropagation();
      const toggle = ce.composedPath().find((el) => el.tagName?.toLowerCase() === "sui-toggle");
      if (!toggle)
        return;
      const val = toggle.getAttribute("value") ?? "";
      const selected = getSelectedValues();
      if (scope.type() === "single") {
        if (selected.has(val))
          setSelectedValues(new Set);
        else
          setSelectedValues(new Set([val]));
      } else {
        if (selected.has(val))
          selected.delete(val);
        else
          selected.add(val);
        setSelectedValues(selected);
      }
    };
    const onKeydown = (e) => {
      if (e.key !== "ArrowRight" && e.key !== "ArrowLeft")
        return;
      const items = enabledToggles();
      if (!items.length)
        return;
      const current = e.composedPath().find((el) => items.includes(el));
      const i = current ? items.indexOf(current) : -1;
      const next = e.key === "ArrowRight" ? (i + 1) % items.length : (i - 1 + items.length) % items.length;
      items[next]?.focusButton?.();
      e.preventDefault();
    };
    const stopSync = effect42(() => {
      scope.value();
      scope.size();
      scope.variant();
      syncToggles();
    });
    const stopSlot = effect42(() => {
      const slot = scope.defaultSlotEl();
      if (!slot)
        return;
      const onSlotChange = () => syncToggles();
      slot.addEventListener("slotchange", onSlotChange);
      return () => slot.removeEventListener("slotchange", onSlotChange);
    });
    const stopHidden = effect42(() => {
      hiddenInput.name = scope.name();
      hiddenInput.value = scope.value();
    });
    host.addEventListener("sui-toggle", onToggle);
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopSync();
      stopSlot();
      stopHidden();
      host.removeEventListener("sui-toggle", onToggle);
      host.removeEventListener("keydown", onKeydown);
      hiddenInput.remove();
    };
  }
});

// packages/ui-components/src/sui-toggle-group/sui-toggle-group.sui
var sui_toggle_group_default = component65("sui-toggle-group", {
  props: {
    type: prop62.union(["single", "multiple"], { default: "single", reflect: true }),
    value: prop62.string({ default: "", reflect: true }),
    size: prop62.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop62.union(["outline", "ghost"], { default: "outline", reflect: true }),
    name: prop62.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    $scope.size = props.size;
    $scope.variant = props.variant;
    $scope.name = props.name;
    let __r = null;
    if (sui_toggle_group_controller_default && typeof sui_toggle_group_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toggle_group_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets65(__root);
    const __e0 = document.createElement("slot");
    attachRef37($scope, "defaultSlotEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-code/sui-code.sui
import { adoptSuiSheets as adoptSuiSheets66, bindAttr as bindAttr34, bindIf as bindIf17, bindText as bindText16, component as component66, prop as prop63 } from "@ship/sui-runtime";
var sui_code_default = component66("sui-code", {
  props: {
    lang: prop63.string({ default: "", reflect: true }),
    block: prop63.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }
  :host([block]) { display: block; }

  .inline {
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    color: var(--foreground);
    border: 1px solid var(--border);
    font-size: 0.92em;
  }
  .block {
    display: block;
    background: var(--muted);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .block header {
    display: flex;
    justify-content: space-between;
    padding: 4px 10px;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted) 60%, var(--background));
    border-bottom: 1px solid var(--border);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: var(--font-weight-medium);
  }
  pre {
    margin: 0;
    padding: 10px 12px;
    overflow-x: auto;
    line-height: var(--leading-normal);
    font-family: inherit;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.lang = props.lang;
    $scope.block = props.block;
    const __wireBoundary_b2 = (anchor) => bindIf17({
      anchor,
      test: () => {
        if ($scope.lang())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("header");
          __e7.setAttribute("part", "label");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c6.push(bindText16(__t8, () => $scope.lang()));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf17({
      anchor,
      test: () => {
        if ($scope.block())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "block");
          __e3.setAttribute("part", "block");
          __e3.setAttribute("role", "region");
          __c2.push(bindAttr34(__e3, "aria-label", () => $scope.lang() ? $scope.lang() + " code block" : "code block"));
          const __ifAnchor4 = document.createComment("if");
          __e3.appendChild(__ifAnchor4);
          __c2.push(__wireBoundary_b2(__ifAnchor4));
          const __e9 = document.createElement("pre");
          __e9.setAttribute("part", "pre");
          const __e10 = document.createElement("slot");
          __e9.appendChild(__e10);
          __e3.appendChild(__e9);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("code");
          __e13.setAttribute("class", "inline");
          __e13.setAttribute("part", "inline");
          const __e14 = document.createElement("slot");
          __e13.appendChild(__e14);
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets66(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
    return $scope;
  }
});

// packages/ui-components/src/sui-copy-button/sui-copy-button.sui
import { adoptSuiSheets as adoptSuiSheets67, attachRef as attachRef38, bindAttr as bindAttr35, bindIf as bindIf18, bindSlotEmpty as bindSlotEmpty5, bindText as bindText17, component as component67, prop as prop64 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-copy-button/sui-copy-button.controller.ts
import { controller as controller48, effect as effect43 } from "@ship/sui-runtime";
var sui_copy_button_controller_default = controller48({
  mount(state, host) {
    const scope = state;
    let resetTimer;
    const handleClick = async () => {
      try {
        await navigator.clipboard.writeText(scope.value());
      } catch {
        const ta = document.createElement("textarea");
        ta.value = scope.value();
        document.body.appendChild(ta);
        ta.select();
        try {
          document.execCommand("copy");
        } catch {}
        ta.remove();
      }
      scope.copied.set(true);
      host.dispatchEvent(new CustomEvent("sui-copy", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
      if (resetTimer)
        clearTimeout(resetTimer);
      resetTimer = setTimeout(() => {
        scope.copied.set(false);
      }, 1200);
    };
    const stopBtn = effect43(() => {
      const btn = scope.buttonEl();
      if (!btn)
        return;
      btn.addEventListener("click", handleClick);
      return () => btn.removeEventListener("click", handleClick);
    });
    return () => {
      stopBtn();
      if (resetTimer)
        clearTimeout(resetTimer);
    };
  }
});

// packages/ui-components/src/sui-copy-button/sui-copy-button.sui
var sui_copy_button_default = component67("sui-copy-button", {
  props: {
    value: prop64.string({ default: "", reflect: true }),
    label: prop64.string({ default: "Copy", reflect: true }),
    copied: prop64.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-body);
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    padding: 4px 8px;
    color: var(--muted-foreground);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }
  button:hover {
    color: var(--foreground);
    background: var(--muted);
  }
  button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 50%), transparent);
  }
  @media (forced-colors: active) {
    button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  :host([copied]) button {
    color: var(--success);
    border-color: var(--success);
  }

  /* Show default label when slot is empty (JSDOM always reports empty, so tests see this) */
  .default-label { display: inline; }
  :host(:not([data-empty])) .default-label { display: none; }

  sui-icon {
    flex: 0 0 auto;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.label = props.label;
    $scope.copied = props.copied;
    let __r = null;
    if (sui_copy_button_controller_default && typeof sui_copy_button_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_copy_button_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf18({
      anchor,
      test: () => {
        if ($scope.copied())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("sui-icon");
          __e4.setAttribute("name", "check");
          __e4.setAttribute("size", "sm");
          __e4.setAttribute("aria-hidden", "true");
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("sui-icon");
          __e7.setAttribute("name", "copy");
          __e7.setAttribute("size", "sm");
          __e7.setAttribute("aria-hidden", "true");
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets67(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __e0.setAttribute("type", "button");
    __cleanups.push(bindAttr35(__e0, "aria-label", () => $scope.label()));
    attachRef38($scope, "buttonEl", __e0);
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b2(__ifAnchor1));
    const __e8 = document.createElement("slot");
    __cleanups.push(bindSlotEmpty5(__e8, host, "data-empty"));
    __e0.appendChild(__e8);
    const __e9 = document.createElement("span");
    __e9.setAttribute("class", "default-label");
    const __t10 = document.createTextNode("");
    __e9.appendChild(__t10);
    __cleanups.push(bindText17(__t10, () => $scope.copied() ? "Copied" : $scope.label()));
    __e0.appendChild(__e9);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-json-view/sui-json-view.sui
import { adoptSuiSheets as adoptSuiSheets68, attachRef as attachRef39, component as component68, prop as prop65 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-json-view/sui-json-view.controller.ts
import { controller as controller49, bindHtml as bindHtml2 } from "@ship/sui-runtime";
function escape(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function renderValue(v, key, level, expanded, depth) {
  const keyPrefix = key !== null ? `<span class="key">"${escape(key)}"</span>: ` : "";
  if (v === null)
    return `${keyPrefix}<span class="null">null</span>`;
  if (typeof v === "string")
    return `${keyPrefix}<span class="s">"${escape(v)}"</span>`;
  if (typeof v === "number")
    return `${keyPrefix}<span class="n">${v}</span>`;
  if (typeof v === "boolean")
    return `${keyPrefix}<span class="b">${String(v)}</span>`;
  const isArr = Array.isArray(v);
  const entries = isArr ? v.map((x, i) => [String(i), x]) : Object.entries(v);
  const open = expanded || level < depth;
  const open1 = isArr ? "[" : "{";
  const close1 = isArr ? "]" : "}";
  const count = entries.length;
  if (count === 0)
    return `${keyPrefix}${open1}${close1}`;
  const children = entries.map(([k, val]) => `<div class="node">${renderValue(val, isArr ? null : k, level + 1, expanded, depth)}</div>`).join("");
  return `<details${open ? " open" : ""}>
<summary><span class="caret">▶</span>${keyPrefix}${open1}<span class="meta">${count} ${isArr ? "items" : "keys"}</span></summary>
<div class="branch">${children}</div>
${close1}</details>`;
}
var sui_json_view_controller_default = controller49({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "tree");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "JSON tree");
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml2(el, () => {
      const data = scope.data();
      const expanded = scope.expanded();
      const depth = scope.depth();
      if (!data)
        return "";
      let parsed;
      try {
        parsed = JSON.parse(data);
      } catch (e) {
        return `<span class="error" part="error" role="alert">Invalid JSON: ${escape(e.message)}</span>`;
      }
      return `<div part="root">${renderValue(parsed, null, 0, expanded, depth)}</div>`;
    });
  }
});

// packages/ui-components/src/sui-json-view/sui-json-view.sui
var sui_json_view_default = component68("sui-json-view", {
  props: {
    data: prop65.string({ default: "", reflect: true }),
    expanded: prop65.bool({ default: false, reflect: true }),
    depth: prop65.number({ default: 1, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  .node { display: block; }
  .branch {
    margin-left: 12px;
    border-left: 1px dashed var(--border);
    padding-left: 10px;
  }
  details > summary {
    list-style: none;
    cursor: pointer;
    user-select: none;
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
  }
  details > summary::-webkit-details-marker { display: none; }
  .caret {
    display: inline-block;
    width: 10px;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast) var(--ease-out);
  }
  details[open] > summary .caret { transform: rotate(90deg); }
  .key { color: var(--primary); font-weight: var(--font-weight-medium); }
  .s { color: var(--success); }
  .n { color: var(--info); }
  .b { color: var(--accent); font-weight: var(--font-weight-medium); }
  .null { color: var(--muted-foreground); font-style: italic; }
  .meta { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: 4px; }
  .error { color: var(--destructive); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.data = props.data;
    $scope.expanded = props.expanded;
    $scope.depth = props.depth;
    let __r = null;
    if (sui_json_view_controller_default && typeof sui_json_view_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_json_view_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets68(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "json-root");
    attachRef39($scope, "contentEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-diff/sui-diff.sui
import { adoptSuiSheets as adoptSuiSheets69, attachRef as attachRef40, component as component69, prop as prop66 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-diff/sui-diff.controller.ts
import { controller as controller50, bindHtml as bindHtml3 } from "@ship/sui-runtime";
function escape2(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function diffLines(a, b) {
  const A = a.split(`
`);
  const B = b.split(`
`);
  const n = A.length;
  const m = B.length;
  const dp = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i2 = n - 1;i2 >= 0; i2--) {
    for (let j2 = m - 1;j2 >= 0; j2--) {
      dp[i2][j2] = A[i2] === B[j2] ? dp[i2 + 1][j2 + 1] + 1 : Math.max(dp[i2 + 1][j2], dp[i2][j2 + 1]);
    }
  }
  const out = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (A[i] === B[j]) {
      out.push({ kind: "ctx", text: A[i] });
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      out.push({ kind: "del", text: A[i] });
      i++;
    } else {
      out.push({ kind: "add", text: B[j] });
      j++;
    }
  }
  while (i < n)
    out.push({ kind: "del", text: A[i++] });
  while (j < m)
    out.push({ kind: "add", text: B[j++] });
  return out;
}
function parseUnified(value) {
  return value.split(`
`).map((line) => {
    if (line.startsWith("@@"))
      return { kind: "hunk", text: line };
    if (line.startsWith("+"))
      return { kind: "add", text: line.slice(1) };
    if (line.startsWith("-"))
      return { kind: "del", text: line.slice(1) };
    return { kind: "ctx", text: line.startsWith(" ") ? line.slice(1) : line };
  });
}
function renderLines(lines) {
  return lines.map((l) => {
    const sign = l.kind === "add" ? "+" : l.kind === "del" ? "-" : l.kind === "hunk" ? "@" : " ";
    const kindLabel = l.kind === "add" ? "added" : l.kind === "del" ? "removed" : l.kind === "hunk" ? "hunk" : "context";
    return `<div class="line ${l.kind}" part="line" role="listitem" aria-label="${kindLabel}: ${escape2(l.text)}"><span class="sign" aria-hidden="true">${sign}</span><span>${escape2(l.text)}</span></div>`;
  }).join("");
}
var sui_diff_controller_default = controller50({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "region");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "diff");
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml3(el, () => {
      const value = scope.value();
      const from = scope.from();
      const to = scope.to();
      let lines;
      if (value) {
        lines = parseUnified(value);
      } else if (from || to) {
        lines = diffLines(from, to);
      } else {
        lines = [];
      }
      return renderLines(lines);
    });
  }
});

// packages/ui-components/src/sui-diff/sui-diff.sui
var sui_diff_default = component69("sui-diff", {
  props: {
    from: prop66.string({ default: "", reflect: true }),
    to: prop66.string({ default: "", reflect: true }),
    value: prop66.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .line {
    display: grid;
    grid-template-columns: 16px 1fr;
    padding: 1px 8px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .line.add { background: var(--tone-success-bg); color: var(--foreground); }
  .line.del { background: var(--tone-destructive-bg); color: var(--foreground); }
  .line.hunk { background: var(--tone-info-bg); color: var(--info); }
  .sign { color: var(--muted-foreground); user-select: none; }
  .line.add .sign { color: var(--success); }
  .line.del .sign { color: var(--destructive); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.from = props.from;
    $scope.to = props.to;
    $scope.value = props.value;
    let __r = null;
    if (sui_diff_controller_default && typeof sui_diff_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_diff_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets69(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "diff-root");
    attachRef40($scope, "contentEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-markdown/sui-markdown.sui
import { adoptSuiSheets as adoptSuiSheets70, attachRef as attachRef41, component as component70, prop as prop67 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-markdown/sui-markdown.controller.ts
import { controller as controller51, bindHtml as bindHtml4 } from "@ship/sui-runtime";
function escape3(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function inline(s) {
  let out = escape3(s);
  out = out.replace(/`([^`]+)`/g, (_, c) => `<code>${c}</code>`);
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<em>$2</em>");
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`);
  return out;
}
function renderMarkdown(src) {
  const lines = src.replace(/\r\n/g, `
`).split(`
`);
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim();
      const buf2 = [];
      i++;
      while (i < lines.length && !/^```/.test(lines[i]))
        buf2.push(lines[i++]);
      i++;
      out.push(`<pre data-lang="${escape3(lang)}"><code>${escape3(buf2.join(`
`))}</code></pre>`);
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h) {
      out.push(`<h${h[1].length}>${inline(h[2])}</h${h[1].length}>`);
      i++;
      continue;
    }
    if (/^\s*[-*]\s+/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^\s*[-*]\s+/.test(lines[i])) {
        buf2.push(`<li>${inline(lines[i].replace(/^\s*[-*]\s+/, ""))}</li>`);
        i++;
      }
      out.push(`<ul>${buf2.join("")}</ul>`);
      continue;
    }
    if (/^\s*\d+\.\s+/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^\s*\d+\.\s+/.test(lines[i])) {
        buf2.push(`<li>${inline(lines[i].replace(/^\s*\d+\.\s+/, ""))}</li>`);
        i++;
      }
      out.push(`<ol>${buf2.join("")}</ol>`);
      continue;
    }
    if (/^>\s?/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        buf2.push(lines[i].replace(/^>\s?/, ""));
        i++;
      }
      out.push(`<blockquote>${inline(buf2.join(" "))}</blockquote>`);
      continue;
    }
    if (line.trim() === "") {
      i++;
      continue;
    }
    const buf = [];
    while (i < lines.length && lines[i].trim() !== "" && !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(lines[i])) {
      buf.push(lines[i]);
      i++;
    }
    out.push(`<p>${inline(buf.join(" "))}</p>`);
  }
  return out.join(`
`);
}
var sui_markdown_controller_default = controller51({
  mount(state) {
    const scope = state;
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml4(el, () => renderMarkdown(scope.value() ?? ""));
  }
});

// packages/ui-components/src/sui-markdown/sui-markdown.sui
var sui_markdown_default = component70("sui-markdown", {
  props: {
    value: prop67.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  h1, h2, h3, h4, h5, h6 {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--font-weight-semibold);
    line-height: var(--leading-tight);
    margin: 1em 0 0.4em;
  }
  h1 { font-size: var(--text-2xl); }
  h2 { font-size: var(--text-xl); }
  h3 { font-size: var(--text-lg); }
  h4, h5, h6 { font-size: var(--text-md); }
  p { margin: 0.5em 0; }
  a { color: var(--primary); text-decoration: underline; text-underline-offset: 2px; }
  code {
    font-family: var(--font-mono);
    font-size: 0.92em;
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    border: 1px solid var(--border);
  }
  pre {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    overflow-x: auto;
  }
  pre code { background: transparent; border: none; padding: 0; }
  blockquote {
    margin: 0.6em 0;
    padding: 4px 12px;
    border-left: 3px solid var(--border);
    color: var(--muted-foreground);
  }
  ul, ol { margin: 0.4em 0; padding-left: 22px; }
  li { margin: 2px 0; }
  strong { font-weight: var(--font-weight-semibold); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    let __r = null;
    if (sui_markdown_controller_default && typeof sui_markdown_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_markdown_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets70(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "md-root");
    attachRef41($scope, "contentEl", __e0);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-toolbar/sui-toolbar.sui
import { adoptSuiSheets as adoptSuiSheets71, attachRef as attachRef42, component as component71, prop as prop68 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-toolbar/sui-toolbar.controller.ts
import { controller as controller52 } from "@ship/sui-runtime";
var sui_toolbar_controller_default = controller52({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "toolbar");
    if (!host.hasAttribute("aria-orientation"))
      host.setAttribute("aria-orientation", "horizontal");
    const focusable = () => {
      const slot = scope.defaultSlot();
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      return els.filter((el) => !el.hasAttribute("disabled"));
    };
    const onKey = (e) => {
      if (e.key !== "ArrowRight" && e.key !== "ArrowLeft")
        return;
      const items = focusable();
      if (!items.length)
        return;
      const active = e.composedPath().find((n) => items.includes(n)) || null;
      const i = active ? items.indexOf(active) : -1;
      const next = e.key === "ArrowRight" ? (i + 1) % items.length : (i - 1 + items.length) % items.length;
      items[next]?.focus();
      e.preventDefault();
    };
    host.addEventListener("keydown", onKey);
    return () => host.removeEventListener("keydown", onKey);
  }
});

// packages/ui-components/src/sui-toolbar/sui-toolbar.sui
var sui_toolbar_default = component71("sui-toolbar", {
  props: {
    justify: prop68.union(["start", "center", "end", "between"], { default: "start", reflect: true }),
    align: prop68.union(["center", "start", "end", "baseline"], { default: "center", reflect: true }),
    gap: prop68.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop68.union(["default", "bordered"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  :host([gap='sm']) { gap: 4px; }
  :host([gap='lg']) { gap: 12px; }
  :host([justify='center']) { justify-content: center; }
  :host([justify='end']) { justify-content: flex-end; }
  :host([justify='between']) { justify-content: space-between; }
  :host([align='start']) { align-items: flex-start; }
  :host([align='end']) { align-items: flex-end; }
  :host([align='baseline']) { align-items: baseline; }
  :host([variant='bordered']) {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }
  ::slotted([slot='spacer']) { flex: 1; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.justify = props.justify;
    $scope.align = props.align;
    $scope.gap = props.gap;
    $scope.variant = props.variant;
    let __r = null;
    if (sui_toolbar_controller_default && typeof sui_toolbar_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toolbar_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets71(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "spacer");
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    attachRef42($scope, "defaultSlot", __e1);
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

// packages/ui-components/src/sui-app-icon/sui-app-icon.sui
import { adoptSuiSheets as adoptSuiSheets72, attachRef as attachRef43, component as component72, prop as prop69 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-app-icon/sui-app-icon.controller.ts
import { controller as controller53, effect as effect44 } from "@ship/sui-runtime";
var APP_GLYPH = {
  dashboard: "layout-dashboard",
  pipelines: "workflow",
  agents: "bot",
  skills: "sparkles",
  git: "git-branch",
  collections: "database",
  canvas: "frame",
  settings: "settings",
  chat: "messages-square",
  events: "radio",
  specs: "file-text",
  theme: "palette",
  setup: "rocket",
  overview: "compass",
  mesh: "network"
};
var APP_ALIASES = {
  "theme-studio": "theme"
};
var APP_NAMES = new Set([...Object.keys(APP_GLYPH), ...Object.keys(APP_ALIASES)]);
var sui_app_icon_controller_default = controller53({
  mount(state, host) {
    const scope = state;
    const stopEffect = effect44(() => {
      const name = scope.name();
      const label = scope.label();
      const resolved = APP_ALIASES[name] ?? name;
      const known = APP_NAMES.has(name);
      const glyph = known ? APP_GLYPH[resolved] : "dot";
      const icon = scope.glyphEl();
      if (icon)
        icon.setAttribute("name", glyph);
      if (label) {
        host.setAttribute("role", "img");
        host.setAttribute("aria-label", label);
        host.removeAttribute("aria-hidden");
      } else {
        host.setAttribute("aria-hidden", "true");
        host.removeAttribute("role");
        host.removeAttribute("aria-label");
      }
    });
    return () => stopEffect();
  }
});

// packages/ui-components/src/sui-app-icon/sui-app-icon.sui
var sui_app_icon_default = component72("sui-app-icon", {
  props: {
    name: prop69.string({ default: "", reflect: true }),
    variant: prop69.union(["glyph", "tile", "solid"], { default: "glyph", reflect: true }),
    size: prop69.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    radius: prop69.string({ default: "", reflect: true }),
    label: prop69.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;

    --app-accent: var(--foreground);
    --tile-size: 20px;
    --glyph-size: 14px;
    --tile-radius: var(--radius-md);

    width: var(--tile-size);
    height: var(--tile-size);
    color: var(--app-accent);
  }

  :host([name='dashboard'])   { --app-accent: var(--app-accent-dashboard); }
  :host([name='pipelines'])   { --app-accent: var(--app-accent-pipelines); }
  :host([name='agents'])      { --app-accent: var(--app-accent-agents); }
  :host([name='skills'])      { --app-accent: var(--app-accent-skills); }
  :host([name='git'])         { --app-accent: var(--app-accent-git); }
  :host([name='collections']) { --app-accent: var(--app-accent-collections); }
  :host([name='canvas'])      { --app-accent: var(--app-accent-canvas); }
  :host([name='settings'])    { --app-accent: var(--app-accent-settings); }
  :host([name='chat'])        { --app-accent: var(--app-accent-chat); }
  :host([name='events'])      { --app-accent: var(--app-accent-events); }
  :host([name='specs'])       { --app-accent: var(--app-accent-specs); }
  :host([name='theme'])       { --app-accent: var(--app-accent-theme); }
  :host([name='setup'])       { --app-accent: var(--app-accent-setup); }
  :host([name='overview'])    { --app-accent: var(--app-accent-overview); }
  :host([name='mesh'])        { --app-accent: var(--app-accent-mesh); }
  :host([name='theme-studio']) { --app-accent: var(--app-accent-theme); }

  :host([size='sm']) { --tile-size: 16px; --glyph-size: 12px; --tile-radius: var(--radius-sm); }
  :host([size='md']) { --tile-size: 20px; --glyph-size: 14px; --tile-radius: var(--radius-md); }
  :host([size='lg']) { --tile-size: 28px; --glyph-size: 18px; --tile-radius: var(--radius-lg); }

  :host([radius='sm'])   { --tile-radius: var(--radius-sm); }
  :host([radius='md'])   { --tile-radius: var(--radius-md); }
  :host([radius='lg'])   { --tile-radius: var(--radius-lg); }
  :host([radius='full']) { --tile-radius: var(--radius-full); }

  .frame {
    width: 100%;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--tile-radius);
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                box-shadow var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .frame,
  :host([variant='glyph']) .frame {
    background: transparent;
    color: var(--app-accent);
  }

  :host([variant='tile']) .frame {
    background: color-mix(in oklab, var(--app-accent) 16%, transparent);
    color: var(--app-accent);
  }

  :host([variant='solid']) .frame {
    background: var(--app-accent);
    color: var(--background);
    box-shadow: var(--shadow-xs);
  }

  sui-icon {
    width: var(--glyph-size);
    height: var(--glyph-size);
  }
  ::slotted(*) {
    width: var(--glyph-size);
    height: var(--glyph-size);
    display: block;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.radius = props.radius;
    $scope.label = props.label;
    let __r = null;
    if (sui_app_icon_controller_default && typeof sui_app_icon_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_app_icon_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets72(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "frame");
    __e0.setAttribute("part", "frame");
    const __e1 = document.createElement("slot");
    const __e2 = document.createElement("sui-icon");
    attachRef43($scope, "glyphEl", __e2);
    __e2.setAttribute("part", "glyph");
    __e2.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
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

// packages/ui-components/src/sui-tree-view/sui-tree-view.sui
import { adoptSuiSheets as adoptSuiSheets73, bindFor as bindFor6, bindIf as bindIf19, bindProp as bindProp3, component as component73 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-tree-view/sui-tree-view.controller.ts
import { controller as controller54, signal as signal12 } from "@ship/sui-runtime";
var sui_tree_view_controller_default = controller54({
  scope: () => {
    const nodes = signal12([]);
    return { nodes };
  },
  mount(state, host) {
    const scope = state;
    const nodesSignal = scope.nodes;
    const initNodes = host.nodes;
    if (initNodes != null)
      nodesSignal.set(initNodes);
    Object.defineProperty(host, "nodes", {
      configurable: true,
      get: () => nodesSignal(),
      set: (v) => nodesSignal.set(v)
    });
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "tree");
  }
});

// packages/ui-components/src/sui-tree-view/sui-tree-view.sui
var sui_tree_view_default = component73("sui-tree-view", {
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_tree_view_controller_default && typeof sui_tree_view_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tree_view_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor6({
      anchor,
      items: () => $scope.nodes(),
      key: (n, __i) => {
        const __child = Object.create($scope);
        __child.n = n;
        __child.__i = __i;
        return (($scope2) => n.id)(__child);
      },
      render: (n, __i) => {
        const __childScope5 = Object.create($scope);
        __childScope5.n = n;
        __childScope5.__i = __i;
        return (($scope2) => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("sui-tree-item");
          __c7.push(bindProp3(__e8, "node", () => $scope2.n));
          __frag6.appendChild(__e8);
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope5);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf19({
      anchor,
      test: () => {
        if ($scope.nodes().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("ul");
          __e3.setAttribute("role", "tree");
          const __forAnchor4 = document.createComment("for");
          __e3.appendChild(__forAnchor4);
          __c2.push(__wireBoundary_b1(__forAnchor4));
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("slot");
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets73(__root);
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

// packages/ui-components/src/sui-tree-view/sui-tree-item.sui
import { adoptSuiSheets as adoptSuiSheets74, attachRef as attachRef44, bindFor as bindFor7, bindIf as bindIf20, bindProp as bindProp4, bindText as bindText18, component as component74, prop as prop70 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-tree-view/sui-tree-item.controller.ts
import { controller as controller55, signal as signal13, derived as derived4, effect as effect45, ref as ref6 } from "@ship/sui-runtime";
var sui_tree_item_controller_default = controller55({
  scope: ({ label }) => {
    const node = signal13(null);
    const nodeLabel = derived4(() => {
      const n = node();
      return n ? n.label : label();
    });
    const nodeChildren = derived4(() => {
      const n = node();
      return n?.children ?? [];
    });
    const rowEl = ref6(null);
    return { node, nodeLabel, nodeChildren, rowEl };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "treeitem");
    const nodeSignal = scope.node;
    const initNode = host.node;
    if (initNode != null)
      nodeSignal.set(initNode);
    Object.defineProperty(host, "node", {
      configurable: true,
      get: () => nodeSignal(),
      set: (v) => nodeSignal.set(v)
    });
    const stopLeafSync = effect45(() => {
      const n = nodeSignal();
      if (n !== null) {
        const isLeaf = !n.children || n.children.length === 0;
        if (isLeaf)
          host.setAttribute("leaf", "");
        else
          host.removeAttribute("leaf");
      }
    });
    const syncAria = effect45(() => {
      const row = scope.rowEl();
      if (!row)
        return;
      if (!scope.leaf()) {
        row.setAttribute("aria-expanded", String(scope.expanded()));
      } else {
        row.removeAttribute("aria-expanded");
      }
      row.setAttribute("aria-selected", String(scope.selected()));
    });
    const toggle = (e) => {
      if (scope.leaf())
        return;
      e.stopPropagation();
      const next = !scope.expanded();
      scope.expanded.set(next);
      host.dispatchEvent(new CustomEvent("sui-toggle", {
        detail: { expanded: next },
        bubbles: true,
        composed: true
      }));
    };
    const select = () => {
      host.dispatchEvent(new CustomEvent("sui-select", {
        bubbles: true,
        composed: true,
        detail: {}
      }));
    };
    const onRowClick = (e) => {
      select();
      toggle(e);
    };
    const onKey = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        select();
      } else if (e.key === "ArrowRight" && !scope.leaf() && !scope.expanded()) {
        e.preventDefault();
        toggle(e);
      } else if (e.key === "ArrowLeft" && scope.expanded()) {
        e.preventDefault();
        toggle(e);
      }
    };
    const stopRow = effect45(() => {
      const row = scope.rowEl();
      if (!row)
        return;
      row.addEventListener("click", onRowClick);
      row.addEventListener("keydown", onKey);
      return () => {
        row.removeEventListener("click", onRowClick);
        row.removeEventListener("keydown", onKey);
      };
    });
    return () => {
      stopLeafSync();
      syncAria();
      stopRow();
    };
  }
});

// packages/ui-components/src/sui-tree-view/sui-tree-item.sui
var sui_tree_item_default = component74("sui-tree-item", {
  props: {
    label: prop70.string({ default: "", reflect: true }),
    expanded: prop70.bool({ default: false, reflect: true }),
    selected: prop70.bool({ default: false, reflect: true }),
    leaf: prop70.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }

  .row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    outline: none;
    color: var(--foreground);
    transition: background-color var(--duration-fast) var(--ease-out);
  }
  .row:hover { background: var(--muted); }
  :host([selected]) .row {
    background: var(--tone-primary-bg);
    color: var(--foreground);
    font-weight: var(--font-weight-medium);
  }
  .row:focus-visible {
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  @media (forced-colors: active) {
    .row:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  .caret {
    display: inline-flex;
    width: 12px;
    height: 12px;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast) var(--ease-out);
  }
  :host([expanded]) .caret { transform: rotate(90deg); }
  :host([leaf]) .caret { visibility: hidden; }

  .label {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .trailing { margin-left: auto; color: var(--muted-foreground); font-size: var(--text-xs); }

  .children {
    margin-left: 16px;
    border-left: 1px dashed var(--border);
    padding-left: 6px;
  }
  :host(:not([expanded])) .children { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.label = props.label;
    $scope.expanded = props.expanded;
    $scope.selected = props.selected;
    $scope.leaf = props.leaf;
    let __r = null;
    if (sui_tree_item_controller_default && typeof sui_tree_item_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tree_item_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b3 = (anchor) => bindFor7({
      anchor,
      items: () => $scope.nodeChildren(),
      key: (c, __i) => {
        const __child = Object.create($scope);
        __child.c = c;
        __child.__i = __i;
        return (($scope2) => c.id)(__child);
      },
      render: (c, __i) => {
        const __childScope14 = Object.create($scope);
        __childScope14.c = c;
        __childScope14.__i = __i;
        return (($scope2) => {
          const __frag15 = document.createDocumentFragment();
          const __c16 = [];
          const __e17 = document.createElement("sui-tree-item");
          __c16.push(bindProp4(__e17, "node", () => $scope2.c));
          __frag15.appendChild(__e17);
          return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
            for (const c2 of __c16)
              try {
                c2();
              } catch (e) {}
          } };
        })(__childScope14);
      }
    });
    const __wireBoundary_b2 = (anchor) => bindIf20({
      anchor,
      test: () => {
        if ($scope.nodeChildren().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __forAnchor13 = document.createComment("for");
          __frag11.appendChild(__forAnchor13);
          __c12.push(__wireBoundary_b3(__forAnchor13));
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("slot");
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets74(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "row");
    __e0.setAttribute("part", "row");
    __e0.setAttribute("tabindex", "0");
    attachRef44($scope, "rowEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "caret");
    __e1.setAttribute("part", "caret");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElement("svg");
    __e2.setAttribute("viewbox", "0 0 12 12");
    __e2.setAttribute("fill", "none");
    __e2.setAttribute("stroke", "currentColor");
    __e2.setAttribute("stroke-width", "1.5");
    const __e3 = document.createElement("polyline");
    __e3.setAttribute("points", "4 3 8 6 4 9");
    __e2.appendChild(__e3);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e4 = document.createElement("slot");
    __e4.setAttribute("name", "icon");
    __e0.appendChild(__e4);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "label");
    __e5.setAttribute("part", "label");
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText18(__t6, () => $scope.nodeLabel()));
    __e0.appendChild(__e5);
    const __e7 = document.createElement("span");
    __e7.setAttribute("class", "trailing");
    const __e8 = document.createElement("slot");
    __e8.setAttribute("name", "trailing");
    __e7.appendChild(__e8);
    __e0.appendChild(__e7);
    __root.appendChild(__e0);
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "children");
    __e9.setAttribute("part", "children");
    __e9.setAttribute("role", "group");
    const __ifAnchor10 = document.createComment("if");
    __e9.appendChild(__ifAnchor10);
    __cleanups.push(__wireBoundary_b2(__ifAnchor10));
    __root.appendChild(__e9);
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

// packages/ui-components/src/sui-table/sui-table.sui
import { adoptSuiSheets as adoptSuiSheets75, bindAttr as bindAttr36, bindClass, bindFor as bindFor8, bindIf as bindIf21, bindText as bindText19, component as component75, on as on4, prop as prop71 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-table/sui-table.controller.ts
import { controller as controller56, signal as signal14, derived as derived5, bindNativeEvent as bindNativeEvent5 } from "@ship/sui-runtime";
var lightDomInstalled = new WeakSet;
function installLightDomStyles(doc) {
  if (lightDomInstalled.has(doc))
    return;
  lightDomInstalled.add(doc);
  try {
    if (typeof CSSStyleSheet !== "undefined" && "replaceSync" in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet;
      sheet.replaceSync(SUI_TABLE_LIGHT_DOM_CSS);
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      return;
    }
  } catch {}
  const style = doc.createElement("style");
  style.setAttribute("data-sui-table", "");
  style.textContent = SUI_TABLE_LIGHT_DOM_CSS;
  doc.head.appendChild(style);
}
var sui_table_controller_default = controller56({
  scope: () => {
    const rows = signal14([]);
    const columns = signal14([]);
    const sortKey = signal14(null);
    const sortDir = signal14("asc");
    const selected = signal14([]);
    const sortedRows = derived5(() => {
      const key = sortKey();
      const dir = sortDir();
      const data = rows();
      if (!key)
        return data;
      return [...data].sort((a, b) => {
        const av = String(a[key] ?? "");
        const bv = String(b[key] ?? "");
        const cmp = av < bv ? -1 : av > bv ? 1 : 0;
        return dir === "asc" ? cmp : -cmp;
      });
    });
    return { rows, columns, sortKey, sortDir, selected, sortedRows };
  },
  on: {
    sortBy(s, field) {
      if (s.sortKey === field) {
        s.sortDir = s.sortDir === "asc" ? "desc" : "asc";
      } else {
        s.sortKey = field;
        s.sortDir = "asc";
      }
    },
    toggleSelect(s, id) {
      const cur = s.selected;
      s.selected = cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id];
    },
    selectAll(s) {
      s.selected = s.rows.map((r) => r.id);
    },
    clearSelection(s) {
      s.selected = [];
    }
  },
  mount(state, host) {
    const scope = state;
    const rowsSignal = scope.rows;
    const colsSignal = scope.columns;
    const initRows = host.rows;
    if (initRows != null)
      rowsSignal.set(initRows);
    const initCols = host.columns;
    if (initCols != null)
      colsSignal.set(initCols);
    Object.defineProperty(host, "rows", {
      configurable: true,
      get: () => rowsSignal(),
      set: (v) => rowsSignal.set(v)
    });
    Object.defineProperty(host, "columns", {
      configurable: true,
      get: () => colsSignal(),
      set: (v) => colsSignal.set(v)
    });
    installLightDomStyles(host.ownerDocument);
    const paintSortChevrons = () => {
      host.querySelectorAll("th[data-sort-key]").forEach((th) => {
        const dir = th.getAttribute("aria-sort") || "none";
        const ind = th.querySelector(".sui-sort-indicator");
        if (!ind)
          return;
        if (dir === "ascending") {
          ind.style.transform = "rotate(180deg)";
          ind.style.opacity = "1";
        } else if (dir === "descending") {
          ind.style.transform = "rotate(0deg)";
          ind.style.opacity = "1";
        } else {
          ind.style.transform = "rotate(0deg)";
          ind.style.opacity = "0.4";
        }
      });
    };
    const enhanceSortableHeaders = () => {
      host.querySelectorAll("th[data-sort-key]").forEach((th) => {
        if (!th.hasAttribute("aria-sort"))
          th.setAttribute("aria-sort", "none");
        if (!th.hasAttribute("tabindex"))
          th.setAttribute("tabindex", "0");
        if (!th.hasAttribute("role"))
          th.setAttribute("role", "columnheader");
        if (!th.querySelector(".sui-sort-indicator")) {
          const chev = document.createElement("span");
          chev.className = "sui-sort-indicator";
          chev.setAttribute("aria-hidden", "true");
          chev.style.cssText = "margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";
          const svgNS = "http://www.w3.org/2000/svg";
          const svg = document.createElementNS(svgNS, "svg");
          svg.setAttribute("width", "10");
          svg.setAttribute("height", "10");
          svg.setAttribute("viewBox", "0 0 10 10");
          svg.setAttribute("fill", "none");
          svg.setAttribute("stroke", "currentColor");
          svg.setAttribute("stroke-width", "1.5");
          svg.setAttribute("stroke-linecap", "round");
          svg.setAttribute("stroke-linejoin", "round");
          svg.setAttribute("aria-hidden", "true");
          const path = document.createElementNS(svgNS, "path");
          path.setAttribute("d", "M2 4l3-2 3 2M2 6l3 2 3-2");
          svg.appendChild(path);
          chev.appendChild(svg);
          th.appendChild(chev);
        }
      });
      paintSortChevrons();
    };
    const syncEmptyState = () => {
      const bodyRow = host.querySelector(":scope > table tbody > tr") || host.querySelector(":scope > table > tr:not(:has(th))") || host.querySelector(":scope > tbody > tr") || host.querySelector(':scope > tr:not([slot="head"])');
      const hasEmptySlot = host.querySelector('[slot="empty"]') != null;
      if (!bodyRow && hasEmptySlot)
        host.setAttribute("data-empty", "");
      else
        host.removeAttribute("data-empty");
    };
    const cycleSort = (th) => {
      const key = th.getAttribute("data-sort-key") || "";
      const current = th.getAttribute("aria-sort") || "none";
      const next = current === "none" ? "ascending" : current === "ascending" ? "descending" : "none";
      host.querySelectorAll("th[data-sort-key]").forEach((peer) => {
        if (peer !== th)
          peer.setAttribute("aria-sort", "none");
      });
      th.setAttribute("aria-sort", next);
      paintSortChevrons();
      host.emit("sui-sort", { key, direction: next });
    };
    const onHeaderClick = (e) => {
      const th = e.target.closest("th[data-sort-key]");
      if (!th || !host.contains(th))
        return;
      cycleSort(th);
    };
    const onHeaderKey = (e) => {
      if (e.key !== "Enter" && e.key !== " ")
        return;
      const th = e.target.closest("th[data-sort-key]");
      if (!th || !host.contains(th))
        return;
      e.preventDefault();
      cycleSort(th);
    };
    const onRowClick = (e) => {
      const td = e.target.closest("td");
      if (!td || !host.contains(td))
        return;
      if (td.classList.contains("sui-table-gutter-cell"))
        return;
      const tr = td.closest("tr");
      if (!tr)
        return;
      const rowId = tr.getAttribute("data-row-id");
      if (!rowId)
        return;
      host.emit("sui-row-click", { id: rowId, index: tr.rowIndex });
    };
    bindNativeEvent5(host, "click", onHeaderClick);
    bindNativeEvent5(host, "click", onRowClick);
    bindNativeEvent5(host, "keydown", onHeaderKey);
    enhanceSortableHeaders();
    syncEmptyState();
    const observer = new MutationObserver(() => {
      enhanceSortableHeaders();
      syncEmptyState();
    });
    observer.observe(host, { childList: true, subtree: true });
    return () => {
      observer.disconnect();
    };
  }
});
var SUI_TABLE_LIGHT_DOM_CSS = `
  sui-table {
    --sui-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  sui-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  sui-table th,
  sui-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--sui-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  sui-table th {
    padding-top: var(--th-py, 10px);
    padding-bottom: var(--th-py, 10px);
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--th-fs, 10.5px);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
    user-select: none;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table > tr:last-of-type > *,
  sui-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  sui-table th[align='right'], sui-table td[align='right'] { text-align: right; }
  sui-table th[align='center'], sui-table td[align='center'] { text-align: center; }

  sui-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  sui-table .muted { color: var(--muted-foreground); }
  sui-table .null { color: var(--muted-foreground); font-style: italic; }

  sui-table > tr:not([slot='head']),
  sui-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  sui-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  sui-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table tr[data-selected] > *,
  sui-table tr[aria-selected='true'] > * {
    background: var(--tone-primary-bg);
  }
  sui-table tr[data-selected] > *:first-child,
  sui-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  sui-table[sticky-header] thead th,
  sui-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  sui-table th[data-sort-key],
  sui-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table th[data-sort-key]:hover,
  sui-table th.sortable:hover {
    color: var(--foreground);
  }
  sui-table th[aria-sort='ascending'],
  sui-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  sui-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  sui-table[data-empty] > table {
    display: none;
  }

  sui-table[sticky-first-col] th:first-child,
  sui-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  sui-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table[sticky-first-col] tbody tr:hover td:first-child {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[gutter] th.sui-table-gutter-cell,
  sui-table[gutter] td.sui-table-gutter-cell {
    width: 40px;
    min-width: 40px;
    max-width: 40px;
    padding: 0;
    text-align: center;
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted-foreground) 5%, var(--background));
    border-right: 1px solid var(--border);
    position: sticky;
    left: 0;
    z-index: 2;
  }
  sui-table[gutter] thead th.sui-table-gutter-cell { z-index: 4; }

  sui-table tr[data-row-id] > td:not(.sui-table-gutter-cell) {
    cursor: pointer;
  }

  @media (forced-colors: active) {
    sui-table th, sui-table td { border-bottom-color: CanvasText; }
  }
`;

// packages/ui-components/src/sui-table/sui-table.sui
var sui_table_default = component75("sui-table", {
  props: {
    variant: prop71.union(["slot", "native", "data"], { default: "slot", reflect: true }),
    density: prop71.union(["compact", "default", "comfortable"], { default: "default", reflect: true }),
    dense: prop71.bool({ default: false, reflect: true }),
    hoverable: prop71.bool({ default: true, reflect: true }),
    striped: prop71.bool({ default: false, reflect: true }),
    "sticky-header": prop71.bool({ default: false, reflect: true }),
    "sticky-first-col": prop71.bool({ default: false, reflect: true }),
    gutter: prop71.bool({ default: false, reflect: true }),
    label: prop71.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    width: 100%;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);

    --row-py: 10px;
    --row-px: var(--space-3);
    --th-py: 10px;
    --th-fs: var(--text-xs);
    --table-radius: var(--radius-lg);
    --row-hover-mix: 45%;
  }

  :host([density='compact']),
  :host([dense]) {
    --row-py: 6px;
    --row-px: var(--space-2);
    --th-py: 6px;
  }
  :host([density='comfortable']) {
    --row-py: 14px;
    --row-px: var(--space-4);
    --th-py: 14px;
  }

  .wrap {
    position: relative;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--table-radius);
    background: var(--card);
    box-shadow: var(--shadow-xs);
    overflow: hidden;
    overflow-x: auto;
    transition: box-shadow var(--duration-fast) var(--ease-out);
  }

  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  :host([sticky-header]) { display: block; }
  :host([sticky-header]) .wrap {
    overflow-y: auto;
    max-height: inherit;
  }

  .empty {
    display: none;
    padding: var(--space-7) var(--space-5);
    text-align: center;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    background: var(--card);
  }
  :host([data-empty]) .empty { display: block; }
  :host([data-empty]) table { display: none; }

  @media (forced-colors: active) {
    .wrap { border-color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.density = props.density;
    $scope.dense = props.dense;
    $scope.hoverable = props.hoverable;
    $scope.striped = props.striped;
    $scope.stickyHeader = props.stickyHeader;
    $scope.stickyFirstCol = props.stickyFirstCol;
    $scope.gutter = props.gutter;
    $scope.label = props.label;
    let __r = null;
    if (sui_table_controller_default && typeof sui_table_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_table_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindFor8({
      anchor,
      items: () => $scope.columns(),
      key: (c, __i) => {
        const __child = Object.create($scope);
        __child.c = c;
        __child.__i = __i;
        return (($scope2) => c.field)(__child);
      },
      render: (c, __i) => {
        const __childScope8 = Object.create($scope);
        __childScope8.c = c;
        __childScope8.__i = __i;
        return (($scope2) => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("th");
          __c10.push(on4(__e11, "click", function() {
            const __h = () => $scope2.sortBy($scope2.c.field);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          __c10.push(bindClass(__e11, "sorted", () => $scope2.sortKey() === $scope2.c.field));
          const __t12 = document.createTextNode("");
          __e11.appendChild(__t12);
          __c10.push(bindText19(__t12, () => $scope2.c.label));
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c2 of __c10)
              try {
                c2();
              } catch (e) {}
          } };
        })(__childScope8);
      }
    });
    const __wireBoundary_b6 = (anchor) => bindFor8({
      anchor,
      items: () => $scope.sortedRows(),
      key: (r, __i) => {
        const __child = Object.create($scope);
        __child.r = r;
        __child.__i = __i;
        return (($scope2) => r.id)(__child);
      },
      render: (r, __i) => {
        const __childScope15 = Object.create($scope);
        __childScope15.r = r;
        __childScope15.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b9 = (anchor2) => bindFor8({
            anchor: anchor2,
            items: () => $scope2.columns(),
            key: (c, __i2) => {
              const __child = Object.create($scope2);
              __child.c = c;
              __child.__i = __i2;
              return (($scope3) => c.field)(__child);
            },
            render: (c, __i2) => {
              const __childScope20 = Object.create($scope2);
              __childScope20.c = c;
              __childScope20.__i = __i2;
              return (($scope3) => {
                const __frag21 = document.createDocumentFragment();
                const __c22 = [];
                const __e23 = document.createElement("td");
                const __t24 = document.createTextNode("");
                __e23.appendChild(__t24);
                __c22.push(bindText19(__t24, () => $scope3.r[$scope3.c.field]));
                __frag21.appendChild(__e23);
                return { nodes: Array.from(__frag21.childNodes), cleanup: () => {
                  for (const c2 of __c22)
                    try {
                      c2();
                    } catch (e) {}
                } };
              })(__childScope20);
            }
          });
          const __frag16 = document.createDocumentFragment();
          const __c17 = [];
          const __e18 = document.createElement("tr");
          __c17.push(bindClass(__e18, "selected", () => $scope2.selected().includes($scope2.r.id)));
          __c17.push(on4(__e18, "click", function() {
            const __h = () => $scope2.toggleSelect($scope2.r.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __forAnchor19 = document.createComment("for");
          __e18.appendChild(__forAnchor19);
          __c17.push(__wireBoundary_b9(__forAnchor19));
          __frag16.appendChild(__e18);
          return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
            for (const c of __c17)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope15);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf21({
      anchor,
      test: () => {
        if ($scope.variant() === "data")
          return 0;
        if ($scope.variant() !== "native")
          return 1;
        return 2;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "wrap");
          __e3.setAttribute("part", "wrap");
          const __e4 = document.createElement("table");
          __e4.setAttribute("part", "table");
          __c2.push(bindAttr36(__e4, "aria-label", () => $scope.label() || null));
          const __e5 = document.createElement("thead");
          __e5.setAttribute("part", "thead");
          const __e6 = document.createElement("tr");
          const __forAnchor7 = document.createComment("for");
          __e6.appendChild(__forAnchor7);
          __c2.push(__wireBoundary_b2(__forAnchor7));
          __e5.appendChild(__e6);
          __e4.appendChild(__e5);
          const __e13 = document.createElement("tbody");
          __e13.setAttribute("part", "tbody");
          const __forAnchor14 = document.createComment("for");
          __e13.appendChild(__forAnchor14);
          __c2.push(__wireBoundary_b6(__forAnchor14));
          __e4.appendChild(__e13);
          __e3.appendChild(__e4);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag25 = document.createDocumentFragment();
          const __c26 = [];
          const __e27 = document.createElement("div");
          __e27.setAttribute("class", "wrap");
          __e27.setAttribute("part", "wrap");
          const __e28 = document.createElement("table");
          __e28.setAttribute("part", "table");
          __c26.push(bindAttr36(__e28, "aria-label", () => $scope.label() || null));
          const __e29 = document.createElement("thead");
          __e29.setAttribute("part", "thead");
          const __e30 = document.createElement("slot");
          __e30.setAttribute("name", "head");
          __e29.appendChild(__e30);
          __e28.appendChild(__e29);
          const __e31 = document.createElement("tbody");
          __e31.setAttribute("part", "tbody");
          const __e32 = document.createElement("slot");
          __e31.appendChild(__e32);
          __e28.appendChild(__e31);
          __e27.appendChild(__e28);
          const __e33 = document.createElement("div");
          __e33.setAttribute("class", "empty");
          __e33.setAttribute("part", "empty");
          __e33.setAttribute("role", "status");
          const __e34 = document.createElement("slot");
          __e34.setAttribute("name", "empty");
          __e33.appendChild(__e34);
          __e27.appendChild(__e33);
          __frag25.appendChild(__e27);
          return { nodes: Array.from(__frag25.childNodes), cleanup: () => {
            for (const c of __c26)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag35 = document.createDocumentFragment();
          const __c36 = [];
          const __e37 = document.createElement("div");
          __e37.setAttribute("class", "wrap");
          __e37.setAttribute("part", "wrap");
          const __e38 = document.createElement("slot");
          __e37.appendChild(__e38);
          const __e39 = document.createElement("div");
          __e39.setAttribute("class", "empty");
          __e39.setAttribute("part", "empty");
          __e39.setAttribute("role", "status");
          const __e40 = document.createElement("slot");
          __e40.setAttribute("name", "empty");
          __e39.appendChild(__e40);
          __e37.appendChild(__e39);
          __frag35.appendChild(__e37);
          return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
            for (const c of __c36)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets75(__root);
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

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.sui
import { adoptSuiSheets as adoptSuiSheets76, attachRef as attachRef45, component as component76, prop as prop72 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.controller.ts
import { controller as controller57, bindHtml as bindHtml5 } from "@ship/sui-runtime";
function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}
function coerce(v, fallback) {
  if (Array.isArray(v))
    return v;
  if (typeof v === "string" && v.trim()) {
    try {
      const parsed = JSON.parse(v);
      return Array.isArray(parsed) ? parsed : fallback;
    } catch {
      return fallback;
    }
  }
  return fallback;
}
function computePositions(nodes, width, height, layout) {
  const map = new Map;
  const missing = nodes.some((n) => typeof n.x !== "number" || typeof n.y !== "number");
  if (layout === "positioned" && !missing) {
    for (const n of nodes)
      map.set(n.id, { x: n.x, y: n.y });
    return map;
  }
  const cx = width * 0.22;
  const cy = height * 0.5;
  const centers = nodes.filter((n) => n.kind === "agent");
  const satellites = nodes.filter((n) => n.kind !== "agent");
  centers.forEach((n, i) => {
    const span = Math.max(1, centers.length);
    const y = cy + (i - (span - 1) / 2) * 80;
    map.set(n.id, { x: cx, y });
  });
  const byKind = new Map;
  for (const n of satellites) {
    const list = byKind.get(n.kind) ?? [];
    list.push(n);
    byKind.set(n.kind, list);
  }
  const kinds = Array.from(byKind.keys());
  kinds.forEach((kind, bandIdx) => {
    const list = byKind.get(kind);
    const bandX = width * (0.5 + (bandIdx + 1) / (kinds.length + 1) * 0.45);
    list.forEach((n, i) => {
      const y = height * (i + 1) / (list.length + 1);
      map.set(n.id, { x: bandX, y });
    });
  });
  return map;
}
function bezierPath(a, b) {
  const dx = Math.max(40, Math.abs(b.x - a.x) * 0.4);
  return `M ${a.x} ${a.y} C ${a.x + dx} ${a.y}, ${b.x - dx} ${b.y}, ${b.x} ${b.y}`;
}
function kindTemplateFor(host, kind) {
  const tpls = host.querySelectorAll('template[slot="node"]');
  for (const t of Array.from(tpls)) {
    if (t.dataset.kind === kind)
      return t;
  }
  return null;
}
function nodeBodyHtml(host, n) {
  const tpl = kindTemplateFor(host, n.kind);
  if (!tpl) {
    const sub = n.sub ? `<div class="node-sub">${esc(n.sub)}</div>` : "";
    return `<div class="node-h"><span class="tag">${esc(n.kind)}</span></div><div class="node-title">${esc(n.title ?? n.id)}</div>${sub}`;
  }
  return tpl.innerHTML.replace(/\$\{title\}/g, esc(n.title ?? "")).replace(/\$\{sub\}/g, esc(n.sub ?? "")).replace(/\$\{id\}/g, esc(n.id));
}
function renderGraphHtml(host, nodesRaw, edgesRaw, width, height, layout, selectedId) {
  const nodes = coerce(nodesRaw, []);
  const edges = coerce(edgesRaw, []);
  const pos = computePositions(nodes, width, height, layout);
  const edgePaths = edges.map((e) => {
    const a = pos.get(e.from);
    const b = pos.get(e.to);
    if (!a || !b)
      return "";
    const d = esc(bezierPath(a, b));
    const dashed = e.dashed ? " data-dashed" : "";
    return `<path d="${d}" data-kind="${esc(e.kind ?? "default")}"${dashed}></path>`;
  }).join("");
  const nodeDivs = nodes.map((n) => {
    const p = pos.get(n.id);
    if (!p)
      return "";
    const left = p.x / width * 100;
    const top = p.y / height * 100;
    const sel = selectedId && selectedId === n.id ? " data-selected" : "";
    const wStyle = n.w ? `--node-w: ${n.w}px;` : "";
    const label = esc(`${n.kind}: ${n.title ?? n.id}`);
    const body = nodeBodyHtml(host, n);
    return `<div class="node" part="node" data-kind="${esc(n.kind)}" data-id="${esc(n.id)}"${sel} tabindex="0" role="button" aria-label="${label}" style="left: ${left}%; top: ${top}%;${wStyle}">${body}</div>`;
  }).join("");
  return `<svg class="edges" part="edges" viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" aria-hidden="true">${edgePaths}</svg>` + nodeDivs;
}
var sui_graph_canvas_controller_default = controller57({
  mount(state, host) {
    const scope = state;
    const canvas = scope.canvasEl();
    if (!canvas)
      return;
    const inner = canvas.ownerDocument.createElement("div");
    canvas.appendChild(inner);
    const stopRender = bindHtml5(inner, () => {
      const nodesRaw = scope.nodes();
      const edgesRaw = scope.edges();
      const width = scope.width() || 1000;
      const height = scope.height() || 600;
      const layout = scope.layout() || "positioned";
      const selectedId = scope.selectedId();
      return renderGraphHtml(host, nodesRaw, edgesRaw, width, height, layout, selectedId);
    });
    let lastHoveredId = "";
    const onClick = (e) => {
      const node = e.target.closest(".node[data-id]");
      if (!node)
        return;
      const id = node.getAttribute("data-id") ?? "";
      const kind = node.getAttribute("data-kind") ?? "";
      host.setAttribute("selected-id", id);
      host.dispatchEvent(new CustomEvent("sui-node-select", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    const onKeyDown = (e) => {
      if (e.key !== "Enter" && e.key !== " ")
        return;
      const node = e.target.closest(".node[data-id]");
      if (!node)
        return;
      e.preventDefault();
      const id = node.getAttribute("data-id") ?? "";
      const kind = node.getAttribute("data-kind") ?? "";
      host.setAttribute("selected-id", id);
      host.dispatchEvent(new CustomEvent("sui-node-select", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    const onMouseOver = (e) => {
      const node = e.target.closest(".node[data-id]");
      if (!node) {
        lastHoveredId = "";
        return;
      }
      const id = node.getAttribute("data-id") ?? "";
      if (id === lastHoveredId)
        return;
      lastHoveredId = id;
      const kind = node.getAttribute("data-kind") ?? "";
      host.dispatchEvent(new CustomEvent("sui-node-hover", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    canvas.addEventListener("click", onClick);
    canvas.addEventListener("keydown", onKeyDown);
    canvas.addEventListener("mouseover", onMouseOver);
    return () => {
      stopRender();
      canvas.removeEventListener("click", onClick);
      canvas.removeEventListener("keydown", onKeyDown);
      canvas.removeEventListener("mouseover", onMouseOver);
    };
  }
});

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.sui
var sui_graph_canvas_default = component76("sui-graph-canvas", {
  props: {
    nodes: prop72.string({ default: "" }),
    edges: prop72.string({ default: "" }),
    width: prop72.number({ default: 1000, reflect: true }),
    height: prop72.number({ default: 600, reflect: true }),
    layout: prop72.union(["positioned", "radial"], { default: "positioned", reflect: true }),
    "selected-id": prop72.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    position: relative;
    width: 100%;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    /* Component knobs — consumers may override on the host. */
    --graph-aspect: 5 / 3;
    --node-w: 180px;
    --edge-default: var(--border);
    --edge-agent: var(--tone-primary-fg, var(--foreground));
    --edge-skill: var(--tone-success-fg);
    --edge-mcp: var(--tone-info-fg);
    --edge-rule: var(--tone-warning-fg);
    --edge-perm: var(--tone-destructive-fg);
    --edge-provider: var(--tone-accent-fg);
  }

  .canvas {
    position: relative;
    width: 100%;
    aspect-ratio: var(--graph-aspect, 5 / 3);
  }

  svg.edges {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  svg.edges path {
    fill: none;
    stroke: var(--edge-default);
    stroke-width: 1.5;
    opacity: 0.85;
  }
  svg.edges path[data-kind='agent']    { stroke: var(--edge-agent); }
  svg.edges path[data-kind='skill']    { stroke: var(--edge-skill); }
  svg.edges path[data-kind='mcp']      { stroke: var(--edge-mcp); }
  svg.edges path[data-kind='rule']     { stroke: var(--edge-rule); }
  svg.edges path[data-kind='perm']     { stroke: var(--edge-perm); }
  svg.edges path[data-kind='provider'] { stroke: var(--edge-provider); }
  svg.edges path[data-dashed] { stroke-dasharray: 4 3; }

  .node {
    position: absolute;
    transform: translate(-50%, -50%);
    width: var(--node-w, 180px);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-3);
    box-shadow: var(--shadow-xs);
    cursor: pointer;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    transition: box-shadow var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }
  .node:hover { box-shadow: var(--shadow-sm); }
  .node:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }
  .node[data-selected] {
    border-color: var(--ring);
    box-shadow: var(--shadow-sm);
  }
  .node[data-kind='agent']    { border-left: 3px solid var(--edge-agent); }
  .node[data-kind='skill']    { border-left: 3px solid var(--edge-skill); }
  .node[data-kind='mcp']      { border-left: 3px solid var(--edge-mcp); }
  .node[data-kind='rule']     { border-left: 3px solid var(--edge-rule); }
  .node[data-kind='perm']     { border-left: 3px solid var(--edge-perm); }
  .node[data-kind='provider'] { border-left: 3px solid var(--edge-provider); }

  .node-h {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .tag {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-semibold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .node-title {
    font-weight: var(--font-weight-semibold);
    color: var(--foreground);
  }
  .node-sub {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }

  ::slotted(template) { display: none; }

  @media (prefers-reduced-motion: reduce) {
    .node { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.nodes = props.nodes;
    $scope.edges = props.edges;
    $scope.width = props.width;
    $scope.height = props.height;
    $scope.layout = props.layout;
    $scope.selectedId = props.selectedId;
    let __r = null;
    if (sui_graph_canvas_controller_default && typeof sui_graph_canvas_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_graph_canvas_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets76(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "canvas");
    __e0.setAttribute("part", "canvas");
    attachRef45($scope, "canvasEl", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "node");
    __e1.setAttribute("hidden", "");
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
