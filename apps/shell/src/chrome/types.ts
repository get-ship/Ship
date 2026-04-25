//! Public types shared by the shell and view authors.

export interface KernelEvent {
  event_type: string;
  entity: string;
  actor?: string;
  timestamp: string;
  payload: unknown;
  [k: string]: unknown;
}

export type EventHandler = (event: KernelEvent) => void;
export type UnsubscribeFn = () => void;

export interface ShipSession {
  id: string;
  workspace_id?: string;
}

export interface ShipEnv {
  scope: "user" | "project";
  version: string;
  dev: boolean;
}

/** Snapshot of the currently-mounted view. */
export interface CurrentView {
  path: string;
  params: Record<string, string>;
  query: Record<string, string>;
}

/**
 * Adapter the shell app registers to let `<ship-shell>` drive navigation
 * without knowing which router implementation is in use.
 */
export interface RouterAdapter {
  navigate(path: string, opts?: { replace?: boolean }): void;
  back(): void;
  current(): CurrentView;
}

/**
 * `window.ship` — the single view-facing API surface. Everything
 * imperative a view might need from the browser app goes through here.
 * Additions land in phases; phase 3 adds shell services (notify,
 * command, chat).
 */
export interface WindowShip {
  /** Programmatic navigation. Delegates to the registered router adapter. */
  navigate(path: string, opts?: { replace?: boolean }): void;

  /** Pop back in history. */
  back(): void;

  /** Current route snapshot. */
  current(): CurrentView;

  /** Show a toast. Shorthand for `dispatchEvent(ship:notify)`. */
  notify(
    message: string,
    tone?: "info" | "success" | "warning" | "destructive",
  ): void;

  /** Open the command palette, optionally prefilled. */
  command(opts?: { query?: string }): void;

  /** Chat drawer services. */
  chat: {
    open(opts?: { workspace_id?: string }): void;
    close(): void;
    send(text: string, workspace_id?: string): Promise<void>;
  };

  /** Subscribe to a kernel event namespace (prefix match). */
  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn;

  /** Session + env snapshot, populated on shell connect. */
  session: ShipSession;
  env: ShipEnv;
}

/**
 * DOM custom events the shell dispatches on `<ship-shell>`.
 * Components subscribe via `this.shell.addEventListener('ship:navigate', …)`.
 */
export type ShellDomEvents = {
  "ship:navigate": { from: string; to: string };
  "ship:view-mounted": { path: string; root: HTMLElement };
  "ship:view-unmounted": { path: string };
  "ship:rail:toggle": Record<string, never>;
  "ship:launcher:open": { anchor?: HTMLElement };
  "ship:launcher:close": Record<string, never>;
  "ship:notify": {
    message?: string;
    description?: string;
    tone?: "info" | "success" | "warning" | "destructive";
    duration?: number;
    actionHtml?: string;
  };
  "ship:command:open": { query?: string };
  "ship:chat:toggle": Record<string, never>;
  "ship:chat:open": { workspace_id?: string };
  "ship:chat:close": Record<string, never>;
  "ship:chat:send": { text: string; workspace_id?: string };
};

declare global {
  interface Window {
    ship?: WindowShip;
  }
}
