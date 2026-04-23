//! `@ship/shell` — shell framework for the Ship browser app.
//!
//! Side effect: registers `<ship-shell>` as a custom element and, on
//! connect, publishes `window.ship`.

export { ShipShell } from "./ship-shell";
export { KernelBridge } from "./kernel-bridge";
export type {
  EventHandler,
  KernelEvent,
  ShipEnv,
  ShipSession,
  UnsubscribeFn,
  WindowShip,
} from "./types";
