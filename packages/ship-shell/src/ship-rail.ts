//! `<ship-rail>` — macOS-dock-style pinned view launcher.
//!
//! Light-DOM, uses the existing `.ship-rail` stylesheet from the shell's
//! index.html. Reads pin state from localStorage via `./pins`. Subscribes
//! to `ship:navigate` on `<ship-shell>` to highlight the active item.
//!
//! Clicking an item dispatches `ship:launcher:close` + calls
//! `window.ship.navigate(path)`. Right-click (or context menu) unpins.
//! A bottom "+" button dispatches `ship:launcher:open` with itself as
//! anchor; `<ship-launcher>` consumes the event.

import { LitElement, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

import { loadPins, unpin } from "./pins";
import { displayFor } from "./view-catalog";

@customElement("ship-rail")
export class ShipRail extends LitElement {
  @state() private pins: string[] = loadPins();
  @state() private active = "";

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  private onNavigate = (e: Event): void => {
    const detail = (e as CustomEvent).detail as { to?: string } | undefined;
    if (!detail?.to) return;
    this.active = this.routeToViewName(detail.to);
  };

  private onPinsChanged = (e: Event): void => {
    const detail = (e as CustomEvent).detail as { pins?: string[] } | undefined;
    if (Array.isArray(detail?.pins)) this.pins = detail.pins;
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.active = this.routeToViewName(location.pathname);
    this.shell?.addEventListener("ship:navigate", this.onNavigate);
    this.shell?.addEventListener(
      "ship:rail:pins-changed" as keyof HTMLElementEventMap,
      this.onPinsChanged,
    );
    this.classList.add("ship-rail");
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:navigate", this.onNavigate);
    this.shell?.removeEventListener(
      "ship:rail:pins-changed" as keyof HTMLElementEventMap,
      this.onPinsChanged,
    );
  }

  createRenderRoot(): Element {
    return this;
  }

  /** Map a URL path to a view name by taking the first path segment. */
  private routeToViewName(path: string): string {
    const seg = path.split("/").filter(Boolean)[0] ?? "dashboard";
    return seg === "" ? "dashboard" : seg;
  }

  private onItemClick(e: MouseEvent, name: string): void {
    e.preventDefault();
    window.ship?.navigate(`/${name === "dashboard" ? "" : name}`);
  }

  private onItemContext(e: MouseEvent, name: string): void {
    e.preventDefault();
    // For now, right-click unpins. Future: context menu with rename/reorder.
    this.pins = unpin(name);
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

  render() {
    // Partition pins into settings (always last) + rest so the spacer
    // pushes Settings to the bottom.
    const settings = this.pins.filter((n) => n === "settings");
    const rest = this.pins.filter((n) => n !== "settings");
    return html`
      ${rest.map((name) => this.renderItem(name))}
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

  private renderItem(name: string) {
    const display = displayFor(name);
    const classes = `rail-item${this.active === name ? " active" : ""}`;
    return html`
      <a
        class=${classes}
        title=${display.label}
        data-view=${name}
        href=${`/${name === "dashboard" ? "" : name}`}
        @click=${(e: MouseEvent) => this.onItemClick(e, name)}
        @contextmenu=${(e: MouseEvent) => this.onItemContext(e, name)}
      >
        <ship-icon name=${display.icon}></ship-icon>
      </a>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-rail": ShipRail;
  }
}
