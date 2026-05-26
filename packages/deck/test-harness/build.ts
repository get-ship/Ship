#!/usr/bin/env bun
// Build the browser harness entries (ses + deck-host inlined) into dist/. Kept
// separate from the dev bundler — this is test infrastructure, not a shipped
// artifact. NOTE: this build itself uses Bun.build for convenience; it is
// dev/CI tooling and ships in nothing (see #163 — bun is dev-only).

import { copyFile, mkdir, rm } from 'node:fs/promises'
import { dirname, join } from 'node:path'

const root = dirname(Bun.fileURLToPath(import.meta.url))
const dist = join(root, 'dist')

await rm(dist, { recursive: true, force: true })
await mkdir(dist, { recursive: true })
const r = await Bun.build({
  entrypoints: [join(root, 'browser/lockdown-smoke.ts')],
  outdir: dist,
  target: 'browser',
  format: 'esm',
  minify: false,
  sourcemap: 'none',
})
if (!r.success) {
  for (const l of r.logs) console.error(String(l))
  process.exit(1)
}
// The HTML wrapper is tracked source (browser/index.html); copy it next to the
// built module so the served dist/ is self-contained.
await copyFile(join(root, 'browser/index.html'), join(dist, 'index.html'))
process.stdout.write(`✓ harness build → ${dist}\n`)
