/**
 * Gallery entry. The heavy lifting lives in `./render.ts` (rendering)
 * and `./samples/*` (per-topic sample data). This module is the single
 * public surface consumed by `main.ts`.
 */

import { renderGalleryAsync } from './render'

export { renderGalleryAsync } from './render'

/**
 * Sync-looking wrapper that kicks off the async render. Returns 0
 * immediately and backfills the count into the `#count` element once
 * discovery + render completes.
 */
export function renderGallery(root: HTMLElement): number {
  void renderGalleryAsync(root).then((n) => {
    const c = document.getElementById('count')
    if (c) c.textContent = `${n} element${n === 1 ? '' : 's'}`
  })
  return 0
}
