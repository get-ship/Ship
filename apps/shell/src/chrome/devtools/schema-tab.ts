//! `<ship-devtools-schema>` — lists the `ship.*` ops referenced by the
//! mounted view's template + manifest, cross-linked to the registry
//! catalog served at `/api/ship/schema`.
//!
//! Purely read-only. Unknown ops (not in the catalog) render with a
//! warning tone — the ast lint rule catches these too, but surfacing
//! them alongside schema entries makes the link concrete.

import { LitElement, css, html } from "lit";
import { customElement, state } from "lit/decorators.js";

interface ArgDescriptor {
  name: string;
  kind: string;
  required?: boolean;
}

interface OpDescriptor {
  name: string;
  description: string;
  args: ArgDescriptor[];
  returns: string;
  permission: string;
  sensitive?: boolean;
}

interface ViewSource {
  template: string;
  manifest: string | null;
}

interface Reference {
  op: string;
  soft: boolean;
  known: boolean;
}

const SHIP_OP_RE = /ship\.(try\.)?([a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)/gi;

function collectReferences(
  src: string,
  ops: Map<string, OpDescriptor>,
): Map<string, Reference> {
  const out = new Map<string, Reference>();
  let m: RegExpExecArray | null;
  SHIP_OP_RE.lastIndex = 0;
  while ((m = SHIP_OP_RE.exec(src)) !== null) {
    const soft = !!m[1];
    const name = `${m[2]}.${m[3]}`;
    const key = `${soft ? "try:" : ""}${name}`;
    if (!out.has(key)) {
      out.set(key, { op: name, soft, known: ops.has(name) });
    }
  }
  return out;
}

@customElement("ship-devtools-schema")
export class ShipDevtoolsSchema extends LitElement {
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
      gap: var(--space-3, 12px);
      padding: 4px var(--space-3, 12px);
      border-bottom: 1px solid var(--border);
      background: var(--sidebar);
      color: var(--muted-foreground);
      font-size: var(--text-xs);
    }
    .body {
      flex: 1;
      overflow: auto;
    }
    details {
      border-bottom: 1px solid
        color-mix(in oklch, var(--border) 40%, transparent);
    }
    details > summary {
      list-style: none;
      cursor: pointer;
      padding: 6px var(--space-3, 12px);
      display: grid;
      grid-template-columns: auto 1fr auto;
      gap: var(--space-3, 12px);
      align-items: baseline;
      font-family: var(--font-mono);
      font-size: var(--text-xs);
    }
    details > summary::-webkit-details-marker {
      display: none;
    }
    .op {
      color: var(--primary);
    }
    .op.soft::before {
      content: "try · ";
      color: var(--muted-foreground);
    }
    .op.unknown {
      color: var(--destructive);
    }
    .perm {
      color: var(--muted-foreground);
    }
    .desc {
      color: var(--muted-foreground);
    }
    .detail {
      padding: 4px var(--space-3, 12px) 10px calc(var(--space-3, 12px) + 14px);
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      display: grid;
      gap: 4px;
    }
    .detail code {
      font-family: var(--font-mono);
      color: var(--foreground);
    }
    .empty {
      padding: var(--space-5, 20px);
      text-align: center;
      color: var(--muted-foreground);
    }
  `;

  @state() private _refs: Reference[] = [];
  @state() private _ops: Map<string, OpDescriptor> = new Map();
  @state() private _error: string | null = null;

  private _lastPath: string | null = null;
  private _onNavigate = (): void => {
    this._lastPath = null;
    void this._refresh();
  };
  private _onDockActivate = (ev: Event): void => {
    const tab = (ev as CustomEvent<{ tab: string }>).detail?.tab;
    if (tab === "schema") void this._refresh();
  };

  connectedCallback(): void {
    super.connectedCallback();
    document
      .querySelector("ship-shell")
      ?.addEventListener("ship:navigate", this._onNavigate);
    this.closest("ship-panel-dock")?.addEventListener(
      "ship:dock:activate",
      this._onDockActivate,
    );
    void this._refresh();
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    document
      .querySelector("ship-shell")
      ?.removeEventListener("ship:navigate", this._onNavigate);
    this.closest("ship-panel-dock")?.removeEventListener(
      "ship:dock:activate",
      this._onDockActivate,
    );
  }

  refresh(): void {
    this._lastPath = null;
    void this._refresh();
  }

  private _stripPrefix(path: string): string {
    const m = path.match(/^\/api\/v\/(.+)$/);
    return m ? m[1] : path.replace(/^\/+/, "");
  }

  private async _loadOps(): Promise<void> {
    if (this._ops.size > 0) return;
    try {
      const resp = await fetch("/api/ship/schema", {
        credentials: "same-origin",
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        ok: boolean;
        data: { ops: OpDescriptor[] };
      };
      const map = new Map<string, OpDescriptor>();
      for (const op of body.data.ops) map.set(op.name, op);
      this._ops = map;
    } catch {
      /* best-effort catalog */
    }
  }

  private async _refresh(): Promise<void> {
    const current = window.ship?.current();
    const path = current?.path ?? location.pathname;
    const stripped = this._stripPrefix(path);
    if (!stripped) {
      this._refs = [];
      return;
    }
    if (stripped === this._lastPath) return;
    this._lastPath = stripped;
    this._error = null;
    await this._loadOps();
    try {
      const resp = await fetch(
        `/api/view-source/${encodeURI(stripped)}`,
        { credentials: "same-origin" },
      );
      if (resp.status === 404) {
        this._refs = [];
        return;
      }
      if (!resp.ok) {
        this._error = `fetch failed: ${resp.status}`;
        return;
      }
      const src = (await resp.json()) as ViewSource;
      const combined = `${src.template}\n${src.manifest ?? ""}`;
      this._refs = [...collectReferences(combined, this._ops).values()].sort(
        (a, b) => a.op.localeCompare(b.op),
      );
    } catch (e) {
      this._error = e instanceof Error ? e.message : String(e);
    }
  }

  render() {
    if (this._error) {
      return html`<div class="empty">${this._error}</div>`;
    }
    if (this._refs.length === 0) {
      return html`<ship-empty-state>
        <ship-icon slot="icon" name="network"></ship-icon>
        <span slot="title">No schema references</span>
        No <code>ship.*</code> calls detected in this view.
      </ship-empty-state>`;
    }
    const known = this._refs.filter((r) => r.known).length;
    return html`
      <div class="toolbar">
        <span>${this._refs.length} ops referenced</span>
        <span style="flex:1"></span>
        <span>${known} known · ${this._refs.length - known} unknown</span>
      </div>
      <div class="body">
        ${this._refs.map((r) => this._renderOp(r))}
      </div>
    `;
  }

  private _renderOp(r: Reference) {
    const desc = this._ops.get(r.op);
    return html`
      <details>
        <summary>
          <span class="op ${r.soft ? "soft" : ""} ${r.known ? "" : "unknown"}"
            >${r.op}</span
          >
          <span class="desc">${desc?.description ?? "(not in catalog)"}</span>
          <span class="perm">${desc?.permission ?? ""}</span>
        </summary>
        ${desc
          ? html`
              <div class="detail">
                ${desc.args.length > 0
                  ? html`<div>
                      args:
                      ${desc.args.map(
                        (a) => html`<code
                          >${a.name}${a.required ? "" : "?"}:${a.kind}</code
                        >&nbsp;`,
                      )}
                    </div>`
                  : html`<div>args: <em>none</em></div>`}
                <div>returns: <code>${desc.returns}</code></div>
                ${desc.sensitive
                  ? html`<div><strong>sensitive</strong></div>`
                  : ""}
              </div>
            `
          : html`<div class="detail">
              Not registered in <code>/api/ship/schema</code>.
            </div>`}
      </details>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-schema": ShipDevtoolsSchema;
  }
}
