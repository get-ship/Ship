//! Phase E-2 — render a single control-flow boundary's body against a
//! caller-supplied initial scope.
//!
//! The attach runtime POSTs the current state/data envelopes plus a
//! boundary key (`b<N>`) to the daemon's `/_ship-set/render-fragment/
//! <view>` endpoint when a control-flow signal flips. The daemon calls
//! this function. The returned HTML is the new body for the boundary
//! — no surrounding `<sui-{kind} key="bN">` wrapper, because the
//! runtime is swapping the wrapper's children, not the wrapper itself.
//!
//! ## How it works
//!
//! 1. `scope::build_with_overrides` merges the caller's JSON overrides
//!    onto the IR's first-paint defaults so the walker evaluates the
//!    same expressions the runtime would have evaluated.
//! 2. A fragment-mode walker (`Walker::new_fragment`) starts in silent
//!    mode and walks the entire IR. Its key counter stays in lockstep
//!    with the registry (same invariant as the full render).
//! 3. The control-flow renderers detect when the boundary they're about
//!    to emit matches the requested target key, suppress their wrapper,
//!    flip the walker visible for the body emission, then go silent
//!    again. The rest of the IR walks silently.
//! 4. The output buffer holds exactly the boundary body's HTML.

use serde_json::Value;

use crate::diagnostics::CompileError;
use crate::ir::Program;
use crate::keys::{BindingKind, BindingRegistry};

use super::scope;
use super::walker::Walker;
use super::{RenderOpts, RenderOutput};

/// Render the body for a single control-flow boundary.
///
/// `key` must be a `b<N>` string for an `If` / `For` / `Match` / `Await`
/// binding. `init_overrides` is a JSON object whose keys match `<state>`
/// / `<data>` declaration names; values replace the IR's first-paint
/// defaults. Unknown keys are merged in too (callers can pass `route`
/// or other ambient values).
pub fn render_fragment(
    program: &Program,
    registry: &BindingRegistry,
    opts: &RenderOpts,
    init_overrides: &Value,
    key: &str,
) -> Result<RenderOutput, CompileError> {
    let _ = opts; // reserved for view-name diagnostics in later phases.

    let entry = registry.iter().find(|b| b.key == key).ok_or_else(|| {
        CompileError::new(format!("render_fragment: unknown binding key: {key}"))
    })?;

    let is_cf = matches!(
        entry.kind,
        BindingKind::If | BindingKind::For | BindingKind::Match | BindingKind::Await
    );
    if !is_cf {
        return Err(CompileError::new(format!(
            "render_fragment requires a control-flow key; '{}' is {}",
            key,
            entry.kind.as_marker_str()
        )));
    }

    let initial = scope::build_with_overrides(program, init_overrides);
    let init_json = initial.init.clone();
    let mut state = Walker::new_fragment(initial.scope, registry, key.to_string());
    state.walk_block(&program.template);

    if !state.target_found() {
        return Err(CompileError::new(format!(
            "render_fragment: registry has '{key}' but the walker never reached it"
        )));
    }

    let html = state.into_html();
    Ok(RenderOutput { html, init: init_json })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings;
    use crate::parse::parse;
    use serde_json::json;

    fn opts() -> RenderOpts {
        RenderOpts { view_name: "test".into() }
    }

    const AGENTS_SRC: &str = r#"<data name="agents" source="ship.agents.summaries()" live />
<state name="items" value="[]" />
<div>
  {#if agents().kind === 'ok'}
    <p data-test="ok">ok branch</p>
  {:else if agents().kind === 'error'}
    <p data-test="err">err branch</p>
  {:else}
    <p data-test="loading">loading branch</p>
  {/if}
</div>"#;

    #[test]
    fn loading_branch_renders_for_target_if() {
        let program = parse(AGENTS_SRC).expect("parse");
        let registry = bindings::build(&program);
        let key = registry
            .iter()
            .find(|b| matches!(b.kind, BindingKind::If))
            .unwrap()
            .key
            .clone();
        let init = json!({ "agents": { "kind": "loading" } });
        let out = render_fragment(&program, &registry, &opts(), &init, &key)
            .expect("render_fragment");
        assert!(
            out.html.contains(r#"data-test="loading""#),
            "expected loading branch body, got: {}",
            out.html
        );
        // No outer wrapper in the fragment output.
        assert!(
            !out.html.contains("<sui-if key="),
            "fragment must not include the boundary wrapper: {}",
            out.html
        );
    }

    #[test]
    fn ok_branch_renders_when_envelope_flips() {
        let program = parse(AGENTS_SRC).expect("parse");
        let registry = bindings::build(&program);
        let key = registry
            .iter()
            .find(|b| matches!(b.kind, BindingKind::If))
            .unwrap()
            .key
            .clone();
        let init = json!({ "agents": { "kind": "ok", "value": { "items": [] } } });
        let out = render_fragment(&program, &registry, &opts(), &init, &key)
            .expect("render_fragment");
        assert!(
            out.html.contains(r#"data-test="ok""#),
            "expected ok branch body, got: {}",
            out.html
        );
    }

    #[test]
    fn unknown_key_is_error() {
        let program = parse(AGENTS_SRC).expect("parse");
        let registry = bindings::build(&program);
        let err = render_fragment(&program, &registry, &opts(), &json!({}), "b9999")
            .expect_err("expected unknown-key error");
        assert!(err.message.contains("unknown binding key"));
    }

    #[test]
    fn non_control_flow_key_is_error() {
        let src = r#"<state name="n" value="0" /><p>{n}</p>"#;
        let program = parse(src).expect("parse");
        let registry = bindings::build(&program);
        let leaf = registry
            .iter()
            .find(|b| matches!(b.kind, BindingKind::Text))
            .unwrap()
            .key
            .clone();
        let err = render_fragment(&program, &registry, &opts(), &json!({}), &leaf)
            .expect_err("expected non-cf error");
        assert!(err.message.contains("control-flow"));
    }

    #[test]
    fn for_branch_emits_iterations_without_outer_wrapper() {
        let src = r#"<state name="xs" value="[1, 2, 3]" />
<ul>
  {#for x of xs}
    <li>{x}</li>
  {/for}
</ul>"#;
        let program = parse(src).expect("parse");
        let registry = bindings::build(&program);
        let key = registry
            .iter()
            .find(|b| matches!(b.kind, BindingKind::For))
            .unwrap()
            .key
            .clone();
        let out = render_fragment(&program, &registry, &opts(), &json!({}), &key)
            .expect("render_fragment");
        // Three <li> bodies.
        assert_eq!(out.html.matches("<li>").count(), 3, "got: {}", out.html);
        // No outer for-wrapper.
        assert!(!out.html.contains("<sui-for key="));
    }
}
