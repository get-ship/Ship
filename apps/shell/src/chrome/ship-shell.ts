//! `<ship-shell>` — root element. Wraps the browser app, owns the
//! kernel-event bridge, publishes `window.ship`, mediates DOM-level
//! shell events (rail toggle, launcher, navigation).

import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { KernelBridge } from "./kernel-bridge";
import type {
  CurrentView,
  EventHandler,
  RouterAdapter,
  ShipEnv,
  ShipSession,
  UnsubscribeFn,
  WindowShip,
} from "./types";

const RAIL_OPEN_KEY = "ship-rail-open";

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
  private router: RouterAdapter | null = null;

  connectedCallback(): void {
    super.connectedCallback();
    this.publishWindowShip();
    this.applyPersistedRailOpen();
    this.addEventListener("ship:rail:toggle", this.onRailToggle);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.removeEventListener("ship:rail:toggle", this.onRailToggle);
  }

  createRenderRoot(): Element {
    return this;
  }

  /**
   * Light-DOM mode with no render output — `<ship-shell>` is a pure
   * wrapper whose children are authored directly in the host HTML.
   * Returning a template that contains a `<slot>` would create a
   * real DOM element (not a shadow slot) and disturb the light-DOM
   * child layout; returning nothing keeps Lit out of the picture.
   */
  render() {
    return null;
  }

  /**
   * Shell app registers its router here. `window.ship.navigate` delegates
   * through it. Called once during app boot.
   */
  registerRouter(adapter: RouterAdapter): void {
    this.router = adapter;
  }

  /** Shell-component registration hook (phase-2+). No-op today. */
  register(_component: { name: string }): void {
    // Phase 2b+: track registered shell components if needed.
  }

  // ── window.ship surface ────────────────────────────────────────────

  private publishWindowShip(): void {
    const shell = this;
    const emit = (name: string, detail?: Record<string, unknown>): void => {
      shell.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
    };
    const api: WindowShip = {
      navigate(path: string, opts?: { replace?: boolean }): void {
        if (!shell.router) {
          if (opts?.replace) location.replace(path);
          else location.assign(path);
          return;
        }
        shell.router.navigate(path, opts);
      },
      back(): void {
        shell.router?.back() ?? history.back();
      },
      current(): CurrentView {
        return (
          shell.router?.current() ?? {
            path: location.pathname,
            params: {},
            query: {},
          }
        );
      },
      notify(message, tone) {
        emit("ship:notify", { message, tone });
      },
      command(opts) {
        emit("ship:command:open", opts ?? {});
      },
      chat: {
        open(opts) {
          emit("ship:chat:open", opts ?? {});
        },
        close() {
          emit("ship:chat:close");
        },
        async send(text: string, workspace_id?: string) {
          const resp = await fetch("/api/chat/send", {
            method: "POST",
            credentials: "same-origin",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text, workspace_id }),
          });
          if (!resp.ok) {
            throw new Error(`chat send failed: ${resp.status}`);
          }
        },
      },
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
    window.ship = api;
  }

  // ── Rail open/close (body class) ───────────────────────────────────

  private applyPersistedRailOpen(): void {
    try {
      if (localStorage.getItem(RAIL_OPEN_KEY)) {
        document.body.classList.add("rail-open");
      }
    } catch {
      /* persistence best-effort */
    }
  }

  private onRailToggle = (): void => {
    const open = document.body.classList.toggle("rail-open");
    try {
      if (open) localStorage.setItem(RAIL_OPEN_KEY, "1");
      else localStorage.removeItem(RAIL_OPEN_KEY);
    } catch {
      /* noop */
    }
  };
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-shell": ShipShell;
  }
}
