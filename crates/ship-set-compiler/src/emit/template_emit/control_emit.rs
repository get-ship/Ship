//! Control-flow emit: `{#if}`, `{#for}`, `{#match}`, and the
//! `<while-loading>` / `<while-error>` boundaries.

use super::super::writer::Writer;
use super::TemplateEmitter;
use crate::ir::*;

impl<'a> TemplateEmitter<'a> {
    pub(in crate::emit) fn emit_if(
        &mut self,
        w: &mut Writer,
        instr: &IfInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let anchor = self.fresh("ifAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"if\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!("{cleanups_var}.push(bindIf({{"));
        w.enter();
        w.line(&format!("anchor: {anchor},"));
        w.line("test: () => {");
        w.enter();
        for (i, b) in instr.branches.iter().enumerate() {
            let idx = self.bank.add(&b.cond.source);
            w.line(&format!("if (__E[{idx}]($scope)) return {i};"));
        }
        let fallback = if instr.else_body.is_some() {
            instr.branches.len() as i32
        } else {
            -1
        };
        w.line(&format!("return {fallback};"));
        w.leave();
        w.line("},");
        w.line("branches: [");
        w.enter();
        for b in &instr.branches {
            self.emit_branch(w, &b.body);
        }
        if let Some(eb) = &instr.else_body {
            self.emit_branch(w, eb);
        }
        w.leave();
        w.line("],");
        w.leave();
        w.line("}));");
    }

    pub(in crate::emit) fn emit_for(
        &mut self,
        w: &mut Writer,
        instr: &ForInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let anchor = self.fresh("forAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"for\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        let it_idx = self.bank.add(&instr.iterable.source);
        w.line(&format!("{cleanups_var}.push(bindFor({{"));
        w.enter();
        w.line(&format!("anchor: {anchor},"));
        w.line(&format!("source: () => __E[{it_idx}]($scope),"));
        if let Some(k) = &instr.key {
            let ki = self.bank.add(&k.source);
            let var = &instr.var_name;
            let idx = instr.index_name.as_deref().unwrap_or("__i");
            w.line(&format!(
                "key: ({var}, {idx}) => {{ const __child = Object.create($scope); __child.{var} = {var}; __child.{idx} = {idx}; return __E[{ki}](__child); }},"
            ));
        }
        let var = &instr.var_name;
        let idx_name = instr.index_name.as_deref().unwrap_or("__i");
        w.line(&format!("render: ({var}, {idx_name}) => {{"));
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let child_scope = self.fresh("childScope");
        w.line(&format!("const {child_scope} = Object.create($scope);"));
        w.line(&format!("{child_scope}.{var} = {var};"));
        w.line(&format!("{child_scope}.{idx_name} = {idx_name};"));
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&format!("const {cl} = [];"));
        // Render with $scope shadowed for nested expressions — we
        // implement that by emitting a lexical const named $scope here.
        w.line(&format!("const $scope = {child_scope};"));
        self.emit_children(w, &instr.body, &frag, &cl);
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("},");
        if let Some(eb) = &instr.else_body {
            w.line("emptyRender: () => {");
            w.enter();
            let frag = self.fresh("frag");
            let cl = self.fresh("c");
            w.line(&format!(
                "const {frag} = document.createDocumentFragment();"
            ));
            w.line(&format!("const {cl} = [];"));
            self.emit_children(w, eb, &frag, &cl);
            w.line(&format!(
                "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
            ));
            w.leave();
            w.line("},");
        }
        w.leave();
        w.line("}));");
    }

    pub(in crate::emit) fn emit_match(
        &mut self,
        w: &mut Writer,
        instr: &MatchInstr,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let anchor = self.fresh("matchAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"match\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        let disc_idx = self.bank.add(&instr.discriminant.source);
        w.line(&format!("{cleanups_var}.push(bindIf({{"));
        w.enter();
        w.line(&format!("anchor: {anchor},"));
        w.line("test: () => {");
        w.enter();
        w.line(&format!("const __d = __E[{disc_idx}]($scope);"));
        for (i, c) in instr.cases.iter().enumerate() {
            let cidx = self.bank.add(&c.test.source);
            w.line(&format!(
                "if (__d === __E[{cidx}]($scope)) return {i};"
            ));
        }
        let fallback = if instr.default.is_some() {
            instr.cases.len() as i32
        } else {
            -1
        };
        w.line(&format!("return {fallback};"));
        w.leave();
        w.line("},");
        w.line("branches: [");
        w.enter();
        for c in &instr.cases {
            self.emit_branch(w, &c.body);
        }
        if let Some(d) = &instr.default {
            self.emit_branch(w, d);
        }
        w.leave();
        w.line("],");
        w.leave();
        w.line("}));");
    }

    pub(super) fn emit_while_loading(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        bind: &str,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let anchor = self.fresh("wlAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-loading\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        w.line(&format!(
            "{cleanups_var}.push(bindWhileLoading({{ anchor: {anchor}, signal: $scope.{bind}, render: () => {{"
        ));
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&format!("const {cl} = [];"));
        self.emit_children(w, &el.children, &frag, &cl);
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("} }));");
    }

    pub(super) fn emit_while_error(
        &mut self,
        w: &mut Writer,
        el: &ElementInstr,
        meta: &WhileErrorMeta,
        parent_var: &str,
        cleanups_var: &str,
    ) {
        let anchor = self.fresh("weAnchor");
        w.line(&format!(
            "const {anchor} = document.createComment(\"while-error\");"
        ));
        w.line(&format!("{parent_var}.appendChild({anchor});"));
        let var = &meta.error_var;
        w.line(&format!(
            "{cleanups_var}.push(bindWhileError({{ anchor: {anchor}, signal: $scope.{b}, render: ({var}) => {{",
            b = meta.bind
        ));
        w.enter();
        let frag = self.fresh("frag");
        let cl = self.fresh("c");
        let child = self.fresh("childScope");
        w.line(&format!("const {child} = Object.create($scope);"));
        w.line(&format!("{child}.{var} = {var};"));
        w.line(&format!(
            "const {frag} = document.createDocumentFragment();"
        ));
        w.line(&format!("const {cl} = [];"));
        w.line(&format!("const $scope = {child};"));
        self.emit_children(w, &el.children, &frag, &cl);
        w.line(&format!(
            "return {{ nodes: Array.from({frag}.childNodes), cleanup: () => {{ for (const c of {cl}) try {{ c(); }} catch (e) {{}} }} }};"
        ));
        w.leave();
        w.line("} }));");
    }
}
