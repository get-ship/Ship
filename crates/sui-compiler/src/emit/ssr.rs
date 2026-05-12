use crate::keys::BindingRegistry;
use crate::ir::Program;
use crate::render::{self, RenderOpts};

/// Emit static SSR HTML for a compiled view. Walks the lowered IR with
/// the default initial scope (loading envelopes for data, empty route)
/// using the same render walker the payload endpoint uses at runtime.
///
/// Returns the body HTML fragment (without the JSON island script) —
/// suitable for writing to `<dist>/<stem>.ssr.html`.
pub fn emit_ssr_html(program: &Program, registry: &BindingRegistry, view_name: &str) -> String {
    let opts = RenderOpts { view_name: view_name.to_string() };
    render::render_split(program, registry, &opts)
        .map(|s| s.html)
        .unwrap_or_default()
}
