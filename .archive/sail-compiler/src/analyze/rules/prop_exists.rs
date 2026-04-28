//! Cross-component prop existence check.
//!
//! Walks the template; for each `<ship-foo bad-prop=…>` component
//! reference, looks up the tag's prop set in
//! [`super::component_catalog`] (built once from embedded `.set`
//! sources). Unknown prop names emit `SAIL/E0201` with a closest-
//! match suggestion via `edit_distance` (duplicated here from
//! `analyze::data_paths` — same algorithm, but the rules module is
//! scoped to itself by the job spec and the data_paths file is off
//! limits).
//!
//! Tags not in the catalog (third-party, app-local) are skipped — v1
//! doesn't speculate. Native HTML attributes that any custom element
//! carries (id, class, style, slot, part, role, ARIA, data-) are
//! allowed regardless of the prop list, since the platform forwards
//! them unchanged.
//!
//! Catalog keys are kebab-cased (templates write `icon-slug`, the
//! component declares `prop iconSlug`) — see [`super::kebab_of`].

use std::collections::BTreeSet;

use crate::parse::ast::{PropBinding, SailFile};
use crate::{Diagnostic, Severity};

pub fn analyze(file: &SailFile, diagnostics: &mut Vec<Diagnostic>) {
    let Some(tpl) = file.template.as_ref() else { return };
    super::for_each_component(tpl, |tag, props| check_component(tag, props, diagnostics));
}

const NATIVE_ATTRS: &[&str] = &[
    "id", "class", "style", "slot", "part", "exportparts", "role",
    "tabindex", "hidden", "title", "lang", "dir", "draggable",
    "contenteditable", "inert", "is", "spellcheck", "autofocus",
];

fn is_native_attr(name: &str) -> bool {
    NATIVE_ATTRS.contains(&name) || name.starts_with("aria-") || name.starts_with("data-")
}

fn check_component(tag: &str, props: &[PropBinding], diagnostics: &mut Vec<Diagnostic>) {
    let Some(known) = super::component_catalog().get(tag) else { return };
    for p in props {
        if known.contains(&p.name) || is_native_attr(&p.name) { continue; }
        let suggestion = nearest(&p.name, known);
        let help = match suggestion {
            Some(s) => format!("did you mean `{s}`?"),
            None => format!("`<{tag}>` declares no `{}` prop", p.name),
        };
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "SAIL/E0201".to_string(),
            message: format!("unknown prop `{}` on <{tag}>", p.name),
            span: p.span.clone(),
            help: Some(help),
            fix: None,
        });
    }
}

fn nearest(target: &str, known: &BTreeSet<String>) -> Option<String> {
    let mut best: Option<(usize, &String)> = None;
    for cand in known {
        let d = edit_distance(target, cand);
        if d > 3 { continue; }
        match best {
            None => best = Some((d, cand)),
            Some((bd, _)) if d < bd => best = Some((d, cand)),
            _ => {}
        }
    }
    best.map(|(_, s)| s.clone())
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr: Vec<usize> = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        curr[0] = i;
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
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

    fn warns(d: &[Diagnostic]) -> Vec<&Diagnostic> {
        d.iter().filter(|x| x.code == "SAIL/E0201").collect()
    }

    #[test]
    fn unknown_prop_warns() {
        let d = run("<template><ship-button bogus=\"x\">go</ship-button></template>");
        let w = warns(&d);
        assert_eq!(w.len(), 1, "got {d:?}");
        assert!(w[0].message.contains("bogus"));
        assert!(w[0].message.contains("ship-button"));
        assert_eq!(w[0].severity, Severity::Error);
    }

    #[test]
    fn known_prop_passes() {
        let d = run("<template><ship-button tone=\"primary\" variant=\"solid\">go</ship-button></template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn unknown_component_skipped() {
        let d = run("<template><my-app-widget bogus=\"x\" /></template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn typo_yields_did_you_mean() {
        let d = run("<template><ship-button tonee=\"primary\">go</ship-button></template>");
        let w = warns(&d);
        assert_eq!(w.len(), 1, "got {d:?}");
        assert!(w[0].help.as_deref().unwrap_or("").contains("tone"));
    }

    #[test]
    fn native_attrs_skipped() {
        let d = run("<template><ship-button class=\"x\" id=\"y\" part=\"go\" aria-label=\"go\" data-test=\"t\" /></template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }

    #[test]
    fn camel_prop_matches_kebab_attr() {
        // `prop iconSlug` on ship-avatar should match `icon-slug=` in templates.
        let d = run("<template><ship-avatar icon-slug=\"x\" /></template>");
        assert!(warns(&d).is_empty(), "got {d:?}");
    }
}
