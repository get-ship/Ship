//! `<ship-create-dialog>` — modal dialog triggered by the rail "+"
//! tile. Collects feature type, name, purpose, starting shape, and
//! optional data sources; POSTs to `/api/create/{surface}`.
//!
//! On success the server scaffolds the feature and emits kernel
//! events shipwright tails; the dialog closes, the chat drawer opens
//! showing the session, and the shell receives a toast when the agent
//! is done.

import { LitElement, css, html } from "lit";
import { customElement, state } from "lit/decorators.js";

type Surface =
  | "view"
  | "skill"
  | "agent"
  | "pipeline"
  | "trigger"
  | "theme";

interface SurfaceDef {
  id: Surface;
  label: string;
  icon: string;
  shapes: string[];
  shapeLabel: string;
}

const SURFACES: SurfaceDef[] = [
  {
    id: "view",
    label: "View",
    icon: "app-window",
    shapes: ["list", "form", "detail", "dashboard", "tabs", "blank"],
    shapeLabel: "Starting shape",
  },
  {
    id: "skill",
    label: "Skill",
    icon: "sparkles",
    shapes: ["task", "reference", "workflow"],
    shapeLabel: "Skill kind",
  },
  {
    id: "agent",
    label: "Agent",
    icon: "bot",
    shapes: ["specialist", "generalist", "reviewer"],
    shapeLabel: "Agent kind",
  },
  {
    id: "pipeline",
    label: "Pipeline",
    icon: "workflow",
    shapes: ["sequential", "fanout", "review-gated"],
    shapeLabel: "Topology",
  },
  {
    id: "trigger",
    label: "Trigger",
    icon: "zap",
    shapes: ["webhook", "schedule", "file-change"],
    shapeLabel: "Kind",
  },
  {
    id: "theme",
    label: "Theme",
    icon: "palette",
    shapes: ["light", "dark", "brand-accent", "high-contrast"],
    shapeLabel: "Starting shape",
  },
];

const KEBAB_RE = /^[a-z][a-z0-9-]{0,48}$/;

@customElement("ship-create-dialog")
export class ShipCreateDialog extends LitElement {
  static styles = css`
    :host {
      display: contents;
    }
    dialog {
      border: 1px solid var(--border);
      border-radius: var(--radius-lg, 10px);
      background: var(--card, var(--background));
      color: var(--foreground);
      padding: 0;
      width: min(520px, 94vw);
      max-height: 86vh;
      box-shadow: 0 16px 48px rgba(0, 0, 0, 0.25);
      font-family: var(--font-body);
    }
    dialog::backdrop {
      background: rgba(0, 0, 0, 0.45);
    }
    header {
      padding: var(--space-4, 16px);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      gap: var(--space-2, 8px);
    }
    header h2 {
      margin: 0;
      font-size: var(--text-base);
      font-weight: var(--font-weight-medium, 500);
    }
    form {
      padding: var(--space-4, 16px);
      display: grid;
      gap: var(--space-4, 16px);
      max-height: calc(86vh - 120px);
      overflow: auto;
    }
    label {
      display: grid;
      gap: 4px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    input[type="text"],
    select,
    textarea {
      font: inherit;
      font-size: var(--text-sm);
      color: var(--foreground);
      background: var(--background);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm, 4px);
      padding: 6px 10px;
    }
    textarea {
      min-height: 60px;
      resize: vertical;
    }
    .surfaces {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 8px;
    }
    .surface {
      appearance: none;
      background: var(--background);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      padding: 10px 8px;
      cursor: pointer;
      display: grid;
      justify-items: center;
      gap: 6px;
      font: inherit;
      font-size: var(--text-xs);
    }
    .surface[aria-pressed="true"] {
      border-color: var(--primary);
      background: color-mix(in oklch, var(--primary) 10%, transparent);
      color: var(--primary);
    }
    footer {
      display: flex;
      justify-content: flex-end;
      gap: var(--space-2, 8px);
      padding: var(--space-4, 16px);
      border-top: 1px solid var(--border);
    }
    button.primary,
    button.secondary {
      appearance: none;
      font: inherit;
      font-size: var(--text-sm);
      padding: 6px 14px;
      border-radius: var(--radius-sm, 4px);
      cursor: pointer;
    }
    button.primary {
      background: var(--primary);
      color: var(--primary-foreground, #fff);
      border: 1px solid var(--primary);
    }
    button.primary:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    button.secondary {
      background: transparent;
      color: var(--muted-foreground);
      border: 1px solid var(--border);
    }
    .error {
      color: var(--destructive);
      font-size: var(--text-xs);
    }
  `;

  @state() private _surface: Surface = "view";
  @state() private _name = "";
  @state() private _purpose = "";
  @state() private _shape = "list";
  @state() private _submitting = false;
  @state() private _error: string | null = null;

  private _onShellOpen = (): void => {
    this.open();
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.closest("ship-shell")?.addEventListener(
      "ship:create:open",
      this._onShellOpen,
    );
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.closest("ship-shell")?.removeEventListener(
      "ship:create:open",
      this._onShellOpen,
    );
  }

  open(): void {
    const dlg = this.renderRoot.querySelector("dialog") as
      | HTMLDialogElement
      | null;
    this._error = null;
    dlg?.showModal();
  }

  private _close(): void {
    (this.renderRoot.querySelector("dialog") as HTMLDialogElement | null)?.close();
  }

  private _def(): SurfaceDef {
    return SURFACES.find((s) => s.id === this._surface)!;
  }

  private _pickSurface(id: Surface): void {
    this._surface = id;
    const def = this._def();
    if (!def.shapes.includes(this._shape)) {
      this._shape = def.shapes[0];
    }
  }

  private _nameValid(): boolean {
    return KEBAB_RE.test(this._name);
  }

  private async _submit(e: Event): Promise<void> {
    e.preventDefault();
    if (!this._nameValid()) {
      this._error = "Name must be kebab-case: a–z, 0–9, and - only.";
      return;
    }
    this._submitting = true;
    this._error = null;
    try {
      const resp = await fetch(
        `/api/create/${encodeURIComponent(this._surface)}`,
        {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: this._name,
            purpose: this._purpose,
            shape: this._shape,
          }),
        },
      );
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        this._error = `create failed: ${resp.status} ${text}`;
        return;
      }
      const body = (await resp.json()) as {
        ok?: boolean;
        data?: { session_id?: string; workspace_id?: string };
        error?: string;
      };
      if (body.ok === false) {
        this._error = body.error ?? "create failed";
        return;
      }
      // Fire a kernel-free shell event so the chat drawer can open onto
      // the new session.
      this.closest("ship-shell")?.dispatchEvent(
        new CustomEvent("ship:create:submitted", {
          detail: {
            surface: this._surface,
            name: this._name,
            session_id: body.data?.session_id,
            workspace_id: body.data?.workspace_id,
          },
          bubbles: true,
        }),
      );
      window.ship?.chat.open({});
      window.ship?.notify?.(`Shipwright is creating ${this._name}…`, "info");
      this._close();
      this._reset();
    } catch (e) {
      this._error = e instanceof Error ? e.message : String(e);
    } finally {
      this._submitting = false;
    }
  }

  private _reset(): void {
    this._name = "";
    this._purpose = "";
    this._shape = "list";
    this._surface = "view";
  }

  render() {
    const def = this._def();
    return html`
      <dialog @cancel=${() => this._close()}>
        <header>
          <ship-icon name="plus" size="sm"></ship-icon>
          <h2>Create new</h2>
        </header>
        <form @submit=${(e: Event) => this._submit(e)}>
          <label>
            <span>Feature type</span>
            <div class="surfaces">
              ${SURFACES.map(
                (s) => html`
                  <button
                    type="button"
                    class="surface"
                    aria-pressed=${this._surface === s.id ? "true" : "false"}
                    @click=${() => this._pickSurface(s.id)}
                  >
                    <ship-icon name=${s.icon}></ship-icon>
                    <span>${s.label}</span>
                  </button>
                `,
              )}
            </div>
          </label>
          <label>
            <span>Name (kebab-case)</span>
            <input
              type="text"
              required
              placeholder="my-${def.id}"
              .value=${this._name}
              @input=${(e: Event) =>
                (this._name = (e.target as HTMLInputElement).value)}
            />
          </label>
          <label>
            <span>Purpose</span>
            <textarea
              placeholder="One-liner: what this ${def.id} does."
              .value=${this._purpose}
              @input=${(e: Event) =>
                (this._purpose = (e.target as HTMLTextAreaElement).value)}
            ></textarea>
          </label>
          <label>
            <span>${def.shapeLabel}</span>
            <select
              .value=${this._shape}
              @change=${(e: Event) =>
                (this._shape = (e.target as HTMLSelectElement).value)}
            >
              ${def.shapes.map(
                (s) => html`<option value=${s}>${s}</option>`,
              )}
            </select>
          </label>
          ${this._error
            ? html`<div class="error">${this._error}</div>`
            : ""}
          <footer>
            <button
              type="button"
              class="secondary"
              @click=${() => this._close()}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="primary"
              ?disabled=${this._submitting || !this._nameValid()}
            >
              ${this._submitting ? "Creating…" : "Create"}
            </button>
          </footer>
        </form>
      </dialog>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-create-dialog": ShipCreateDialog;
  }
}
