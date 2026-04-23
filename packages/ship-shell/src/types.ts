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

/**
 * `window.ship` — the single view-facing API surface. Everything
 * imperative a view might need the browser app for goes through here.
 * Additions land in phases; the phase-1 surface is intentionally small.
 */
export interface WindowShip {
  /** Subscribe to a kernel event namespace (prefix match). */
  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn;

  /** Session + env snapshot, populated on shell connect. */
  session: ShipSession;
  env: ShipEnv;
}

declare global {
  interface Window {
    ship?: WindowShip;
  }
}
