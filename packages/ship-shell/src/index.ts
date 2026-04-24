//! `@ship/shell` — shell framework for the Ship browser app.
//!
//! Side effects: registers custom elements `<ship-shell>`, `<ship-rail>`,
//! `<ship-launcher>`, `<ship-nav-top>`, `<ship-panel-dock>`,
//! `<ship-toast-host>` and publishes `window.ship` on first shell
//! connect.

export { ShipShell } from "./ship-shell";
export { ShipRail } from "./ship-rail";
export { ShipLauncher } from "./ship-launcher";
export { ShipNavTop } from "./ship-nav-top";
export { ShipPanelDock } from "./ship-panel-dock";
export type { DockTab } from "./ship-panel-dock";
export { ShipDevtoolsSource } from "./devtools/source-tab";
export { ShipDevtoolsSignals } from "./devtools/signals-tab";
export { ShipDevtoolsLint } from "./devtools/lint-tab";
export { ShipDevtoolsSchema } from "./devtools/schema-tab";
export { ShipCreateDialog } from "./ship-create-dialog";
export { ShipToastHost } from "./ship-toast-host";
export { KernelBridge } from "./kernel-bridge";
export { DEFAULT_PINS, humanize } from "./view-catalog";
export { fetchPins, savePins, pin, unpin } from "./pins";
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
