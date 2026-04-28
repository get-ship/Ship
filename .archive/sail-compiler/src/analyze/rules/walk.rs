//! Whole-file walkers used by liveness rules.
//!
//! [`for_each_expr`] yields every JS expression reachable from the
//! template (text, attributes, control-flow heads, prop bindings,
//! handlers, slot props, host attrs). [`for_each_attr_expr`] is the
//! same walk restricted to attribute-position expressions —
//! `redundant_aria_string` uses it to confirm a derived is wired into
//! an `aria-*` attribute somewhere. [`collect_all_references`] joins
//! the template walk with every script body that can read a signal,
//! producing the set of names live anywhere in the file.

use std::collections::HashSet;

use crate::analyze::reactivity::extract_idents;
use crate::parse::ast::{Attr, AttrValue, Expr, Node, SailFile, Template};

pub(crate) fn for_each_expr<F>(tpl: &Template, mut f: F)
where
    F: FnMut(&Expr),
{
    fn walk_attrs(attrs: &[Attr], f: &mut impl FnMut(&Expr)) {
        for a in attrs {
            match &a.value {
                AttrValue::Static(_) => {}
                AttrValue::Interpolated(e)
                | AttrValue::BindTwoWay(e)
                | AttrValue::OnEvent { handler: e, .. } => f(e),
            }
        }
    }
    fn walk(n: &Node, f: &mut impl FnMut(&Expr)) {
        match n {
            Node::Element { attrs, children, .. } => {
                walk_attrs(attrs, f);
                children.iter().for_each(|c| walk(c, f));
            }
            Node::Component { props, events, slots, children, .. } => {
                for p in props { f(&p.value); }
                for e in events { f(&e.handler); }
                for s in slots { s.body.iter().for_each(|c| walk(c, f)); }
                children.iter().for_each(|c| walk(c, f));
            }
            Node::Host { attrs } => walk_attrs(attrs, f),
            Node::Interpolation(e) | Node::RawHtml(e) => f(e),
            Node::If { cond, then_, elifs, else_ } => {
                f(cond);
                then_.iter().for_each(|c| walk(c, f));
                for (e, b) in elifs { f(e); b.iter().for_each(|c| walk(c, f)); }
                if let Some(b) = else_ { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::For { iter, key, body, empty, .. } => {
                f(iter);
                if let Some(k) = key { f(k); }
                body.iter().for_each(|c| walk(c, f));
                if let Some(e) = empty { e.iter().for_each(|c| walk(c, f)); }
            }
            Node::Await { promise, loading, then_, catch } => {
                f(promise);
                loading.iter().for_each(|c| walk(c, f));
                if let Some((_, b)) = then_ { b.iter().for_each(|c| walk(c, f)); }
                if let Some((_, b)) = catch { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::Slot { props, fallback, .. } => {
                for (_, e) in props { f(e); }
                fallback.iter().for_each(|c| walk(c, f));
            }
            Node::SlotFill { body, .. } => body.iter().for_each(|c| walk(c, f)),
            Node::Text(_) => {}
        }
    }
    tpl.root.iter().for_each(|n| walk(n, &mut f));
}

pub(crate) fn for_each_attr_expr<F>(tpl: &Template, mut f: F)
where
    F: FnMut(&Expr),
{
    fn walk_attrs(attrs: &[Attr], f: &mut impl FnMut(&Expr)) {
        for a in attrs {
            match &a.value {
                AttrValue::Static(_) => {}
                AttrValue::Interpolated(e)
                | AttrValue::BindTwoWay(e)
                | AttrValue::OnEvent { handler: e, .. } => f(e),
            }
        }
    }
    fn walk(n: &Node, f: &mut impl FnMut(&Expr)) {
        match n {
            Node::Element { attrs, children, .. } => {
                walk_attrs(attrs, f);
                children.iter().for_each(|c| walk(c, f));
            }
            Node::Component { props, events, slots, children, .. } => {
                for p in props { f(&p.value); }
                for e in events { f(&e.handler); }
                for s in slots { s.body.iter().for_each(|c| walk(c, f)); }
                children.iter().for_each(|c| walk(c, f));
            }
            Node::Host { attrs } => walk_attrs(attrs, f),
            Node::If { then_, elifs, else_, .. } => {
                then_.iter().for_each(|c| walk(c, f));
                for (_, b) in elifs { b.iter().for_each(|c| walk(c, f)); }
                if let Some(b) = else_ { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::For { body, empty, .. } => {
                body.iter().for_each(|c| walk(c, f));
                if let Some(e) = empty { e.iter().for_each(|c| walk(c, f)); }
            }
            Node::Await { loading, then_, catch, .. } => {
                loading.iter().for_each(|c| walk(c, f));
                if let Some((_, b)) = then_ { b.iter().for_each(|c| walk(c, f)); }
                if let Some((_, b)) = catch { b.iter().for_each(|c| walk(c, f)); }
            }
            Node::Slot { fallback, .. } => fallback.iter().for_each(|c| walk(c, f)),
            Node::SlotFill { body, .. } => body.iter().for_each(|c| walk(c, f)),
            Node::Interpolation(_) | Node::RawHtml(_) | Node::Text(_) => {}
        }
    }
    tpl.root.iter().for_each(|n| walk(n, &mut f));
}

pub(crate) fn collect_all_references(file: &SailFile) -> HashSet<String> {
    let mut out: HashSet<String> = HashSet::new();
    if let Some(tpl) = file.template.as_ref() {
        for_each_expr(tpl, |e| {
            for n in extract_idents(&e.raw) { out.insert(n); }
        });
    }
    let Some(script) = file.script.as_ref() else { return out };
    for d in &script.derived {
        for n in extract_idents(&d.expr) { out.insert(n); }
    }
    for e in &script.effects {
        for n in extract_idents(&e.body) { out.insert(n); }
        for d in &e.deps { out.insert(d.clone()); }
    }
    for f in &script.functions {
        for n in extract_idents(&f.raw) { out.insert(n); }
    }
    for d in &script.data {
        for n in extract_idents(&d.expr) { out.insert(n); }
    }
    for p in &script.props {
        if let Some(def) = &p.default {
            for n in extract_idents(def) { out.insert(n); }
        }
    }
    for s in &script.state {
        for n in extract_idents(&s.initial) {
            if n != s.name { out.insert(n); }
        }
    }
    if let Some(m) = &script.mount {
        for n in extract_idents(m) { out.insert(n); }
    }
    if let Some(u) = &script.unmount {
        for n in extract_idents(u) { out.insert(n); }
    }
    for h in &script.form_lifecycle {
        for n in extract_idents(&h.body) { out.insert(n); }
    }
    for md in &script.module_decls {
        if let Some(init) = &md.initializer {
            for n in extract_idents(init) {
                if n != md.name { out.insert(n); }
            }
        }
        out.insert(md.name.clone());
    }
    out
}
