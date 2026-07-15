//! Registration-shaped membership: compose a space from an EXPLICIT list
//! of member stack roots — no directory scanning, no space-root privilege.
//!
//! This is the target composition entry for the stack seam (blueprint:
//! `stack-resolver` / `req-res-membership`): the space is a runtime fact
//! and its members are REGISTERED, so the caller (a daemon reading the
//! space registry) supplies the roots. Invariants encoded here rather
//! than inferred:
//!
//! - **No scanning.** A stack on disk beside a member never composes.
//! - **The push fence applies to every member.** No member manifest can
//!   grant permissions or ship space-active hooks — space-level push
//!   policy belongs to the runtime (user-authored), never to a stack.
//! - **Per-member lock.** Each member's `ship.lock` rides beside its own
//!   manifest (lockfile-beside-manifest); the cache tier is the union.
//! - **Determinism.** Members compose in caller-supplied order (the
//!   registry's stable order); `compose_space`'s equal-rank law
//!   ("later declared wins") applies within the Space tier.
//!
//! [`roll_up`](crate::roll_up) (scan-based, space-root PIN) remains for
//! existing callers until the daemons feed membership from the registry.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use stack_model::{ComposedLibrary, ExtSource, ExtensionDescriptor, RollupInputs};
use serde_json::Value;
use ship_substrate::ShipHome;
use ship_substrate::ports::LocalFsBroker;

use crate::{descriptor, merge_ts_declarations, space_loader, ts_resolver};

/// The member roots of the space `anchor` belongs to, from the registry.
///
/// Resolution: `anchor`'s space (the `spaces` projection row for its
/// canonical path) -> that space's registered `space_stacks` rows, in
/// registration order. An anchor with no space row or no registrations
/// composes ALONE — the correct semantics for a single-stack space, not
/// a fallback (nothing is scanned either way).
pub fn resolve_member_roots(anchor: &Path) -> Vec<PathBuf> {
    // The registry read runs on a dedicated OS thread: the substrate's
    // sqlite `block_on` uses `block_in_place` when a tokio handle is
    // ambient, which panics on current-thread runtimes. A plain thread
    // has no tokio context, so the read is safe from every caller
    // (daemon, CLI, current-thread test).
    let probe = anchor.to_path_buf();
    let members: Vec<PathBuf> = std::thread::spawn(move || {
        ship_substrate::project::resolve_active_space(&probe)
            .and_then(|space_id| ship_substrate::db::spaces::list_stacks(&space_id).ok())
            .map(|rows| {
                rows.into_iter()
                    .map(|r| PathBuf::from(r.root_path))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    })
    .join()
    .unwrap_or_default();
    if members.is_empty() {
        vec![anchor.to_path_buf()]
    } else {
        members
    }
}

/// Compose the space `anchor` belongs to from its REGISTERED membership.
pub async fn roll_up_registered(
    anchor: &Path,
    cache_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    let roots = resolve_member_roots(anchor);
    roll_up_members(&roots, cache_root).await
}

/// Synchronous bridge over [`roll_up_registered`] for callers that cannot
/// `.await` (e.g. ship-code's sync `AgentPlatform::compile_agent_config`).
/// Reuses the calling runtime via `block_in_place` when one is current;
/// falls back to a throwaway current-thread runtime for plain sync callers.
pub fn roll_up_registered_blocking(
    anchor: &Path,
    cache_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(roll_up_registered(anchor, cache_root))
        }),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("roll_up_registered_blocking: building fallback runtime")?
            .block_on(roll_up_registered(anchor, cache_root)),
    }
}

/// Synchronous bridge over [`roll_up_members`] — same contract as
/// [`roll_up_registered_blocking`], for callers that computed their own
/// member roots (e.g. ship-code's worktree stand-in).
pub fn roll_up_members_blocking(
    member_roots: &[PathBuf],
    cache_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(roll_up_members(member_roots, cache_root))
        }),
        Err(_) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("roll_up_members_blocking: building fallback runtime")?
            .block_on(roll_up_members(member_roots, cache_root)),
    }
}

/// Compose a space from explicit member stack roots.
pub async fn roll_up_members(
    member_roots: &[PathBuf],
    cache_root: Option<&Path>,
) -> Result<ComposedLibrary> {
    let fs = LocalFsBroker::new();

    // 1. Per-member descriptor: load + TS-eval each member root. The
    //    push fence is applied at the identity source — permissions +
    //    hooks are zeroed BEFORE the descriptor is built, so a member
    //    cannot grant itself anything regardless of compose order.
    let mut member_descriptors = Vec::with_capacity(member_roots.len());
    for root in member_roots {
        let mut lib = space_loader::load_root_library(&fs, root)
            .await
            .with_context(|| {
                format!("roll_up_members: load root library for {}", root.display())
            })?;
        let declarations = ts_resolver::resolve_ts_files(&lib.imported_modules, &Value::Null)
            .await
            .with_context(|| format!("roll_up_members: TS eval for {}", root.display()))?;
        merge_ts_declarations(&mut lib, declarations, &fs, root);
        lib.imported_modules.clear();

        lib.permissions = stack_model::Permissions::default();
        lib.hooks = Vec::new();

        let content_hash = descriptor::extension_library_content_hash(&lib)
            .with_context(|| format!("roll_up_members: content hash for {}", root.display()))?;
        member_descriptors.push(ExtensionDescriptor {
            source: ExtSource::Space,
            space_root: root.clone(),
            descriptor: lib,
            content_hash,
        });
    }

    // 2. Cache tier: the union of every member's own lock.
    let default_cache_root;
    let resolved_cache_root: &Path = match cache_root {
        Some(p) => p,
        None => {
            default_cache_root = ShipHome::from_env().cache_dir().to_path_buf();
            &default_cache_root
        }
    };
    let mut extensions: Vec<ExtensionDescriptor> = Vec::new();
    for root in member_roots {
        let lock_path = root.join("ship.lock");
        let bundles =
            ship_store::dep_skills::load_cached_extensions(&lock_path, Some(resolved_cache_root))
                .with_context(|| {
                    format!("roll_up_members: cached extensions for {}", lock_path.display())
                })?;
        for bundle in &bundles {
            extensions.push(ExtensionDescriptor {
                source: ExtSource::Cache,
                space_root: root.clone(),
                descriptor: stack_model::ExtensionLibrary {
                    agent_profiles: bundle.agent_profiles.clone(),
                    skills: bundle.skills.clone(),
                    views: bundle.views.clone(),
                    ..Default::default()
                },
                content_hash: bundle.content_hash.clone(),
            });
        }
    }

    // 3. Named permission presets are PULL definitions (agents reference
    //    them by name) and must pool across members — but `compose_space`
    //    passes config fields through from the FIRST Space descriptor
    //    verbatim. Pool every member's presets into the first descriptor,
    //    first-wins per name in member (registration) order — the same
    //    collision law the per-root harvest applies.
    if !member_descriptors.is_empty() {
        let mut pooled = std::collections::BTreeMap::new();
        for desc in &member_descriptors {
            for (name, preset) in &desc.descriptor.permissions_presets {
                pooled.entry(name.clone()).or_insert_with(|| preset.clone());
            }
        }
        member_descriptors[0].descriptor.permissions_presets = pooled;
    }

    // 4. Members compose after the cache tier, in caller order.
    extensions.extend(member_descriptors);

    let rollup_inputs = RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    };
    let mut composed =
        stack_model::compose_space(&rollup_inputs).context("roll_up_members: compose_space")?;
    composed.library.rollup_inputs = Some(rollup_inputs);
    Ok(composed)
}
