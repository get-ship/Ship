# shell2 Playwright probe — evidence for shell2-fundamentals

**Captured**: 2026-05-13 against `http://127.0.0.1:3270/` (vite dev). Headless chromium, 1440×900.
**Screenshots**: `.artifacts/sui-arc/probe-shots/*.png`
**Full trace**: `.artifacts/sui-arc/probe-shots/trace.json`
**Console log**: `.artifacts/sui-arc/probe-shots/console.log`

---

## Rail inventory (11 elements found inside `<sui-rail>`)

| # | Tag | href | title | text shown |
|---|---|---|---|---|
| 0 | a | `/` | Dashboard | "Dashboard" |
| 1 | a | `/agents` | Agents | "Agents" |
| 2 | a | `/git` | Git | "Git" |
| 3 | a | `/ship-ui-studio` | UI Studio | "UI Studio" |
| 4 | a | `/sui-demo` | SUI Demo | "SUI Demo" |
| 5 | a | `/demos` | Demos | "Demos" |
| 6 | a | `/mesh` | Mission Control | "Mission Control" |
| 7 | button | — | Switch project | (icon) |
| 8 | button | — | Open launcher | (icon) |
| 9 | button | — | Create new | (icon) |
| 10 | button | — | Show rail | (icon — this is the toggle) |

**Bugs from this inventory**:
- Rail renders text labels alongside icons (Dashboard, Agents, Git, etc) → violates user spec ("ICONS ONLY when visible")
- Items 4 (`/sui-demo`) and 5 (`/demos`) point at routes that **don't exist** — user gets "Page not found"
- Item 10 ("Show rail") is the toggle button **inside the rail itself** — user wants the toggle in the top-nav, not in the rail. Plus user said NO arrow / no in-rail toggle.

## Top-nav inventory (17 elements found inside `<sui-nav-top>`)

| # | Tag | title | text |
|---|---|---|---|
| 0 | button | Open command bar | "Ask Ship or jump to…⌘K" |
| 1 | button | — | "Start session" |
| 2 | button | Close dialog | (icon) |
| 3 | a | Ship home | (logo) |
| 4 | button | — | (mystery) |
| 5 | button | — | (mystery) |
| 6 | button | Choose theme | "SHIP Light" |
| 7-13 | buttons | per-theme | 7 themes inline (SHIP Light, SHIP Dark, Paper, Halo, Glass, Console, High Contrast) |
| 14-16 | buttons | — | (mystery) |

**Bugs from this inventory**:
- 17 buttons. User asked for **3**: Menu (toggles rail), CLI (debug panel), and the chat button is to be **removed**.
- The 7-theme inline strip is bloat. Should be a single "Choose theme" trigger that opens a picker.
- 3 mystery buttons with no labels.
- No "Menu" button that toggles the rail (user explicitly asked for one).
- No "CLI" button that opens debug panel.

## Routing observations

Direct GET to `/sui/<view>` (daemon serves):

| Route | HTTP | Renders meaningfully? |
|---|---|---|
| `/sui/home` | 200 | ✓ Counter + live |
| `/sui/agents` | 200 | ✓ Empty-state OK |
| `/sui/overview` | 200 | ✓ Stats card |
| `/sui/sessions` | 200 | ✓ Empty-state OK |
| `/sui/workspaces` | 200 | ✓ Empty-state OK |
| `/sui/chat` | 200 | ✓ Composer |
| `/sui/git` | 200 | ⚠️ **`len=0` — empty body** |
| `/sui/canvas` | 200 | ⚠️ Loading… stuck |
| `/sui/specs` | 200 | ⚠️ Loading… stuck |
| `/sui/dashboard` | 200 | ⚠️ Loading dashboard… stuck |
| `/sui/theme-studio` | 200 | ⚠️ Loading tokens… stuck |
| `/sui/skills` | **TIMEOUT** | hangs networkidle |
| `/sui/mesh` | **TIMEOUT** | hangs networkidle |
| `/sui/events` | **TIMEOUT** | hangs networkidle |
| `/sui/settings` | **TIMEOUT** | hangs networkidle |

Click via shell2 rail → routes through internal navigation (no `/sui/` prefix):

| Clicked link | Result |
|---|---|
| `/` (Dashboard) | shows landing page |
| `/agents` | ✓ renders |
| `/git` | ❌ **`Mount failed: Failed to resolve module specifier "@ship/sui-..."`** |
| `/ship-ui-studio` | ✓ renders |
| `/sui-demo` | ❌ **`Page not found - The view sui-demo isn't registered`** |
| `/demos` | ❌ **`Page not found - The view demos isn't registered`** |
| `/mesh` | ✓ renders |

## Console errors (selected)

```
[error] [sui] effect threw: TypeError: r.headerEl is not a function
[error] [sui] effect threw: TypeError: r.bodyEl is not a function
[error] Failed to load resource: the server responded with a status of 500 (Internal Server Error)
[reqfail] http://127.0.0.1:3270/_sui/data?expr=ship.agents.summaries() - net::ERR_ABORTED
[reqfail] http://127.0.0.1:3270/api/v1/view-source/sui-demo - net::ERR_ABORTED
[error] Failed to load resource: the server responded with a status of 404 (Not Found)  (x several)
```

**Diagnosis**:
- `r.headerEl is not a function` + `r.bodyEl is not a function` — refs from some sui-* component (probably a sui-dialog or sui-panel that has `<bind:this on="headerEl">` and is reading the ref incorrectly in an effect)
- 500 on a daemon endpoint
- `ship.agents.summaries()` SSE aborts — maybe view dispose race
- 404s on view-source endpoints for non-existent views

## Concrete fix list for shell2-fundamentals

1. **Rail layout (mandatory)**
   - REMOVE text labels from rail items. Icons only. Tooltips (`title` attr) cover the label info.
   - REMOVE the in-rail "Show rail" toggle button (#10). Toggle moves to top-nav.
   - REMOVE rail items pointing at non-existent routes: `/sui-demo`, `/demos` (#4, #5).
   - Routes that exist: `/dashboard`, `/agents`, `/overview`, `/sessions`, `/workspaces`, `/chat`, `/skills`, `/mesh`, `/events`, `/git`, `/canvas`, `/specs`, `/collections`, `/pipelines`, `/pipelines-board`, `/theme-studio`, `/setup`, `/ship-ui-studio`. The rail should expose the principal ones, not point at vanished demos.

2. **Top-nav (mandatory)**
   - Reduce to 3 buttons + logo + theme picker trigger:
     - Logo (anchor to `/`)
     - **Menu button** → dispatches `sui-rail:toggle` to hide/show the rail
     - **CLI button** → opens the debug pane (sui-devtools)
     - Theme picker trigger (single button → opens picker; not 7 inline)
   - REMOVE the chat button, all 3 mystery buttons, the orphan "Close dialog" button, and the inline theme strip.

3. **Rail toggle states (mandatory)**
   - Two states only: **visible (icons only)** OR **hidden**. NOT three states (icon/full/hidden).
   - Persist state in localStorage.

4. **Project picker (mandatory)**
   - Must populate from `ship.workspaces.list()` (now an SSE op per #122).
   - Active workspace highlighted.

5. **Launcher button (mandatory)**
   - Dispatches `ship:launcher:open`. shell2/main.ts must handle it (open the launcher dialog).

6. **Plus button (mandatory)**
   - Dispatches `ship:create:open`. shell2 must handle it.

7. **Mount errors (mandatory)**
   - `/git` view's "Failed to resolve module specifier @ship/sui-…" — bare specifier in a view payload that's served outside a bundler. Likely the view emits `import { ... } from '@ship/sui-runtime'` raw. Daemon needs to rewrite to a runtime URL OR the view-payload needs the importmap inlined.
   - `r.headerEl/bodyEl is not a function` console error — find the component, fix the ref access pattern.

8. **Skills / mesh / events / settings hang on networkidle**
   - These have SSE subscriptions that never settle (network stays busy = networkidle never fires). May need to use `waitUntil: 'load'` in shell2's navigation OR the SSE source should reconnect-and-quiet faster.

9. **Right drawer (mandatory)**
   - Remove from index.html / chrome layout.

## Recommended verification pattern (gate)

After shell2-fundamentals lands, the agent MUST run this probe (or extension) and assert:
- ≥3 rail items, all pointing at routes that return 200 + non-empty content
- Top-nav has exactly the 3 buttons + logo + theme trigger
- Rail toggle hides/shows the rail element
- No console errors during normal navigation

A Playwright e2e is mandated in the spec. This probe artifact is the reference.

The probe script lives at `apps/shell2/probe.mjs` — runnable as `cd apps/shell2 && node probe.mjs`.
