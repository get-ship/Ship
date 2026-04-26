//! `<ship-launcher>` — popover showing all views, with pin/unpin,
//! fuzzy filter, keyboard nav, and drag-reorder of pinned rows.
//!
//! Anchored to the rail's brand button (`ship:launcher:open` carries
//! `{ anchor }`). Light-DOM element — its CSS lives in
//! `apps/shell/index.html` under the `.ship-launcher*` rules so the
//! existing token cascade applies without a Shadow DOM seam.
//!
//! Storage: pin order is `pins.ts` (a string[] in user settings).
//! Drag-reorder writes the new order through `savePins`.

import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

import { fetchPins, savePins } from "./pins";
import { humanize } from "./view-catalog";

interface ApiView {
  name: string;
  type: string;
  source: string;
  rail?: { label: string; icon: string };
}

// Match the rail's registry — known Ship apps render via <ship-app-icon>
// (per-app glyph tinted by --app-accent-{name}). Unknown view ids fall
// back to <ship-icon> using the manifest's rail.icon Lucide glyph.
const KNOWN_APP_NAMES = new Set([
  "dashboard", "pipelines", "agents", "skills", "git", "collections",
  "canvas", "settings", "chat", "events", "specs", "theme",
  "theme-studio", "setup", "overview", "mesh",
]);

interface Project {
  id: string;
  name: string;
  path: string;
  label?: string;
}

@customElement("ship-launcher")
export class ShipLauncher extends LitElement {
  @state() private isOpen = false;
  @state() private views: ApiView[] = [];
  @state() private pins: string[] = [];
  @state() private filter = "";
  @state() private cursor = 0;
  @state() private project: Project | null = null;
  @state() private dragName: string | null = null;
  @state() private dragOver: string | null = null;

  private filterTimer: ReturnType<typeof setTimeout> | null = null;
  private rafFocus = 0;

  /** Light-DOM render so global CSS in apps/shell/index.html applies. */
  createRenderRoot(): Element {
    return this;
  }

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  // ── public API ───────────────────────────────────────────────
  open(): void {
    if (this.isOpen) return;
    this.isOpen = true;
    this.setAttribute("open", "");
    this.cursor = 0;
    void this.load();
  }

  close(): void {
    if (!this.isOpen) return;
    this.isOpen = false;
    this.removeAttribute("open");
    this.filter = "";
  }

  // ── lifecycle ────────────────────────────────────────────────
  private onShellOpen = (e: Event): void => {
    void (e as CustomEvent).detail;
    this.open();
  };

  private onShellClose = (): void => {
    this.close();
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.shell?.addEventListener("ship:launcher:open", this.onShellOpen);
    this.shell?.addEventListener("ship:launcher:close", this.onShellClose);
    document.addEventListener("keydown", this.onGlobalKey);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:launcher:open", this.onShellOpen);
    this.shell?.removeEventListener("ship:launcher:close", this.onShellClose);
    document.removeEventListener("keydown", this.onGlobalKey);
    if (this.filterTimer) clearTimeout(this.filterTimer);
    if (this.rafFocus) cancelAnimationFrame(this.rafFocus);
  }

  private async load(): Promise<void> {
    const [pins, _viewsLoaded, project] = await Promise.all([
      fetchPins(),
      this.fetchViews(),
      this.fetchProject(),
    ]);
    void _viewsLoaded;
    this.pins = pins;
    this.project = project;
    // Auto-focus filter after render.
    this.rafFocus = requestAnimationFrame(() => {
      const inp = this.querySelector<HTMLInputElement>(".launcher-filter input");
      inp?.focus();
    });
  }

  private async fetchViews(): Promise<void> {
    try {
      const resp = await fetch("/api/views", { credentials: "same-origin" });
      if (!resp.ok) return;
      const body = (await resp.json()) as { data?: { views?: ApiView[] } };
      this.views = body.data?.views ?? [];
    } catch {
      this.views = [];
    }
  }

  private async fetchProject(): Promise<Project | null> {
    try {
      const resp = await fetch("/api/projects", { credentials: "same-origin" });
      if (!resp.ok) return null;
      const body = (await resp.json()) as {
        projects: Project[];
        active?: string;
      };
      return (
        body.projects.find((p) => p.id === body.active) ??
        body.projects[0] ??
        null
      );
    } catch {
      return null;
    }
  }

  // ── keyboard ─────────────────────────────────────────────────
  private onGlobalKey = (e: KeyboardEvent): void => {
    if (!this.isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      this.close();
      return;
    }
    if (e.key === "/" && !this.isFilterFocused()) {
      e.preventDefault();
      this.querySelector<HTMLInputElement>(".launcher-filter input")?.focus();
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      this.cursor = Math.min(this.visibleRows().length - 1, this.cursor + 1);
      this.scrollCursorIntoView();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      this.cursor = Math.max(0, this.cursor - 1);
      this.scrollCursorIntoView();
    } else if (e.key === "Enter") {
      const row = this.visibleRows()[this.cursor];
      if (!row) return;
      e.preventDefault();
      if (e.metaKey || e.ctrlKey) {
        this.dispatchEvent(
          new CustomEvent("ship:launcher:open-pane", {
            detail: { name: row.name },
            bubbles: true,
          }),
        );
      }
      this.navigate(row.name);
    }
  };

  private isFilterFocused(): boolean {
    const inp = this.querySelector<HTMLInputElement>(".launcher-filter input");
    return inp === document.activeElement;
  }

  private scrollCursorIntoView(): void {
    requestAnimationFrame(() => {
      const sel = this.querySelector<HTMLElement>(".launcher-row.is-cursor");
      sel?.scrollIntoView({ block: "nearest" });
    });
  }

  // ── data shaping ─────────────────────────────────────────────
  private match(v: ApiView): boolean {
    const f = this.filter.trim().toLowerCase();
    if (!f) return true;
    const label = (v.rail?.label ?? humanize(v.name)).toLowerCase();
    return label.includes(f) || v.source.toLowerCase().includes(f);
  }

  private pinnedRows(): ApiView[] {
    const map = new Map(this.views.map((v) => [v.name, v]));
    return this.pins
      .map((n) => map.get(n))
      .filter((v): v is ApiView => !!v && this.match(v));
  }

  private otherRows(): ApiView[] {
    const pinSet = new Set(this.pins);
    return this.views.filter((v) => !pinSet.has(v.name) && this.match(v));
  }

  private visibleRows(): ApiView[] {
    return [...this.pinnedRows(), ...this.otherRows()];
  }

  // ── actions ──────────────────────────────────────────────────
  private navigate(name: string): void {
    window.ship?.navigate(`/${name === "dashboard" ? "" : name}`);
    this.close();
  }

  private async togglePin(e: Event, name: string): Promise<void> {
    e.preventDefault();
    e.stopPropagation();
    const next = this.pins.includes(name)
      ? this.pins.filter((n) => n !== name)
      : [...this.pins, name];
    this.pins = next;
    await savePins(next);
    this.shell?.dispatchEvent(
      new CustomEvent("ship:rail:pins-changed", {
        detail: { pins: next },
        bubbles: true,
      }),
    );
  }

  private onFilterInput = (e: Event): void => {
    const v = (e.target as HTMLInputElement).value;
    if (this.filterTimer) clearTimeout(this.filterTimer);
    this.filterTimer = setTimeout(() => {
      this.filter = v;
      this.cursor = 0;
    }, 80);
  };

  // ── drag reorder of pinned rows ──────────────────────────────
  private onDragStart(e: DragEvent, name: string): void {
    this.dragName = name;
    e.dataTransfer?.setData("text/plain", name);
    if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
  }

  private onDragOver(e: DragEvent, name: string): void {
    if (!this.dragName) return;
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    if (this.dragOver !== name) this.dragOver = name;
  }

  private async onDrop(e: DragEvent, target: string): Promise<void> {
    e.preventDefault();
    const src = this.dragName;
    this.dragName = null;
    this.dragOver = null;
    if (!src || src === target) return;
    if (!this.pins.includes(src) || !this.pins.includes(target)) return;
    const without = this.pins.filter((n) => n !== src);
    const idx = without.indexOf(target);
    const next = [...without.slice(0, idx), src, ...without.slice(idx)];
    this.pins = next;
    await savePins(next);
    this.shell?.dispatchEvent(
      new CustomEvent("ship:rail:pins-changed", {
        detail: { pins: next },
        bubbles: true,
      }),
    );
  }

  private onDragEnd = (): void => {
    this.dragName = null;
    this.dragOver = null;
  };

  // ── render ───────────────────────────────────────────────────
  render() {
    if (!this.isOpen) return nothing;
    const pinned = this.pinnedRows();
    const others = this.otherRows();
    const total = pinned.length + others.length;
    return html`
      <div
        class="launcher-mask"
        @click=${() => this.close()}
        @dragover=${(e: DragEvent) => e.preventDefault()}
      ></div>
      <div
        class="launcher-panel"
        role="dialog"
        aria-label="App launcher"
        aria-modal="true"
        @click=${(e: Event) => e.stopPropagation()}
      >
        ${this.renderHead()}
        ${this.renderFilter(total)}
        <div class="launcher-body">
          ${this.renderSection("Pinned to rail", pinned, 0, true)}
          ${this.renderSection("All views", others, pinned.length, false)}
          ${total === 0 ? html`<div class="launcher-empty">No views match.</div>` : nothing}
        </div>
        ${this.renderFoot()}
      </div>
    `;
  }

  private renderHead() {
    const path = this.project?.path ?? "";
    const title = this.project?.label ?? this.project?.name ?? "Ship";
    return html`
      <div class="launcher-head">
        <ship-app-icon name="settings" variant="solid" size="md"></ship-app-icon>
        <span class="launcher-title">${title}</span>
        <span class="launcher-ws">${path}</span>
      </div>
    `;
  }

  private renderFilter(count: number) {
    return html`
      <div class="launcher-filter">
        <ship-icon name="search" size="sm"></ship-icon>
        <input
          type="text"
          placeholder="Filter views…"
          aria-label="Filter views"
          @input=${this.onFilterInput}
          @keydown=${(e: KeyboardEvent) => {
            // Don't let the input swallow our nav keys.
            if (
              e.key === "ArrowDown" ||
              e.key === "ArrowUp" ||
              e.key === "Enter" ||
              e.key === "Escape"
            ) {
              this.onGlobalKey(e);
            }
          }}
        />
        <span class="launcher-count">${count}</span>
      </div>
    `;
  }

  private renderSection(
    label: string,
    rows: ApiView[],
    indexBase: number,
    isPinned: boolean,
  ) {
    if (rows.length === 0) return nothing;
    return html`
      <div class="launcher-section">
        <div class="launcher-section-h">
          ${isPinned
            ? html`<ship-icon name="pin" size="sm"></ship-icon>`
            : nothing}
          <span>${label}</span>
          <span class="launcher-section-spacer"></span>
          ${isPinned
            ? html`<span class="launcher-hint">drag to reorder</span>`
            : nothing}
        </div>
        ${rows.map((v, i) => this.renderRow(v, indexBase + i, isPinned))}
      </div>
    `;
  }

  private renderRow(v: ApiView, idx: number, isPinned: boolean) {
    const label = v.rail?.label ?? humanize(v.name);
    const fallbackIcon = v.rail?.icon ?? "app-window";
    const known = KNOWN_APP_NAMES.has(v.name);
    const pinned = this.pins.includes(v.name);
    const isCursor = idx === this.cursor;
    const isDragOver = this.dragOver === v.name;
    const isDragging = this.dragName === v.name;
    const cls = [
      "launcher-row",
      isCursor ? "is-cursor" : "",
      isDragOver ? "is-drag-over" : "",
      isDragging ? "is-dragging" : "",
    ]
      .filter(Boolean)
      .join(" ");
    return html`
      <div
        class=${cls}
        role="button"
        tabindex="-1"
        draggable=${isPinned ? "true" : "false"}
        @click=${() => this.navigate(v.name)}
        @mouseenter=${() => (this.cursor = idx)}
        @dragstart=${(e: DragEvent) => isPinned && this.onDragStart(e, v.name)}
        @dragover=${(e: DragEvent) => isPinned && this.onDragOver(e, v.name)}
        @drop=${(e: DragEvent) => isPinned && this.onDrop(e, v.name)}
        @dragend=${this.onDragEnd}
      >
        ${known
          ? html`<ship-app-icon name=${v.name} variant="tile" size="md"></ship-app-icon>`
          : html`<ship-icon name=${fallbackIcon} size="md"></ship-icon>`}
        <span class="launcher-row-name">
          <span class="launcher-row-label">${label}</span>
          <span class="launcher-row-path">${v.source}</span>
        </span>
        <button
          class="launcher-pin ${pinned ? "is-pinned" : ""}"
          aria-label=${pinned ? "Unpin from rail" : "Pin to rail"}
          title=${pinned ? "Unpin from rail" : "Pin to rail"}
          @click=${(e: MouseEvent) => this.togglePin(e, v.name)}
        >
          <ship-icon name="pin" size="sm"></ship-icon>
        </button>
      </div>
    `;
  }

  private renderFoot() {
    return html`
      <div class="launcher-foot">
        <a class="launcher-add" href="#" @click=${(e: MouseEvent) => e.preventDefault()}>
          <ship-icon name="plus" size="sm"></ship-icon>
          <span>Install view</span>
        </a>
        <span class="launcher-section-spacer"></span>
        <span class="launcher-kbd">⌘</span>
        <span class="launcher-kbd">⇧</span>
        <span class="launcher-kbd">P</span>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-launcher": ShipLauncher;
  }
}
