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
use compiler::{ComposedLibrary, ExtSource, ExtensionDescriptor, RollupInputs};
use serde_json::Value;
use ship_substrate::ShipHome;
use ship_substrate::ports::LocalFsBroker;

use crate::{descriptor, merge_ts_declarations, space_loader, ts_resolver};

/// Compose a space from explicit member stack roots.
pub async fn roll_up_members(
    member_roots: &[PathBuf],
    cache_root: Option<&Path>,
    library_root: Option<&Path>,
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

        lib.permissions = compiler::Permissions::default();
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
                descriptor: compiler::ExtensionLibrary {
                    agent_profiles: bundle.agent_profiles.clone(),
                    skills: bundle.skills.clone(),
                    views: bundle.views.clone(),
                    ..Default::default()
                },
                content_hash: bundle.content_hash.clone(),
            });
        }
    }

    // 3. Enabled tier (user-global library dir).
    let default_library_root;
    let resolved_library_root: &Path = match library_root {
        Some(p) => p,
        None => {
            default_library_root = ShipHome::from_env().library_dir();
            &default_library_root
        }
    };
    let enabled_library = compiler::load_library(resolved_library_root).with_context(|| {
        format!(
            "roll_up_members: enabled-tier load failed for {}",
            resolved_library_root.display()
        )
    })?;
    let enabled_content_hash = descriptor::extension_library_content_hash(&enabled_library)
        .context("roll_up_members: enabled-tier content hash")?;
    extensions.push(ExtensionDescriptor {
        source: ExtSource::UserGlobal,
        space_root: member_roots.first().cloned().unwrap_or_default(),
        descriptor: enabled_library,
        content_hash: enabled_content_hash,
    });

    // 4. Members compose after the lower tiers, in caller order.
    extensions.extend(member_descriptors);

    let rollup_inputs = RollupInputs {
        extensions,
        precedence: RollupInputs::default_precedence(),
    };
    let mut composed =
        compiler::compose_space(&rollup_inputs).context("roll_up_members: compose_space")?;
    composed.library.rollup_inputs = Some(rollup_inputs);
    Ok(composed)
}
