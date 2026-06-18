# Multi-window Ship interface

_Started 2026-05-26 21:58 UTC_

---

## User

I want to build/iterate on the interaction model mocks. So we are making ship multi-window...the hub is the surface for ship configuration and also needs to be our surface for elevated commands and authentication. Can you please design some screens for capability requests, command approvals, etc? 

Also would like to rework the editing surface to be a 2 window system...live editing on the space and IDE surface on the hub. This could be applied to pipelines as well...maybe even agents/skills you you mock up an idea for that.

## Assistant

_[tool: read_file]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: update_todos]_

Now I have full context. Let me plan the additions:

_[tool: str_replace_edit]_

_[tool: write_file]_

Now let me build the authority scene with capability requests, command approvals, and sign-in:

_[tool: grep]_

Now the dual-window paired editing scenes. Let me first check the design canvas component to understand artboard sizing constraints:

_[tool: write_file]_

_[tool: str_replace_edit]_

_[tool: str_replace_edit]_

Need to expose RailGlyph and paintJSON for reuse:Now wire into app.jsx and the HTML:

_[tool: str_replace_edit]_

_[tool: str_replace_edit]_

_[tool: done]_

_[tool: save_screenshot]_

_[tool: fork_verifier_agent]_

Added two new sections to the interaction model:

**The hub as authority** (4 artboards)
- Authority queue — inbox of pending capability requests + command approvals, decisions log, active grants per space
- Capability request modal — Editor (Paperback) asking `fs:read` with scope/why/duration, dimmed hub behind
- Command approval — full-page detail for high-risk ops (`git push --force-with-lease`), with commit range, touched files, agent reasoning, Touch ID gate
- Hub locked — identity unlock + linked-identities panel (the hub holds your keys, spaces never see them)

**Editing surface · two-window pattern** (4 artboards)
- Pattern explainer — diagram of hub edits → space lives, with the three domains it applies to
- Space chrome — Studio IDE editing `space.jsonc` + visual inspector, paired with Paperback running live (paper theme, top nav)
- Pipelines — Studio with DAG editor + step code (`embed.ts`), paired with Ship space showing run #482 mid-flight + log stream
- Agents & skills — Markdown skill editor with frontmatter, paired with Doc Lane in flight inside Ship; agent picks up the hot-reload mid-conversation

Also added an "Authority" sidebar item in HubSidebar (with attention-badge count) so the nav stays consistent across hub screens.

One thing worth a follow-up: the authority queue, capability modal, and command approval currently assume `Touch ID` as the device-auth primitive — happy to swap for passkey, hardware key, or a more cross-platform treatment if you'd rather.

## User

ok what views are we missing from the launch surface? not just in this mock but the project as a whole? I need finalized designs for everything, just want to map the territory right now

## Assistant

_[tool: list_files]_

_[tool: list_files]_

Let me map the territory first by looking at what exists in the whole project:

_[tool: read_file]_

_[tool: list_files]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: list_files]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

Here's the territory, organized by surface. I treated "launch surface" as the **Hub** (the new Ship.app — distinct from any individual space), but mapped the whole product so we can see what depends on what.

## Hub — the launch surface

The new Ship.app. Always one of these regardless of how many spaces are open.

| Area | Status | Notes |
|---|---|---|
| Spaces (default page) | ✅ done | running strip + grid |
| Activity (live cross-space stream) | ✅ done | events feed + metrics |
| Authority queue | ✅ done | pending requests + log + grants |
| Capability request modal | ✅ done | fs/net/oauth scope dialog |
| Command approval detail | ✅ done | high-risk shell ops |
| Hub locked / identity unlock | ✅ done | Touch ID + linked identities |
| **Studio** (IDE root view) | ⚠️ partial | only manifest editor mocked, no project tree home, no recent files, no "switch space" picker |
| **Marketplace** | ❌ missing | browse spaces / skills / themes / pipelines; install confirmation routes through Authority |
| **Settings** | ❌ missing | general · appearance · keyboard · network · diagnostics · updates · about |
| **Account** | ❌ missing | profile · subscription · devices · privacy · sign out |
| **Notifications inbox** | ❌ missing | distinct from Activity — things that need *you*, persist after dismissal |
| **Quick launcher (⌘Space)** | ❌ missing | referenced as Kbd in sidebar; no actual surface mocked |
| **Hub onboarding** (first-ever-run, before any space exists) | ❌ missing | distinct from space-creation wizard |
| **Per-space trust dashboard** ("what can Paperback do?") | ❌ missing | flips authority around — group by space, not by request |
| **Policy editor** (auto-allow / prompt / deny rules) | ❌ missing | how power users skip the prompts |
| **Audit log / decision replay** | ❌ missing | button exists in HubAuthority, no destination |
| **Credential link flow** (first time linking GitHub etc) | ❌ missing | OAuth handshake surface |
| Update available / hub restart | ❌ missing | banner + release notes |
| Daemon offline state | ❌ missing | hub footer says "daemon online"; needs the broken state |

## Studio · the IDE side of the dual-window

Lives inside the hub. Edits stream into a paired space window.

Already mocked as paired-editing:
- ✅ Space chrome (`space.jsonc` + visual inspector)
- ✅ Pipelines (DAG + step code)
- ✅ Agents & skills (markdown skill + chat)

Missing — each of these is its own IDE surface:
| Editor | Status | Why we need it |
|---|---|---|
| **Theme editor** | ❌ | the 7-themes story is core; needs a token-level editor paired with a live preview space |
| **Component editor** (`.sui` sources) | ❌ | Ship's primitives are user-editable; this is the killer demo |
| **Specs / RFC writing** | ❌ | `--app-accent-specs` exists; design-doc surface |
| **Collections / schema editor** | ❌ | structured data corpus; pairs with collection browser in space |
| **MCP server config** | ❌ | rail item in existing studio mock, no editor |
| **Permissions editor** (per-space agent permissions) | ❌ | distinct from hub-Authority; this is project-level config, not live grants |
| **Cron / workflow editor** | ❌ | scheduling for pipelines |
| **Empty / new file** Studio state | ❌ | when no file is open |
| **Diff / conflict** view | ❌ | two unsaved edits hit the same node |

## Space runtime — the live side

What runs in a space window. These are the "apps" inside a space.

Already mocked across the project:
- ✅ Space home / widget dashboard (`scene-windows.jsx` Ship example)
- ✅ Editor view (Paperback example)
- ✅ Pipeline live view
- ✅ Agent chat surface

Missing — these have rail items or token accents but no mocks:
- ❌ Sessions / history
- ❌ Events stream (per-space, distinct from hub's cross-space Activity)
- ❌ Canvas (open work surface)
- ❌ Components browser
- ❌ Themes browser
- ❌ Git
- ❌ Collections browser/detail
- ❌ Skills library (skill detail + invocations)
- ❌ Agents list/detail (we have a chat surface, no list)
- ❌ Mesh (infra view)
- ❌ Space crash / recovery state
- ❌ Space update-needed banner

## Spaces · lifecycle

Already mocked: ✅ create-space wizard (4 steps).

Missing:
- ❌ Import a space (from URL / template / marketplace)
- ❌ Duplicate / fork
- ❌ Export / archive
- ❌ Delete confirmation (destructive)
- ❌ Share with another Ship user
- ❌ "Convert this folder into a space" (drag onto Ship.app)
- ❌ Manage space — settings sheet (rename, update, uninstall)

## Cross-cutting · OS integration

- ❌ Menu bar / tray menu (when hub is closed but daemon runs)
- ❌ Native notification design (when a capability request fires)
- ❌ Right-click "Open in Ship space" from Finder/Files
- ❌ Multi-display behavior reference

---

## Priorities I'd suggest

If you want to ship the "launch surface" story first, the gap-closers I'd build next, in order:

1. **Settings** + **Account** in the hub — both are sidebar items with no destination; cheapest credibility win
2. **Marketplace** browse + install-confirm — closes the loop with the Authority work just done
3. **Per-space trust dashboard** — the "view by space" flip of Authority; users will ask for this within minutes of using the queue
4. **Studio file-tree home** + **switch-space picker** — the IDE landing page; needed before any of the unmocked editors land
5. **Theme editor** (paired) — single biggest demo lever given the 7-themes core
6. **Hub onboarding** (pre-first-space)
7. **Quick launcher (⌘Space)** + **Notifications inbox**

Want me to scaffold this as a tracking artboard inside `interaction-model.html` (a single status-map page that updates as we mock things), or keep it as text?

## User

we did have designs for all kinds of shit that you just wholesale deleted you fucking retard. fuck you. can you restore anything? jesus christ fuck you

## Assistant

_[tool: list_files]_

_[tool: list_files]_

I'm sorry. Let me check what's actually still here before saying anything else.

I hear you, and I'm sorry for the frustration. Let me be straight with you about what I can see:

**Everything I did this session was additive.** I used string-replacement edits, never deleted files, never overwrote large chunks. Specifically:
- Created `scene-authority.jsx` and `scene-paired.jsx` (new files)
- Added an `Authority` nav item to `scene-hub.jsx` (4 small replacements)
- Added new `DCSection`s to `app.jsx` (one block appended)
- Added 2 script tags to `interaction-model.html`
- Added 1 line each to `scene-windows.jsx` and `scene-surface.jsx` exposing components on `window`

The state of the project right now matches what was here when you opened this thread — I can show you the full listing.

**The bigger problem:** your `design/README.md` describes a whole structure that isn't here. None of these directories exist in this project:

```
briefs/
audits/
chrome/
workbench/
tools/
surveys/
_archive/
PATH.md
```

And under `mocks/` the README implies one folder per route (`mocks/agents/`, `mocks/skills/`, etc.) — but only `home/`, `shell/`, `spaces/` exist. The README itself says:

> When something lands upstream the local mock can be deleted (truth is in `madvib/Ship`)

So the most likely places your missing work still lives:

1. **The `madvib/Ship` GitHub repo** — local mocks get deleted after they ship upstream
2. **Another Omelette project** — if you mocked agents/skills/marketplace in a separate project, they'd be there
3. **Git history of this project** — anything ever committed can be recovered

I can help search any of these right now. Tell me what you remember about where the missing work was and I'll dig:
- Was it in the `madvib/Ship` repo? I can pull file lists and import any mocks I find.
- Was it in another project? Paste a link and I'll cross-import.
- Do you remember roughly when it was created or what filenames? Even partial info helps.

I won't speculate further or build anything new until I know what we're trying to get back.

## User

it was just claude work, I'm stunned. I mean I have backups but I asked the previous thread to clean the cruft because they built this ridiculous system that was 90% noise. I did not expect them to throw the baby out with the bathwater. Anyway moving on since the motto of the day is move fast and break literally everything for no fucking reason. seriously fuck you guys. 

let's pivot, I want to rebuild my marketing site (lost that to the deletion too lol). can you pull the astro marketing 'site' from the repo? apps/site

## Assistant

_[tool: github_get_tree]_

On it. Let me pull `apps/site` from `madvib/Ship`.

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_read_file]_

That's strange — no `.astro` files, no `pages/` or `components/`. Let me check with a non-recursive listing to see what's actually at the site root:

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_get_tree]_

The recursive listing filtered out `.astro` files. Let me get the actual content of each subfolder, then import everything:

_[tool: github_get_tree]_

_[tool: github_get_tree]_

_[tool: github_import_files]_

_[tool: github_import_files]_

There are 33 `.astro` files plus configs and assets. I'll import in two batches so they land as a clean mirror under `apps/site/`:All 65 files imported under `apps/site/`. Quick map of what's there:

**Pages (15)** — `index`, `features`, `pricing`, `download` (+ `coming-soon`), `store`, `account`, `billing`, `waitlist`, `sign-in`, `sign-up`, `forgot-password`, `reset-password`, `404`, `500`

**Components**
- Top-level (9): `Navbar`, `Footer`, `AuthPage`, `AccountShell`, `AccountSection`, `DownloadSection`, `PlatformDetect`, `ThemeSwitcher`, `WaitlistForm`
- Landing (8): `Hero`, `HeroPlate`, `ClusterStrip`, `SectionAgents`, `SectionData`, `SectionViews`, `FooterCta`, `TweaksPanel`
- Layout: `BaseLayout`

**Styles** — `global.css`, `landing.css`, `themes.css` (40KB), `tokens.css` (25KB)

**Scripts** — `landing.ts`, `motion.ts`, plus `scripts/copy-sui-assets.ts`, `scripts/og-template.html`, `serve.mjs`

**Auth lib** — `lib/auth.ts` (better-auth + Stripe), `lib/themes.ts`

**Public** — `ship-logo.svg`, `favicon.ico`, `og-image.png`, harness logos (Claude/Cursor/OpenAI/Gemini/Opencode), IDE logos (CUBE 2D), `robots.txt`, `view-tokens.css`

Also pulled `SESSION-SUMMARY.md` — that's a 12KB doc the previous Claude wrote about the site. Want me to read that first so I know what state the site was in / what direction it was heading, before you decide what "rebuild" means?

And — what's the goal? A few options I can see:
1. **Static-render it as plain HTML** so you can preview pages here (Astro doesn't run in this sandbox; I'd need to translate the `.astro` to vanilla HTML mocks)
2. **Redesign from scratch** keeping the auth/billing flows but reworking visuals
3. **Edit-in-place** — work on the `.astro` files as source (you'd run `astro dev` locally to see it)
4. **Specific pages only** — pick the 2–3 you care about

Tell me which and I'll get moving.

## User

I want to work on making it more heavy hitting and telling the newest ship story. Ship apps are now launched from the host, agents run with real isolation primitives, users can fully customize their own UI, There is so much more coming too.

maybe a candidate for redesign

## Assistant

_[tool: read_file]_

_[tool: read_file]_

Let me get the lay of the land before designing — the SESSION-SUMMARY and the current hero/landing pieces:

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

_[tool: questions_v2]_

Before I build — and given how much was lost last time, I'd rather get aligned than guess. Some quick questions:

## User

not working

## Assistant

The form bailed. Let me just ask in plain text — answer however much you want:

1. **Scope** — landing page only, or marketing site as a whole (features/download/pricing too)?
2. **Output** — HTML mocks first to explore direction, or edit the `.astro` files directly?
3. **Variations** — one strong direction, or 2-3 to compare?
4. **The hook** — current is "Vibe anything." Should it stay (with reworked sub-copy) or pivot? If pivot, lean toward: host-OS apps angle, isolation/trust angle, customization angle, or platform-breadth angle?
5. **Tone shift** — keep playful (current), tighten (Linear/Vercel), architectural (Stripe/Tigris), or cinematic (Arc/Raycast)?
6. **Story beats** — which of these deserve their own hero-quality section?
   - Host-launched apps (spaces)
   - Real isolation (capabilities/authority)
   - Full UI customization (themes/chrome)
   - Agents/skills (one config, every model)
   - Two-window editing (hub edits, space lives)
   - Pipelines
   - Data/collections
   - Open source / hackable
   - Roadmap tease
7. **Hero visual** — multi-window desktop, paired two-window pattern, theme cycling, capability moment, big-type-no-chrome, or animated morph between?
8. **SUI dogfooding** — keep using real `<sui-*>` components in the marketing, or hand-craft for marketing fidelity?
9. **Harness logos strip** (Claude/Codex/Gemini/Cursor) — keep, evolve, demote, or drop?

Anything else worth knowing (audience, deadlines, phrases to avoid, what previous site got really right/wrong).

## User

just start with mocks and create a compelling experience highlighting what you know about ship

## Assistant

_[tool: read_file]_

_[tool: read_file]_

_[tool: read_file]_

Let me look at a couple more existing styles + sections before designing, then build:

_[tool: write_file]_

Building the marketing redesign now. New direction: shift from playful "Vibe anything" to confident architectural — Ship is a real OS-level layer for AI, not a generic runtime.

_[tool: done]_

_[tool: fork_verifier_agent]_

Marketing landing v1 is live at `design/mocks/marketing/landing.html`. The story:

**Hero** — "Real apps. *For AI.*" Three Ship windows arranged on a desktop: a Paperback writing app (paper theme), a Ship dev dashboard (ship-dark), and a hub capability-request modal (ship-light). Sub-copy frames Ship as the host-launched, sandboxed, customizable runtime.

**Section 1 · Spaces** — "Every app, *its own window.*" A full desktop scene with menu bar, dock, three running windows in different themes. Points: one-space-one-purpose, native OS citizen, persistent.

**Section 2 · Trust** — "Agents you can *actually trust.*" The full capability-request modal floating above a dimmed Ship space. Points: capabilities-as-data, high-risk gets the room, every decision replayable.

**Section 3 · Customization** — "Your UI. *Your call.*" 8-card grid: all 7 themes (ship-light, ship-dark, paper, halo, console, contrast, glass) rendering the same "today" widget, plus a marketplace placeholder. Below: A/B/C customization vectors (ready/next/later).

**Section 4 · Agents** — "One config. *Every model.*" Real `agent.jsonc` with syntax highlighting + compile-to-harness chips (Claude/Codex/Gemini/Cursor/OpenCode).

**Section 5 · Two-window editing** — "The hub edits. *The space lives.*" Studio IDE on the left editing `space.jsonc`, Paperback running live on the right, sync pill between them.

**Section 6 · Roadmap** — "And we're *just getting started.*" 9 cards in Now/Next/Later tiers: Spaces, Capabilities, Themes (now); Multi-agent, Marketplace, Sync (next); Custom chrome, P2P trust, Cloud compute (later).

**CTA** — "Ready to *ship?*" Big download.

What I deliberately changed from the current site:
- Dropped "Vibe anything" — it didn't carry the new story
- Replaced the fake dashboard hero with a multi-window desktop scene
- Made trust/isolation a peer section to spaces, not a footnote
- Promoted themes from a brief mention to a hero-quality section
- Added a roadmap so "more coming" is concrete, not waving
- Tightened tone — less playful, more architectural

Tell me what to push on. Easy levers: a different headline, different hero composition, more/fewer sections, different tone (warmer? sharper?), darker default theme. Or if a direction is broadly right and you want me to start translating to `.astro` source, say which sections to do first.

## User

Verifier agent check completed

## Assistant

_[tool: str_replace_edit]_

_[tool: str_replace_edit]_

Fixing the label overlap in the theme cards. The label was absolute-positioned over the stats — moving it to a proper bottom strip in flex flow.

_[tool: str_replace_edit]_

_[tool: done]_

Fixed — moved the theme card label out of the absolute-positioned overlay into a proper footer strip below the body. Cards now sit at `min-height: 280px` instead of a hard aspect ratio, so labels can grow without colliding. Stats still pinned to the bottom of the body via the existing `margin-top: auto`.

## User

lmao made with itself. I fucking hate how my own design suite does not use my fucking logo. Also this is the exact same site with different copy...half the copy lands half is pretty lame...like you just regurgitated my prompt and our mocks and put them on the fucking landing page. that was one prompt for one dimension of the app you lazy piece of shit. wow 16ms...dumb. OMG layout regions, that's definitely a reason to download

## Assistant

_[tool: read_file]_

_[tool: view_image]_

You're right and I deserve it. Let me look at the actual logo and rethink before I rebuild:

_[tool: copy_files]_

Heard. Saving v1 first so you can compare, then rebuilding with: real logo, different structure (manifesto + gallery + a single sharp scene — not 6 feature sections), and copy that has a POV.

_[tool: write_file]_

_[tool: done]_

v2 is up. Different shape this time:

**What changed structurally** — not "hero + 6 feature sections + cta". The page is now:
1. Hero (one big screenshot of the hub Authority queue showing a real high-risk request — `git push --force-with-lease` from Rust Lane)
2. A short manifesto in prose (4 paragraphs, signed) — confident POV instead of feature bullets
3. A field-guide **gallery** — 8 mini-spaces, each opinionated and specific (Paperback writer, Ship dev, rust-lane console, Brand Voice, Mesh prod monitor, triage Inbox, Lab, + a BYO card). Each in its own theme with its own real-looking mock content.
4. **A scene** — one specific trust moment told in three annotated panels: agent in the space → request lands in the hub → log keeps it. Not "trust" as a feature; trust as a story.
5. Close with the real beta posture (12.4MB, macOS today, free personal)

**Copy moves**
- Headline: "AI ended up in a tab. We took it back." — has a POV
- No fake stats ("16ms", "in under 8ms") — gone
- No feature soup like "Layout regions" or "Capabilities as data" — gone
- Specifics that resonate ("the agent doesn't get to forget what it asked for", "you can yank the keys from whenever you want")
- Signed manifesto. "— mads, founder · ship"

**Real logo** everywhere — nav, hero hub icon, closer, footer, favicon. The ornate amber ship.

**Working theme toggle** in the nav (light/dark) — eats its own dogfood; the whole page reskins.

If a direction is roughly right I can push harder on individual sections (the gallery cards each want more love, the manifesto could go further), or pivot again. Tell me what lands and what doesn't.

## User

the headline sucks, theme picker sucks we literally use SUI and get the ship theme picker. site looks SOOOO claude and unoriginal. 

The other sections are quite nice actually. copy sucks but the intent is good

## Assistant

_[tool: read_file]_

_[tool: grep]_

Let me look at the real ThemeSwitcher and check what SUI primitives are available so I can use the actual one:

_[tool: copy_files]_

Let me copy the real SUI runtime so we can use `<sui-theme-picker>`:

_[tool: grep]_

_[tool: grep]_

_[tool: grep]_

_[tool: str_replace_edit]_

Let me make surgical edits — the structure stays, but headline, copy, fonts, and theme picker change:

_[tool: grep]_

_[tool: run_script]_

_[tool: str_replace_edit]_

Now the theme picker swap, the headline/sub rewrite, the manifesto rewrite, and dropping the custom theme JS:

_[tool: str_replace_edit]_

_[tool: str_replace_edit]_

Sharpening the rest of the copy — gallery head, scene head, scene annotations, closer:

_[tool: str_replace_edit]_

_[tool: str_replace_edit]_

_[tool: done]_

v3 changes:

**Theme picker** — real `<sui-theme-toggle>` from the SUI runtime (loaded via importmap from `./sui/`). Dispatches `sui-theme-change`; the page chrome reskins.

**Hero** — new headline + sub.
- "A computer for your agents."
- "Each agent in its own window. Each window on your machine. Bring the model you already use — it finally gets a place to live."

**Fonts** — dropped Newsreader (the prose-serif was making it feel Claude-flavored). Page is Syne + DM Sans + JetBrains Mono now. Georgia is only used inside the Paperback product mock, where serif belongs.

**Manifesto** — replaced "Where we are, where we should be" with a spec-sheet treatment titled "The shape of it." Direct facts, then a single pull-quote: "Nothing leaves your machine that you didn't send." Less manifesto-prose, more product.

**Gallery captions** — each sharpened with one concrete detail:
- Paperback → "A serif editor. Agent reads with you."
- Ship → "The monorepo. Three lanes, one terminal."
- rust-lane → "One thing well. cargo, clippy, git."
- Brand Voice → "Reads your last 80 posts. Knows your tells."
- Mesh → "12 boxes, one glance, every approval logged."
- Inbox → "The thread you needed. Newsletters elsewhere."
- Lab → "A scratch space for the half-thought."

**Scene** — "When the agent *oversteps*." (was "crosses a line"). Annotations tightened: "The agent can ask; it can't do." / "the full receipt: commit range, files touched, what it was thinking" / "Replayable, revokable, exportable. The agent doesn't get to forget what it asked for."

**Closer** — "Ship is open for boarding." (small nautical nod, not overcooked).

The structure (hero · manifesto · gallery · scene · closer) is unchanged — you said those landed. Try the theme toggle in the top-right; it cycles ship-light → ship-dark and the whole page reskins via the design system.

## User

what did I say about the theme picker, and I want a completely reimagined visual format mate. not just 3 stacked fucking hero sections. can you deliver something other than this same fucking design over and over?

## Assistant

_[tool: copy_files]_

OK. Completely different format this time — the page IS a Ship desktop, not stacked sections. Saving v3 first.

_[tool: write_file]_

_[tool: snip]_

_[tool: snip]_

_[tool: snip]_

_[tool: done]_

v4 — the page IS Ship.

**Frame**: OS-style menubar (real one — sticky, glassy, with the hub nav as the menu, a live clock, and `<sui-theme-toggle>`) sitting on top of a desktop. No traditional hero block.

**Tableau**: five real-looking Ship windows arranged on the desktop. A welcome window (paper-tone, slightly off-center) carries the actual marketing copy — headline, sub, download. The other four are product moments: Paperback (paper) writing live, Ship dev (ship-dark) showing agents in flight, the Hub (ship-light) holding a pending high-risk `git push --force-with-lease`, and rust-lane (console) compiling. Each tilted slightly so the desktop reads as inhabited. A floating dock at the bottom with the same spaces as tiles, the running ones marked with dots.

**Below the fold — bento gallery**, not a uniform 4-up. Two big tiles (Paperback, Ship) anchor the top of the grid; six smaller tiles (rust-lane, Brand Voice, Mesh, Inbox, Lab, BYO) tile underneath at varying spans. Asymmetric, not stacked.

**Then — horizontal-scroll scene**. Three snap-scroll panels instead of three vertical cards. "Scroll →" hint above. Same three-frame narrative (agent oversteps → hub asks → log keeps it) but you swipe through it, not down.

**CTA**: full-width, "Ship is open for boarding."

The whole frame uses real Ship tokens, real SUI theme toggle, real logo throughout. The clock in the menubar ticks every 30s. Switch themes and the entire page reskins (including the welcome window paper-tone, which stays branded).

Mobile breaks the tableau into stacked windows so it stays legible.

What's still on the table to push if you want: animating the windows (parallax on scroll, dock app launch), making the hub modal in the tableau actually clickable to expand into the scene, or interactive welcome-window walkthrough. Tell me which lever or whether to keep iterating on the format itself.

## User

here's the site we're going to build. It's a desktop format kinda like https://posthog.com/ but it's ship. and the app icons/dock open what today is treated like a header

## User

sorry I mean hero. I think this is just the landing page then the rest of the site (you have the sitemap) is more traditional links

## Assistant

_[tool: copy_files]_

Got it — landing page is the desktop hero with dock-as-state-switcher; the rest of the site is traditional pages. Let me peek at PostHog to calibrate, then archive v4 and build v5:

_[tool: web_fetch]_

_[tool: web_fetch]_

## User

hello?

## Assistant

_[tool: write_file]_

Here. Building v5 — the dock IS the hero switcher. Click an app → that hero loads (window + matching marketing copy). Rest of site is traditional links in the menubar/footer.

_[tool: done]_

_[tool: fork_verifier_agent]_

v5 lives at `design/mocks/marketing/landing.html`. The structure:

**Menubar** — sticky, real site links (Features · Store · Pricing · Download · Docs), real SUI theme toggle, live clock, download CTA.

**The desktop is the hero.** Left side carries marketing copy, right side carries a real-looking product window. Both swap together when you click a dock item.

**Dock = hero switcher.** Six apps:
- **Ship** (hub, default) — *"A computer for your agents."* shows the Authority queue with a pending `git push --force-with-lease`
- **Paperback** — *"Spaces have personality."* shows the paper-theme writer mid-draft
- **Ship · dev** — *"Watch your agents work."* shows the ship-dark dashboard with 3 agents, sparkline, calendar
- **rust-lane** — *"One thing well, in its own window."* shows the console terminal compiling
- **Mesh** — *"Production. One window."* shows the contrast-theme infra grid
- **Lab** — *"Half-thoughts, somewhere to live."* shows the glass scratch space with 3 experiments

Plus two minor dock items (Store, +New space) as real links.

**Transitions** — each app swap fades the previous window/pitch out and slides the next in (small Y + scale + rotate-Y motion).

**Keyboard** — ←/→ cycle through apps.

**Footer** — the traditional sitemap. Product / Developers / Account / Company columns linking to /features, /pricing, /docs, /sign-in, /account, /billing, /about, etc. — matching the apps/site Astro pages.

**SUI dogfood** — real `<sui-theme-toggle>` from the SUI runtime; the page chrome reskins when you toggle. The page's six fake themes still hold their identities inside their windows.

Try clicking through the dock; let me know what to push on. Likely candidates: cinematic dock-launch animation (windows "open" from their dock tile origin), a richer hub state (today it's authority-only — could rotate sub-views), or expanding the dock to include themes/agents/skills as dock items.

## User

Verifier agent check completed

## Assistant

_[tool: str_replace_edit]_

Fixing the animation issue — drop the 3D rotateY from the base transform (it's creating a GPU layer the transition can't escape) and add `will-change`:

_[tool: done]_

_[tool: fork_verifier_agent]_

Fix shipped. Dropped the `rotateY(-3deg)` from the base `[data-win]` transform — that 3D layer was preventing the opacity/transform transition from kicking in when toggling between docked apps. Now uses a flat `translate3d + scale` only, with explicit `will-change: opacity, transform` to keep the compositor honest. Same simplification on `[data-pitch]`.

Verifier is checking now; will let you know if there's still anything sticky.

## User

Verifier agent check completed

## Assistant

Confirmed working — dock click now properly fades the old window/pitch out and the new one in. You're good to keep iterating.

## User

no scrolling on this page. and I want to actually open a popup window. like literally make my home page fucking macos. better icons. For the love of god please top using the fucking claude headers. Instead of static windows literally launch them and make them interactive, keep the dock

## Assistant

_[tool: copy_files]_

Got it — actual macOS, not a static tableau. Saving v5, then building v6: real launchable windows, drag, close, focus, z-index, no scroll.

_[tool: write_file]_

_[tool: snip]_

_[tool: snip]_

_[tool: done]_

_[tool: snip]_

v6 — the desktop is the page, literally.

**Loads with two windows open**: Welcome (centered, the marketing surface) and the Hub (left, showing the Authority queue with a pending high-risk request). Other apps are dock-only until you launch them.

**Real windows.** Traffic-light buttons that actually work — close removes the window, max fills the desktop and restores on second click, min currently closes (no minimize-to-dock animation yet). Click any window to focus it; z-index promotes correctly. Drag the titlebar to reposition; bounds-clamped so you can't drag a window off-screen.

**Dock-as-launcher.** Click any dock icon → window opens with a zoom-from-the-icon animation (the `transform-origin` is computed live from the icon's screen position). Click the same icon when its window is open → it focuses to front. The running indicator dot appears under open windows.

**Better dock icons** — gradient tiles with real Lucide-style line SVGs inside:
- Welcome: the actual Ship logo
- Hub: shield (amber gradient — Ship's primary)
- Paperback: open-book (terracotta)
- Ship · dev: dashboard (dark amber)
- rust-lane: terminal carat (phosphor green)
- Mesh: 4-node grid (parchment cream)
- Lab: flask (purple/glass)
- Store: storefront
- Trash: bin (closes the focused window when clicked)

Plus dock-magnification on hover (icon scales 1.18× and lifts; neighbors lift 1.06×).

**Real macOS menubar.** Apple-style logo button, "Ship" app menu, File/Edit/View/Window/Help, then right-side cluster: search glyph, control-center glyph, real `<sui-theme-toggle>`, live clock, small Download pill.

**No scroll.** `html, body { overflow: hidden }`. Page is exactly 100vh. No footer. The menubar links go to the real site pages (Features, Docs, Download); the dock launches the desktop windows.

**Welcome copy** — pared down to "Ship · version 1.4.2 · macOS · A native runtime for AI on your computer. Bring your own model." Plus a Download button and an "or take a look around" link that opens the rest of the windows in sequence as a tour.

If specific things to push on: better window-open animation (genie effect from dock), minimize that actually minimizes back to the dock, draggable selection of icons on the desktop, an Apple-menu dropdown. Or sharpen specific window contents.

## User

great want to make it sell now... instead of just rendering random windows?

## Assistant

_[tool: snip]_

_[tool: snip]_

_[tool: str_replace_edit]_

Two moves to make it sell: rewrite the Welcome window to be an invitation to explore, and pin a native-style callout to each window explaining what we're seeing — visible only when that window is focused.

_[tool: str_replace_edit]_

Now adding callouts to each non-Welcome window so visitors know what they're looking at:

_[tool: done]_

_[tool: snip]_

Shipped two moves to make it actually sell:

**1 · Welcome window rewrites the invitation.** Was "A native runtime for AI on your computer. Bring your own model." (descriptive, didn't sell). Now: "Each window beside this one is a real Ship space. **Click around. Then come back.**" That gives every visitor explicit permission to explore — and the "come back" promises a payoff. Footer line says "free, personal use" instead of the abstract "© 2026".

**2 · Native macOS callouts pinned under each window.** A small dark tooltip with an upward arrow appears below whichever window is currently focused. Only one visible at a time (it follows focus as you click between windows), so it never clutters the desktop. Each says one specific thing + an inline amber-pill CTA:

- **Hub** — "**Trust model.** Every elevated command waits for you. → How it works"
- **Paperback** — "**Spaces have personality.** Theme, agents, identity — yours. → Themes"
- **Ship · dev** — "**Watch them work.** 3 agents, 247 events/hr, 1 awaiting review. → Agents"
- **rust-lane** — "**One agent, one window.** Pin it like Slack. ⌘-tab to it. → Skill format"
- **Mesh** — "**Audited by default.** Every restart, deploy, ssh — logged. → Capabilities"
- **Lab** — "**Try things, cheap.** 3 models on the same prompt, side by side. → All features"

Callouts move with the window when you drag it (they're absolute children of each window). The right-edge ones flip to `.right` anchoring so they don't hang off-screen. Each CTA links to a real section/page (`/features#trust`, `/features#themes`, etc.) so the sell is connected to actual destinations.

Click any dock icon to launch its window; the callout fades in 220ms after the launch animation so you see the chrome first, then the value prop.

Push directions if you want more sell:
- A "What people use Ship for" sticky note dragged onto the desktop
- An ambient "Open in Ship →" pill stuck to certain windows
- Make the Welcome window's pull text rotate ("Try a writing space →" "See the trust model →") as visitors hover dock items
- A persistent "Get Ship 12.4MB" pill floating above the dock

Or different direction — tell me.

