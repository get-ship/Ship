#!/usr/bin/env bash
# check-runtime-access.sh
#
# Enforce modular-runtime-03 spec acceptance: handlers should reach the
# runtime through the `RuntimeAccess` trait surface, not through the
# concrete `Arc<ShipRuntime>`.
#
# Scope: only the directories that should be handler-trait-bound:
#   apps/shipd/src/services/   (mr-02 service modules)
#   apps/shipd/src/collection_api.rs / artifacts_api.rs
#   apps/shipd/src/runtime_api.rs / chat / mesh / hooks
#
# Excluded by design:
#   - apps/shipd/src/access.rs / ship_runtime.rs / lib.rs / main.rs /
#     router.rs / rest_api.rs / server.rs — assembly + access layer
#   - any *_tests.rs file — test fixtures construct the runtime concretely
#   - apps/shipd/src/supervisor/ — internal helpers, not HTTP handlers
#   - apps/shipd/src/managers/ — same reason
#
# Mirrors the spec's grep acceptance:
#   rg "Arc<ShipRuntime>" handler-paths   → zero hits

set -u

cd "$(dirname "$0")/../.."

if ! command -v rg >/dev/null 2>&1; then
    echo "check-runtime-access: ripgrep (rg) not installed; skipping"
    exit 0
fi

handler_paths=(
    apps/shipd/src/services/
    apps/shipd/src/collection_api.rs
    apps/shipd/src/artifacts_api.rs
    apps/shipd/src/runtime_api.rs
    apps/shipd/src/hooks_api.rs
    apps/shipd/src/actor_api.rs
    apps/shipd/src/actors_api.rs
    apps/shipd/src/annotations_api.rs
    apps/shipd/src/feedback_api.rs
    apps/shipd/src/specs_api.rs
    apps/shipd/src/lint_api.rs
    apps/shipd/src/logs_api.rs
    apps/shipd/src/view_errors_api.rs
    apps/shipd/src/view_sessions_api.rs
    apps/shipd/src/settings_api.rs
    apps/shipd/src/project_api/
)

violations=$(rg --line-number 'Arc<ShipRuntime>|: *&ShipRuntime\b|: *ShipRuntime\b|::ShipRuntime::' \
    "${handler_paths[@]}" 2>/dev/null \
    | grep -Ev '/tests?\.rs:|_tests?\.rs:' \
    | grep -Ev '^[^:]*://' \
    | grep -Ev 'new_test\(' \
    || true)

if [ -n "$violations" ]; then
    echo "check-runtime-access: handler files reaching into ShipRuntime directly:"
    echo "$violations"
    echo
    echo "Reach the runtime through crate::access (RuntimeAccess /"
    echo "ServicesAccess / KernelAccess / MeshAccess / AuthAccess /"
    echo "ConfigAccess). Handlers take State<ApiState> and call"
    echo "state.services().X() or the relevant sub-trait method."
    exit 1
fi

echo "check-runtime-access: ok"
exit 0
