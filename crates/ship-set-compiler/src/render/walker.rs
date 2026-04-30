//! The actual IR walker. Owns the output buffer + key allocator and
//! emits HTML one instruction at a time.

use crate::ir::{AttrInstr, ElementInstr, Instr};

use super::{
    EmitDecision, KeyAllocator, control_flow_kind, decide_element,
    escape::escape_attr_value, marker_attr_name,
};

pub(crate) struct Walker {
    out: String,
    keys: KeyAllocator,
}

impl Walker {
    pub(crate) fn new() -> Self {
        Self { out: String::new(), keys: KeyAllocator::default() }
    }

    pub(crate) fn into_html(self) -> String {
        self.out
    }

    /// Walk a flat sequence of instructions.
    pub(crate) fn walk_block(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            self.walk_instr(instr);
        }
    }

    fn walk_instr(&mut self, instr: &Instr) {
        // Control-flow placeholders: emit one empty <template> with the
        // marker attribute and STOP. D-3 will recurse into the body.
        if let Some(kind) = control_flow_kind(instr) {
            let key = self.keys.alloc();
            self.out.push_str(&format!(
                "<template data-set-{kind}=\"{key}\"></template>"
            ));
            return;
        }

        match instr {
            Instr::StaticText { text, .. } => self.out.push_str(text),
            Instr::Comment { data, .. } => {
                self.out.push_str("<!--");
                self.out.push_str(data);
                self.out.push_str("-->");
            }
            Instr::DynamicText { .. } => {
                let key = self.keys.alloc();
                self.out.push_str(&format!("<!--set:text:{key}--><!--/set-->"));
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(_) | Instr::For(_) | Instr::Match(_) => {
                // Already handled by the control-flow short-circuit above.
                unreachable!("control-flow handled by control_flow_kind branch");
            }
        }
    }

    fn walk_element(&mut self, el: &ElementInstr) {
        match decide_element(el) {
            EmitDecision::Skip => {}
            EmitDecision::Void => {
                self.emit_open_tag(el, /*self_terminate=*/ false);
                // Void elements: no children, no closing tag.
            }
            EmitDecision::Normal => {
                self.emit_open_tag(el, /*self_terminate=*/ false);
                self.walk_block(&el.children);
                self.out.push_str(&format!("</{}>", el.tag));
            }
        }
    }

    /// Emit `<tag attr1="v1" attr2="v2">`. For void elements the
    /// caller must skip the closing tag.
    fn emit_open_tag(&mut self, el: &ElementInstr, _self_terminate: bool) {
        self.out.push('<');
        self.out.push_str(&el.tag);
        for attr in &el.attrs {
            self.emit_attr(attr);
        }
        self.out.push('>');
    }

    fn emit_attr(&mut self, attr: &AttrInstr) {
        if let AttrInstr::Static { name, value, .. } = attr {
            self.out.push(' ');
            self.out.push_str(name);
            // A static attribute may have an empty value (e.g. `<input disabled>`).
            // Round-trip the same shape we render for real values.
            self.out.push_str("=\"");
            self.out.push_str(&escape_attr_value(value));
            self.out.push('"');
            return;
        }

        let Some(marker_name) = marker_attr_name(attr) else {
            // Static is the only `None` case and was handled above.
            return;
        };
        let key = self.keys.alloc();
        self.out.push(' ');
        self.out.push_str(&marker_name);
        self.out.push_str("=\"");
        self.out.push_str(&key);
        self.out.push('"');
    }
}
