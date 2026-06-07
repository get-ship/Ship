// packages/components/primitives/src/sui-badge/sui-badge.sui
import { adoptSuiSheets, component, prop } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_badge_default = component("sui-badge", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    tone: prop.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop.union(["solid", "soft", "outline"], { default: "soft", reflect: true }),
    size: prop.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    pill: prop.bool({ default: false, reflect: true })
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
    letter-spacing: var(--tracking-wide);
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
    border-color: color-mix(in oklch, var(--tone) 100%, var(--foreground) 6%);
    color: var(--tone-contrast);
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
    adoptSuiSheets(__root);
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

// packages/components/primitives/src/sui-button/sui-button.sui
import { adoptSuiSheets as adoptSuiSheets2, bindAttr, bindIf, bindSlotEmpty, component as component2, prop as prop2 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_button_default = component2("sui-button", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    tone: prop2.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop2.union(["solid", "soft", "outline", "ghost", "default"], { default: "default", reflect: true }),
    size: prop2.union(["xs", "sm", "md", "lg"], { default: "md", reflect: true }),
    type: prop2.union(["button", "submit", "reset"], { default: "button", reflect: true }),
    disabled: prop2.bool({ default: false, reflect: true }),
    loading: prop2.bool({ default: false, reflect: true }),
    label: prop2.string({ default: "" }),
    pressed: prop2.union(["true", "false", "mixed"], { default: "false", reflect: true }),
    expanded: prop2.string({ default: "", reflect: true })
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
    :host([size='xs']) { --btn-h: 22px; --btn-fs: 11px; --btn-px: 8px;  --btn-gap: 4px; }
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
      border-color: color-mix(in oklch, var(--tone) 100%, var(--foreground) 6%);
      color: var(--tone-contrast);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, var(--background) 8%);
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
    /* Default-tone ghost hover lightens because \`--tone\` resolves to
     * \`--foreground\` — 12% foreground reads heavy on light themes. */
    :host([variant='ghost']:not([tone])) button:hover:not(:disabled),
    :host([variant='ghost'][tone='default']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 8%, transparent);
    }

    /* Active press — feels clicked across all variants. */
    :host(:not([disabled]):not([loading])) button:active:not(:disabled) {
      transform: translateY(1px) scale(0.98);
    }
    @media (prefers-reduced-motion: reduce) {
      button { transition: none; }
      button:active { transform: none !important; }
    }

    /* Focus ring — consume the system token. Outline (not stacked
     * box-shadow) so \`forced-colors: active\` still draws a system
     * ring and the ring isn't clipped by neighbouring shadows. */
    button:focus-visible {
      outline: var(--focus-ring);
      outline-offset: var(--focus-ring-offset);
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
    const __wireBoundary_b7 = (anchor) => bindIf({
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
    adoptSuiSheets2(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __cleanups.push(bindAttr(__e0, "type", () => $scope.type()));
    __cleanups.push(bindAttr(__e0, "disabled", () => $scope.disabled() || $scope.loading()));
    __cleanups.push(bindAttr(__e0, "aria-busy", () => $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr(__e0, "aria-disabled", () => $scope.disabled() || $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr(__e0, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr(__e0, "aria-pressed", () => $scope.pressed()));
    __cleanups.push(bindAttr(__e0, "aria-expanded", () => $scope.expanded()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "content");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    const __e3 = document.createElement("slot");
    __cleanups.push(bindSlotEmpty(__e3, host, "data-icon-only"));
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

// packages/components/primitives/src/sui-progress/sui-progress.sui
import { adoptSuiSheets as adoptSuiSheets3, bindAttr as bindAttr2, component as component3, prop as prop3 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_progress_default = component3("sui-progress", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    value: prop3.number({ default: 0 }),
    max: prop3.number({ default: 100 }),
    tone: prop3.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop3.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    indeterminate: prop3.bool({ default: false, reflect: true }),
    label: prop3.string({ default: "" })
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
    adoptSuiSheets3(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "track");
    __e0.setAttribute("part", "track");
    __e0.setAttribute("role", "progressbar");
    __cleanups.push(bindAttr2(__e0, "aria-valuenow", () => $scope.indeterminate() ? null : $scope.value()));
    __e0.setAttribute("aria-valuemin", "0");
    __cleanups.push(bindAttr2(__e0, "aria-valuemax", () => $scope.max()));
    __cleanups.push(bindAttr2(__e0, "aria-busy", () => $scope.indeterminate() ? "true" : "false"));
    __cleanups.push(bindAttr2(__e0, "aria-label", () => $scope.label()));
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "fill");
    __e1.setAttribute("part", "fill");
    __cleanups.push(bindAttr2(__e1, "style", () => $scope.indeterminate() ? "" : "width: clamp(0%, " + $scope.value() / ($scope.max() || 1) * 100 + "%, 100%)"));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-separator/sui-separator.sui
import { adoptSuiSheets as adoptSuiSheets4, bindAttr as bindAttr3, component as component4, prop as prop4 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_separator_default = component4("sui-separator", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    orientation: prop4.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    variant: prop4.union(["solid", "dashed"], { default: "solid", reflect: true }),
    decorative: prop4.bool({ default: false, reflect: true })
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
    $scope.decorative = props.decorative;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets4(__root);
    const __e0 = document.createElement("hr");
    __e0.setAttribute("class", "sep");
    __e0.setAttribute("part", "sep");
    __cleanups.push(bindAttr3(__e0, "role", () => $scope.decorative() ? "none" : "separator"));
    __cleanups.push(bindAttr3(__e0, "aria-orientation", () => $scope.decorative() ? null : $scope.orientation()));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-stat/sui-stat.sui
import { adoptSuiSheets as adoptSuiSheets5, component as component5, prop as prop5 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-stat/sui-stat.controller.ts
import { controller, effect } from "@ship/sui-runtime";
var sui_stat_controller_default = controller({
  mount(state, host) {
    const scope = state;
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      }
    };
    const stopSync = effect(() => {
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

// packages/components/primitives/src/sui-stat/sui-stat.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_stat_default = component5("sui-stat", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    tone: prop5.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "default", reflect: true }),
    interactive: prop5.bool({ default: false, reflect: true }),
    size: prop5.union(["sm", "md", "lg"], { default: "md", reflect: true })
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
    letter-spacing: var(--tracking-wide);
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
    font-family: var(--heading-font);
    font-weight: var(--font-weight-bold);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    color: var(--tone-fg);
  }
  :host([size='sm']) .value { font-size: var(--text-xl, 20px); }
  :host([size='md']) .value { font-size: var(--text-3xl, 30px); }
  :host([size='lg']) .value { font-size: var(--text-4xl, 36px); }

  /* .hint scales alongside .value — keeps the two metadata strings
   * in proportion across the size scale. */
  :host([size='sm']) .hint { font-size: var(--text-xs); }
  :host([size='md']) .hint { font-size: var(--text-sm); }
  :host([size='lg']) .hint { font-size: var(--text-md); }

  .trend {
    display: inline-flex;
    align-items: center;
  }
  .hint {
    font-size: var(--text-xs);
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
    adoptSuiSheets5(__root);
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

// packages/components/primitives/src/sui-status-dot/sui-status-dot.sui
import { adoptSuiSheets as adoptSuiSheets6, bindAttr as bindAttr4, component as component6, prop as prop6 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_status_dot_default = component6("sui-status-dot", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    tone: prop6.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    pulse: prop6.bool({ default: false, reflect: true }),
    size: prop6.union(["sm", "md"], { default: "md", reflect: true }),
    label: prop6.string({ default: "" })
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
    adoptSuiSheets6(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "dot");
    __e0.setAttribute("part", "dot");
    __cleanups.push(bindAttr4(__e0, "role", () => $scope.label() ? "img" : null));
    __cleanups.push(bindAttr4(__e0, "aria-label", () => $scope.label() || null));
    __cleanups.push(bindAttr4(__e0, "aria-hidden", () => $scope.label() ? null : "true"));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-table/sui-table.sui
import { adoptSuiSheets as adoptSuiSheets7, bindAttr as bindAttr5, bindClass, bindFor, bindIf as bindIf2, bindText, component as component7, on, prop as prop7, signal as signal2 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-table/sui-table.controller.ts
import { controller as controller2, signal, derived, bindNativeEvent } from "@ship/sui-runtime";
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
var sui_table_controller_default = controller2({
  scope: () => {
    const rows = signal([]);
    const columns = signal([]);
    const sortKey = signal(null);
    const sortDir = signal("asc");
    const selected = signal([]);
    const sortedRows = derived(() => {
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
    const initRows = host.rows;
    if (initRows != null)
      state.rows.set(initRows);
    const initCols = host.columns;
    if (initCols != null)
      state.columns.set(initCols);
    Object.defineProperty(host, "rows", {
      configurable: true,
      get: () => state.rows(),
      set: (v) => state.rows.set(v)
    });
    Object.defineProperty(host, "columns", {
      configurable: true,
      get: () => state.columns(),
      set: (v) => state.columns.set(v)
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
          chev.style.cssText = "margin-inline-start:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";
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
    bindNativeEvent(host, "click", onHeaderClick);
    bindNativeEvent(host, "click", onRowClick);
    bindNativeEvent(host, "keydown", onHeaderKey);
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
    text-align: start;
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

// packages/components/primitives/src/sui-table/sui-table.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_table_default = component7("sui-table", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    variant: prop7.union(["slot", "native", "data"], { default: "slot", reflect: true }),
    density: prop7.union(["compact", "default", "comfortable"], { default: "default", reflect: true }),
    dense: prop7.bool({ default: false, reflect: true }),
    hoverable: prop7.bool({ default: true, reflect: true }),
    striped: prop7.bool({ default: false, reflect: true }),
    "sticky-header": prop7.bool({ default: false, reflect: true }),
    "sticky-first-col": prop7.bool({ default: false, reflect: true }),
    gutter: prop7.bool({ default: false, reflect: true }),
    label: prop7.string({ default: "" })
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
    const __wireBoundary_b2 = (anchor) => bindFor({
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
          __childScope8.__v_field = signal2(0);
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("th");
          __c10.push(on(__e11, "click", function() {
            const __h = () => $scope2.sortBy($scope2.c.field);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          __c10.push(bindClass(__e11, "sorted", () => (void $scope2.__v_field(), $scope2.sortKey() === $scope2.c.field)));
          const __t12 = document.createTextNode("");
          __e11.appendChild(__t12);
          __t12.textContent = $scope2.c.label;
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c2 of __c10)
              try {
                c2();
              } catch (e) {}
          }, update: (__new13, __newI14) => {
            const __old = $scope2.c;
            $scope2.c = __new13;
            $scope2.__i = __newI14;
            if (__old.label !== $scope2.c.label) {
              __t12.textContent = $scope2.c.label;
            }
            if (__old.field !== $scope2.c.field) {
              $scope2.__v_field.set((c2) => c2 + 1);
            }
          } };
        })(__childScope8);
      }
    });
    const __wireBoundary_b6 = (anchor) => bindFor({
      anchor,
      items: () => $scope.sortedRows(),
      key: (r, __i) => {
        const __child = Object.create($scope);
        __child.r = r;
        __child.__i = __i;
        return (($scope2) => r.id)(__child);
      },
      render: (r, __i) => {
        const __childScope17 = Object.create($scope);
        __childScope17.r = r;
        __childScope17.__i = __i;
        return (($scope2) => {
          __childScope17.__v_id = signal2(0);
          const __wireBoundary_b9 = (anchor2) => bindFor({
            anchor: anchor2,
            items: () => $scope2.columns(),
            key: (c, __i2) => {
              const __child = Object.create($scope2);
              __child.c = c;
              __child.__i = __i2;
              return (($scope3) => c.field)(__child);
            },
            render: (c, __i2) => {
              const __childScope22 = Object.create($scope2);
              __childScope22.c = c;
              __childScope22.__i = __i2;
              return (($scope3) => {
                const __frag23 = document.createDocumentFragment();
                const __c24 = [];
                const __e25 = document.createElement("td");
                const __t26 = document.createTextNode("");
                __e25.appendChild(__t26);
                __t26.textContent = $scope3.r[$scope3.c.field];
                __frag23.appendChild(__e25);
                return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
                  for (const c2 of __c24)
                    try {
                      c2();
                    } catch (e) {}
                }, update: (__new27, __newI28) => {
                  const __old = $scope3.c;
                  $scope3.c = __new27;
                  $scope3.__i = __newI28;
                  if (__old.field !== $scope3.c.field) {
                    __t26.textContent = $scope3.r[$scope3.c.field];
                  }
                } };
              })(__childScope22);
            }
          });
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("tr");
          __c19.push(bindClass(__e20, "selected", () => (void $scope2.__v_id(), $scope2.selected().includes($scope2.r.id))));
          __c19.push(on(__e20, "click", function() {
            const __h = () => $scope2.toggleSelect($scope2.r.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __forAnchor21 = document.createComment("for");
          __e20.appendChild(__forAnchor21);
          __c19.push(__wireBoundary_b9(__forAnchor21));
          __frag18.appendChild(__e20);
          return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
            for (const c of __c19)
              try {
                c();
              } catch (e) {}
          }, update: (__new29, __newI30) => {
            const __old = $scope2.r;
            $scope2.r = __new29;
            $scope2.__i = __newI30;
            if (__old.id !== $scope2.r.id) {
              $scope2.__v_id.set((c) => c + 1);
            }
          } };
        })(__childScope17);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf2({
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
          __c2.push(bindAttr5(__e4, "aria-label", () => $scope.label() || null));
          const __e5 = document.createElement("thead");
          __e5.setAttribute("part", "thead");
          const __e6 = document.createElement("tr");
          const __forAnchor7 = document.createComment("for");
          __e6.appendChild(__forAnchor7);
          __c2.push(__wireBoundary_b2(__forAnchor7));
          __e5.appendChild(__e6);
          __e4.appendChild(__e5);
          const __e15 = document.createElement("tbody");
          __e15.setAttribute("part", "tbody");
          const __forAnchor16 = document.createComment("for");
          __e15.appendChild(__forAnchor16);
          __c2.push(__wireBoundary_b6(__forAnchor16));
          __e4.appendChild(__e15);
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
          const __frag31 = document.createDocumentFragment();
          const __c32 = [];
          const __e33 = document.createElement("div");
          __e33.setAttribute("class", "wrap");
          __e33.setAttribute("part", "wrap");
          const __e34 = document.createElement("table");
          __e34.setAttribute("part", "table");
          __c32.push(bindAttr5(__e34, "aria-label", () => $scope.label() || null));
          const __e35 = document.createElement("thead");
          __e35.setAttribute("part", "thead");
          const __e36 = document.createElement("slot");
          __e36.setAttribute("name", "head");
          __e35.appendChild(__e36);
          __e34.appendChild(__e35);
          const __e37 = document.createElement("tbody");
          __e37.setAttribute("part", "tbody");
          const __e38 = document.createElement("slot");
          __e37.appendChild(__e38);
          __e34.appendChild(__e37);
          __e33.appendChild(__e34);
          const __e39 = document.createElement("div");
          __e39.setAttribute("class", "empty");
          __e39.setAttribute("part", "empty");
          __e39.setAttribute("role", "status");
          const __e40 = document.createElement("slot");
          __e40.setAttribute("name", "empty");
          __e39.appendChild(__e40);
          __e33.appendChild(__e39);
          __frag31.appendChild(__e33);
          return { nodes: Array.from(__frag31.childNodes), cleanup: () => {
            for (const c of __c32)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag41 = document.createDocumentFragment();
          const __c42 = [];
          const __e43 = document.createElement("div");
          __e43.setAttribute("class", "wrap");
          __e43.setAttribute("part", "wrap");
          const __e44 = document.createElement("slot");
          __e43.appendChild(__e44);
          const __e45 = document.createElement("div");
          __e45.setAttribute("class", "empty");
          __e45.setAttribute("part", "empty");
          __e45.setAttribute("role", "status");
          const __e46 = document.createElement("slot");
          __e46.setAttribute("name", "empty");
          __e45.appendChild(__e46);
          __e43.appendChild(__e45);
          __frag41.appendChild(__e43);
          return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
            for (const c of __c42)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets7(__root);
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

// packages/components/primitives/src/sui-toolbar/sui-toolbar.sui
import { adoptSuiSheets as adoptSuiSheets8, attachRef, component as component8, prop as prop8, signal as signal3 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toolbar/sui-toolbar.controller.ts
import { controller as controller3 } from "@ship/sui-runtime";
var sui_toolbar_controller_default = controller3({
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

// packages/components/primitives/src/sui-toolbar/sui-toolbar.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_toolbar_default = component8("sui-toolbar", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    justify: prop8.union(["start", "center", "end", "between", "split"], { default: "start", reflect: true }),
    align: prop8.union(["center", "start", "end", "baseline"], { default: "center", reflect: true }),
    gap: prop8.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop8.union(["default", "bordered"], { default: "default", reflect: true })
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
  /* \`split\` — implicit spacer pushes the first child to the left and
   * the rest to the right, without requiring an explicit
   * <span slot="spacer"></span> in the markup. */
  :host([justify='split']) { justify-content: flex-start; }
  :host([justify='split']) ::slotted(:first-child) { margin-right: auto; }
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
    $scope["defaultSlot"] = signal3(null);
    let __r = null;
    if (sui_toolbar_controller_default && typeof sui_toolbar_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toolbar_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets8(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "spacer");
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    attachRef($scope, "defaultSlot", __e1);
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

// packages/components/charts/src/sui-bar-chart/sui-bar-chart.sui
import { adoptSuiSheets as adoptSuiSheets9, bindAttr as bindAttr6, bindFor as bindFor2, bindIf as bindIf3, bindText as bindText2, component as component9, prop as prop9, signal as signal5 } from "@ship/sui-runtime";

// packages/components/charts/src/sui-bar-chart/sui-bar-chart.controller.ts
import { controller as controller4, effect as effect2, signal as signal4 } from "@ship/sui-runtime";

// packages/components/charts/src/chart-util.ts
var DEFAULT_TONE = "var(--foreground)";
var TONE_VARS = {
  default: DEFAULT_TONE,
  primary: "var(--primary)",
  info: "var(--info)",
  success: "var(--success)",
  warning: "var(--warning)",
  destructive: "var(--destructive)",
  accent: "var(--accent)",
  muted: "var(--muted-foreground)"
};
function toneColor(tone) {
  if (!tone)
    return DEFAULT_TONE;
  return TONE_VARS[tone] ?? DEFAULT_TONE;
}
function parseNums(s) {
  if (!s)
    return [];
  return s.split(",").map((p) => Number.parseFloat(p.trim())).filter((n) => Number.isFinite(n));
}
function parseLabels(s) {
  if (!s)
    return [];
  return s.split(",").map((p) => p.trim());
}
function linePath(data, w, h, pad = 2) {
  if (data.length === 0)
    return { d: "", points: [] };
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const innerH = h - pad * 2;
  const step = data.length === 1 ? 0 : w / (data.length - 1);
  const points = data.map((v, i) => {
    const x = data.length === 1 ? w / 2 : i * step;
    const y = pad + innerH - (v - min) / range * innerH;
    return [x, y];
  });
  const d = points.map(([x, y], i) => `${i === 0 ? "M" : "L"}${x.toFixed(2)},${y.toFixed(2)}`).join(" ");
  return { d, points };
}

// packages/components/charts/src/sui-bar-chart/sui-bar-chart.controller.ts
var sui_bar_chart_controller_default = controller4({
  scope: () => {
    const bars = signal4([]);
    const isEmpty = signal4(true);
    return { bars, isEmpty };
  },
  mount(state, _host) {
    const stop = effect2(() => {
      const nums = parseNums(state.data());
      if (nums.length === 0) {
        state.bars.set([]);
        state.isEmpty.set(true);
        return;
      }
      const labels = parseLabels(state.labels());
      const w = state.width();
      const h = state.height();
      const rawMax = state.max();
      const maxVal = rawMax > 0 ? rawMax : Math.max(...nums, 0);
      const safeMax = maxVal <= 0 ? 1 : maxVal;
      const color = toneColor(state.tone());
      const gap = 4;
      const radius = 3;
      const horizontal = state.orientation() === "horizontal";
      const n = nums.length;
      state.bars.set(nums.map((v, i) => {
        const ratio = Math.max(0, v) / safeMax;
        let x, y, barW, barH;
        if (horizontal) {
          barH = (h - gap * (n - 1)) / n;
          y = i * (barH + gap);
          barW = ratio * w;
          x = 0;
        } else {
          barW = (w - gap * (n - 1)) / n;
          x = i * (barW + gap);
          barH = ratio * h;
          y = h - barH;
        }
        return {
          id: `b${i}`,
          x,
          y,
          w: Math.max(0, barW),
          h: Math.max(0, barH),
          rx: radius,
          ry: radius,
          color,
          title: `${labels[i] ?? String(v)}: ${v}`
        };
      }));
      state.isEmpty.set(false);
    });
    return () => stop();
  }
});

// packages/components/charts/src/sui-bar-chart/sui-bar-chart.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_bar_chart_default = component9("sui-bar-chart", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    data: prop9.string({ default: "", reflect: true }),
    labels: prop9.string({ default: "" }),
    tone: prop9.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "primary", reflect: true }),
    orientation: prop9.union(["vertical", "horizontal"], { default: "vertical", reflect: true }),
    width: prop9.number({ default: 300 }),
    height: prop9.number({ default: 200 }),
    max: prop9.number({ default: 0 }),
    label: prop9.string({ default: "bar chart" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    width: 100%;
    height: 200px;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; width: 100%; height: 100%; overflow: hidden; }

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
    $scope.data = props.data;
    $scope.labels = props.labels;
    $scope.tone = props.tone;
    $scope.orientation = props.orientation;
    $scope.width = props.width;
    $scope.height = props.height;
    $scope.max = props.max;
    $scope.label = props.label;
    let __r = null;
    if (sui_bar_chart_controller_default && typeof sui_bar_chart_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_bar_chart_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b4 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.bars(),
      key: (b, __i) => {
        const __child = Object.create($scope);
        __child.b = b;
        __child.__i = __i;
        return (($scope2) => b.id)(__child);
      },
      render: (b, __i) => {
        const __childScope8 = Object.create($scope);
        __childScope8.b = b;
        __childScope8.__i = __i;
        return (($scope2) => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElementNS("http://www.w3.org/2000/svg", "rect");
          {
            const __av12 = $scope2.b.x;
            if (__av12 == null || __av12 === false || __av12 === "")
              __e11.removeAttribute("x");
            else
              __e11.setAttribute("x", __av12 === true ? "" : String(__av12));
          }
          {
            const __av13 = $scope2.b.y;
            if (__av13 == null || __av13 === false || __av13 === "")
              __e11.removeAttribute("y");
            else
              __e11.setAttribute("y", __av13 === true ? "" : String(__av13));
          }
          {
            const __av14 = $scope2.b.w;
            if (__av14 == null || __av14 === false || __av14 === "")
              __e11.removeAttribute("width");
            else
              __e11.setAttribute("width", __av14 === true ? "" : String(__av14));
          }
          {
            const __av15 = $scope2.b.h;
            if (__av15 == null || __av15 === false || __av15 === "")
              __e11.removeAttribute("height");
            else
              __e11.setAttribute("height", __av15 === true ? "" : String(__av15));
          }
          {
            const __av16 = $scope2.b.rx;
            if (__av16 == null || __av16 === false || __av16 === "")
              __e11.removeAttribute("rx");
            else
              __e11.setAttribute("rx", __av16 === true ? "" : String(__av16));
          }
          {
            const __av17 = $scope2.b.ry;
            if (__av17 == null || __av17 === false || __av17 === "")
              __e11.removeAttribute("ry");
            else
              __e11.setAttribute("ry", __av17 === true ? "" : String(__av17));
          }
          {
            const __av18 = $scope2.b.color;
            if (__av18 == null || __av18 === false || __av18 === "")
              __e11.removeAttribute("fill");
            else
              __e11.setAttribute("fill", __av18 === true ? "" : String(__av18));
          }
          const __e19 = document.createElementNS("http://www.w3.org/2000/svg", "title");
          const __t20 = document.createTextNode("");
          __e19.appendChild(__t20);
          __t20.textContent = $scope2.b.title;
          __e11.appendChild(__e19);
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10)
              try {
                c();
              } catch (e) {}
          }, update: (__new21, __newI22) => {
            const __old = $scope2.b;
            $scope2.b = __new21;
            $scope2.__i = __newI22;
            if (__old.x !== $scope2.b.x) {
              {
                const __av12 = $scope2.b.x;
                if (__av12 == null || __av12 === false || __av12 === "")
                  __e11.removeAttribute("x");
                else
                  __e11.setAttribute("x", __av12 === true ? "" : String(__av12));
              }
            }
            if (__old.y !== $scope2.b.y) {
              {
                const __av13 = $scope2.b.y;
                if (__av13 == null || __av13 === false || __av13 === "")
                  __e11.removeAttribute("y");
                else
                  __e11.setAttribute("y", __av13 === true ? "" : String(__av13));
              }
            }
            if (__old.w !== $scope2.b.w) {
              {
                const __av14 = $scope2.b.w;
                if (__av14 == null || __av14 === false || __av14 === "")
                  __e11.removeAttribute("width");
                else
                  __e11.setAttribute("width", __av14 === true ? "" : String(__av14));
              }
            }
            if (__old.h !== $scope2.b.h) {
              {
                const __av15 = $scope2.b.h;
                if (__av15 == null || __av15 === false || __av15 === "")
                  __e11.removeAttribute("height");
                else
                  __e11.setAttribute("height", __av15 === true ? "" : String(__av15));
              }
            }
            if (__old.rx !== $scope2.b.rx) {
              {
                const __av16 = $scope2.b.rx;
                if (__av16 == null || __av16 === false || __av16 === "")
                  __e11.removeAttribute("rx");
                else
                  __e11.setAttribute("rx", __av16 === true ? "" : String(__av16));
              }
            }
            if (__old.ry !== $scope2.b.ry) {
              {
                const __av17 = $scope2.b.ry;
                if (__av17 == null || __av17 === false || __av17 === "")
                  __e11.removeAttribute("ry");
                else
                  __e11.setAttribute("ry", __av17 === true ? "" : String(__av17));
              }
            }
            if (__old.color !== $scope2.b.color) {
              {
                const __av18 = $scope2.b.color;
                if (__av18 == null || __av18 === false || __av18 === "")
                  __e11.removeAttribute("fill");
                else
                  __e11.setAttribute("fill", __av18 === true ? "" : String(__av18));
              }
            }
            if (__old.title !== $scope2.b.title) {
              __t20.textContent = $scope2.b.title;
            }
          } };
        })(__childScope8);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.isEmpty())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __c2.push(bindAttr6(__e3, "viewBox", () => "0 0 " + $scope.width() + " " + $scope.height()));
          __e3.setAttribute("aria-hidden", "true");
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __c5.push(bindAttr6(__e6, "viewBox", () => "0 0 " + $scope.width() + " " + $scope.height()));
          __e6.setAttribute("preserveAspectRatio", "none");
          __e6.setAttribute("role", "img");
          __c5.push(bindAttr6(__e6, "aria-label", () => $scope.label()));
          const __forAnchor7 = document.createComment("for");
          __e6.appendChild(__forAnchor7);
          __c5.push(__wireBoundary_b4(__forAnchor7));
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets9(__root);
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

// packages/components/charts/src/sui-sparkline/sui-sparkline.sui
import { adoptSuiSheets as adoptSuiSheets10, bindAttr as bindAttr7, bindIf as bindIf4, component as component10, prop as prop10 } from "@ship/sui-runtime";

// packages/components/charts/src/sui-sparkline/sui-sparkline.controller.ts
import { controller as controller5, effect as effect3, signal as signal6 } from "@ship/sui-runtime";
var sui_sparkline_controller_default = controller5({
  scope: () => {
    const hasData = signal6(false);
    const strokePath = signal6("");
    const areaPath = signal6("");
    const lineColor = signal6("var(--primary)");
    return { hasData, strokePath, areaPath, lineColor };
  },
  mount(state, _host) {
    const stop = effect3(() => {
      const nums = parseNums(state.data());
      const w = state.width();
      const h = state.height();
      const color = toneColor(state.tone());
      state.lineColor.set(color);
      if (nums.length < 2) {
        state.hasData.set(false);
        state.strokePath.set("");
        state.areaPath.set("");
        return;
      }
      const { d, points } = linePath(nums, w, h, 2);
      state.strokePath.set(d);
      if (state.fill()) {
        const first = points[0];
        const last = points[points.length - 1];
        state.areaPath.set(`${d} L${last[0].toFixed(2)},${h} L${first[0].toFixed(2)},${h} Z`);
      } else {
        state.areaPath.set("");
      }
      state.hasData.set(true);
    });
    return () => stop();
  }
});

// packages/components/charts/src/sui-sparkline/sui-sparkline.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_sparkline_default = component10("sui-sparkline", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    data: prop10.string({ default: "", reflect: true }),
    tone: prop10.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "primary", reflect: true }),
    width: prop10.number({ default: 80 }),
    height: prop10.number({ default: 20 }),
    fill: prop10.bool({ default: false, reflect: true }),
    label: prop10.string({ default: "sparkline" })
  },
  shadow: true,
  styles: `
  :host {
    --sui-sparkline-w: auto;
    --sui-sparkline-h: 40px;
    display: inline-block;
    width: var(--sui-sparkline-w);
    height: var(--sui-sparkline-h);
    line-height: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; overflow: visible; }

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
    $scope.data = props.data;
    $scope.tone = props.tone;
    $scope.width = props.width;
    $scope.height = props.height;
    $scope.fill = props.fill;
    $scope.label = props.label;
    let __r = null;
    if (sui_sparkline_controller_default && typeof sui_sparkline_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_sparkline_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindIf4({
      anchor,
      test: () => {
        if ($scope.fill())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __c6.push(bindAttr7(__e7, "d", () => $scope.areaPath()));
          __c6.push(bindAttr7(__e7, "fill", () => $scope.lineColor()));
          __e7.setAttribute("fill-opacity", "0.18");
          __e7.setAttribute("stroke", "none");
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
    const __wireBoundary_b0 = (anchor) => bindIf4({
      anchor,
      test: () => {
        if ($scope.hasData())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __c2.push(bindAttr7(__e3, "viewBox", () => "0 0 " + $scope.width() + " " + $scope.height()));
          __c2.push(bindAttr7(__e3, "width", () => $scope.width()));
          __c2.push(bindAttr7(__e3, "height", () => $scope.height()));
          __e3.setAttribute("role", "img");
          __c2.push(bindAttr7(__e3, "aria-label", () => $scope.label()));
          const __ifAnchor4 = document.createComment("if");
          __e3.appendChild(__ifAnchor4);
          __c2.push(__wireBoundary_b5(__ifAnchor4));
          const __e8 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __c2.push(bindAttr7(__e8, "d", () => $scope.strokePath()));
          __e8.setAttribute("fill", "none");
          __c2.push(bindAttr7(__e8, "stroke", () => $scope.lineColor()));
          __e8.setAttribute("stroke-width", "1.5");
          __e8.setAttribute("stroke-linecap", "round");
          __e8.setAttribute("stroke-linejoin", "round");
          const __t9 = document.createTextNode(`
    `);
          __e8.appendChild(__t9);
          __e3.appendChild(__e8);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __c11.push(bindAttr7(__e12, "viewBox", () => "0 0 " + $scope.width() + " " + $scope.height()));
          __c11.push(bindAttr7(__e12, "width", () => $scope.width()));
          __c11.push(bindAttr7(__e12, "height", () => $scope.height()));
          __e12.setAttribute("aria-hidden", "true");
          const __t13 = document.createTextNode(`
  `);
          __e12.appendChild(__t13);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets10(__root);
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

// apps/shell/src/components/sui-brand-mark/sui-brand-mark.sui
import { adoptSuiSheets as adoptSuiSheets11, component as component11, prop as prop11 } from "@ship/sui-runtime";

// apps/shell/src/components/sui-brand-mark/sui-brand-mark.controller.ts
import { controller as controller6, effect as effect4 } from "@ship/sui-runtime";
var sui_brand_mark_controller_default = controller6({
  mount(state, host) {
    const scope = state;
    const stopAria = effect4(() => {
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
    return () => stopAria();
  }
});

// apps/shell/src/components/sui-brand-mark/sui-brand-mark.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_brand_mark_default = component11("sui-brand-mark", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    size: prop11.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop11.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    flex: none;
    --ship-mark-size: 28px;
    width: var(--ship-mark-size);
    height: var(--ship-mark-size);
  }
  :host([size='sm']) { --ship-mark-size: 22px; }
  :host([size='md']) { --ship-mark-size: 28px; }
  :host([size='lg']) { --ship-mark-size: 48px; }

  svg { width: 100%; height: 100%; display: block; }

  /* Color ramp tuned for both light and dark themes. The dark variants
     pull toward --foreground (theme-aware "ink") with TIGHT ratios so
     they read as a subtle shading step from --primary, not as crushed-
     black blobs that disappear against a dark --background (the
     original mix-with-black variants were invisible on console theme).
     Tint pulls toward --background for highlight pops. */
  .c-base            { fill: color-mix(in oklch, var(--primary) 14%, var(--background)); }
  .c-primary         { fill: var(--primary); }
  .c-primary-deep    { fill: color-mix(in oklch, var(--primary) 88%, var(--foreground)); }
  .c-primary-dark    { fill: color-mix(in oklch, var(--primary) 78%, var(--foreground)); }
  .c-primary-darker  { fill: color-mix(in oklch, var(--primary) 65%, var(--foreground)); }
  .c-primary-tint    { fill: color-mix(in oklch, var(--primary) 75%, var(--background)); }
  .c-highlight       { fill: color-mix(in oklch, var(--primary) 35%, var(--background)); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.size = props.size;
    $scope.label = props.label;
    let __r = null;
    if (sui_brand_mark_controller_default && typeof sui_brand_mark_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_brand_mark_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets11(__root);
    const __e0 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e0.setAttribute("part", "svg");
    __e0.setAttribute("viewBox", "-260.519 -293.795 520.039 579.462");
    __e0.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    __e0.setAttribute("aria-hidden", "true");
    const __e1 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e1.setAttribute("class", "c-base");
    __e1.setAttribute("d", "m 37.859,-202.58 c 68.55,-48.403 139.29,-75.913 221.53,-91.215 1.46,74.118 -9.19,146.6 -38.18,215.442 -2.16,5.131 -11.56,26.09 -14,29.148 -17.48,38.54 -53.08,81.206 -83.72,110.199 13.88,89.476 -30.2,148.768 -98.61,200.33 -10.82,8.155 -21.07,16.77 -32.41,24.343 l 7.04,-50.335 C 4.379,201.23 8.879,176.396 -10.501,145.908 c -12.74,5.519 -23.31,9.031 -36.414,13.192 -10.632,-7.89 -24.44,-20.832 -34.505,-29.889 a 1439 1439 0 0 1 -49.255,-45.534 c 1.927,-13.94 4.834,-25.498 8.097,-39.197 a 40.745 40.745 0 0 0 -1.527,-0.855 c -30.198,-16.31 -60.006,-6.739 -90.71,2.22 l -45.704,13.441 c 46.247,-86.958 90.417,-158.954 200.275,-158.821 28.716,-42.953 56.653,-72.657 98.103,-103.045 z");
    __e0.appendChild(__e1);
    const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e2.setAttribute("transform", "matrix(1,0,0,1.0001,-25.05,41.507)");
    const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e3.setAttribute("class", "c-primary");
    __e3.setAttribute("transform", "translate(-999.27,-470.824)");
    __e3.setAttribute("d", "m 1062.18,226.76 c 1.2,0.002 3.63,-0.316 4.38,0.392 4.35,4.108 5,10.407 5.95,15.994 0.18,1.002 -1.02,3.24 -1.05,4.64 -11.29,9.093 -26.56,21.181 -36.68,31.153 -52.703,51.928 -86.249,114.407 -107.509,184.621 l 16.468,15.508 c 11.237,-13.897 34.812,-39.538 43.623,-52.359 -0.399,-11.072 -0.674,-22.422 -1.46,-33.452 -3.151,-3.235 -7.781,-8.707 -6.803,-13.43 4.738,-22.864 42.361,-9.134 21.611,13.333 l 0.59,17.242 40.6,-48.181 -0.95,-27.952 c -16.4,-10.234 -14.6,-31.177 1.85,-34.513 a 19.507 19.507 0 0 1 14.86,3.125 c 12.84,8.794 8.65,22.6 -2.65,30.604 0.45,10.896 1.14,22.503 1.02,33.351 -7.8,11.868 -31.75,39.316 -41.62,50.897 -23.358,27.401 -47.256,57.555 -71.26,84.08 -6.322,-5.9 -14.397,-13.848 -21.124,-18.9 -1.396,6.973 -3.427,15.812 -4.441,22.704 7.523,7.854 15.379,14.749 23.461,21.99 A 14818 14818 0 0 1 1086,356.353 c -5.36,-16.486 -0.55,-32.5 17.41,-37.547 a 23.981 23.981 0 0 1 18.75,2.594 27.95 27.95 0 0 1 12.9,17.791 c 5.04,22.018 -12.99,33.488 -33.28,31.044 l -96.5,114.035 c -15.635,18.496 -32.333,39.433 -48.548,57.034 7.131,6.95 17.565,16.835 25.478,22.801 6.917,-2.36 12.44,-4.283 19.14,-7.213 l -21.275,-19.263 c 23.745,-29.2 49.905,-58.504 74.165,-87.449 8.7,-10.376 35.31,-42.817 44.28,-51.03 l 31.55,-2.036 c 3.52,-5 8.58,-11.838 15.07,-12.303 24.86,-1.782 27,34.163 3.86,35.685 -4.84,0.3 -9.59,-1.413 -13.13,-4.732 a 48.48 48.48 0 0 1 -4.2,-4.482 c -8.85,0.702 -17.71,1.296 -26.57,1.782 -9.56,10.282 -21.2,24.11 -30.24,35.088 5.82,-0.26 12.21,-0.402 17.95,-1.064 5.31,-6.283 14.72,-13.197 21.2,-5.675 13.06,15.151 -2.02,36.747 -20.62,18.918 -10.74,1.525 -20.73,2.496 -31.5,3.489 -17.04,19.001 -32.98,39.329 -49.7,58.642 -3.18,3.674 -6.27,7.443 -9.14,11.365 5.25,4.553 11.8,10.548 17.16,14.73 88.13,-43.546 156.57,-102.878 198.46,-193.501 3.07,1.032 17.52,11.392 15.95,15.87 -0.91,2.623 -4.08,5.022 -3.69,8.551 l 0.6,0.642 c -17.48,38.536 -53.08,81.198 -83.72,110.188 13.88,89.467 -30.2,148.754 -98.61,200.311 -10.82,8.154 -21.07,16.767 -32.41,24.34 l 7.04,-50.33 c 4.87,-34.099 9.37,-58.93 -10.01,-89.415 -12.74,5.518 -23.31,9.03 -36.414,13.191 -10.632,-7.889 -24.44,-20.83 -34.505,-29.886 a 1439 1439 0 0 1 -49.255,-45.53 c 1.927,-13.939 4.834,-25.495 8.097,-39.193 a 40.746 40.746 0 0 0 -1.527,-0.855 c -30.198,-16.309 -60.006,-6.738 -90.709,2.219 L 763.802,488.6 c 46.247,-86.95 90.417,-158.939 200.275,-158.806 28.716,-42.948 56.653,-72.649 98.103,-103.034 z");
    __e2.appendChild(__e3);
    __e0.appendChild(__e2);
    const __e4 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e4.setAttribute("class", "c-primary");
    __e4.setAttribute("d", "m 88.719,89.527 14.56,-11.796 c 2.25,56.243 -9.67,87.496 -48.38,129.286 a 329 329 0 0 1 -19.78,19.378 c -2.6,3.002 -10.42,9.652 -13.85,11.714 -1.51,-1.65 2.42,-17.721 2.76,-21.95 2.7,-32.617 0.99,-50.438 -14.53,-78.733 19.38,-9.554 34.52,-18.391 53.03,-29.643 7.99,-4.911 18.23,-12.737 26.19,-18.256 z");
    __e0.appendChild(__e4);
    const __e5 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e5.setAttribute("class", "c-primary-dark");
    __e5.setAttribute("d", "m 9.499,137.426 c 19.38,-9.554 34.52,-18.392 53.03,-29.643 0.96,42.87 -2.64,65.37 -24.68,103.808 a 698 698 0 0 1 -11.99,19.99 c -1.19,1.88 -3.91,5.107 -4.59,6.528 -1.51,-1.65 2.42,-17.722 2.76,-21.951 2.7,-32.617 0.99,-50.437 -14.53,-78.732 z");
    __e0.appendChild(__e5);
    const __e6 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e6.setAttribute("transform", "matrix(1,0,0,1.0001,69.33,152.063)");
    const __e7 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e7.setAttribute("class", "c-highlight");
    __e7.setAttribute("transform", "translate(-1093.651,-581.375)");
    __e7.setAttribute("d", "m 1113.04,518.844 14.56,-11.794 c 2.25,56.237 -9.67,87.487 -48.38,129.273 a 329 329 0 0 1 -19.78,19.376 c 0.9,-4.233 10.82,-13.49 14.2,-17.904 31.25,-40.914 40.09,-69.335 39.4,-118.951 z");
    __e6.appendChild(__e7);
    __e0.appendChild(__e6);
    const __e8 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e8.setAttribute("class", "c-primary");
    __e8.setAttribute("d", "m -204.725,4.205 a 291 291 0 0 1 13.498,-18.034 c 34.429,-42.493 64.378,-58.087 117.133,-63.643 l -8.936,17.08 c -4.557,9.533 -9.506,19.48 -13.617,29.166 -8.128,18.775 -14.033,34.139 -19.897,53.927 -28.648,-10.782 -45.445,-12.292 -76.066,-5.32 -6.304,1.435 -16.62,3.792 -22.488,6.112 l -0.83,-0.2 c -0.081,-3.945 8.667,-15.579 11.203,-19.088 z");
    __e0.appendChild(__e8);
    const __e9 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e9.setAttribute("class", "c-primary-dark");
    __e9.setAttribute("d", "m -96.647,-31.226 c -8.128,18.774 -14.033,34.138 -19.897,53.926 -28.648,-10.782 -45.445,-12.292 -76.066,-5.32 -6.304,1.436 -16.62,3.792 -22.488,6.112 l 2.936,-4.154 c 6.633,-3.36 11.2,-11.853 17.225,-16.142 a 190 190 0 0 1 98.29,-34.422 z");
    __e0.appendChild(__e9);
    const __e10 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e10.setAttribute("transform", "matrix(1,0,0,1.0001,-139.41,-35.83)");
    const __e11 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e11.setAttribute("class", "c-highlight");
    __e11.setAttribute("transform", "translate(-884.912,-393.497)");
    __e11.setAttribute("d", "m 819.596,433.528 a 291 291 0 0 1 13.498,-18.032 c 34.429,-42.489 64.378,-58.081 117.133,-63.637 l -8.936,17.079 c -50.71,5.731 -87.986,26.201 -120.622,66.196 z");
    __e10.appendChild(__e11);
    __e0.appendChild(__e10);
    const __e12 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e12.setAttribute("class", "c-primary-tint");
    __e12.setAttribute("d", "m -215.098,23.491 -0.83,-0.2 c -0.081,-3.944 8.667,-15.578 11.203,-19.088 l 1.073,1.606 c -2.673,4.155 -6.183,9.287 -8.51,13.528 z");
    __e0.appendChild(__e12);
    const __e13 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e13.setAttribute("class", "c-base");
    __e13.setAttribute("d", "m 81.819,-94.803 c 5.49,-1.875 11.47,0.986 13.46,6.438 1.99,5.451 -0.75,11.491 -6.16,13.59 -3.6,1.397 -7.67,0.74 -10.65,-1.72 a 10.661 10.661 0 0 1 -3.69,-10.137 10.655 10.655 0 0 1 7.04,-8.17 z m 38.82,61.923 a 6.933 6.933 0 0 1 9.91,7.067 6.922 6.922 0 0 1 -4.32,5.608 6.925 6.925 0 0 1 -8.9,-3.643 6.93 6.93 0 0 1 3.31,-9.033 z m -99.93,-85.997 c 3.61,-1.208 7.52,0.728 8.74,4.33 a 6.906 6.906 0 0 1 -4.29,8.76 c -3.62,1.248 -7.56,-0.684 -8.8,-4.31 a 6.915 6.915 0 0 1 4.35,-8.78 z m 60.29,137.067 a 5.34 5.34 0 0 1 1.19,10.604 5.337 5.337 0 0 1 -5.63,-7.476 5.33 5.33 0 0 1 4.44,-3.128 z m -113.073,-71.985 a 5.314 5.314 0 1 1 1.526,10.517 5.314 5.314 0 0 1 -1.526,-10.517 z");
    __e0.appendChild(__e13);
    const __e14 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e14.setAttribute("transform", "matrix(1,0,0,1.0001,148.689,-171.5)");
    const __e15 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e15.setAttribute("class", "c-primary-dark");
    __e15.setAttribute("transform", "translate(-1173.01,-257.837)");
    __e15.setAttribute("d", "m 1062.18,226.76 c 68.55,-48.398 139.29,-75.906 221.53,-91.206 1.46,74.11 -9.19,146.586 -38.18,215.42 -2.16,5.131 -11.56,26.087 -14,29.145 l -0.6,-0.642 c -0.39,-3.529 2.78,-5.928 3.69,-8.551 1.57,-4.478 -12.88,-14.838 -15.95,-15.87 9.19,-20.306 17.63,-40.995 23.1,-62.641 -39.02,-10.764 -65.86,-27.631 -88.86,-62.298 -5.69,-8.582 -7.87,-15.669 -12.57,-23.895 -26.77,13.597 -44.27,24.653 -68.88,41.564 0.03,-1.4 1.23,-3.638 1.05,-4.64 -0.95,-5.587 -1.6,-11.886 -5.95,-15.994 -0.75,-0.708 -3.18,-0.39 -4.38,-0.392 z");
    __e14.appendChild(__e15);
    __e0.appendChild(__e14);
    const __e16 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e16.setAttribute("transform", "matrix(1,0,0,1.0001,186.114,-210.793)");
    const __e17 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e17.setAttribute("class", "c-primary-darker");
    __e17.setAttribute("transform", "translate(-1210.436,-218.548)");
    __e17.setAttribute("d", "m 1250.09,165.429 c 3.14,-1.68 7.17,-1.998 10.64,-2.074 5.19,13.98 -9.06,92.084 -13.7,110.386 -20.39,-5.262 -39.85,-14.169 -55.66,-28.351 -5.3,-4.822 -10.58,-9.786 -14.7,-15.677 -7.5,-9.181 -13.69,-21.025 -17.65,-32.185 a 554 554 0 0 1 91.07,-32.099 z");
    __e16.appendChild(__e17);
    __e0.appendChild(__e16);
    const __e18 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e18.setAttribute("class", "c-primary-darker");
    __e18.setAttribute("d", "m 225.769,-263.917 c 3.14,-1.68 7.17,-1.998 10.64,-2.075 -22.93,24.45 -45.77,57.988 -69.36,82.044 -5.3,-4.823 -10.58,-9.787 -14.7,-15.679 -7.5,-9.182 -13.69,-21.027 -17.65,-32.188 a 554 554 0 0 1 91.07,-32.102 z");
    __e0.appendChild(__e18);
    const __e19 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e19.setAttribute("class", "c-base");
    __e19.setAttribute("d", "m 225.769,-263.917 1.03,1.002 c -3.97,3.339 -12.42,6.563 -17.22,9.074 -15.41,8.063 -49.29,22.405 -60.63,34.61 0.48,5.262 3.39,16.17 3.4,19.604 -7.5,-9.182 -13.69,-21.027 -17.65,-32.188 a 554 554 0 0 1 91.07,-32.102 z");
    __e0.appendChild(__e19);
    const __e20 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e20.setAttribute("class", "c-primary");
    const __e21 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e21.setAttribute("d", "M 82,-174 L 105,-155.5 L 82,-137 L 79,-140 L 95,-155.5 L 79,-171 Z");
    __e20.appendChild(__e21);
    const __e22 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e22.setAttribute("d", "M 73,-174 L 50,-155.5 L 73,-137 L 76,-140 L 60,-155.5 L 76,-171 Z");
    __e20.appendChild(__e22);
    __e0.appendChild(__e20);
    const __e23 = document.createElementNS("http://www.w3.org/2000/svg", "g");
    __e23.setAttribute("class", "c-primary");
    const __e24 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e24.setAttribute("d", "M 152,-112 C 155,-112 157,-110 157,-107 L 157,-97 C 157,-94 159,-92 162,-92 C 159,-92 157,-90 157,-87 L 157,-77 C 157,-74 155,-72 152,-72 L 154,-74 C 155,-74 155.5,-75 155.5,-77 L 155.5,-88 C 155.5,-90 157,-91.5 159,-92 C 157,-92.5 155.5,-94 155.5,-96 L 155.5,-107 C 155.5,-109 155,-110 154,-110 Z");
    __e23.appendChild(__e24);
    const __e25 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e25.setAttribute("d", "M 170,-112 C 167,-112 165,-110 165,-107 L 165,-97 C 165,-94 163,-92 160,-92 C 163,-92 165,-90 165,-87 L 165,-77 C 165,-74 167,-72 170,-72 L 168,-74 C 167,-74 166.5,-75 166.5,-77 L 166.5,-88 C 166.5,-90 165,-91.5 163,-92 C 165,-92.5 166.5,-94 166.5,-96 L 166.5,-107 C 166.5,-109 167,-110 168,-110 Z");
    __e23.appendChild(__e25);
    __e0.appendChild(__e23);
    const __e26 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e26.setAttribute("class", "c-primary-deep");
    __e26.setAttribute("d", "m 135.529,-158 a 9.328 9.328 0 0 1 9.36,1.356 9.357 9.357 0 0 1 3.37,8.843 9.346 9.346 0 0 1 -6.09,7.241 c -4.78,1.69 -10.03,-0.742 -11.84,-5.478 -1.8,-4.737 0.51,-10.046 5.2,-11.963 z");
    __e0.appendChild(__e26);
    const __e27 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e27.setAttribute("class", "c-primary-deep");
    __e27.setAttribute("d", "m 117.009,-133.024 c 4.84,-0.51 9.19,2.958 9.79,7.791 0.59,4.834 -2.81,9.248 -7.63,9.92 a 8.922 8.922 0 0 1 -8.33,-3.432 8.922 8.922 0 0 1 6.17,-14.279 z");
    __e0.appendChild(__e27);
    const __e28 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e28.setAttribute("class", "c-primary-deep");
    __e28.setAttribute("d", "m 41.779,54.134 a 8.72 8.72 0 0 1 8.75,1.647 8.72 8.72 0 0 1 -3.37,14.938 c -4.51,1.29 -9.24,-1.229 -10.69,-5.695 -1.44,-4.468 0.9,-9.28 5.31,-10.89 z");
    __e0.appendChild(__e28);
    const __e29 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e29.setAttribute("class", "c-primary-deep");
    __e29.setAttribute("d", "m -11.151,-86.701 c 4.32,-1.506 9.07,0.693 10.71,4.971 a 8.49 8.49 0 0 1 -13.19,9.709 8.508 8.508 0 0 1 -3.11,-8.072 8.504 8.504 0 0 1 5.59,-6.608 z");
    __e0.appendChild(__e29);
    const __e30 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e30.setAttribute("class", "c-primary-deep");
    __e30.setAttribute("d", "m -224.315,190.455 c 14.097,-49.323 43.106,-96.44 102.002,-82.843 -27.216,6.952 -41.695,15.62 -51.954,42.984 8.138,-5.982 13.406,-8.387 22.563,-12.091 8.419,-2.171 12.696,-2.931 21.66,-3.392 l -12.891,13.261 c -13.335,20.04 -17.858,41.781 -28.162,64.39 23.13,-13.78 44.692,-20.476 63.835,-39.292 2.957,-4.012 6.304,-8.354 9.024,-12.484 0.11,7.582 0.309,14.019 -0.484,21.567 -1.812,10.794 -3.69,15.559 -8.655,25.363 23.383,-14.25 33.134,-28.943 34.707,-56.823 13.89,28.535 3.752,60.864 -18.62,81.56 -19.355,17.908 -46.063,31.637 -71.364,38.842 18.049,-15.242 26.402,-28.775 32.663,-51.253 -17.925,15.746 -36.221,23.825 -56.863,35.179 -9.92,5.455 -20.796,12.759 -30.127,19.25 l -9.494,7.363 c -1.947,-0.907 -2.89,-1.816 -4.57,-3.172 a 591 591 0 0 1 8.149,-15.386 c 0.617,-1.964 2.833,-6.26 3.84,-8.2 13.502,-26.024 18.813,-57.96 35.429,-82.17 -15.771,6.96 -20.384,10.53 -32.988,22.655 -3.151,-2.913 -3.874,-3.32 -7.7,-5.308 z");
    __e0.appendChild(__e30);
    const __e31 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e31.setAttribute("class", "c-primary-darker");
    __e31.setAttribute("d", "m -222.896,263.475 c -0.296,3.12 -0.461,3.893 0.277,6.912 2.193,2.503 2.673,2.728 5.638,4.284 l -9.494,7.362 c -1.947,-0.907 -2.89,-1.816 -4.57,-3.172 a 591 591 0 0 1 8.149,-15.386 z m 115.634,-90.004 c 2.957,-4.012 6.304,-8.354 9.024,-12.484 0.11,7.582 0.309,14.019 -0.484,21.567 -2.405,-5.637 -3.294,-5.651 -8.54,-9.083 z m -22.781,-38.359 -12.892,13.261 c -0.237,-3.359 2.087,-3.089 1.022,-7.813 -2.32,-1.989 -6.46,-1.788 -9.791,-2.056 8.419,-2.17 12.696,-2.93 21.66,-3.392 z m -94.272,55.342 c 3.826,1.988 4.549,2.395 7.7,5.307 -4.177,4.952 -8.564,10.801 -12.595,15.955 1.47,-8.103 2.597,-13.375 4.895,-21.262 z");
    __e0.appendChild(__e31);
    const __e32 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e32.setAttribute("class", "c-primary-deep");
    __e32.setAttribute("d", "m -231.044,278.86 c 1.68,1.357 2.622,2.266 4.569,3.173 -4.234,3.743 -8.522,7.204 -12.903,10.762 2.63,-4.73 5.41,-9.378 8.334,-13.934 z");
    __e0.appendChild(__e32);
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

// apps/shell/src/components/sui-theme-picker/sui-theme-picker.sui
import { adoptSuiSheets as adoptSuiSheets12, attachRef as attachRef2, bindAttr as bindAttr8, bindFor as bindFor3, bindText as bindText3, component as component12, effect as effect6, on as on2, prop as prop12, signal as signal8 } from "@ship/sui-runtime";

// apps/shell/src/components/sui-theme-picker/sui-theme-picker.controller.ts
import { controller as controller7, signal as signal7, derived as derived2, effect as effect5, ref, chrome } from "@ship/sui-runtime";

// packages/components/ui-components/src/tokens/themes.ts
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

// apps/shell/src/components/sui-theme-picker/sui-theme-picker.controller.ts
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
var sui_theme_picker_controller_default = controller7({
  scope: (init) => {
    const initAny = init;
    const activeTheme = signal7(readStoredTheme());
    const panelOpen = signal7(false);
    const groups = signal7(GROUPS.map((g) => ({
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
    const triggerBtn = ref(null);
    const panelEl = ref(null);
    function openCustomize() {
      internals.panel?.hidePopover?.();
      chrome.navigate("/theme-studio");
    }
    return {
      activeTheme,
      panelOpen,
      groups,
      dotColor,
      triggerLabel,
      togglePanel,
      selectTheme,
      openCustomize,
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
    const stopPanelSync = effect5(() => {
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
    const stopKeyBind = effect5(() => {
      const panel = scope.panelEl();
      if (!panel)
        return;
      panel.addEventListener("keydown", onPanelKeydown);
      return () => panel.removeEventListener("keydown", onPanelKeydown);
    });
    const stopFocusOnOpen = effect5(() => {
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

// apps/shell/src/components/sui-theme-picker/sui-theme-picker.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"842d083f","dirty":true,"builtAt":"2026-05-20T20:01:25Z"} */
var sui_theme_picker_default = component12("sui-theme-picker", {
  _compilerVersion: { cargo: "0.1.0", git: "842d083f", dirty: true, builtAt: "2026-05-20T20:01:25Z" },
  props: {
    placement: prop12.union(["top", "top-end", "bottom", "bottom-end", "left", "left-end", "right", "right-end"], { default: "bottom-end", reflect: true }),
    "label-style": prop12.union(["text", "compact"], { default: "text", reflect: true }),
    persist: prop12.bool({ default: true, reflect: true })
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
    padding: var(--space-1) var(--space-2);
    font: inherit;
    font-size: var(--text-base, 14px);
    color: var(--foreground);
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
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary) 50%, transparent);
  }

  .dot {
    width: 10px; height: 10px;
    border-radius: 999px;
    background: var(--primary);
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--foreground) 15%, transparent);
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
  :host([label-style='compact']) .caret { display: none; }

  .caret {
    width: 10px; height: 6px;
    flex: 0 0 auto;
    color: var(--muted-foreground);
  }

  /* Native popover panel. Reset the UA's default centering and let
     the controller place us via getBoundingClientRect on the trigger. */
  .panel {
    position: fixed;
    inset: auto;
    margin: 0;
    border: 0;
    padding: var(--space-1);
    background: var(--popover, var(--card));
    color: var(--popover-foreground, var(--card-foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg);
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

  /* Customize footer — bottom-pinned link that routes to /theme-studio for
     per-token editing. Visually separated from the theme cards by a divider. */
  .customize {
    appearance: none;
    background: transparent;
    border: 0;
    border-top: 1px solid var(--border);
    margin-top: var(--space-1);
    padding: 10px var(--space-3);
    cursor: pointer;
    text-align: left;
    font: inherit;
    font-size: 12.5px;
    color: var(--foreground);
    display: flex;
    align-items: center;
    gap: var(--space-2);
    border-radius: 0 0 var(--radius-md, 6px) var(--radius-md, 6px);
    transition: background var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .customize:hover {
    background: var(--hover-bg, color-mix(in oklch, currentColor 6%, transparent));
  }
  .customize:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 2px color-mix(in oklch, var(--primary) 50%, transparent);
  }
  .customize-meta {
    flex: 1;
    color: var(--muted-foreground);
    font-size: 11px;
  }
  .customize-chev {
    color: var(--muted-foreground);
    flex: 0 0 auto;
  }

  .group { display: flex; flex-direction: column; gap: 2px; }
  .group-h {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted-foreground);
    padding: 0 var(--space-2) 2px;
    opacity: 0.7;
  }
  .group-rows { display: flex; flex-direction: column; gap: var(--space-1); }

  /* Card — full-width entry per theme. Aria-selected drives a tinted
     background + radio fill. */
  .card {
    appearance: none;
    background: transparent;
    border: 0;
    border-radius: var(--radius-lg);
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
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary) 50%, transparent);
  }
  .card[aria-selected='true'] {
    background: color-mix(in oklch, var(--primary) 10%, transparent);
  }

  .row {
    display: flex; align-items: center; gap: var(--space-2);
    padding: 0 var(--space-1);
  }
  .name {
    flex: 1;
    font-family: var(--font-body, system-ui);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .radio {
    width: 12px; height: 12px;
    border-radius: 999px;
    border: 1.5px solid color-mix(in oklch, var(--foreground) 40%, transparent);
    background: transparent;
    position: relative;
    flex-shrink: 0;
  }
  .card[aria-selected='true'] .radio { border-color: var(--primary); }
  .card[aria-selected='true'] .radio::after {
    content: '';
    position: absolute;
    inset: 2px;
    border-radius: 999px;
    background: var(--primary);
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
    z-index: var(--z-elevated);
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
    padding: 10px var(--space-3) 11px;
    background: var(--card);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-card-rest, var(--shadow-sm));
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
    font-size: var(--text-base); line-height: 1.05;
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
    padding: 1px var(--space-1);
    border-radius: 3px;
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
    color: var(--muted-foreground);
  }
  .tc-controls { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
  .tc-cta {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground);
  }
  .tc-ghost {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: var(--space-1) var(--space-2);
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
    $scope["panelEl"] = signal8(null);
    $scope["triggerBtn"] = signal8(null);
    let __r = null;
    if (sui_theme_picker_controller_default && typeof sui_theme_picker_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_theme_picker_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b6 = (anchor) => bindFor3({
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
          const __wireBoundary_b9 = (anchor2) => bindFor3({
            anchor: anchor2,
            items: () => $scope2.g.themes,
            key: (t, __i2) => {
              const __child = Object.create($scope2);
              __child.t = t;
              __child.__i = __i2;
              return (($scope3) => t.id)(__child);
            },
            render: (t, __i2) => {
              const __childScope18 = Object.create($scope2);
              __childScope18.t = t;
              __childScope18.__i = __i2;
              return (($scope3) => {
                __childScope18.__v_id = signal8(0);
                const __frag19 = document.createDocumentFragment();
                const __c20 = [];
                const __e21 = document.createElement("button");
                __e21.setAttribute("class", "card");
                __e21.setAttribute("role", "option");
                __e21.setAttribute("type", "button");
                {
                  const __av22 = $scope3.t.id;
                  if (__av22 == null || __av22 === false || __av22 === "")
                    __e21.removeAttribute("data-theme-card");
                  else
                    __e21.setAttribute("data-theme-card", __av22 === true ? "" : String(__av22));
                }
                {
                  const __av23 = $scope3.t.label;
                  if (__av23 == null || __av23 === false || __av23 === "")
                    __e21.removeAttribute("aria-label");
                  else
                    __e21.setAttribute("aria-label", __av23 === true ? "" : String(__av23));
                }
                __c20.push(bindAttr8(__e21, "aria-selected", () => (void $scope3.__v_id(), $scope3.activeTheme() === $scope3.t.id ? "true" : "false")));
                __c20.push(bindAttr8(__e21, "tabindex", () => (void $scope3.__v_id(), $scope3.activeTheme() === $scope3.t.id ? "0" : "-1")));
                __c20.push(on2(__e21, "click", function() {
                  const __h = () => $scope3.selectTheme($scope3.t.id);
                  return typeof __h === "function" ? __h.bind(undefined) : () => {};
                }(), { modifiers: [] }));
                const __e24 = document.createElement("span");
                __e24.setAttribute("class", "tc-bg");
                {
                  const __av25 = $scope3.t.id;
                  if (__av25 == null || __av25 === false || __av25 === "")
                    __e24.removeAttribute("data-theme-preview");
                  else
                    __e24.setAttribute("data-theme-preview", __av25 === true ? "" : String(__av25));
                }
                __e24.setAttribute("aria-hidden", "true");
                const __e26 = document.createElement("span");
                __e26.setAttribute("class", "tc-overlay");
                __e24.appendChild(__e26);
                const __e27 = document.createElement("span");
                __e27.setAttribute("class", "tc-card");
                const __e28 = document.createElement("span");
                __e28.setAttribute("class", "tc-card-head");
                const __e29 = document.createElement("span");
                __e29.setAttribute("class", "tc-eyebrow");
                const __t30 = document.createTextNode("THEME");
                __e29.appendChild(__t30);
                __e28.appendChild(__e29);
                const __e31 = document.createElement("span");
                __e31.setAttribute("class", "tc-heading");
                const __t32 = document.createTextNode("");
                __e31.appendChild(__t32);
                __t32.textContent = $scope3.t.label;
                __e28.appendChild(__e31);
                __e27.appendChild(__e28);
                const __e33 = document.createElement("span");
                __e33.setAttribute("class", "tc-typo");
                const __e34 = document.createElement("span");
                __e34.setAttribute("class", "tc-body");
                const __t35 = document.createTextNode("The quick brown fox");
                __e34.appendChild(__t35);
                __e33.appendChild(__e34);
                const __e36 = document.createElement("span");
                __e36.setAttribute("class", "tc-mono");
                const __t37 = document.createTextNode("");
                __e36.appendChild(__t37);
                __t37.textContent = "{ build: ok }";
                __e33.appendChild(__e36);
                __e27.appendChild(__e33);
                const __e38 = document.createElement("span");
                __e38.setAttribute("class", "tc-controls");
                const __e39 = document.createElement("span");
                __e39.setAttribute("class", "tc-cta");
                const __t40 = document.createTextNode("Action");
                __e39.appendChild(__t40);
                __e38.appendChild(__e39);
                const __e41 = document.createElement("span");
                __e41.setAttribute("class", "tc-ghost");
                const __t42 = document.createTextNode("Cancel");
                __e41.appendChild(__t42);
                __e38.appendChild(__e41);
                const __e43 = document.createElement("span");
                __e43.setAttribute("class", "tc-chip tc-chip-accent");
                const __t44 = document.createTextNode("New");
                __e43.appendChild(__t44);
                __e38.appendChild(__e43);
                const __e45 = document.createElement("span");
                __e45.setAttribute("class", "tc-chip tc-chip-muted");
                const __t46 = document.createTextNode("Idle");
                __e45.appendChild(__t46);
                __e38.appendChild(__e45);
                __e27.appendChild(__e38);
                __e24.appendChild(__e27);
                __e21.appendChild(__e24);
                const __e47 = document.createElement("span");
                __e47.setAttribute("class", "row");
                const __e48 = document.createElement("span");
                __e48.setAttribute("class", "name");
                const __t49 = document.createTextNode("");
                __e48.appendChild(__t49);
                __t49.textContent = $scope3.t.label;
                __e47.appendChild(__e48);
                const __e50 = document.createElement("span");
                __e50.setAttribute("class", "radio");
                __e50.setAttribute("aria-hidden", "true");
                __e47.appendChild(__e50);
                __e21.appendChild(__e47);
                __frag19.appendChild(__e21);
                return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
                  for (const c of __c20)
                    try {
                      c();
                    } catch (e) {}
                }, update: (__new51, __newI52) => {
                  const __old = $scope3.t;
                  $scope3.t = __new51;
                  $scope3.__i = __newI52;
                  if (__old.id !== $scope3.t.id) {
                    {
                      const __av22 = $scope3.t.id;
                      if (__av22 == null || __av22 === false || __av22 === "")
                        __e21.removeAttribute("data-theme-card");
                      else
                        __e21.setAttribute("data-theme-card", __av22 === true ? "" : String(__av22));
                    }
                    {
                      const __av25 = $scope3.t.id;
                      if (__av25 == null || __av25 === false || __av25 === "")
                        __e24.removeAttribute("data-theme-preview");
                      else
                        __e24.setAttribute("data-theme-preview", __av25 === true ? "" : String(__av25));
                    }
                  }
                  if (__old.label !== $scope3.t.label) {
                    {
                      const __av23 = $scope3.t.label;
                      if (__av23 == null || __av23 === false || __av23 === "")
                        __e21.removeAttribute("aria-label");
                      else
                        __e21.setAttribute("aria-label", __av23 === true ? "" : String(__av23));
                    }
                    __t32.textContent = $scope3.t.label;
                    __t49.textContent = $scope3.t.label;
                  }
                  if (__old.id !== $scope3.t.id) {
                    $scope3.__v_id.set((c) => c + 1);
                  }
                } };
              })(__childScope18);
            }
          });
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("section");
          __e12.setAttribute("class", "group");
          {
            const __av13 = $scope2.g.label;
            if (__av13 == null || __av13 === false || __av13 === "")
              __e12.removeAttribute("aria-label");
            else
              __e12.setAttribute("aria-label", __av13 === true ? "" : String(__av13));
          }
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "group-h");
          const __t15 = document.createTextNode("");
          __e14.appendChild(__t15);
          __t15.textContent = $scope2.g.label;
          __e12.appendChild(__e14);
          const __e16 = document.createElement("div");
          __e16.setAttribute("class", "group-rows");
          const __forAnchor17 = document.createComment("for");
          __e16.appendChild(__forAnchor17);
          __c11.push(__wireBoundary_b9(__forAnchor17));
          __e12.appendChild(__e16);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          }, update: (__new53, __newI54) => {
            const __old = $scope2.g;
            $scope2.g = __new53;
            $scope2.__i = __newI54;
            if (__old.label !== $scope2.g.label) {
              {
                const __av13 = $scope2.g.label;
                if (__av13 == null || __av13 === false || __av13 === "")
                  __e12.removeAttribute("aria-label");
                else
                  __e12.setAttribute("aria-label", __av13 === true ? "" : String(__av13));
              }
              __t15.textContent = $scope2.g.label;
            }
          } };
        })(__childScope9);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets12(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("class", "trigger");
    __e0.setAttribute("part", "trigger");
    __e0.setAttribute("type", "button");
    attachRef2($scope, "triggerBtn", __e0);
    __e0.setAttribute("aria-label", "Choose theme");
    __e0.setAttribute("aria-haspopup", "dialog");
    __cleanups.push(bindAttr8(__e0, "aria-expanded", () => $scope.panelOpen() ? "true" : "false"));
    __cleanups.push(on2(__e0, "click", function() {
      const __h = () => $scope.togglePanel();
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElement("sui-icon");
    __e1.setAttribute("class", "tp-icon");
    __e1.setAttribute("part", "dot");
    __e1.setAttribute("name", "palette");
    __e1.setAttribute("size", "sm");
    __cleanups.push(effect6(() => {
      const __v = $scope.dotColor();
      if (__v == null)
        __e1.style.removeProperty("color");
      else
        __e1.style.setProperty("color", String(__v));
    }));
    __e1.setAttribute("aria-hidden", "true");
    __e0.appendChild(__e1);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "label");
    __e2.setAttribute("part", "label");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText3(__t3, () => $scope.triggerLabel()));
    __e0.appendChild(__e2);
    const __e4 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e4.setAttribute("class", "caret");
    __e4.setAttribute("viewBox", "0 0 10 6");
    __e4.setAttribute("aria-hidden", "true");
    const __e5 = document.createElementNS("http://www.w3.org/2000/svg", "path");
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
    attachRef2($scope, "panelEl", __e6);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "grid");
    __e7.setAttribute("part", "grid");
    __e7.setAttribute("role", "listbox");
    __e7.setAttribute("aria-label", "Theme");
    const __forAnchor8 = document.createComment("for");
    __e7.appendChild(__forAnchor8);
    __cleanups.push(__wireBoundary_b6(__forAnchor8));
    const __e55 = document.createElement("button");
    __e55.setAttribute("class", "customize");
    __e55.setAttribute("type", "button");
    __e55.setAttribute("data-theme-customize", "");
    __cleanups.push(on2(__e55, "click", function() {
      const __h = $scope.openCustomize;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e56 = document.createElement("span");
    const __t57 = document.createTextNode("Customize…");
    __e56.appendChild(__t57);
    __e55.appendChild(__e56);
    const __e58 = document.createElement("span");
    __e58.setAttribute("class", "customize-meta");
    const __t59 = document.createTextNode("Per-token editor");
    __e58.appendChild(__t59);
    __e55.appendChild(__e58);
    const __e60 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e60.setAttribute("class", "customize-chev");
    __e60.setAttribute("width", "10");
    __e60.setAttribute("height", "10");
    __e60.setAttribute("viewBox", "0 0 10 10");
    __e60.setAttribute("aria-hidden", "true");
    const __e61 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e61.setAttribute("d", "M3 1l4 4-4 4");
    __e61.setAttribute("fill", "none");
    __e61.setAttribute("stroke", "currentColor");
    __e61.setAttribute("stroke-width", "1.5");
    __e61.setAttribute("stroke-linecap", "round");
    __e61.setAttribute("stroke-linejoin", "round");
    __e60.appendChild(__e61);
    __e55.appendChild(__e60);
    __e7.appendChild(__e55);
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
