// packages/ship-set-components/src/define.ts
import { hydrate } from "@ship-set/dom";
function defineComponent(template, opts = {}) {
  const doc = opts.doc ?? (typeof document !== "undefined" ? document : void 0);
  if (!doc) {
    throw new Error(
      "[ship-set/components] defineComponent: no document available \u2014 pass `opts.doc` when calling outside the browser/jsdom."
    );
  }
  const merged = opts.styles ? template.replace("</component>", `<style>${opts.styles}</style></component>`) : template;
  const container = doc.createElement("div");
  hydrate(merged, void 0, { container });
}

// packages/ship-set-components/src/set-status-dot/set-status-dot.html.set
var set_status_dot_html_default = `<component name="set-status-dot" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="success" reflect />
  <prop name="pulse" type="bool" default="false" reflect />
  <prop name="size" values="sm|md" default="md" reflect />
  <prop name="label" type="string" default="" />

  <style>
    :host {
      display: inline-flex;
      vertical-align: middle;
      /* Per-host tone channel \u2014 \`:host([tone=...])\` rules below override
       * --tone-fg so the dot color tracks the prop. Same pattern the Lit
       * primitives use. */
      --tone-fg: var(--success);
    }
    :host([tone='default']) { --tone-fg: var(--foreground); }
    :host([tone='primary']) { --tone-fg: var(--primary); }
    :host([tone='info'])    { --tone-fg: var(--info); }
    :host([tone='success']) { --tone-fg: var(--success); }
    :host([tone='warning']) { --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone-fg: var(--destructive); }
    :host([tone='accent'])  { --tone-fg: var(--accent); }
    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }

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

    :host([pulse]) .dot { animation: set-status-dot-pulse 2.5s infinite; }
    @keyframes set-status-dot-pulse {
      0%, 100% { opacity: 1; transform: scale(1); }
      50%      { opacity: 0.75; transform: scale(0.9); }
    }

    @media (prefers-reduced-motion: reduce) {
      :host([pulse]) .dot { animation: none; }
    }
  </style>
</component>

<span class="dot"
      part="dot"
      role={label() ? 'img' : ''}
      aria-label={label()}
      aria-hidden={label() ? '' : 'true'}></span>
`;

// packages/ship-set-components/src/set-status-dot/set-status-dot.ts
function registerSetStatusDot() {
  defineComponent(set_status_dot_html_default);
}

// packages/ship-set-components/src/set-spinner/set-spinner.html.set
var set_spinner_html_default = `<component name="set-spinner" shadow>
  <prop name="size" values="xs|sm|md|lg" default="md" reflect />
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="label" type="string" default="" />

  <style>
    :host { display: inline-flex; vertical-align: middle; --tone-fg: currentColor; --sz: 16px; --bw: 2px; }
    :host([tone='default']) { --tone-fg: var(--muted-foreground); }
    :host([tone='primary']) { --tone-fg: var(--primary); }
    :host([tone='info'])    { --tone-fg: var(--info); }
    :host([tone='success']) { --tone-fg: var(--success); }
    :host([tone='warning']) { --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone-fg: var(--destructive); }
    :host([tone='accent'])  { --tone-fg: var(--accent); }
    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }
    :host([size='xs']) { --sz: 10px; --bw: 1.5px; }
    :host([size='sm']) { --sz: 12px; --bw: 1.5px; }
    :host([size='md']) { --sz: 16px; --bw: 2px; }
    :host([size='lg']) { --sz: 24px; --bw: 2.5px; }
    .ring {
      width: var(--sz); height: var(--sz);
      border: var(--bw) solid currentColor;
      border-color: var(--tone-fg);
      border-right-color: transparent;
      border-radius: 50%;
      animation: set-spinner-spin 0.7s linear infinite;
      display: inline-block;
    }
    @keyframes set-spinner-spin { to { transform: rotate(360deg); } }
    @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }
  </style>
</component>

<span class="ring"
      part="ring"
      role={label() ? 'status' : ''}
      aria-label={label()}
      aria-hidden={label() ? '' : 'true'}></span>
`;

// packages/ship-set-components/src/set-spinner/set-spinner.ts
function registerSetSpinner() {
  defineComponent(set_spinner_html_default);
}

// packages/ship-set-components/src/set-badge/set-badge.html.set
var set_badge_html_default = `<component name="set-badge" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="variant" values="solid|soft|outline" default="soft" reflect />
  <prop name="size" values="sm|md" default="md" reflect />

  <style>
    :host {
      display: inline-flex;
      vertical-align: middle;
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='primary']) { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }
    :host([tone='info'])    { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }
    :host([tone='success']) { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }
    :host([tone='warning']) { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }
    :host([tone='accent'])  { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }
    :host([tone='muted'])   { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }

    .root {
      display: inline-flex;
      align-items: center;
      gap: 4px;
      padding: 1px 8px;
      font: inherit;
      font-family: var(--font-body);
      font-weight: var(--font-weight-medium, 500);
      font-size: 11px;
      line-height: 1.5;
      letter-spacing: 0.01em;
      border-radius: var(--radius-sm);
      border: 1px solid transparent;
      white-space: nowrap;
    }
    :host([size='md']) .root { font-size: 12px; padding: 2px 9px; }

    :host([variant='solid']) .root { background: var(--tone); color: var(--tone-contrast); border-color: color-mix(in oklch, var(--tone) 100%, black 6%); }
    :host([variant='soft'])  .root { background: color-mix(in oklch, var(--tone) 14%, transparent); color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 22%, transparent); }
    :host([variant='outline']) .root { background: transparent; color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 50%, transparent); }
  </style>
</component>

<span class="root" part="root">
  <slot></slot>
</span>
`;

// packages/ship-set-components/src/set-badge/set-badge.ts
function registerSetBadge() {
  defineComponent(set_badge_html_default);
}

// packages/ship-set-components/src/set-chip/set-chip.html.set
var set_chip_html_default = `<component name="set-chip" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="variant" values="solid|soft|outline|ghost" default="soft" reflect />
  <prop name="size" values="sm|md" default="sm" reflect />

  <style>
    :host {
      display: inline-flex;
      vertical-align: middle;
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='primary']) { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }
    :host([tone='info'])    { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }
    :host([tone='success']) { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }
    :host([tone='warning']) { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }
    :host([tone='accent'])  { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }
    :host([tone='muted'])   { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }

    .root {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 3px 10px;
      font-family: var(--font-body);
      font-weight: var(--font-weight-medium, 500);
      font-size: 12px;
      line-height: 1.4;
      border-radius: var(--radius-full);
      border: 1px solid transparent;
      white-space: nowrap;
    }
    :host([size='md']) .root { font-size: 13px; padding: 5px 12px; }

    :host([variant='solid']) .root { background: var(--tone); color: var(--tone-contrast); border-color: color-mix(in oklch, var(--tone) 100%, black 6%); }
    :host([variant='soft'])  .root { background: color-mix(in oklch, var(--tone) 14%, transparent); color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 22%, transparent); }
    :host([variant='outline']) .root { background: transparent; color: var(--tone-fg); border-color: color-mix(in oklch, var(--tone) 50%, transparent); }
    :host([variant='ghost']) .root { background: transparent; color: var(--tone-fg); border-color: transparent; }

    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
  </style>
</component>

<span class="root" part="root">
  <slot name="leading"></slot>
  <slot></slot>
  <slot name="trailing"></slot>
</span>
`;

// packages/ship-set-components/src/set-chip/set-chip.ts
function registerSetChip() {
  defineComponent(set_chip_html_default);
}

// packages/ship-set-components/src/set-card/set-card.html.set
var set_card_html_default = `<component name="set-card" shadow>
  <prop name="padding" values="none|sm|md|lg" default="md" reflect />
  <prop name="variant" values="surface|outline|ghost" default="surface" reflect />
  <prop name="interactive" type="bool" default="false" reflect />

  <style>
    :host { display: block; }
    :host([hidden]) { display: none; }

    .root {
      display: flex;
      flex-direction: column;
      border-radius: var(--radius-lg);
      background: var(--card);
      border: 1px solid var(--border);
      color: var(--foreground);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    :host([variant='outline'])  .root { background: transparent; }
    :host([variant='ghost'])    .root { background: transparent; border-color: transparent; }

    :host([padding='none']) .root { padding: 0; }
    :host([padding='sm'])   .root { padding: 12px; }
    :host([padding='md'])   .root { padding: 16px 18px; }
    :host([padding='lg'])   .root { padding: 24px 28px; }

    :host([interactive]) .root { cursor: pointer; }
    :host([interactive]) .root:hover {
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      box-shadow: var(--shadow-sm);
    }

    .header { display: contents; }
    .footer { display: contents; }

    ::slotted([slot='header']) { display: block; }
    ::slotted([slot='footer']) { display: block; margin-top: 12px; }
  </style>
</component>

<div class="root" part="root">
  <slot name="header"></slot>
  <slot></slot>
  <slot name="footer"></slot>
</div>
`;

// packages/ship-set-components/src/set-card/set-card.ts
function registerSetCard() {
  defineComponent(set_card_html_default);
}

// packages/ship-set-components/src/set-link/set-link.html.set
var set_link_html_default = `<component name="set-link" shadow>
  <prop name="href" type="string" default="" reflect />
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="variant" values="default|underline|subtle" default="default" reflect />
  <prop name="target" type="string" default="" reflect />
  <prop name="rel" type="string" default="" reflect />

  <style>
    :host { display: inline; --tone-fg: currentColor; }
    :host([tone='primary']) { --tone-fg: var(--primary); }
    :host([tone='info'])    { --tone-fg: var(--info); }
    :host([tone='success']) { --tone-fg: var(--success); }
    :host([tone='warning']) { --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone-fg: var(--destructive); }
    :host([tone='accent'])  { --tone-fg: var(--accent); }
    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }

    a {
      color: var(--tone-fg);
      text-decoration: none;
      cursor: pointer;
      border-radius: 2px;
      transition: color var(--duration-fast) var(--ease-out), text-decoration-color var(--duration-fast) var(--ease-out);
    }
    a:hover { text-decoration: underline; text-decoration-thickness: 1px; text-underline-offset: 2px; }
    :host([variant='underline']) a { text-decoration: underline; text-decoration-thickness: 1px; text-underline-offset: 2px; text-decoration-color: color-mix(in oklch, var(--tone-fg) 40%, transparent); }
    :host([variant='subtle']) a:hover { color: color-mix(in oklch, var(--tone-fg) 80%, var(--foreground)); }
    a:focus-visible { outline: 2px solid var(--ring-color, currentColor); outline-offset: 2px; }
  </style>
</component>

<a part="link"
   href={href()}
   target={target()}
   rel={rel()}>
  <slot></slot>
</a>
`;

// packages/ship-set-components/src/set-link/set-link.ts
function registerSetLink() {
  defineComponent(set_link_html_default);
}

// packages/ship-set-components/src/set-divider/set-divider.html.set
var set_divider_html_default = `<component name="set-divider" shadow>
  <prop name="orientation" values="horizontal|vertical" default="horizontal" reflect />
  <prop name="tone" values="default|muted|primary|destructive" default="default" reflect />

  <style>
    :host { display: block; --tone-bg: var(--border); }
    :host([tone='muted'])       { --tone-bg: var(--border-subtle, color-mix(in oklch, var(--border) 50%, transparent)); }
    :host([tone='primary'])     { --tone-bg: color-mix(in oklch, var(--primary) 50%, var(--border)); }
    :host([tone='destructive']) { --tone-bg: color-mix(in oklch, var(--destructive) 50%, var(--border)); }

    .line {
      background: var(--tone-bg);
      border: 0;
    }
    :host([orientation='horizontal']) .line { width: 100%; height: 1px; margin: var(--space-2) 0; }
    :host([orientation='vertical']) {
      display: inline-block;
      vertical-align: middle;
      align-self: stretch;
    }
    :host([orientation='vertical']) .line { width: 1px; height: 1em; margin: 0 var(--space-2); display: inline-block; }
  </style>
</component>

<hr class="line" part="line" role="separator" aria-orientation={orientation()}>
`;

// packages/ship-set-components/src/set-divider/set-divider.ts
function registerSetDivider() {
  defineComponent(set_divider_html_default);
}

// packages/ship-set-components/src/set-skeleton/set-skeleton.html.set
var set_skeleton_html_default = `<component name="set-skeleton" shadow>
  <prop name="shape" values="text|line|block|circle" default="text" reflect />
  <prop name="size" values="sm|md|lg" default="md" reflect />

  <style>
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
      animation: set-skeleton-pulse 1.5s ease-in-out infinite;
      border-radius: var(--radius-sm);
    }
    :host([shape='circle']) .sk { border-radius: 50%; }

    /* Text/line \u2014 narrow heights with full-width default. */
    :host([shape='text']) .sk,
    :host([shape='line']) .sk { width: 100%; }
    :host([size='sm'][shape='text']) .sk,
    :host([size='sm'][shape='line']) .sk { height: 8px; }
    :host([size='md'][shape='text']) .sk,
    :host([size='md'][shape='line']) .sk { height: 12px; }
    :host([size='lg'][shape='text']) .sk,
    :host([size='lg'][shape='line']) .sk { height: 16px; }

    /* Block \u2014 square-ish, scales with parent. */
    :host([shape='block']) .sk { width: 100%; min-height: 80px; }

    /* Circle \u2014 fixed sizes. */
    :host([shape='circle'][size='sm']) .sk { width: 24px; height: 24px; }
    :host([shape='circle'][size='md']) .sk { width: 32px; height: 32px; }
    :host([shape='circle'][size='lg']) .sk { width: 48px; height: 48px; }

    @keyframes set-skeleton-pulse {
      0%   { background-position: 200% 0; }
      100% { background-position: -200% 0; }
    }
    @media (prefers-reduced-motion: reduce) {
      .sk { animation: none; }
    }
  </style>
</component>

<span class="sk" part="sk" aria-hidden="true"></span>
`;

// packages/ship-set-components/src/set-skeleton/set-skeleton.ts
function registerSetSkeleton() {
  defineComponent(set_skeleton_html_default);
}

// packages/ship-set-components/src/set-stat/set-stat.html.set
var set_stat_html_default = `<component name="set-stat" shadow>
  <prop name="value" type="string" default="" />
  <prop name="label" type="string" default="" />
  <prop name="hint"  type="string" default="" />
  <prop name="tone"  values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="size"  values="sm|md|lg" default="md" reflect />

  <style>
    :host { display: inline-flex; vertical-align: top; --tone-fg: var(--foreground); }
    :host([tone='primary']) { --tone-fg: var(--primary); }
    :host([tone='info'])    { --tone-fg: var(--info); }
    :host([tone='success']) { --tone-fg: var(--success); }
    :host([tone='warning']) { --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone-fg: var(--destructive); }
    :host([tone='accent'])  { --tone-fg: var(--accent); }
    :host([tone='muted'])   { --tone-fg: var(--muted-foreground); }

    .root {
      display: flex;
      flex-direction: column;
      gap: 2px;
    }
    .label {
      font-size: 11px;
      font-weight: var(--font-weight-medium, 500);
      color: var(--muted-foreground);
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    .value {
      font-family: var(--font-display, var(--font-body));
      font-weight: var(--font-weight-semibold, 600);
      font-variant-numeric: tabular-nums;
      color: var(--tone-fg);
      line-height: 1.1;
    }
    :host([size='sm']) .value { font-size: 18px; }
    :host([size='md']) .value { font-size: 24px; }
    :host([size='lg']) .value { font-size: 36px; }
    .hint { font-size: 11px; color: var(--muted-foreground); }
  </style>
</component>

<div class="root" part="root">
  {#if label() !== ''}
    <span class="label" part="label">{label()}</span>
  {/if}
  <span class="value" part="value">{value()}<slot></slot></span>
  {#if hint() !== ''}
    <span class="hint" part="hint">{hint()}</span>
  {/if}
</div>
`;

// packages/ship-set-components/src/set-stat/set-stat.ts
function registerSetStat() {
  defineComponent(set_stat_html_default);
}

// packages/ship-set-components/src/set-mark/set-mark.html.set
var set_mark_html_default = `<component name="set-mark" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="warning" reflect />

  <style>
    :host { display: inline; --tone: var(--warning); --tone-fg: var(--warning); }
    :host([tone='default'])     { --tone: var(--foreground); --tone-fg: var(--foreground); }
    :host([tone='primary'])     { --tone: var(--primary); --tone-fg: var(--primary); }
    :host([tone='info'])        { --tone: var(--info); --tone-fg: var(--info); }
    :host([tone='success'])     { --tone: var(--success); --tone-fg: var(--success); }
    :host([tone='warning'])     { --tone: var(--warning); --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }
    :host([tone='accent'])      { --tone: var(--accent); --tone-fg: var(--accent); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

    mark {
      background: color-mix(in oklch, var(--tone) 20%, transparent);
      color: var(--tone-fg);
      padding: 0 2px;
      border-radius: 2px;
    }
  </style>
</component>

<mark part="mark"><slot></slot></mark>
`;

// packages/ship-set-components/src/set-mark/set-mark.ts
function registerSetMark() {
  defineComponent(set_mark_html_default);
}

// packages/ship-set-components/src/set-kbd/set-kbd.html.set
var set_kbd_html_default = `<component name="set-kbd" shadow>
  <prop name="size" values="sm|md" default="md" reflect />

  <style>
    :host { display: inline-flex; vertical-align: baseline; }

    kbd {
      display: inline-flex;
      align-items: center;
      gap: 2px;
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

    :host([size='sm']) kbd { font-size: 10px; padding: 1px 4px; }
    :host([size='md']) kbd { font-size: 11px; padding: 2px 5px; }
  </style>
</component>

<kbd part="kbd"><slot></slot></kbd>
`;

// packages/ship-set-components/src/set-kbd/set-kbd.ts
function registerSetKbd() {
  defineComponent(set_kbd_html_default);
}

// packages/ship-set-components/src/set-avatar/set-avatar.html.set
var set_avatar_html_default = `<component name="set-avatar" shadow>
  <prop name="size" values="xs|sm|md|lg|xl" default="md" reflect />
  <prop name="shape" values="round|square" default="round" reflect />
  <prop name="src" type="string" default="" />
  <prop name="name" type="string" default="" />
  <prop name="alt" type="string" default="" />

  <style>
    :host {
      display: inline-flex;
      vertical-align: middle;
      --sz: 32px;
      --fs: 13px;
    }
    :host([size='xs']) { --sz: 20px; --fs: 10px; }
    :host([size='sm']) { --sz: 26px; --fs: 11px; }
    :host([size='md']) { --sz: 32px; --fs: 13px; }
    :host([size='lg']) { --sz: 40px; --fs: 15px; }
    :host([size='xl']) { --sz: 56px; --fs: 22px; }

    .root {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--sz);
      height: var(--sz);
      flex: none;
      background: color-mix(in oklch, var(--accent) 30%, var(--muted));
      color: var(--accent-foreground, var(--foreground));
      font-family: var(--font-body);
      font-weight: var(--font-weight-semibold, 600);
      font-size: var(--fs);
      letter-spacing: -0.01em;
      overflow: hidden;
      user-select: none;
    }
    :host([shape='round']) .root  { border-radius: 50%; }
    :host([shape='square']) .root { border-radius: var(--radius-sm); }

    img {
      width: 100%;
      height: 100%;
      object-fit: cover;
      display: block;
    }
  </style>
</component>

<span class="root" part="root" aria-label={alt() || name()}>
  {#if src() !== ''}
    <img part="image" src={src()} alt={alt() || name()} />
  {:else}
    {name() ? name().slice(0, 1).toUpperCase() : '?'}
  {/if}
</span>
`;

// packages/ship-set-components/src/set-avatar/set-avatar.ts
function registerSetAvatar() {
  defineComponent(set_avatar_html_default);
}

// packages/ship-set-components/src/set-progress/set-progress.html.set
var set_progress_html_default = `<component name="set-progress" shadow>
  <prop name="value" type="number" default="0" />
  <prop name="max" type="number" default="100" />
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent" default="primary" reflect />
  <prop name="size" values="sm|md|lg" default="md" reflect />
  <prop name="indeterminate" type="bool" default="false" reflect />
  <prop name="label" type="string" default="" />

  <style>
    :host { display: block; --tone: var(--primary); --h: 6px; }
    :host([tone='default']) { --tone: var(--foreground); }
    :host([tone='primary']) { --tone: var(--primary); }
    :host([tone='info'])    { --tone: var(--info); }
    :host([tone='success']) { --tone: var(--success); }
    :host([tone='warning']) { --tone: var(--warning); }
    :host([tone='destructive']) { --tone: var(--destructive); }
    :host([tone='accent'])  { --tone: var(--accent); }
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
      animation: set-progress-indet 1.4s ease-in-out infinite;
    }
    @keyframes set-progress-indet {
      0%   { transform: translateX(-100%); }
      50%  { transform: translateX(150%); }
      100% { transform: translateX(350%); }
    }
    @media (prefers-reduced-motion: reduce) {
      :host([indeterminate]) .fill { animation-duration: 3s; }
    }
  </style>
</component>

<div class="track"
     part="track"
     role="progressbar"
     aria-valuenow={indeterminate() ? '' : value()}
     aria-valuemin="0"
     aria-valuemax={max()}
     aria-label={label()}>
  <div class="fill"
       part="fill"
       style={indeterminate() ? '' : ('width:' + Math.min(100, Math.max(0, (value() / Math.max(1, max())) * 100)) + '%')}></div>
</div>
`;

// packages/ship-set-components/src/set-progress/set-progress.ts
function registerSetProgress() {
  defineComponent(set_progress_html_default);
}

// packages/ship-set-components/src/set-button/set-button.html.set
var set_button_html_default = `<component name="set-button" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="default" reflect />
  <prop name="variant" values="solid|soft|outline|ghost|default" default="default" reflect />
  <prop name="size" values="sm|md|lg" default="md" reflect />
  <prop name="type" values="button|submit|reset" default="button" reflect />
  <prop name="disabled" type="bool" default="false" reflect />
  <prop name="loading" type="bool" default="false" reflect />
  <prop name="label" type="string" default="" />
  <prop name="pressed" type="string" default="" reflect />
  <prop name="expanded" type="string" default="" reflect />

  <style>
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
    :host([hidden]) { display: none; }

    /* Tone matrix */
    :host([tone='primary'])     { --tone: var(--primary); --tone-fg: var(--primary); --tone-contrast: var(--primary-foreground); }
    :host([tone='info'])        { --tone: var(--info); --tone-fg: var(--info); --tone-contrast: var(--info-foreground); }
    :host([tone='success'])     { --tone: var(--success); --tone-fg: var(--success); --tone-contrast: var(--success-foreground); }
    :host([tone='warning'])     { --tone: var(--warning); --tone-fg: var(--warning); --tone-contrast: var(--warning-foreground); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); --tone-contrast: var(--destructive-foreground); }
    :host([tone='accent'])      { --tone: var(--accent); --tone-fg: var(--accent); --tone-contrast: var(--accent-foreground); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }

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

    /* Default (no variant attr) \u2014 neutral bordered button. */
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

    /* Active press \u2014 feels clicked across all variants. */
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

    /* Loading state \u2014 fade the label, overlay a spinner. */
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
      animation: set-button-spin 0.7s linear infinite;
    }
    @keyframes set-button-spin { to { transform: rotate(360deg); } }

    /* Icon-only \u2014 collapse padding to a square when default slot is empty. */
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
  </style>
</component>

<button
  part="button"
  type={type()}
  disabled={disabled() || loading()}
  aria-busy={loading() ? 'true' : 'false'}
  aria-disabled={(disabled() || loading()) ? 'true' : 'false'}
  aria-label={label()}
  aria-pressed={pressed()}
  aria-expanded={expanded()}
>
  <span class="content">
    <slot name="leading"></slot>
    <slot></slot>
    <slot name="trailing"></slot>
  </span>
  {#if loading()}
    <span class="spinner-wrap" aria-hidden="true"><span class="spinner"></span></span>
  {/if}
</button>
`;

// packages/ship-set-components/src/set-button/set-button.ts
function registerSetButton() {
  defineComponent(set_button_html_default);
}

// packages/ship-set-components/src/set-tabs/set-tabs.html.set
var set_tabs_html_default = `<component name="set-tabs" shadow>
  <prop name="variant" values="underline|pill|enclosed" default="underline" reflect />

  <style>
    :host { display: block; }
    .strip {
      display: flex;
      align-items: center;
      gap: 4px;
      flex-wrap: wrap;
      border-bottom: 1px solid var(--border);
    }
    :host([variant='pill']) .strip {
      gap: 6px;
      border-bottom: 0;
      padding: 4px;
      background: var(--muted);
      border-radius: var(--radius-md);
    }
    :host([variant='enclosed']) .strip {
      gap: 0;
      border-bottom: 1px solid var(--border);
    }

    ::slotted(set-tab),
    ::slotted([slot='tab']) {
      flex: 0 0 auto;
    }
  </style>
</component>

<div class="strip" part="strip" role="tablist">
  <slot></slot>
</div>
`;

// packages/ship-set-components/src/set-tabs/set-tabs.ts
function registerSetTabs() {
  defineComponent(set_tabs_html_default);
}

// packages/ship-set-components/src/set-tab/set-tab.html.set
var set_tab_html_default = `<component name="set-tab" shadow>
  <prop name="name" type="string" default="" reflect />
  <prop name="active" type="bool" default="false" reflect />
  <prop name="count" type="string" default="" />
  <prop name="disabled" type="bool" default="false" reflect />

  <style>
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
      transition:
        color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out);
    }
    button:hover:not(:disabled) {
      color: var(--foreground);
    }
    :host([active]) button {
      color: var(--foreground);
      border-bottom-color: var(--primary);
    }
    button:disabled {
      cursor: not-allowed;
      opacity: 0.5;
    }
    .count {
      font-size: 11px;
      font-weight: var(--font-weight-medium, 500);
      color: var(--muted-foreground);
      padding: 0 6px;
      border-radius: var(--radius-full);
      background: color-mix(in oklch, var(--muted) 80%, transparent);
    }
    :host([active]) .count { color: var(--primary); background: color-mix(in oklch, var(--primary) 12%, transparent); }
    button:focus-visible {
      outline: 2px solid color-mix(in oklch, var(--primary) 55%, transparent);
      outline-offset: -2px;
    }
  </style>
</component>

<button
  part="button"
  type="button"
  role="tab"
  data-name={name()}
  aria-selected={active() ? 'true' : 'false'}
  aria-disabled={disabled() ? 'true' : 'false'}
  disabled={disabled()}
>
  <slot></slot>
  {#if count() !== ''}
    <span class="count" aria-hidden="true">{count()}</span>
  {/if}
</button>
`;

// packages/ship-set-components/src/set-tab/set-tab.ts
function registerSetTab() {
  defineComponent(set_tab_html_default);
}

// packages/ship-set-components/src/set-empty-state/set-empty-state.html.set
var set_empty_state_html_default = `<component name="set-empty-state" shadow>
  <prop name="tone" values="default|primary|info|success|warning|destructive|accent|muted" default="muted" reflect />
  <prop name="size" values="sm|md|lg" default="md" reflect />
  <prop name="variant" values="default|inset" default="default" reflect />

  <style>
    :host {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 12px;
      padding: 32px 24px;
      text-align: center;
      color: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
    }
    :host([tone='default'])     { --tone-fg: var(--foreground); }
    :host([tone='primary'])     { --tone-fg: var(--primary); }
    :host([tone='info'])        { --tone-fg: var(--info); }
    :host([tone='success'])     { --tone-fg: var(--success); }
    :host([tone='warning'])     { --tone-fg: var(--warning); }
    :host([tone='destructive']) { --tone-fg: var(--destructive); }
    :host([tone='accent'])      { --tone-fg: var(--accent); }
    :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

    :host([variant='inset']) {
      background: var(--card);
      border: 1px dashed var(--border);
      border-radius: var(--radius-lg);
    }

    :host([size='sm']) { padding: 16px; gap: 8px; }
    :host([size='md']) { padding: 32px 24px; gap: 12px; }
    :host([size='lg']) { padding: 56px 32px; gap: 16px; }

    .icon-slot { color: var(--tone-fg); }
    ::slotted([slot='icon']) {
      color: var(--tone-fg);
    }

    .title {
      color: var(--foreground);
      font-family: var(--font-display, var(--font-body));
      font-weight: var(--font-weight-semibold, 600);
      font-size: 16px;
      margin: 0;
    }
    :host([size='sm']) .title { font-size: 14px; }
    :host([size='lg']) .title { font-size: 20px; }

    .body {
      max-width: 40ch;
      font-size: 13px;
      color: var(--muted-foreground);
    }

    .action-slot { display: inline-flex; gap: 8px; margin-top: 8px; }
  </style>
</component>

<div class="icon-slot" part="icon"><slot name="icon"></slot></div>
<h3 class="title" part="title"><slot name="title"></slot></h3>
<div class="body" part="body"><slot></slot></div>
<div class="action-slot" part="action"><slot name="action"></slot></div>
`;

// packages/ship-set-components/src/set-empty-state/set-empty-state.ts
function registerSetEmptyState() {
  defineComponent(set_empty_state_html_default);
}

// packages/ship-set-components/src/set-input/set-input.html.set
var set_input_html_default = `<component name="set-input" shadow>
  <prop name="value" type="string" default="" />
  <prop name="placeholder" type="string" default="" />
  <prop name="type" values="text|search|email|url|tel|password|number" default="text" reflect />
  <prop name="size" values="sm|md|lg" default="md" reflect />
  <prop name="tone" values="default|primary|info|success|warning|destructive" default="default" reflect />
  <prop name="disabled" type="bool" default="false" reflect />
  <prop name="readonly" type="bool" default="false" reflect />
  <prop name="invalid" type="bool" default="false" reflect />
  <prop name="name" type="string" default="" />
  <prop name="label" type="string" default="" />
  <prop name="autocomplete" type="string" default="" />

  <style>
    :host {
      display: inline-flex;
      vertical-align: middle;
      width: 100%;
      --tone-border: var(--border);
      --tone-ring: var(--primary);
      --h: 34px;
      --fs: 13px;
      --px: 10px;
    }
    :host([tone='primary'])     { --tone-border: var(--primary); --tone-ring: var(--primary); }
    :host([tone='success'])     { --tone-border: var(--success); --tone-ring: var(--success); }
    :host([tone='warning'])     { --tone-border: var(--warning); --tone-ring: var(--warning); }
    :host([tone='destructive']),
    :host([invalid])            { --tone-border: var(--destructive); --tone-ring: var(--destructive); }
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
      transition: border-color var(--duration-fast) var(--ease-out), box-shadow var(--duration-fast) var(--ease-out);
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
  </style>
</component>

<label class="wrap" part="wrap">
  <slot name="leading"></slot>
  <input
    part="input"
    type={type()}
    value={value()}
    placeholder={placeholder()}
    name={name()}
    autocomplete={autocomplete()}
    disabled={disabled()}
    readonly={readonly()}
    aria-label={label()}
    aria-invalid={invalid() ? 'true' : 'false'}
  />
  <slot name="trailing"></slot>
</label>
`;

// packages/ship-set-components/src/set-input/set-input.ts
function registerSetInput() {
  defineComponent(set_input_html_default);
}

// packages/ship-set-components/src/index.ts
function registerAll() {
  registerSetStatusDot();
  registerSetSpinner();
  registerSetBadge();
  registerSetChip();
  registerSetCard();
  registerSetLink();
  registerSetDivider();
  registerSetSkeleton();
  registerSetStat();
  registerSetMark();
  registerSetKbd();
  registerSetAvatar();
  registerSetProgress();
  registerSetButton();
  registerSetTabs();
  registerSetTab();
  registerSetEmptyState();
  registerSetInput();
}
if (typeof document !== "undefined") {
  registerAll();
}
export {
  defineComponent,
  registerAll,
  registerSetAvatar,
  registerSetBadge,
  registerSetButton,
  registerSetCard,
  registerSetChip,
  registerSetDivider,
  registerSetEmptyState,
  registerSetInput,
  registerSetKbd,
  registerSetLink,
  registerSetMark,
  registerSetProgress,
  registerSetSkeleton,
  registerSetSpinner,
  registerSetStat,
  registerSetStatusDot,
  registerSetTab,
  registerSetTabs
};
