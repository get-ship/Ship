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

  // Let module scripts finish registering custom elements.
  await new Promise((r) => setTimeout(r, 50));

  const rendered = renderGallery(root);
  count.textContent = `${rendered} element${rendered === 1 ? "" : "s"}`;
}

void boot();
