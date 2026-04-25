//! `<ship-devtools-components>` — browsable catalog of ship-elements
//! and any other custom elements the page registers, sourced from
//! `/api/assets/custom-elements.json` (the CEM manifest).
//!
//! Phase 2 scope: browse + filter + copy-to-clipboard snippet. Live
//! preview with prop editing lands once we have a sandbox iframe
//! that can safely mount a single element.

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

interface CemAttribute {
  name: string;
  description?: string;
  type?: { text?: string };
  default?: string;
}

interface CemDeclaration {
  kind: string;
  name: string;
  tagName?: string;
  customElement?: boolean;
  description?: string;
  attributes?: CemAttribute[];
  members?: Array<{ kind: string; name: string; description?: string }>;
  slots?: Array<{ name: string; description?: string }>;
}

interface CemModule {
  kind: string;
  path: string;
  declarations?: CemDeclaration[];
}

interface Cem {
  modules: CemModule[];
}

interface CatalogEntry {
  tag: string;
  description: string;
  attributes: CemAttribute[];
  slots: Array<{ name: string; description?: string }>;
}

function flatten(cem: Cem): CatalogEntry[] {
  const out: CatalogEntry[] = [];
  for (const m of cem.modules ?? []) {
    for (const d of m.declarations ?? []) {
      if (!d.customElement || !d.tagName) continue;
      out.push({
        tag: d.tagName,
        description: (d.description ?? "").trim(),
        attributes: d.attributes ?? [],
        slots: d.slots ?? [],
      });
    }
  }
  out.sort((a, b) => a.tag.localeCompare(b.tag));
  return out;
}

function snippet(entry: CatalogEntry): string {
  const attrs = entry.attributes
    .slice(0, 3)
    .filter((a) => !a.default || a.default === "false")
    .map((a) => {
      const typeText = a.type?.text ?? "";
      if (typeText.includes("boolean")) return ` ${a.name}`;
      return ` ${a.name}="…"`;
    })
    .join("");
  const hasDefaultSlot = entry.slots.some((s) => !s.name || s.name === "");
  return hasDefaultSlot
    ? `<${entry.tag}${attrs}>\n  \n</${entry.tag}>`
    : `<${entry.tag}${attrs}></${entry.tag}>`;
}

@customElement("ship-devtools-components")
export class ShipDevtoolsComponents extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      min-height: 0;
      font-size: var(--text-sm);
    }
    .toolbar {
      display: flex;
      align-items: center;
      gap: var(--space-2, 8px);
      padding: 4px var(--space-3, 12px);
      border-bottom: 1px solid var(--border);
      background: var(--sidebar);
      color: var(--muted-foreground);
      font-size: var(--text-xs);
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
      max-width: 280px;
    }
    .body {
      flex: 1;
      overflow: auto;
    }
    details {
      border-bottom: 1px solid
        color-mix(in oklch, var(--border) 40%, transparent);
    }
    summary {
      list-style: none;
      cursor: pointer;
      padding: 6px var(--space-3, 12px);
      display: grid;
      grid-template-columns: auto 1fr auto;
      gap: var(--space-3, 12px);
      align-items: baseline;
      font-size: var(--text-xs);
    }
    summary::-webkit-details-marker {
      display: none;
    }
    .tag {
      font-family: var(--font-mono);
      color: var(--primary);
    }
    .desc {
      color: var(--muted-foreground);
    }
    .copy {
      appearance: none;
      background: transparent;
      border: 1px solid var(--border);
      border-radius: var(--radius-sm, 4px);
      padding: 1px 8px;
      font: inherit;
      font-size: 10px;
      color: var(--muted-foreground);
      cursor: pointer;
    }
    .copy:hover {
      color: var(--foreground);
    }
    .panel {
      padding: 0 var(--space-3, 12px) 10px
        calc(var(--space-3, 12px) + 14px);
      display: grid;
      gap: 8px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .panel pre {
      margin: 0;
      padding: 6px 10px;
      background: var(--muted);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm, 4px);
      font-family: var(--font-mono);
      font-size: 11px;
      overflow-x: auto;
    }
    .attrs {
      display: grid;
      grid-template-columns: auto 1fr;
      gap: 2px 12px;
    }
    .attrs dt {
      font-family: var(--font-mono);
      color: var(--foreground);
    }
    .attrs dd {
      margin: 0;
    }
    .empty {
      padding: var(--space-5, 20px);
      text-align: center;
      color: var(--muted-foreground);
    }
  `;

  @state() private _entries: CatalogEntry[] = [];
  @state() private _filter = "";
  @state() private _error: string | null = null;

  connectedCallback(): void {
    super.connectedCallback();
    void this._load();
  }

  refresh(): void {
    this._entries = [];
    void this._load();
  }

  private async _load(): Promise<void> {
    if (this._entries.length > 0) return;
    try {
      const resp = await fetch("/api/assets/custom-elements.json", {
        credentials: "same-origin",
      });
      if (!resp.ok) {
        this._error = `fetch failed: ${resp.status}`;
        return;
      }
      const cem = (await resp.json()) as Cem;
      this._entries = flatten(cem);
    } catch (e) {
      this._error = e instanceof Error ? e.message : String(e);
    }
  }

  private _onFilter = (ev: Event): void => {
    this._filter = (ev.target as HTMLInputElement).value;
  };

  private async _copy(snip: string): Promise<void> {
    try {
      await navigator.clipboard.writeText(snip);
      window.ship?.notify?.("Copied", "success");
    } catch {
      window.ship?.notify?.("Copy failed", "warning");
    }
  }

  render() {
    if (this._error) {
      return html`<div class="empty">${this._error}</div>`;
    }
    if (this._entries.length === 0) {
      return html`<ship-empty-state>
        <ship-icon slot="icon" name="package"></ship-icon>
        <span slot="title">Loading components…</span>
        Fetching the component catalog from the daemon.
      </ship-empty-state>`;
    }
    const needle = this._filter.toLowerCase();
    const shown = needle
      ? this._entries.filter(
          (e) =>
            e.tag.toLowerCase().includes(needle) ||
            e.description.toLowerCase().includes(needle),
        )
      : this._entries;
    return html`
      <div class="toolbar">
        <input
          type="text"
          placeholder="Filter components…"
          .value=${this._filter}
          @input=${this._onFilter}
        />
        <span>${shown.length} / ${this._entries.length}</span>
      </div>
      <div class="body">
        ${shown.length === 0
          ? html`<ship-empty-state>
              <ship-icon slot="icon" name="search"></ship-icon>
              <span slot="title">No matches</span>
              No components match the current filter.
            </ship-empty-state>`
          : shown.map((e) => this._renderEntry(e))}
      </div>
    `;
  }

  private _renderEntry(e: CatalogEntry) {
    const snip = snippet(e);
    const firstLine = e.description.split("\n")[0] ?? "";
    return html`
      <details>
        <summary>
          <span class="tag">&lt;${e.tag}&gt;</span>
          <span class="desc">${firstLine}</span>
          <button
            class="copy"
            title="Copy snippet"
            @click=${(ev: MouseEvent) => {
              ev.preventDefault();
              ev.stopPropagation();
              void this._copy(snip);
            }}
          >
            Copy
          </button>
        </summary>
        <div class="panel">
          <pre>${snip}</pre>
          ${e.attributes.length > 0
            ? html`
                <div>
                  <strong>Attributes</strong>
                  <dl class="attrs">
                    ${e.attributes.map(
                      (a) => html`
                        <dt>${a.name}${a.default ? `=${a.default}` : ""}</dt>
                        <dd>${a.description ?? ""}</dd>
                      `,
                    )}
                  </dl>
                </div>
              `
            : nothing}
          ${e.slots.length > 0
            ? html`
                <div>
                  <strong>Slots</strong>:
                  ${e.slots
                    .map((s) => (s.name ? s.name : "(default)"))
                    .join(", ")}
                </div>
              `
            : nothing}
        </div>
      </details>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-components": ShipDevtoolsComponents;
  }
}
