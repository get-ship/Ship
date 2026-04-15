import { renderGallery } from "./gallery";

/**
 * Devtools entry — wait a tick for ship-elements.js to finish registering
 * custom elements, then enumerate and render.
 *
 * `customElements.whenDefined` only works for a known tag, so we poll-check
 * a well-known element instead. ship-elements.js resolves synchronously on
 * module load, so one microtask is enough in practice.
 */
async function boot() {
  const root = document.getElementById("gallery") as HTMLElement;
  const count = document.getElementById("count") as HTMLElement;

  // Wait for ship-elements.js module to finish running its define() calls.
  // Anchor on a small set of known-late tags; the 50ms timeout is a hard
  // upper bound for environments where whenDefined never resolves (e.g. a
  // tag name was removed from the bundle).
  await Promise.race([
    Promise.all([
      customElements.whenDefined("ship-button"),
      customElements.whenDefined("ship-card"),
      customElements.whenDefined("ship-dialog"),
    ]),
    new Promise((r) => setTimeout(r, 300)),
  ]);

  const rendered = renderGallery(root);
  count.textContent = `${rendered} element${rendered === 1 ? "" : "s"}`;
}

void boot();
