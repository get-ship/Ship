---
name: no-slop
audience: internal
stable-id: no-slop
description: Ten commandments for building software with AI agents. No slop.. Hard-won, opinionated, concise.
tags: [philosophy, quality, agents, principles]
authors: [ship]
---
# The Ten Commandments of Agentic Software

 1. **This is real software.** No placeholders. No TODOs. No fake data. Every element works or it doesn't exist.

 2. **Brevity is a feature.** One sentence beats three. Don't narrate your reasoning.

 3. **What you leave behind becomes architecture.** The next agent treats your scaffold as intentional design. Surface depth is a liability.

 4. **Surface problems. Never pass them.** Don't fix out-of-scope issues, but never ignore one either. A known bug is manageable. An invisible one compounds.

 5. **Ask, then build.** "Is this the right approach?" before 500 lines saves a rewrite.

 6. **Use the API. Never circumvent it.** No raw SQL. No shell scripts bypassing interfaces. If an API exists, use it. If it doesn't, flag it.

 7. **Don't reach for an LLM when idempotency is required.** Agentic programs have more entropy than classical programs. Use deterministic tools for deterministic jobs.

 8. **Context rot is the root of all evil.** Stale context produces stale output. Read before you assume. Verify your understanding is current.

 9. **Don't trust. Verify.** Test the behavior, not the claim. Code that "should work" doesn't.

10. **Don't shoehorn.** Recent conversation, recent commits, and recent context are not relevant to every task. Do the work that was asked for, not a remix of what you just saw.

11. **Document the present, not the past.** Code, comments, and skills describe what is — not what was tried, what was rejected, or how the design evolved. The next agent never saw your conversation. No "previously," no "we used to," no "now" (which implies a past it can't see), no narration of the alternatives you considered. State the rule. State what the code does. Move on.

12. **Don't retry tests hoping for different output.** Run the full test suite once. If it fails, read the failure, identify the root cause, fix the cause, run again. Two failed runs in a row from the same cause are a bug to diagnose, not a flake to retry. Three is the system burning memory because you stopped reading the failures. If a test hangs, vitest's `testTimeout` will catch it — read the timeout report and fix the never-resolving Promise, don't bump the timeout.