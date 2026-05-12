use super::super::writer::Writer;

pub(super) fn emit_view_export(w: &mut Writer, body: Writer) {
    w.line("export default view((ctx) => {");
    w.push_str(&body.into_string());
    w.line("});");
}
