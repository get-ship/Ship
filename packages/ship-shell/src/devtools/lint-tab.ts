//! `<ship-devtools-lint>` — runs `POST /api/lint/view` for the
//! mounted view's top-level directory and renders findings.
//!
//! Findings are file:line:col [rule] message entries. Errors render
//! first, then warnings. Rows have no edit affordance yet — that
//! lands in phase 2 (editable source tab + inline diagnostics).

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

interface Finding {
  file: string;
  line: number;
  col: number;
  rule: string;
  severity: "error" | "warn" | string;
  message: string;
}

@customElement("ship-devtools-lint")
export class ShipDevtoolsLint extends LitElement {
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
      color: var(--muted-foreground);
      background: var(--sidebar);
      font-size: var(--text-xs);
    }
    .toolbar .name {
      color: var(--foreground);
      font-weight: var(--font-weight-medium, 500);
    }
    .count {
      padding: 1px 6px;
      border-radius: 999px;
      font-size: 10px;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    .count.err {
      color: var(--destructive);
      border: 1px solid var(--destructive);
    }
    .count.warn {
      color: var(--warning, var(--muted-foreground));
      border: 1px solid color-mix(in oklch, var(--warning, var(--muted-foreground)) 60%, transparent);
    }
    .body {
      flex: 1;
      overflow: auto;
      font-family: var(--font-mono);
      font-size: var(--text-xs);
    }
    .row {
      display: grid;
      grid-template-columns: auto auto 1fr;
      gap: 12px;
      padding: 4px var(--space-3, 12px);
      align-items: baseline;
      border-bottom: 1px solid
        color-mix(in oklch, var(--border) 30%, transparent);
    }
    .sev {
      width: 52px;
      text-transform: uppercase;
      font-size: 10px;
      letter-spacing: 0.06em;
    }
    .sev.err {
      color: var(--destructive);
    }
    .sev.warn {
      color: var(--warning, var(--muted-foreground));
    }
    .loc {
      color: var(--muted-foreground);
      white-space: nowrap;
    }
    .rule {
      color: var(--primary);
      margin-right: 8px;
    }
    .empty {
      padding: var(--space-5, 20px);
      text-align: center;
      color: var(--muted-foreground);
    }
  `;

  @state() private _findings: Finding[] = [];
  @state() private _viewName: string | null = null;
  @state() private _loading = false;
  @state() private _error: string | null = null;

  private _onNavigate = (): void => {
    void this._refresh();
  };
  private _onDockActivate = (ev: Event): void => {
    const tab = (ev as CustomEvent<{ tab: string }>).detail?.tab;
    if (tab === "lint") void this._refresh();
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
    void this._refresh();
  }

  private _topLevelName(): string | null {
    const p = window.ship?.current().path ?? location.pathname;
    const m = p.match(/^\/api\/v\/([^/]+)/);
    if (m) return m[1];
    const seg = p.replace(/^\/+/, "").split("/")[0];
    return seg || null;
  }

  private async _refresh(): Promise<void> {
    const name = this._topLevelName();
    this._viewName = name;
    if (!name) {
      this._findings = [];
      return;
    }
    this._loading = true;
    this._error = null;
    try {
      const resp = await fetch("/api/lint/view", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name }),
      });
      if (resp.status === 404) {
        this._findings = [];
        return;
      }
      if (!resp.ok) {
        this._error = `lint failed: ${resp.status}`;
        return;
      }
      const body = (await resp.json()) as {
        ok: boolean;
        data?: { findings: Finding[] };
        error?: string;
      };
      if (!body.ok) {
        this._error = body.error ?? "lint failed";
        return;
      }
      this._findings = body.data?.findings ?? [];
    } catch (e) {
      this._error = e instanceof Error ? e.message : String(e);
    } finally {
      this._loading = false;
    }
  }

  private _sortedFindings(): Finding[] {
    const sev = (s: string): number =>
      s === "error" ? 0 : s === "warn" ? 1 : 2;
    return [...this._findings].sort((a, b) => {
      const s = sev(a.severity) - sev(b.severity);
      if (s !== 0) return s;
      if (a.file !== b.file) return a.file.localeCompare(b.file);
      if (a.line !== b.line) return a.line - b.line;
      return a.col - b.col;
    });
  }

  render() {
    const counts = this._findings.reduce(
      (acc, f) => {
        if (f.severity === "error") acc.err++;
        else if (f.severity === "warn") acc.warn++;
        return acc;
      },
      { err: 0, warn: 0 },
    );
    const findings = this._sortedFindings();
    return html`
      <div class="toolbar">
        ${this._viewName
          ? html`<span class="name">${this._viewName}</span>`
          : html`<span>No view</span>`}
        <span style="flex:1"></span>
        ${counts.err > 0
          ? html`<span class="count err">${counts.err} error</span>`
          : nothing}
        ${counts.warn > 0
          ? html`<span class="count warn">${counts.warn} warn</span>`
          : nothing}
        ${this._loading ? html`<span>Linting…</span>` : nothing}
      </div>
      <div class="body">
        ${this._error
          ? html`<div class="empty">${this._error}</div>`
          : findings.length === 0
            ? html`<div class="empty">
                ${this._viewName
                  ? "No findings."
                  : "Navigate to a view to lint it."}
              </div>`
            : findings.map(
                (f) => html`
                  <div class="row">
                    <span
                      class="sev ${f.severity === "error" ? "err" : "warn"}"
                    >
                      ${f.severity}
                    </span>
                    <span class="loc">${f.file}:${f.line}:${f.col}</span>
                    <span>
                      <span class="rule">[${f.rule}]</span>${f.message}
                    </span>
                  </div>
                `,
              )}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-lint": ShipDevtoolsLint;
  }
}
