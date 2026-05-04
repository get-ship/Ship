# Ship

**Ship is an application runtime for AI-native applications.** You describe what your app should do — its data, its agents, its surfaces, its rules — and Ship runs it. Local-first, with cloud sync as the optional paid layer.

---

## Who Ship is for

People and small teams who want to build apps for personal or internal use. Not for shipping consumer startups. Not for one-shot demos. The platform pre-decides the boring choices (framework, database, deployment, auth) so the parts you describe are the parts that make your app yours.

## How you start

Open Ship. Studio launches. A default project is waiting.

Describe what you want to build, in plain language:

> "A reading list that summarizes articles I add."
>
> "A daily journal that suggests reflections."
>
> "An expense tracker that pulls categories from receipts I upload."

A first-party agent reads your description and scaffolds a real Ship app — **collections** for the data, **agents** for the work, **pipelines** to connect them, **views** for the surface, **permissions** for the boundaries. The components appear in your project as you watch.

The app runs the moment it's scaffolded. You use it. You edit a view's template; it hot-reloads. You ask the agent to add a feature; new components appear. You hand the project to a friend; they install it and run the same app.

Five minutes. Real data. Real agents. Real interactivity. No deploy step. No framework choice. Yours.

This is the 80% path. The other 20% is hand-authoring components — same primitives, same runtime, just direct.

## What an app on Ship is

Ship apps live inside **projects**. A project is your working environment — desktop-shaped, repo-shaped. The default project lives at `~/.ship/projects/default` and exists the first time Ship runs. You add more projects as you need them, optionally synced.

Inside a project you mix and match **components**:

- **Views** — the surfaces people interact with. Colloquially, "the app."
- **Agents** — LLM-driven workers with personas, skills, tools.
- **Skills** — reusable units of capability agents can call.
- **Pipelines** — orchestrations of agent and actor steps.
- **Collections** — schemaful structured data.
- **Files** — blob storage with metadata.
- **Permissions / Rules** — what the app and its agents can and can't do.
- **MCP integrations** — connections to external systems.

Components are declarative. You write them as config, not code. The runtime knows how to execute them. Edits are hot — change a file, the running system reflects it.

## What Ship handles for you

- **Storage** — KV for small values, Files for blobs, Collections for records, Events for runtime state. All four flow through one event log so sync works uniformly across them.
- **Compute** — Agents and pipelines run as supervised actors with mailboxes. Crashes are handled. Conversations persist across restarts. Replay is built in.
- **Sync** — Ship Cloud replicates the event log between machines. Teams share projects through it. Changes show up live.
- **Observability** — Every meaningful state change is an event. Replay any session. Audit any action. Debug by scrubbing through the log.
- **Authoring** — Edit a config file in your editor, in Studio, or by hand. The runtime reflects it. Hot reload by default. No build step.

## How it runs

Ship is a local daemon plus a **Studio shell** — the persistent chrome you use to navigate, edit, and run your projects. Today Studio is a desktop app. In the next 12–18 months it becomes a forked browser tab where Ship projects sit alongside web pages.

Sync goes through **Ship Cloud** — the paid layer. Cloud handles team identity, project replication between team members, and the marketplace. Solo users with no cloud account run entirely local; everything except sharing works without an internet connection.

Local-first means: your data is in your filesystem and your local databases. Cloud sync is a property added on top, not a hosting model that owns your data.

## The primitives

| Tier | Primitive | What it is |
|---|---|---|
| **Storage** | KV | Bounded key-value store. Skill vars, app state, settings. |
| | Files | Blob storage with metadata. Generated artifacts, uploads. |
| | Collections | Schemaful structured data. The records your app holds. |
| | Events | Append-only log plus derived projections. The runtime's spine. |
| **Compute** | Actors | Supervised processes with mailboxes. The Ship-internal compute primitive. |
| | Agents | LLM-driven actors with profile, skills, tools, permissions. |
| | Pipelines | DAGs of agent and actor steps. |
| **Configuration** | Agent profiles | Who an agent is — model, prompts, skills, tools. |
| | Skills | Reusable capability units agents call. |
| | Rules | Behavioral constraints. |
| | Permissions | What's allowed at the actor and view layer. |
| | MCP integrations | External tool bindings. |
| | Views | UI surfaces, built with Ship UI (Ship's own rendering framework). |

The runtime substrate — event log, actor supervision, projections, mesh routing, sessions, jobs — is **L0**. Users don't author L0; they use it through the primitives above.

## Storage modalities

Two complementary stores, both first-class:

- **Filesystem-canonical** for declarative config. You edit agent profiles, skills, rules, permissions, view templates as files in your project directory. Familiar, diff-friendly, editable by any tool. The daemon's filesystem watcher emits a `<domain>.changed` event whenever a file changes — that's how the runtime learns and how sync propagates.
- **Event-canonical** for operational state. Actor lifecycle, session activity, pipeline runs, KV writes, file uploads, collection mutations all emit events into the log. The log is the source of truth; SQLite projections feed reads.

Both modalities flow through the same event log, so cloud sync is one mechanism: replicate the log; rebuild projections on the other side; let the filesystem watcher reconcile config-file projections back to disk. One sync substrate, two storage shapes layered on top.

## What's in the box

Ship ships with **first-party views** — chat, dashboard, settings, project switcher, and the agent that scaffolds new apps. They are built exactly the same way third-party views are. They scaffold to global config on first run. They're the reference for what good Ship views look like, because the team builds Ship using Ship.

There is no special framework hidden inside the daemon for first-party views. If you can read a first-party view's source, you can build one yourself.

## The Shop

Ship Shop (the marketplace) is a distribution layer for components people share — views, pipelines, agents, skills, collection schemas, MCP integrations. Most marketplace items are bundled around a view: install the view, it brings the components it depends on.

Sharing is for teams and friends. Quality bar is "useful," not "consumer-polished."

## What's not in v1

- **Triggers as a user-facing primitive.** The reactive engine exists internally; user-defined triggers (`when this event happens, run this action`) come after v1.
- **Marketplace polish.** v1 supports installing components by URL or repo. The curated Shop UI lands later.
- **Cross-project access.** Projects are isolated by default in v1. Cross-project queries are an explicit affordance later.
- **Browser-fork Studio shell.** v1 has the desktop Studio app. The browser shell is 12–18 months out, but everything in v1 is built so the shell can swap without changing the app contract.
- **Cloud team workflows beyond sync.** v1 cloud is sync, identity, and the marketplace. Comments, notifications, permission inheritance across teams come later.
