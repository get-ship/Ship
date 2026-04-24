//! `<ship-rail>` — macOS-dock-style pinned view launcher.
//!
//! Light-DOM, uses the existing `.ship-rail` stylesheet from the shell's
//! index.html. Pin state lives in daemon settings (`shell.rail.pinned`);
//! display metadata (label, icon) comes from each view's manifest via
//! `/api/views`. Subscribes to `ship:navigate` on `<ship-shell>` to
//! highlight the active item. Right-click on an item unpins. A bottom
//! "+" button dispatches `ship:launcher:open`.

import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

import { fetchPins, unpin } from "./pins";
import { humanize } from "./view-catalog";

interface ApiView {
  name: string;
  type: string;
  source: string;
  rail?: { label: string; icon: string };
}

@customElement("ship-rail")
export class ShipRail extends LitElement {
  @state() private pins: string[] = [];
  @state() private active = "";
  @state() private viewsByName: Map<string, ApiView> = new Map();
  @state() private activeSessionCount = 0;
  @state() private projects: Array<{ id: string; name: string; path: string; label?: string }> = [];
  @state() private activeProject: string | null = null;
  @state() private projectSwitcherOpen = false;
  private _sessionPoll: ReturnType<typeof setInterval> | null = null;

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  private onNavigate = (e: Event): void => {
    const to = (e as CustomEvent).detail?.to as string | undefined;
    if (!to) return;
    this.active = this.routeToViewName(to);
  };

  private onPinsChanged = (e: Event): void => {
    const detail = (e as CustomEvent).detail as { pins?: string[] } | undefined;
    if (Array.isArray(detail?.pins)) this.pins = detail.pins;
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.active = this.routeToViewName(location.pathname);
    this.classList.add("ship-rail");
    this.shell?.addEventListener("ship:navigate", this.onNavigate);
    this.shell?.addEventListener(
      "ship:rail:pins-changed" as keyof HTMLElementEventMap,
      this.onPinsChanged,
    );
    window.addEventListener("ship:session:changed", this.onSessionChanged);
    void this.load();
    void this.pollSessions();
    void this.loadProjects();
    this._sessionPoll = setInterval(() => void this.pollSessions(), 15000);
  }

  private async loadProjects(): Promise<void> {
    try {
      const resp = await fetch("/api/projects", {
        credentials: "same-origin",
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        projects: Array<{ id: string; name: string; path: string; label?: string }>;
        active?: string;
      };
      this.projects = body.projects;
      this.activeProject = body.active ?? null;
    } catch {
      /* empty */
    }
  }

  private async switchProject(id: string): Promise<void> {
    try {
      await fetch("/api/projects/switch", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id }),
      });
      this.activeProject = id;
      this.projectSwitcherOpen = false;
      this.shell?.dispatchEvent(
        new CustomEvent("ship:project:change", {
          detail: { id },
          bubbles: true,
        }),
      );
    } catch {
      /* empty */
    }
  }

  private _currentProject(): { id: string; name: string; path: string; label?: string } | null {
    return (
      this.projects.find((p) => p.id === this.activeProject) ??
      this.projects[0] ??
      null
    );
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:navigate", this.onNavigate);
    this.shell?.removeEventListener(
      "ship:rail:pins-changed" as keyof HTMLElementEventMap,
      this.onPinsChanged,
    );
    window.removeEventListener("ship:session:changed", this.onSessionChanged);
    if (this._sessionPoll) clearInterval(this._sessionPoll);
  }

  private onSessionChanged = (): void => {
    void this.pollSessions();
  };

  private async pollSessions(): Promise<void> {
    try {
      const resp = await fetch("/api/pty-sessions", {
        credentials: "same-origin",
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        sessions: Array<{ status: string }>;
      };
      this.activeSessionCount = body.sessions.filter(
        (s) => s.status !== "ended",
      ).length;
    } catch {
      /* leave previous count */
    }
  }

  private onChatsClick(e: MouseEvent): void {
    e.preventDefault();
    window.ship?.navigate("/chats");
  }

  createRenderRoot(): Element {
    return this;
  }

  private async load(): Promise<void> {
    const [pins, views] = await Promise.all([fetchPins(), fetchViews()]);
    this.pins = pins;
    this.viewsByName = new Map(views.map((v) => [v.name, v]));
  }

  private routeToViewName(path: string): string {
    const seg = path.split("/").filter(Boolean)[0] ?? "dashboard";
    return seg === "" ? "dashboard" : seg;
  }

  private onItemClick(e: MouseEvent, name: string): void {
    e.preventDefault();
    window.ship?.navigate(`/${name === "dashboard" ? "" : name}`);
  }

  private async onItemContext(e: MouseEvent, name: string): Promise<void> {
    e.preventDefault();
    this.pins = await unpin(name, this.pins);
    this.shell?.dispatchEvent(
      new CustomEvent("ship:rail:pins-changed", {
        detail: { pins: this.pins },
        bubbles: true,
      }),
    );
  }

  private openLauncher(e: MouseEvent): void {
    e.preventDefault();
    const anchor = e.currentTarget as HTMLElement;
    this.shell?.dispatchEvent(
      new CustomEvent("ship:launcher:open", {
        detail: { anchor },
        bubbles: true,
      }),
    );
  }

  private openCreate(e: MouseEvent): void {
    e.preventDefault();
    this.shell?.dispatchEvent(
      new CustomEvent("ship:create:open", { bubbles: true }),
    );
  }

  render() {
    const settings = this.pins.filter((n) => n === "settings");
    const rest = this.pins.filter((n) => n !== "settings");
    const cur = this._currentProject();
    return html`
      <div class="rail-project" style="position: relative">
        <a
          class="rail-item rail-project-badge"
          title=${cur ? `Project: ${cur.label ?? cur.name}` : "No project"}
          aria-label="Switch project"
          href="#"
          @click=${(e: MouseEvent) => {
            e.preventDefault();
            this.projectSwitcherOpen = !this.projectSwitcherOpen;
          }}
        >
          <span class="rail-project-initial">
            ${(cur?.label ?? cur?.name ?? "?").charAt(0).toUpperCase()}
          </span>
        </a>
        ${this.projectSwitcherOpen ? this._renderProjectPopover() : ""}
      </div>
      ${rest.map((name) => this.renderItem(name))}
      <a
        class="rail-item rail-new"
        title="Create new…"
        aria-label="Create new feature"
        href="#"
        @click=${(e: MouseEvent) => this.openCreate(e)}
      >
        <ship-icon name="plus"></ship-icon>
      </a>
      <a
        class="rail-item rail-chats${this.active === "chats" ? " active" : ""}"
        title="Chats (${this.activeSessionCount} active)"
        aria-label="Mission Control"
        href="/chats"
        data-badge=${this.activeSessionCount > 0
          ? String(this.activeSessionCount)
          : ""}
        @click=${(e: MouseEvent) => this.onChatsClick(e)}
      >
        <ship-icon name="messages-square"></ship-icon>
      </a>
      <div class="rail-spacer"></div>
      <a
        class="rail-launcher"
        title="Launcher"
        aria-label="Open app launcher"
        href="#"
        @click=${(e: MouseEvent) => this.openLauncher(e)}
      >
        <ship-icon name="grid"></ship-icon>
      </a>
      ${settings.length > 0 ? settings.map((name) => this.renderItem(name)) : nothing}
    `;
  }

  private _renderProjectPopover() {
    return html`
      <div class="rail-project-popover"
           @click=${(e: MouseEvent) => e.stopPropagation()}>
        <div class="rail-project-title">Projects</div>
        ${this.projects.length === 0
          ? html`<div class="rail-project-empty">No projects tracked.</div>`
          : this.projects.map(
              (p) => html`
                <div
                  class="rail-project-row ${p.id === this.activeProject ? "active" : ""}"
                  @click=${() => void this.switchProject(p.id)}
                >
                  <span class="rail-project-row-name">${p.label ?? p.name}</span>
                  <span class="rail-project-row-path">${p.path}</span>
                </div>
              `,
            )}
        <div class="rail-project-sep"></div>
        <div class="rail-project-row add" @click=${() => this._addProjectFlow()}>
          <span>+ Add project</span>
        </div>
      </div>
    `;
  }

  private async _addProjectFlow(): Promise<void> {
    const path = window.prompt("Path to project directory:");
    if (!path) return;
    try {
      const resp = await fetch("/api/projects", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path }),
      });
      if (!resp.ok) {
        const err = await resp.text().catch(() => "");
        window.ship?.notify?.(`Add failed: ${err || resp.status}`, "warning");
        return;
      }
      const body = (await resp.json()) as {
        projects: Array<{ id: string; name: string; path: string; label?: string }>;
        active?: string;
      };
      this.projects = body.projects;
      this.activeProject = body.active ?? null;
      window.ship?.notify?.("Project added.", "success");
    } catch (e) {
      window.ship?.notify?.(String(e), "destructive");
    }
  }

  private renderItem(name: string) {
    const v = this.viewsByName.get(name);
    const label = v?.rail?.label ?? humanize(name);
    const icon = v?.rail?.icon ?? "app-window";
    const classes = `rail-item${this.active === name ? " active" : ""}`;
    return html`
      <a
        class=${classes}
        title=${label}
        data-view=${name}
        href=${`/${name === "dashboard" ? "" : name}`}
        @click=${(e: MouseEvent) => this.onItemClick(e, name)}
        @contextmenu=${(e: MouseEvent) => this.onItemContext(e, name)}
      >
        <ship-icon name=${icon}></ship-icon>
      </a>
    `;
  }
}

async function fetchViews(): Promise<ApiView[]> {
  try {
    const resp = await fetch("/api/views", { credentials: "same-origin" });
    if (!resp.ok) return [];
    const body = (await resp.json()) as { data?: { views?: ApiView[] } };
    return body.data?.views ?? [];
  } catch {
    return [];
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-rail": ShipRail;
  }
}
