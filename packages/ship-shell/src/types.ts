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
 * Additions land in phases; phase-2a adds `navigate` + `current`.
 */
export interface WindowShip {
  /** Programmatic navigation. Delegates to the registered router adapter. */
  navigate(path: string, opts?: { replace?: boolean }): void;

  /** Pop back in history. */
  back(): void;

  /** Current route snapshot. */
  current(): CurrentView;

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
};

declare global {
  interface Window {
    ship?: WindowShip;
  }
}
