# j2 Shell Harvest — Behavioral Inventory

**Job:** `j2-shell-harvest`
**Date:** 2026-05-16
**Method:** Playwright (headless chromium, 1440×900) driving the running daemon at `http://127.0.0.1:9385`. Three probe scripts were run — a per-route navigator, a chrome-surface deep dive, and a shadow-DOM-aware behavior probe. Raw probe sources in `/tmp/j2-probe/probe*.mjs`; raw outputs in `/tmp/j2-probe/out*.json`. Latency numbers are wall-clock from the in-page `performance.now()` clock; DOM-state assertions are from in-page `evaluate()` snapshots taken after each interaction.

## Reading guide

Every surface gets:
- **Overview** — what it does for the user, one paragraph.
- **Behavior log** — observed interactions in plain prose. Numbers are from Playwright; assertions are from DOM inspection.
- **PRESERVE** — behaviors the SUI replacement should keep.
- **AVOID** — observed anti-patterns. Don't carry them forward.
- **Open questions** — design calls the port should not make alone.

The intent docs that come out of this should treat the PRESERVE/AVOID/open-questions lists as inputs, not as a spec.

---

## Global findings (apply across the whole shell)

These showed up in every surface and only need to be stated once.

- **Routing base is broken.** Navigo is instantiated with base `/` (`apps/shell/src/main.ts:583`) but the daemon serves the SPA from `/shell/`. Direct URLs like `/shell/dashboard` therefore land on the `notFound()` handler ("The route '/shell/dashboard' doesn't exist."). The shell only loads view content if you load `/shell/` and then trigger a navigation in-app via `window.router.navigate('/dashboard')`. This is the documented "navigo-base" bug; every probe in this report worked around it by navigating client-side from a freshly loaded `/shell/`. **The SUI replacement must own the URL/base contract end-to-end: the same URL must be deep-linkable, reload-safe, and routable by the SPA.**
- **`body { transition: all }`** (the inline reset in `apps/shell/index.html`). This is a CSS shotgun — every property change pays an animation cost, and yet the actual theme swap happens in a single frame anyway (the body bg jumps from light to dark within 75 ms of `setShipTheme`, with no interpolation). Result: cost without payoff. Replace with a short list of explicit properties (`background-color`, `color`, `border-color`).
- **Three daemon endpoints 404 in the console on most loads:** `GET /api/v1/view-source/shell/`, `GET /api/v1/lint/view`, `GET /api/settings`. The job spec calls these out as pre-existing mismatches; they show up as red console noise on every navigation but don't block render.
- **`pageerror: Cannot set properties of null (setting 'data')`** fires repeatedly on first paint (8+ occurrences in a single session). Origin appears to be a datastar binding evaluating before its target node mounts. Doesn't block the view but pollutes the console and any error-reporter dashboard.
- **`ship-code-editor` Lit warning** fires twice per session: "The following properties on element ship-code-editor will not trigger updates as expected because they are set using class fields: value, language, placeholder, readonly, line-numbers, size, tone…". Sloppy class-fields-vs-reactive-properties; users will see stale editor state if a parent re-renders.
- **`/api/v1/v/chat` returns 500** under the current daemon state — the chat view is dead. The shell renders `mount.ts`'s error fallback ("The daemon couldn't render this view") with a Retry button. The error fallback itself behaves well; the underlying view is broken.
- **`/api/v1/v/git` did not fetch on a `router.navigate('/git')` call after a prior `/setup` mount** (no network request fired). The view-root retained the previous setup content unchanged. Suggests Navigo de-dupes the in-app navigation in some state transition; users would experience clicking the Git rail tile and seeing the previous view stick.
- **Real focus does not land on the input** when the command bar opens. After `⌘K`, `document.activeElement.closest('ship-command-bar')` is truthy (focus is inside the bar's host), but `document.activeElement.tagName === 'INPUT'` is false. Typing still routes correctly because the keydown handler is on the bar element, not the input — but screen-reader users get no announce, and users who Tab once after `⌘K` jump to a `ship-button` outside the bar (no focus trap).

---

## Chrome surfaces

### Top nav (`.ship-topbar` + `ship-nav-top`)

**Overview.** A 36px strip across the top of the viewport. Hosts the command bar trigger in the center and (via slot) a session/theme indicator on the right. The trigger is the only interactive affordance in the bar itself — the visual "label" is a button with `aria-label="Open command bar"` and inner text "Ask commander or jump to… ⌘K".

**Behavior log.**
- Top bar height: 36px, full-width grid row.
- The trigger button is a `<button class="trigger">` with an inline `<ship-icon name="search">`, a `.label` span ("Ask commander or jump to…"), and a `<kbd>⌘K</kbd>` hint glued to the right edge. Visual hierarchy reads as a search field, which trains the right expectation.
- Clicking the trigger and pressing `⌘K` both open the modal; the keyboard path measured ~80ms from dispatch to `open` attribute set on `<ship-command-bar>`.
- No visible mode chip in the trigger — once the bar is open you discover modes by typing a prefix. There is no visible legend on first paint.

**PRESERVE.**
- **Single trigger surface for all command verbs.** One affordance for nav, command, mention, AI prompt collapses the cognitive load. The label-as-prompt ("Ask commander or jump to…") is good copy because it implies the bar handles both navigation and AI in one place.
- **`⌘K` keyboard hint visually in the trigger.** Users learn the shortcut without having to read docs.
- **Trigger is button-shaped, not input-shaped, until opened.** Avoids the "user types into the closed bar and nothing happens" footgun.

**AVOID.**
- The trigger has no aria-keyshortcuts attribute; screen readers don't get told ⌘K exists.
- The trigger is centered with no left/right siblings — the topbar feels half-empty on wide screens. Project picker and view-picker are pushed into the rail/status bar instead.

**Open questions.**
- Should the topbar host a breadcrumb/route indicator (currently nothing tells you where you are except the rail highlight, which is also broken — see Rail)? Or should the route name live in the status bar / view header?
- Should the trigger collapse to icon-only at narrow widths, or stay always-text?

---

### Command palette (`<ship-command-bar>`)

**Overview.** A Lit web-component with a shadow root that renders both the topbar trigger and (when open) a modal flyout. Routes input by prefix: `/` → slash commands, `>` → fuzzy nav, `@` → mention picker, plain text → AI message to current session. Roughly 1800 LOC in `ship-command-bar.ts` — the single most complex piece of chrome.

**Behavior log.**
- **Open latency:** `⌘K` → `open` attribute set: ~80ms. Modal HTML is ~4989 chars on open, so initial paint is cheap. No measurable jank.
- **Default mode on open: `find`.** Input placeholder reads "Jump to a view…". The flyout shows the 8 default views (Dashboard, Pipelines, Agents, Skills, Git, Collections, Canvas, Events) with paths beside each. So opening the bar with no input is already a nav launcher — you don't have to type anything to jump somewhere.
- **`/` prefix:** flyout swaps to slash-command list with 6 items: `/new`, `/chats`, `/settings`, `/lint`, `/end`, `/close`. Each has a one-line description right of the verb. Mode transition is instant (single input frame, no animation).
- **`>` prefix:** same as default — find mode. Both routes work, redundancy is intentional ("you can be explicit").
- **`@` prefix:** mention picker. Returns "agents view", "artifacts view", "canvas view"… — interesting: it includes `artifacts` and `chat` views which do not exist as routes registered in `main.ts`. The mention index is sourced from the daemon's view catalog, which is broader than the SPA's route table. **Click-through on an `@artifacts` mention would dead-end.**
- **Plain text:** flyout empties (totalCount: 0). The bar is in AI mode; pressing Enter would send to the current session. There is no visible "you're in AI mode" affordance other than the absence of suggestions, which is a weak signal.
- **ESC:** closes the modal, returns focus to body. Confirmed: `bar.hasAttribute('open')` flips to false within ~50ms, `document.activeElement` leaves the bar.
- **Tab inside the open bar:** Tab does NOT stay inside the modal. The first Tab press lands on a `ship-button` outside the bar (with no useful label). No focus trap. This is a regression vs. any reasonable command-palette UX.
- **Focus target:** `document.activeElement` after open is the `<ship-command-bar>` host element, not the inner `<input>`. Typing still works because the bar's host listens for keydown. But this means screen-reader users get no `aria-activedescendant` lifecycle, and Tab-from-bar is broken (above).

**PRESERVE.**
- **Single bar, four modes via prefix.** `/` `>` `@` text are intuitive once known. Don't fragment back into separate palettes.
- **`find` is the default-on-open mode, with the 8 default views already listed.** You can be ⌘K → Enter → on a view without typing anything. This is faster than any per-view tab navigation.
- **The "command + description" two-column row.** `/new — Scaffold a view, skill, agent, …` is good — verb first, semantic gloss right. The future SUI version should keep this row shape.
- **80ms open latency.** Set as a budget for the SUI replacement. Anything over 100ms will feel laggy.
- **Modal renders in shadow DOM** so it doesn't pollute the global DOM, and the trigger and modal are the same component (single source of state).

**AVOID.**
- **No focus trap.** Tab leaves the open palette. Hard requirement for the replacement: trap focus to the modal until Escape.
- **Focus on open lands on the host element, not the input.** Re-target focus to the input on every open. Bonus: set `aria-activedescendant` to the currently-highlighted item in the flyout so screen readers can follow.
- **Empty `aria-keyshortcuts`/`aria-controls`/`aria-expanded` on the trigger.** Wire up ARIA combobox/listbox idiom (`role="combobox"`, `aria-controls="<flyout-id>"`, `aria-expanded`, `aria-activedescendant`).
- **Mention index includes non-routable views.** Either filter to the SPA's registered routes or fix the routes. Don't let a working `@` complete to a 404.
- **AI mode has no visible indicator.** Add a mode chip (left of the input, or as a fading border tint) so users know whether Enter sends a message or runs a command.
- **`/` slash-list is hand-curated.** New verbs require a code change in `ship-command-bar.ts`. The replacement should source the list from a registry (so adding a verb doesn't require touching the palette).

**Open questions.**
- Should `>` actually be removed since empty-input does the same thing? Or kept as a docs-friendly way to teach the prefix system?
- Should AI text-mode send on Enter (current) or require a modifier (e.g., Cmd+Enter)? Plain Enter risks message-send when the user meant to pick a slash command they hadn't finished typing.
- Should the mention picker's results be scoped (mentions in a chat vs. mentions in a doc would be different sets)?
- Is the slash list the right place for `/lint` and `/end`? Those feel like context-sensitive commands that should appear conditionally (only show `/lint` when on a view that supports it).

---

### Rail (`<ship-rail>` / `.ship-rail`)

**Overview.** Vertical rail on the left, 48px wide nominally but uses `display: contents` so it has no measurable width itself — child anchors lay out in the grid column. Hosts 11 items: project switcher (top), 7 pinned view tiles (Pipelines/Agents/Dashboard/Skills/Git/Collections/Mission Control), launcher tile, "Create new feature" tile, settings tile (bottom).

**Behavior log.**
- 11 items total. 8 use `<a href="/...">` with a `title` attribute and no `aria-label` (icon-only tile, browser tooltip on hover delay ~700ms — not customizable). 3 use both `title` and `aria-label`: Mission Control ("Mission Control · 5 active"), Open app launcher, Create new feature, Switch project.
- **Tile transitions:** `background 0.12s cubic-bezier(0.16,1,0.3,1), color 0.12s cubic-bezier(0.16,1,0.3,1)`. Fast, snappy, identical curve across the rail. Feels good.
- **Active state:** no rail item ever has `aria-current="page"`, `.active`, or `data-active`. On `router.navigate('/agents')`, the agents tile does not visually distinguish itself. The `updateNav()` function in `main.ts` is supposed to update `[data-route]` items but only operates on elements found at boot via `querySelectorAll`. Since `<ship-rail>` renders its anchors inside its own renderRoot, the boot-time scan misses them.
- **Hover background change:** synthetic pointerenter+mouseover did not change background. Real cursor hover should — `:hover` CSS is defined — but the bar's pinned tile got no hover affordance in our synthetic test, so we can only confirm that the CSS exists, not the visual delta.
- **Project switcher (top):** `aria-label="Switch project"`, `title="Project: madvib"`, text "M". Tile size 32×32. Background already tinted `oklch(0.67 0.16 58 / 0.18)` — the orange-with-18%-alpha that indicates "current". So the project tile uses a persistent tint rather than a hover state.
- **Keyboard nav:** focusing the first rail item then pressing ArrowDown does NOT move focus to the next item (stays on the first tile). Arrow-key navigation is not implemented — users must Tab through each item.
- **No collapse/expand:** the rail is always visible at the same width. No way to hide it for full-bleed views.

**PRESERVE.**
- **48px tile column with mixed icon+icon-text items.** The width hits the sweet spot — wide enough to feel tappable on touch, narrow enough not to dominate.
- **Snappy 120ms transitions on background/color only.** Don't widen this to `transition: all`.
- **Project switcher uses persistent tint (not just hover) to indicate identity.** Carries the project's brand color into the chrome. The "M" monogram pattern is good — short, recognizable, no icon file to ship.
- **Tile size differentiation:** "Create new feature" is 36×32 (slightly smaller, top-of-bottom group), settings is bottom-aligned. The implicit grouping reads naturally.
- **Mission Control title includes a live count ("Mission Control · 5 active").** Tooltip-as-status is a clever cheap channel.

**AVOID.**
- **Anchors with `title` but no `aria-label`.** Title attribute is poor a11y — screen readers may read it inconsistently, and sighted keyboard users get no on-screen label until they hover for ~700ms. **Every rail item needs an `aria-label` (and a visible label or visible-on-focus label).**
- **No active-route indicator on the rail.** Users have no idea where they are in the view space. The `updateNav()` boot-time scan misses anchors rendered inside `<ship-rail>`'s renderRoot. The SUI replacement must make active state declarative (e.g., reactive to the router state) rather than imperatively scanning the DOM at boot.
- **No arrow-key navigation.** A vertical rail without arrow keys is needlessly hostile to keyboard users. Implement up/down to traverse, Enter to activate, Home/End to jump.
- **`.ship-rail { display: contents }`** has cascading consequences: width is 0, ARIA structure is flattened, you can't draw a column border on the rail itself. It was done so grid layout would work — but it makes the rail invisible to screen readers as a navigation region.
- **Mention/launcher/create tiles share the same anchor shape as nav tiles** (`<a href="#">`). Clicking sometimes navigates, sometimes opens a popover — there's no visual cue which is which.

**Open questions.**
- Should the rail collapse on dense views (theme studio, events) and expand on hover/keyboard? Or always stick at 48px?
- Should rail item order be user-controllable (drag-to-reorder), pin-managed, or fixed by app config? Code comments in `apps/shell/src/chrome/pins.ts` say "drag-to-reorder lands in a later phase" — should the SUI port land it instead of carrying the deferred TODO forward?
- Is "Mission Control" the right destination for `/mesh`, or should it be renamed in the route (UX label vs. tech label mismatch)?

---

### Status bar (`.ship-statusbar`, bottom)

**Overview.** 24px-high bottom strip. Renders 7 cells: connection dot, branch name, ab indicator, view-status start, (anonymous), view-status end, theme indicator. Static layout — cells are injected at boot, contents update via polling (15s) and events.

**Behavior log.**
- Height: 24px exactly. Full viewport width.
- Cells observed: `status-dot` (empty), `status-branch="madvib"`, `status-ab` (empty), `view-status-start` (empty), (one anonymous cell), `view-status-end` (empty), `status-theme="ship-light"`.
- **`status-theme` shows the raw theme id `ship-light`, not a human label.** Clicking it does nothing (no popover opens). The `setShipTheme()` global works but the status cell doesn't bind to it.
- `status-branch` updates via polling — 15s interval per `setInterval(updateStatus, 15000)` in main.ts. Not subscribed to git events, so branch changes lag.
- No daemon-health indicator visible — `status-dot` is empty in this run. Expected behavior is a green/red dot but the binding is missing or asleep.

**PRESERVE.**
- **24px height.** Wide enough to read, narrow enough to feel ambient. Don't let SUI version push this to 32+.
- **Branch in the chrome (not buried in a settings panel).** Git context is one of the few things users want always-visible.
- **Cell-based layout via per-cell IDs.** Easy to inject view-scoped content (e.g., "saving…" from a view-side script writing to `view-status-end`). The `view-status-start` / `view-status-end` cells are a good extension point.

**AVOID.**
- **`status-theme` cell is dead.** It displays text but doesn't open the theme picker. Either wire it up or remove it.
- **`status-dot` is empty.** Daemon health is the kind of thing users need at a glance; not having it is worse than having a broken indicator. Make it work or drop the cell.
- **Polling (15s) for branch state.** Should subscribe to `git.*` mesh events instead — branch and ahead/behind shouldn't lag.
- **Raw theme id as a label.** "ship-light" is not a user-facing string; show "SHIP Light" or the localized name.

**Open questions.**
- Should the status bar host workspace-id at all? Or is that better in the project switcher tooltip?
- Should view-status content live in the status bar (current) or in the view's own header? Two surfaces fighting for "status" feels muddled.

---

### Dock (`<ship-panel-dock>`)

**Overview.** Bottom-attached panel container — terminal/output/etc. Default closed (opacity:0, height:0). Used for ambient panels that need to be present but not always visible.

**Behavior log.**
- Initial state: `opacity: 0`, `height: 0`, `transform: matrix(1,0,0,1,0,8)` (translateY 8px), `display: flex`. Attribute `active-tab="terminal"`. Inner HTML is 4592 chars (rendered but hidden).
- **Open/close transition:** `height 0.2s cubic-bezier(0.16,1,0.3,1), opacity 0.2s cubic-bezier(0.16,1,0.3,1), transform 0.2s cubic-bezier(0.16,1,0.3,1)`. Synchronized 200ms, same curve as the rail tiles (just slower). Slides up + fades in.
- Could not exercise the open path from the rail in this run (the launcher tile click toggled the launcher, not the dock).
- The dock is rendered into the third grid row of the body, at `y=884` in a 900px viewport, which means it sits flush against the status bar with 16px slack. Position is `relative` (not fixed) — it doesn't overlay content, it pushes the view content up when it opens.

**PRESERVE.**
- **Push-not-overlay layout.** Dock takes layout space when open instead of covering the view. Less disorienting than a typical floating drawer.
- **Pre-rendered content with opacity-and-height collapse.** Open is just a CSS change, not a fresh mount — fast and state-preserving.
- **200ms tri-property animation (height+opacity+transform) with one easing.** Looks intentional, feels coherent.

**AVOID.**
- **`active-tab="terminal"` is hard-coded** at boot. The dock has the bones of a multi-tab surface but the wiring to switch tabs from the chrome isn't visible. Either ship the tab UI or remove the attribute.
- **No keyboard shortcut to open/close the dock.** Has to be triggered through the chrome somewhere; not discoverable.
- **Display: flex on a height-0 hidden element** is harmless but pretests should test that focus and scroll do not get trapped in the hidden subtree.

**Open questions.**
- What's the dock for in v1? Terminal only, or a real ambient panel container (logs, jobs, chat history, etc.)?
- Should the dock have memory of its open state across navigation? (Currently appears to reset.)
- Is opening the dock a per-view affordance (view says "I have a dock pane") or a global affordance (chrome opens dock with last-active tab)?

---

### Launcher (`<ship-launcher>`)

**Overview.** Full-app launcher / pin manager. Opened from the launcher tile in the rail. Currently functions as a pin-management surface: lists what is and isn't pinned to the rail.

**Behavior log.**
- Initial state: `display: none`, `opacity: 1`, `transition: all` (the entire-property shotgun again).
- Clicking the launcher rail tile sets `open` attribute on the launcher. After click, inner HTML jumps from 7 to 16292 chars. So the launcher contents are NOT pre-rendered — they hydrate on open.
- Visible action items observed after open: "Unpin from rail" x7, "Pin to rail" x3 — i.e., the launcher's surface mode is "manage which views appear on the rail" rather than "find/open any view".
- Rect after open: `{x:0, y:0, w:0, h:0}` — the launcher renders content but does not occupy a measurable rect, likely because it uses fixed positioning inside a portal-style host. The Playwright snapshot couldn't measure it.
- No discoverable keyboard shortcut to open the launcher.

**PRESERVE.**
- **The launcher is THE pin-management surface.** Centralizing pin/unpin into one place (instead of a per-tile context menu) is good — discoverability of "I can change the rail" comes from one obvious entry point.
- **Lazy hydration:** content not in the DOM until first open. Boot stays cheap.

**AVOID.**
- **Launcher is JUST pin management.** The name implies "any app any view from anywhere" — opening it expecting an Alfred-style picker and getting a rail-pin manager is a bait-and-switch. Either rename the affordance ("Manage rail") or make it actually launch (overlap with cmdK's find mode is real here).
- **`transition: all`** again — replace with explicit list.
- **No close-on-Escape and no focus trap.** Same modal hygiene issue as cmdK.
- **Pin/unpin items don't show what would happen visually** (no preview of where the pin would appear in the rail).

**Open questions.**
- Does the launcher merge with cmdK in the SUI replacement, or stay separate? They overlap in purpose (jump to a view, but launcher manages set of jumpable views).
- Should "Pin to rail" support drag-to-position, or is FIFO append enough for v1?
- Should pin state be per-project or global to a user?

---

### Toast host (`<ship-toast-host>`)

**Overview.** Document-level toast/notification host. Position attribute (`bottom-right` in index.html) determines anchor. Listens for `ship:notify` window events.

**Behavior log.**
- Dispatched `new CustomEvent('ship:notify', { detail: { level:'info', message:'probe ping' } })`. **No toast appeared.** Toast count remained 0 after 80ms wait. The host element exists but did not respond to the event.
- The mismatch suggests either a) the event name expected by ship-toast-host is different, or b) the handler is conditional and the test event was filtered out, or c) the host hasn't upgraded yet at the time of the dispatch.

**PRESERVE.**
- **Position-as-attribute on the host element** (`position="bottom-right"`) — clean, declarative.
- **Decoupled via window event,** so any view can fire notifications without a service-locator import.

**AVOID.**
- **The contract isn't met by the obvious event.** Either the event shape is undocumented or the host has a bug. Whichever — the SUI replacement must specify and validate the exact event shape it consumes, and surface a console warning when it receives a malformed event.

**Open questions.**
- Should toasts auto-dismiss? At what timing? Currently couldn't even fire one to test.
- Should there be a single toast queue (one at a time, others queue) or a stack (multiple visible)?
- Should errors get a different surface (toasts are bad for sticky/diagnostic content)?

---

### Theme switching

**Overview.** Theme is stored in `localStorage.ship-site-theme`, applied via `data-theme` and a `.dark` class on `<html>`. Boot-time inline script applies the theme before first paint to prevent FOUC. Global `window.setShipTheme(id)` switches at runtime.

**Behavior log.**
- `window.setShipTheme('ship-dark')` flips `data-theme` and `.dark` class within one frame. Sampled body bg every 50ms over 400ms: first sample at t=75ms already shows the final color. No interpolation observed despite `body { transition: all }`.
- **Why no interpolation?** Tokens are `oklch()` color values. Browser support for animating between two `oklch()` values is uneven; in this run the bg snapped instantly. The `transition: all` is a cost we pay (every property change), and the actual color change can't honor it.
- Boot-time inline script in `index.html` runs synchronously before the first paint and reads from localStorage — no FOUC.
- The `syncThemeFromSettings()` IIFE fires-and-forgets on boot to reconcile localStorage with the daemon's settings; if the daemon settings disagree, the page re-themes after boot.
- Theme picker is not visually accessible from the chrome. `status-theme` displays the id but does not open a picker. No top-bar control. `setShipTheme` is global-only, which means users can switch themes only by knowing the function name and typing it into devtools — broken end-user UX.

**PRESERVE.**
- **Pre-paint inline boot script** — flashes are unacceptable, and this avoids them.
- **Two-channel theme application** — `data-theme` attribute (for non-dark themes like `paper`, `halo`, `console`) plus `.dark` class (for the binary light/dark switch some libraries depend on). Don't unify these until proven safe.
- **Reconcile with daemon on boot** so themes follow the user across devices/browsers.
- **localStorage cache** for instant reads on subsequent boots.

**AVOID.**
- **No visible theme picker in the chrome.** Users discover themes only via the settings view. Make the picker accessible from one click on the status bar or topbar.
- **`body { transition: all }`** — wastes cycles. Honest theme transitions (color/background/border-color only, 150-250ms) would still be cheap and look intentional, instead of the current 1-frame snap.
- **Raw theme id (`ship-light`) shown to user** in the status bar. Localize/humanize.
- **Theme switch animation is non-existent in practice** (snap to final color). Either acknowledge it (no transition) or fix the oklch interpolation (use a CSS custom property for the *named* token instead of the resolved color, so the cascade can interpolate via fallback channels).

**Open questions.**
- How many themes are in scope for v1? Code declares 7 (`ship-light, ship-dark, paper, halo, console, contrast, glass`). All shipped? All maintained?
- Should theme follow OS `prefers-color-scheme` by default? Currently does on first boot only.
- Per-view theme overrides (e.g., theme-studio always in light for accurate swatch preview)?

---

## Views

### Dashboard (`/dashboard` → `/api/v1/v/dashboard`)

**Overview.** The landing view. Aggregates: jobs (recent), agents (count + sample), latest events, quick-links. h1 reads "SHIP" with project subhead — branding-heavy.

**Behavior log.**
- Mount latency: 9ms from `router.navigate` to first DOM in `#ship-view-root`. Network fetch returned 200 from `/api/v1/v/dashboard`.
- Total payload: 39144 chars of HTML in the view root. Includes a lot of inline `<style>` blocks per partial.
- H2 sections: "Recent jobs", "Agents 7", "Latest events", "Quick links". Each appears to be its own j2 partial.
- Buttons sampled include things like "gate", "review", "ui", "sui", "+3" (overflow chips). Many short-label buttons read as tag chips rather than actions — visual ambiguity (chip vs. button vs. badge).
- No `<table>` elements; one `<ul>`. Bulk of layout is custom divs.
- `aria-busy` / skeleton elements: 0. Loading state is not represented in the j2 template; the daemon must render fully before sending.
- Failed requests on this view: `GET /api/v1/view-source/shell/` (the pre-existing 404 noise).

**PRESERVE.**
- **9ms mount.** Server-rendered HTML pre-baked into one fetch is fast. Carry this budget.
- **Four-section grid (jobs / agents / events / quick-links)** is a sensible information density for a landing view. Don't bury one of these in a tab.
- **Counts in headers ("Agents 7")** as inline labels — terse, useful.

**AVOID.**
- **Inline CSS per partial.** The view root contains style blocks (`.ship-form-field { display: flex; ... }`) inline. This means token changes don't reach those styles, and tokens-via-CSS-vars only work where the partials happened to use them. Pull all styling out of the j2 partials into the design-system.
- **No skeleton/loading state.** The daemon's full-render approach means there's nothing to show if a section is slow. Going forward, view sections should each be able to render with a loading placeholder.
- **Chip-shaped buttons** ("gate", "review", "+3") that don't clearly indicate whether they're filters, badges, or actions. Each section that uses chips needs an interaction spec, not just CSS.

**Open questions.**
- Should Dashboard be the landing view for v1, or should it become a customizable home (drag-to-arrange cards)?
- Should agent/event/job counts be live (SSE/mesh-subscribed) or refresh-on-mount?
- Quick-links — who curates? Hard-coded today.

---

### Canvas (`/canvas` → `/api/v1/v/canvas`)

**Overview.** A free-form text/feedback canvas. h1 "Canvas", h2 "Recent Feedback". Sparse — 7101 chars of HTML.

**Behavior log.**
- Mount: 41ms. 200 response. View renders successfully but very lean — no buttons, no inputs, no tabs.
- Renders a heading and a "Recent Feedback" section; appears to be mostly a placeholder/shell that loads content separately (or has nothing to show).
- Subroute `/canvas/:doc` is registered in `main.ts` for a doc-specific canvas — but the index view does not link to any docs in this run.

**PRESERVE.**
- **Cheap mount + dedicated subroute pattern** (`/canvas` + `/canvas/:doc`). Don't collapse these into a single noisy view.

**AVOID.**
- **The index has no entry-point to a `:doc` route.** Users land on `/canvas` and cannot get into any document from the UI.
- **No empty-state copy** — just an h1 and a heading. If there's nothing yet, say so.

**Open questions.**
- What IS the canvas for? Feedback capture? Free-form notes? Draft documents? The view label is ambiguous and the implementation doesn't disambiguate.
- Should canvas docs auto-list on the index, or only show what's pinned/recent?

---

### Collections (`/collections` → `/api/v1/v/collections`)

**Overview.** Generic collection browser/editor. h3 "Specs" (so the default collection is `specs`). 53594 chars — heavyweight. Lots of checkboxes (8 sampled, all `type=checkbox`, no placeholders) — suggests a multi-select grid.

**Behavior log.**
- Mount: 27ms.
- Header h3 reads "Specs" — the view is generic but defaults to the `specs` collection. `/collections/:collection` is wired for switching.
- 8 checkboxes detected at the top of the view — likely select-all + per-row. No native `<table>` (uses divs with role or class-based grid).
- No tabs detected, no buttons sampled (selectors did not find named buttons in the shallow snapshot).
- 0 cards, 0 lists, 0 tables — view layout is custom enough that generic selectors miss its primitives.

**PRESERVE.**
- **Generic collection view with route per collection (`/collections/:collection`)** — one view definition, many surfaces. Carry this composition into the SUI port; don't write one view per collection type.
- **Checkbox-driven multi-select on rows** — when you have a collection of editable rows, multi-select is the right paradigm.

**AVOID.**
- The h3 is "Specs" — but the URL is `/collections` (no `:collection` segment). User has no on-page indication that they're looking at a *default* and can switch. Make the header a picker, not a static title.
- 53KB of HTML in one shot with no virtualization mentioned — at 1000+ rows this view will jank. Plan for virtualized rendering in the SUI port.

**Open questions.**
- How many collection types in v1? (Specs, Skills? Agents? Threads?)
- Are collections schema-driven (each row has typed columns) or free-form (kv-bag)?
- Should multi-select be persistent across navigation, or transient?

---

### Agents — list (`/agents` → `/api/v1/v/agents`)

**Overview.** Card-grid of agents. h1 "Agents". 24158 chars. 77 elements matched the "card-ish" selector — but most are nested fragments of the same card, so the actual agent count is lower (the probe was greedy).

**Behavior log.**
- Mount: 23ms.
- Cards show: agent name as a heading ("Commander"), one-line description ("Ship orchestrator — captures intent, dispatches specialists…"), a single metadata line ("5 skills · 1 MCP · 0 rules"). The "summary" pattern is good for scanning.
- Card transition is `all` — same shotgun pattern.
- Card box-shadow on synthetic hover does not change (could be a real-:hover thing only; could be that hover affordance doesn't exist). Cards are not visually clickable in the probe data.
- **No `<a href="/agents/:id">` links found inside any card.** The list view does not link to detail views. There is no way to click into an agent.
- Buttons in the view: chips like "gate", "review", "+3", "ui", "sui" — appear to be skill/tag chips per card, with overflow `+N`. Whether those chips themselves navigate is unclear (probe couldn't activate them).

**PRESERVE.**
- **Card summary shape** (name + 1-line gloss + metadata count line) is good for scanning.
- **Skill chips visible at-a-glance** so users can identify "the agent with `review` skill" without clicking in.
- **`+N` overflow pattern for chips** is clean — keeps card heights uniform.

**AVOID.**
- **No detail link from list to detail.** Most important miss. Cards must be navigable.
- **`transition: all`** on cards.
- **No empty state** for "no agents yet" (probe didn't trigger this, but the j2 partial doesn't appear to render one).
- **No filter/sort/search controls** on the list — at 7 agents the user gets a card-grid; at 70 it'll be unscannable.

**Open questions.**
- Card click → detail, or chip click → filter list? Cards need a primary action and skill chips need either inert or filtering behavior — both, but the spec needs to choose.
- Should the list group by author / kind / pinned-ness?
- How is "selected agent" expressed in URL (`?selected=`)?

---

### Agents — detail (`/agents/:id`)

**Overview.** Detail page for one agent. Not exercised in the probe — the list view doesn't expose any links to `/agents/:id`, so the probe couldn't navigate into a detail. The route is wired (`onRoute("/agents/:id", ...)`) and `/agents/:id/wire` / `/agents/:id/settings` sub-routes exist.

**Behavior log.**
- Could not reach this route from the UI. Direct in-app navigation would work (`window.router.navigate('/agents/<id>')`) but no agent-id is exposed in the list view's DOM.
- Sub-modes registered: `/agents/:id/wire` (wire/canvas layout, default) and `/agents/:id/settings` (panels: skills, MCP, permissions, rules).
- Static sub-route `/agents/create` is registered BEFORE the dynamic `/agents/:id` (good — prevents Navigo matching "create" as an id). Worth carrying that pattern explicitly into the SUI router config.

**PRESERVE.**
- **Two sub-modes for the same detail (`/wire` canvas, `/settings` panels).** Different mental models for the same agent — the route segment lets users link to either.
- **Static-before-dynamic route registration discipline** is a real correctness issue. The j2 shell encodes it. Make the SUI router enforce or warn on the pattern.

**AVOID.**
- **No link from list to detail** (see Agents — list).
- **No documented contract** for what `?` query params the detail page accepts. (Probe couldn't test.)

**Open questions.**
- Is `/wire` or `/settings` the right default sub-mode? Code uses `/wire`.
- Should detail have an "edit" mode separate from `/settings`?

---

### Agents — create (`/agents/create`)

**Overview.** Multi-step wizard for creating an agent. Four labeled steps in the page chrome: "1 Template", "2 Identity", "3 Wiring", "4 Review". Each section has its own h2 header inline.

**Behavior log.**
- Mount: 60ms (slowest of the v1 views).
- 4 step-tab buttons, 4 sections (Template / Identity / Wiring / Review) — all rendered into the page at once (long-scroll wizard rather than a step-at-a-time flow).
- 16 card-like elements — likely template cards in step 1.
- No inputs detected by the probe (because inputs are likely inside ship-* web components with shadow DOM that the shallow probe couldn't pierce).

**PRESERVE.**
- **Numbered step chips at the top** ("1 Template, 2 Identity, …") — good for orienting users in a flow.
- **Single-page long-scroll wizard** (vs. step-at-a-time modal) gives users a sense of the whole shape upfront. Don't force-modal this in v2.
- **Template cards as visual choices in step 1.** Picking from cards is faster than dropdown.

**AVOID.**
- **No step validation indicator.** The chips don't show completed/incomplete state. Users have no idea where they are in the flow.
- **Wizard scrolls the whole document** — no sticky navigation. On a small viewport, the user loses orientation.
- **"Cancel" link in header** (per focused-text "New agent Cancel ...") with no confirmation — risk of accidental data loss on a multi-step form.

**Open questions.**
- Long-scroll wizard vs. stepped — both are valid. Pick.
- Save-as-draft? Currently there's no persistence between sessions visible.
- Are templates user-extensible or app-curated?

---

### Skills (`/skills` → `/api/v1/v/skills`)

**Overview.** Skills browser. 65432 chars (the heaviest non-theme-studio view). Filter chips at the top ("All 52, Smart 25, Docs 27") then a list of skills (one per row, presumably). Has a search input ("Search skills...").

**Behavior log.**
- Mount: 16ms.
- Filter chips: "All 52", "Smart 25", "Docs 27" — three kinds (or counts), with count inline.
- Visible buttons (after filter chips): skill names like "cmdr", "code-review", "collection-authoring", "configure-agent", "design-workflow", "diff-review", "drizzle-studio" — these read as list rows / nav items.
- Search input: `<input placeholder="Search skills...">` present.
- Probe typed "collection" into search; row count before and after was 0 because the probe's "row" selector missed the underlying primitive (skills are probably rendered inside a `<ship-list>` shadow-DOM component).
- No native `<table>`, no `<ul>`, no detected cards — fully custom layout.
- Filter chip "Smart" and "Docs" suggest a binary categorization of skills. Numbers add up (25+27=52).
- View name from the URL is `/skills/:id`; the index has no `<a href>` to individual skills in the shallow scan — but the rows are interactive (buttons), so the click handler probably calls `window.router.navigate`.

**PRESERVE.**
- **Filter chips with live counts.** "All 52, Smart 25, Docs 27" lets users mentally model the dataset before clicking.
- **Search input as a primary affordance.** At 50+ skills, search is the only fast path.
- **Two-category split (Smart vs Docs)** — a forced binary helps scanning.
- **List density** — appears to be one skill per row, narrow row height. Right shape for a long list.

**AVOID.**
- **Click target is a `<button>` not `<a>`.** Means no right-click "open in new tab", no copy-link. Use `<a href>` for navigation, not buttons.
- **No keyboard nav between rows.** (Probe couldn't drive arrow-key down a button list — would need explicit code, didn't see it.)
- **Skills are in shadow-DOM components** — view-level virtualization, count, and a11y are hidden from the host. Make the row primitive expose enough state to the chrome.

**Open questions.**
- Should the filter chips be sticky on scroll?
- Should search support fuzzy matching, prefix, or both?
- Does the `Smart` vs `Docs` binary scale? What about a third category?

---

### Chat (`/chat`)

**Overview.** The chat view. **Currently broken.** `/api/v1/v/chat` returns 500. The shell renders `mount.ts`'s error fallback.

**Behavior log.**
- Mount: 40ms (the fallback HTML mounts immediately).
- View payload from daemon: HTTP 500.
- Rendered fallback: "The daemon couldn't render this view. The details below may help. GET /api/v1/v/chat 500 Internal Server Error" with a Retry button.
- The fallback's "Retry" button calls `window.__shipRetryMount(endpoint)` — a clean lower-level→higher-level bridge. (This is wired via `mount.ts` writing a global; `main.ts` exposes the implementation.)

**PRESERVE.**
- **The mount-error fallback is well-designed.** Shows status code, endpoint, and a Retry. Don't lose this when porting.
- **`__shipRetryMount` indirection** keeps mount.ts decoupled from the router. Good seam.

**AVOID.**
- **No alternative path forward in the fallback** — Retry is the only option. If the daemon is genuinely broken, the user is stuck. Add "Go to dashboard" / "Report this error" buttons.

**Open questions.**
- The chat view is one of the 9 v1 default views. Fixing the 500 is out-of-scope for this job (per spec) but in-scope for v1 launch.
- Does chat live in the dock instead of in a view? The `app-agent-modal` in index.html overlaps in purpose.

---

### Settings (`/settings` → `/api/v1/v/settings`)

**Overview.** App settings — theme, providers, terminal, advanced. h1 "Settings". 19402 chars. Section-per-area layout.

**Behavior log.**
- Mount: 46ms.
- Sections: Theme, Providers, Terminal, Advanced. Each is an h2 + content block.
- The probe's control extraction returned empty arrays for each section — likely because the inputs are ship-* web components (shadow DOM hides them from generic `input, button, select` selectors). So the *primitive* controls exist but the host can't see them.
- 0 native form controls visible in the view-root scan.
- No save button found — likely settings auto-save (good) or have inline save in each ship-* component.

**PRESERVE.**
- **Sectioned settings (Theme/Providers/Terminal/Advanced)** — opinionated grouping. Don't fragment.
- **Auto-save (or per-section save inside ship components)** — fewer footguns than a single global Save.

**AVOID.**
- **Settings inputs are not introspectable** from the page level — host can't tell which settings exist. If a settings index/search is wanted later, the design needs to expose them via a registry.
- **No "test" / "validate" affordance** on Provider settings (probe couldn't see one).

**Open questions.**
- Should settings be searchable / `⌘K`-findable per-control?
- Per-project settings vs. user-global — where's the line?
- Theme settings here vs. Theme Studio — overlap or different audiences?

---

### Setup (`/setup` → `/api/v1/v/setup`)

**Overview.** First-run setup wizard. h1 "Welcome to SHIP". Sections: "Pick a project folder", "Connect an AI provider", "Ship is ready".

**Behavior log.**
- Mount: 73ms (a tick slower because the partial includes a welcome animation).
- 3 h2 sections — linear "do this, then this, then done" flow on a single page.
- No tabs, no step chips. Implicit progress by scroll position.
- Failed requests on this view include a *very long* GET to `/api/v1/view/settings/sse` with serialized datastar state encoded into the query string (~1KB URL). The signal: setup-time settings are SSE-streamed via a single beefy query string, which is fragile (URL length limits, encoding bugs).
- `transition: all` on the wrap (`.setup-view`) — same shotgun pattern.

**PRESERVE.**
- **Linear three-step welcome** — folder → provider → done. Don't add steps.
- **Sectioned content rather than separate routes** — keeps the user on one URL through the whole flow; back-button safe.

**AVOID.**
- **No progress indicator.** Users scroll and find the next section; nothing tells them how far through they are or that they've completed step 1.
- **No "you can skip and come back later"** — for a first-run flow, an escape hatch is critical.
- **Datastar state serialized into a URL query string for SSE.** Replace with POST + server-side state or a dedicated state channel.

**Open questions.**
- Should setup be a modal over a default view, or its own route (current)?
- Should setup persist its position so a user who closed the tab resumes mid-flow?
- Connect-provider should sniff for env vars / existing credentials — currently surfaces a form regardless.

---

### Git (`/git` → `/api/v1/v/git`)

**Overview.** Git view — was probably tabs + split panes (per source comment). **Currently the route does not mount fresh content** — after a `/setup` navigation, calling `router.navigate('/git')` did NOT fire a `/api/v1/v/git` request and left the previous setup HTML in place. So git is effectively broken from in-app navigation in this state.

**Behavior log.**
- Mount: 25ms — but mountAt is just "the view-root has > 200 chars", which is true from the stale content.
- Network: no `GET /api/v1/v/git` observed.
- Page content showed setup's headings ("Welcome to SHIP", "Pick a project folder", …) — verified the stale-state issue.
- View source (`.ship/views/git/index.html.j2`) does exist and describes "Layout B. Tabs + split panes, composed from ship-set primitives. Data: fetched via `ship.git.*` inline. Overview is one call; diff + …" — so the intended view is tab + split.

**PRESERVE.**
- **Intent of the view (tabs + split panes for status / diff / log / branches)** is the right shape for a git surface. Don't deviate.
- **Inline `ship.git.*` SDK calls** (per source comment) — view fetches data directly via the typed SDK, not via a custom endpoint.

**AVOID.**
- **In-app navigation can fail silently and leave previous view content in place.** This is the single worst chrome bug observed. The SUI replacement's router must guarantee that navigation either succeeds with new content or shows an error fallback — it must never leave stale state.
- We could not observe actual git-view behavior because of the above.

**Open questions.**
- Are diffs paginated or all-at-once?
- Branch ops (checkout, push, rebase) — confirm or in a separate UI?
- Conflict resolution: in-shell or punt to terminal?

---

### Pipelines (`/pipelines` → `/api/v1/v/pipelines`)

**Overview.** Pipeline list. h1 "Pipelines". 20909 chars. Shows pipeline name + one-line description + step count per row.

**Behavior log.**
- Mount: 39ms.
- Rows observed include `multi-provider-smoke Three-step pipeline 3 steps`, `shell-smoke Pure-shell headless smoke — 3 steps`, `ship-dev Ship's own dev loop as a Ship p…`, `smoke Fast pipeline mechanics smoke test 3 steps`, `tdd Test-driven development — tests firs 3 steps`. The "N steps" is shown as a chip after the description.
- **No `<a href>` to `/pipelines/:slug`** — rows are buttons. Cannot copy a link to a pipeline.
- 0 cards/lists detected (custom layout).
- Pre-rendered list — no loading state.

**PRESERVE.**
- **One row per pipeline with name + tagline + step count** — terse, scannable.
- **Step count as a chip** — quick complexity signal at a glance.

**AVOID.**
- **Rows are `<button>` not `<a>`** — same anti-pattern as Skills. No right-click open, no link copy.
- **Description truncation cuts off in the middle of words** ("shell-smoke Pure-shell headless smoke — ") — use a CSS `text-overflow: ellipsis` with a hover-to-expand instead of hard truncation.
- **No search/filter** on pipeline list — at 5+ pipelines it's still scannable; at 20+ it won't be.

**Open questions.**
- Should the list show last-run status (success / fail / running)? Currently doesn't.
- Should there be a "favorites"/"pinned" pipelines section?

---

### Pipelines — focus (`/pipelines/focus/:name`) and History (`/pipelines/history`)

**Overview.** Two sub-routes: `/pipelines/focus/:name` (single-pipeline detail) and `/pipelines/history` (cross-pipeline run history).

**Behavior log.**
- `/pipelines/focus/:name`: not exercised (no link from index).
- `/pipelines/history`: mounted at 45ms. h1 "History". 4607 chars — very lean, probably needs hydration. The probe saw no rows.
- The history route forwards `location.search` into the daemon URL so pagination/search survives SPA nav — good URL discipline (worth preserving in SUI).

**PRESERVE.**
- **Query-string passthrough from SPA route to view fetch** — keeps URL state truthful. Carry into SUI router.

**AVOID.**
- **No links from `/pipelines` index to `/pipelines/focus/<slug>`** — same dead-end as Agents list.

**Open questions.**
- Should history live as a tab on /pipelines or as a separate route (current)?

---

### Pipelines-board (`/pipelines-board`)

**Overview.** Listed in spec; **no route registered** in `apps/shell/src/main.ts` and no working entry from chrome. The directory exists at `.ship/views/pipelines-board/`. Not reachable from the SPA.

**Behavior log.**
- Navigating to `/pipelines-board` via `router.navigate` hit the `notFound()` handler.
- View source exists but daemon is the only way to see it (and the SPA doesn't link to it).

**PRESERVE.**
- (n/a — view not reachable.)

**AVOID.**
- **Orphan view files** with no route wiring. Either register or remove.

**Open questions.**
- Is pipelines-board the kanban-style runner view? If so, should it merge with `/pipelines` as a view-mode rather than a separate route?

---

### Events (`/events` → `/api/v1/v/events`)

**Overview.** Event log. h1 "Events". 168170 chars — by far the heaviest view (4× any other).

**Behavior log.**
- Mount: 6ms (fastest of all v1 views) — but mount-at-200-chars is misleading here; the 168KB takes longer for the browser to actually render.
- Filters present: "All types", "supervisor.sweep_completed", "supervisor.sweep_started", "All actors", "ship" — a pair of facet selectors (type + actor), each rendered as a chip list.
- 0 native tables — events render as custom rows.
- **No virtualization detected** — 168KB of HTML implies the daemon serialized many event rows into a single render. At 10× the events, this will jank.
- No skeleton state.
- No scroll observed by probe (`document.scrollY` stayed at 0 after `window.scrollTo(0, 400)`), which suggests the events list is inside a constrained scroll container (its own scroll), not the document body.

**PRESERVE.**
- **Faceted filters (type + actor) as primary controls.** Right shape for an event log. Keep the facet-chip pattern.
- **Independent scroll for the events panel** (not document scroll). Means the chrome stays visible while users scroll long event lists.

**AVOID.**
- **168KB of HTML in one render.** Switch to virtualized rendering (windowed list) in the SUI port. Even at v1 scale this is wasteful; at v2 scale it's broken.
- **No event detail expansion** detected — each row is opaque. Users need a way to see the event payload (typically click row → expand or open detail panel).
- **Filter is static** (server-rendered) — switching a facet requires a fresh `GET` instead of client-side filtering on already-loaded rows.

**Open questions.**
- Should events stream live (SSE) or refresh on demand?
- Tail-vs-paginate UX — should the view always tail latest, or remember scroll position?
- Multi-select rows for bulk replay / ack?

---

### Mesh (`/mesh` → `/api/v1/v/mesh`) — labeled "Mission Control" in rail

**Overview.** Mesh view = the actor/process/integration health surface. h1 "Mesh", h2 "Actors", h2 "Ship processes", h2 "Integrations". 3 tables.

**Behavior log.**
- Mount: 23ms.
- Three h2 sections each rendered as a native `<table>` (only view in the audit that uses native tables).
- No buttons sampled — the page is read-only.

**PRESERVE.**
- **Native `<table>` use here is right** — tabular data with column semantics, sortable in principle. Tables aren't the enemy; misused tables are. Keep this pattern for genuinely tabular surfaces.
- **Three-section vertical layout** (Actors / Processes / Integrations) is the right cut of the mesh universe.
- **Rail label "Mission Control"** is more evocative than the route name "Mesh" — the visual chrome label can be marketing while the route segment stays technical. Worth preserving the split.

**AVOID.**
- **Tables are read-only** — no row actions (restart, inspect, drain). Add row-level actions; that's what users will want.
- **No search/filter** on actor/process tables.

**Open questions.**
- Should this be live (SSE-streamed) or refresh-on-mount?
- Should "Mission Control" be the rail label everywhere, or only in the rail?

---

### Overview (`/overview` → `/api/v1/v/overview`)

**Overview.** A meta-view over Ship's own architecture / status. 31222 chars. Filter chips include "all, spec, building, shipping, complete" (status taxonomy) and "all, primitive, runtime, interface, meta" (architectural layer taxonomy).

**Behavior log.**
- Mount: 10ms.
- Two facet-filter chip groups (status + layer).
- No detected headings (the partials use custom semantic markup).

**PRESERVE.**
- **Dual-facet chip groups** (status + layer) — the right shape for a multi-axis browse.
- **Status taxonomy (spec / building / shipping / complete)** is meaningful; bake it into the SUI port if Overview survives.

**AVOID.**
- **No h1/h2 detected** — page is hard to navigate by structure for screen readers.
- It's unclear who this view is for — internal Ship development team, or user-facing? Decide before porting.

**Open questions.**
- Is Overview a v1 default view, or an internal-dev-only surface?
- If kept: should it be customizable (saved facet sets)?

---

### Specs (`/specs` → `/api/v1/v/specs`)

**Overview.** Specs index. h1 "Specs". 6226 chars — very lean, probably a stub.

**Behavior log.**
- Mount: 6ms.
- Has a 2-column layout (per CSS: `grid-template-columns: 280px 1fr`) — left rail + main pane. So the view IS structured but the content wasn't loaded in the probe sample.
- No buttons or inputs detected.

**PRESERVE.**
- **Two-column layout (sidebar list + main detail)** is the right shape for a docs/spec browser.

**AVOID.**
- **Empty-on-load** — view is structured but no spec is loaded by default. Pick a default to show (most-recent, or alphabetically first).

**Open questions.**
- How is "spec" distinct from collections/specs? They overlap.
- Should specs be authored in-shell or only viewed?

---

### Theme Studio (`/theme-studio` → `/api/v1/v/theme-studio`)

**Overview.** Live token editor + preview. The second-largest view (114357 chars). Layout: left editor pane (360px) + right preview (1fr) per CSS grid. 224 swatches, 224 inputs — a token-per-input editor.

**Behavior log.**
- Mount: 4ms (fastest) — but 114KB is heavy to actually paint.
- Editor uses 224 `<input>` elements (one per token).
- Preview pane renders representative typography ("Primary headline", "Body copy lives in muted-foreground…") and primary/destructive button samples to show theme application.
- "Computing contrast… Pair Ratio Min AA body AAA Status" — contrast checker UI present.
- No native `<table>` rendering but there's one (the contrast checker output).
- No tabs visible — single-pane editor.
- `data-theme` attribute change does propagate live into the preview (per the design-system docs in CLAUDE.md context).

**PRESERVE.**
- **Live token editor with side-by-side preview** — direct manipulation is the right paradigm for design tokens.
- **Contrast checker baked in** (showing AA/AAA pair ratios) — every token editor should ship this.
- **224 token inputs in one panel** is information-dense; users who came here came for control. Don't paginate.

**AVOID.**
- **No grouping/sectioning of 224 inputs** — overwhelming on first sight. Add collapsible groups (color, type, space, motion).
- **`transition: all`** appears on preview elements — when editing tokens, transitions slow down feedback. Either remove or honor an "instant" toggle.
- **No undo/redo** detected; with 224 inputs and live preview, accidental edits are easy.

**Open questions.**
- Should theme-studio be a v1 default view (it's currently in the catalog)? Or an internal-dev tool?
- Export format: CSS-vars, JSON, both?
- Should there be a "compare two themes" mode (left = current, right = draft)?

---

### Views requested by acceptance spec but not present as routes

The job spec lists 21 views to cover. The following are mentioned but not implemented as j2 routes in this codebase:
- `/home` — no route; `/dashboard` plays this role.
- `/artifacts` — no route, no view directory. The mention picker (`@`) suggests it as a target — dead reference.
- `/sessions` — no route, no view directory.
- `/ship-ui-studio` — no route here; the comment in `main.ts` says "lives at /sui/ship-ui-studio (standalone view-host) until shell 2.0 ports it in".
- `/agents/[id]` detail — route registered but no entry-point from the list view.
- `/agents/[id]/wire`, `/agents/[id]/settings` — sub-modes registered but no entry.
- `/pipelines/focus/:name`, `/pipelines/run/:slug` — sub-routes registered but no entry from `/pipelines`.

**AVOID.** Routes registered without entry-points are technical debt: callers can hit them only via copy-paste URL or `window.router.navigate` from devtools.

**Open question.** Should v1 ship a `/sessions` view (chat history index), or is session listing exclusively the responsibility of the command bar's mentions / session picker?

---

## Cross-cutting recommendations for the SUI replacement

These are the patterns the harvest surfaced that show up across multiple surfaces and need design decisions before per-view porting begins.

1. **URL contract is canonical.** Every view must be deep-linkable and reload-safe. The current `Navigo("/")` + `/shell/` base mismatch is the symptom of an unowned URL contract — fix the contract first, port second.
2. **Active-route state is router-state, not DOM-scan state.** The j2 shell's `updateNav()` boot-time scan misses dynamically-rendered anchors and never updates. SUI should bind active state reactively to router output.
3. **List → detail navigation is non-optional.** Every list view must have `<a href="/detail/:id">` rows, not buttons. This affects Agents, Pipelines, Skills, Specs at minimum.
4. **No `transition: all` anywhere.** Audit every component for explicit transition properties.
5. **Focus management is a feature, not an emergent property.** Cmd-K, dock, launcher, agents/create wizard — all need designed focus-on-open, focus-trap-on-modal, focus-restore-on-close.
6. **Use `<a href>` for navigation, `<button>` for actions.** This is a recurring mismatch.
7. **Empty states and loading states are part of the view contract.** Multiple views render blank when there's no data; the SUI port should require both.
8. **Mention picker, `/` slash list, and rail items must source from a single registry.** Today they're separately curated and drift apart (mention picker references `@artifacts` while no `/artifacts` route exists).
9. **Tooltips beat `title` attributes.** Replace every `title="…"` with a proper ARIA-labeled tooltip primitive that triggers on focus too, not just hover.
10. **Server-rendered j2 + datastar SSE is the right baseline for fast first paint** (most views mount in <50ms). Don't lose this; the SUI port should preserve "view content arrives in one HTTP fetch" as a performance budget.

---

## Notes on the probe methodology

- All measurements taken from a single Playwright session: headless Chromium, 1440×900 viewport, dev daemon at `:9385`.
- Three scripts: `/tmp/j2-probe/probe.mjs` (initial per-route survey), `probe2.mjs` (chrome + per-route deep), `probe3.mjs` (shadow-DOM-aware + chrome behavior).
- Numbers are wall-clock from `performance.now()` taken inside the page. They are useful as relative budgets, not absolute SLAs.
- Several behaviors required shadow-DOM traversal (`element.renderRoot`) to inspect — the host-level DOM is misleading because Lit components hide their internals.
- Synthetic mouse events (`pointerenter`, `mouseover`) do not always fire CSS `:hover`. Where hover-state is reported as "did not change", real-cursor behavior may differ; verbal descriptions reflect *what the probe could measure*, and visual hover affordances should be re-verified by a human pass before final design lock.
- This report intentionally avoids screenshots. Behaviors over pictures.
