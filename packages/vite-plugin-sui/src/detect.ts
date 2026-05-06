import { basename, dirname } from 'node:path'

import type { ViteFileMode } from './types.js'

/**
 * Default component-vs-view detection.
 *
 * Heuristic: a `.sui` file is a component when its filename stem equals
 * the immediate parent directory's name. Every component in this monorepo
 * (`packages/ui-components/src/<tag>/<tag>.sui`) follows that convention,
 * and views (`views/<name>/page.sui`, `apps/.../some.sui`) all violate it.
 *
 * Front-matter detection (`<!-- @component name="x" -->` etc.) was
 * considered and rejected: every existing component already lives at
 * `<dir>/<dir>.sui` and front matter would just be redundant ceremony.
 * If a future view ends up needing component-mode compilation despite
 * not matching the heuristic, the consumer overrides via `Options.detect`.
 */
export function defaultDetect(absPath: string): ViteFileMode {
  if (!absPath.endsWith('.sui')) {
    // Plugin is the only caller; should never trip this branch.
    return { kind: 'view' }
  }
  const stem = basename(absPath, '.sui')
  const parent = basename(dirname(absPath))
  if (stem === parent && stem.length > 0) {
    return { kind: 'component', meta: { name: stem, shadow: true } }
  }
  return { kind: 'view' }
}
