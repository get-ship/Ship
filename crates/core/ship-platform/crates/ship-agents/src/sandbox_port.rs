//! Ship host implementation of the bridge [`SandboxPort`] seam.
//!
//! All concrete sandbox policy (isolation decision, launch wrapping, sidecar
//! runtime preparation, scoped token mint/drop, dispatcher registration) lives
//! here — ship-side, not in the fleet runtime crate. The bridge executor only
//! spawns processes and talks to the trait.

use Future;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bridge::dispatch_ports::{LaunchCommand, SessionContext};
use bridge::sandbox_port::{PreparedSandbox, SandboxPort};
use bridge::SandboxOwner;
use bridge::ProcessRuntime;
use ship_isolation::tier::IsolationProfile;

/// Ship implementation of [`SandboxPort`].
#[derive(Debug, Default)]
pub struct ShipSandboxPort;

#[async_trait]
impl SandboxPort for ShipSandboxPort {
    async fn prepare_launch(
        &self,
        ctx: &SessionContext,
        launch: &mut LaunchCommand,
    ) -> Result<PreparedSandbox> {
        let mut out = PreparedSandbox::default();

        // The agent descriptor has no isolation field today; future overrides
        // will be folded in from the `SpawnPolicy` role map on the dispatch wire.
        let agent_override: Option<IsolationProfile> = None;
        let decision = ship_isolation::decision::decide(agent_override.as_ref(), None);
        let enforce = ship_isolation::decision::enforcement_enabled();

        if let Ok(d) = &decision {
            tracing::info!(
                slug = ctx.slug,
                strategy = %d.resolved.strategy,
                tier = %d.resolved.tier,
                downgraded = d.resolved.downgraded,
                enforce,
                "isolation decision"
            );
            out.env.push((
                "SHIP_ISOLATION_TIER".into(),
                d.resolved.tier.to_string(),
            ));
            out.env.push((
                "SHIP_ISOLATION_STRATEGY".into(),
                d.resolved.strategy.to_string(),
            ));

            let mut active = "0";
            if enforce {
                use ship_isolation::harness::{HarnessSandboxMechanism, tier1_mechanism_for};
                use ship_isolation::settings::{WrapContext, wrap_launch};
                use ship_isolation::IsolationStrategy;

                let harness_id = ctx.launch.harness.as_deref().unwrap_or("claude");
                let resolved_strategy = match d.resolved.strategy {
                    IsolationStrategy::HarnessSandbox => {
                        match tier1_mechanism_for(harness_id, d.capabilities.os) {
                            HarnessSandboxMechanism::Native { .. } => IsolationStrategy::HarnessNative,
                            HarnessSandboxMechanism::SrtWrapper { .. } => IsolationStrategy::SrtWrapper,
                            HarnessSandboxMechanism::Unavailable { .. } => IsolationStrategy::Noop,
                        }
                    }
                    other => other,
                };

                match resolved_strategy {
                    IsolationStrategy::HarnessNative => {
                        let mech = tier1_mechanism_for(harness_id, d.capabilities.os);
                        let flag = match &mech {
                            HarnessSandboxMechanism::Native { flag, .. } => *flag,
                            _ => None,
                        };
                        if let Some(w) = wrap_launch(
                            IsolationStrategy::HarnessNative,
                            std::path::PathBuf::from(&launch.bin),
                            launch.args.clone(),
                            &WrapContext {
                                harness_native_flag: flag,
                                ..Default::default()
                            },
                        ) {
                            launch.bin = w.bin.to_string_lossy().into_owned();
                            launch.args = w.args;
                            active = "1";
                            tracing::info!(
                                slug = ctx.slug,
                                harness = harness_id,
                                flag = ?flag,
                                "isolation enforced: HarnessNative"
                            );
                        }
                    }
                    IsolationStrategy::SrtWrapper => {
                        let bun = which_bin("bun");
                        let srt = resolve_srt_entry();
                        let settings_path = ctx.work_dir.join(".artifacts").join(format!(
                            "srt-{}.json",
                            ctx.slug.replace('/', "_")
                        ));
                        if let (Some(bun_path), Some(srt_path)) = (bun.as_ref(), srt.as_ref()) {
                            if let Err(e) = write_srt_settings(&settings_path, &ctx.work_dir) {
                                tracing::warn!(
                                    slug = ctx.slug,
                                    error = %e,
                                    "srt settings write failed; running unwrapped"
                                );
                            } else if let Some(w) = wrap_launch(
                                IsolationStrategy::SrtWrapper,
                                std::path::PathBuf::from(&launch.bin),
                                launch.args.clone(),
                                &WrapContext {
                                    bun_bin: Some(bun_path),
                                    srt_entry: Some(srt_path),
                                    srt_settings_path: Some(&settings_path),
                                    harness_native_flag: None,
                                },
                            ) {
                                launch.bin = w.bin.to_string_lossy().into_owned();
                                launch.args = w.args;
                                active = "1";
                                tracing::info!(
                                    slug = ctx.slug,
                                    "isolation enforced: SrtWrapper"
                                );
                            }
                        } else {
                            tracing::warn!(
                                slug = ctx.slug,
                                bun = bun.is_some(),
                                srt = srt.is_some(),
                                "SrtWrapper enforce requested but bun / srt missing"
                            );
                        }
                    }
                    IsolationStrategy::Noop => {}
                    IsolationStrategy::DockerEngine => {
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            launch,
                            "http",
                            |o| Some(ship_isolation::backends::Docker::container_reachable_url(o)),
                            || prepare_docker_sandbox(&ctx),
                        )
                        .await?
                        {
                            out.remote_runtime = Some(runtime);
                            out.scoped_token = token;
                            active = "1";
                        }
                    }
                    #[cfg(target_os = "linux")]
                    IsolationStrategy::Firecracker => {
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            launch,
                            "vsock",
                            |_| Some("http://127.0.0.1:8088".to_string()),
                            || prepare_firecracker_sandbox(&ctx),
                        )
                        .await?
                        {
                            out.remote_runtime = Some(runtime);
                            out.scoped_token = token;
                            active = "1";
                        }
                    }
                    IsolationStrategy::LimaVm => {
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            launch,
                            "http",
                            |o| Some(ship_isolation::backends::Lima::guest_reachable_url(o)),
                            || prepare_lima_sandbox(&ctx),
                        )
                        .await?
                        {
                            out.remote_runtime = Some(runtime);
                            out.scoped_token = token;
                            active = "1";
                        }
                    }
                    IsolationStrategy::Wsl2 => {
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            launch,
                            "http",
                            |o| Some(o.to_string()),
                            || prepare_wsl_sandbox(&ctx),
                        )
                        .await?
                        {
                            out.remote_runtime = Some(runtime);
                            out.scoped_token = token;
                            active = "1";
                        }
                    }
                    other => {
                        return Err(anyhow::anyhow!(
                            "isolation.enforce is on but strategy '{}' has no wired \
                             spawn backend — refusing to run rather than execute \
                             unsandboxed. Pick a supported strategy or wire the backend.",
                            other
                        ));
                    }
                }
            }
            out.env.push(("SHIP_ISOLATION_ACTIVE".into(), active.into()));
        } else if let Err(e) = &decision {
            tracing::warn!(
                slug = ctx.slug,
                error = %e,
                "isolation decision failed — agent will run unsandboxed"
            );
            out.env.push(("SHIP_ISOLATION_ACTIVE".into(), "0".into()));
        }

        Ok(out)
    }

    async fn register_actor(&self,
        ctx: &SessionContext,
        actor_id: &str,
    ) {
        let agent_override: Option<IsolationProfile> = None;
        let decision = ship_isolation::decision::decide(agent_override.as_ref(), None);
        if let Ok(d) = &decision {
            let project_key = ctx.work_dir.to_string_lossy().to_string();
            let isolation_profile = IsolationProfile {
                tier: Some(d.resolved.tier),
                strategy: Some(d.resolved.strategy),
                policy: None,
            };
            let dispatcher = ship_device::sandbox::dispatcher();
            match dispatcher
                .ensure_default_sandbox(project_key, isolation_profile)
                .await
            {
                Ok(sandbox_id) => {
                    if let Err(e) = dispatcher.register_actor(&sandbox_id, actor_id).await {
                        tracing::warn!(
                            slug = ctx.slug,
                            actor_id,
                            "sandbox dispatcher register_actor failed: {e}"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        slug = ctx.slug,
                        "sandbox dispatcher ensure_default_sandbox failed: {e}"
                    );
                }
            }
        }
    }

    async fn notify_actor_exit(&self,
        actor_id: &str,
    ) {
        if let Err(e) = ship_device::sandbox::dispatcher().notify_actor_exit(actor_id).await {
            tracing::debug!(
                actor_id,
                "sandbox dispatcher notify_actor_exit: {e}"
            );
        }
    }

    async fn drop_scoped_token(&self, token: &str) {
        if let Some(registry) = ship_auth::scopes::session_registry() {
            ship_auth::scopes::drop_session(&registry, token).await;
        }
    }
}

/// `which`-style lookup. Returns the first match on `$PATH`, or `None`.
fn which_bin(name: &str) -> Option<std::path::PathBuf> {
    let which = if cfg!(target_os = "windows") { "where" } else { "which" };
    let out = std::process::Command::new(which).arg(name).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let first = s.lines().next()?.trim();
    if first.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(first))
    }
}

fn resolve_srt_entry() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let p = std::path::PathBuf::from(home)
        .join(".bun/install/global/node_modules/@anthropic-ai/sandbox-runtime");
    if p.is_dir() { Some(p) } else { None }
}

fn write_srt_settings(
    path: &std::path::Path,
    work_dir: &std::path::Path,
) -> anyhow::Result<()> {
    use bridge::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut policy = SandboxPolicy::workspace_default(work_dir.to_path_buf());
    policy.mounts = vec![FsMount {
        host_path: work_dir.to_path_buf(),
        sandbox_path: work_dir.to_path_buf(),
        mode: MountMode::ReadWrite,
    }];
    policy.network = NetworkPolicy::Allowlist {
        hosts: vec!["api.anthropic.com".into(), "console.anthropic.com".into()],
    };
    let rendered = ship_isolation::backends::render_settings(&policy);
    std::fs::write(path, serde_json::to_vec_pretty(&rendered)?)?;
    Ok(())
}

struct BackendPrepared {
    isolator: Arc<dyn bridge::Isolator>,
    sandbox: bridge::SandboxHandle,
    endpoint: Option<String>,
}

fn resolve_sidecar_bin() -> std::path::PathBuf {
    if let Ok(s) = ship_isolation::decision::strategies_settings()
        && !s.docker.sidecar_bin.is_empty()
    {
        return std::path::PathBuf::from(s.docker.sidecar_bin);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("ship-sidecar");
        if candidate.is_file() {
            return candidate;
        }
    }
    std::path::PathBuf::from("/usr/local/bin/ship-sidecar")
}

async fn prepare_docker_sandbox(
    ctx: &SessionContext,
) -> Result<BackendPrepared, bridge::IsolatorError> {
    use ship_isolation::backends::{Docker, DockerConfig};
    use bridge::{FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy};

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(bridge::IsolatorError::Other)?;
    let cfg = DockerConfig {
        engine_bin: settings.docker.engine_bin.clone(),
        image: settings.docker.image.clone(),
        name_prefix: format!(
            "{}-{}",
            settings.docker.name_prefix,
            ctx.slug.replace('/', "_")
        ),
        sidecar_bin: resolve_sidecar_bin(),
    };
    let isolator: Arc<dyn bridge::Isolator> = Arc::new(Docker::new(cfg));
    isolator.available().await?;

    let mut policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    policy.mounts = vec![FsMount {
        host_path: ctx.work_dir.clone(),
        sandbox_path: ctx.work_dir.clone(),
        mode: MountMode::ReadWrite,
    }];
    policy.limits = ResourceLimits {
        memory_bytes: settings.docker.resources.memory_bytes,
        cpu_quota_micros: settings.docker.resources.cpu_quota_micros,
        pids_max: settings.docker.resources.pids_max,
    };
    let allowlist = ctx
        .launch
        .harness
        .as_deref()
        .map(ship_isolation::harness::HarnessSandboxMechanism::default_allowlist)
        .unwrap_or_default();
    policy.network = if allowlist.is_empty() {
        NetworkPolicy::Open
    } else {
        NetworkPolicy::Allowlist {
            hosts: allowlist.into_iter().map(|s| s.to_string()).collect(),
        }
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Docker::sidecar_endpoint(&sandbox);
    Ok(BackendPrepared { isolator, sandbox, endpoint })
}

#[cfg(target_os = "linux")]
async fn prepare_firecracker_sandbox(
    ctx: &SessionContext,
) -> Result<BackendPrepared, bridge::IsolatorError> {
    use ship_isolation::backends::{Firecracker, FirecrackerConfig};
    use bridge::{FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy};

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(bridge::IsolatorError::Other)?;
    let fc = &settings.firecracker;
    let firecracker_bin = if !fc.bin.is_empty() {
        std::path::PathBuf::from(&fc.bin)
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join("firecracker")))
            .filter(|p| p.is_file())
            .unwrap_or_else(|| std::path::PathBuf::from("firecracker"))
    };
    let app_plane_daemon = ship_isolation::decision::daemon_origin().and_then(|o| {
        o.strip_prefix("http://")
            .or_else(|| o.strip_prefix("https://"))
            .map(|s| s.trim_end_matches('/').to_string())
    });
    let cfg = FirecrackerConfig {
        firecracker_bin,
        kernel: std::path::PathBuf::from(&fc.kernel),
        rootfs: std::path::PathBuf::from(&fc.rootfs),
        app_plane_daemon,
    };
    let isolator: Arc<dyn bridge::Isolator> = Arc::new(Firecracker::new(cfg));
    isolator.available().await?;

    let mut policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    policy.mounts = vec![FsMount {
        host_path: ctx.work_dir.clone(),
        sandbox_path: ctx.work_dir.clone(),
        mode: MountMode::ReadWrite,
    }];
    policy.limits = ResourceLimits {
        memory_bytes: fc.resources.memory_bytes,
        cpu_quota_micros: fc.resources.cpu_quota_micros,
        pids_max: fc.resources.pids_max,
    };
    policy.network = NetworkPolicy::DenyAll;

    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Firecracker::sidecar_endpoint(&sandbox);
    Ok(BackendPrepared { isolator, sandbox, endpoint })
}

async fn prepare_lima_sandbox(
    ctx: &SessionContext,
) -> Result<BackendPrepared, bridge::IsolatorError> {
    use ship_isolation::backends::{Lima, LimaConfig};
    use bridge::SandboxPolicy;

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(bridge::IsolatorError::Other)?;
    let lima = &settings.lima;
    let cfg = LimaConfig {
        limactl_bin: std::path::PathBuf::from(&lima.bin),
        instance: lima.instance.clone(),
        sidecar_path: lima.sidecar_path.clone(),
    };
    let isolator: Arc<dyn bridge::Isolator> = Arc::new(Lima::new(cfg));
    isolator.available().await?;

    let policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Lima::sidecar_endpoint(&sandbox);
    Ok(BackendPrepared { isolator, sandbox, endpoint })
}

async fn prepare_wsl_sandbox(
    ctx: &SessionContext,
) -> Result<BackendPrepared, bridge::IsolatorError> {
    use ship_isolation::backends::{Wsl, WslConfig};
    use bridge::SandboxPolicy;

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(bridge::IsolatorError::Other)?;
    let wsl = &settings.wsl;
    let cfg = WslConfig {
        wsl_bin: std::path::PathBuf::from(&wsl.bin),
        distro: wsl.distro.clone(),
        sidecar_path: wsl.sidecar_path.clone(),
    };
    let isolator: Arc<dyn bridge::Isolator> = Arc::new(Wsl::new(cfg));
    isolator.available().await?;

    let policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Wsl::sidecar_endpoint(&sandbox);
    Ok(BackendPrepared { isolator, sandbox, endpoint })
}

async fn mint_scoped_token(
    ctx: &SessionContext,
    launch: &mut LaunchCommand,
) -> Option<String> {
    match ship_auth::scopes::session_registry() {
        Some(registry) => {
            match ship_auth::scopes::mint_session(&registry,
                ctx.agent.clone(),
                vec![],
                None,
            )
            .await
            {
                Ok(token) => {
                    launch.env.insert("SHIP_TOKEN".into(), token.clone());
                    Some(token)
                }
                Err(e) => {
                    tracing::warn!(slug = ctx.slug, "sandbox token mint failed: {e}");
                    None
                }
            }
        }
        None => {
            tracing::warn!(
                slug = ctx.slug,
                "no session registry installed; sandboxed actor has no scoped token"
            );
            None
        }
    }
}

async fn wire_sidecar_sandbox<F, Fut>(
    ctx: &SessionContext,
    tier: ship_isolation::tier::IsolationTier,
    strategy: ship_isolation::IsolationStrategy,
    launch: &mut LaunchCommand,
    scheme: &'static str,
    daemon_url: fn(&str) -> Option<String>,
    prepare: F,
) -> Result<Option<(Arc<dyn ProcessRuntime>, Option<String>)>>
where
    F: FnOnce() -> Fut,
    Fut: Future<
        Output = Result<BackendPrepared, bridge::IsolatorError>,
    >,
{
    let project_key = ctx.work_dir.to_string_lossy().to_string();
    let dispatcher = ship_device::sandbox::dispatcher();
    let isolation_profile = IsolationProfile {
        tier: Some(tier),
        strategy: Some(strategy),
        policy: None,
    };
    let sandbox_id = match dispatcher
        .ensure_default_sandbox(project_key, isolation_profile)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(
                slug = ctx.slug,
                error = %e,
                "ensure_default_sandbox failed for {strategy}; running unwrapped"
            );
            return Ok(None);
        }
    };

    let existing = dispatcher.endpoint_for(&sandbox_id).await.and_then(|e| match e {
        ship_device::sandbox::RuntimeEndpoint::Remote { addr, .. } => Some(addr),
        _ => None,
    });
    let endpoint = if let Some(addr) = existing {
        addr
    } else {
        let prepared = prepare()
            .await
            .map_err(|e| anyhow::anyhow!("{strategy} prepare: {e}"))?;
        let Some(endpoint) = prepared.endpoint.clone() else {
            return Err(anyhow::anyhow!(
                "{strategy} prepared without a sidecar endpoint"
            ));
        };
        let owner = Arc::new(bridge::SandboxOwner::new(
            prepared.isolator.clone(),
            prepared.sandbox.clone(),
        ));
        if let Err(e) = dispatcher.attach_sandbox_owner(&sandbox_id, owner).await {
            tracing::warn!(slug = ctx.slug, "attach_sandbox_owner failed: {e}");
        }
        if let Err(e) = dispatcher
            .mark_remote_running(&sandbox_id, endpoint.clone(), scheme)
            .await
        {
            tracing::warn!(slug = ctx.slug, "mark_remote_running failed: {e}");
        }
        endpoint
    };

    let runtime = ship_isolation::runtime::SidecarRuntime::shared(endpoint);
    if let Some(origin) = ship_isolation::decision::daemon_origin() {
        if let Some(url) = daemon_url(&origin) {
            launch.env.insert("SHIP_DAEMON_URL".into(), url);
        }
    }
    let minted = mint_scoped_token(ctx, launch).await;
    tracing::info!(
        slug = ctx.slug,
        sandbox_id = %sandbox_id.0,
        strategy = %strategy,
        "isolation enforced: sidecar over v1"
    );
    Ok(Some((runtime, minted)))
}

#[cfg(test)]
mod tests {
    use bridge::dispatch_ports::LaunchProfile;

    use super::*;

    // Re-created from the ship-code mint_token_tests: the scoped-token
    // mint/drop path is ship-side wiring, not fleet runtime code.
    #[tokio::test]
    async fn scoped_token_round_trips_through_ship_auth() {
        let registry = ship_auth::scopes::session_registry();
        if registry.is_none() {
            // In most test environments the auth registry is installed by
            // shipd init; skip if not available.
            return;
        }
        let Some(registry) = ship_auth::scopes::session_registry() else {
            // In most test environments the auth registry is installed by
            // shipd init; skip if not available.
            return;
        };
        let ctx = SessionContext {
            session_id: "token-test".into(),
            slug: "token-slug".into(),
            agent: "test-agent".into(),
            branch: "main".into(),
            spec_path: ".artifacts/job-spec.md".into(),
            spec_content: None,
            work_dir: std::env::temp_dir(),
            env: Default::default(),
            launch: LaunchProfile::default(),
            exit_tx: None,
            step: None,
            invocation: bridge::dispatch_ports::InvocationMode::Print,
            keepalive: false,
            repo_path: None,
            base_ref: None,
        };
        let mut launch = LaunchCommand {
            bin: "claude".into(),
            args: vec![],
            env: Default::default(),
        };
        let token = mint_scoped_token(&ctx, &mut launch).await;
        assert!(
            token.is_some(),
            "mint_scoped_token should succeed when a registry is installed"
        );
        assert_eq!(
            launch.env.get("SHIP_TOKEN"),
            token.as_ref(),
            "SHIP_TOKEN env must equal the minted token"
        );
        ship_auth::scopes::drop_session(
            &registry,
            token.as_ref().unwrap(),
        ).await;
        // Dropping must succeed; the public validate API is not exposed by
        // ship_auth::scopes, so we assert the token was non-empty and the drop
        // did not panic.
        assert!(!token.as_ref().unwrap().is_empty());
    }
}
