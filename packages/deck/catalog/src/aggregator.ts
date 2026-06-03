// Core aggregator — given a repo root, walks per-component CEMs from
// published packages (each package's `customElements` manifest) AND from
// `.ship/sui-out/**/*.custom-elements.json` (non-packaged origins:
// shell-local, view-local). Merges into the canonical CatalogDocument
// shape defined in `./types.ts`.
//
// Filesystem-only — no runtime daemon assumptions. Output is byte-stable
// across runs given the same input set (sort by tagName).

import { readFile, readdir } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { basename, join, relative, resolve, sep } from 'node:path'
import type {
  CatalogDocument,
  CatalogEntry,
  CemDeclaration,
  CemModule,
  CemRoot,
  ComponentOrigin,
  WorkspacePackage,
} from './types.js'
import { discoverWorkspacePackages, packIdForName } from './workspace.js'

const SUI_OUT = '.ship/sui-out'
const SKIP_DIRS = new Set(['node_modules', 'dist', 'target', '.git'])
const CEM_SUFFIX = '.custom-elements.json'

async function readCemFile(path: string): Promise<CemRoot | null> {
  try {
    const raw = await readFile(path, 'utf8')
    return JSON.parse(raw) as CemRoot
  } catch {
    return null
  }
}

async function walkCemFiles(dir: string, out: string[]): Promise<void> {
  let entries
  try {
    entries = await readdir(dir, { withFileTypes: true })
  } catch {
    return
  }
  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue
    const p = join(dir, entry.name)
    if (entry.isDirectory()) await walkCemFiles(p, out)
    else if (entry.isFile() && entry.name.endsWith(CEM_SUFFIX)) out.push(p)
  }
}

/** Decide origin tag for a non-packaged CEM. `.ship/sui-out/` mirrors the
 *  repo source tree under `packages/...`, `apps/<app>/...`, `views/<name>/...`.
 *  Anything starting with `apps/shell/` is shell-local; `views/<name>/` is
 *  view-local; everything else (rare) gets a `workspace:` fallback so the
 *  picker can still display it under its source directory. */
export function originForNonPackagedSource(repoRelativeSourcePath: string): ComponentOrigin {
  const norm = repoRelativeSourcePath.split(sep).join('/')
  if (norm.startsWith('apps/shell/')) return 'app:shell'
  const viewMatch = norm.match(/^views\/([^/]+)\//)
  if (viewMatch) return `view:${viewMatch[1]}`
  const assetsViewMatch = norm.match(/assets\/views\/([^/]+)\//)
  if (assetsViewMatch) return `view:${assetsViewMatch[1]}`
  return `workspace:${norm.split('/')[0] ?? 'unknown'}`
}

/** Repo-relative source path for a per-component CEM. CEMs land at
 *  `.ship/sui-out/<mirrored-source-tree>/<stem>.custom-elements.json`;
 *  the `.sui` source for the component sits at the same mirrored path
 *  with the `.custom-elements.json` swapped for `.sui`. */
function sourcePathForCem(repoRoot: string, cemAbs: string): string {
  const repoRel = relative(repoRoot, cemAbs)
  const mirroredRel = repoRel.startsWith(SUI_OUT + sep)
    ? repoRel.slice(SUI_OUT.length + 1)
    : repoRel
  const stem = basename(mirroredRel, CEM_SUFFIX)
  const dir = mirroredRel.slice(0, mirroredRel.length - basename(mirroredRel).length)
  return (dir + stem + '.sui').split(sep).join('/')
}

function declToEntryBase(decl: CemDeclaration): Omit<
  CatalogEntry,
  'package' | 'private' | 'sourcePath' | 'bundleUrl' | 'manifestUrl'
> | null {
  if (!decl.tagName || !decl.customElement) return null
  const description = (decl.description ?? '').trim()
  return {
    tagName: decl.tagName,
    description,
    firstDesc: description.split('\n')[0] ?? '',
    attributes: decl.attributes ?? [],
    slots: decl.slots ?? [],
    events: decl.events ?? [],
    methods: decl.members ?? [],
    cssProperties: decl.cssProperties ?? [],
    cssParts: decl.cssParts ?? [],
    examples: decl.examples ?? [],
  }
}

interface BuildContext {
  packageOrigin: ComponentOrigin
  isPrivate: boolean
  bundleUrl: (decl: CemDeclaration) => string
  manifestUrl: string
  /** Source-path resolver: takes the cem module's `path` (relative to the
   *  CEM source) and returns a repo-relative source path. */
  sourcePathFor: (mod: CemModule, decl: CemDeclaration) => string
}

function modulesToEntries(modules: CemModule[], ctx: BuildContext): CatalogEntry[] {
  const out: CatalogEntry[] = []
  for (const mod of modules) {
    for (const decl of mod.declarations ?? []) {
      const base = declToEntryBase(decl)
      if (!base) continue
      out.push({
        ...base,
        package: ctx.packageOrigin,
        private: ctx.isPrivate,
        sourcePath: ctx.sourcePathFor(mod, decl),
        bundleUrl: ctx.bundleUrl(decl),
        manifestUrl: ctx.manifestUrl,
      })
    }
  }
  return out
}

async function entriesFromPackagedManifest(
  repoRoot: string,
  pkg: WorkspacePackage,
): Promise<CatalogEntry[]> {
  if (!pkg.customElements) return []
  const manifestAbs = resolve(pkg.root, pkg.customElements)
  if (!existsSync(manifestAbs)) return []
  const cem = await readCemFile(manifestAbs)
  if (!cem?.modules) return []
  const packId = packIdForName(pkg.name)
  // Bundle name = npm-shaped `dist/<bundle>.js`. We can't recover the
  // exact bundle filename from a CEM module path (they're component
  // .sui filenames, not bundle names) — but every Ship pack today
  // produces a single umbrella bundle named after its `entries` key
  // (`sui-charts.js`, etc.). Read the pack's `dist/` directory and use
  // the first `.js` we find. Multi-bundle packs aren't shipped today;
  // when they arrive, the pack will declare a bundle-name mapping.
  const bundleName = await firstJsInDist(pkg.root)
  // A package with a CEM manifest but NO built `dist/` JS bundle is not a
  // shippable pack — its components are baked into the deck framework umbrella
  // (`/_sui/components.js`, side-effect-loaded by every host) and served from
  // `/_sui/sui-components`. Emitting a `/_sui/packs/<id>/index.js` bundleUrl for
  // it points the view-host's eager pack-loader (view-host.html) at a 404 on
  // every page; combined with the live-reload that drove a reload loop. Keep
  // such packages WORKSPACE-ONLY (private) so they never enter `published.json`
  // — the umbrella already defines their custom elements.
  const isShippablePack = bundleName != null
  const bundleUrl = bundleName
    ? `/_sui/packs/${packId}/${bundleName}`
    : `/_sui/packs/${packId}/index.js`
  const manifestUrl = `/_sui/packs/${packId}/custom-elements.json`
  // Source path: pack CEMs use bare filenames like `sui-line-chart.sui`
  // for their module.path. Map back to the pack's `src/<stem>/` tree.
  const pkgRelRoot = relative(repoRoot, pkg.root).split(sep).join('/')
  const sourcePathFor = (mod: CemModule): string => {
    const modPath = mod.path ?? ''
    const stem = modPath.endsWith('.sui') ? modPath.slice(0, -4) : modPath
    return stem ? `${pkgRelRoot}/src/${stem}/${stem}.sui` : `${pkgRelRoot}/src/${modPath}`
  }
  return modulesToEntries(cem.modules, {
    packageOrigin: pkg.name,
    isPrivate: pkg.private || !isShippablePack,
    bundleUrl: () => bundleUrl,
    manifestUrl,
    sourcePathFor,
  })
}

async function firstJsInDist(pkgRoot: string): Promise<string | null> {
  const distDir = join(pkgRoot, 'dist')
  try {
    const entries = await readdir(distDir, { withFileTypes: true })
    const js = entries.find((e) => e.isFile() && e.name.endsWith('.js'))
    return js?.name ?? null
  } catch {
    return null
  }
}

async function entriesFromSuiOut(
  repoRoot: string,
  packages: WorkspacePackage[],
): Promise<CatalogEntry[]> {
  const root = join(repoRoot, SUI_OUT)
  if (!existsSync(root)) return []
  const cemFiles: string[] = []
  await walkCemFiles(root, cemFiles)
  cemFiles.sort()
  // Workspace package lookup: sort longest-first so nested packages win.
  const packagedRoots: Array<{ rel: string; pkg: WorkspacePackage }> = packages.map((p) => ({
    rel: relative(repoRoot, p.root).split(sep).join('/'),
    pkg: p,
  }))
  packagedRoots.sort((a, b) => b.rel.length - a.rel.length)
  const out: CatalogEntry[] = []
  for (const cemAbs of cemFiles) {
    const sourcePath = sourcePathForCem(repoRoot, cemAbs)
    const owning = findOwningPackage(sourcePath, packagedRoots)
    // If the owning package declares a customElements manifest, its
    // entries come from that aggregated manifest (canonical) — skip the
    // per-component CEM to avoid duplicate/stale data.
    if (owning?.pkg.customElements) continue
    const cem = await readCemFile(cemAbs)
    if (!cem?.modules) continue
    // Origin choice:
    //   • Owning workspace package with a customElements manifest →
    //     handled above (skipped here, served from the aggregated CEM).
    //   • Shell-local (sourcePath under `apps/shell/`) → `app:shell`,
    //     even though apps/shell is a registered workspace package.
    //     Treating it as a published pack would expose every chrome
    //     tag in `published.json`; we keep them workspace-only.
    //   • Other owning workspace package → its npm name, preserving
    //     the previous behavior for non-pack utility packages.
    //   • Unowned source → path heuristic (`view:<name>`,
    //     `workspace:<dir>`).
    const isShellLocal = sourcePath.startsWith('apps/shell/')
    const origin = isShellLocal
      ? originForNonPackagedSource(sourcePath)
      : (owning?.pkg.name ?? originForNonPackagedSource(sourcePath))
    const isPrivate = owning?.pkg.private ?? true
    const entries = modulesToEntries(cem.modules, {
      packageOrigin: origin,
      isPrivate,
      bundleUrl: (decl) => `/_sui/sui-components/${decl.tagName}.js`,
      manifestUrl: '',
      sourcePathFor: () => sourcePath,
    })
    out.push(...entries)
  }
  return out
}

function findOwningPackage(
  repoRelPath: string,
  packagedRoots: Array<{ rel: string; pkg: WorkspacePackage }>,
): { rel: string; pkg: WorkspacePackage } | null {
  for (const r of packagedRoots) {
    if (repoRelPath === r.rel || repoRelPath.startsWith(r.rel + '/')) return r
  }
  return null
}

export interface AggregateOptions {
  repoRoot: string
}

export async function buildCatalogs(
  opts: AggregateOptions,
): Promise<{ workspace: CatalogDocument; published: CatalogDocument }> {
  const packages = await discoverWorkspacePackages(opts.repoRoot)
  const packagedEntries: CatalogEntry[] = []
  for (const pkg of packages) {
    if (!pkg.customElements) continue
    packagedEntries.push(...(await entriesFromPackagedManifest(opts.repoRoot, pkg)))
  }
  const nonPackagedEntries = await entriesFromSuiOut(opts.repoRoot, packages)
  const all = [...packagedEntries, ...nonPackagedEntries]
  all.sort((a, b) => a.tagName.localeCompare(b.tagName))
  // Dedup by tagName — first-occurrence wins. Packaged entries come first
  // so a duplicate from `.ship/sui-out/` for a packaged tag is dropped.
  const seen = new Set<string>()
  const deduped: CatalogEntry[] = []
  for (const e of all) {
    if (seen.has(e.tagName)) continue
    seen.add(e.tagName)
    deduped.push(e)
  }
  const workspace: CatalogDocument = {
    schemaVersion: '1',
    count: deduped.length,
    entries: deduped,
  }
  const publishedEntries = deduped.filter((e) => !e.private)
  const published: CatalogDocument = {
    schemaVersion: '1',
    count: publishedEntries.length,
    entries: publishedEntries,
  }
  return { workspace, published }
}

export function stringifyCatalog(doc: CatalogDocument): string {
  return JSON.stringify(doc, null, 2) + '\n'
}
