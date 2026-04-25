//! `<ship-devtools-source>` — Source tab body for the panel dock.
//!
//! Phase 1: read-only pane showing the mounted view's `index.html.j2`
//! + manifest from `/api/view-source/{*path}`.
//!
//! Phase 2: edit mode. For user-scoped views the template editor is
//! editable; ⌘/Ctrl+S (or the Save button) PUTs to
//! `/api/views/{name}/template`. Lint diagnostics on failure render
//! inline. Project-scoped views show a "Fork to edit" affordance
//! which calls `POST /api/views/{name}/fork` and reloads the source.
//!
//! Requires the `code-editor` pack to be loaded before first render.

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
    .actions {
      display: inline-flex;
      align-items: center;
      gap: 6px;
    }
    button.edit-action {
      appearance: none;
      font: inherit;
      font-size: var(--text-xs);
      padding: 2px 10px;
      border-radius: var(--radius-sm, 4px);
      border: 1px solid var(--border);
      background: var(--background);
      color: var(--foreground);
      cursor: pointer;
    }
    button.edit-action.primary {
      background: var(--primary);
      color: var(--primary-foreground, #fff);
      border-color: var(--primary);
    }
    button.edit-action:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    .dirty-dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: var(--primary);
      display: inline-block;
    }
    .diagnostics {
      padding: 4px var(--space-3, 12px);
      background: color-mix(in oklch, var(--destructive) 10%, transparent);
      color: var(--destructive);
      font-size: var(--text-xs);
      font-family: var(--font-mono);
      border-top: 1px solid var(--destructive);
      max-height: 120px;
      overflow: auto;
    }
  `;

  @state() private _src: ViewSource | null = null;
  @state() private _error: string | null = null;
  @state() private _loading = false;
  @state() private _draft: string | null = null;
  @state() private _saving = false;
  @state() private _saveError: string | null = null;
  @state() private _diagnostics: Array<{
    line: number;
    col: number;
    rule: string;
    message: string;
  }> = [];

  private _completionSources: Array<{
    trigger: string;
    items: Array<{ label: string; detail?: string; info?: string; insert?: string }>;
  }> | null = null;

  private _editorEl(): (HTMLElement & {
    setDiagnostics?: (d: Array<{
      line: number;
      column?: number;
      severity: "error" | "warning" | "info";
      message: string;
      source?: string;
    }>) => void;
    setCompletions?: (
      s: Array<{
        trigger: string;
        items: Array<{ label: string; detail?: string; info?: string; insert?: string }>;
      }>,
    ) => void;
  }) | null {
    return this.renderRoot.querySelector(
      'ship-code-editor[language="html"]',
    ) as (HTMLElement & {
      setDiagnostics?: (d: Array<{
        line: number;
        column?: number;
        severity: "error" | "warning" | "info";
        message: string;
        source?: string;
      }>) => void;
    }) | null;
  }

  private _pushEditorDiagnostics(
    findings: Array<{
      line: number;
      col: number;
      rule: string;
      message: string;
      severity?: string;
    }>,
  ): void {
    const ed = this._editorEl();
    if (!ed?.setDiagnostics) return;
    const mapped = findings.map((f) => ({
      line: f.line,
      column: f.col,
      severity:
        f.severity === "warn" || f.severity === "warning"
          ? ("warning" as const)
          : f.severity === "info"
            ? ("info" as const)
            : ("error" as const),
      message: f.message,
      source: f.rule,
    }));
    ed.setDiagnostics(mapped);
  }

  private _clearEditorDiagnostics(): void {
    const ed = this._editorEl();
    ed?.setDiagnostics?.([]);
  }

  private async _loadCompletionSources(): Promise<
    Array<{
      trigger: string;
      items: Array<{
        label: string;
        detail?: string;
        info?: string;
        insert?: string;
      }>;
    }>
  > {
    if (this._completionSources) return this._completionSources;
    const [macros, schema] = await Promise.all([
      fetch("/api/assets/jinja-macros.json", { credentials: "same-origin" })
        .then((r) => (r.ok ? r.json() : null))
        .catch(() => null),
      fetch("/api/ship/schema", { credentials: "same-origin" })
        .then((r) => (r.ok ? r.json() : null))
        .catch(() => null),
    ]);

    const sources: Array<{
      trigger: string;
      items: Array<{
        label: string;
        detail?: string;
        info?: string;
        insert?: string;
      }>;
    }> = [];

    if (Array.isArray(macros)) {
      const items = (
        macros as Array<{
          name: string;
          signature?: string;
          summary?: string;
          description?: string;
        }>
      ).map((m) => ({
        label: m.name,
        detail: m.signature,
        info: m.summary ?? m.description,
        insert: `${m.name}()`,
      }));
      // Both {{ }} and {% %} blocks can reference macros, so register
      // the same items under two triggers.
      sources.push({ trigger: "{{ ", items });
      sources.push({ trigger: "{% ", items });
    }

    if (
      schema &&
      (schema as { data?: { ops?: unknown[] } }).data?.ops &&
      Array.isArray((schema as { data: { ops: unknown[] } }).data.ops)
    ) {
      const ops = (
        schema as {
          data: {
            ops: Array<{
              name: string;
              description?: string;
              returns?: string;
              permission?: string;
              args?: Array<{ name: string; kind: string; required?: boolean }>;
            }>;
          };
        }
      ).data.ops;
      const items = ops.map((op) => {
        const argList =
          op.args
            ?.map((a) => `${a.name}${a.required ? "" : "?"}: ${a.kind}`)
            .join(", ") ?? "";
        return {
          label: op.name,
          detail: `→ ${op.returns ?? "?"}`,
          info: `${op.description ?? ""}\n\nargs: ${argList}\nperm: ${op.permission ?? "?"}`,
          insert: `${op.name}()`,
        };
      });
      sources.push({ trigger: "ship.", items });
      sources.push({ trigger: "ship.try.", items });
    }

    this._completionSources = sources;
    return sources;
  }

  private async _wireCompletions(): Promise<void> {
    const ed = this._editorEl();
    if (!ed?.setCompletions) return;
    const sources = await this._loadCompletionSources();
    ed.setCompletions(sources);
  }

  private async _loadLintMarkers(viewName: string): Promise<void> {
    try {
      const resp = await fetch("/api/lint/view", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: viewName }),
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        ok: boolean;
        data?: {
          findings: Array<{
            line: number;
            col: number;
            rule: string;
            message: string;
            severity?: string;
            file: string;
          }>;
        };
      };
      // Only markers for the template file we're showing. Nested
      // partials surface in the Lint tab instead.
      const templateFile = this._src?.template_path ?? "";
      const relevant =
        body.data?.findings.filter((f) =>
          templateFile.endsWith(f.file) || f.file.endsWith(templateFile.split("/").pop() ?? ""),
        ) ?? [];
      // Wait a frame so the editor has mounted before we push.
      requestAnimationFrame(() => this._pushEditorDiagnostics(relevant));
    } catch {
      /* best-effort */
    }
  }

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
      this._draft = null;
      this._diagnostics = [];
      return;
    }
    if (stripped === this._lastPath && this._src) return;
    this._lastPath = stripped;
    this._draft = null;
    this._diagnostics = [];
    this._saveError = null;
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
      // Fire-and-forget lint on load so gutter markers show up even
      // before the first save attempt. Only for editable views —
      // bundled/project views already show read-only.
      if (this._src.scope === "global") {
        void this._loadLintMarkers(this._src.view_name.split(".")[0]);
        // Wait a frame so the editor has mounted before we inject
        // completion sources.
        requestAnimationFrame(() => void this._wireCompletions());
      }
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

  private get _editable(): boolean {
    return this._src?.scope === "global";
  }

  private get _forkable(): boolean {
    return this._src?.scope === "project";
  }

  private get _dirty(): boolean {
    return (
      this._draft !== null && this._src !== null && this._draft !== this._src.template
    );
  }

  private _templateRelPath(): string {
    const src = this._src;
    if (!src) return "index.html.j2";
    // Derive the view-relative path from the absolute template_path by
    // trimming everything up to and including the view name dir.
    const marker = `/views/${src.view_name.split(".")[0]}/`;
    const idx = src.template_path.indexOf(marker);
    if (idx < 0) return "index.html.j2";
    return src.template_path.slice(idx + marker.length);
  }

  private _onTemplateChange = (e: Event): void => {
    const detail = (e as CustomEvent<{ value: string }>).detail;
    if (typeof detail?.value === "string") this._draft = detail.value;
  };

  private _onTemplateSave = (e: Event): void => {
    const detail = (e as CustomEvent<{ value: string }>).detail;
    if (typeof detail?.value === "string") this._draft = detail.value;
    void this._save();
  };

  private async _save(): Promise<void> {
    if (!this._src || !this._editable || this._draft === null || !this._dirty) {
      return;
    }
    this._saving = true;
    this._saveError = null;
    this._diagnostics = [];
    try {
      const viewName = this._src.view_name.split(".")[0];
      const resp = await fetch(
        `/api/views/${encodeURIComponent(viewName)}/template`,
        {
          method: "PUT",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            file: this._templateRelPath(),
            content: this._draft,
          }),
        },
      );
      if (resp.status === 422) {
        const body = (await resp.json()) as {
          findings: Array<{
            line: number;
            col: number;
            rule: string;
            message: string;
            severity?: string;
          }>;
        };
        this._diagnostics = body.findings ?? [];
        this._pushEditorDiagnostics(this._diagnostics);
        this._saveError = "Lint errors — fix before saving.";
        return;
      }
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        this._saveError = `save failed: ${resp.status} ${text}`;
        return;
      }
      // Commit the draft to baseline and force a view re-render so the
      // user sees their change immediately.
      this._src = { ...this._src, template: this._draft };
      this._diagnostics = [];
      this._clearEditorDiagnostics();
      window.ship?.notify?.("Saved", "success");
      const current = window.ship?.current();
      if (current?.path) {
        window.ship?.navigate(current.path, { replace: true });
      }
    } catch (e) {
      this._saveError = e instanceof Error ? e.message : String(e);
    } finally {
      this._saving = false;
    }
  }

  private async _fork(): Promise<void> {
    if (!this._src || !this._forkable) return;
    const viewName = this._src.view_name.split(".")[0];
    this._saving = true;
    this._saveError = null;
    try {
      const resp = await fetch(
        `/api/views/${encodeURIComponent(viewName)}/fork`,
        { method: "POST", credentials: "same-origin" },
      );
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        this._saveError = `fork failed: ${resp.status} ${text}`;
        return;
      }
      window.ship?.notify?.(`${viewName} forked — now editable`, "success");
      // Refresh so the source-api picks up the user-scope copy.
      this._lastPath = null;
      await this._maybeRefresh();
    } catch (e) {
      this._saveError = e instanceof Error ? e.message : String(e);
    } finally {
      this._saving = false;
    }
  }

  private _manifestLang(name: string | null): string {
    if (!name) return "";
    if (name.endsWith(".yaml") || name.endsWith(".yml")) return "yaml";
    if (name.endsWith(".jsonc")) return "jsonc";
    return "json";
  }

  render() {
    if (this._loading && !this._src) {
      return html`<ship-empty-state>
        <ship-icon slot="icon" name="file-code"></ship-icon>
        <span slot="title">Loading source…</span>
        Fetching the view template from the daemon.
      </ship-empty-state>`;
    }
    if (this._error) {
      return html`<ship-empty-state>
        <ship-icon slot="icon" name="alert-triangle"></ship-icon>
        <span slot="title">Source error</span>
        ${this._error}
      </ship-empty-state>`;
    }
    const src = this._src;
    if (!src) {
      return html`<ship-empty-state>
        <ship-icon slot="icon" name="file-code"></ship-icon>
        <span slot="title">No source available</span>
        This path is not served by a file-routed view.
      </ship-empty-state>`;
    }
    const dirty = this._dirty;
    const displayValue = this._draft ?? src.template;
    return html`
      <header>
        <span class="name">${src.view_name}</span>
        <span class="scope">${src.scope}</span>
        ${dirty ? html`<span class="dirty-dot" title="Unsaved changes"></span>` : nothing}
        <span style="flex:1"></span>
        <span>${src.template_path}</span>
        <span class="actions">
          ${this._forkable
            ? html`<button
                class="edit-action primary"
                ?disabled=${this._saving}
                @click=${() => void this._fork()}
              >
                Fork to edit
              </button>`
            : nothing}
          ${this._editable
            ? html`<button
                class="edit-action primary"
                ?disabled=${this._saving || !dirty}
                @click=${() => void this._save()}
                title="Save (⌘/Ctrl+S)"
              >
                ${this._saving ? "Saving…" : "Save"}
              </button>`
            : nothing}
        </span>
      </header>
      <div class="split">
        <div class="pane">
          <div class="pane-head">index.html.j2</div>
          <ship-code-editor
            language="html"
            ?readonly=${!this._editable}
            .value=${displayValue}
            @ship-change=${this._onTemplateChange}
            @ship-save=${this._onTemplateSave}
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
      ${this._saveError || this._diagnostics.length > 0
        ? html`<div class="diagnostics">
            ${this._saveError
              ? html`<div>${this._saveError}</div>`
              : nothing}
            ${this._diagnostics.map(
              (d) =>
                html`<div>${d.line}:${d.col} [${d.rule}] ${d.message}</div>`,
            )}
          </div>`
        : nothing}
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-devtools-source": ShipDevtoolsSource;
  }
}
