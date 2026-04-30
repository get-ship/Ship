//! IR walker. Owns the output buffer + key allocator + scope and emits
//! HTML one instruction at a time. Phase D-3 fills in control-flow
//! bodies against the initial scope; D-2 brought renderable-subset
//! evaluation into text/attribute output, and D-1 nailed the marker
//! shapes. On evaluator failure we fall back to the marker-only shape
//! so client-side attachment still works.

use crate::ir::{AttrInstr, ElementInstr, Instr};

use super::control_flow;
use super::escape::{escape_attr_value, escape_text_value};
use super::eval::{Scope, eval, stringify, truthy};
use super::{EmitDecision, KeyAllocator, decide_element};

pub(crate) struct Walker {
    pub(super) out: String,
    pub(super) keys: KeyAllocator,
    pub(super) scope: Scope,
}

impl Walker {
    pub(crate) fn new(scope: Scope) -> Self {
        Self { out: String::new(), keys: KeyAllocator::default(), scope }
    }

    pub(crate) fn into_html(self) -> String {
        self.out
    }

    pub(crate) fn walk_block(&mut self, instrs: &[Instr]) {
        for instr in instrs {
            self.walk_instr(instr);
        }
    }

    pub(super) fn walk_instr(&mut self, instr: &Instr) {
        match instr {
            Instr::StaticText { text, .. } => self.out.push_str(text),
            Instr::Comment { data, .. } => {
                self.out.push_str("<!--");
                self.out.push_str(data);
                self.out.push_str("-->");
            }
            Instr::DynamicText { expr } => {
                let key = self.keys.alloc();
                let inner = eval(&expr.source, &self.scope)
                    .map(|v| escape_text_value(&stringify(&v)))
                    .unwrap_or_default();
                self.out.push_str(&format!("<!--set:text:{key}-->{inner}<!--/set-->"));
            }
            Instr::Element(el) => self.walk_element(el),
            Instr::If(if_instr) => control_flow::render_if(self, if_instr),
            Instr::For(for_instr) => control_flow::render_for(self, for_instr),
            Instr::Match(match_instr) => control_flow::render_match(self, match_instr),
        }
    }

    pub(super) fn walk_element(&mut self, el: &ElementInstr) {
        self.walk_element_with_extra(el, &[]);
    }

    pub(super) fn walk_element_with_extra(
        &mut self,
        el: &ElementInstr,
        extra: &[(&str, &str)],
    ) {
        match decide_element(el) {
            EmitDecision::Skip => {}
            EmitDecision::Void => self.emit_open_tag(el, extra),
            EmitDecision::Normal => {
                self.emit_open_tag(el, extra);
                self.walk_block(&el.children);
                self.out.push_str(&format!("</{}>", el.tag));
            }
        }
    }

    pub(super) fn push_raw(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn emit_open_tag(&mut self, el: &ElementInstr, extra: &[(&str, &str)]) {
        self.out.push('<');
        self.out.push_str(&el.tag);

        let class_text = self.collect_classes(&el.attrs);
        let style_text = self.collect_styles(&el.attrs);
        let first_class_pos = el.attrs.iter().position(is_class_related);
        let first_style_pos = el.attrs.iter().position(is_style_related);

        for (i, attr) in el.attrs.iter().enumerate() {
            if Some(i) == first_class_pos && !class_text.is_empty() {
                self.push_attr("class", &class_text);
            }
            if Some(i) == first_style_pos && !style_text.is_empty() {
                self.push_attr("style", &style_text);
            }
            self.emit_attr_marker(attr);
        }
        for (name, value) in extra {
            self.push_attr(name, value);
        }
        self.out.push('>');
    }

    /// Walk attributes once and gather every class fragment we want in
    /// the merged `class="..."` attribute — static `class="..."` values
    /// plus any `class:NAME={truthy expr}` toggles.
    fn collect_classes(&self, attrs: &[AttrInstr]) -> String {
        let mut classes: Vec<String> = Vec::new();
        for attr in attrs {
            match attr {
                AttrInstr::Static { name, value, .. } if name == "class" => {
                    if !value.is_empty() {
                        classes.push(value.clone());
                    }
                }
                AttrInstr::Class { name, expr, .. } => {
                    if let Ok(v) = eval(&expr.source, &self.scope) {
                        if truthy(&v) {
                            classes.push(name.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        classes.join(" ")
    }

    fn collect_styles(&self, attrs: &[AttrInstr]) -> String {
        let mut styles: Vec<String> = Vec::new();
        for attr in attrs {
            match attr {
                AttrInstr::Static { name, value, .. } if name == "style" => {
                    if !value.is_empty() {
                        styles.push(trim_trailing_semi(value));
                    }
                }
                AttrInstr::Style { name, expr, .. } => {
                    if let Ok(v) = eval(&expr.source, &self.scope) {
                        if truthy(&v) {
                            styles.push(format!("{name}: {}", stringify(&v)));
                        }
                    }
                }
                _ => {}
            }
        }
        styles.join("; ")
    }

    fn emit_attr_marker(&mut self, attr: &AttrInstr) {
        match attr {
            AttrInstr::Static { name, value, .. } => {
                if name == "class" || name == "style" {
                    return;
                }
                self.push_attr(name, value);
            }
            AttrInstr::Expr { name, expr, .. } => {
                let key = self.keys.alloc();
                if let Ok(v) = eval(&expr.source, &self.scope) {
                    self.push_attr(name, &stringify(&v));
                }
                self.push_marker(&format!("data-set-attr:{name}"), &key);
            }
            AttrInstr::Class { name, .. } => {
                let key = self.keys.alloc();
                self.push_marker(&format!("data-set-class:{name}"), &key);
            }
            AttrInstr::Style { name, .. } => {
                let key = self.keys.alloc();
                self.push_marker(&format!("data-set-style:{name}"), &key);
            }
            AttrInstr::BindValue { expr, .. } => {
                let key = self.keys.alloc();
                if let Ok(v) = eval(&expr.source, &self.scope) {
                    self.push_attr("value", &stringify(&v));
                }
                self.push_marker("data-set-bind:value", &key);
            }
            AttrInstr::BindChecked { expr, .. } => {
                let key = self.keys.alloc();
                if let Ok(v) = eval(&expr.source, &self.scope) {
                    if truthy(&v) {
                        self.out.push(' ');
                        self.out.push_str("checked");
                    }
                }
                self.push_marker("data-set-bind:checked", &key);
            }
            AttrInstr::On { event, .. } => {
                let key = self.keys.alloc();
                self.push_marker(&format!("data-set-on:{event}"), &key);
            }
            AttrInstr::DynamicAttr { .. } => {
                let key = self.keys.alloc();
                self.push_marker("data-set-attr-dynamic", &key);
            }
        }
    }

    fn push_attr(&mut self, name: &str, value: &str) {
        self.out.push(' ');
        self.out.push_str(name);
        self.out.push_str("=\"");
        self.out.push_str(&escape_attr_value(value));
        self.out.push('"');
    }

    fn push_marker(&mut self, name: &str, key: &str) {
        self.out.push(' ');
        self.out.push_str(name);
        self.out.push_str("=\"");
        self.out.push_str(key);
        self.out.push('"');
    }
}

fn is_class_related(attr: &AttrInstr) -> bool {
    matches!(attr, AttrInstr::Static { name, .. } if name == "class")
        || matches!(attr, AttrInstr::Class { .. })
}

fn is_style_related(attr: &AttrInstr) -> bool {
    matches!(attr, AttrInstr::Static { name, .. } if name == "style")
        || matches!(attr, AttrInstr::Style { .. })
}

fn trim_trailing_semi(s: &str) -> String {
    s.trim_end().trim_end_matches(';').trim_end().to_string()
}
