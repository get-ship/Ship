/// The stack root for the current space: the nearest ancestor carrying
/// `stack.config.ts`. Errors when no ancestor is a stack root. Worktree-aware
/// via `get_stack_root`, so it resolves from subdirs and linked worktrees.
pub fn space_ship_dir_required() -> anyhow::Result<std::path::PathBuf> {
    ship_substrate::project::get_stack_root(None)
}
