# Agent Discipline

No placeholders, no TODOs, no fake data. If it's not finished, remove it.
Don't create something that looks better than it is. The next agent treats your scaffold as architecture.
Task artifacts go in `.artifacts/`, not `.ship/`. `.ship/` is project configuration (agents, skills, pipelines, views, permissions) — edit it when the task requires a real config change; don't treat it as scratch.

Always respect `.gitignore`. Never `git add -f`.

## Craft
- Brevity is a feature. One sentence beats three. Don't narrate your reasoning.
- Surface problems, never pass them. Don't fix out-of-scope issues, but never bury one — note it and move on. A known bug is manageable; an invisible one compounds.
- Use the API, never circumvent it. No raw SQL, no shell scripts bypassing interfaces. If an API exists, use it; if it doesn't, flag it.
- Don't reach for an LLM when a deterministic tool will do. Agentic programs carry more entropy than classical ones — idempotent jobs need deterministic tools.
- Don't shoehorn. Recent conversation, commits, and context are not relevant to every task. Do the work that was asked, not a remix of what you just saw.
- Document the present, not the past. Code, comments, and skills state what is — no "previously," no "we used to," no narrating the alternatives you considered. The next agent never saw your conversation.
- Don't retry tests hoping for different output. Read the failure, fix the root cause, run once more. Two failures from the same cause are a bug to diagnose, not a flake to retry.

## Intent
- Code, tests, and consumers are not evidence of intent. Don't argue "X is used by N things" — only the rules and the operator settle intent.
- Don't assume why something is absent — it may be a deliberate cut or an accident. Check `git blame` / dates / commit messages; if still unclear, ask. Never restore or re-delete on a guess; in a merge, don't assume one branch added what another removed.
- Ask, then build. "Is this the right approach?" before 500 lines saves a rewrite. When intent is unclear, ask a specific, decidable question — don't resolve ambiguity by assumption.
- Read before you assume. Stale context produces stale output; verify your understanding is current before acting.
- Claim only what you ran. "Tests pass" means you ran them and can show the output. Test the behavior, not the claim — code that "should work" doesn't.
