#!/usr/bin/env bun
/**
 * OpenAPI augmentation — x-codeSamples injection.
 *
 * Reads an OpenAPI spec from stdin (or a file), walks every operation, and
 * injects an `x-codeSamples` array with curl + JavaScript fetch snippets.
 * Writes the augmented spec to stdout (or a file).
 *
 * Sequencing — this is the single source of truth for the committed
 * `apps/docs/openapi.json`:
 *
 *   cargo run ship openapi export --pretty  → RAW SPEC
 *                                           │
 *                                           ▼
 *                                    augment-openapi  ← this script
 *                                           │
 *                                           ▼
 *                             apps/docs/openapi.json (COMMITTED)
 *
 * Both `pnpm run openapi:generate` and `check-openapi-drift` run this
 * augmentation on the raw cargo output before writing / comparing. As long
 * as the script is deterministic (it is — fixed ordering, no clock / rng),
 * drift-check stays exact.
 *
 * Public endpoints (those with an empty `security: []` override) drop the
 * `Authorization` header. Path parameters are substituted with a realistic
 * placeholder (`{id}` → `01J3Z5ZQXAM3E4T7W`, `{slug}` → `example-slug`, …).
 * Request-body endpoints use the schema's `example` value when present, else
 * an auto-generated skeleton derived from the `$ref`ed schema.
 */
import { readFileSync, writeFileSync } from "node:fs";

type Json = any;

const BASE_URL = "http://localhost:9315";
const METHODS = ["get", "post", "put", "patch", "delete", "head", "options"] as const;
type HttpMethod = (typeof METHODS)[number];

/** Realistic path-parameter placeholders. */
const PARAM_PLACEHOLDERS: Record<string, string> = {
  id: "01J3Z5ZQXAM3E4T7W",
  job_id: "01J3Z5ZQXAM3E4T7W",
  artifact_id: "01J3Z5ZQXAM3E4T7W",
  session_id: "sess-2f9c",
  workspace_id: "default",
  agent_id: "agent-writer",
  spec_id: "write-readme",
  slug: "write-readme",
  name: "write-readme",
  collection: "notes",
  item_id: "intro",
  claim_token: "pt_01J3Z5ZQX",
  path: "webhook-example",
  view_id: "chat",
  skill_id: "ship-writing",
  key: "defaults.provider",
};

function fillPath(path: string): string {
  return path.replace(/\{([^}]+)\}/g, (_, name) => {
    return PARAM_PLACEHOLDERS[name] ?? `example-${name}`;
  });
}

/** Derive a compact request-body example from the spec.
 *
 *  Order of precedence per OpenAPI spec:
 *  1. `requestBody.content[*].example`
 *  2. `requestBody.content[*].schema.example` (inline or resolved `$ref`)
 *  3. `undefined` — no body rendered.
 */
function resolveRef(spec: Json, ref: string): Json | undefined {
  if (!ref.startsWith("#/")) return undefined;
  const parts = ref.slice(2).split("/");
  let node: Json = spec;
  for (const p of parts) {
    if (node == null) return undefined;
    node = node[p];
  }
  return node;
}

function getBodyExample(spec: Json, op: Json): Json | undefined {
  const body = op.requestBody;
  if (!body?.content) return undefined;
  for (const mt of Object.keys(body.content)) {
    const c = body.content[mt];
    if (c.example !== undefined) return c.example;
    const schema = c.schema;
    if (!schema) continue;
    if (schema.example !== undefined) return schema.example;
    if (schema.$ref) {
      const resolved = resolveRef(spec, schema.$ref);
      if (resolved?.example !== undefined) return resolved.example;
    }
  }
  return undefined;
}

function isPublic(op: Json): boolean {
  // Operation-level `security: []` → explicitly public. Any `security`
  // requirement (global default or per-operation) means a bearer is
  // required.
  if (!Array.isArray(op.security)) return false;
  return op.security.length === 0;
}

function formatCurlBody(value: Json): string {
  // Compact JSON, then single-quote-wrap for shell. Escape embedded single
  // quotes using the `'"'"'` idiom so readers can copy-paste as-is.
  const json = JSON.stringify(value);
  const escaped = json.replace(/'/g, `'"'"'`);
  return ` \\\n  -d '${escaped}'`;
}

function buildCurl(method: string, path: string, op: Json, spec: Json): string {
  const url = `${BASE_URL}${fillPath(path)}`;
  const headers: string[] = [];
  if (!isPublic(op)) {
    headers.push(`-H "Authorization: Bearer $SHIP_TOKEN"`);
  }
  const body = getBodyExample(spec, op);
  if (body !== undefined) {
    headers.push(`-H "Content-Type: application/json"`);
  }
  const methodUpper = method.toUpperCase();
  const headerStr = headers.length > 0 ? ` \\\n  ${headers.join(" \\\n  ")}` : "";
  const bodyStr = body !== undefined ? formatCurlBody(body) : "";
  return `curl -X ${methodUpper} ${url}${headerStr}${bodyStr}`;
}

function buildFetch(method: string, path: string, op: Json, spec: Json): string {
  const url = `${BASE_URL}${fillPath(path)}`;
  const hdrs: string[] = [];
  if (!isPublic(op)) {
    hdrs.push(`    "Authorization": \`Bearer \${process.env.SHIP_TOKEN}\`,`);
  }
  const body = getBodyExample(spec, op);
  if (body !== undefined) {
    hdrs.push(`    "Content-Type": "application/json",`);
  }
  const methodUpper = method.toUpperCase();
  const hdrBlock =
    hdrs.length > 0 ? `\n  headers: {\n${hdrs.join("\n")}\n  },` : "";
  const bodyBlock =
    body !== undefined
      ? `\n  body: JSON.stringify(${JSON.stringify(body, null, 2).split("\n").join("\n  ")}),`
      : "";
  return `const res = await fetch("${url}", {\n  method: "${methodUpper}",${hdrBlock}${bodyBlock}\n});\nconst data = await res.json();`;
}

/** Convert a camelCase / snake_case operationId into a human-readable title.
 *
 *  starlight-openapi renders the page heading as `operation.summary ??
 *  operationId`. Our upstream Rust exporter fills `summary` with the raw
 *  "METHOD /path — descriptor" string which reads like a descriptor, not a
 *  page title. We rewrite it here: derive a clean Title Case label from
 *  `operationId` and demote the original summary into the body description
 *  so the descriptor text is still rendered, just below the title.
 *
 *    listAgents            → "List Agents"
 *    rebuildProjections    → "Rebuild Projections"
 *    serveCodeEditor       → "Serve Code Editor"
 *    actor_spawn           → "Actor Spawn"
 */
function humanizeOperationId(id: string): string {
  if (!id) return id;
  // snake_case → space, camelCase → split before capitals.
  const spaced = id
    .replace(/[_-]+/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/\s+/g, " ")
    .trim();
  return spaced
    .split(" ")
    .map((w) => (w ? w[0].toUpperCase() + w.slice(1) : w))
    .join(" ");
}

function rewriteTitleAndDescription(op: Json): void {
  const originalSummary: string | undefined =
    typeof op.summary === "string" ? op.summary : undefined;
  const originalDescription: string =
    typeof op.description === "string" ? op.description : "";

  // The raw summary from the Rust exporter is typically a descriptor like
  // "POST /api/runtime/rebuild-projections — rebuild all projections from
  // events." or a multi-line blurb. Anything that contains a slash, an
  // em-dash, or a newline is treated as descriptor-text, not a title.
  const looksLikeDescriptor =
    !!originalSummary &&
    (/[\/\n]/.test(originalSummary) || originalSummary.includes("—"));

  if (looksLikeDescriptor && op.operationId) {
    // Move descriptor into description so it still renders in the body.
    // Prefer descriptor if we don't already have a description; otherwise
    // prepend so the explicit description wins the reading order.
    const descriptor = originalSummary.trim();
    op.description = originalDescription
      ? `${descriptor}\n\n${originalDescription}`
      : descriptor;
    op.summary = humanizeOperationId(op.operationId);
  } else if (!originalSummary && op.operationId) {
    // No summary at all — give the page a title from the operationId.
    op.summary = humanizeOperationId(op.operationId);
  }
}

function augment(spec: Json): Json {
  const paths = spec.paths ?? {};
  for (const path of Object.keys(paths).sort()) {
    const item = paths[path];
    for (const method of METHODS) {
      const op = item[method];
      if (!op || typeof op !== "object") continue;
      // Clean up title-vs-description before adding code samples so the
      // rewritten fields land in the committed spec too.
      rewriteTitleAndDescription(op);
      // `lang: "bash"` (not "curl") so expressive-code in starlight-openapi
      // has a syntax-highlighter registered. Label keeps the cURL framing for
      // readers; `x-codeSamples` consumers that filter by `lang` still get
      // a valid shell snippet.
      op["x-codeSamples"] = [
        { lang: "bash", label: "curl", source: buildCurl(method, path, op, spec) },
        { lang: "javascript", label: "fetch", source: buildFetch(method, path, op, spec) },
      ];
    }
  }
  return spec;
}

function main() {
  const args = process.argv.slice(2);
  let inPath: string | undefined;
  let outPath: string | undefined;
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--in") inPath = args[++i];
    else if (args[i] === "--out") outPath = args[++i];
  }
  const raw = inPath
    ? readFileSync(inPath, "utf8")
    : readFileSync(0, "utf8");
  const spec = JSON.parse(raw);
  augment(spec);
  const serialized = `${JSON.stringify(spec, null, 2)}\n`;
  if (outPath) writeFileSync(outPath, serialized);
  else process.stdout.write(serialized);
}

main();
