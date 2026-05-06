import { existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { dirname, isAbsolute, join, resolve } from 'node:path'

import type { Plugin } from 'vite'

import { compileSui } from './compile.ts'
import { defaultDetect } from './detect.ts'
import type { Options, ViteFileMode } from './types.ts'

export type { Options, ViteFileMode } from './types.ts'
export { defaultDetect } from './detect.ts'

export default function suiPlugin(opts: Options = {}): Plugin {
  let resolvedCompiler: string | null = null
  let viteRoot = process.cwd()

  function ensureCompiler(): string {
    if (resolvedCompiler) return resolvedCompiler
    const explicit = opts.compilerPath
    if (explicit) {
      if (!existsSync(explicit)) {
        throw new Error(`vite-plugin-sui: compilerPath does not exist: ${explicit}`)
      }
      resolvedCompiler = explicit
      return resolvedCompiler
    }
    const candidate = locateCompiler(viteRoot)
    if (!candidate) {
      throw new Error(
        'vite-plugin-sui: could not locate sui-compiler. Set Options.compilerPath, ' +
          'install via `ship runtime install`, or run `cargo build -p sui-compiler --release`.',
      )
    }
    resolvedCompiler = candidate
    return resolvedCompiler
  }

  return {
    name: 'vite-plugin-sui',

    config() {
      return {
        resolve: { extensions: ['.ts', '.js', '.sui'] },
      }
    },

    configResolved(resolved) {
      viteRoot = resolved.root
    },

    async transform(src, id) {
      if (!id.endsWith('.sui')) return null
      const mode = resolveMode(id, opts)
      const result = await compileSui({
        compilerPath: ensureCompiler(),
        filePath: id,
        source: src,
        mode,
        noDts: opts.noDts ?? true,
      })
      return { code: result.code, map: result.map }
    },

    handleHotUpdate(ctx) {
      if (!ctx.file.endsWith('.sui')) return
      // Vite's default invalidation already re-fires `transform` for any
      // module that imports this `.sui` file, which is what we want. We
      // return the same module set explicitly so the contract is visible
      // to callers reading this hook in isolation.
      return ctx.modules
    },
  }
}

function resolveMode(absPath: string, opts: Options): ViteFileMode {
  if (opts.detect) {
    const explicit = opts.detect(absPath)
    if (explicit) return explicit
  }
  return defaultDetect(absPath)
}

/**
 * Walks the candidate locations for a sui-compiler binary in order:
 *   1. ~/.ship/runtime/sui-compiler  — Step 3 toolchain layout (production)
 *   2. <repo>/target/release/sui-compiler — in-repo dev (cargo build --release)
 *   3. <repo>/target/debug/sui-compiler   — in-repo dev (cargo build)
 *
 * Returns the first one that exists, or null. The walk for (2) and (3)
 * climbs from the Vite root looking for a Cargo workspace marker, so the
 * plugin works for any project layout where the .sui sources live below
 * the workspace root.
 */
function locateCompiler(viteRoot: string): string | null {
  const home = join(homedir(), '.ship', 'runtime', 'sui-compiler')
  if (existsSync(home)) return home

  const workspaceRoot = findCargoWorkspace(viteRoot)
  if (workspaceRoot) {
    const release = join(workspaceRoot, 'target', 'release', 'sui-compiler')
    if (existsSync(release)) return release
    const debug = join(workspaceRoot, 'target', 'debug', 'sui-compiler')
    if (existsSync(debug)) return debug
  }
  return null
}

function findCargoWorkspace(start: string): string | null {
  // Walk up and remember the topmost ancestor that contains a Cargo.toml.
  // That's the workspace root — `target/` lives there by default. Member
  // crates also have Cargo.toml, so we can't return on the first hit.
  let cur = isAbsolute(start) ? start : resolve(start)
  let topmost: string | null = null
  for (let i = 0; i < 16; i++) {
    if (existsSync(join(cur, 'Cargo.toml'))) topmost = cur
    const parent = dirname(cur)
    if (parent === cur) break
    cur = parent
  }
  return topmost
}
