# Shell Spec

Concrete shape for `<ship-shell>` + `window.ship`. Proposal — feedback before implementation.

## Principle

**Shell = host. Views = apps.** Same render pipeline, same `ship.*` SDK, different lifecycles and different framework surface.

- **Views** mount on navigation, unmount on exit. Live in `.ship/views/`, `~/.ship/views/`, or the future store cache. First-party views are bundled but built AS third-party — they discover framework features the same way.
- **Shell** is persistent, bundled with the app binary (or plugin). Provides scaffolding (rail, nav, cmdk, chat drawer, terminal, toast), exposes `window.ship` to views, mounts view outlets. Shell is code, not a template; no `.ship/views/` equivalent.

Views call the shell through `window.ship` (imperative) and through kernel events (persistent). Shell components mount views into slots and manage routing. Neither knows the other's implementation.

## `<ship-shell>` — root component

Lives in the browser app's `index.html`. Wraps everything.

```html
<ship-shell>
  <ship-nav-top slot="top"></ship-nav-top>
  <ship-rail slot="left"></ship-rail>
  <ship-terminal slot="bottom"></ship-terminal>
  <ship-chat-drawer slot="right"></ship-chat-drawer>
  <ship-cmdk></ship-cmdk>            <!-- overlay, no slot -->
  <ship-toast-host></ship-toast-host>
  <!-- Default slot = view outlet (mount point for file-routed views) -->
</ship-shell>
```

**Responsibilities:**

1. **Mount `window.ship`** on first connect. Shell components register themselves after this is ready (idempotent).
2. **Expose named slots** `top`, `left`, `right`, `bottom` + default slot for view. Layout is CSS grid; shell components decide their own dimensions via slot assignment.
3. **Own the single SSE connection** to the daemon event kernel. Multiplex events to subscribed shell components + view code. Reconnect on drop.
4. **Manage the router.** View mount/unmount is shell-driven. Views never manipulate history directly.
5. **Dispatch a typed DOM event bus** for shell-component↔shell-component communication. Events bubble on `<ship-shell>`; components subscribe.

**Contract with children:**

Shell components register via `this.closest('ship-shell')?.register({ name, subscribe, unsubscribe })`. The shell calls lifecycle hooks on connected/disconnected. Components get `this.shell` = the `<ship-shell>` instance after `connectedCallback`.

## `window.ship` — view-facing API

Small, stable, typed. Exposed by `<ship-shell>` on mount. Third-party views consume it exactly as first-party views do.

```typescript
interface WindowShip {
  // ── Navigation ─────────────────────────────────────────────
  navigate(path: string, opts?: { replace?: boolean }): void;
  back(): void;
  current(): { path: string; params: Record<string, string>; query: Record<string, string> };

  // ── Shell services ─────────────────────────────────────────
  notify(msg: string, tone?: 'info' | 'success' | 'warning' | 'destructive'): void;
  command(opts?: { query?: string }): void;       // open cmdk, optionally with query
  chat: {
    open(opts?: { workspace_id?: string }): void;
    close(): void;
    send(text: string, workspace_id?: string): Promise<void>;
  };
  terminal: {
    open(): void;
    close(): void;
    run(cmd: string): Promise<{ exit: number; stdout: string; stderr: string }>;
  };

  // ── Kernel events ──────────────────────────────────────────
  // Subscribe to a kernel event namespace. Returns unsubscribe fn.
  // Delegates to <ship-shell>'s single SSE connection — no per-subscriber socket.
  subscribe(namespace: string, handler: (event: KernelEvent) => void): () => void;

  // ── Signals (Datastar bridge) ──────────────────────────────
  // Read/write page-scoped Datastar signals imperatively. Useful when
  // a shell component needs to peek at the active view's signal state.
  signals: {
    get(key: string): unknown;
    set(key: string, value: unknown): void;
    watch(key: string, handler: (value: unknown) => void): () => void;
  };

  // ── Session / env ──────────────────────────────────────────
  session: { id: string; workspace_id?: string };
  env: { scope: 'user' | 'project'; version: string; dev: boolean };
}

declare global {
  interface Window { ship: WindowShip }
}
```

**Design rules:**

1. **Everything on `window.ship` is synchronous-looking**, even if internally async (returns a Promise that fires after mount when needed). Third-party views shouldn't have to wait for shell-ready.
2. **No "expose the internals" trap.** No `window.ship.shell`, no `window.ship.router`. The surface is the contract. Shell components change, API doesn't.
3. **Kernel-event subscription goes through `window.ship.subscribe`**, not a raw `new EventSource()`. Single connection, broadcast internally. Cheaper, and views don't manage auth.

## DOM event conventions

Shell-component communication happens via custom events bubbling on `<ship-shell>`. Naming:

- `ship:navigate` — route is about to change. `{ from, to }`
- `ship:view-mounted` — view finished rendering. `{ path, root: HTMLElement }`
- `ship:view-unmounted` — view detached. `{ path }`
- `ship:notify` — toast request. `{ message, tone }`
- `ship:command:open` — command palette. `{ query? }`
- `ship:chat:open` / `ship:chat:close`
- `ship:terminal:open` / `ship:terminal:close`

Views rarely dispatch these — they call `window.ship.*` which internally dispatches them. Shell components mostly dispatch them. Either can listen.

## Kernel-event bridge

Single SSE connection opened by `<ship-shell>` on first subscribe, closed after last unsubscribe with a 30s grace. Multiplexes events to all handlers by namespace match (prefix-based, same as `manifest.events:` semantics).

```typescript
// Internal contract
class KernelBridge {
  subscribe(namespace: string, handler: EventHandler): UnsubscribeFn;
  private socket: EventSource | null;
  private handlers: Map<string, Set<EventHandler>>;
  private connect(): void;   // opens /api/events/stream with auth
  private route(event: KernelEvent): void;  // prefix-match + fan out
}
```

The existing routed-SSE (`/api/sse/{*path}`) stays — it's the view-level re-render trigger (manifest.events drives it per-view). The kernel bridge is complementary: it's for JS-level reactions (chat updating when `annotation.created` fires, toast showing when `job.failed` fires, etc.), not template re-renders.

## Shell component contract

A shell component is a web component that lives under `<ship-shell>`. It:

1. Uses a custom element tag starting with `ship-` (same convention as primitives).
2. Subscribes to kernel events via `this.shell.subscribe(namespace, handler)` in `connectedCallback`; cleans up in `disconnectedCallback`.
3. Dispatches DOM events by calling `window.ship.*` or `this.shell.dispatchEvent()`.
4. Does NOT fetch view HTML directly. It asks `window.ship.navigate(path)` or mounts a slot that contains a view outlet the shell renders into.
5. Holds no daemon authentication state — the shell owns auth.

**Example: `<ship-chat-drawer>`**

```typescript
@customElement('ship-chat-drawer')
export class ShipChatDrawer extends LitElement {
  connectedCallback() {
    super.connectedCallback();
    // When open, mount /api/v/chat into our slot. Do NOT subscribe
    // to chat-specific kernel events here — the chat view itself does that.
  }

  private open() {
    this.mountView('/api/v/chat');
    this.hidden = false;
  }

  // The drawer is dumb scaffolding. It knows how to:
  //   - Slide in/out
  //   - Mount a view URL via window.ship.openView(url, { into: this })
  //   - Respond to ship:chat:open / ship:chat:close DOM events
  // It does NOT know what chat IS.
}
```

The chat VIEW handles its own data + kernel subscriptions (via `manifest.events` + `ship.chat.state()` on re-render). The DRAWER just contains it.

## Distribution model

- **Shell components** ship with `@ship/shell` package. The shell app imports them. Users can swap implementations (e.g. `@ship/shell-minimal` with just rail + cmdk).
- **View primitives** ship with `@ship/set` (existing). Available to both shell and views.
- **Third-party shell components** are a future: plugin mechanism where `~/.ship/shell/*.js` bundles are loaded by `<ship-shell>` at startup. Out of scope for this pass; keep the API small enough to not preclude it.

## Migration plan

Current state: `apps/shell/src/main.ts` is a 400+ line orchestrator with inlined rail toggle, drawer toggle, cmdk, router, theme, status. Scaffolding is hard-wired into the shell app.

Phased:

**Phase 1 — extract the root.** Create `<ship-shell>` in `packages/ship-shell/`. Move the single-SSE kernel bridge + `window.ship` bootstrap there. `index.html` wraps its existing structure in `<ship-shell>`. Nothing visible changes.

**Phase 2 — extract rail + nav.** Move left rail and top nav into `<ship-rail>` and `<ship-nav-top>`. They register with the shell. Shell `main.ts` shrinks.

**Phase 3 — extract cmdk, toast, drawer.** Move cmdk, toast-host, chat-drawer into components. They subscribe to `ship:command:open` / `ship:notify` / `ship:chat:open` DOM events.

**Phase 4 — extract terminal.** More involved (PTY state), but same pattern.

**Phase 5 — thin `main.ts` to composition only.** `main.ts` becomes: import shell components, assemble in `index.html`, wire router. All logic moves into web components.

**Phase 6 — package boundary.** Split `packages/ship-shell/` from `apps/shell/`. The shell app depends on the shell package.

Each phase: green build, dogfooded (the browser app still works), committed before moving on.

## What this unlocks

1. **Third-party shell components** — someone writes `ship-shell-alt-rail` and swaps it in. Same mechanism as third-party views.
2. **Typed API surface** — view authors get autocomplete for `window.ship.*`. Schema generation (registry-side, step 6 of framework plan) extends into `window.ship`.
3. **Clean extension story** — when a user-authored view emits `annotation.created`, the chat drawer's internal view subscribes via manifest events. No shell-side coupling. The framework handles it.
4. **App becomes composition** — `apps/shell/` shrinks to a composition layer + shell package dependency. The binary could swap shell packages at build time.

## Open questions

1. **How do views discover the `window.ship` type?** Ship a `.d.ts` in a package consumers install or reference. Document it for now.
2. **Is `signals.watch` performant?** Peeking into Datastar signals could be noisy. Lazy — only attach listeners when something actually watches.
3. **When does chat-as-view's SSE connection come from?** If chat subscribes via manifest events through routed SSE, that's a second SSE socket on top of the shell's kernel bridge. Might consolidate later; not blocking.
4. **Does the shell need routing for third-party shell component plugins?** E.g. a plugin that wants to register a command palette entry. Probably `window.ship.register` — defer until a real plugin appears.
5. **PTY / terminal** — WebSocket-like, not SSE. How does it fit `window.ship.terminal.run(cmd)`? Likely a separate long-lived connection managed by `<ship-terminal>`. Defer concrete design to phase 4.

## Non-goals

- Not building a plugin marketplace today.
- Not invoking shell components from view templates (shell is pure browser; templates render in the daemon).
- Not exposing kernel event emission from arbitrary views via `window.ship`. Views POST to their own HTTP endpoints, which emit kernel events server-side. `window.ship.subscribe` is read-only.

## Vocabulary

- **Shell** — the browser app. `apps/shell/` package (composition) + `packages/ship-shell/` package (components). `<ship-shell>` root element. `@ship/shell` npm package name.
- **Shell components** — web components that live under `<ship-shell>` and provide user-visible scaffolding (rail, nav, cmdk, chat drawer, terminal, toast host). Not views.
- **View** — content that mounts into the shell's view outlet. File-routed under `.ship/views/`, rendered by the daemon, fetched as HTML.
- **`window.ship`** — the single typed API the shell exposes to view authors. Everything imperative a view might need goes through it.
