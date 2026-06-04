#!/usr/bin/env bash
# Grep-gate (agents arc S5b — writer cutover): the consolidated FsBroker WRITE
# door in `ship-config/src/agents_write.rs` is the ONLY writer of
# `.ship/agents/<id>.jsonc` def files among the cut-over consumers. ZERO raw
# `std::fs`/`tokio::fs` WRITES of agent-def content (write/create/remove/rename/
# trash) live in the repointed surface — the daemon CRUD now routes through the
# door, not `FilesService`.
#
# Detection: a "def writer" is a raw-fs MUTATING call whose subject — on the
# same line or within a small window above it — is an agents directory or a
# `.jsonc` def path, and which is NOT a permissions/mcp/skills/rules file (those
# are other domains, out of this slice).
#
# Allow-listed (documented follows, NOT silent passes):
#   - the consolidated FsBroker door + the reader + their tests (by design);
#   - `apps/shipd/**` — behind the deck SUI-bundle build wall; the command/agents
#     wiring drives the door via `ctx.state.agents()` and is operator-verified at
#     merge (FLAGGED, S6 repoint of shipd's own raw-fs readers/writers);
#   - `apps/ship-cli/**` — the CLI's LEGACY scaffold/validate paths the door does
#     not serve (FLAGGED, separate follow);
#   - `FilesService` itself (`services/files.rs`) — the GENERIC file service other
#     domains (skills/rules/mcp/views) still use; only agent-def writes moved off
#     it, so its raw fs is in-bounds (its own paths are not def paths).
#   - inline `#[cfg(test)]` modules — test scaffolding (fixture dir setup) and the
#     watcher tests that DELIBERATELY write def files raw to simulate the external
#     hand-edit the watcher re-indexes (the design's 5% power case). The scan
#     stops at the file's first `#[cfg(test)]` (Rust convention puts test modules
#     at the bottom), mirroring the reader gate's test-file allow-list.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

ALLOW_RE='(crates/core/ship-platform/crates/ship-config/src/agents\.rs|crates/core/ship-platform/crates/ship-config/src/agents_write\.rs|crates/core/ship-platform/crates/ship-config/src/agents_tests\.rs|crates/core/ship-platform/src/services/files\.rs|apps/shipd/|apps/ship-cli/|tests/|_tests\.rs|/tests\.rs$)'

# Raw-fs mutating shapes (both `std::fs::` and a bare `fs::` after `use std::fs`).
WRITE_RE='((std::)?fs::write|(std::)?fs::remove_file|(std::)?fs::remove_dir|(std::)?fs::rename|(std::)?fs::create_dir|(std::)?fs::File::create|(std::)?fs::OpenOptions|tokio::fs::write|tokio::fs::remove_file|tokio::fs::remove_dir|tokio::fs::rename|tokio::fs::create_dir|tokio::fs::File::create|tokio::fs::OpenOptions)'
# A def path: an agents dir or an `<id>.jsonc` under it. Exclude the sibling
# non-def configs that also live near agents tokens (permissions/mcp/skills).
DEF_RE='(agents_dir|agents_ns|join."agents"|.ship/agents|agents/[^"]*[.]jsonc|.agent_id..jsonc|.id..jsonc|.new_id..jsonc)'
NONDEF_RE='(permissions[.]|mcp[.]|skills/|rules/)'

offenders=""
while IFS= read -r -d '' file; do
  case "$file" in
    */target/*) continue ;;
  esac
  printf '%s\n' "$file" | grep -Eq "$ALLOW_RE" && continue

  hits="$(awk -v write_re="$WRITE_RE" -v def_re="$DEF_RE" -v nondef_re="$NONDEF_RE" '
    { lines[NR] = $0 }
    END {
      # The cutover targets PRODUCTION consumers. Inline `#[cfg(test)]` modules
      # (Rust convention: bottom of file) are test scaffolding — fixture dir
      # setup, and watcher tests that DELIBERATELY write def files raw to
      # simulate the external hand-edit the watcher re-indexes. Mirror the reader
      # gate, which allow-lists test files; here we stop at the first test cfg.
      last = NR
      for (i = 1; i <= NR; i++) if (lines[i] ~ /#\[cfg\(test\)\]/) { last = i - 1; break }
      for (i = 1; i <= last; i++) {
        line = lines[i]
        stripped = line; sub(/^[ \t]*/, "", stripped)
        if (stripped ~ /^\/\//) continue
        if (line ~ write_re) {
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
  echo "FAIL: raw-fs agent-def writers found outside the consolidated write door:" >&2
  printf '%b\n' "$offenders" >&2
  echo "" >&2
  echo "Route def writes through ship_config::agents_write (FsBroker write door)." >&2
  exit 1
fi

echo "OK: ship_config::agents_write is the only agent-def writer among cut-over consumers."
