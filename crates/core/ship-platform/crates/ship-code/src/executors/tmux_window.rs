//! TmuxWindowExecutor — executes pipeline steps as tmux windows inside a
//! shared per-slug session. Output captured via `tee` into
//! `.artifacts/pipeline/NN-<step>.log`. Use for interactive-attach
//! pipelines. Not a PTY.
//!
//! A pipeline run gets one tmux session named `ship-<slug>` (full slug,
//! sanitized for tmux-forbidden chars). Each step becomes a window whose
//! internal id we track by tmux's `@N` window id (not by name) so that
//! send-keys can never collide with leftover windows of the same name
//! from prior runs. The session outlives the run so retrospection is
//! possible; a periodic sweep (not this module's concern) handles TTL.
//!
//! Fallback: `TmuxWindowExecutor::new()` returns Err on Windows or when
//! tmux is absent, and dispatch falls back to ProcessExecutor.

use crate::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, SessionContext, SessionExecutor,
};
use crate::helpers;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
#[cfg(unix)]
use libc;
use ship_harnesses::harness_adapter::{SubmitMode, adapter_for_or_default};

/// Job executor that runs each pipeline step inside a tmux window.
pub(crate) struct TmuxWindowExecutor;

/// State stored on the ExecutorHandle.
///
/// `window` is a tmux window id (`@N`) — NOT a name. Targeting by id is
/// unambiguous even if other windows in the session share a name.
struct TmuxWindowState {
    session: String,
    window: String,
}

impl TmuxWindowExecutor {
    /// Construct a TmuxWindowExecutor. Returns Err when tmux is not usable.
    pub fn new() -> Result<Self> {
        if cfg!(windows) {
            return Err(anyhow!("TmuxWindowExecutor is unavailable on Windows"));
        }
        if !tmux_available() {
            return Err(anyhow!("tmux binary not found on PATH"));
        }
        Ok(Self)
    }
}

/// tmux session name derived from a run slug. tmux disallows `.` and `:`
/// in session names — sanitize to `_`. We use the full slug (no truncation)
/// so concurrent jobs with similar-looking slug prefixes don't collide.
pub(crate) fn session_name(slug: &str) -> String {
    let sanitized: String = slug
        .chars()
        .map(|c| if c == '.' || c == ':' { '_' } else { c })
        .collect();
    format!("ship-{sanitized}")
}

/// Log file path (relative to the worktree) for a given step.
/// Convention: `.artifacts/pipeline/NN-<step>.log`.
pub(crate) fn step_log_rel(index: usize, step_id: &str) -> String {
    format!(".artifacts/pipeline/{:02}-{}.log", index + 1, step_id)
}

/// Create the step-log dir and return the tee target for HEADLESS pipeline
/// steps. Interactive sessions get no step log (the TUI owns the tty;
/// nothing is tee'd) and must not create the `.artifacts` tree in a bare
/// dispatch worktree — `.artifacts` is retired for everything but the
/// pipeline step logs, which keep it until the pipeline runtime picks
/// their new home.
pub(crate) fn prepare_step_logging(
    work_dir: &std::path::Path,
    interactive: bool,
    index: usize,
    step_id: &str,
) -> Result<Option<String>> {
    if interactive {
        return Ok(None);
    }
    std::fs::create_dir_all(work_dir.join(".artifacts/pipeline"))?;
    Ok(Some(step_log_rel(index, step_id)))
}

/// Choose the autostart text typed into the TUI pane once the harness is
/// ready (c-2026-07-05T12-36-00-000Z-b2c3d4e5). The pane is a keystroke
/// transport: a multi-line body fragments into one submitted turn per
/// line, so content is delivered by REFERENCE — a one-line pointer to the
/// spec source file (the mission-scratch path the dispatch carried) —
/// whenever one is available. A single-line prompt is typed verbatim.
/// Multi-line content with no readable pointer is undeliverable and
/// surfaces as an error rather than fragmenting silently. With no content
/// at all, the adapter's static kick keeps promptless dispatches working.
pub(crate) fn select_autostart(
    spec_path: &str,
    spec_file_exists: bool,
    autostart_prompt: Option<&str>,
    adapter_kick: &str,
) -> std::result::Result<Option<String>, String> {
    if !spec_path.is_empty() && spec_file_exists {
        return Ok(Some(format!("Read {spec_path} and execute it.")));
    }
    if let Some(p) = autostart_prompt.map(str::trim).filter(|p| !p.is_empty()) {
        if !p.contains('\n') {
            return Ok(Some(p.to_string()));
        }
        let hint = if spec_path.is_empty() {
            "pass source_spec_path so the agent can read the spec from disk".to_string()
        } else {
            format!("spec source {spec_path} does not exist on this host")
        };
        return Err(format!(
            "multi-line prompt cannot be typed into a TUI pane \
             (each newline submits a partial turn); {hint}"
        ));
    }
    if adapter_kick.is_empty() {
        Ok(None)
    } else {
        Ok(Some(adapter_kick.to_string()))
    }
}

/// Is the tmux binary on PATH and responsive to `-V`?
///
/// Retries with backoff before concluding tmux is unusable. Under heavy
/// parallel subprocess load (workspace-wide test runs, dispatching many agents
/// at once) the bare `tmux -V` intermittently fails to fork/start — a
/// transient OS-level resource hiccup, not a real "tmux is gone" signal.
/// A single probe was flaky enough to make the dispatcher mis-route to the
/// headless fallback (and to make detection disagree with itself between two
/// calls in the same code path). Four attempts with 50/100/200ms backoff make
/// a false negative vanishingly unlikely while costing nothing on the common
/// first-try success.
pub(crate) fn tmux_available() -> bool {
    fn check() -> bool {
        tmux_cmd()
            .arg("-V")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    for backoff_ms in [50u64, 100, 200] {
        if check() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
    }
    check()
}

/// Build a `tmux` command with the per-mission socket prefix.
/// Gap 4: every `tmux` call in this module routes through this
/// helper so a helmd boot with `HELMD_TMUX_SOCKET` (or
/// `SHIP_TMUX_SOCKET`) targets a separate server. The helper
/// preserves the same return type as `Cmd::new("tmux")` so the
pub(crate) fn tmux_cmd() -> ship_substrate::spawn_bounded::Cmd {
    helpers::tmux_args()
        .into_iter()
        .fold(ship_substrate::spawn_bounded::Cmd::new("tmux"), |cmd, arg| {
            cmd.arg(arg)
        })
}

/// Does a tmux session by this name already exist?
///
/// `.output()` (not `.status()`): a missing session makes tmux print
/// "can't find session: <name>" on stderr, and `.status()` inherits the
/// daemon's stderr — the probe's expected first-dispatch miss would land
/// in the daemon log looking like a delivery failure.
fn session_exists(name: &str) -> bool {
    tmux_cmd()
        .args(["has-session", "-t", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create the session (or a new window in an existing session) for this
/// step and return the tmux window id (`@N`) of the newly created window.
///
/// We use `-P -F '#{window_id}'` to capture the id on stdout. This id is
/// globally unique within a tmux server — the safe target for subsequent
/// send-keys, list-windows, and kill-window calls.
fn create_window(session: &str, step_id: &str, work_dir: &str) -> Result<String> {
    let (args, label): (&[&str], &str) = if !session_exists(session) {
        (
            &[
                "new-session",
                "-d",
                "-P",
                "-F",
                "#{window_id}",
                "-s",
                session,
                "-n",
                step_id,
                "-c",
                work_dir,
            ],
            "tmux new-session",
        )
    } else {
        (
            &[
                "new-window",
                "-d",
                "-P",
                "-F",
                "#{window_id}",
                "-t",
                session,
                "-n",
                step_id,
                "-c",
                work_dir,
            ],
            "tmux new-window",
        )
    };
    let out = tmux_cmd()
        .args(args)
        .output()
        .map_err(|e| anyhow!("{label} failed: {e}"))?;
    if !out.status.success() {
        return Err(anyhow!("{label} returned non-zero"));
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !id.starts_with('@') {
        return Err(anyhow!("{label} did not print a window id (got {id:?})"));
    }
    Ok(id)
}

/// True if `window_id` (e.g. `@29`) is still listed in `session`.
fn window_id_exists(session: &str, window_id: &str) -> bool {
    let Ok(out) = tmux_cmd()
        .args(["list-windows", "-t", session, "-F", "#{window_id}"])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|l| l == window_id)
}

/// The pid of the shell in `window_target`'s pane (e.g. `ship-foo:@29`).
/// Used to give tmux-backed actors a real pid for `/proc` liveness. Returns
/// `None` if tmux can't report it — callers fall back to activity-based
/// liveness, exactly as before.
fn pane_pid(window_target: &str) -> Option<u32> {
    let out = tmux_cmd()
        .args(["list-panes", "-t", window_target, "-F", "#{pane_pid}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .and_then(|l| l.trim().parse::<u32>().ok())
}

/// Capture the visible text of `window_target`'s pane. Returns an empty
/// string when the capture fails (closed pane, tmux error) — the caller
/// treats that as "not ready yet" and retries.
///
/// The `capture-pane` `-p` flag prints to stdout, and we grab the last
/// 200 lines (a reasonable window for any harness's startup prompt).
/// Multi-byte UTF-8 from the harness is preserved (we slice by line, not
/// by byte index, so a half-char at the boundary is impossible).
pub(crate) fn capture_pane(window_target: &str) -> String {
    let out = tmux_cmd()
        .args(["capture-pane", "-p", "-t", window_target, "-S", "-200"])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => String::new(),
    }
}

/// Pane-reader seam: lets tests inject a fake pane buffer (a `Vec<String>`
/// of consecutive capture outputs) without driving a real tmux server.
pub(crate) trait PaneReader {
    fn read(&mut self, window_target: &str) -> String;
}

/// Production pane reader — shells out to `tmux capture-pane`.
pub(crate) struct TmuxPaneReader;
impl PaneReader for TmuxPaneReader {
    fn read(&mut self, window_target: &str) -> String {
        capture_pane(window_target)
    }
}

/// Poll `window_target`'s pane until `ready_pattern` appears in the captured
/// text, or `timeout` elapses. Polls every `poll_interval` (default 250ms —
/// fast enough for a snappy harness, slow enough not to slam tmux).
///
/// Returns `Ok(())` on ready, `Err` with the elapsed time + last seen
/// (truncated) pane on timeout. The Err message includes the session
/// name when the caller passes one — that is the operator's primary
/// "where do I look?" breadcrumb when an autostart fails.
///
/// (c-2026-07-05T12-36-00-000Z-b2c3d4e5): the autostart race that the
/// retired `dispatch.sh` handled with a fixed sleep + send-keys-then-pray.
/// The poll replaces both: the harness signals its own readiness (the
/// prompt marker it actually waits for), and we send the autostart only
/// after seeing it.
pub(crate) fn wait_for_ready(
    window_target: &str,
    ready_pattern: &str,
    timeout: std::time::Duration,
    poll_interval: std::time::Duration,
    reader: &mut dyn PaneReader,
) -> Result<(), String> {
    let started = std::time::Instant::now();
    let mut last_seen = String::new();
    loop {
        let text = reader.read(window_target);
        last_seen = text.clone();
        if text.contains(ready_pattern) {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            let truncated = if last_seen.len() > 200 {
                format!("…{}", &last_seen[last_seen.len() - 200..])
            } else {
                last_seen
            };
            return Err(format!(
                "harness never reached ready pattern {ready_pattern:?} within {:?} \
                 (last pane: {truncated:?})",
                timeout
            ));
        }
        std::thread::sleep(poll_interval);
    }
}

/// Find a window id by name in the session. Returns `None` if there are
/// zero or multiple matches — on rehydration we would rather mark the job
/// failed than risk attaching to the wrong window.
fn find_unique_window_by_name(session: &str, name: &str) -> Option<String> {
    let out = tmux_cmd()
        .args([
            "list-windows",
            "-t",
            session,
            "-F",
            "#{window_id} #{window_name}",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut matches = text.lines().filter_map(|l| {
        let (id, n) = l.split_once(' ')?;
        if n == name {
            Some(id.to_string())
        } else {
            None
        }
    });
    let first = matches.next()?;
    if matches.next().is_some() {
        tracing::warn!(
            session,
            name,
            "tmux-window-executor: multiple windows share this name — refusing to rehydrate"
        );
        return None;
    }
    Some(first)
}

/// Emit `tmux set-environment` for each entry in `env` so a pane started in
/// the session inherits the value before its first command. Mirrors the
/// process-side `merge_session_env_into_launch` on `launch.env`. The
/// `runner` indirection lets tests assert the exact argv without driving a
/// real tmux server — production code passes a closure that shells out
/// to `tmux` via `ship_substrate::spawn_bounded::Cmd`.
///
/// The function is best-effort: the first failed call short-circuits the
/// rest and surfaces an error string for the caller to log. `SHIP_MISSION_SCRATCH`
/// (the helm-side mission scratch path) reaches the pane here — the same
/// seam the process executor implements.
pub(crate) fn emit_tmux_env<R, I, K, V>(
    session: &str,
    env: I,
    runner: R,
) -> std::result::Result<(), String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
    R: Fn(&[&str]) -> std::result::Result<(), String>,
{
    for (name, value) in env {
        runner(&[
            "set-environment",
            "-t",
            session,
            name.as_ref(),
            value.as_ref(),
        ])
        .map_err(|e| format!("tmux set-environment {} failed: {e}", name.as_ref()))?;
    }
    Ok(())
}

#[async_trait]
impl SessionExecutor for TmuxWindowExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle> {
        let session = session_name(&ctx.slug);
        let (step_idx, step_id) = ctx
            .step
            .as_ref()
            .map(|s| (s.index, s.id.clone()))
            .unwrap_or((0, ctx.slug.clone()));

        let work_dir = ctx
            .work_dir
            .to_str()
            .ok_or_else(|| anyhow!("worktree path is not valid UTF-8"))?
            .to_string();

        // A bare dispatch (no pipeline step) is an INTERACTIVE agent session:
        // run the harness TUI directly in the window so an operator who
        // `tmux attach`es gets a live, drivable agent. A pipeline step is
        // headless: a one-shot command tee'd to its step log.
        let interactive = ctx.step.is_none();
        // Headless steps get a log dir so `tee -a` doesn't race; interactive
        // sessions create nothing.
        let log_rel = prepare_step_logging(&ctx.work_dir, interactive, step_idx, &step_id)?;

        let window_id = create_window(&session, &step_id, &work_dir)?;
        let window_target = format!("{session}:{window_id}");

        let adapter = adapter_for_or_default(ctx.launch.harness.as_deref())
            .map_err(|e| anyhow!("tmux-window executor: {e}"))?;
        let cmd = match &ctx.launch.resume_session {
            Some(sid) => adapter
                .build_resume_args(&ctx.slug, sid, &ctx.launch)
                .map_err(|e| anyhow!("tmux-window: build_resume_args failed: {e}"))?,
            None if interactive => adapter.build_interactive(&ctx.slug, &ctx.launch),
            None => adapter.build_headless(&ctx.slug, &ctx.launch),
        };
        let shell_cmd = cmd.to_shell_string();
        // Per-dispatch env reaches the harness shell via `tmux
        // set-environment` so the pane inherits it before the first
        // send-keys. This is the tmux side of the `SHIP_MISSION_SCRATCH`
        // seam — the process executor folds the same map into
        // `launch.env`. A failure here logs a warning rather than failing
        // the dispatch (mirrors the best-effort posture the rest of
        // tmux_window takes with harness-missing). See `emit_tmux_env`
        // for the testable form.
        if let Err(e) = emit_tmux_env(&session, &ctx.env, |argv| {
            tmux_cmd()
                .args(argv)
                .status()
                .map(|_| ())
                .map_err(|e| e.to_string())
        }) {
            tracing::warn!(
                slug = ctx.slug, session,
                "tmux set-environment encountered a non-fatal failure: {e}"
            );
        }
        // Never pipe an interactive TUI through `tee` — it needs the
        // window's tty. Headless steps tee to capture their run log.
        let run_cmd = match &log_rel {
            None => shell_cmd,
            Some(log) => format!("{shell_cmd} 2>&1 | tee -a {log}"),
        };
        let status = tmux_cmd()
            .args(["send-keys", "-t", &window_target, &run_cmd, "Enter"])
            .status()
            .map_err(|e| anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow!("tmux send-keys returned non-zero"));
        }

        // Interactive path: wait for the harness to signal readiness, THEN
        // send the autostart prompt. This closes the autostart race in
        // c-2026-07-05T12-36-00-000Z-b2c3d4e5 — without the poll, the
        // autostart races the harness TUI's first paint and either gets
        // typed into a menu prompt ("can't find session: <name>") or
        // silently lost (the agent boots into the TUI at its idle prompt).
        // The poll retries rather than failing on the first probe, and
        // caps at 60s — long enough for slow harness startup (omp can
        // take >15s on first run) without blocking dispatch forever.
        if interactive {
            let ready_pattern = adapter.ready_pattern();
            if !ready_pattern.is_empty() {
                let mut reader = TmuxPaneReader;
                if let Err(msg) = wait_for_ready(
                    &window_target,
                    ready_pattern,
                    std::time::Duration::from_secs(60),
                    std::time::Duration::from_millis(250),
                    &mut reader,
                ) {
                    // Log one clear error including the session name and
                    // last-seen pane text — the operator's first question
                    // is "where do I attach to debug?", and the session
                    // name is the answer.
                    tracing::error!(slug = ctx.slug, session, window = window_id, "{msg}");
                    return Err(anyhow!(
                        "tmux-window executor: harness autostart readiness \
                         failed for session {session}: {msg}"
                    ));
                }
                // Send the autostart now that the TUI is at its idle
                // prompt. The dispatch's own content wins — a pointer to
                // the spec source file, or a single-line prompt — and the
                // adapter's static kick is the last resort. Headless steps
                // don't get a separate autostart; `build_headless` already
                // inlines the prompt into the harness command.
                let spec_file_exists = !ctx.spec_path.is_empty()
                    && std::path::Path::new(&ctx.spec_path).is_file();
                let autostart = select_autostart(
                    &ctx.spec_path,
                    spec_file_exists,
                    ctx.launch.autostart_prompt.as_deref(),
                    adapter.autostart_input(),
                )
                .map_err(|e| {
                    anyhow!("tmux-window executor: autostart for session {session}: {e}")
                })?;
                if let Some(msg) = autostart {
                    // Gap 1: harness-aware submit mode. Claude's TUI
                    // has a frame boundary between text input and the
                    // submit gesture; the inline `<msg> Enter`
                    // `tmux send-keys` burst lands as a single
                    // paste-like frame and the trailing Enter is
                    // swallowed as a paste-newline. Spaced mode splits
                    // the call: <msg> first, then a sleep, then
                    // `send-keys Enter` as a separate call. The
                    // 150ms gap is enough for the TUI to settle into
                    // a fresh input frame; omp keeps the default
                    // (Inline) and pays no cost.
                    match adapter.submit_mode() {
                        SubmitMode::Inline => {
                            let status = tmux_cmd()
                                .args(["send-keys", "-t", &window_target, &msg, "Enter"])
                                .status()
                                .map_err(|e| anyhow!("tmux autostart send-keys failed: {e}"))?;
                            if !status.success() {
                                return Err(anyhow!("tmux autostart send-keys returned non-zero"));
                            }
                        }
                        SubmitMode::Spaced => {
                            let status = tmux_cmd()
                                .args(["send-keys", "-t", &window_target, &msg])
                                .status()
                                .map_err(|e| anyhow!("tmux autostart (msg) send-keys failed: {e}"))?;
                            if !status.success() {
                                return Err(anyhow!("tmux autostart (msg) send-keys returned non-zero"));
                            }
                            // 150ms — claude's TUI refresh rate is ~60ms;
                            // 150ms leaves a clear frame boundary for the
                            // submit Enter to land in a fresh input frame.
                            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                            let status = tmux_cmd()
                                .args(["send-keys", "-t", &window_target, "Enter"])
                                .status()
                                .map_err(|e| anyhow!("tmux autostart (Enter) send-keys failed: {e}"))?;
                            if !status.success() {
                                return Err(anyhow!("tmux autostart (Enter) send-keys returned non-zero"));
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            slug = ctx.slug,
            session,
            window = window_id,
            window_name = step_id,
            log = log_rel.as_deref().unwrap_or("(interactive — no step log)"),
            "tmux-window-executor: step launched"
        );

        let slug = ctx.slug.clone();
        let session_id = ctx.session_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        let session_mon = session.clone();
        let window_mon = window_id.clone();
        std::thread::spawn(move || {
            monitor_window(&session_mon, &window_mon, &slug, &session_id, exit_tx);
        });

        // Record the pane's pid so the actor projection's /proc liveness probe
        // works for tmux steps too. Without it, pid is None and `actor.live`
        // falls back to a 90s activity timer — an idle-but-alive tmux agent
        // reads as dead. The pane pid (the shell running the tee'd agent) is
        // alive exactly while the window is, so it tracks liveness honestly.
        // Flows through `handle.pid` → `process.spawned` → the projection.
        let pid = pane_pid(&window_target);

        Ok(ExecutorHandle {
            pid,
            inner: Box::new(TmuxWindowState {
                session,
                window: window_id,
            }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return false,
        };
        window_id_exists(&state.session, &state.window)
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        // Gap 5: reap the pane's process group before killing the
        // window. `kill-window` only kills tmux; the harness
        // process and its descendants (claude's tool subagents,
        // bash subshells, etc.) survive as orphans — 17 claude
        // processes (3.3GB) were found after session kills. The
        // pane's pid IS the session leader for the harness's
        // process group (tmux's default), so `kill(-pgid, SIGTERM)`
        // reaches every descendant. Best-effort: the kill may
        if let Some(pid) = pane_pid(&target) {
            helpers::kill_process_group(pid, libc::SIGTERM);
        }
        let _ = tmux_cmd()
            .args(["kill-window", "-t", &target])
            .status();
        Ok(())
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        let state = match handle.inner.downcast_ref::<TmuxWindowState>() {
            Some(s) => s,
            None => return Ok(()),
        };
        let target = format!("{}:{}", state.session, state.window);
        let status = tmux_cmd()
            .args(["send-keys", "-t", &target, message, "Enter"])
            .status()
            .map_err(|e| anyhow!("tmux send-keys failed: {e}"))?;
        if !status.success() {
            return Err(anyhow!("tmux send-keys returned non-zero"));
        }
        Ok(())
    }

    async fn resume_handle(
        &self,
        slug: &str,
        _harness: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        let session = session_name(slug);
        // After a restart the in-memory window id is gone; look it up by
        // name. Prefer the active step's window; fall back to the slug-
        // named window used for pre-pipeline single-step jobs.
        let name = current_step
            .map(str::to_string)
            .unwrap_or_else(|| slug.to_string());
        let window = find_unique_window_by_name(&session, &name)?;
        Some(ExecutorHandle {
            pid: None,
            inner: Box::new(TmuxWindowState { session, window }),
        })
    }
}

fn monitor_window(
    session: &str,
    window_id: &str,
    slug: &str,
    session_id: &str,
    exit_tx: Option<tokio::sync::mpsc::UnboundedSender<ExitSignal>>,
) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        if !window_id_exists(session, window_id) {
            tracing::info!(
                session,
                window = window_id,
                slug,
                "tmux-window-executor: window closed, signalling exit"
            );
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal {
                    slug: slug.to_string(),
                    session_id: session_id.to_string(),
                    exit_code: None,
                    cause: ExitCause::SessionDestroyed,
                });
            }
            return;
        }
    }
}

#[cfg(test)]
#[path = "tmux_window_tests.rs"]
mod tests;
