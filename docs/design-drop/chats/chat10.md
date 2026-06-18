# Ship App Architecture

_Started 2026-05-26 11:45 UTC_

---

## User

help me mock something up. It's less of a shakeup for the UI itself as much as the interaction mechanism. So we have modeled some designs for the desktop/home screen and ship studio. I found a way to enable each project/space in ship to register it's own app launcher, icon, and to take ownership of a window. This means that a space truly owns it's theme, maybe even chrome (with a scaffolded framework available).  Ship the app now becomes the hub for the store, account management, system settings, system utilities and configuration (studio surface could still live in ship app). 

What sorts of progress can we make on the design here, and what needs to be adapted?

also as ever I want to clean this project, there are 500+ fucking files and they all get sent to my agents on handoff, it's bloody ridiculous. We burn so many extra tokens because of this too

## Assistant

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: read_file]_

Let me look at the existing desktop mock and studio mock to ground what we're adapting.

_[tool: read_file]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: questions_v2]_

Got it. Before I jump in, I want to make sure I'm building the right thing. Two threads to align on — the interaction concept itself, and what kind of artifact you want out of it.

Here's what I have grounded so far (correct me where I'm wrong):
- `design/mocks/home/desktop.html` — current "desktop with app icons + dock" — your most recent home thinking
- `design/mocks/shell/home/home.html` and `studio/shell.html` — the older inline-shell home + studio surface
- The shift: each **project/space** becomes the unit of ownership — it registers its own launcher icon, gets its own window/chrome/theme, with a scaffolded framework. **Ship.app itself** demotes from "the everything app" to **system hub** (store, account, system settings, utilities, config) + studio.

A few questions before I plan the mocks:

## User

not working

## Assistant

No problem — let me just ask inline.

To get the right thing built, the answers I most need:

1. **Window model.** When I'm in two spaces, do I see (a) two separate OS windows, (b) one Ship window with space-tabs/panes, (c) full-screen workspaces I swipe between, or (d) hybrid where the Ship hub is always running and spaces detach into their own windows?

2. **Ship.app's new role.** Strict "System Preferences" app (settings/store/account only)? Or still the launcher/hub pointing outward at spaces? Or background process with a menu-bar tray?

3. **Studio.** You said it could still live in Ship app — confirm? Or is Studio itself a space ("the space that builds spaces")?

4. **Chrome override depth.** Spaces can override (a) theme only, (b) theme + layout regions (rail, top bar, dock), or (c) everything inside the window border?

5. **Registration mechanism.** Manifest file in the space directory? A "register space" UI in Ship.app? A store/marketplace?

6. **Deliverable shape.** Side-by-side variations on a canvas? A walkthrough storyboard (install → launcher → themed window → back to hub)? Or two polished mocks (new Ship.app hub + one example themed space)?

7. **Example spaces to mock.** Give me 2–3 concrete projects with distinct vibes (e.g. "Brand Voice — writing-themed", "Rust Lane — terminal-themed"). Or I'll invent.

8. **Cleanup aggressiveness.** Of the 500+ files: aggressive (delete `_archive/`, `briefs/`, `surveys/`, `explorations/`, stale mocks) / moderate / conservative / plan-first?

Answer however many feel obvious and tell me to decide on the rest.

## User

1. a 2. still an important hub, like the studio ide will be in the hub, marketplace, account and global settings, activity monitoring, and yes there should be a launch surface but that is a convenience. Users would use ship hub to add a new space for example 3. may need reimagining, I was thinking studio is the editing surface for customization vectors but we have a choice for how much of this is shared between projects and the hub. 4. eventually C but I think A for now...maybe a preference for dock/nav layout/style 5. We allow creating a space, adding a name, icon etc then ship handles the host OS details across linux/macos/win automatically. 6. So on launch ship will guide users through creating their first space. We could also use a little disclaimer there. mocks alongside the creation flow most beneficial. Don't need to storyboard a host os launcher lol 7. Ship software project with our theme and views...thinking we keep the homescreen with widgets and second one can be a publishing app or something, does not matter. I am more interested in the component surface. 8. aggressive, need to keep sui but I have no fucking clue what most of this shit you put in here is

## Assistant

_[tool: update_todos]_

Clear. Plan:

**Cleanup first** (aggressive — keeps `sui/` runtime + upstream mirrors as source truth; nukes research dumps, old explorations, marketing landing, audits).

**Then build one canvas** with: Ship Hub variants, first-run + create-space flow, two space windows side-by-side (app-per-space), and the component surface (manifest + Studio customization editor). I'll lean into the last one since that's where you said your interest is.

Let me execute cleanup now, then start the canvas.

_[tool: delete_file]_

Cleanup — one shot:

_[tool: update_todos]_

_[tool: copy_starter_component]_

_[tool: copy_starter_component]_

_[tool: copy_starter_component]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: write_file]_

Now I'll build the canvas. Splitting into scene files to keep each manageable:

_[tool: write_file]_

_[tool: write_file]_

_[tool: write_file]_

_[tool: write_file]_

_[tool: read_file]_

_[tool: write_file]_

_[tool: write_file]_

_[tool: show_html]_

_[tool: done]_

_[tool: update_todos]_

_[tool: fork_verifier_agent]_

Open at `design/mocks/spaces/interaction-model.html`. Built as a single canvas with four sections you can pan/zoom + focus any artboard:

**Cleanup** — killed ~120 files: `design/_archive`, `design/briefs` (research dumps with duplicate sui-runtime), `design/audits`, `design/explorations`, `design/surveys`, `design/tools`, `design/workbench`, stale per-view mocks, marketing landing at root. Kept `sui/` + upstream mirrors (`chrome/`, `packs/`, `views/`) as source truth.

**Mocks (11 artboards):**
1. **Ship Hub** — sidebar (Spaces / Studio / Marketplace / Activity / Settings / Account) + spaces grid w/ "running now" strip and create-space card. Second variant: **Activity** stream across all spaces.
2. **Onboarding** — welcome w/ disclaimer about OS registration, then 3-step wizard (name+icon → theme+chrome prefs → "installed, here's where to find it").
3. **Spaces in the wild** — two OS windows side-by-side: Ship/dev (ship-dark, rail+dock+widgets) and Paperback/publishing (paper theme, top nav, no dock, editor view). Conveys app-per-space.
4. **Component surface** (the bit you cared about most) —
   - **Vectors reference**: A (theme tokens — *ready*), B (layout regions — *later*), C (custom chrome — *future*). Locks the RFC.
   - **`space.jsonc` manifest**: declarative model on the left, what it produces on the right (OS launcher registration, window chrome preview, default view).
   - **Studio · live editor**: 340px control panel (identity / appearance / chrome / OS integration / advanced disabled) + scaled-down live preview with an unsaved-diff chip.

**Open questions** I made calls on — push back if any are wrong:
- I drew Studio as a section inside the Hub (not a standalone space), and the editor opens *per-space* from there.
- Vectors B (dock/nav layout) is shown in the wizard step 2 and in the editor, but flagged "later" on the RFC card. If you want to ship those preferences day-one (you mentioned "maybe a preference for dock/nav layout/style") I should move B from "later" to "ready" and re-paint that card.
- No tweaks panel — let me know if you want one to A/B variants live (e.g. toggle hub layout, swap example spaces).

