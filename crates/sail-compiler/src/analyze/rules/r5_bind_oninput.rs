//! R5 — bind_value_with_oninput_redundancy.
//!
//! `bind:value={x}` already attaches the input listener that mirrors
//! the platform input back into `x`. Pairing it with `oninput={…}` (or
//! the legacy `on:input={…}`) on the same element double-wires the
//! same event and is almost always a stale port from a one-way
//! version. Keep one or the other.
//!
//! Components don't carry the bind-vs-event split through to the
//! analyzer (the parser drops `bind:` / `on:` attrs from the
//! `Component` node's prop/event lists), so this rule fires only on
//! `Node::Element`. That covers the canonical case — `bind:value` on a
//! native `<input>` / `<textarea>` — which is what the corpus uses.

use crate::parse::ast::{Attr, AttrValue, Node, SailFile, Template};
use crate::{Diagnostic, Severity};

pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let Some(tpl) = file.template.as_ref() else { return };
    walk_template(tpl, diagnostics);
}

fn walk_template(tpl: &Template, diagnostics: &mut Vec<Diagnostic>) {
    for n in &tpl.root {
        walk_node(n, diagnostics);
    }
}

fn walk_node(n: &Node, diagnostics: &mut Vec<Diagnostic>) {
    match n {
        Node::Element { tag, attrs, children } => {
            check_element(tag, attrs, diagnostics);
            for c in children {
                walk_node(c, diagnostics);
            }
        }
        Node::Component { children, .. } | Node::SlotFill { body: children, .. } => {
            for c in children {
                walk_node(c, diagnostics);
            }
        }
        Node::If { then_, elifs, else_, .. } => {
            for c in then_ {
                walk_node(c, diagnostics);
            }
            for (_, body) in elifs {
                for c in body {
                    walk_node(c, diagnostics);
                }
            }
            if let Some(b) = else_ {
                for c in b {
                    walk_node(c, diagnostics);
                }
            }
        }
        Node::For { body, empty, .. } => {
            for c in body {
                walk_node(c, diagnostics);
            }
            if let Some(e) = empty {
                for c in e {
                    walk_node(c, diagnostics);
                }
            }
        }
        Node::Await { loading, .. } => {
            for c in loading {
                walk_node(c, diagnostics);
            }
        }
        Node::Slot { fallback, .. } => {
            for c in fallback {
                walk_node(c, diagnostics);
            }
        }
        _ => {}
    }
}

fn check_element(tag: &str, attrs: &[Attr], diagnostics: &mut Vec<Diagnostic>) {
    let bind_value = attrs
        .iter()
        .any(|a| a.name == "bind:value" && matches!(a.value, AttrValue::BindTwoWay(_)));
    if !bind_value {
        return;
    }
    let oninput = attrs.iter().any(|a| {
        matches!(&a.value, AttrValue::OnEvent { event, .. } if event == "input")
    });
    if !oninput {
        return;
    }
    diagnostics.push(Diagnostic {
        severity: Severity::Warning,
        code: "SAIL/W0105".to_string(),
        message: format!(
            "<{tag}> has both `bind:value` and an input listener — they fire on the same event"
        ),
        span: None,
        help: Some(
            "bind:value attaches the input listener; oninput is redundant. Use one or the other"
                .to_string(),
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn run(src: &str) -> Vec<Diagnostic> {
        let file = parse::parse(src).expect("parse");
        let mut d = Vec::new();
        analyze(&file, &mut d);
        d
    }

    #[test]
    fn bind_with_oninput_warns() {
        let d = run("<template><input bind:value={draft} oninput={touch} /></template>");
        let w: Vec<_> = d.iter().filter(|x| x.code == "SAIL/W0105").collect();
        assert_eq!(w.len(), 1, "got {d:?}");
        assert!(w[0].help.as_deref().unwrap().contains("bind:value"));
    }

    #[test]
    fn bind_with_legacy_on_input_warns() {
        let d = run("<template><input bind:value={draft} on:input={touch} /></template>");
        let w: Vec<_> = d.iter().filter(|x| x.code == "SAIL/W0105").collect();
        assert_eq!(w.len(), 1, "got {d:?}");
    }

    #[test]
    fn bind_only_passes() {
        let d = run("<template><input bind:value={draft} /></template>");
        assert!(d.iter().all(|x| x.code != "SAIL/W0105"));
    }

    #[test]
    fn separate_elements_pass() {
        let d = run(
            "<template><input bind:value={draft} /><input oninput={touch} /></template>",
        );
        assert!(d.iter().all(|x| x.code != "SAIL/W0105"), "got {d:?}");
    }

    #[test]
    fn oninput_only_passes() {
        let d = run("<template><input oninput={touch} /></template>");
        assert!(d.iter().all(|x| x.code != "SAIL/W0105"));
    }

    #[test]
    fn unrelated_event_listener_passes() {
        // bind:value with onclick (different event) is fine.
        let d = run(
            "<template><input bind:value={draft} onclick={focusNext} /></template>",
        );
        assert!(d.iter().all(|x| x.code != "SAIL/W0105"), "got {d:?}");
    }

    #[test]
    fn bind_with_on_bind_callback_passes() {
        // `on:bind` is the side-effect callback hook for `bind:value`,
        // not a redundant input listener — it must not fire R5.
        let d = run(
            "<template><input bind:value={draft} on:bind={notify} /></template>",
        );
        assert!(d.iter().all(|x| x.code != "SAIL/W0105"), "got {d:?}");
    }
}
