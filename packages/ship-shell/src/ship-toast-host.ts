//! `<ship-toast-host>` — event-driven toast surface.
//!
//! Internally owns a `<ship-toast-container>` from `@ship/set`. Listens
//! for `ship:notify` on `<ship-shell>` and forwards each event to the
//! container's programmatic `show()` API. Views call
//! `window.ship.notify(msg, tone?)`; chrome components dispatch
//! `ship:notify` directly if they want more control (title/description/
//! action/duration).
//!
//! Tone mapping follows the ship-toast `tone` vocabulary:
//!   info | success | warning | destructive

import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators.js";

export type NotifyTone = "info" | "success" | "warning" | "destructive";

export interface NotifyDetail {
  /** Primary message. Rendered as the toast title. */
  message?: string;
  /** Secondary supporting text. */
  description?: string;
  /** Visual tone. */
  tone?: NotifyTone;
  /** Auto-dismiss duration (ms). 0 = sticky. */
  duration?: number;
  /** Optional HTML slotted into the toast's `actions` slot. */
  actionHtml?: string;
}

interface ShipToastContainerLike extends HTMLElement {
  show(opts: {
    tone?: string;
    duration?: number;
    title?: string;
    description?: string;
    actionHtml?: string;
  }): HTMLElement;
}

@customElement("ship-toast-host")
export class ShipToastHost extends LitElement {
  /** Position forwarded to the internal container. */
  @property() position = "bottom-right";

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  private onNotify = (e: Event): void => {
    const detail = ((e as CustomEvent).detail ?? {}) as NotifyDetail;
    const container = this.querySelector(
      "ship-toast-container",
    ) as ShipToastContainerLike | null;
    if (!container) return;
    container.show({
      tone: detail.tone,
      duration: detail.duration,
      title: detail.message,
      description: detail.description,
      actionHtml: detail.actionHtml,
    });
  };

  connectedCallback(): void {
    super.connectedCallback();
    this.shell?.addEventListener("ship:notify", this.onNotify);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:notify", this.onNotify);
  }

  createRenderRoot(): Element {
    return this;
  }

  render() {
    return html`
      <ship-toast-container position=${this.position}></ship-toast-container>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-toast-host": ShipToastHost;
  }
}
