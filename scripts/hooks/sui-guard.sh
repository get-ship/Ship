#!/usr/bin/env bash
# scripts/hooks/sui-guard.sh — pre-edit guard for `.sui` + `.controller.ts`.
#
# Wired into a Claude Code (or any harness with PreToolUse hooks)
# `PreToolUse` matcher on `Edit|Write` of paths matching `**/*.sui` or
# `**/*.controller.ts`. The hook emits the SUI cardinals (mirror of
# docs/ship-sui/SKILL.md § Hard rules) so the agent sees them BEFORE
# the edit lands, not after the lint rejects it.
#
# Wire it in `~/.claude/settings.json` (per the project's harness
# convention; the in-repo `.claude/` is gitignored):
#
#   {
#     "hooks": {
#       "PreToolUse": [
#         {
#           "matcher": "Edit|Write",
#           "command": "scripts/hooks/sui-guard.sh \"$CLAUDE_FILE_PATH\""
#         }
#       ]
#     }
#   }
#
# Exit codes: 0 always (the hook is advisory injection, not a blocker —
# the lint at compile time IS the blocker).

set -u

path="${1:-}"

# Only emit on SUI files. Anything else gets a fast-path no-op so the hook
# stays cheap on every Edit/Write.
case "$path" in
  *.sui|*.controller.ts) ;;
  *) exit 0 ;;
esac

cat <<'EOF'
[sui-guard] SUI authoring — the 7 cardinals (sui-lint will catch violations):

  1. Templates declare, controllers do. No <script> in templates; no
     innerHTML / outerHTML / insertAdjacentHTML in controllers. Use
     bindHtml(el, () => string) for HTML-blob cases.

  2. Reads + writes — one form per context:
     - Templates / derived / effect:  x() reads + tracks, x.set(v) writes.
       The scope passes raw Signal<T>. NEVER state.x here.
     - Controller imperative (on:, mount, unmount):  state.x reads value,
       state.x = v writes. NEVER state.x() or state.x.set(v).
       Escape hatch for updater writes: state.$signals.x.set(prev => ...)

  3. Mutations in on:* handlers only. state.x = v in derived/effect/
     template is a type error.

  4. Subscriptions in <data> elements. Never call ship.* outside <data>
     (or inside <form mutation>).

  5. Block control flow: {#if} / {#for} / {#match} / {#while-loading} /
     {#while-error}. {#for} REQUIRES key={EXPR}.

  6. No customElements.define in view code. The compiler emits it for
     components; pack umbrellas register all tags on import.

  7. Dynamic routes need defineRoutes at shell startup. Per-controller
     defineRoutes lives in a separate runtime instance and won't reach
     the shell's signal proxy.

  Lint rules that promote to compile errors (sui-lint Severity::Error):
    sui-controller-purity        innerHTML / outerHTML / insertAdjacentHTML
    sui-controller-state-cast    state as unknown as <Scope> casts
    sui-controller-navigation    fetch/XHR in controllers
    sui-controller-context       ship-client misuse
    sui-controller-set-value     setAttribute('value', x) silent footgun
    sui-state-set-call           state.x.set(v) after #179 unification
    sui-css-token / css-literal / css-content-text / prop-value / attr-bool-prop

EOF

exit 0
