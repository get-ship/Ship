// Shared infrastructure for tests/agents/.
//
// Responsibilities:
//   - Daemon API client (auth, JSON fetch, typed responses for the surfaces
//     these tests use).
//   - Connect RPC client (replaces the deleted
//     POST /api/v1/agent_sessions/dispatch REST surface — the dispatch
//     primitive now goes through AgentsService.Dispatch in ship.ops.v1).
//   - Mesh / actor poll helpers — match the idiom from tests/loops/.
//   - Cleanup drop-guard so leaked sessions don't survive panics.
//   - PATH binary detection.
//
// Tests only import from here; no test file should hand-roll fetch calls.

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, realpathSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
// Shared prod-home refusal — mirrors the Rust fail-loud guard in
// crates/core/ship-substrate/src/project/global.rs and run.mjs's #185.
// These tests mutate live daemon kernel state; refuse to target a daemon
// whose home is the operator's prod ~/.ship (unless SHIP_LOOPS_ALLOW_PROD=1).
import { assertNotProdHome, resolveHome } from "../loops/_prod-guard.mjs";

// Per-worktree daemon discovery — matches scripts/dev/_lib.sh::daemon_dir.
//
// 1. Realpath the git toplevel for cwd (or cwd itself if not a git repo).
// 2. sha256 the path, take the first 8 hex chars.
// 3. Daemon dir = $SHIP_HOME/machine/daemons/<hash>/.
// 4. Port + token files live inside that dir.
//
// `ship status` is the canonical lookup but shells out + parses; the
// hash matches and is dependency-free.

function gitToplevel(): string {
  const r = spawnSync("git", ["rev-parse", "--show-toplevel"], {
    encoding: "utf8",
  });
  if (r.status === 0 && r.stdout) return r.stdout.trim();
  return process.cwd();
}

function daemonDir(): string {
  const root = realpathSync(gitToplevel());
  const hash = createHash("sha256").update(root).digest("hex").slice(0, 8);
  // Refuse to resolve a daemon dir under prod ~/.ship. Throws (caught nowhere)
  // so the suite aborts at module load rather than mutating prod state.
  const home = assertNotProdHome(resolveHome());
  return join(home, "machine/daemons", hash);
}

const DAEMON_DIR = daemonDir();

let cachedBaseUrl: string | null = null;
async function baseUrl(): Promise<string> {
  if (cachedBaseUrl) return cachedBaseUrl;
  if (process.env.SHIPD_URL) return (cachedBaseUrl = process.env.SHIPD_URL);
  const portPath = join(DAEMON_DIR, "port");
  try {
    const port = (await readFile(portPath, "utf8")).trim();
    return (cachedBaseUrl = `http://localhost:${port}`);
  } catch {
    throw new Error(
      `no SHIPD_URL env and no port file at ${portPath} — start the daemon for this worktree (\`just dev\`) before running tests/agents/`,
    );
  }
}

let cachedToken: string | null = null;
export async function daemonToken(): Promise<string> {
  if (cachedToken) return cachedToken;
  if (process.env.TOKEN) return (cachedToken = process.env.TOKEN);
  const tokenPath = join(DAEMON_DIR, "token");
  try {
    cachedToken = (await readFile(tokenPath, "utf8")).trim();
    return cachedToken;
  } catch {
    throw new Error(
      `no TOKEN env and no token at ${tokenPath} — start the daemon (\`just dev\`) before running tests/agents/`,
    );
  }
}

export type ApiResponse<T = unknown> = { status: number; body: T };

export async function api<T = unknown>(
  path: string,
  init: RequestInit = {},
): Promise<ApiResponse<T>> {
  const token = await daemonToken();
  const r = await fetch((await baseUrl()) + path, {
    ...init,
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      ...(init.headers ?? {}),
    },
  });
  const text = await r.text();
  let body: unknown = text;
  try {
    body = JSON.parse(text);
  } catch {
    /* leave as string */
  }
  return { status: r.status, body: body as T };
}

// ── PATH detection ─────────────────────────────────────────────────────────

const PATH_SEP = process.platform === "win32" ? ";" : ":";

export function binaryOnPath(name: string): boolean {
  const pathEnv = process.env.PATH;
  if (!pathEnv) return false;
  for (const dir of pathEnv.split(PATH_SEP)) {
    if (!dir) continue;
    if (existsSync(join(dir, name))) return true;
    if (
      process.platform === "win32" &&
      (existsSync(join(dir, `${name}.exe`)) || existsSync(join(dir, `${name}.cmd`)))
    ) {
      return true;
    }
  }
  return false;
}

// ── Connect RPC primitive ────────────────────────────────────────────────
// Replaces the deleted `POST /api/v1/agent_sessions/dispatch` REST surface.
// Connect over HTTP+JSON is just `POST /<svc>/<method>` with the protocol
// header; the body is the request message and the response is the result
// message, both as plain JSON. The Connect server in `apps/shipd/src/ops_rpc.rs`
// mounts the generated service paths; see `proto/ship/ops/v1/agents.proto`
// for the request/response shapes.

interface ConnectEnvelope<T> {
  status: number;
  body: T;
}

async function connectCall<TReq, TRes>(
  method: string,
  body: TReq,
): Promise<ConnectEnvelope<TRes>> {
  const token = await daemonToken();
  const r = await fetch(`${await baseUrl()}/ship.ops.v1.AgentsService/${method}`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${token}`,
      "Content-Type": "application/json",
      "Connect-Protocol-Version": "1",
    },
    body: JSON.stringify(body),
  });
  const text = await r.text();
  let parsed: unknown = text;
  try {
    parsed = JSON.parse(text);
  } catch {
    /* leave as string */
  }
  return { status: r.status, body: parsed as TRes };
}

// ── Dispatch ───────────────────────────────────────────────────────────────
// Connects to `AgentsService.Dispatch` (proto/ship/ops/v1/agents.proto:457).
// Replaces the deleted `POST /api/v1/agent_sessions/dispatch` REST handler.

export type Invocation = "pty" | "print" | "stream";

// The dispatch primitive replaced the old `auto: bool` with three
// orthogonal axes (see runtime_api::dispatch_request validation):
//   - open_tab:   does this open an attachable PTY surface? (UX)
//   - invocation: how the harness is invoked — pty/print/stream
//   - keepalive:  stay alive awaiting the next push after the turn?
// Defaults here match the common compat-test case: a background PTY
// harness that comes up, idles, and is cancelled — no terminal attach,
// no keepalive. Tests that need a different shape (transport, push)
// set the axes explicitly.
export interface DispatchInput {
  agent: string;
  harness: string;
  prompt?: string;
  spec?: string;
  workspace?: string;
  model?: string;
  open_tab?: boolean;
  invocation?: Invocation;
  keepalive?: boolean;
}

// Wire shape — `DispatchAgentRequest` in agents.proto. Snake-case fields
// mirror the proto exactly; Connect over HTTP+JSON uses JSON conventions.
interface DispatchAgentWire {
  agent: string;
  harness: string;
  prompt?: string;
  spec?: string;
  workspace?: string;
  model?: string;
  open_tab?: boolean;
  invocation?: Invocation;
  keepalive?: boolean;
}

// Wire shape — `DispatchResult` in agents.proto. All fields are strings
// except `resume` (bool) and `billing_warning` (optional string).
interface DispatchResultWire {
  session_id: string;
  slug: string;
  branch: string;
  resume?: boolean;
  studio_url?: string;
  billing_warning?: string;
}

export interface Dispatched {
  status: number;
  slug: string;
  sessionId: string;
  branch: string;
}

export async function dispatch(input: DispatchInput): Promise<Dispatched> {
  const wire: DispatchAgentWire = {
    agent: input.agent,
    harness: input.harness,
    prompt: input.prompt,
    spec: input.spec,
    workspace: input.workspace,
    model: input.model,
    open_tab: input.open_tab ?? false,
    invocation: input.invocation ?? "pty",
    keepalive: input.keepalive ?? false,
  };
  const { status, body } = await connectCall<DispatchAgentWire, DispatchResultWire>(
    "Dispatch",
    wire,
  );
  if (status !== 200 || !body.session_id || !body.slug) {
    throw new Error(`dispatch failed: status=${status} body=${JSON.stringify(body)}`);
  }
  return {
    status,
    slug: body.slug,
    sessionId: body.session_id,
    branch: body.branch,
  };
}

// ── Mesh / actor polling ───────────────────────────────────────────────────

export interface MeshActor {
  actor_id: string;
  kind?: string;
  live?: boolean;
}

const POLL_INTERVAL_MS = 250;
const POLL_MAX_ATTEMPTS = 60; // 15s default

export async function waitForMeshActor(
  slug: string,
  opts: { maxAttempts?: number } = {},
): Promise<MeshActor | null> {
  const max = opts.maxAttempts ?? POLL_MAX_ATTEMPTS;
  for (let i = 0; i < max; i++) {
    const { body } = await api<{
      data?: { actors?: MeshActor[] };
    }>("/api/v1/mesh");
    const hit = body.data?.actors?.find((a) => a.actor_id === slug);
    if (hit) return hit;
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
  return null;
}

export interface ActorRow {
  kind: string;
  live: boolean;
  pid: number | null;
  harness: string | null;
  transport: string | null;
  last_event_at: string | null;
}

export async function waitForActorRow(
  slug: string,
  opts: { maxAttempts?: number } = {},
): Promise<ActorRow | null> {
  const max = opts.maxAttempts ?? POLL_MAX_ATTEMPTS;
  for (let i = 0; i < max; i++) {
    const { status, body } = await api<{
      data?: {
        kind?: string;
        live?: boolean;
        pid?: number | null;
        harness?: string | null;
        transport?: string | null;
        last_event_at?: string | null;
      };
    }>(`/api/v1/actors/${slug}`);
    const a = body.data;
    if (status === 200 && a?.kind) {
      return {
        kind: a.kind,
        live: a.live ?? false,
        pid: a.pid ?? null,
        harness: a.harness ?? null,
        transport: a.transport ?? null,
        last_event_at: a.last_event_at ?? null,
      };
    }
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
  return null;
}

/**
 * Poll for an actor row where a specific field becomes populated. Use
 * this instead of `waitForActorRow` when you intend to assert on
 * harness, transport, pid, etc. The bare row arrives as soon as
 * `process.spawned` lands, but those fields only fill in once
 * `actor.created` projects, and the two events race. Predicate
 * polling closes the gap.
 *
 * Returns the row once `predicate(row)` is true, or the last observed
 * row if the deadline expires (test asserts on it and fails clearly).
 * Null if the row never materialises at all.
 */
export async function waitForActorField(
  slug: string,
  predicate: (a: ActorRow) => boolean,
  opts: { timeoutMs?: number; intervalMs?: number } = {},
): Promise<ActorRow | null> {
  const timeout = opts.timeoutMs ?? 15_000;
  const interval = opts.intervalMs ?? 250;
  const deadline = Date.now() + timeout;
  let last: ActorRow | null = null;
  while (Date.now() < deadline) {
    last = await getActorRow(slug);
    if (last && predicate(last)) return last;
    await new Promise((r) => setTimeout(r, interval));
  }
  return last;
}

/** Fetch the current actor row without polling (returns null on 404). */
export async function getActorRow(slug: string): Promise<ActorRow | null> {
  const { status, body } = await api<{
    data?: {
      kind?: string;
      live?: boolean;
      pid?: number | null;
      harness?: string | null;
      transport?: string | null;
      last_event_at?: string | null;
    };
  }>(`/api/v1/actors/${slug}`);
  const a = body.data;
  if (status !== 200 || !a?.kind) return null;
  return {
    kind: a.kind,
    live: a.live ?? false,
    pid: a.pid ?? null,
    harness: a.harness ?? null,
    transport: a.transport ?? null,
    last_event_at: a.last_event_at ?? null,
  };
}

/**
 * Poll until the actor row for `slug` is gone or no longer live — i.e.
 * the dispatched process has actually exited. Use after cancelSession
 * in keepalive tests so a long-lived harness child is confirmed dead
 * before the test yields; otherwise its teardown load bleeds into the
 * next test file and starves later dispatches (the actor.created emit
 * is a fire-and-forget thread sensitive to machine load). Best-effort:
 * returns when clear or when the deadline passes.
 */
export async function waitForActorGone(
  slug: string,
  opts: { timeoutMs?: number } = {},
): Promise<void> {
  const deadline = Date.now() + (opts.timeoutMs ?? 10_000);
  while (Date.now() < deadline) {
    const row = await getActorRow(slug);
    if (!row || row.live === false) return;
    await new Promise((r) => setTimeout(r, 250));
  }
}

export async function waitForMeshClear(
  slug: string,
  opts: { maxAttempts?: number } = {},
): Promise<boolean> {
  const max = opts.maxAttempts ?? 40; // 10s
  for (let i = 0; i < max; i++) {
    const { body } = await api<{ data?: { actors?: MeshActor[] } }>(
      "/api/v1/mesh",
    );
    const present = body.data?.actors?.some((a) => a.actor_id === slug);
    if (!present) return true;
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
  return false;
}

// ── Cleanup ────────────────────────────────────────────────────────────────

export async function cancelSession(slug: string): Promise<void> {
  await api(`/api/v1/agent_sessions/${slug}/cancel`, {
    method: "POST",
    body: JSON.stringify({ reason: "test-cleanup" }),
  }).catch(() => {
    /* best-effort */
  });
}

// ── Session message (annotations / push-to-agent) ──────────────────────────

export interface Annotation {
  kind: string; // "text" | "file" | "diff" | "comment"
  content: string;
  source?: { kind: string; ref: string };
}

export async function sendSessionMessage(
  sessionId: string,
  text: string,
  annotations: Annotation[] = [],
): Promise<{ status: number; body: unknown }> {
  return api(`/api/v1/sessions/${sessionId}/message`, {
    method: "POST",
    body: JSON.stringify({
      text,
      annotations: annotations.map((a) => ({
        type: a.kind,
        content: a.content,
        ...(a.source ? { source: { kind: a.source.kind, ref: a.source.ref } } : {}),
      })),
    }),
  });
}

/**
 * Tracks a session for guaranteed cleanup. Wrap dispatch + assertions
 * in `withSession` so a panic still cancels the run.
 */
export async function withSession<T>(
  input: DispatchInput,
  fn: (s: Dispatched) => Promise<T>,
): Promise<T> {
  const dispatched = await dispatch(input);
  try {
    return await fn(dispatched);
  } finally {
    await cancelSession(dispatched.slug);
  }
}

// ── Event stream subscription ──────────────────────────────────────────────

export interface EventEnvelope {
  id: string;
  event_type: string;
  entity_id: string;
  actor_id?: string;
  target_actor_id?: string;
  payload_json: string;
  ts?: string;
}

/**
 * Poll `/api/v1/events/poll` for events matching `predicate`. Backed by
 * the persisted event log — sees EVERY persisted event including
 * directed events (events with `target_actor_id` set, which the SSE
 * stream filters away by kernel design — see `collectEvents` below for
 * the broadcast-only path).
 *
 * Returns when `stopAfter` matches fire or `timeoutMs` elapses.
 * Capture a `sinceId` BEFORE the action that should emit, then poll
 * after the action — that filters out unrelated old events.
 */
export async function pollForEvents(
  predicate: (e: EventEnvelope) => boolean,
  opts: {
    sinceId?: string;
    timeoutMs?: number;
    stopAfter?: number;
    pollIntervalMs?: number;
    /** Server-side filter: only events whose entity_id OR payload.slug
     *  matches. Sharply narrows the result set when many concurrent
     *  dispatches are flowing. Use the dispatched slug; events that
     *  carry the slug in entity_id or payload come through. */
    slug?: string;
    /** Server-side filter: namespace prefix (e.g. "hook" matches
     *  hook.*). Cheap to combine with slug. */
    namespace?: string;
  } = {},
): Promise<EventEnvelope[]> {
  const timeoutMs = opts.timeoutMs ?? 8_000;
  const interval = opts.pollIntervalMs ?? 250;
  const deadline = Date.now() + timeoutMs;
  const collected: EventEnvelope[] = [];
  let cursor = opts.sinceId;

  while (Date.now() < deadline) {
    const qs = new URLSearchParams();
    if (cursor) qs.set("since_id", cursor);
    if (opts.slug) qs.set("slug", opts.slug);
    if (opts.namespace) qs.set("namespace", opts.namespace);
    qs.set("limit", "200");
    const { body } = await api<{ data?: { events?: EventEnvelope[] } }>(
      `/api/v1/events/poll?${qs}`,
    );
    const events = body.data?.events ?? [];
    if (events.length > 0) {
      cursor = events[events.length - 1]!.id;
      for (const e of events) {
        if (predicate(e)) {
          collected.push(e);
          if (opts.stopAfter && collected.length >= opts.stopAfter) {
            return collected;
          }
        }
      }
    }
    await new Promise((r) => setTimeout(r, interval));
  }
  return collected;
}

/** Latest persisted event id — use as a `sinceId` cursor anchor.
 *  The poll endpoint returns events ASC (oldest of the recent 5000
 *  first — `list_recent_events_at` does `ORDER BY id DESC LIMIT 5000`
 *  then `.reverse()`). To get the newest, ask for the full recent
 *  window and read the LAST element. Skipping the workaround would
 *  put the cursor 5000 events in the past — making tests miss newly
 *  emitted events on busy daemons. */
export async function latestEventId(): Promise<string | undefined> {
  const { body } = await api<{ data?: { events?: EventEnvelope[] } }>(
    `/api/v1/events/poll?limit=5000`,
  );
  const evs = body.data?.events;
  return evs && evs.length > 0 ? evs[evs.length - 1]!.id : undefined;
}

/**
 * Open an SSE subscription, collect every event matching `predicate`
 * until `untilMs` elapses or `stopAfter` matches fire. Returns the
 * collected envelopes.
 *
 * WARNING: SSE is broadcast-only. The kernel's `deliver()` short-circuits
 * directed events (`target_actor_id` set) to only reach that actor's
 * mailbox — broadcast subscribers (including this stream) never see
 * them. For directed events (studio.chat_turn, mesh.message, …) use
 * `pollForEvents` instead. See
 * `.artifacts/agents-arc/discussions/directed-events-invisible-to-sse.md`.
 */
export async function collectEvents(
  predicate: (e: EventEnvelope) => boolean,
  opts: { windowMs: number; stopAfter?: number } = { windowMs: 5_000 },
): Promise<EventEnvelope[]> {
  const collected: EventEnvelope[] = [];
  const token = await daemonToken();
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), opts.windowMs);

  try {
    const r = await fetch(`${await baseUrl()}/api/v1/events/stream`, {
      headers: { Authorization: `Bearer ${token}` },
      signal: controller.signal,
    });
    if (!r.ok || !r.body) {
      throw new Error(`event stream open failed: status=${r.status}`);
    }
    const reader = r.body.getReader();
    const decoder = new TextDecoder();
    let buf = "";

    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      buf += decoder.decode(value, { stream: true });
      let idx: number;
      while ((idx = buf.indexOf("\n\n")) !== -1) {
        const frame = buf.slice(0, idx);
        buf = buf.slice(idx + 2);
        const dataLine = frame
          .split("\n")
          .find((l) => l.startsWith("data:"));
        if (!dataLine) continue;
        let envelope: EventEnvelope;
        try {
          envelope = JSON.parse(dataLine.slice(5).trim());
        } catch {
          continue;
        }
        if (predicate(envelope)) {
          collected.push(envelope);
          if (opts.stopAfter && collected.length >= opts.stopAfter) {
            controller.abort();
            return collected;
          }
        }
      }
    }
  } catch (e) {
    if (!(e instanceof Error && controller.signal.aborted)) throw e;
  } finally {
    clearTimeout(timer);
  }
  return collected;
}

/**
 * Subscribe to the SSE stream and resolve when an event matching
 * `predicate` arrives, or reject on timeout. Aborts the SSE connection
 * on resolution.
 */
export async function waitForEvent(
  predicate: (e: EventEnvelope) => boolean,
  opts: { timeoutMs?: number } = {},
): Promise<EventEnvelope> {
  const timeoutMs = opts.timeoutMs ?? 15_000;
  const token = await daemonToken();
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const r = await fetch(`${await baseUrl()}/api/v1/events/stream`, {
      headers: { Authorization: `Bearer ${token}` },
      signal: controller.signal,
    });
    if (!r.ok || !r.body) {
      throw new Error(`event stream open failed: status=${r.status}`);
    }

    const reader = r.body.getReader();
    const decoder = new TextDecoder();
    let buf = "";

    while (true) {
      const { value, done } = await reader.read();
      if (done) throw new Error("event stream closed before match");
      buf += decoder.decode(value, { stream: true });

      // SSE messages are separated by blank lines; each contains
      // a `data:` line we parse as JSON.
      let idx: number;
      while ((idx = buf.indexOf("\n\n")) !== -1) {
        const frame = buf.slice(0, idx);
        buf = buf.slice(idx + 2);
        const dataLine = frame
          .split("\n")
          .find((l) => l.startsWith("data:"));
        if (!dataLine) continue;
        let envelope: EventEnvelope;
        try {
          envelope = JSON.parse(dataLine.slice(5).trim());
        } catch {
          continue;
        }
        if (predicate(envelope)) {
          controller.abort();
          return envelope;
        }
      }
    }
  } catch (e) {
    if (controller.signal.aborted) {
      throw new Error(
        `waitForEvent: no matching event within ${timeoutMs}ms`,
      );
    }
    throw e;
  } finally {
    clearTimeout(timer);
  }
}
