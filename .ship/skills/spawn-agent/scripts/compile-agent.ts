#!/usr/bin/env bun
// Portable agent compiler — reads ONLY the canonical ship source
// (`<ship-root>/.ship/`), never the target repo, never `~/.ship`.
// Writes the standard harness output set (CLAUDE.md + .claude/skills/* +
// .mcp.json) into a worktree so dispatched agents have a complete config
// even when the target repo is not docked.
//
// Mirrors `ship use` output shape — see compiler/src/compile/{mcp.rs,
// context.rs, harness.rs} for the source of truth we copy.
//
// Usage:
//   bun compile-agent.ts \
//     --agent <id> --out <worktree-path> \
//     [--ship-root <path>] [--with <skill-id> ...] \
//     [--harness <claude|omp|codex|...>] [--dry-run]
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

// ── Args ────────────────────────────────────────────────────────────────────

type Args = {
  agent: string;
  out: string;
  shipRoot: string;
  with: string[];
  harness: string;
  dryRun: boolean;
};

function parseArgs(argv: string[]): Args {
  const out: Args = {
    agent: "",
    out: "",
    shipRoot: "",
    with: [],
    harness: "claude",
    dryRun: false,
  };
  for (let i = 0; i < argv.length; ) {
    const a = argv[i];
    const next = () => {
      const v = argv[i + 1];
      if (v === undefined) throw new Error(`flag ${a} requires a value`);
      i += 2;
      return v;
    };
    switch (a) {
      case "--agent":    out.agent = next(); break;
      case "--out":      out.out = next(); break;
      case "--ship-root": out.shipRoot = next(); break;
      case "--with":     out.with.push(next()); break;
      case "--harness":  out.harness = next(); break;
      case "--dry-run":  out.dryRun = true; i++; break;
      case "-h":
      case "--help": {
        process.stderr.write(
          "Usage: bun compile-agent.ts --agent <id> --out <path> " +
          "[--ship-root <path>] [--with <skill> ...] " +
          "[--harness <id>] [--dry-run]\n",
        );
        process.exit(0);
      }
      default:
        throw new Error(`unknown flag: ${a}`);
    }
  }
  if (!out.agent) throw new Error("--agent is required");
  if (!out.out) throw new Error("--out is required");
  return out;
}

// ── JSONC (JSON with comments) ──────────────────────────────────────────────

/** Strip // and /* *​/ comments + trailing commas, preserving strings. */
function stripJsonc(src: string): string {
  let out = "";
  let i = 0;
  let inStr = false;
  let quote = "";
  while (i < src.length) {
    const c = src[i];
    const n = src[i + 1];
    if (inStr) {
      out += c;
      if (c === "\\" && i + 1 < src.length) {
        out += n;
        i += 2;
        continue;
      }
      if (c === quote) inStr = false;
      i++;
      continue;
    }
    if (c === '"' || c === "'") {
      inStr = true;
      quote = c;
      out += c;
      i++;
      continue;
    }
    if (c === "/" && n === "/") {
      while (i < src.length && src[i] !== "\n") i++;
      continue;
    }
    if (c === "/" && n === "*") {
      i += 2;
      while (i < src.length && !(src[i] === "*" && src[i + 1] === "/")) i++;
      i += 2;
      continue;
    }
    out += c;
    i++;
  }
  // Strip trailing commas.
  return out.replace(/,(\s*[}\]])/g, "$1");
}

function readJsonc<T = any>(path: string): T {
  const raw = readFileSync(path, "utf8");
  return JSON.parse(stripJsonc(raw)) as T;
}

// ── Frontmatter (YAML) for SKILL.md ─────────────────────────────────────────

/** Parse a tiny subset of YAML frontmatter: string keys, list values. */
function parseFrontmatter(src: string): Record<string, string | string[]> {
  const fm: Record<string, string | string[]> = {};
  if (!src.startsWith("---")) return fm;
  const end = src.indexOf("\n---", 3);
  if (end < 0) return fm;
  const block = src.slice(3, end).replace(/^\n/, "");
  const lines = block.split("\n");
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const m = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!m) {
      i++;
      continue;
    }
    const key = m[1];
    const rest = m[2].trim();
    if (rest === "" || rest === "|" || rest === ">") {
      // Block-form list: `key:\n  - a\n  - b`.
      const items: string[] = [];
      i++;
      while (i < lines.length) {
        const li = lines[i].match(/^\s+-\s+(.*)$/);
        if (!li) break;
        items.push(li[1].replace(/^["']|["']$/g, "").trim());
        i++;
      }
      fm[key] = items;
      continue;
    }
    if (rest.startsWith("[") && rest.endsWith("]")) {
      fm[key] = rest
        .slice(1, -1)
        .split(",")
        .map((s) => s.trim().replace(/^["']|["']$/g, ""))
        .filter((s) => s.length > 0);
      i++;
      continue;
    }
    fm[key] = rest.replace(/^["']|["']$/g, "");
    i++;
  }
  return fm;
}

// ── Resolution helpers ──────────────────────────────────────────────────────

type Agent = {
  agent: { id: string; name?: string; description?: string };
  skills: { refs: string[] };
  mcp: { servers: string[] };
  rules: { inline: string };
  permissions?: { preset?: string; tools_deny?: string[] };
  plugins?: { install?: string[]; scope?: string };
};

type McpServer = {
  name: string;
  command: string;
  args: string[];
  server_type: "stdio" | "http" | "sse";
  url?: string;
  disabled?: boolean;
  env?: Record<string, string>;
  timeout_secs?: number;
};

function resolveShipRoot(flagValue: string): string {
  if (flagValue) return resolve(flagValue);
  // Default: this script lives at <ship>/.ship/skills/spawn-agent/scripts/.
  // Path components: ship, .ship, skills, spawn-agent, scripts, compile-agent.ts
  // → 5 dirname() calls land us on <ship>. If the result doesn't look like
  // a ship source (no .ship/agents/), fail with a clear error so the
  // caller can pass --ship-root explicitly.
  const here = new URL(import.meta.url).pathname;
  let dir = here;
  for (let i = 0; i < 5; i++) dir = dirname(dir);
  return dir;
}

function readAgent(shipRoot: string, id: string): Agent {
  const p = join(shipRoot, ".ship", "agents", `${id}.jsonc`);
  if (!existsSync(p)) {
    throw new Error(
      `agent '${id}' not found at ${p} (only ships in <ship-root>/.ship/agents/ are valid)`,
    );
  }
  return readJsonc<Agent>(p);
}

function listAgentIds(shipRoot: string): string[] {
  const dir = join(shipRoot, ".ship", "agents");
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((f) => f.endsWith(".jsonc"))
    .map((f) => f.replace(/\.jsonc$/, ""))
    .sort();
}

function readMcpServers(shipRoot: string): Record<string, McpServer> {
  const p = join(shipRoot, ".ship", "mcp.jsonc");
  if (!existsSync(p)) {
    throw new Error(`mcp.jsonc not found at ${p}`);
  }
  const doc = readJsonc<{ mcp: { servers: Record<string, McpServer> } }>(p);
  return doc.mcp.servers;
}

function readSkill(shipRoot: string, ref: string): {
  raw: string;
  fm: Record<string, string | string[]>;
  path: string;
} {
  const p = join(shipRoot, ".ship", "skills", ref, "SKILL.md");
  if (!existsSync(p)) {
    throw new Error(`skill '${ref}' not found at ${p}`);
  }
  const raw = readFileSync(p, "utf8");
  return { raw, fm: parseFrontmatter(raw), path: p };
}

// ── Placeholder render ─────────────────────────────────────────────────────

/** Render `{{ runtime.* }}` placeholders. Unknown ones are dropped (no
 *  literal `{{ }}` survives in the output). Mirrors the simple known
 *  set in the job spec. */
function renderPlaceholders(
  text: string,
  ctx: { model: string; agents: string; skills: string },
): string {
  const dict: Record<string, string> = {
    "runtime.model": ctx.model,
    "runtime.agents": ctx.agents,
    "runtime.skills": ctx.skills,
  };
  // Only touch placeholders that are explicitly `{{ runtime.<name> }}` —
  // do NOT strip other `{{ ... }}` (Svelte transition syntax
  // `transition:fade={{ duration: 200 }}` is a common false positive).
  // Unknown `runtime.*` placeholders drop to empty; everything else
  // passes through verbatim.
  return text.replace(
    /\{\{\s*(runtime\.[a-zA-Z0-9_.]+)\s*\}\}/g,
    (_m, key) => (key in dict ? dict[key] : ""),
  );
}

// ── Skill resolution ───────────────────────────────────────────────────────

/** Resolve an agent's skill refs + `--with` extras, including best-effort
 *  `requires:` → `grants:` dependency closure. Logs warnings for unresolved
 *  requires (does not fail the compile — job spec says best-effort). */
function resolveSkills(
  shipRoot: string,
  agentSkills: string[],
  extras: string[],
): { ids: string[]; warnings: string[] } {
  const warnings: string[] = [];
  const seen = new Set<string>();
  const granted = new Map<string, string>(); // capability → skill id

  // First pass: collect all `grants:` from candidate skills so we can
  // resolve `requires:` later. We only scan the skills that the agent
  // actually references (or adds via --with) plus any that the dep closure
  // pulls in.
  const queue: string[] = [...agentSkills, ...extras];
  for (const ref of queue) {
    if (seen.has(ref)) continue;
    seen.add(ref);
    const s = readSkill(shipRoot, ref);
    for (const g of asList(s.fm.grants)) granted.set(g, ref);
  }

  // Second pass: for each resolved skill, pull in any other skill that
  // `grants:` a capability it `requires:`. Iterate to fixed point.
  let changed = true;
  const all = new Set(seen);
  while (changed) {
    changed = false;
    for (const ref of [...all]) {
      const s = readSkill(shipRoot, ref);
      for (const cap of asList(s.fm.requires)) {
        const grantor = granted.get(cap);
        if (grantor && !all.has(grantor)) {
          all.add(grantor);
          // Also fold the new grantor's grants into the map.
          const gs = readSkill(shipRoot, grantor);
          for (const g of asList(gs.fm.grants)) granted.set(g, grantor);
          changed = true;
        } else if (!grantor) {
          warnings.push(
            `skill '${ref}' requires capability '${cap}' — no skill in scope grants it; ignoring`,
          );
        }
      }
    }
  }

  // Preserve agent-declared order, then extras, then auto-resolved deps.
  const ordered: string[] = [];
  const seen2 = new Set<string>();
  for (const s of [...agentSkills, ...extras]) {
    if (seen2.has(s)) continue;
    ordered.push(s);
    seen2.add(s);
  }
  for (const s of all) {
    if (seen2.has(s)) continue;
    ordered.push(s);
    seen2.add(s);
  }
  return { ids: ordered, warnings };
}

function asList(v: string | string[] | undefined): string[] {
  if (v === undefined) return [];
  if (Array.isArray(v)) return v;
  return [v];
}

// ── Output builders ────────────────────────────────────────────────────────

/** Build the .mcp.json payload for an agent. Matches the `ship use` shape
 *  for Claude: `{mcpServers:{...}}` with stdio / http entries that mirror
 *  `compiler/src/compile/mcp.rs::server_entry`. */
function buildMcpConfig(
  servers: Record<string, McpServer>,
  agentWants: string[],
): {
  payload: string;
  resolved: { id: string; type: string; entry: Record<string, unknown> }[];
} {
  const out: Record<string, unknown> = {};
  const resolved: { id: string; type: string; entry: Record<string, unknown> }[] = [];
  for (const id of agentWants) {
    const s = servers[id];
    if (!s) continue;
    if (s.disabled) continue;
    let entry: Record<string, unknown> = {};
    let type: string;
    if (s.server_type === "stdio") {
      entry = {
        command: s.command,
        ...(s.args.length > 0 ? { args: s.args } : {}),
        ...(s.env && Object.keys(s.env).length > 0 ? { env: s.env } : {}),
      };
      type = "stdio";
    } else if (s.server_type === "http" || s.server_type === "sse") {
      entry = { url: s.url ?? "" };
      type = s.server_type;
    } else {
      entry = {};
      type = "unknown";
    }
    if (s.timeout_secs !== undefined) {
      entry.startup_timeout_sec = s.timeout_secs;
    }
    out[id] = entry;
    resolved.push({ id, type, entry });
  }
  // Match the spec's requested top-level shape: a `_ship` marker that
  // downstream tooling can detect, plus the `mcpServers` map. The
  // marker is portable-compile specific (different note from ship use)
  // so consumers can tell which compiler produced the file.
  const payload = {
    _ship: { managed: true, note: "portable compile" },
    mcpServers: out,
  };
  return { payload: JSON.stringify(payload, null, 2) + "\n", resolved };
}

function buildClaudeMd(
  agent: Agent,
  skills: { ids: string[]; warnings: string[] },
  ctx: { agents: string },
): string {
  const rulesBody = agent.rules?.inline ?? "";
  const rendered = renderPlaceholders(rulesBody, {
    model: "sonnet",
    agents: ctx.agents,
    skills: skills.ids.join(", "),
  });
  const marker = `<!-- ship: agent=${agent.agent.id} (portable compile) -->`;
  const warnings =
    skills.warnings.length > 0
      ? "\n\n<!-- ship-warnings:\n" +
        skills.warnings.map((w) => `  - ${w}`).join("\n") +
        "\n-->\n"
      : "";
  return `${marker}\n\n${rendered.trim()}\n${warnings}`;
}

// ── Main ───────────────────────────────────────────────────────────────────

function main(): void {
  const args = parseArgs(process.argv.slice(2));
  const shipRoot = resolveShipRoot(args.shipRoot);
  if (!existsSync(join(shipRoot, ".ship", "agents"))) {
    throw new Error(
      `ship root '${shipRoot}' has no .ship/agents/ — pass --ship-root <path>`,
    );
  }

  const agent = readAgent(shipRoot, args.agent);
  const allAgentIds = listAgentIds(shipRoot);
  const skills = resolveSkills(shipRoot, agent.skills?.refs ?? [], args.with);
  const mcpServers = readMcpServers(shipRoot);
  const mcp = buildMcpConfig(mcpServers, agent.mcp?.servers ?? []);

  const outDir = resolve(args.out);
  const claudeMd = buildClaudeMd(agent, skills, { agents: allAgentIds.join(", ") });
  const mcpJson = mcp.payload;

  process.stdout.write(`agent:        ${agent.agent.id}\n`);
  process.stdout.write(`ship-root:    ${shipRoot}\n`);
  process.stdout.write(`out:          ${outDir}\n`);
  process.stdout.write(`harness:      ${args.harness}\n`);
  process.stdout.write(`skills (${skills.ids.length}):  ${skills.ids.join(", ")}\n`);
  process.stdout.write(
    `mcp servers (${mcp.resolved.length}): ${mcp.resolved.map((s) => `${s.id}:${s.type}`).join(", ")}\n`,
  );
  if (skills.warnings.length > 0) {
    for (const w of skills.warnings) process.stdout.write(`warn: ${w}\n`);
  }

  if (args.dryRun) {
    process.stdout.write("(dry-run — no files written)\n");
    return;
  }

  // Always write CLAUDE.md — Claude and omp both read it. (omp uses
  // CLAUDE.md as its compiled-config format too.)
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "CLAUDE.md"), claudeMd);

  // Always write .mcp.json — Claude reads it; omp picks it up via the
  // `mcp` config surface too.
  writeFileSync(join(outDir, ".mcp.json"), mcpJson);

  // Per-skill copies. Claude discovers `.claude/skills/<id>/SKILL.md`;
  // omp reads the same path. Empty content is fine (skips missing).
  for (const ref of skills.ids) {
    const { raw } = readSkill(shipRoot, ref);
    const dir = join(outDir, ".claude", "skills", ref);
    mkdirSync(dir, { recursive: true });
    const rendered = renderPlaceholders(raw, {
      model: "sonnet",
      agents: allAgentIds.join(", "),
      skills: skills.ids.join(", "),
    });
    writeFileSync(join(dir, "SKILL.md"), rendered);
  }

  process.stdout.write(
    `wrote: ${join(outDir, "CLAUDE.md")}, ${join(outDir, ".mcp.json")}, ` +
      `${skills.ids.length} skill(s) → ${join(outDir, ".claude", "skills")}\n`,
  );
}

try {
  main();
} catch (e) {
  process.stderr.write(`error: ${(e as Error).message}\n`);
  process.exit(1);
}
