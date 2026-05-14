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
  readdirSync, copyFileSync, watch, unlinkSync, rmdirSync,
} from "fs"
import { join, resolve, dirname, basename } from "path"

const SCRIPT_DIR = dirname(new URL(import.meta.url).pathname)
const REPO_ROOT = resolve(SCRIPT_DIR, "../../../")
const SKILLS_DIR = join(REPO_ROOT, "docs")
const OUTPUT_DIR = join(SCRIPT_DIR, "../src/content/docs")
// Output is treated as build artifact (gitignored). Override via
// COLLECT_DOCS_OUT for the drift checker which collects into a tempdir.
//
// Default canonical target: `apps/docs/src/content/docs`. Per-group
// output lands in `<group>/` subdirectories directly under it — the
// astro.config.mjs topic `autogenerate` paths resolve against the same
// flat layout. The drift checker walks the same dir.
const GENERATED_DIR = process.env.COLLECT_DOCS_OUT
  ? resolve(process.env.COLLECT_DOCS_OUT)
  : OUTPUT_DIR
const PUBLIC_DIR = join(SCRIPT_DIR, "../public")
const SHIP_ELEMENTS_SRC = join(REPO_ROOT, "apps/shipd/assets/ship-elements.js")
const SHIP_ELEMENTS_DEST = join(PUBLIC_DIR, "ship-elements.js")
// `agent-only` files never ship to any site. `internal` files are hidden from
// the public (user) build but rendered in internal mode so contributors can
// view internal-audience docs locally.
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

function run(): void {
  if (!existsSync(SKILLS_DIR)) {
    console.log("[collect-docs] No skills directory, skipping.")
    return
  }

  const audience = process.env.AUDIENCE === "internal" ? "internal" : "user"
  console.log(`[collect-docs] audience=${audience}`)

  mkdirSync(GENERATED_DIR, { recursive: true })
  // Track what we write this run. Files inside GENERATED_DIR that aren't in
  // this set at the end get removed — lets us avoid a full rmSync that would
  // race with Astro's file handles.
  const written = new Set<string>()

  const skillDirs = readdirSync(SKILLS_DIR, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name)

  let total = 0

  for (const skillId of skillDirs) {
    const docsDir = join(SKILLS_DIR, skillId, "references/docs")
    if (!existsSync(docsDir)) continue

    // Collect .md and .mdoc files that have a group: field (project docs)
    const allFiles = readdirSync(docsDir, { withFileTypes: true })
      .filter((e) => e.isFile() && /\.(md|mdoc)$/.test(e.name))
      .map((e) => e.name)

    // Only include files with group: frontmatter — that's the doc-skill signal
    const docFiles = allFiles.filter((file) => {
      const text = readFileSync(join(docsDir, file), "utf8")
      const fm = parseFrontmatter(text)
      return fm.group !== null
    })

    if (docFiles.length === 0) continue

    // Read SKILL.md for fallback metadata
    const skillMdPath = join(SKILLS_DIR, skillId, "SKILL.md")
    const skillMeta = existsSync(skillMdPath)
      ? parseFrontmatter(readFileSync(skillMdPath, "utf8"))
      : {}

    for (const file of docFiles) {
      const text = readFileSync(join(docsDir, file), "utf8")
      const meta = parseFrontmatter(text)

      const skipSet = audience === "internal" ? SKIP_AUDIENCES_INTERNAL : SKIP_AUDIENCES_USER
      if (skipSet.has(meta.audience ?? "public")) continue

      const title = meta.title ?? skillId
      const group = meta.group ?? skillId
      const isIndex = basename(file, ".mdoc") === "index"
      const label = isIndex ? null : (meta.title ?? null)

      // Output path: <GENERATED_DIR>/<group>/<filename>.mdoc.
      // In production GENERATED_DIR is `apps/docs/src/content/docs/reference`
      // (the drift checker walks that path). The collector default
      // `OUTPUT_DIR = apps/docs/src/content/docs` is the WRONG path —
      // overridden via COLLECT_DOCS_OUT by the checker and by the
      // OUTPUT_DIR-via-`reference/` resolution below.
      const groupDir = join(GENERATED_DIR, group.toLowerCase().replace(/\s+/g, "-"))
      mkdirSync(groupDir, { recursive: true })

      const body = applyAudienceBlocks(stripLeadingH1(stripFrontmatter(text)), audience)
      const fm = buildStarlightFrontmatter({
        title,
        description: meta.description ?? null,
        label,
        order: meta.order ?? null,
      })

      const outPath = join(groupDir, file)
      writeFileSync(outPath, fm + body, "utf8")
      written.add(outPath)
      total++
    }

    console.log(`[collect-docs] ${skillId} → ${docFiles.length} page(s)`)
  }

  // Sweep orphans: any file under GENERATED_DIR not written this run.
  // Files at the root of GENERATED_DIR are user-committed (e.g. the homepage
  // index.md) and never get swept — only generated group subdirectories do.
  const sweep = (dir: string, isRoot: boolean): boolean => {
    if (!existsSync(dir)) return true
    let isEmpty = true
    for (const e of readdirSync(dir, { withFileTypes: true })) {
      const p = join(dir, e.name)
      if (e.isDirectory()) {
        if (!sweep(p, false)) isEmpty = false
      } else if (isRoot) {
        isEmpty = false
      } else if (written.has(p)) {
        isEmpty = false
      } else {
        try { unlinkSync(p) } catch { /* handle in flight — retry next run */ }
      }
    }
    if (isEmpty && dir !== GENERATED_DIR) {
      try { rmdirSync(dir) } catch { /* non-empty due to race */ }
    }
    return isEmpty
  }
  sweep(GENERATED_DIR, true)

  console.log(`[collect-docs] Total: ${total} pages collected.`)

  if (existsSync(SHIP_ELEMENTS_SRC)) {
    mkdirSync(PUBLIC_DIR, { recursive: true })
    copyFileSync(SHIP_ELEMENTS_SRC, SHIP_ELEMENTS_DEST)
    console.log(`[collect-docs] Copied ship-elements.js to public/.`)
  } else {
    console.log(`[collect-docs] ship-elements.js not found at ${SHIP_ELEMENTS_SRC}. Run 'cd packages/ui && bun run bundle:elements' first.`)
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
