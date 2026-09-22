// FILE_CAP_OVERRIDE: flat HarnessService leaf handlers (moved from helmd).
// One service, one file; flatten map is the bulk of the surface.
//! `ship.internal.v1.HarnessService` — leaf wire over harness-config-compiler.
//!
//! Mounted on shipd (j-02s4a; previously helmd :2469). Unauthed localhost
//! leaf like GitService so helm SPA/server clients dial without a session
//! token. DescriptorCache memoizes CompileOutput per (space, hash, harness).

use std::path::PathBuf;
use std::sync::Arc;

use super::descriptor_cache::DescriptorCache;
use connectrpc::{RequestContext, Response, Router as ConnectRouter, ServiceResult};
use ship_proto::ship::internal::v1::{
    CompileResponse, CompiledFile, Harness, HarnessService, HarnessServiceExt,
    ListHarnessesResponse, OwnedCompileRequestView, OwnedListHarnessesRequestView,
};
use ship_substrate::ShipHome;
use ship_substrate::ports::SpaceId;
use space_resolver::descriptor::canonical_content_hash;

/// Stateless handler — ListHarnesses is pure table read; Compile rolls up
/// the project tree and emits harness-native config files.
pub struct HarnessImpl;

impl HarnessService for HarnessImpl {
    #[allow(refining_impl_trait)]
    async fn list_harnesses(
        &self,
        _ctx: RequestContext,
        _req: OwnedListHarnessesRequestView,
    ) -> ServiceResult<ListHarnessesResponse> {
        let harnesses = harness_config_compiler::list_harnesses()
            .iter()
            .map(|h| Harness {
                id: h.id.to_string(),
                name: h.name.to_string(),
                supports_mcp: h.features.supports_mcp,
                supports_hooks: h.features.supports_hooks,
                supports_tool_permissions: h.features.supports_tool_permissions,
                supports_memory: h.features.supports_memory,
                mcp_config_path: h.mcp_config_path.unwrap_or_default().to_string(),
                ..Default::default()
            })
            .collect();
        Response::ok(ListHarnessesResponse {
            harnesses,
            ..Default::default()
        })
    }

    #[allow(refining_impl_trait)]
    async fn compile(
        &self,
        _ctx: RequestContext,
        req: OwnedCompileRequestView,
    ) -> ServiceResult<CompileResponse> {
        let repo_path = req.repo_path.to_string();
        if repo_path.is_empty() {
            return Err(connectrpc::ConnectError::invalid_argument(
                "repo_path is required",
            ));
        }
        let repo = PathBuf::from(&repo_path);

        let lib = space_resolver::roll_up_registered_blocking(&repo, None)
            .map(|composed| composed.library)
            .map_err(|e| {
                connectrpc::ConnectError::invalid_argument(format!(
                    "roll_up({}) failed: {e}",
                    repo.display()
                ))
            })?;
        let active_agent_override: Option<String> = if req.active_agent.is_empty() {
            None
        } else {
            Some(req.active_agent.to_string())
        };
        let mut resolved =
            stack_model::resolve_library(&lib, None, active_agent_override.as_deref());

        let space_id: Option<SpaceId> =
            ship_substrate::project::resolve_active_space(&repo).map(SpaceId::new);

        let space_rules: Vec<agent_schema::Rule> = match space_id.as_ref() {
            Some(space) => match ship_space::rules::list_space_rules(space.as_str()).await {
                Ok(rules) => {
                    if !rules.is_empty() {
                        tracing::debug!(
                            space = %space.as_str(),
                            count = rules.len(),
                            "shipd harness: appending space rules to resolved compile set",
                        );
                    }
                    rules
                        .into_iter()
                        .map(ship_space::rules::SpaceRule::into_stack_rule)
                        .collect()
                }
                Err(e) => {
                    tracing::warn!(
                        space = %space.as_str(),
                        error = %e,
                        "shipd harness: failed to read space rules; compiling without them",
                    );
                    Vec::new()
                }
            },
            None => Vec::new(),
        };

        let content_hash: String =
            canonical_content_hash(&(&lib, &active_agent_override, &space_rules)).map_err(|e| {
                connectrpc::ConnectError::invalid_argument(format!(
                    "canonical_content_hash failed: {e}"
                ))
            })?;
        resolved.rules.extend(space_rules);

        let harnesses: Vec<String> = if !req.harness.is_empty() {
            vec![req.harness.to_string()]
        } else if !resolved.harnesses.is_empty() {
            resolved.harnesses.clone()
        } else {
            vec!["claude".to_string()]
        };

        let cache = space_id.as_ref().map(|_| descriptor_cache());
        let mut files: Vec<CompiledFile> = Vec::new();
        for h in &harnesses {
            let compiled = match (space_id.as_ref(), cache.as_ref()) {
                (Some(space), Some(cache)) => Arc::clone(
                    &cache
                        .get_or_load(space, &content_hash, h, || async {
                            harness_config_compiler::compile(&resolved, h).map_err(|e| {
                                anyhow::anyhow!("compile({h}): {e}")
                            })
                        })
                        .await
                        .map_err(|e| {
                            connectrpc::ConnectError::internal(format!("compile({h}): {e}"))
                        })?,
                ),
                _ => match harness_config_compiler::compile(&resolved, h) {
                    Ok(out) => Arc::new(out),
                    Err(_) => continue,
                },
            };
            flatten_compile_output(h, &compiled, &mut files);
        }

        let dry_run = req.dry_run;
        if !dry_run {
            for f in &files {
                let target = repo.join(&f.path);
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&target, &f.content) {
                    return Err(connectrpc::ConnectError::internal(format!(
                        "write {}: {e}",
                        target.display()
                    )));
                }
            }
        }

        Response::ok(CompileResponse {
            harnesses,
            files,
            requested_grants: resolved.requested_grants,
            dry_run,
            ..Default::default()
        })
    }
}

fn flatten_compile_output(
    harness_id: &str,
    out: &harness_config_compiler::CompileOutput,
    files: &mut Vec<CompiledFile>,
) {
    if let Some(path) = &out.mcp_config_path
        && !out.mcp_servers.is_null()
        && out.mcp_servers != serde_json::json!({})
    {
        let content =
            serde_json::to_string_pretty(&out.mcp_servers).unwrap_or_else(|_| "{}".to_string());
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: format!("{content}\n"),
            ..Default::default()
        });
    }

    if let Some(content) = &out.context_content {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: context_path_for(harness_id),
            content: content.clone(),
            ..Default::default()
        });
    }

    for (path, content) in &out.skill_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }
    for (path, content) in &out.agent_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }
    for (path, content) in &out.rule_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }

    if let Some(patch) = &out.claude_settings_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".claude/settings.json".to_string(),
            content: serde_json::to_string_pretty(patch).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(lsp) = &out.claude_lsp_config {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".claude/lsp.json".to_string(),
            content: serde_json::to_string_pretty(lsp).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(patch) = &out.codex_config_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/config.toml".to_string(),
            content: patch.clone(),
            ..Default::default()
        });
    }
    if let Some(hooks) = &out.codex_hooks_json {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/hooks.json".to_string(),
            content: serde_json::to_string_pretty(hooks).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(policy) = &out.codex_policy_content {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/rules/ship.rules".to_string(),
            content: policy.clone(),
            ..Default::default()
        });
    }
    if let Some(cfg) = &out.opencode_config_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: "opencode.json".to_string(),
            content: serde_json::to_string_pretty(cfg).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(hooks) = &out.cursor_hooks_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/hooks.json".to_string(),
            content: serde_json::to_string_pretty(hooks).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(perms) = &out.cursor_cli_permissions {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/cli.json".to_string(),
            content: serde_json::to_string_pretty(perms).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
    if let Some(env) = &out.cursor_environment_json {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/environment.json".to_string(),
            content: serde_json::to_string_pretty(env).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
}

fn context_path_for(harness_id: &str) -> String {
    match harness_id {
        "claude" => "CLAUDE.md".to_string(),
        "codex" => "AGENTS.md".to_string(),
        "cursor" => ".cursor/rules/_project.md".to_string(),
        "opencode" => "AGENTS.md".to_string(),
        "pi" => "AGENTS.md".to_string(),
        _ => "AGENTS.md".to_string(),
    }
}

/// Process-wide descriptor cache for CompileOutput memoization.
/// Cache root = ShipHome::cache_dir(); opens without a kernel (content-hash
/// drift still busts hits). Same contract as the former helmd mount.
static DESCRIPTOR_CACHE: std::sync::LazyLock<DescriptorCache> = std::sync::LazyLock::new(|| {
    let cache_root = ShipHome::from_env().cache_dir().to_path_buf();
    let cache = DescriptorCache::open(cache_root, None).expect("shipd: descriptor cache open");
    let _sub = cache.spawn_invalidation_subscriber();
    cache
});

fn descriptor_cache() -> DescriptorCache {
    DESCRIPTOR_CACHE.clone()
}

/// Mount `ship.internal.v1.HarnessService` onto the Connect router.
pub(super) fn mount(router: ConnectRouter) -> ConnectRouter {
    Arc::new(HarnessImpl).register(router)
}

/// Standalone axum router for the harness wire (no auth — matches former
/// helmd leaf). Merged at the shipd root so helm clients dial ListHarnesses
/// / Compile without a session token.
pub fn axum_router() -> axum::Router {
    mount(ConnectRouter::new()).into_axum_router()
}
