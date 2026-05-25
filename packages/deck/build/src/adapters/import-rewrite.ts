import { basename, dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { existsSync } from 'node:fs'

export function findWorkspaceRoot(startDir: string): string {
  let dir = startDir
  while (true) {
    if (existsSync(join(dir, 'Cargo.toml'))) return dir
    const parent = dirname(dir)
    if (parent === dir) return startDir
    dir = parent
  }
}

/**
 * Bundler plugin: rewrite `./<dir>/<stem>.compiled.js` imports to the project
 * mirror tree (`<repo-root>/.ship/sui-out/<src-rel>/<stem>.compiled.js`) and
 * re-anchor the emitted code's `@ship/*` imports to the workspace source
 * packages.
 *
 * Source files reference the compiled artifact as a sibling for ergonomic
 * authoring, but the artifact lives in the mirror tree. The mirror tree is
 * outside the importer's node_modules walk, so re-anchoring `@ship/*` to the
 * package source keeps workspace resolution intact across the redirect.
 *
 * Bun and esbuild share plugin shape (Bun mirrored esbuild's API), so this
 * single factory works for both bundler adapters.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function mirrorResolverPlugin(packageRoot: string): any {
  const repoRoot = findWorkspaceRoot(packageRoot)
  const mirrorRoot = join(repoRoot, '.ship/sui-out')
  const packagesRoot = join(repoRoot, 'packages')
  // Workspace package dirs are nested one or two levels under packages/
  // (e.g. packages/sui/runtime, packages/deck/build, packages/components/primitives,
  // packages/assets). The original importer can't see these via node_modules
  // walk when bundling from the mirror tree, so resolve @ship/<name> by
  // probing the known umbrella dirs in order.
  const umbrellaDirs = ['', 'sui', 'deck', 'components']
  return {
    name: 'sui-compiled-mirror',
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    setup(build: any) {
      build.onResolve(
        { filter: /\.compiled\.js$/ },
        ({ path: id, importer }: { path: string; importer?: string }) => {
          if (!importer) return null
          const importerDir = dirname(importer)
          const sourceArtifact = isAbsolute(id) ? id : resolve(importerDir, id)
          const sourceDir = dirname(sourceArtifact)
          const stem = basename(sourceArtifact).replace(/\.compiled\.js$/, '')
          const rel = relative(repoRoot, sourceDir)
          const mirrorFile = join(mirrorRoot, rel, `${stem}.compiled.js`)
          if (existsSync(mirrorFile)) return { path: mirrorFile }
          return null
        },
      )
      build.onResolve(
        { filter: /^@ship\// },
        ({ path: id, importer }: { path: string; importer?: string }) => {
          // Shared-singleton framework modules stay EXTERNAL for every importer
          // (mirror compiled.js AND the pack's own src) so each pack/view shares
          // the ONE instance the host import map provides (`@ship/sui-runtime` →
          // /_sui/runtime.js, etc.). Inlining a copy into a pack creates
          // multiple runtime instances → split-brain signals (`state.$signals is
          // undefined` at mount). This MUST run before the mirror-importer guard
          // below, since the pack's `src/index.ts` imports aren't under the
          // mirror root. (#108 de-dup; the deck bundler already externalizes
          // these for its own targets — this covers the sui-build pack builds.)
          const sm = id.match(/^@ship\/([^/]+)/)
          const SHARED_SINGLETONS = new Set(['sui-runtime', 'sui-dom', 'sdk-client'])
          if (sm && SHARED_SINGLETONS.has(sm[1]!)) return { path: id, external: true }
          if (!importer) return null
          if (!importer.startsWith(mirrorRoot)) return null
          const m = id.match(/^@ship\/([^/]+)(\/.*)?$/)
          if (!m) return null
          const rawName = m[1]!
          const subpath = m[2] ?? ''
          // Npm names like `@ship/sui-runtime` correspond to dir `runtime`
          // under the `sui` umbrella (the `sui-` prefix was dropped from
          // dir names during the deck restructure). Try both forms.
          const dirCandidates = rawName.startsWith('sui-')
            ? [rawName.slice(4), rawName]
            : [rawName]
          for (const dirName of dirCandidates) {
            for (const umbrella of umbrellaDirs) {
              const pkgRoot = umbrella ? join(packagesRoot, umbrella, dirName) : join(packagesRoot, dirName)
              if (!existsSync(join(pkgRoot, 'package.json'))) continue
              const candidate = subpath
                ? join(pkgRoot, 'src', subpath.replace(/^\//, '')) + '.ts'
                : join(pkgRoot, 'src/index.ts')
              if (existsSync(candidate)) return { path: candidate }
              const fallback = join(pkgRoot, 'src/index.ts')
              if (existsSync(fallback)) return { path: fallback }
            }
          }
          return null
        },
      )
    },
  }
}
