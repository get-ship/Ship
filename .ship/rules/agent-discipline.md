# Agent Discipline

No placeholders, no TODOs, no fake data. If it's not finished, remove it.
Don't create something that looks better than it is. The next agent treats your scaffold as architecture.
Task artifacts go in `.artifacts/`, not `.ship/`. `.ship/` is project configuration (agents, skills, pipelines, views, permissions) — edit it when the task requires a real config change; don't treat it as scratch.

Always respect `.gitignore`. Never `git add -f`.

## Intent
- Code, tests, and consumers are not evidence of intent. Don't argue "X is used by N things" — only the rules and the operator settle intent.
- Don't assume why something is absent — it may be a deliberate cut or an accident. Check `git blame` / dates / commit messages; if still unclear, ask. Never restore or re-delete on a guess; in a merge, don't assume one branch added what another removed.
- When intent is unclear, ask a specific, decidable question. Don't resolve ambiguity by assumption.
- Claim only what you ran. "Tests pass" means you ran them and can show the output.
