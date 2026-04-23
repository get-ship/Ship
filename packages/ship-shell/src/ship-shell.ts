//! `<ship-shell>` — root element. Wraps the browser app, hosts the
//! kernel-event bridge, publishes `window.ship`.
//!
//! Phase 1 surface: `subscribe`, `session`, `env`. Navigation, chrome
//! services (notify, command, chat, terminal), and signal bridging
//! land in later phases — `window.ship` evolves, `<ship-shell>` stays.

import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { KernelBridge } from "./kernel-bridge";
import type {
  EventHandler,
  ShipEnv,
  ShipSession,
  UnsubscribeFn,
  WindowShip,
} from "./types";

@customElement("ship-shell")
export class ShipShell extends LitElement {
  static styles = css`
    :host {
      display: contents;
    }
  `;

  /** Session snapshot — populated by the app on mount. */
  @property({ type: Object })
  session: ShipSession = { id: "" };

  /** Env snapshot — populated by the app on mount. */
  @property({ type: Object })
  env: ShipEnv = { scope: "project", version: "", dev: false };

  private bridge = new KernelBridge();

  connectedCallback(): void {
    super.connectedCallback();
    this.publishWindowShip();
  }

  /** Light-DOM slots only — no shadow-root so external CSS still applies. */
  createRenderRoot(): Element {
    return this;
  }

  render() {
    return html`<slot></slot>`;
  }

  // ── window.ship surface ────────────────────────────────────────────

  private publishWindowShip(): void {
    const shell = this;
    const api: WindowShip = {
      subscribe(namespace: string, handler: EventHandler): UnsubscribeFn {
        return shell.bridge.subscribe(namespace, handler);
      },
      get session() {
        return shell.session;
      },
      get env() {
        return shell.env;
      },
    };
    // Assign idempotently. If a shell is torn down and re-mounted,
    // overwrite; the old bridge is GC'd once its handlers unsubscribe.
    window.ship = api;
  }

  // ── Shell-component registration hook (phase-2+) ───────────────────
  // Children call this.closest('ship-shell')?.register(...) — no-op today.
  register(_component: { name: string }): void {
    // Phase 2: track registered shell components, expose them to
    // `window.ship` consumers if needed.
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-shell": ShipShell;
  }
}
