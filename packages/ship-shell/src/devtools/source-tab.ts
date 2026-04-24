//! `<ship-devtools-source>` — read-only Source tab body for the
//! panel dock. Shows the mounted view's `index.html.j2` + manifest
//! file contents straight off disk.
//!
//! Subscribes to shell navigation (`ship:navigate` on `<ship-shell>`)
//! + `ship:dock:activate` on the dock, fetching `/api/view-source/{*path}`
//! only when the Source tab is visible to avoid unnecessary work.
//!
//! Requires the `code-editor` pack to be loaded before first render.
//! The shell boots this pack at startup so Source is ready immediately.

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

interface ViewSource {
  view_name: string;
  template_path: string;
  template: string;
  manifest_filename: string | null;
  manifest: string | null;
  scope: string;
}

@customElement("ship-devtools-source")
export class ShipDevtoolsSource extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      min-height: 0;
      font-size: var(--text-sm);
    }
    header {
      display: flex;
      align-items: center;
      gap: var(--space-3, 12px);
      padding: var(--space-2, 8px) var(--space-3, 12px);
      border-bottom: 1px solid var(--border);
      color: var(--muted-foreground);
      font-size: var(--text-xs);
    }
    header .name {
      color: var(--foreground);
      font-weight: var(--font-weight-medium, 500);
    }
    header .scope {
      padding: 1px 6px;
      border: 1px solid var(--border);
      border-radius: 999px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      font-size: 10px;
    }
    .split {
      flex: 1;
      min-height: 0;
      display: grid;
      grid-template-columns: 1fr;
      grid-template-rows: 1fr auto minmax(0, 40%);
    }
    .pane {
      min-height: 0;
      display: flex;
      flex-direction: column;
    }
    .pane-head {
      display: flex;
      align-items: center;
      gap: var(--space-2, 8px);
      padding: 4px var(--space-3, 12px);
      background: var(--sidebar);
      border-bottom: 1px solid var(--border);
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      font-family: var(--font-mono);
    }
    .pane-head.manifest {
      border-top: 1px solid var(--border);
    }
    ship-code-editor {
      flex: 1;
      min-height: 0;
    }
    .empty {
      padding: var(--space-5, 20px);
      color: var(--muted-foreground);
      font-size: var(--text-sm);
    }
    .error {
      color: var(--destructive);
    }
  `;

  @state() private _src: ViewSource | null = null;
  @state() private _error: string | null = null;
  @state() private _loading = false;

  private _lastPath: string | null = null;
  private _onNavigate = (): void => {
    this._maybeRefresh();
  };

  connectedCallback(): void {
    super.connectedCallback();
    document
      .querySelector("ship-shell")
      ?.addEventListener("ship:navigate", this._onNavigate);
    void this._maybeRefresh();
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    document
      .querySelector("ship-shell")
      ?.removeEventListener("ship:navigate", this._onNavigate);
  }

  /** Force a refresh. Called by the dock on tab activation. */
  refresh(): void {
    this._lastPath = null;
    void this._maybeRefresh();
  }

  private async _maybeRefresh(): Promise<void> {
    const current = window.ship?.current();
    const path = current?.path ?? location.pathname;
    const stripped = this._stripPrefix(path);
    if (!stripped) {
      this._src = null;
      this._error = null;
      return;
    }
    if (stripped === this._lastPath && this._src) return;
    this._lastPath = stripped;
    this._loading = true;
    this._error = null;
    try {
      const resp = await fetch(
        `/api/view-source/${encodeURI(stripped)}`,
        { credentials: "same-origin" },
      );
      if (resp.status === 404) {
        this._src = null;
        this._error = null;
        return;
      }
      if (!resp.ok) {
        this._error = `fetch failed: ${resp.status}`;
        return;
      }
      this._src = (await resp.json()) as ViewSource;
    } catch (e) {
      this._error = e instanceof Error ? e.message : String(e);
    } finally {
      this._loading = false;
    }
  }

  /** Strip `/api/v/` prefix so only the view path remains. */
  private _stripPrefix(path: string): string {
    const m = path.match(/^\/api\/v\/(.+)$/);
    if (m) return m[1];
    // Shell routes can also expose views at root paths; fall back to the
    // trimmed pathname minus leading slash.
    return path.replace(/^\/+/, "");
  }

  private _manifestLang(name: string | null): string {
    if (!name) return "";
    if (name.endsWith(".yaml") || name.endsWith(".yml")) return "yaml";
    if (name.endsWith(".jsonc")) return "jsonc";
    return "json";
  }

  render() {
    if (this._loading && !this._src) {
      return html`<div class="empty">Loading source…</div>`;
    }
    if (this._error) {
      return html`<div class="empty error">${this._error}</div>`;
    }
    const src = this._src;
    if (!src) {
      return html`<div class="empty">
        No file-routed view at this path.
      </div>`;
    }
    return html`
      <header>
        <span class="name">${src.view_name}</span>
        <span class="scope">${src.scope}</span>
        <span style="flex:1"></span>
        <span>${src.template_path}</span>
      </header>
      <div class="split">
        <div class="pane">
          <div class="pane-head">index.html.j2</div>
          <ship-code-editor
            language="html"
            readonly
            .value=${src.template}
          ></ship-code-editor>
        </div>
        ${src.manifest
          ? html`
              <div class="pane-head manifest">
                ${src.manifest_filename}
              </div>
              <div class="pane">
                <ship-code-editor
                  language=${this._manifestLang(src.manifest_filename)}
                  readonly
                  .value=${src.manifest}
                ></ship-code-editor>
              </div>
            `
          : nothing}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-source": ShipDevtoolsSource;
  }
}
