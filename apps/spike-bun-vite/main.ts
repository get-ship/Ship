// Spike entry: imports a `.sui` file, runs it through Vite's transform
// pipeline (which calls our plugin → sui-compiler), and mounts the result.
//
// We don't assert visual output here — the spike is about the toolchain.
// What we want to know:
//   1. Vite's `transform` hook fires on .sui imports under Bun.
//   2. The compiled output's @ship/ui-runtime / @ship/ui-dom imports resolve.
//   3. HMR re-fires `transform` on .sui edits.
//   4. `vite build` produces a static bundle with the .sui transformed.

import view from "./app.sui";

const root = document.getElementById("app");
if (root && view && typeof (view as { __mount?: unknown }).__mount === "function") {
  (view as { __mount: (el: Element) => void }).__mount(root);
}

// HMR sanity probe — accept this module so editing app.sui or main.ts
// triggers a hot replace rather than a full reload.
if (import.meta.hot) {
  import.meta.hot.accept();
}
