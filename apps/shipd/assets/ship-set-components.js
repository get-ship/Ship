// packages/ship-set-components/src/set-status-dot/set-status-dot.compiled.js
import { bindAttr, component, prop } from "@ship-set/runtime";
var __scopeEval = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E = [];
__E[0] = __scopeEval("label() ? 'img' : ''");
__E[1] = __scopeEval("label()");
__E[2] = __scopeEval("label() ? '' : 'true'");
var set_status_dot_compiled_default = component("set-status-dot", {
  props: {
    "tone": prop.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "success", reflect: true }),
    "pulse": prop.bool({ default: false, reflect: true }),
    "size": prop.union(["sm", "md"], { default: "md", reflect: true }),
    "label": prop.string({ default: "" })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      /* Per-host tone channel \xE2\x80\x94 `:host([tone=...])` rules below override\n       * --tone-fg so the dot color tracks the prop. Same pattern the Lit\n       * primitives use. */\n      --tone-fg: var(--success);\n    }\n    :host([tone='default']) { --tone-fg: var(--foreground); }\n    :host([tone='primary']) { --tone-fg: var(--primary); }\n    :host([tone='info'])    { --tone-fg: var(--info); }\n    :host([tone='success']) { --tone-fg: var(--success); }\n    :host([tone='warning']) { --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone-fg: var(--destructive); }\n    :host([tone='accent'])  { --tone-fg: var(--accent); }\n    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }\n\n    .dot {\n      display: inline-block;\n      width: 8px;\n      height: 8px;\n      border-radius: 50%;\n      background: var(--tone-fg);\n      box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);\n    }\n    :host([size='sm']) .dot {\n      width: 6px;\n      height: 6px;\n      box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent);\n    }\n\n    :host([pulse]) .dot { animation: set-status-dot-pulse 2.5s infinite; }\n    @keyframes set-status-dot-pulse {\n      0%, 100% { opacity: 1; transform: scale(1); }\n      50%      { opacity: 0.75; transform: scale(0.9); }\n    }\n\n    @media (prefers-reduced-motion: reduce) {\n      :host([pulse]) .dot { animation: none; }\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    $scope.pulse = props.pulse;
    $scope.size = props.size;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "dot");
    __e1.setAttribute("part", "dot");
    __cleanups.push(bindAttr(__e1, "role", () => __E[0]($scope)));
    __cleanups.push(bindAttr(__e1, "aria-label", () => __E[1]($scope)));
    __cleanups.push(bindAttr(__e1, "aria-hidden", () => __E[2]($scope)));
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n");
    __root.appendChild(__t2);
    return $scope;
  }
});

// packages/ship-set-components/src/set-spinner/set-spinner.compiled.js
import { bindAttr as bindAttr2, component as component2, prop as prop2 } from "@ship-set/runtime";
var __scopeEval2 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E2 = [];
__E2[0] = __scopeEval2("label() ? 'status' : ''");
__E2[1] = __scopeEval2("label()");
__E2[2] = __scopeEval2("label() ? '' : 'true'");
var set_spinner_compiled_default = component2("set-spinner", {
  props: {
    "size": prop2.union(["xs", "sm", "md", "lg"], { default: "md", reflect: true }),
    "tone": prop2.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "label": prop2.string({ default: "" })
  },
  shadow: true,
  styles: "\n    :host { display: inline-flex; vertical-align: middle; --tone-fg: currentColor; --sz: 16px; --bw: 2px; }\n    :host([tone='default']) { --tone-fg: var(--muted-foreground); }\n    :host([tone='primary']) { --tone-fg: var(--primary); }\n    :host([tone='info'])    { --tone-fg: var(--info); }\n    :host([tone='success']) { --tone-fg: var(--success); }\n    :host([tone='warning']) { --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone-fg: var(--destructive); }\n    :host([tone='accent'])  { --tone-fg: var(--accent); }\n    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }\n    :host([size='xs']) { --sz: 10px; --bw: 1.5px; }\n    :host([size='sm']) { --sz: 12px; --bw: 1.5px; }\n    :host([size='md']) { --sz: 16px; --bw: 2px; }\n    :host([size='lg']) { --sz: 24px; --bw: 2.5px; }\n    .ring {\n      width: var(--sz); height: var(--sz);\n      border: var(--bw) solid currentColor;\n      border-color: var(--tone-fg);\n      border-right-color: transparent;\n      border-radius: 50%;\n      animation: set-spinner-spin 0.7s linear infinite;\n      display: inline-block;\n    }\n    @keyframes set-spinner-spin { to { transform: rotate(360deg); } }\n    @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "ring");
    __e1.setAttribute("part", "ring");
    __cleanups.push(bindAttr2(__e1, "role", () => __E2[0]($scope)));
    __cleanups.push(bindAttr2(__e1, "aria-label", () => __E2[1]($scope)));
    __cleanups.push(bindAttr2(__e1, "aria-hidden", () => __E2[2]($scope)));
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n");
    __root.appendChild(__t2);
    return $scope;
  }
});

// packages/ship-set-components/src/set-badge/set-badge.compiled.js
import { component as component3, prop as prop3 } from "@ship-set/runtime";
var set_badge_compiled_default = component3("set-badge", {
  props: {
    "tone": prop3.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "variant": prop3.union(["solid", "soft", "outline"], { default: "soft", reflect: true }),
    "size": prop3.union(["sm", "md"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      --tone: var(--foreground);\n      --tone-fg: var(--foreground);\n      --tone-contrast: var(--background);\n    }\n    :host([tone='primary']) { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }\n    :host([tone='info'])    { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }\n    :host([tone='success']) { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }\n    :host([tone='warning']) { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }\n    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }\n    :host([tone='accent'])  { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }\n    :host([tone='muted'])   { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n\n    .root {\n      display: inline-flex;\n      align-items: center;\n      gap: 4px;\n      padding: 1px 8px;\n      font: inherit;\n      font-family: var(--font-body);\n      font-weight: var(--font-weight-medium, 500);\n      font-size: 11px;\n      line-height: 1.5;\n      letter-spacing: 0.01em;\n      border-radius: var(--radius-sm);\n      border: 1px solid transparent;\n      white-space: nowrap;\n    }\n    :host([size='md']) .root { font-size: 12px; padding: 2px 9px; }\n\n    :host([variant='solid']) .root { background: var(--tone); color: var(--tone-contrast); border-color: color-mix(in oklch, var(--tone) 100%, black 6%); }\n    :host([variant='soft'])  .root { background: color-mix(in oklch, var(--tone) 14%, transparent); color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 22%, transparent); }\n    :host([variant='outline']) .root { background: transparent; color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 50%, transparent); }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "root");
    __e1.setAttribute("part", "root");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n");
    __e1.appendChild(__t4);
    __root.appendChild(__e1);
    const __t5 = document.createTextNode("\n");
    __root.appendChild(__t5);
    return $scope;
  }
});

// packages/ship-set-components/src/set-chip/set-chip.compiled.js
import { component as component4, prop as prop4 } from "@ship-set/runtime";
var set_chip_compiled_default = component4("set-chip", {
  props: {
    "tone": prop4.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "variant": prop4.union(["solid", "soft", "outline", "ghost"], { default: "soft", reflect: true }),
    "size": prop4.union(["sm", "md"], { default: "sm", reflect: true })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      --tone: var(--foreground);\n      --tone-fg: var(--foreground);\n      --tone-contrast: var(--background);\n    }\n    :host([tone='primary']) { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }\n    :host([tone='info'])    { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }\n    :host([tone='success']) { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }\n    :host([tone='warning']) { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }\n    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }\n    :host([tone='accent'])  { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }\n    :host([tone='muted'])   { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n\n    .root {\n      display: inline-flex;\n      align-items: center;\n      gap: 6px;\n      padding: 3px 10px;\n      font-family: var(--font-body);\n      font-weight: var(--font-weight-medium, 500);\n      font-size: 12px;\n      line-height: 1.4;\n      border-radius: var(--radius-full);\n      border: 1px solid transparent;\n      white-space: nowrap;\n    }\n    :host([size='md']) .root { font-size: 13px; padding: 5px 12px; }\n\n    :host([variant='solid']) .root { background: var(--tone); color: var(--tone-contrast); border-color: color-mix(in oklch, var(--tone) 100%, black 6%); }\n    :host([variant='soft'])  .root { background: color-mix(in oklch, var(--tone) 14%, transparent); color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 22%, transparent); }\n    :host([variant='outline']) .root { background: transparent; color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 50%, transparent); }\n    :host([variant='ghost']) .root { background: transparent; color: var(--tone-fg); border-color: transparent; }\n\n    ::slotted([slot='leading']),\n    ::slotted([slot='trailing']) {\n      display: inline-flex;\n      align-items: center;\n      flex: 0 0 auto;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "root");
    __e1.setAttribute("part", "root");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e3.setAttribute("name", "leading");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n  ");
    __e1.appendChild(__t4);
    const __e5 = document.createElement("slot");
    __e1.appendChild(__e5);
    const __t6 = document.createTextNode("\n  ");
    __e1.appendChild(__t6);
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "trailing");
    __e1.appendChild(__e7);
    const __t8 = document.createTextNode("\n");
    __e1.appendChild(__t8);
    __root.appendChild(__e1);
    const __t9 = document.createTextNode("\n");
    __root.appendChild(__t9);
    return $scope;
  }
});

// packages/ship-set-components/src/set-card/set-card.compiled.js
import { component as component5, prop as prop5 } from "@ship-set/runtime";
var set_card_compiled_default = component5("set-card", {
  props: {
    "padding": prop5.union(["none", "sm", "md", "lg"], { default: "md", reflect: true }),
    "variant": prop5.union(["surface", "outline", "ghost"], { default: "surface", reflect: true }),
    "interactive": prop5.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: block; }\n    :host([hidden]) { display: none; }\n\n    .root {\n      display: flex;\n      flex-direction: column;\n      border-radius: var(--radius-lg);\n      background: var(--card);\n      border: 1px solid var(--border);\n      color: var(--foreground);\n      transition:\n        background-color var(--duration-fast) var(--ease-out),\n        border-color var(--duration-fast) var(--ease-out),\n        box-shadow var(--duration-fast) var(--ease-out);\n    }\n\n    :host([variant='outline'])  .root { background: transparent; }\n    :host([variant='ghost'])    .root { background: transparent; border-color: transparent; }\n\n    :host([padding='none']) .root { padding: 0; }\n    :host([padding='sm'])   .root { padding: 12px; }\n    :host([padding='md'])   .root { padding: 16px 18px; }\n    :host([padding='lg'])   .root { padding: 24px 28px; }\n\n    :host([interactive]) .root { cursor: pointer; }\n    :host([interactive]) .root:hover {\n      border-color: color-mix(in oklch, var(--border) 160%, transparent);\n      box-shadow: var(--shadow-sm);\n    }\n\n    .header { display: contents; }\n    .footer { display: contents; }\n\n    ::slotted([slot='header']) { display: block; }\n    ::slotted([slot='footer']) { display: block; margin-top: 12px; }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.padding = props.padding;
    $scope.variant = props.variant;
    $scope.interactive = props.interactive;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "root");
    __e1.setAttribute("part", "root");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e3.setAttribute("name", "header");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n  ");
    __e1.appendChild(__t4);
    const __e5 = document.createElement("slot");
    __e1.appendChild(__e5);
    const __t6 = document.createTextNode("\n  ");
    __e1.appendChild(__t6);
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "footer");
    __e1.appendChild(__e7);
    const __t8 = document.createTextNode("\n");
    __e1.appendChild(__t8);
    __root.appendChild(__e1);
    const __t9 = document.createTextNode("\n");
    __root.appendChild(__t9);
    return $scope;
  }
});

// packages/ship-set-components/src/set-link/set-link.compiled.js
import { bindAttr as bindAttr3, component as component6, prop as prop6 } from "@ship-set/runtime";
var __scopeEval3 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E3 = [];
__E3[0] = __scopeEval3("href()");
__E3[1] = __scopeEval3("target()");
__E3[2] = __scopeEval3("rel()");
var set_link_compiled_default = component6("set-link", {
  props: {
    "href": prop6.string({ default: "", reflect: true }),
    "tone": prop6.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "variant": prop6.union(["default", "underline", "subtle"], { default: "default", reflect: true }),
    "target": prop6.string({ default: "", reflect: true }),
    "rel": prop6.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline; --tone-fg: currentColor; }\n    :host([tone='primary']) { --tone-fg: var(--primary); }\n    :host([tone='info'])    { --tone-fg: var(--info); }\n    :host([tone='success']) { --tone-fg: var(--success); }\n    :host([tone='warning']) { --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone-fg: var(--destructive); }\n    :host([tone='accent'])  { --tone-fg: var(--accent); }\n    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }\n\n    a {\n      color: var(--tone-fg);\n      text-decoration: none;\n      cursor: pointer;\n      border-radius: 2px;\n      transition: color var(--duration-fast) var(--ease-out), text-decoration-color var(--duration-fast) var(--ease-out);\n    }\n    a:hover { text-decoration: underline; text-decoration-thickness: 1px; text-underline-offset: 2px; }\n    :host([variant='underline']) a { text-decoration: underline; text-decoration-thickness: 1px; text-underline-offset: 2px; text-decoration-color: color-mix(in oklch, var(--tone-fg) 40%, transparent); }\n    :host([variant='subtle']) a:hover { color: color-mix(in oklch, var(--tone-fg) 80%, var(--foreground)); }\n    a:focus-visible { outline: 2px solid var(--ring-color, currentColor); outline-offset: 2px; }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.href = props.href;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.target = props.target;
    $scope.rel = props.rel;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("a");
    __e1.setAttribute("part", "link");
    __cleanups.push(bindAttr3(__e1, "href", () => __E3[0]($scope)));
    __cleanups.push(bindAttr3(__e1, "target", () => __E3[1]($scope)));
    __cleanups.push(bindAttr3(__e1, "rel", () => __E3[2]($scope)));
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n");
    __e1.appendChild(__t4);
    __root.appendChild(__e1);
    const __t5 = document.createTextNode("\n");
    __root.appendChild(__t5);
    return $scope;
  }
});

// packages/ship-set-components/src/set-divider/set-divider.compiled.js
import { bindAttr as bindAttr4, component as component7, prop as prop7 } from "@ship-set/runtime";
var __scopeEval4 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E4 = [];
__E4[0] = __scopeEval4("orientation()");
var set_divider_compiled_default = component7("set-divider", {
  props: {
    "orientation": prop7.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    "tone": prop7.union(["default", "muted", "primary", "destructive"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: block; --tone-bg: var(--border); }\n    :host([tone='muted'])       { --tone-bg: var(--border-subtle, color-mix(in oklch, var(--border) 50%, transparent)); }\n    :host([tone='primary'])     { --tone-bg: color-mix(in oklch, var(--primary) 50%, var(--border)); }\n    :host([tone='destructive']) { --tone-bg: color-mix(in oklch, var(--destructive) 50%, var(--border)); }\n\n    .line {\n      background: var(--tone-bg);\n      border: 0;\n    }\n    :host([orientation='horizontal']) .line { width: 100%; height: 1px; margin: var(--space-2) 0; }\n    :host([orientation='vertical']) {\n      display: inline-block;\n      vertical-align: middle;\n      align-self: stretch;\n    }\n    :host([orientation='vertical']) .line { width: 1px; height: 1em; margin: 0 var(--space-2); display: inline-block; }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.orientation = props.orientation;
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("hr");
    __e1.setAttribute("class", "line");
    __e1.setAttribute("part", "line");
    __e1.setAttribute("role", "separator");
    __cleanups.push(bindAttr4(__e1, "aria-orientation", () => __E4[0]($scope)));
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n");
    __root.appendChild(__t2);
    return $scope;
  }
});

// packages/ship-set-components/src/set-skeleton/set-skeleton.compiled.js
import { component as component8, prop as prop8 } from "@ship-set/runtime";
var set_skeleton_compiled_default = component8("set-skeleton", {
  props: {
    "shape": prop8.union(["text", "line", "block", "circle"], { default: "text", reflect: true }),
    "size": prop8.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline-block; vertical-align: middle; }\n    :host([shape='block']) { display: block; }\n\n    .sk {\n      display: block;\n      background: linear-gradient(\n        90deg,\n        var(--muted) 0%,\n        color-mix(in oklch, var(--muted) 75%, var(--background)) 50%,\n        var(--muted) 100%\n      );\n      background-size: 200% 100%;\n      animation: set-skeleton-pulse 1.5s ease-in-out infinite;\n      border-radius: var(--radius-sm);\n    }\n    :host([shape='circle']) .sk { border-radius: 50%; }\n\n    /* Text/line \xE2\x80\x94 narrow heights with full-width default. */\n    :host([shape='text']) .sk,\n    :host([shape='line']) .sk { width: 100%; }\n    :host([size='sm'][shape='text']) .sk,\n    :host([size='sm'][shape='line']) .sk { height: 8px; }\n    :host([size='md'][shape='text']) .sk,\n    :host([size='md'][shape='line']) .sk { height: 12px; }\n    :host([size='lg'][shape='text']) .sk,\n    :host([size='lg'][shape='line']) .sk { height: 16px; }\n\n    /* Block \xE2\x80\x94 square-ish, scales with parent. */\n    :host([shape='block']) .sk { width: 100%; min-height: 80px; }\n\n    /* Circle \xE2\x80\x94 fixed sizes. */\n    :host([shape='circle'][size='sm']) .sk { width: 24px; height: 24px; }\n    :host([shape='circle'][size='md']) .sk { width: 32px; height: 32px; }\n    :host([shape='circle'][size='lg']) .sk { width: 48px; height: 48px; }\n\n    @keyframes set-skeleton-pulse {\n      0%   { background-position: 200% 0; }\n      100% { background-position: -200% 0; }\n    }\n    @media (prefers-reduced-motion: reduce) {\n      .sk { animation: none; }\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.shape = props.shape;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "sk");
    __e1.setAttribute("part", "sk");
    __e1.setAttribute("aria-hidden", "true");
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n");
    __root.appendChild(__t2);
    return $scope;
  }
});

// packages/ship-set-components/src/set-stat/set-stat.compiled.js
import { bindIf, bindText, component as component9, prop as prop9 } from "@ship-set/runtime";
var __scopeEval5 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E5 = [];
__E5[0] = __scopeEval5("label() !== ''");
__E5[1] = __scopeEval5("label()");
__E5[2] = __scopeEval5("value()");
__E5[3] = __scopeEval5("hint() !== ''");
__E5[4] = __scopeEval5("hint()");
var set_stat_compiled_default = component9("set-stat", {
  props: {
    "value": prop9.string({ default: "" }),
    "label": prop9.string({ default: "" }),
    "hint": prop9.string({ default: "" }),
    "tone": prop9.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "size": prop9.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline-flex; vertical-align: top; --tone-fg: var(--foreground); }\n    :host([tone='primary']) { --tone-fg: var(--primary); }\n    :host([tone='info'])    { --tone-fg: var(--info); }\n    :host([tone='success']) { --tone-fg: var(--success); }\n    :host([tone='warning']) { --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone-fg: var(--destructive); }\n    :host([tone='accent'])  { --tone-fg: var(--accent); }\n    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }\n\n    .root {\n      display: flex;\n      flex-direction: column;\n      gap: 2px;\n    }\n    .label {\n      font-size: 11px;\n      font-weight: var(--font-weight-medium, 500);\n      color: var(--muted-foreground);\n      letter-spacing: 0.04em;\n      text-transform: uppercase;\n    }\n    .value {\n      font-family: var(--font-display, var(--font-body));\n      font-weight: var(--font-weight-semibold, 600);\n      font-variant-numeric: tabular-nums;\n      color: var(--tone-fg);\n      line-height: 1.1;\n    }\n    :host([size='sm']) .value { font-size: 18px; }\n    :host([size='md']) .value { font-size: 24px; }\n    :host([size='lg']) .value { font-size: 36px; }\n    .hint { font-size: 11px; color: var(--muted-foreground); }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.value = props.value;
    $scope.label = props.label;
    $scope.hint = props.hint;
    $scope.tone = props.tone;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "root");
    __e1.setAttribute("part", "root");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __ifAnchor3 = document.createComment("if");
    __e1.appendChild(__ifAnchor3);
    __cleanups.push(bindIf({
      anchor: __ifAnchor3,
      test: () => {
        if (__E5[0]($scope)) return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __t6 = document.createTextNode("\n    ");
          __frag4.appendChild(__t6);
          const __e7 = document.createElement("span");
          __e7.setAttribute("class", "label");
          __e7.setAttribute("part", "label");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c5.push(bindText(__t8, () => __E5[1]($scope)));
          __frag4.appendChild(__e7);
          const __t9 = document.createTextNode("\n  ");
          __frag4.appendChild(__t9);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5) try {
              c();
            } catch (e) {
            }
          } };
        }
      ]
    }));
    const __t10 = document.createTextNode("\n  ");
    __e1.appendChild(__t10);
    const __e11 = document.createElement("span");
    __e11.setAttribute("class", "value");
    __e11.setAttribute("part", "value");
    const __t12 = document.createTextNode("");
    __e11.appendChild(__t12);
    __cleanups.push(bindText(__t12, () => __E5[2]($scope)));
    const __e13 = document.createElement("slot");
    __e11.appendChild(__e13);
    __e1.appendChild(__e11);
    const __t14 = document.createTextNode("\n  ");
    __e1.appendChild(__t14);
    const __ifAnchor15 = document.createComment("if");
    __e1.appendChild(__ifAnchor15);
    __cleanups.push(bindIf({
      anchor: __ifAnchor15,
      test: () => {
        if (__E5[3]($scope)) return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag16 = document.createDocumentFragment();
          const __c17 = [];
          const __t18 = document.createTextNode("\n    ");
          __frag16.appendChild(__t18);
          const __e19 = document.createElement("span");
          __e19.setAttribute("class", "hint");
          __e19.setAttribute("part", "hint");
          const __t20 = document.createTextNode("");
          __e19.appendChild(__t20);
          __c17.push(bindText(__t20, () => __E5[4]($scope)));
          __frag16.appendChild(__e19);
          const __t21 = document.createTextNode("\n  ");
          __frag16.appendChild(__t21);
          return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
            for (const c of __c17) try {
              c();
            } catch (e) {
            }
          } };
        }
      ]
    }));
    const __t22 = document.createTextNode("\n");
    __e1.appendChild(__t22);
    __root.appendChild(__e1);
    const __t23 = document.createTextNode("\n");
    __root.appendChild(__t23);
    return $scope;
  }
});

// packages/ship-set-components/src/set-mark/set-mark.compiled.js
import { component as component10, prop as prop10 } from "@ship-set/runtime";
var set_mark_compiled_default = component10("set-mark", {
  props: {
    "tone": prop10.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "warning", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline; --tone: var(--warning); --tone-fg: var(--warning); }\n    :host([tone='default'])     { --tone: var(--foreground); --tone-fg: var(--foreground); }\n    :host([tone='primary'])     { --tone: var(--primary); --tone-fg: var(--primary); }\n    :host([tone='info'])        { --tone: var(--info); --tone-fg: var(--info); }\n    :host([tone='success'])     { --tone: var(--success); --tone-fg: var(--success); }\n    :host([tone='warning'])     { --tone: var(--warning); --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }\n    :host([tone='accent'])      { --tone: var(--accent); --tone-fg: var(--accent); }\n    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }\n\n    mark {\n      background: color-mix(in oklch, var(--tone) 20%, transparent);\n      color: var(--tone-fg);\n      padding: 0 2px;\n      border-radius: 2px;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("mark");
    __e1.setAttribute("part", "mark");
    const __e2 = document.createElement("slot");
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
    const __t3 = document.createTextNode("\n");
    __root.appendChild(__t3);
    return $scope;
  }
});

// packages/ship-set-components/src/set-kbd/set-kbd.compiled.js
import { component as component11, prop as prop11 } from "@ship-set/runtime";
var set_kbd_compiled_default = component11("set-kbd", {
  props: {
    "size": prop11.union(["sm", "md"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline-flex; vertical-align: baseline; }\n\n    kbd {\n      display: inline-flex;\n      align-items: center;\n      gap: 2px;\n      padding: 2px 5px;\n      font-family: var(--font-mono);\n      font-weight: var(--font-weight-medium, 500);\n      font-size: 11px;\n      line-height: 1.2;\n      color: var(--foreground);\n      background: var(--muted);\n      border: 1px solid var(--border);\n      border-bottom-width: 2px;\n      border-radius: var(--radius-sm);\n      box-shadow: var(--shadow-xs);\n      white-space: nowrap;\n    }\n\n    :host([size='sm']) kbd { font-size: 10px; padding: 1px 4px; }\n    :host([size='md']) kbd { font-size: 11px; padding: 2px 5px; }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("kbd");
    __e1.setAttribute("part", "kbd");
    const __e2 = document.createElement("slot");
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
    const __t3 = document.createTextNode("\n");
    __root.appendChild(__t3);
    return $scope;
  }
});

// packages/ship-set-components/src/set-avatar/set-avatar.compiled.js
import { bindAttr as bindAttr5, bindIf as bindIf2, bindText as bindText2, component as component12, prop as prop12 } from "@ship-set/runtime";
var __scopeEval6 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E6 = [];
__E6[0] = __scopeEval6("alt() || name()");
__E6[1] = __scopeEval6("src() !== ''");
__E6[2] = __scopeEval6("src()");
__E6[3] = __scopeEval6("name() ? name().slice(0, 1).toUpperCase() : '?'");
var set_avatar_compiled_default = component12("set-avatar", {
  props: {
    "size": prop12.union(["xs", "sm", "md", "lg", "xl"], { default: "md", reflect: true }),
    "shape": prop12.union(["round", "square"], { default: "round", reflect: true }),
    "src": prop12.string({ default: "" }),
    "name": prop12.string({ default: "" }),
    "alt": prop12.string({ default: "" })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      --sz: 32px;\n      --fs: 13px;\n    }\n    :host([size='xs']) { --sz: 20px; --fs: 10px; }\n    :host([size='sm']) { --sz: 26px; --fs: 11px; }\n    :host([size='md']) { --sz: 32px; --fs: 13px; }\n    :host([size='lg']) { --sz: 40px; --fs: 15px; }\n    :host([size='xl']) { --sz: 56px; --fs: 22px; }\n\n    .root {\n      display: inline-flex;\n      align-items: center;\n      justify-content: center;\n      width: var(--sz);\n      height: var(--sz);\n      flex: none;\n      background: color-mix(in oklch, var(--accent) 30%, var(--muted));\n      color: var(--accent-foreground, var(--foreground));\n      font-family: var(--font-body);\n      font-weight: var(--font-weight-semibold, 600);\n      font-size: var(--fs);\n      letter-spacing: -0.01em;\n      overflow: hidden;\n      user-select: none;\n    }\n    :host([shape='round']) .root  { border-radius: 50%; }\n    :host([shape='square']) .root { border-radius: var(--radius-sm); }\n\n    img {\n      width: 100%;\n      height: 100%;\n      object-fit: cover;\n      display: block;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.size = props.size;
    $scope.shape = props.shape;
    $scope.src = props.src;
    $scope.name = props.name;
    $scope.alt = props.alt;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "root");
    __e1.setAttribute("part", "root");
    __cleanups.push(bindAttr5(__e1, "aria-label", () => __E6[0]($scope)));
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __ifAnchor3 = document.createComment("if");
    __e1.appendChild(__ifAnchor3);
    __cleanups.push(bindIf2({
      anchor: __ifAnchor3,
      test: () => {
        if (__E6[1]($scope)) return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __t6 = document.createTextNode("\n    ");
          __frag4.appendChild(__t6);
          const __e7 = document.createElement("img");
          __e7.setAttribute("part", "image");
          __c5.push(bindAttr5(__e7, "src", () => __E6[2]($scope)));
          __c5.push(bindAttr5(__e7, "alt", () => __E6[0]($scope)));
          __frag4.appendChild(__e7);
          const __t8 = document.createTextNode("\n  ");
          __frag4.appendChild(__t8);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5) try {
              c();
            } catch (e) {
            }
          } };
        },
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __t11 = document.createTextNode("\n    ");
          __frag9.appendChild(__t11);
          const __t12 = document.createTextNode("");
          __frag9.appendChild(__t12);
          __c10.push(bindText2(__t12, () => __E6[3]($scope)));
          const __t13 = document.createTextNode("\n  ");
          __frag9.appendChild(__t13);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10) try {
              c();
            } catch (e) {
            }
          } };
        }
      ]
    }));
    const __t14 = document.createTextNode("\n");
    __e1.appendChild(__t14);
    __root.appendChild(__e1);
    const __t15 = document.createTextNode("\n");
    __root.appendChild(__t15);
    return $scope;
  }
});

// packages/ship-set-components/src/set-progress/set-progress.compiled.js
import { bindAttr as bindAttr6, component as component13, prop as prop13 } from "@ship-set/runtime";
var __scopeEval7 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E7 = [];
__E7[0] = __scopeEval7("indeterminate() ? '' : value()");
__E7[1] = __scopeEval7("max()");
__E7[2] = __scopeEval7("label()");
__E7[3] = __scopeEval7("indeterminate() ? '' : ('width: clamp(0%, ' + (value() / (max() || 1) * 100) + '%, 100%)')");
var set_progress_compiled_default = component13("set-progress", {
  props: {
    "value": prop13.number({ default: 0 }),
    "max": prop13.number({ default: 100 }),
    "tone": prop13.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "primary", reflect: true }),
    "size": prop13.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    "indeterminate": prop13.bool({ default: false, reflect: true }),
    "label": prop13.string({ default: "" })
  },
  shadow: true,
  styles: "\n    :host { display: block; --tone: var(--primary); --h: 6px; }\n    :host([tone='default']) { --tone: var(--foreground); }\n    :host([tone='primary']) { --tone: var(--primary); }\n    :host([tone='info'])    { --tone: var(--info); }\n    :host([tone='success']) { --tone: var(--success); }\n    :host([tone='warning']) { --tone: var(--warning); }\n    :host([tone='destructive']) { --tone: var(--destructive); }\n    :host([tone='accent'])  { --tone: var(--accent); }\n    :host([size='sm']) { --h: 4px; }\n    :host([size='md']) { --h: 6px; }\n    :host([size='lg']) { --h: 10px; }\n\n    .track {\n      width: 100%;\n      height: var(--h);\n      background: var(--muted);\n      border-radius: var(--radius-full);\n      overflow: hidden;\n      position: relative;\n    }\n\n    .fill {\n      height: 100%;\n      background: var(--tone);\n      border-radius: var(--radius-full);\n      transition: width var(--duration-fast) var(--ease-out);\n    }\n\n    :host([indeterminate]) .fill {\n      width: 30%;\n      animation: set-progress-indet 1.4s ease-in-out infinite;\n    }\n    @keyframes set-progress-indet {\n      0%   { transform: translateX(-100%); }\n      50%  { transform: translateX(150%); }\n      100% { transform: translateX(350%); }\n    }\n    @media (prefers-reduced-motion: reduce) {\n      :host([indeterminate]) .fill { animation-duration: 3s; }\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.value = props.value;
    $scope.max = props.max;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.indeterminate = props.indeterminate;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "track");
    __e1.setAttribute("part", "track");
    __e1.setAttribute("role", "progressbar");
    __cleanups.push(bindAttr6(__e1, "aria-valuenow", () => __E7[0]($scope)));
    __e1.setAttribute("aria-valuemin", "0");
    __cleanups.push(bindAttr6(__e1, "aria-valuemax", () => __E7[1]($scope)));
    __cleanups.push(bindAttr6(__e1, "aria-label", () => __E7[2]($scope)));
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "fill");
    __e3.setAttribute("part", "fill");
    __cleanups.push(bindAttr6(__e3, "style", () => __E7[3]($scope)));
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n");
    __e1.appendChild(__t4);
    __root.appendChild(__e1);
    const __t5 = document.createTextNode("\n");
    __root.appendChild(__t5);
    return $scope;
  }
});

// packages/ship-set-components/src/set-button/set-button.compiled.js
import { bindAttr as bindAttr7, bindIf as bindIf3, component as component14, prop as prop14 } from "@ship-set/runtime";
var __scopeEval8 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E8 = [];
__E8[0] = __scopeEval8("type()");
__E8[1] = __scopeEval8("disabled() || loading()");
__E8[2] = __scopeEval8("loading() ? 'true' : 'false'");
__E8[3] = __scopeEval8("(disabled() || loading()) ? 'true' : 'false'");
__E8[4] = __scopeEval8("label()");
__E8[5] = __scopeEval8("pressed()");
__E8[6] = __scopeEval8("expanded()");
__E8[7] = __scopeEval8("loading()");
var set_button_compiled_default = component14("set-button", {
  props: {
    "tone": prop14.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    "variant": prop14.union(["solid", "soft", "outline", "ghost", "default"], { default: "default", reflect: true }),
    "size": prop14.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    "type": prop14.union(["button", "submit", "reset"], { default: "button", reflect: true }),
    "disabled": prop14.bool({ default: false, reflect: true }),
    "loading": prop14.bool({ default: false, reflect: true }),
    "label": prop14.string({ default: "" }),
    "pressed": prop14.string({ default: "", reflect: true }),
    "expanded": prop14.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      --tone: var(--foreground);\n      --tone-fg: var(--foreground);\n      --tone-contrast: var(--background);\n      --btn-h: 34px;\n      --btn-px: var(--space-3);\n      --btn-fs: 13px;\n      --btn-fw: var(--font-weight-medium, 500);\n      --btn-gap: 8px;\n    }\n    :host([hidden]) { display: none; }\n\n    /* Tone matrix */\n    :host([tone='primary'])     { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }\n    :host([tone='info'])        { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }\n    :host([tone='success'])     { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }\n    :host([tone='warning'])     { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }\n    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }\n    :host([tone='accent'])      { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }\n    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n\n    /* Size matrix */\n    :host([size='sm']) { --btn-h: 28px; --btn-fs: 12px; --btn-px: 10px; --btn-gap: 6px; }\n    :host([size='md']) { --btn-h: 34px; --btn-fs: 13px; --btn-px: 12px; --btn-gap: 8px; }\n    :host([size='lg']) { --btn-h: 40px; --btn-fs: 14px; --btn-px: 16px; --btn-gap: 8px; }\n\n    button {\n      display: inline-flex;\n      align-items: center;\n      justify-content: center;\n      gap: var(--btn-gap);\n      min-height: var(--btn-h);\n      padding: 0 var(--btn-px);\n      font-family: var(--font-body);\n      font-weight: var(--btn-fw);\n      font-size: var(--btn-fs);\n      line-height: 1;\n      letter-spacing: -0.005em;\n      border-radius: var(--radius-md);\n      border: 1px solid transparent;\n      background: transparent;\n      color: var(--foreground);\n      cursor: pointer;\n      user-select: none;\n      white-space: nowrap;\n      isolation: isolate;\n      position: relative;\n      transition:\n        background-color var(--duration-fast) var(--ease-out),\n        border-color var(--duration-fast) var(--ease-out),\n        box-shadow var(--duration-fast) var(--ease-out),\n        transform var(--duration-fast) var(--ease-out);\n    }\n\n    /* Default (no variant attr) \xE2\x80\x94 neutral bordered button. */\n    :host(:not([variant])) button,\n    :host([variant='default']) button {\n      background: var(--background);\n      border-color: var(--border);\n      color: var(--foreground);\n      box-shadow: var(--shadow-xs);\n    }\n    :host(:not([variant])) button:hover:not(:disabled),\n    :host([variant='default']) button:hover:not(:disabled) {\n      background: color-mix(in oklch, var(--muted) 70%, var(--background));\n      border-color: color-mix(in oklch, var(--border) 160%, transparent);\n      transform: translateY(-1px);\n    }\n\n    /* Solid */\n    :host([variant='solid']) button {\n      background: var(--tone);\n      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);\n      color: var(--tone-contrast);\n      box-shadow: var(--shadow-xs);\n    }\n    :host([variant='solid']) button:hover:not(:disabled) {\n      background: color-mix(in oklch, var(--tone) 92%, white 8%);\n      transform: translateY(-1px);\n      box-shadow: var(--shadow-sm);\n    }\n\n    /* Soft */\n    :host([variant='soft']) button {\n      background: color-mix(in oklch, var(--tone) 14%, transparent);\n      border-color: color-mix(in oklch, var(--tone) 22%, transparent);\n      color: var(--tone-fg);\n    }\n    :host([variant='soft']) button:hover:not(:disabled) {\n      background: color-mix(in oklch, var(--tone) 22%, transparent);\n    }\n\n    /* Outline */\n    :host([variant='outline']) button {\n      background: transparent;\n      border-color: color-mix(in oklch, var(--tone) 55%, transparent);\n      color: var(--tone-fg);\n    }\n    :host([variant='outline']) button:hover:not(:disabled) {\n      background: color-mix(in oklch, var(--tone) 10%, transparent);\n    }\n\n    /* Ghost */\n    :host([variant='ghost']) button {\n      background: transparent;\n      border-color: transparent;\n      color: var(--tone-fg);\n    }\n    :host([variant='ghost']) button:hover:not(:disabled) {\n      background: color-mix(in oklch, var(--tone) 12%, transparent);\n    }\n\n    /* Active press \xE2\x80\x94 feels clicked across all variants. */\n    :host(:not([disabled]):not([loading])) button:active:not(:disabled) {\n      transform: translateY(1px) scale(0.98);\n    }\n    @media (prefers-reduced-motion: reduce) {\n      button { transition: none; }\n      button:active { transform: none !important; }\n    }\n\n    /* Focus ring */\n    button:focus-visible {\n      outline: none;\n      box-shadow:\n        0 0 0 2px var(--background),\n        0 0 0 4px color-mix(in oklch, var(--tone) 55%, transparent);\n    }\n\n    /* Disabled */\n    button:disabled {\n      cursor: not-allowed;\n      color: color-mix(in oklch, var(--muted-foreground) 65%, transparent);\n      background: var(--muted);\n      border-color: color-mix(in oklch, var(--border) 80%, transparent);\n      box-shadow: none;\n      transform: none;\n      filter: saturate(0.6);\n    }\n    :host([variant='ghost']) button:disabled,\n    :host([variant='outline']) button:disabled {\n      background: transparent;\n    }\n\n    /* Loading state \xE2\x80\x94 fade the label, overlay a spinner. */\n    :host([loading]) button { cursor: progress; }\n    :host([loading]) .content { opacity: 0; }\n    .content {\n      display: inline-flex;\n      align-items: center;\n      gap: var(--btn-gap);\n      transition: opacity var(--duration-fast) var(--ease-out);\n    }\n    .spinner-wrap {\n      position: absolute;\n      inset: 0;\n      display: inline-flex;\n      align-items: center;\n      justify-content: center;\n      pointer-events: none;\n    }\n    .spinner {\n      width: 14px;\n      height: 14px;\n      border: 1.5px solid currentColor;\n      border-right-color: transparent;\n      border-radius: 50%;\n      animation: set-button-spin 0.7s linear infinite;\n    }\n    @keyframes set-button-spin { to { transform: rotate(360deg); } }\n\n    /* Icon-only \xE2\x80\x94 collapse padding to a square when default slot is empty. */\n    :host([data-icon-only]) button {\n      padding-inline: 0;\n      width: var(--btn-h);\n      min-width: var(--btn-h);\n    }\n\n    ::slotted([slot='leading']),\n    ::slotted([slot='trailing']) {\n      display: inline-flex;\n      align-items: center;\n      flex: 0 0 auto;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.type = props.type;
    $scope.disabled = props.disabled;
    $scope.loading = props.loading;
    $scope.label = props.label;
    $scope.pressed = props.pressed;
    $scope.expanded = props.expanded;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("button");
    __e1.setAttribute("part", "button");
    __cleanups.push(bindAttr7(__e1, "type", () => __E8[0]($scope)));
    __cleanups.push(bindAttr7(__e1, "disabled", () => __E8[1]($scope)));
    __cleanups.push(bindAttr7(__e1, "aria-busy", () => __E8[2]($scope)));
    __cleanups.push(bindAttr7(__e1, "aria-disabled", () => __E8[3]($scope)));
    __cleanups.push(bindAttr7(__e1, "aria-label", () => __E8[4]($scope)));
    __cleanups.push(bindAttr7(__e1, "aria-pressed", () => __E8[5]($scope)));
    __cleanups.push(bindAttr7(__e1, "aria-expanded", () => __E8[6]($scope)));
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n  ");
    __root.appendChild(__t2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "content");
    const __t4 = document.createTextNode("\n    ");
    __e3.appendChild(__t4);
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "leading");
    __e3.appendChild(__e5);
    const __t6 = document.createTextNode("\n    ");
    __e3.appendChild(__t6);
    const __e7 = document.createElement("slot");
    __e3.appendChild(__e7);
    const __t8 = document.createTextNode("\n    ");
    __e3.appendChild(__t8);
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "trailing");
    __e3.appendChild(__e9);
    const __t10 = document.createTextNode("\n  ");
    __e3.appendChild(__t10);
    __root.appendChild(__e3);
    const __t11 = document.createTextNode("\n  ");
    __root.appendChild(__t11);
    const __ifAnchor12 = document.createComment("if");
    __root.appendChild(__ifAnchor12);
    __cleanups.push(bindIf3({
      anchor: __ifAnchor12,
      test: () => {
        if (__E8[7]($scope)) return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __t15 = document.createTextNode("\n    ");
          __frag13.appendChild(__t15);
          const __e16 = document.createElement("span");
          __e16.setAttribute("class", "spinner-wrap");
          __e16.setAttribute("aria-hidden", "true");
          const __e17 = document.createElement("span");
          __e17.setAttribute("class", "spinner");
          __e16.appendChild(__e17);
          __frag13.appendChild(__e16);
          const __t18 = document.createTextNode("\n  ");
          __frag13.appendChild(__t18);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14) try {
              c();
            } catch (e) {
            }
          } };
        }
      ]
    }));
    const __t19 = document.createTextNode("\n");
    __root.appendChild(__t19);
    const __t20 = document.createTextNode("\n");
    __root.appendChild(__t20);
    return $scope;
  }
});

// packages/ship-set-components/src/set-tabs/set-tabs.compiled.js
import { component as component15, prop as prop15 } from "@ship-set/runtime";
var set_tabs_compiled_default = component15("set-tabs", {
  props: {
    "variant": prop15.union(["underline", "pill", "enclosed"], { default: "underline", reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: block; }\n    .strip {\n      display: flex;\n      align-items: center;\n      gap: 4px;\n      flex-wrap: wrap;\n      border-bottom: 1px solid var(--border);\n    }\n    :host([variant='pill']) .strip {\n      gap: 6px;\n      border-bottom: 0;\n      padding: 4px;\n      background: var(--muted);\n      border-radius: var(--radius-md);\n    }\n    :host([variant='enclosed']) .strip {\n      gap: 0;\n      border-bottom: 1px solid var(--border);\n    }\n\n    ::slotted(set-tab),\n    ::slotted([slot='tab']) {\n      flex: 0 0 auto;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.variant = props.variant;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "strip");
    __e1.setAttribute("part", "strip");
    __e1.setAttribute("role", "tablist");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n");
    __e1.appendChild(__t4);
    __root.appendChild(__e1);
    const __t5 = document.createTextNode("\n");
    __root.appendChild(__t5);
    return $scope;
  }
});

// packages/ship-set-components/src/set-tab/set-tab.compiled.js
import { bindAttr as bindAttr8, bindIf as bindIf4, bindText as bindText3, component as component16, prop as prop16 } from "@ship-set/runtime";
var __scopeEval9 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E9 = [];
__E9[0] = __scopeEval9("name()");
__E9[1] = __scopeEval9("active() ? 'true' : 'false'");
__E9[2] = __scopeEval9("disabled() ? 'true' : 'false'");
__E9[3] = __scopeEval9("disabled()");
__E9[4] = __scopeEval9("count() !== ''");
__E9[5] = __scopeEval9("count()");
var set_tab_compiled_default = component16("set-tab", {
  props: {
    "name": prop16.string({ default: "", reflect: true }),
    "active": prop16.bool({ default: false, reflect: true }),
    "count": prop16.string({ default: "" }),
    "disabled": prop16.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: "\n    :host { display: inline-flex; }\n    button {\n      display: inline-flex;\n      align-items: center;\n      gap: 6px;\n      padding: 8px 14px;\n      font: inherit;\n      font-family: var(--font-body);\n      font-size: 13px;\n      font-weight: var(--font-weight-medium, 500);\n      color: var(--muted-foreground);\n      background: transparent;\n      border: 0;\n      border-bottom: 2px solid transparent;\n      cursor: pointer;\n      border-radius: 0;\n      transition:\n        color var(--duration-fast) var(--ease-out),\n        border-color var(--duration-fast) var(--ease-out),\n        background-color var(--duration-fast) var(--ease-out);\n    }\n    button:hover:not(:disabled) {\n      color: var(--foreground);\n    }\n    :host([active]) button {\n      color: var(--foreground);\n      border-bottom-color: var(--primary);\n    }\n    button:disabled {\n      cursor: not-allowed;\n      opacity: 0.5;\n    }\n    .count {\n      font-size: 11px;\n      font-weight: var(--font-weight-medium, 500);\n      color: var(--muted-foreground);\n      padding: 0 6px;\n      border-radius: var(--radius-full);\n      background: color-mix(in oklch, var(--muted) 80%, transparent);\n    }\n    :host([active]) .count { color: var(--primary); background: color-mix(in oklch, var(--primary) 12%, transparent); }\n    button:focus-visible {\n      outline: 2px solid color-mix(in oklch, var(--primary) 55%, transparent);\n      outline-offset: -2px;\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.name = props.name;
    $scope.active = props.active;
    $scope.count = props.count;
    $scope.disabled = props.disabled;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("button");
    __e1.setAttribute("part", "button");
    __e1.setAttribute("type", "button");
    __e1.setAttribute("role", "tab");
    __cleanups.push(bindAttr8(__e1, "data-name", () => __E9[0]($scope)));
    __cleanups.push(bindAttr8(__e1, "aria-selected", () => __E9[1]($scope)));
    __cleanups.push(bindAttr8(__e1, "aria-disabled", () => __E9[2]($scope)));
    __cleanups.push(bindAttr8(__e1, "disabled", () => __E9[3]($scope)));
    __root.appendChild(__e1);
    const __t2 = document.createTextNode("\n  ");
    __root.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __root.appendChild(__e3);
    const __t4 = document.createTextNode("\n  ");
    __root.appendChild(__t4);
    const __ifAnchor5 = document.createComment("if");
    __root.appendChild(__ifAnchor5);
    __cleanups.push(bindIf4({
      anchor: __ifAnchor5,
      test: () => {
        if (__E9[4]($scope)) return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __t8 = document.createTextNode("\n    ");
          __frag6.appendChild(__t8);
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "count");
          __e9.setAttribute("aria-hidden", "true");
          const __t10 = document.createTextNode("");
          __e9.appendChild(__t10);
          __c7.push(bindText3(__t10, () => __E9[5]($scope)));
          __frag6.appendChild(__e9);
          const __t11 = document.createTextNode("\n  ");
          __frag6.appendChild(__t11);
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7) try {
              c();
            } catch (e) {
            }
          } };
        }
      ]
    }));
    const __t12 = document.createTextNode("\n");
    __root.appendChild(__t12);
    const __t13 = document.createTextNode("\n");
    __root.appendChild(__t13);
    return $scope;
  }
});

// packages/ship-set-components/src/set-empty-state/set-empty-state.compiled.js
import { component as component17, prop as prop17 } from "@ship-set/runtime";
var set_empty_state_compiled_default = component17("set-empty-state", {
  props: {
    "tone": prop17.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true }),
    "size": prop17.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    "variant": prop17.union(["default", "inset"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: "\n    :host {\n      display: flex;\n      flex-direction: column;\n      align-items: center;\n      justify-content: center;\n      gap: 12px;\n      padding: 32px 24px;\n      text-align: center;\n      color: var(--muted-foreground);\n      --tone-fg: var(--muted-foreground);\n    }\n    :host([tone='default'])     { --tone-fg: var(--foreground); }\n    :host([tone='primary'])     { --tone-fg: var(--primary); }\n    :host([tone='info'])        { --tone-fg: var(--info); }\n    :host([tone='success'])     { --tone-fg: var(--success); }\n    :host([tone='warning'])     { --tone-fg: var(--warning); }\n    :host([tone='destructive']) { --tone-fg: var(--destructive); }\n    :host([tone='accent'])      { --tone-fg: var(--accent); }\n    :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }\n\n    :host([variant='inset']) {\n      background: var(--card);\n      border: 1px dashed var(--border);\n      border-radius: var(--radius-lg);\n    }\n\n    :host([size='sm']) { padding: 16px; gap: 8px; }\n    :host([size='md']) { padding: 32px 24px; gap: 12px; }\n    :host([size='lg']) { padding: 56px 32px; gap: 16px; }\n\n    .icon-slot { color: var(--tone-fg); }\n    ::slotted([slot='icon']) {\n      color: var(--tone-fg);\n    }\n\n    .title {\n      color: var(--foreground);\n      font-family: var(--font-display, var(--font-body));\n      font-weight: var(--font-weight-semibold, 600);\n      font-size: 16px;\n      margin: 0;\n    }\n    :host([size='sm']) .title { font-size: 14px; }\n    :host([size='lg']) .title { font-size: 20px; }\n\n    .body {\n      max-width: 40ch;\n      font-size: 13px;\n      color: var(--muted-foreground);\n    }\n\n    .action-slot { display: inline-flex; gap: 8px; margin-top: 8px; }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.variant = props.variant;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "icon-slot");
    __e1.setAttribute("part", "icon");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
    const __t3 = document.createTextNode("\n");
    __root.appendChild(__t3);
    const __e4 = document.createElement("h3");
    __e4.setAttribute("class", "title");
    __e4.setAttribute("part", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
    const __t6 = document.createTextNode("\n");
    __root.appendChild(__t6);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "body");
    __e7.setAttribute("part", "body");
    const __e8 = document.createElement("slot");
    __e7.appendChild(__e8);
    __root.appendChild(__e7);
    const __t9 = document.createTextNode("\n");
    __root.appendChild(__t9);
    const __e10 = document.createElement("div");
    __e10.setAttribute("class", "action-slot");
    __e10.setAttribute("part", "action");
    const __e11 = document.createElement("slot");
    __e11.setAttribute("name", "action");
    __e10.appendChild(__e11);
    __root.appendChild(__e10);
    const __t12 = document.createTextNode("\n");
    __root.appendChild(__t12);
    return $scope;
  }
});

// packages/ship-set-components/src/set-input/set-input.compiled.js
import { bindAttr as bindAttr9, component as component18, prop as prop18 } from "@ship-set/runtime";
var __scopeEval10 = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E10 = [];
__E10[0] = __scopeEval10("type()");
__E10[1] = __scopeEval10("value()");
__E10[2] = __scopeEval10("placeholder()");
__E10[3] = __scopeEval10("name()");
__E10[4] = __scopeEval10("autocomplete()");
__E10[5] = __scopeEval10("disabled()");
__E10[6] = __scopeEval10("readonly()");
__E10[7] = __scopeEval10("label()");
__E10[8] = __scopeEval10("invalid() ? 'true' : 'false'");
var set_input_compiled_default = component18("set-input", {
  props: {
    "value": prop18.string({ default: "" }),
    "placeholder": prop18.string({ default: "" }),
    "type": prop18.union(["text", "search", "email", "url", "tel", "password", "number"], { default: "text", reflect: true }),
    "size": prop18.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    "tone": prop18.union(["default", "primary", "info", "success", "warning", "destructive"], { default: "default", reflect: true }),
    "disabled": prop18.bool({ default: false, reflect: true }),
    "readonly": prop18.bool({ default: false, reflect: true }),
    "invalid": prop18.bool({ default: false, reflect: true }),
    "name": prop18.string({ default: "" }),
    "label": prop18.string({ default: "" }),
    "autocomplete": prop18.string({ default: "" })
  },
  shadow: true,
  styles: "\n    :host {\n      display: inline-flex;\n      vertical-align: middle;\n      width: 100%;\n      --tone-border: var(--border);\n      --tone-ring: var(--primary);\n      --h: 34px;\n      --fs: 13px;\n      --px: 10px;\n    }\n    :host([tone='primary'])     { --tone-border: var(--primary); --tone-ring: var(--primary); }\n    :host([tone='success'])     { --tone-border: var(--success); --tone-ring: var(--success); }\n    :host([tone='warning'])     { --tone-border: var(--warning); --tone-ring: var(--warning); }\n    :host([tone='destructive']),\n    :host([invalid])            { --tone-border: var(--destructive); --tone-ring: var(--destructive); }\n    :host([size='sm']) { --h: 28px; --fs: 12px; --px: 8px; }\n    :host([size='md']) { --h: 34px; --fs: 13px; --px: 10px; }\n    :host([size='lg']) { --h: 40px; --fs: 14px; --px: 12px; }\n\n    .wrap {\n      display: inline-flex;\n      align-items: center;\n      gap: 6px;\n      width: 100%;\n      min-height: var(--h);\n      padding: 0 var(--px);\n      background: var(--background);\n      border: 1px solid var(--tone-border);\n      border-radius: var(--radius-md);\n      transition: border-color var(--duration-fast) var(--ease-out), box-shadow var(--duration-fast) var(--ease-out);\n    }\n    .wrap:focus-within {\n      border-color: var(--tone-ring);\n      box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-ring) 25%, transparent);\n    }\n    :host([disabled]) .wrap {\n      background: var(--muted);\n      cursor: not-allowed;\n      opacity: 0.65;\n    }\n\n    input {\n      flex: 1;\n      min-width: 0;\n      border: 0;\n      outline: 0;\n      background: transparent;\n      padding: 0;\n      font: inherit;\n      font-family: var(--font-body);\n      font-size: var(--fs);\n      color: var(--foreground);\n    }\n    input::placeholder { color: var(--muted-foreground); }\n    input:disabled { cursor: not-allowed; }\n\n    ::slotted([slot='leading']),\n    ::slotted([slot='trailing']) {\n      display: inline-flex;\n      align-items: center;\n      flex: 0 0 auto;\n      color: var(--muted-foreground);\n    }\n  ",
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.value = props.value;
    $scope.placeholder = props.placeholder;
    $scope.type = props.type;
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.disabled = props.disabled;
    $scope.readonly = props.readonly;
    $scope.invalid = props.invalid;
    $scope.name = props.name;
    $scope.label = props.label;
    $scope.autocomplete = props.autocomplete;
    const __root = host.shadowRoot ?? host;
    const __t0 = document.createTextNode("\n\n");
    __root.appendChild(__t0);
    const __e1 = document.createElement("label");
    __e1.setAttribute("class", "wrap");
    __e1.setAttribute("part", "wrap");
    const __t2 = document.createTextNode("\n  ");
    __e1.appendChild(__t2);
    const __e3 = document.createElement("slot");
    __e3.setAttribute("name", "leading");
    __e1.appendChild(__e3);
    const __t4 = document.createTextNode("\n  ");
    __e1.appendChild(__t4);
    const __e5 = document.createElement("input");
    __e5.setAttribute("part", "input");
    __cleanups.push(bindAttr9(__e5, "type", () => __E10[0]($scope)));
    __cleanups.push(bindAttr9(__e5, "value", () => __E10[1]($scope)));
    __cleanups.push(bindAttr9(__e5, "placeholder", () => __E10[2]($scope)));
    __cleanups.push(bindAttr9(__e5, "name", () => __E10[3]($scope)));
    __cleanups.push(bindAttr9(__e5, "autocomplete", () => __E10[4]($scope)));
    __cleanups.push(bindAttr9(__e5, "disabled", () => __E10[5]($scope)));
    __cleanups.push(bindAttr9(__e5, "readonly", () => __E10[6]($scope)));
    __cleanups.push(bindAttr9(__e5, "aria-label", () => __E10[7]($scope)));
    __cleanups.push(bindAttr9(__e5, "aria-invalid", () => __E10[8]($scope)));
    __e1.appendChild(__e5);
    const __t6 = document.createTextNode("\n  ");
    __e1.appendChild(__t6);
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "trailing");
    __e1.appendChild(__e7);
    const __t8 = document.createTextNode("\n");
    __e1.appendChild(__t8);
    __root.appendChild(__e1);
    const __t9 = document.createTextNode("\n");
    __root.appendChild(__t9);
    return $scope;
  }
});

// packages/ship-set-components/src/define.ts
function defineComponent(_template, _opts = {}) {
  if (typeof console !== "undefined") {
    console.warn(
      "[ship-set/components] defineComponent is deprecated; components are now AOT-compiled by ship-set-compiler."
    );
  }
}

// packages/ship-set-components/src/index.ts
function registerAll() {
}
export {
  defineComponent,
  registerAll
};
