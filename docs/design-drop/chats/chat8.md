# Desktop User Apps Launcher

_Started 2026-05-22 09:18 UTC_

---

## User

<system-info comment="Only acknowledge these if relevant">
Project title is now "Ship"
User is viewing file: design/mocks/home/desktop.html
Current date is now May 22, 2026
</system-info>

<default aesthetic>
If no references, art direction or design systems were provided, use this default professional modern, minimal aesthetic as a base. Declare your choice out loud so you stick to it. Guidance:
- Choose a type pairing from web-safe set or Google Fonts. Helvetica is a good choice. Avoid hard-to-read or overly stylized fonts. Use 1-3 fonts only.
- Foreground and background: choose a color tone (warm, cool, neutral, something in-between). Use subtly-toned whites and blacks; avoid saturations above 0.02 for whites.
- Accents: choose 0-2 additional accent colors using oklch. All accents should share same chroma and lightness; vary hue.
- NEVER write out an SVG yourself that's more complicated than a square, circle, diamond, etc.
- For imagery, never hand-draw SVGs; use subtly-striped SVG placeholders instead with monospace explainers for what should be dropped there (e.g. “product shot”)

CRITICAL: ignore default aesthetic entirely if given other aesthetic instructions like reference images, design systems or guidance, or if there are files in the project already.
</default aesthetic>

<pasted_text name="Pasted text (45 lines)">
 Mock: the Ship navigation shell — one IA to reconcile dock, rail, and Studio

  This is not a surface design. It's the frame everything else hangs in. We
  approve the navigation model, not the visuals — no surface gets built off this
  until the IA is signed.

  Context — three nav models exist in this repo and they conflict:
  - design/chrome/shell.html — flat left rail (Canvas · Collections · Artifacts ·
   Agents · Skills · Git · UI Studio · Settings)
  - design/mocks/home/desktop.html — OS-desktop dock of per-domain apps over a
  widget home
  - briefs/00-overview.md — Studio as a /studio/* sub-deck (rail + sections:
  agents · skills · components · themes · pipelines · projects · library)

  Produce ONE reconciled model.

  The mock must commit to an answer for each:
  1. Top-level mechanism — dock, rail, or both (e.g. desktop home shows the dock;
   inside an app a rail appears)? Draw the actual chosen chrome.
  2. Studio — one destination that opens a sub-deck of sections, or its sections
  flat at top level?
  3. Library — top-level "explore" destination or a Studio section? Show how a
  domain with both faces — components (browse installed packs vs the storybook
  editor) — is placed without duplicating it in two navs.
  4. Scope — how user ("Defaults"), project, and agent scope appear in nav, per
  audits/scope-model.html (no flat /permissions or /mcp; Defaults is a rail
  group).
  5. Distribution — does nav distinguish first-party (bundled) from
  reference/published (git, canvas) from default-apps — or is that invisible to
  the user?

  Form:
  - ONE screen: full chrome (status strip · dock/rail · content region) with a
  representative content area mounted — e.g. the agents list against
  agents.summaries.json — so the frame is shown in use, not empty.
  - Optional second state: the explore→edit transition (Library item → opens its
  editor).
  - Real <sui-*> primitives, tokens only (no raw px/hex), fixture-shaped data
  from crates/core/ship-platform/fixtures/data/*.json, light + dark.
  - A notes block (in-page or sidecar .md) listing the IA decisions it commits to
   + open questions.

  Out of scope: per-surface polish, new domains, anything not about the
  navigation frame.

</pasted_text>


<system-reminder>Auto-injected reminder (ignore if not relevant): do not recreate copyrighted or branded UI unless the user's email domain matches that company. Create original designs instead.</system-reminder>

## Assistant

_[tool: list_files]_

_[tool: list_files]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: grep]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: grep]_

_[tool: grep]_

_[tool: read_file]_

_[tool: grep]_

_[tool: grep]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: grep]_

_[tool: read_file]_

I have enough context. The brief is very specific — commit to ONE reconciled IA. Let me build it.

My design choices, declared up front:
1. **Mechanism: rail-only.** No dock. The desktop-dock home was a beautiful demo but a second nav model — kill it.
2. **Studio: zone, not destination.** No `/studio/*` indirection — sections live flat in the rail under a "STUDIO" group label. The deck-within-a-deck nesting is what makes the IA feel confused.
3. **Library: top-level "Explore" group.** It's the install/discover surface. Components has two faces — *editor* lives at Studio › Components (configure), *browse* is Library's Components facet. Opening a library item deep-links into the domain surface.
4. **Scope: rail groups.** WORK · STUDIO · DEFAULTS · EXPLORE. Per scope-model.html: Defaults is its own top group; project sits in Studio (Project Settings); agent scope is inline in agent detail. No flat `/permissions` or `/mcp`.
5. **Distribution: source badge on cards, not nav.** First-party/project/user/published as metadata only.

Let me look at icons + look at one more reference before building.

_[tool: write_file]_

Building the reconciliation mock at `design/mocks/shell/nav-frame.html`. One screen, IA-committed, with the agents list mounted in the content region against fixture-shaped data, plus a second state for Library→editor.

_[tool: done]_

_[tool: fork_verifier_agent]_

Built `design/mocks/shell/nav-frame.html` — the single reconciled frame.

**The five IA decisions committed in this mock:**
1. **One rail, no dock.** Desktop dock retired (`mocks/home/desktop.html` becomes superseded).
2. **Studio is a zone, not a destination** — its sections sit flat in the rail under a `STUDIO` group header. No `/studio/*` indirection.
3. **Library = Explore** (its own rail group). Components has two faces resolved: *browse installed packs* = Library; *storybook editor* = Studio › Components. Library items deep-link into the domain editor, never duplicate it (shown in State 02).
4. **Defaults is a top-level rail group** per `audits/scope-model.html` — user scope. Project scope → inside Studio › Project settings. Agent scope → inline in agent detail. No flat `/permissions` or `/mcp`.
5. **Distribution = card pill, not nav.** `bundled` · `project` · `user` · `library` — same vocabulary everywhere.

The frame is shown in use (agents list mounted against `ship.agents.summaries()`-shaped data), and group headers carry their scope path (`this session` · `.ship/` · `~/.ship/` · `remote`) so provenance is legible from the chrome alone. Light + dark via the toggle top-right. Five open questions noted at the bottom (pipelines placement, components sub-rail, Defaults rename, install landing, strip vs bar).

## User

no this is shit. I want desktop as the launcher for USER apps. then studio is a configuration surface that ship owns. When a user runs their apps there is no sidebar or studio chrome, that's the point.

## Assistant

_[tool: copy_files]_

Got it — I had the model upside down. Resetting:

- **Desktop = user-app launcher.** The dock-and-widgets home is right. It's where USER apps live.
- **Studio = Ship's config surface.** Rail + sections. Ship owns this; users don't run their apps here.
- **User apps run full-bleed.** No rail, no dock, no Ship chrome.

So three modes, not one IA. Let me preserve v1 (the wrong answer) and rebuild.

_[tool: write_file]_

_[tool: done]_

_[tool: fork_verifier_agent]_

Reset and rebuilt around the right mental model. v1 is preserved at `nav-frame-v1.html` as the rejected approach.

**The model now: three modes, three chromes.**

1. **Desktop** (Mode 01) — dock + widget home. The dock launches USER apps (Canvas, Sessions, Artifacts, Git, etc. + anything the user builds). Studio and Settings sit after a divider as escapes, not "apps".
2. **User app** (Mode 02) — full-bleed. Zero Ship chrome. The only Ship UI is a 24px home dot top-left; `⌘.` returns to Desktop, `⌘,` opens Studio.
3. **Studio** (Mode 03) — Ship's config surface, its own rail with three groups: **Configure** (`.ship/` — agents · skills · components · themes · pipelines · project), **Defaults** (`~/.ship/` — permissions · MCP · harnesses), **Explore** (remote — library · publishing). Has a "STUDIO" badge in the top bar + Close. Library item → deep-links into the Configure editor (shown as the transition state).

Source pills (`bundled`/`project`/`user`/`library`) carry distribution as metadata across all three modes.

Open Qs at the bottom flag: how Studio enters/exits (`⌘,` modal vs replace), whether first-party views are dock apps, whether the home dot is required or pure keystroke, where pipeline runs live, and whether the dock is user-customizable.

## User

this is much closer, but I wonder if full bleed user apps is too much. At present I maintain a small top and bottom rail. apps can request space on these too. 

a unified command pallete cmdk is extremely load bearing for the OS. 

"cmd/crtl"+ "k" to open but "_"+"_" also opens the modal with agent chat from anywhere. 

It's nice to keep that search bar always visible. Then I have a bottom drawer with a terminal and dev tools, always accessible. 

The fact that ship is workspace/project oriented makes the persistent rail justified in my view but the topology is admittedly arbitrary. We could just as easily to studio + home + workspaces with nested views. That's something to consider and I think some invariants are necessary. Home is per user but I wonder if there should be project home too. Ugh its complicated!

## Assistant

_[tool: questions_v2]_

Yeah this is exactly the load-bearing topology question and it's worth nailing before another v3. Let me ask a focused round before I build.

## User

not loading

## Assistant

Sorry — let me just ask in prose. Here are the load-bearing questions I need answered to commit on v3:

**1. Persistent chrome — anything else besides top + bottom strip?**
You mentioned strips, not a left rail. Are strips the *only* invariant, or is there also a thin always-there left rail (for workspace switcher + Home/Studio/Workspaces nav)?

**2. Topology — what are the top-level destinations?**
Right now Home, Studio, and Workspaces all exist. Which model:
- (a) Home (user) contains Workspaces; each workspace has its own home + nested views. Studio = global modal.
- (b) Home + Project Home + Studio as three peers.
- (c) Just Project Home + Studio (workspace is required; user-level is auth/billing).
- (d) Spaces deck — every workspace is a tab.

**3. User Home vs Project Home — both, or one?**
If both: User Home = across-projects (recent workspaces, global inbox, account); Project Home = the dock+widgets we mocked.

**4. Top strip contents** (left→right) — pick what's in:
Ship mark · workspace switcher · breadcrumb · cmdk search · run/agent status pills · daemon health · git branch · app-injected slot · Studio/settings · account · clock?

**5. Bottom strip contents** — pick what's in:
Terminal toggle · DevTools toggle · active-run indicator · event sparkline · app-injected slot · cursor position · theme toggle · connection · notifications?

**6. The `__` keystroke for agent chat — spell it out.** (Space-space? `//`? `⌘J`? `⌘I`? Other?)

**7. Cmdk scope** — search only, search+actions, search+actions+workspace-switch, or search+actions+agent-chat?

**8. "Apps request space on strips" — concretely what?** Tray icons only, tray + live status slot, full mini-widgets (VS Code style segments), or full toolbar injection?

**9. Studio entry — modal, replace, or destination?**

**10. When a user app runs:** does it own everything between top + bottom strips (strips persist), or can apps go true full-bleed and hide strips too?

**11. Workspace switching surface** — top-strip dropdown, cmdk only, dedicated Spaces view, or all of the above?

Even partial answers help — anything you commit to, I'll pin as an invariant and design the rest around it.

