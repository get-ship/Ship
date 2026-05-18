---
group: Getting Started
title: First Launch
section: guide
order: 2
---

The first time you launch Ship, the app opens in your browser. A short setup walks you through three things:

1. **Welcome** — orients you. Ship runs entirely on your machine.
2. **Pick a project folder** — start a new project or open an existing one. Ship sets up the project workspace.
3. **Connect a provider** — pick Claude, Cursor, Gemini, Codex, or OpenCode and paste an API key. You can skip this and connect later.

That's it. After setup you land on the dashboard.

## If the browser doesn't open

Run `ship daemon status` in your terminal — it prints the URL the daemon is serving on (the port is auto-derived per worktree). Open that URL directly.
