#!/usr/bin/env bash
# ship-dispatch:pre-push v1
# Blocks pushes of refs/heads/job/* — those are dispatched agent branches
# the operator gates locally. Operator bypass: SHIP_ALLOW_PUSH=1 git push ...
if [[ -n "${SHIP_ALLOW_PUSH:-}" ]]; then exit 0; fi
while read -r local_ref local_sha remote_ref remote_sha; do
  case "$remote_ref" in
    refs/heads/job/*)
      echo "" >&2
      echo "  ✖ git push blocked — $remote_ref is a dispatched agent branch." >&2
      echo "    Operator gates locally and merges from the main repo." >&2
      echo "    Intentional push: SHIP_ALLOW_PUSH=1 git push ..." >&2
      echo "" >&2
      exit 1 ;;
  esac
done
exit 0
