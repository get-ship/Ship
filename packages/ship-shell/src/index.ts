//! `@ship/shell` — shell framework for the Ship browser app.
//!
//! Side effects: registers custom elements `<ship-shell>`, `<ship-rail>`,
//! `<ship-launcher>` and publishes `window.ship` on first shell connect.

export { ShipShell } from "./ship-shell";
export { ShipRail } from "./ship-rail";
export { ShipLauncher } from "./ship-launcher";
export { ShipNavTop } from "./ship-nav-top";
export { ShipToastHost } from "./ship-toast-host";
export { KernelBridge } from "./kernel-bridge";
export { displayFor, BUILTIN_DISPLAY } from "./view-catalog";
export { loadPins, pin, unpin } from "./pins";
export type { ViewDisplay } from "./view-catalog";
export type {
  CurrentView,
  EventHandler,
  KernelEvent,
  RouterAdapter,
  ShellDomEvents,
  ShipEnv,
  ShipSession,
  UnsubscribeFn,
  WindowShip,
} from "./types";
