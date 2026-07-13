//! omp CLI adapter — the `@oh-my-pi/pi-coding-agent` fork. Same CLI family as
//! `pi` (identical `--mode json` headless stream and flags), just a different
//! binary name. This adapter is a thin delegation to [`PiAdapter`]: every
//! method forwards to pi, and the only rewrite is the launch binary
//! (`pi` → `omp`). Keeping it a delegation (not a fork of pi's logic) means
//! pi's flag/parse behavior stays the single source of truth.

use agent_driver::dispatch_ports::{LaunchCommand, LaunchProfile};
use agent_driver::{AgentDriver, HarnessCapabilities, HarnessEvent, ResumeError};

use super::pi::PiAdapter;

/// Rewrite a pi-built command to launch the `omp` binary instead. The only
/// difference between the two harnesses is the executable name.
fn rebin(mut c: LaunchCommand) -> LaunchCommand {
    c.bin = "omp".into();
    c
}

pub struct OmpAdapter;

#[async_trait::async_trait]
impl AgentDriver for OmpAdapter {
    fn id(&self) -> &str { "omp" }

    fn matches(&self, id: &str) -> bool { id == "omp" }

    fn caps(&self) -> HarnessCapabilities { PiAdapter.caps() }

    fn build_interactive(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        rebin(PiAdapter.build_interactive(slug, profile))
    }

    fn build_headless(&self, slug: &str, profile: &LaunchProfile) -> LaunchCommand {
        rebin(PiAdapter.build_headless(slug, profile))
    }

    fn build_resume_args(
        &self,
        slug: &str,
        session_id: &str,
        profile: &LaunchProfile,
    ) -> Result<LaunchCommand, ResumeError> {
        PiAdapter.build_resume_args(slug, session_id, profile).map(rebin)
    }

    // omp's interactive prompt renders the pi glyph (`π`) — pi-coding-agent
    // theme `icon.pi` — followed by a space; pi's is the chevron (`›`). Same
    // delegate-to-pi shape as the rest of the adapter, but the ready marker
    // is the one field the two harnesses do NOT share. See
    // c-2026-07-05T12-36-00-000Z-b2c3d4e5 — matching pi's pattern made
    // every omp dispatch time out at the 60s readiness poll.
    fn ready_pattern(&self) -> &str { "π " }
    fn autostart_input(&self) -> &str { PiAdapter.autostart_input() }
    fn exit_command(&self) -> &str { PiAdapter.exit_command() }

    fn parse_output(&self, line: &str) -> Option<HarnessEvent> {
        PiAdapter.parse_output(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_profile() -> LaunchProfile {
        LaunchProfile { persist_session: true, ..Default::default() }
    }

    #[test]
    fn id_and_binary_are_omp() {
        assert_eq!(OmpAdapter.id(), "omp");
        // binary() defaults to id() — the executable on PATH is `omp`.
        assert_eq!(OmpAdapter.binary(), "omp");
    }

    #[test]
    fn matches_only_omp() {
        assert!(OmpAdapter.matches("omp"));
        assert!(!OmpAdapter.matches("pi"));
    }

    #[test]
    fn caps_mirror_pi() {
        assert_eq!(OmpAdapter.caps(), PiAdapter.caps());
    }

    #[test]
    fn interactive_rebins_to_omp_keeping_pi_args() {
        let cmd = OmpAdapter.build_interactive("my-job", &default_profile());
        let pi_cmd = PiAdapter.build_interactive("my-job", &default_profile());
        assert_eq!(cmd.bin, "omp");
        assert_eq!(cmd.args, pi_cmd.args);
        assert_eq!(cmd.env.get("SHIP_ACTOR_ID").unwrap(), "my-job");
    }

    #[test]
    fn headless_rebins_to_omp_keeping_pi_args() {
        let profile = LaunchProfile {
            autostart_prompt: Some("do the thing".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let cmd = OmpAdapter.build_headless("slug-x", &profile);
        let pi_cmd = PiAdapter.build_headless("slug-x", &profile);
        assert_eq!(cmd.bin, "omp");
        assert_eq!(cmd.args, pi_cmd.args);
        assert!(cmd.args.contains(&"--print".to_string()));
    }

    #[test]
    fn resume_rebins_to_omp() {
        let cmd = OmpAdapter
            .build_resume_args("s", "019e49eb-8a84", &default_profile())
            .expect("omp supports resume via pi");
        assert_eq!(cmd.bin, "omp");
        assert!(cmd.args.contains(&"--session".to_string()));
    }

    #[test]
    fn parse_output_delegates_to_pi() {
        let line = r#"{"type":"session","version":3,"id":"abc-123","cwd":"/x"}"#;
        assert_eq!(OmpAdapter.parse_output(line), PiAdapter.parse_output(line));
        assert!(OmpAdapter.parse_output("not json").is_none());
    }

    // The omp TUI prompts with the pi glyph (`π`) followed by a space — NOT
    // the chevron (`›`) that pi uses. The harness-launcher polls a tmux pane
    // for this exact marker before firing the autostart prompt; if the
    // pattern is wrong, every omp dispatch times out
    // (c-2026-07-05T12-36-00-000Z-b2c3d4e5). This test pins the per-harness
    // discipline: omp's ready marker is its own — not delegated to pi.
    #[test]
    fn ready_pattern_is_omp_specific_pi_glyph() {
        let omp_pattern = OmpAdapter.ready_pattern();
        let pi_pattern = PiAdapter.ready_pattern();
        assert_ne!(
            omp_pattern, pi_pattern,
            "omp must have its own ready pattern, not reuse pi's \
             (the autostart race in c-2026-07-05T12-36-00-000Z-b2c3d4e5 \
              started because these were equal)"
        );
        // omp's TUI renders `π ` (pi glyph + space) at the input prompt
        // (theme `icon.pi` defaults to U+03C0). The substring must contain
        // the pi glyph so `wait_for_ready` polls match at the actual prompt
        // rather than at any other place the chevron might appear.
        assert!(
            omp_pattern.contains('\u{03C0}'),
            "omp ready pattern must contain the pi glyph, got {omp_pattern:?}"
        );
    }
}
