# Chrome / Host Spec

Concrete shape for `<ship-host>` + `window.ship`. Proposal — feedback before implementation.

## Principle

**Chrome = host. Views = apps.** Same render pipeline, same `ship.*` SDK, different lifecycles and different framework surface.

- **Views** mount on navigation, unmount on exit. Live in `.ship/views/`, `~/.ship/views/`, or the future store cache. First-party views are bundled but built AS third-party — they discover framework features the same way.
- **Chrome** is persistent, bundled with the shell (or plugin). It provides `window.ship` to views and mounts view outlets. No `.ship/views/` equivalent; chrome is code, not a template.

Views call chrome through `window.ship` (imperative) and through kernel events (persistent). Chrome calls views through DOM (mount into slot) and signals (Datastar). Neither knows the other's implementation.

## `<ship-host>` — root host component

Lives in the shell's `index.html`. Wraps everything.

```html
<ship-host>
  <ship-nav-top slot="top"></ship-nav-top>
  <ship-rail slot="left"></ship-rail>
  <ship-terminal slot="bottom"></ship-terminal>
  <ship-chat-drawer slot="right"></ship-chat-drawer>
  <ship-cmdk></ship-cmdk>       <!-- overlay, no slot -->
  <ship-toast-host></ship-toast-host>
  <!-- Default slot = view outlet (mount point for file-routed views) -->
</ship-host>
```

**Responsibilities:**

1. **Mount `window.ship` API** on first connect. Chrome components register themselves after this is ready (idempotent).
2. **Expose `<slot>` named `top`, `left`, `right`, `bottom`** + default slot for view. Layout is CSS grid; chrome decides its own chrome dimensions via slot assignment.
3. **Own the single SSE connection to the daemon event kernel.** Multiplex events to subscribed chrome + view code. Reconnects on drop.
4. **Manage the router.** View mount/unmount is host-driven. Views never directly manipulate history.
5. **Dispatch a typed DOM event bus** for chrome↔chrome communication. Events bubble on `<ship-host>`; chrome components subscribe.

**Contract with children:**

Chrome components register via `this.closest('ship-host')?.register({ name, subscribe, unsubscribe })`. Host calls lifecycle hooks on connected/disconnected. Chrome components get `this.host` = the `<ship-host>` instance after `connectedCallback`.

## `window.ship` — view-facing API

Small, stable, typed. Exposed by `<ship-host>` on mount. Third-party views consume it exactly as first-party views do.

```typescript
interface WindowShip {
  // ── Navigation ─────────────────────────────────────────────
  navigate(path: string, opts?: { replace?: boolean }): void;
  back(): void;
  current(): { path: string; params: Record<string, string>; query: Record<string, string> };

  // ── Chrome services ────────────────────────────────────────
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
  // Delegates to <ship-host>'s single SSE connection — no per-subscriber socket.
  subscribe(namespace: string, handler: (event: KernelEvent) => void): () => void;

  // Emit a kernel event (POST to /api/events/emit if/when that exists;
  // today: view emits via its own POST endpoint, kernel routes).
  // Mostly for symmetry; views typically emit via their own HTTP surface.

  // ── Signals (Datastar bridge) ──────────────────────────────
  // Read/write page-scoped Datastar signals imperatively. Useful when
  // a chrome component needs to peek at the active view's signal state.
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

1. **Everything on `window.ship` is synchronous-looking**, even if internally async (returns a Promise that fires after mount when needed). Third-party views shouldn't have to wait for chrome-ready.
2. **No "expose the internals" trap.** No `window.ship.host`, no `window.ship.router`. The surface is the contract. Chrome components change, API doesn't.
3. **Kernel-event subscription goes through `window.ship.subscribe`**, not a raw `new EventSource()`. Single connection, broadcast internally. This is cheaper and means views don't manage auth.

## DOM event conventions

Chrome↔chrome communication happens via custom events bubbling on `<ship-host>`. Naming:

- `ship:navigate` — route is about to change. `{ from, to }`
- `ship:view-mounted` — view finished rendering. `{ path, root: HTMLElement }`
- `ship:view-unmounted` — view detached. `{ path }`
- `ship:notify` — toast request. `{ message, tone }`
- `ship:command:open` — command palette. `{ query? }`
- `ship:chat:open` / `ship:chat:close`
- `ship:terminal:open` / `ship:terminal:close`

Views rarely dispatch these — they call `window.ship.*` which internally dispatches them. Chrome components mostly dispatch them. Either can listen.

## Kernel-event bridge

Single SSE connection opened by `<ship-host>` on first subscribe, closed after last unsubscribe with a 30s grace. Multiplexes events to all handlers by namespace match (prefix-based, same as `manifest.events:` semantics).

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

## Chrome component contract

A chrome component is a web component that lives under `<ship-host>`. It:

1. Has a custom element tag starting with `ship-` (same convention as primitives).
2. Subscribes to kernel events via `this.host.subscribe(namespace, handler)` in `connectedCallback`; cleans up in `disconnectedCallback`.
3. Dispatches DOM events by calling `window.ship.*` or by `this.host.dispatchEvent()`.
4. Does NOT fetch view HTML directly. It asks `window.ship.navigate(path)` or mounts a slot to contain a view outlet that the host renders into.
5. Holds no daemon authentication state — the host owns auth.

**Example: `<ship-chat-drawer>`**

```typescript
@customElement('ship-chat-drawer')
export class ShipChatDrawer extends LitElement {
  private _unsub?: () => void;

  connectedCallback() {
    super.connectedCallback();
    // When open, mount /api/v/chat into our slot. But do NOT subscribe
    // to chat-specific kernel events here — the chat view itself does that.
  }

  private open() {
    this.mountView('/api/v/chat');
    this.hidden = false;
  }

  // The drawer is dumb chrome. It knows how to:
  //   - Slide in/out
  //   - Mount a view URL via window.ship.openView(url, { into: this })
  //   - Respond to ship:chat:open / ship:chat:close DOM events
  // It does NOT know what chat IS.
}
```

The chat VIEW handles its own data + kernel subscriptions (via `manifest.events` + `ship.chat.state()` on re-render). The DRAWER just contains it.

## Distribution model

- **Chrome components** ship with `@ship/chrome` package. Shell imports them. User can swap implementations (e.g. `@ship/chrome-minimal` with just rail + cmdk).
- **View primitives** ship with `@ship/set` (existing). Available to both chrome and views.
- **Third-party chrome** is a future: a plugin mechanism where `~/.ship/chrome/*.js` bundles are loaded by the host at startup. Out of scope for this pass; keep the API surface small enough to not preclude it.

## Migration plan

Current state: `apps/shell/src/main.ts` is a 400+ line orchestrator with inlined rail toggle, drawer toggle, cmdk, router, theme, status. Chrome is hard-wired into the shell app.

Phased:

**Phase 1 — extract the host.** Create `<ship-host>` in `packages/ship-chrome/`. Move the single-SSE kernel bridge + `window.ship` bootstrap there. Shell `index.html` wraps its existing structure in `<ship-host>`. Nothing visible changes.

**Phase 2 — extract rail + nav.** Move left rail and top nav into `<ship-rail>` and `<ship-nav-top>`. They register with the host. Shell main.ts shrinks.

**Phase 3 — extract cmdk, toast, drawer.** Move cmdk, toast-host, chat-drawer into components. They subscribe to `ship:command:open` / `ship:notify` / `ship:chat:open` DOM events.

**Phase 4 — extract terminal.** More involved (PTY state), but same pattern.

**Phase 5 — thin main.ts to composition only.** `main.ts` becomes: import chrome components, assemble in `index.html`, wire router. All logic is in web components.

**Phase 6 — package boundary.** Split `packages/ship-chrome/` from shell. Shell depends on chrome package.

Each phase: green build, dogfooded (the web shell still works), committed before moving on.

## What this unlocks

1. **Third-party chrome** — someone writes `ship-chrome-alt-rail` and swaps it in. Same mechanism as third-party views.
2. **Typed API surface** — view authors get autocomplete for `window.ship.*`. Schema generation (registry-side, step 6 of framework plan) extends into `window.ship`.
3. **Clean extension story** — when a user-authored view emits `annotation.created`, the chat drawer's internal view subscribes via manifest events. No chrome-side coupling. The framework handles it.
4. **Shell becomes distribution** — the shell app is just a composition + chrome package dependency. The shell binary could swap chrome packages at build time.

## Open questions

1. **How do views discover the `window.ship` type?** Probably ship a `.d.ts` in a package consumers install or reference. For now: document it.
2. **Is `signals.watch` performant?** Peeking into Datastar signals from chrome could be noisy. Lazy — only attach listeners when something actually watches.
3. **When does chat-as-view's SSE connection come from?** If chat subscribes via manifest events through routed SSE, that's a second SSE socket (on top of the host's kernel bridge). Might consolidate later; not blocking.
4. **Does chrome need its own view-routing equivalent for third-party chrome plugins?** E.g. a chrome plugin that wants to register a command palette entry. Probably a `window.ship.register` surface — but defer until a real plugin appears.
5. **PTY / terminal** — the terminal is special: WebSocket-like, not SSE. How does it fit `window.ship.terminal.run(cmd)`? Likely a separate long-lived connection managed by the host or by `<ship-terminal>`. Defer concrete design to phase 4.

## Non-goals

- We're not building a plugin marketplace today.
- We're not invoking chrome components from view templates (chrome is pure browser; templates render in daemon).
- We're not exposing kernel event emission from arbitrary views via `window.ship` — views POST to their own HTTP endpoints, which emit kernel events server-side. `window.ship.subscribe` is read-only.
