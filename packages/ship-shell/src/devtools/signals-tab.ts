//! `<ship-devtools-signals>` — live Datastar signal inspector as a
//! panel dock tab.
//!
//! Extraction from the floating `signal-inspector.ts`: the snapshot
//! logic moves into a Lit component so the dock can host it next to
//! other devtools.
//!
//! Sampling: polls every 500ms while visible. Datastar doesn't expose
//! a public change stream, so a sampler is the only stable route.
//! Hidden tabs unmount their interval via Lit lifecycle, so cost is
//! only paid when the tab is in view.

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

function collectSignals(): Record<string, unknown> {
  const ds = (document as unknown as { _ds?: { signals?: unknown } })._ds;
  if (ds?.signals) {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(ds.signals as Record<string, unknown>)) {
      out[k] = (v as { value?: unknown })?.value ?? v;
    }
    return out;
  }
  const alt =
    (window as unknown as { ds?: { signals?: unknown } }).ds ??
    (globalThis as unknown as { __ds?: { signals?: unknown } }).__ds;
  if (alt?.signals) {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(
      alt.signals as Record<string, unknown>,
    )) {
      out[k] = (v as { value?: unknown })?.value ?? v;
    }
    return out;
  }
  const out: Record<string, unknown> = {};
  document.querySelectorAll("[data-signals]").forEach((el) => {
    const raw = el.getAttribute("data-signals");
    if (!raw) return;
    try {
      const parsed = new Function(`return (${raw})`)();
      Object.assign(out, parsed);
    } catch {
      /* ignore parse errors */
    }
  });
  const view = document.getElementById("ship-view-root");
  if (view) {
    const walk = (node: Element): void => {
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
      for (const child of Array.from(node.children)) walk(child);
    };
    walk(view);
  }
  return out;
}

function renderValue(v: unknown): string {
  if (typeof v === "string") {
    return v.length > 80 ? `"${v.slice(0, 77)}…"` : `"${v}"`;
  }
  try {
    return JSON.stringify(v);
  } catch {
    return String(v);
  }
}

@customElement("ship-devtools-signals")
export class ShipDevtoolsSignals extends LitElement {
  static styles = css`
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
      border-bottom: 1px solid
        color-mix(in oklch, var(--border) 40%, transparent);
      align-items: baseline;
    }
    .row:last-child {
      border-bottom: none;
    }
    .k {
      color: var(--primary);
      white-space: nowrap;
      flex-shrink: 0;
      font-weight: var(--font-weight-medium, 500);
    }
    .v {
      color: var(--muted-foreground);
      overflow-wrap: anywhere;
    }
    .empty {
      padding: var(--space-5, 20px);
      text-align: center;
      color: var(--muted-foreground);
    }
  `;

  @state() private _signals: Record<string, unknown> = {};
  @state() private _filter = "";

  private _timer: ReturnType<typeof setInterval> | null = null;

  connectedCallback(): void {
    super.connectedCallback();
    this._sample();
    this._timer = setInterval(() => this._sample(), 500);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    if (this._timer) clearInterval(this._timer);
    this._timer = null;
  }

  /** Public poke for the dock to force an immediate sample. */
  refresh(): void {
    this._sample();
  }

  private _sample(): void {
    this._signals = collectSignals();
  }

  private _onFilter = (ev: Event): void => {
    this._filter = (ev.target as HTMLInputElement).value;
  };

  render() {
    const keys = Object.keys(this._signals).sort();
    const needle = this._filter.toLowerCase();
    const filtered = needle
      ? keys.filter((k) => k.toLowerCase().includes(needle))
      : keys;
    return html`
      <div class="toolbar">
        <input
          type="text"
          placeholder="Filter signals…"
          .value=${this._filter}
          @input=${this._onFilter}
        />
        <span>${filtered.length} / ${keys.length}</span>
      </div>
      <div class="body">
        ${filtered.length === 0
          ? html`<div class="empty">
              ${needle
                ? "No signals match filter."
                : "No Datastar signals on this view."}
            </div>`
          : filtered.map(
              (k) => html`
                <div class="row">
                  <span class="k">${k}</span>
                  <span class="v">${renderValue(this._signals[k])}</span>
                </div>
              `,
            )}
        ${nothing}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-signals": ShipDevtoolsSignals;
  }
}
