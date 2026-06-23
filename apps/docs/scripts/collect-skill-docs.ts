/**
 * collect-skill-docs.ts
 *
 * Collects doc pages from docs/<skill>/references/docs/ into the Astro content
 * directory. Both .md and .mdoc are eligible — the `group:` frontmatter field
 * is the signal that a file is project documentation (as opposed to agent
 * usage notes).
 *
 * Site structure is driven by frontmatter in each .mdoc file:
 *   - title (required): page title
 *   - description: meta description
 *   - audience: public (default) | internal | agent-only
 *   - group: sidebar group (e.g. "CLI", "MCP", "Compiler")
 *   - order: sort position within group
 *
 * Run as prebuild/predev via tsx.
 */
import {
  readFileSync, writeFileSync, mkdirSync, existsSync,
  readdirSync, copyFileSync, watch, rmSync,
} from "fs"
import { join, resolve, dirname, basename } from "path"
import { collectComponentDocs } from "./collect-component-docs"

const SCRIPT_DIR = dirname(new URL(import.meta.url).pathname)
const REPO_ROOT = resolve(SCRIPT_DIR, "../../../")
const SKILLS_DIR = join(REPO_ROOT, "docs")
// Generated SDK reference: `cargo xtask gen-skills` emits one op page per op
// under skills/sdk/<skill>/references/docs/*.mdoc, each stamped with a
// `group: "Ops: <domain>"` frontmatter — the SAME doc-skill contract the
// hand-authored docs/ tree uses. Harvesting both roots fulfils the
// "one generator, two consumers" promise in crates/xtask/src/gen_skills.rs:
// the agent's reference and the public API docs are the same source files.
const SDK_SKILLS_DIR = join(REPO_ROOT, "skills/sdk")
// Doc-skill roots, walked in order. `srcPrefix` is the repo-relative path
// stamped into each generated file's AUTO-GENERATED banner so an integrator
// who hits "View Source" is pointed back at the true source page.
const SKILL_ROOTS: ReadonlyArray<{ dir: string; srcPrefix: string }> = [
  { dir: SKILLS_DIR, srcPrefix: "docs" },
  { dir: SDK_SKILLS_DIR, srcPrefix: "skills/sdk" },
]
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/content/docs")
// Output is treated as build artifact (gitignored). Override via
// COLLECT_DOCS_OUT for the drift checker which collects into a tempdir.
//
// Starlight hard-codes the docs collection path to `src/content/docs/`
// (see node_modules/@astrojs/starlight/utils/collection.ts — last-commit
// timestamps, remark plugin lookups, and starlight-sidebar-topics all
// dead-reckon against it), so this dir name is non-negotiable. To keep
// the "is this generated?" boundary visible despite the inflexible
// name, we (1) gitignore the whole tree, (2) stamp an AUTO-GENERATED
// HTML banner at the top of every emitted file, and (3) park
// hand-authored content next door in src/content-source/, copied in
// during the same run. Per-group output lands in `<group>/`
// subdirectories under OUTPUT_DIR — astro.config.mjs autogenerate
// paths resolve against the flat layout.
const GENERATED_DIR = process.env.COLLECT_DOCS_OUT
  ? resolve(process.env.COLLECT_DOCS_OUT)
  : OUTPUT_DIR
// Hand-authored content lives here (gitTRACKED) and is copied into
// GENERATED_DIR on each run alongside the skill-derived output. Today
// the only file is index.md — the docs homepage.
const HAND_AUTHORED_DIR = join(SCRIPT_DIR, "../src/content-source")
const PUBLIC_DIR = join(SCRIPT_DIR, "../public")
// SUI bundles served as static assets. Mirror the names the daemon uses
// under `/api/assets/` so the importmap in astro.config.mjs resolves the
// same way against either the docs site or the daemon.
// Post-carve: assets/ lives in the external sui repo (committed source).
// TEMP: relative path → absolute `/home/madvib/dev/sui/...` after layout
// reshuffle.
const SUI_ASSETS_DIR = join(REPO_ROOT, "..", "..", "sui", "crates", "deck", "deck-core", "assets")
// The framework bundles below are what the deck bundler bakes into
// SUI_ASSETS_DIR and what the importmap resolves against.
const SUI_BUNDLES = [
  "sui-runtime.js",
  "sui-dom.js",
  "sui-sdk-client.js",
] as const
// The daemon serves a single pre-concatenated `/api/assets/view-tokens.css`
// (tokens + themes), baked by the deck bundler to
// SUI_ASSETS_DIR/view-tokens.css. Any SUI primitive that links to that URL
// from its shadow root (notably `<sui-theme-picker>`, on every docs page
// via the header) resolves against the same canonical file the daemon
// serves at runtime.
const VIEW_TOKENS_SOURCE = join(SUI_ASSETS_DIR, "view-tokens.css")
const SKIP_AUDIENCES_USER = new Set(["internal", "agent-only"])
const SKIP_AUDIENCES_INTERNAL = new Set(["agent-only"])

function extractField(block: string, field: string): string | null {
  const m = block.match(new RegExp(`^${field}:\\s*(.+)$`, "m"))
  return m ? m[1].trim().replace(/^["']|["']$/g, "") : null
}

function parseFrontmatter(text: string) {
  const m = text.match(/^---\n([\s\S]*?)\n---/)
  if (!m) return {}
  const b = m[1]
  return {
    title: extractField(b, "title"),
    description: extractField(b, "description"),
    audience: extractField(b, "audience"),
    group: extractField(b, "group"),
    order: extractField(b, "order"),
  }
}

function stripFrontmatter(text: string): string {
  return text.replace(/^---\n[\s\S]*?\n---\n*/, "").trimStart()
}

/** Split into frontmatter (including delimiters + trailing newline) and body. */
function splitFrontmatter(text: string): { frontmatter: string; body: string } {
  const m = text.match(/^---\n[\s\S]*?\n---\n*/)
  if (!m) return { frontmatter: "", body: text }
  return { frontmatter: m[0], body: text.slice(m[0].length) }
}

/**
 * HTML comment stamped at the top of every generated file's body. Survives
 * Markdoc/Markdown rendering as a hidden HTML comment in the output, so an
 * integrator who hits "View Source" in the browser sees where to actually
 * edit. Also visible immediately in any editor that opens the generated
 * file.
 */
function autoGenBanner(sourcePath: string): string {
  return (
    `<!-- AUTO-GENERATED by apps/docs/scripts/collect-skill-docs.ts.\n` +
    `     Source: ${sourcePath}\n` +
    `     Edits to this file are blown away on the next watcher tick. -->\n\n`
  )
}

/** Remove the first h1 if present — Starlight renders its own from title. */
function stripLeadingH1(text: string): string {
  return text.replace(/^#\s+.+\n+/, "")
}

/**
 * Block-level audience filtering.
 *
 *   {% internal %} ... {% endinternal %}   contributor-only (internals).
 *                                          Stripped in user mode, kept in
 *                                          internal mode.
 *
 *   {% user-only %} ... {% enduser-only %} engaging visuals for users.
 *                                          Kept in both site modes. Agent
 *                                          stripping happens compiler-side
 *                                          on `ship use` (TODO — not here).
 *
 * Mode is driven by the AUDIENCE env var: "internal" keeps everything; anything
 * else (default) strips {% internal %} blocks and internal-audience pages.
 */
function applyAudienceBlocks(text: string, audience: string): string {
  // {% internal %}...{% endinternal %} — drop block + markers in user mode;
  // keep content but drop markers in internal mode.
  let out = audience === "internal"
    ? text.replace(/\{%\s*(end)?internal\s*%\}\n?/g, "")
    : text.replace(/\{%\s*internal\s*%\}[\s\S]*?\{%\s*endinternal\s*%\}\n*/g, "")
  // {% user-only %}...{% enduser-only %} — always keep content, strip markers.
  // (Agent-side stripping is compiler-side, not this script.)
  out = out.replace(/\{%\s*(end)?user-only\s*%\}\n?/g, "")
  return out
}

function esc(s: string): string {
  return `"${s.replace(/"/g, '\\"')}"`
}

function buildStarlightFrontmatter(meta: {
  title: string
  description: string | null
  label: string | null
  order: string | null
}): string {
  const lines = ["---", `title: ${esc(meta.title)}`]
  if (meta.description) lines.push(`description: ${esc(meta.description)}`)
  if (meta.label || meta.order) {
    lines.push("sidebar:")
    if (meta.label) lines.push(`  label: ${esc(meta.label)}`)
    if (meta.order) lines.push(`  order: ${meta.order}`)
  }
  lines.push("---", "")
  return lines.join("\n")
}

/**
 * Slug a `group:` value into a (possibly nested) output directory. Spaces
 * become dashes; a `: ` segment separator becomes a path separator so the
 * generated SDK pages (`group: "Ops: git"`) nest under `ops/git/` and a
 * single Starlight `autogenerate: { directory: "ops" }` topic builds the
 * per-domain subgroups. Flat groups ("CLI", "Smart Skills") are unaffected.
 */
function groupSlug(group: string): string {
  return group
    .toLowerCase()
    .replace(/:\s*/g, "/")
    .replace(/\s+/g, "-")
}

/**
 * Walk one doc-skill root (`<root>/<skill>/references/docs/*.{md,mdoc}`),
 * emit every page carrying a `group:` frontmatter into GENERATED_DIR under
 * its group slug. Returns the page count. `srcPrefix` is the repo-relative
 * prefix stamped into each file's AUTO-GENERATED banner.
 */
function collectSkillRoot(rootDir: string, srcPrefix: string, audience: string): number {
  let count = 0
  const skillDirs = readdirSync(rootDir, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)

  for (const skillId of skillDirs) {
    const docsDir = join(rootDir, skillId, "references/docs")
    if (!existsSync(docsDir)) continue

    // Collect .md and .mdoc files that have a group: field (project docs)
    const allFiles = readdirSync(docsDir, { withFileTypes: true })
      .filter((e) => e.isFile() && /\.(md|mdoc)$/.test(e.name))
      .map((e) => e.name)

    // Only include files with a `group:` frontmatter field — that's the
    // doc-skill signal. `parseFrontmatter` yields `null` for a present-but-
    // empty `group:` and `undefined` when the file has no frontmatter at all
    // (e.g. a plain `# Title` ADR); `!= null` excludes both so frontmatterless
    // pages never leak into a `<skillId>/` orphan dir.
    const docFiles = allFiles.filter((file) => {
      const text = readFileSync(join(docsDir, file), "utf8")
      const fm = parseFrontmatter(text)
      return fm.group != null
    })

    if (docFiles.length === 0) continue

    for (const file of docFiles) {
      const text = readFileSync(join(docsDir, file), "utf8")
      const meta = parseFrontmatter(text)

      const skipSet = audience === "internal" ? SKIP_AUDIENCES_INTERNAL : SKIP_AUDIENCES_USER
      if (skipSet.has(meta.audience ?? "public")) continue

      const title = meta.title ?? skillId
      const group = meta.group ?? skillId
      const isIndex = basename(file, ".mdoc") === "index"
      const label = isIndex ? null : (meta.title ?? null)

      // Output path: <GENERATED_DIR>/<group-slug>/<filename>.mdoc. GENERATED_DIR
      // is apps/docs/src/content/docs (Starlight's fixed collection root) for a
      // real run, or the drift checker's tempdir via COLLECT_DOCS_OUT.
      const groupDir = join(GENERATED_DIR, groupSlug(group))
      mkdirSync(groupDir, { recursive: true })

      const body = applyAudienceBlocks(stripLeadingH1(stripFrontmatter(text)), audience)
      const fm = buildStarlightFrontmatter({
        title,
        description: meta.description ?? null,
        label,
        order: meta.order ?? null,
      })

      const outPath = join(groupDir, file)
      const sourceRel = `${srcPrefix}/${skillId}/references/docs/${file}`
      writeFileSync(outPath, fm + autoGenBanner(sourceRel) + body, "utf8")
      count++
    }

    console.log(`[collect-docs] ${srcPrefix}/${skillId} → ${docFiles.length} page(s)`)
  }
  return count
}

function run(): void {
  if (!existsSync(SKILLS_DIR)) {
    console.log("[collect-docs] No skills directory, skipping.")
    return
  }

  const audience = process.env.AUDIENCE === "internal" ? "internal" : "user"
  console.log(`[collect-docs] audience=${audience}`)

  // Delete-then-regen (idempotency): the entire GENERATED_DIR tree is a build
  // artifact derived from `docs/<skill>/references/docs/` + content-source/.
  // Wiping it first guarantees a clean run produces exactly the current source
  // set — no stale orphans survive (e.g. a source that loses its `group:`
  // frontmatter, or a renamed group dir). Hand-authored content is safe: it
  // lives in src/content-source/ (outside GENERATED_DIR) and is copied back in
  // below.
  rmSync(GENERATED_DIR, { recursive: true, force: true })
  mkdirSync(GENERATED_DIR, { recursive: true })

  let total = 0

  for (const root of SKILL_ROOTS) {
    if (!existsSync(root.dir)) continue
    total += collectSkillRoot(root.dir, root.srcPrefix, audience)
  }

  // Component reference — one page per public pack component, generated from
  // the packs' CEM output into `components/<family>/` inside the same
  // delete-then-regen tree (so the drift checker covers these pages too).
  total += collectComponentDocs(GENERATED_DIR)

  // Copy hand-authored content from src/content-source/ into the same
  // generated tree so Starlight's content collection (which points only
  // at GENERATED_DIR) sees a single unified set of entries. Today the
  // only hand-authored file is index.md — the docs homepage. Anything
  // dropped into src/content-source/ goes through this pass on every
  // run, with the same auto-gen banner stamped on top so an integrator
  // who opens it gets pointed back at the source path.
  if (existsSync(HAND_AUTHORED_DIR)) {
    mkdirSync(GENERATED_DIR, { recursive: true })
    for (const name of readdirSync(HAND_AUTHORED_DIR)) {
      const src = join(HAND_AUTHORED_DIR, name)
      const dst = join(GENERATED_DIR, name)
      const text = readFileSync(src, "utf8")
      const { frontmatter, body } = splitFrontmatter(text)
      const sourceRel = `apps/docs/src/content-source/${name}`
      writeFileSync(dst, frontmatter + autoGenBanner(sourceRel) + body, "utf8")
      total++
    }
  }

  console.log(`[collect-docs] Total: ${total} pages collected.`)

  mkdirSync(PUBLIC_DIR, { recursive: true })
  let copied = 0
  for (const name of SUI_BUNDLES) {
    const src = join(SUI_ASSETS_DIR, name)
    if (!existsSync(src)) {
      console.log(`[collect-docs] ${name} not found at ${src}. Bake the framework bundles first: 'cargo run -p deck-cli -- build-assets'.`)
      continue
    }
    copyFileSync(src, join(PUBLIC_DIR, name))
    copied++
  }
  if (copied > 0) console.log(`[collect-docs] Copied ${copied}/${SUI_BUNDLES.length} SUI bundles to public/.`)

  const apiAssetsDir = join(PUBLIC_DIR, "api", "assets")
  mkdirSync(apiAssetsDir, { recursive: true })
  if (existsSync(VIEW_TOKENS_SOURCE)) {
    copyFileSync(VIEW_TOKENS_SOURCE, join(apiAssetsDir, "view-tokens.css"))
    console.log(`[collect-docs] Emitted /api/assets/view-tokens.css.`)
  } else {
    console.log(`[collect-docs] view-tokens source missing: ${VIEW_TOKENS_SOURCE}`)
  }
  for (const name of SUI_BUNDLES) {
    const src = join(SUI_ASSETS_DIR, name)
    if (existsSync(src)) copyFileSync(src, join(apiAssetsDir, name))
  }

  // `<sui-icon>` fetches each lucide icon from `/_deck/icons/<name>.svg`.
  // The docs site is a static deploy with no daemon — split the committed
  // JSON into one file per icon so the same per-icon URLs resolve here.
  const sUiDir = join(PUBLIC_DIR, "_deck")
  mkdirSync(sUiDir, { recursive: true })
  const lucideSrc = join(SUI_ASSETS_DIR, "lucide-icons.json")
  if (existsSync(lucideSrc)) {
    const iconsDir = join(sUiDir, "icons")
    mkdirSync(iconsDir, { recursive: true })
    const wrap = (inner: string) =>
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" ` +
      `stroke="currentColor" stroke-width="2" stroke-linecap="round" ` +
      `stroke-linejoin="round">${inner}</svg>`
    const map: Record<string, string> = JSON.parse(readFileSync(lucideSrc, "utf8"))
    for (const [name, inner] of Object.entries(map)) {
      writeFileSync(join(iconsDir, `${name}.svg`), wrap(inner))
    }
    console.log(`[collect-docs] Emitted ${Object.keys(map).length} icons under /_deck/icons/.`)
  }

}

run()

if (process.argv.includes("--watch")) {
  console.log(`[collect-docs] watching ${SKILLS_DIR} for changes...`)
  let pending: NodeJS.Timeout | null = null
  const schedule = (filename: string | null) => {
    if (!filename || !/\.(md|mdoc)$/.test(filename)) return
    if (pending) clearTimeout(pending)
    pending = setTimeout(() => {
      console.log(`[collect-docs] change: ${filename} — re-running`)
      try { run() } catch (e) { console.error(`[collect-docs] ${e}`) }
    }, 150)
  }
  watch(SKILLS_DIR, { recursive: true }, (_event, filename) => schedule(filename))
  // Keep the process alive.
  setInterval(() => {}, 1 << 30)
}
