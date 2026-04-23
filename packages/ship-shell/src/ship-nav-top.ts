//! `<ship-nav-top>` — top bar: brand, breadcrumbs, view actions, cmdk,
//! rail toggle, theme picker slot, fullscreen, chat toggle.
//!
//! Light-DOM. Preserves the `.ship-topbar` stylesheet from index.html.
//! Breadcrumbs auto-update via `ship:navigate` events. Action buttons
//! dispatch `ship:rail:toggle`, `ship:command:open`, `ship:chat:toggle`
//! on `<ship-shell>` — topbar owns no state beyond what it renders.
//!
//! Notable light-DOM ids preserved for mount-time contract with
//! `apps/shell/src/mount.ts`:
//!   - `#topbar-brand`        — home link
//!   - `#breadcrumbs`         — updated here from ship:navigate
//!   - `#view-topbar-actions` — per-view action slot (mount.ts fills it)
//!   - `#cmdk-open`           — command-palette trigger
//!   - `#toggle-rail`         — dispatches ship:rail:toggle
//!   - `#theme-picker`        — theme picker host (legacy popover)
//!   - `#toggle-fullscreen`
//!   - `#toggle-drawer`       — dispatches ship:chat:toggle

import { LitElement, html } from "lit";
import { customElement, state } from "lit/decorators.js";

import { BUILTIN_DISPLAY, displayFor } from "./view-catalog";

@customElement("ship-nav-top")
export class ShipNavTop extends LitElement {
  @state() private path: string = location.pathname;

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  private onNavigate = (e: Event): void => {
    const to = (e as CustomEvent).detail?.to as string | undefined;
    if (typeof to === "string") this.path = to;
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.classList.add("ship-topbar");
    this.shell?.addEventListener("ship:navigate", this.onNavigate);
  }

  firstUpdated(): void {
    // Notify the shell app that nav-top DOM exists. Listeners wire up
    // legacy bits like the theme grid which expect specific ids.
    this.shell?.dispatchEvent(
      new CustomEvent("ship:nav-top:ready", { bubbles: true }),
    );
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:navigate", this.onNavigate);
  }

  createRenderRoot(): Element {
    return this;
  }

  private onBrand = (e: MouseEvent): void => {
    e.preventDefault();
    window.ship?.navigate("/");
  };

  private dispatch(name: string): void {
    this.shell?.dispatchEvent(new CustomEvent(name, { bubbles: true }));
  }

  private onFullscreen = (): void => {
    if (document.fullscreenElement) void document.exitFullscreen();
    else void document.documentElement.requestFullscreen();
  };

  render() {
    return html`
      <a
        class="brand"
        id="topbar-brand"
        href="/"
        title="Ship home"
        @click=${this.onBrand}
      >
        <ship-mark size="sm" aria-label="Ship"></ship-mark>
      </a>
      <div class="sep"></div>
      <div class="crumbs" id="breadcrumbs">${this.renderCrumbs()}</div>
      <span
        id="view-topbar-actions"
        style="display:inline-flex;gap:4px;align-items:center"
      ></span>
      <span class="spacer"></span>
      <button
        class="cmdk-trigger"
        id="cmdk-open"
        title="Command palette"
        @click=${() => this.dispatch("ship:command:open")}
      >
        <ship-icon name="search" size="sm"></ship-icon>
        <span>Search or jump to...</span>
        <kbd>⌘K</kbd>
      </button>
      <ship-button
        variant="ghost"
        size="sm"
        id="toggle-rail"
        title="Toggle navigation (⌘B)"
        aria-label="Toggle navigation"
        @click=${() => this.dispatch("ship:rail:toggle")}
      >
        <ship-icon slot="leading" name="menu"></ship-icon>
      </ship-button>
      <ship-popover placement="bottom-end" id="theme-picker">
        <ship-button
          slot="trigger"
          variant="ghost"
          size="sm"
          aria-label="Choose theme"
          title="Theme"
        >
          <ship-icon slot="leading" name="palette"></ship-icon>
        </ship-button>
        <div class="theme-grid" role="listbox" aria-label="Theme"></div>
      </ship-popover>
      <ship-button
        variant="ghost"
        size="sm"
        id="toggle-fullscreen"
        title="Fullscreen (F11)"
        aria-label="Toggle fullscreen"
        @click=${this.onFullscreen}
      >
        <ship-icon slot="leading" name="maximize"></ship-icon>
      </ship-button>
      <ship-button
        variant="ghost"
        size="sm"
        id="toggle-drawer"
        title="Chat (⌘/)"
        aria-label="Toggle chat drawer"
        @click=${() => this.dispatch("ship:chat:toggle")}
      >
        <ship-icon slot="leading" name="message-square-plus"></ship-icon>
      </ship-button>
    `;
  }

  /** Render breadcrumbs from `this.path`. Leaf segment is the current
   * title (highlighted); earlier segments become clickable links that
   * navigate through `window.ship`. */
  private renderCrumbs() {
    const segs = this.path.split("/").filter(Boolean);
    if (segs.length === 0) segs.push("dashboard");
    return segs.map((seg, i) => {
      const isLast = i === segs.length - 1;
      const href = "/" + segs.slice(0, i + 1).join("/");
      const label = i === 0 ? displayFor(seg).label : seg;
      return isLast
        ? html`<span class="current" id="breadcrumbs-title">${label}</span>`
        : html`<a
              href=${href}
              @click=${(e: MouseEvent) => {
                e.preventDefault();
                window.ship?.navigate(href);
              }}
              >${label}</a
            ><span class="chevron">›</span>`;
    });
  }
}

// Silence unused-import lint: BUILTIN_DISPLAY is exposed for future
// callers that might want to list canonical labels.
void BUILTIN_DISPLAY;

declare global {
  interface HTMLElementTagNameMap {
    "ship-nav-top": ShipNavTop;
  }
}
