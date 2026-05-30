# Job Spec: Fix Tauri shell Rust let-chains syntax error

**Branch:** job/fix-tauri-letchains (from main)  
**Blocking:** #123 buildchain verification  
**Agent:** general-purpose or code-reviewer

## Goal

Resolve the compilation error in apps/shell/src-tauri that uses Rust 2024 let-chains syntax in an edition 2021 crate. The error blocks `cargo test --workspace`.

## Problem

**File:** apps/shell/src-tauri/src/lib.rs  
**Lines:** 69–70  
**Error:** `let chains are only allowed in Rust 2024 or later`

**Code:**
```rust
if let Some(port) = crate::daemon::daemon_status().port
    && let Ok(url) = format!("http://127.0.0.1:{port}{target}").parse()
{
    let _ = window.navigate(url);
}
```

**Root Cause:** Commit e4b5fea9f (feat(shell): hub builds to .deck/hub-scaffold/, Tauri navigates to /hub) refactored the hub + space navigation into a symmetric code path but used Rust 2024 let-chains syntax without updating the crate edition.

**Crate Edition:** 2021 (in apps/shell/src-tauri/Cargo.toml)

## What to Do

### Option A: Revert to nested if-let (recommended)

Replace the let-chains with pre-chains nested if-let:

```rust
if let Some(port) = crate::daemon::daemon_status().port {
    if let Ok(url) = format!("http://127.0.0.1:{port}{target}").parse() {
        let _ = window.navigate(url);
    }
}
```

**Rationale:** Syntactic convenience only; nesting is idiomatic Rust 2021 and low-risk.

### Option B: Upgrade crate edition to 2024

If deliberate 2024 adoption:
1. Update `edition = "2021"` → `"2024"` in apps/shell/src-tauri/Cargo.toml.
2. Verify all workspace dependencies support Rust 2024.
3. Run `cargo test --workspace` to check for new 2024-only constraint issues.

**Rationale:** Future-proofs the crate but requires verifying workspace-wide compatibility.

## Acceptance Criteria

1. Edit apps/shell/src-tauri/src/lib.rs (lines 69–72 for Option A, or Cargo.toml for Option B).
2. Run `cargo test --workspace` — no compilation errors.
3. Commit with message: `fix(shell): resolve Rust let-chains syntax in edition 2021 crate` (Option A) or `chore(shell): upgrade crate edition to 2024` (Option B).
4. No test failures introduced by the change (compilation itself is the gate).

## What NOT to Do

- Do not silently ignore the error — it blocks the buildchain.
- Do not use #![feature(let_chains)] — avoid nightly features unless explicitly required.
- Do not modify other files unless the fix requires it (it shouldn't).

## Follow-Up

After merge, return to the buildchain verification job and re-run:
```
cargo test --workspace
just bench-check  (once justfile is wired)
```

Confirm the full buildchain is clean.
