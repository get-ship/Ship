#!/usr/bin/env bash
# Grep-gate (agents arc S2 — reader cutover): the consolidated FsBroker reader
# in `ship-config/src/agents.rs` is the ONLY reader of `.ship/agents/<id>.jsonc`
# def files among the cut-over consumers. ZERO raw `std::fs`/`tokio::fs` READS
# of agent-def CONTENT live in the repointed surface.
#
# Detection: a "def reader" is a raw-fs READ call whose subject — on the same
# line or within a small window above it — is an agents directory or a `.jsonc`
# def path, and which is NOT a permissions/mcp/skills file (those are other
# domains, out of this slice).
#
# Allow-listed (documented follows, NOT silent passes):
#   - the consolidated reader + its S5 write half + its tests (FsBroker, by design);
#   - `apps/shipd/**` — behind the deck SUI-bundle build wall; repoint in S3/S6
#     when shipd is buildable in-slice (FLAGGED);
#   - `apps/ship-cli/src/validate.rs` — `ship validate` walks the LEGACY
#     `agents/profiles` + `agents/presets` compat dirs the reader deliberately
#     does not serve; its semantics differ (FLAGGED, separate follow);
#   - `apps/ship-cli/src/init.rs` — an emptiness PROBE (`read_dir` to decide
#     whether to scaffold a default agent); reads no def content (FLAGGED).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

ALLOW_RE='(crates/core/ship-platform/crates/ship-config/src/agents\.rs|crates/core/ship-platform/crates/ship-config/src/agents_write\.rs|crates/core/ship-platform/crates/ship-config/src/agents_tests\.rs|apps/shipd/|apps/ship-cli/src/validate\.rs|apps/ship-cli/src/init\.rs|tests/|_tests\.rs|/tests\.rs$)'

# Raw-fs read shapes (both `std::fs::` and a bare `fs::` after `use std::fs`).
READ_RE='((std::)?fs::read_dir|(std::)?fs::read_to_string|(std::)?fs::read[^_a-zA-Z]|tokio::fs::read_dir|tokio::fs::read_to_string|tokio::fs::read[^_a-zA-Z])'
# A def path: an agents dir or an `<id>.jsonc` under it. Exclude the sibling
# non-def configs that also live near agents tokens (permissions/mcp/skills).
DEF_RE='(agents_dir|agents_ns|join."agents"|.ship/agents|agents/[^"]*[.]jsonc|.agent_id..jsonc|.id..jsonc)'
NONDEF_RE='(permissions[.]|mcp[.]|skills/|rules/)'

offenders=""
while IFS= read -r -d '' file; do
  case "$file" in
    */target/*) continue ;;
  esac
  printf '%s\n' "$file" | grep -Eq "$ALLOW_RE" && continue

  hits="$(awk -v read_re="$READ_RE" -v def_re="$DEF_RE" -v nondef_re="$NONDEF_RE" '
    { lines[NR] = $0 }
    END {
      for (i = 1; i <= NR; i++) {
        line = lines[i]
        stripped = line; sub(/^[ \t]*/, "", stripped)
        if (stripped ~ /^\/\//) continue
        if (line ~ read_re) {
          win = line
          for (j = i-4; j < i; j++) if (j >= 1) win = win "\n" lines[j]
          if (win ~ def_re && win !~ nondef_re) printf("%d:%s\n", i, line)
        }
      }
    }
  ' "$file")"

  if [ -n "$hits" ]; then
    offenders="${offenders}\n--- ${file} ---\n${hits}"
  fi
done < <(find crates apps -name '*.rs' -type f -print0)

if [ -n "$offenders" ]; then
  echo "FAIL: raw-fs agent-def readers found outside the consolidated reader:" >&2
  printf '%b\n' "$offenders" >&2
  echo "" >&2
  echo "Route def reads through ship_config::agents (FsBroker reader)." >&2
  exit 1
fi

echo "OK: ship_config::agents is the only agent-def reader among cut-over consumers."
