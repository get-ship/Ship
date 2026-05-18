use super::super::writer::Writer;

pub(super) fn emit_view_export(w: &mut Writer, body: Writer) {
    // The view() factory returns `{ setup }`; we splice the version
    // stamp onto the same object so it travels with the default
    // export. Bun.build / esbuild tree-shake module-level
    // `export const _compilerVersion` away when no consumer touches
    // it, but every host that hydrates a view reads the default
    // export — so attaching the stamp here keeps it visible after
    // bundling without changing the view's hydration contract.
    let cv = crate::compiler_version();
    let stamp = format!(
        "{{\"cargo\":\"{}\",\"git\":\"{}\",\"dirty\":{},\"builtAt\":\"{}\"}}",
        cv.cargo, cv.git, cv.dirty, cv.built_at,
    );
    w.line("export default Object.assign(view((ctx) => {");
    w.push_str(&body.into_string());
    w.line(&format!("}}), {{ _compilerVersion: {stamp} }});"));
}
