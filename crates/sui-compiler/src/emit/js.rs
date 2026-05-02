//! JS module emitter.
//!
//! Layout of the emitted module:
//!
//! ```js
//! import { signal, view, … } from "@ship/ui-runtime";
//! import setup from "./view.ts";        // when paired
//!
//! const __scopeEval = (src) =>
//!   new Function("$scope", "with ($scope) { return (" + src + "); }");
//! const __E = [];                       // pre-compiled expression closures
//! __E[0] = __scopeEval("count()");
//! …
//!
//! export default view((ctx) => {
//!   const $scope = Object.create(null);
//!   $scope.ship = ctx.ship; …
//!   $scope.count = signal(0);
//!   …
//!   const __wireBoundary_b3 = (anchor) => bindIf({ … });
//!   …
//!   return Object.assign({}, $scope, {
//!     __mount(container) { … }
//!     __b3_attach: (target) => { … },
//!   });
//! });
//! ```
//!
//! The expression-closure trick uses `new Function` so the emitted
//! module is itself plain ESM (strict by spec) but each closure runs
//! in a non-strict body where `with($scope)` is legal.
//!
//! Phase 2 client-side control flow: each `{#if}` / `{#for}` / `{#match}`
//! emits a hoisted `__wireBoundary_b<N>` (used by both `__mount` and
//! `__b<N>_attach`) plus a `__b<N>_attach` named property on the view's
//! setup return. The compiled module exports no manifest entries for
//! `if` / `for` / `match` / `await` kinds — `attach()` finds these
//! boundaries by walking SSR'd `<sui-{kind} key="…">` elements and
//! dispatching to the matching attach handler on the setup return.

use super::template_emit::{ExprBank, TemplateEmitter};
use super::util::{camel_of, js_ident};
use super::writer::{js_string, Writer};
use crate::ir::*;
use crate::islands::collect_islands;
use crate::keys::{BindingKind, BindingRegistry};
use crate::lower::{directives::prop_default_literal, Lowered};
use crate::{CompileMode, CompileOpts};

pub fn emit_module(
    program: &Program,
    lowered: &Lowered,
    bindings: &BindingRegistry,
    opts: &CompileOpts,
    mode: CompileMode,
) -> String {
    let mut w = Writer::new();
    let islands = collect_islands(&program.template);
    if !islands.is_empty() {
        let list = islands
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(", ");
        w.line(&format!("export const ISLANDS = [{list}];"));
        w.nl();
    }
    emit_imports(&mut w, lowered, opts);
    emit_helpers(&mut w);

    let mut bank = ExprBank::new();
    let mut body = Writer::new();
    body.enter();
    emit_setup_body(&mut body, program, bindings, &mut bank, mode, opts);
    body.leave();

    emit_expr_bank(&mut w, &bank);

    match mode {
        CompileMode::Component => emit_component_export(&mut w, program, body, opts),
        _ => emit_view_export(&mut w, body),
    }
    // The `bindings` manifest is read by the view runtime's
    // `attach()` to re-wire DOM bindings on hydration. Components
    // register via `customElements.define()` and re-bind themselves
    // through the lifecycle, so the manifest is dead weight in the
    // component bundle — skip it. Trimming this saves ~9 KB
    // ungzipped across the 18 ported primitives.
    if !matches!(mode, CompileMode::Component) {
        emit_bindings(&mut w, bindings);
    }
    w.into_string()
}

fn emit_bindings(w: &mut Writer, bindings: &BindingRegistry) {
    w.nl();
    w.line("export const bindings = [");
    w.enter();
    for b in bindings.iter() {
        if b.suppressed {
            continue;
        }
        // Phase 2: control-flow boundaries no longer surface in the
        // manifest. attach() finds them by walking SSR'd
        // `<sui-{kind} key="b<N>">` elements and dispatching to the
        // setup return's `__b<N>_attach` handler.
        if matches!(
            b.kind,
            BindingKind::If
                | BindingKind::For
                | BindingKind::Match
                | BindingKind::Await
        ) {
            continue;
        }
        let deps_js = b
            .deps
            .iter()
            .map(|d| js_string(d))
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(rk) = &b.render_key {
            w.line(&format!(
                "{{ key: {}, kind: {}, expr: {}, deps: [{}], renderKey: {} }},",
                js_string(&b.key),
                js_string(&b.kind.as_marker_str()),
                js_string(&b.expr),
                deps_js,
                js_string(rk),
            ));
        } else {
            w.line(&format!(
                "{{ key: {}, kind: {}, expr: {}, deps: [{}] }},",
                js_string(&b.key),
                js_string(&b.kind.as_marker_str()),
                js_string(&b.expr),
                deps_js,
            ));
        }
    }
    w.leave();
    w.line("];");
}

fn emit_imports(w: &mut Writer, lowered: &Lowered, opts: &CompileOpts) {
    let is_shadow = opts.component_meta.as_ref().map(|m| m.shadow).unwrap_or(false);
    let mut imports: Vec<String> = lowered.runtime_imports.iter().map(|s| s.to_string()).collect();
    if is_shadow {
        let sym = "suiTonesSheet".to_string();
        if !imports.contains(&sym) {
            imports.push(sym);
            imports.sort();
        }
    }
    w.line("// AUTO-GENERATED by sui-compiler. Do not edit.");
    if imports.is_empty() {
        w.line("import \"@ship/ui-runtime\";");
    } else {
        w.line(&format!(
            "import {{ {} }} from \"@ship/ui-runtime\";",
            imports.join(", ")
        ));
    }
    if let Some(setup) = &opts.setup_url {
        w.line(&format!("import setup from {};", js_string(setup)));
    }
    w.nl();
}

fn emit_helpers(w: &mut Writer) {
    w.line("const __scopeEval = (src) =>");
    w.line("  new Function(\"$scope\", \"with ($scope) { return (\" + src + \"); }\");");
    w.nl();
}

fn emit_expr_bank(w: &mut Writer, bank: &ExprBank) {
    if bank.entries.is_empty() {
        return;
    }
    w.line("const __E = [];");
    for (i, src) in bank.entries.iter().enumerate() {
        w.line(&format!(
            "__E[{}] = __scopeEval({});",
            i,
            js_string(src)
        ));
    }
    w.nl();
}

fn emit_view_export(w: &mut Writer, body: Writer) {
    w.line("export default view((ctx) => {");
    w.push_str(&body.into_string());
    w.line("});");
}

#[allow(clippy::expect_used)]
fn emit_component_export(w: &mut Writer, program: &Program, body: Writer, opts: &CompileOpts) {
    let meta = opts.component_meta.as_ref().expect("component_meta required for component mode");
    w.line(&format!("export default component({}, {{", js_string(&meta.name)));
    w.enter();
    emit_props_object(w, &program.props);
    if meta.shadow {
        w.line("shadow: true,");
    }
    if !program.styles.is_empty() {
        w.line(&format!("styles: {},", js_string(&program.styles)));
    }
    w.line("setup(ctx) {");
    w.push_str(&body.into_string());
    w.line("},");
    w.leave();
    w.line("});");
}

fn emit_props_object(w: &mut Writer, props: &[PropDecl]) {
    if props.is_empty() {
        return;
    }
    w.line("props: {");
    w.enter();
    for p in props {
        w.line(&format!("{}: {},", js_string(&p.name), prop_call(p)));
    }
    w.leave();
    w.line("},");
}

fn prop_call(p: &PropDecl) -> String {
    let opts_parts = build_prop_opts(p);
    let opts_str = if opts_parts.is_empty() {
        String::new()
    } else {
        format!("{{ {} }}", opts_parts.join(", "))
    };
    match &p.kind {
        PropKind::String => format!("prop.string({opts_str})"),
        PropKind::Bool => format!("prop.bool({opts_str})"),
        PropKind::Number => format!("prop.number({opts_str})"),
        PropKind::Union(values) => {
            let values_js = values
                .iter()
                .map(|v| js_string(v))
                .collect::<Vec<_>>()
                .join(", ");
            if opts_str.is_empty() {
                format!("prop.union([{values_js}])")
            } else {
                format!("prop.union([{values_js}], {opts_str})")
            }
        }
    }
}

fn build_prop_opts(p: &PropDecl) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(d) = prop_default_literal(p) {
        parts.push(format!("default: {d}"));
    }
    if p.reflect {
        parts.push("reflect: true".into());
    }
    if p.bindable {
        parts.push("bindable: true".into());
    }
    parts
}

fn emit_setup_body(
    w: &mut Writer,
    program: &Program,
    bindings: &BindingRegistry,
    bank: &mut ExprBank,
    mode: CompileMode,
    opts: &CompileOpts,
) {
    let is_component = matches!(mode, CompileMode::Component);
    if is_component {
        w.line("const { props, host } = ctx;");
        w.line("if (!ctx.cleanups) ctx.cleanups = [];");
        w.line("const __cleanups = ctx.cleanups;");
    } else {
        w.line("const { ship, route, navigate } = ctx;");
    }
    w.line("const $scope = Object.create(null);");
    if is_component {
        emit_component_scope_seed(w, program);
    } else {
        w.line("$scope.ship = ship;");
        w.line("$scope.route = route;");
        w.line("$scope.navigate = navigate;");
    }
    emit_data_decls(w, program);
    emit_state_decls(w, program);
    if opts.setup_url.is_some() {
        if is_component {
            // Component path: pass $scope as ctx.scope so the controller's
            // on-handlers can mutate state signals. Store __r outside the
            // if-block so we can read LIFECYCLE_KEY after the template renders.
            emit_component_setup_call(w);
        } else {
            emit_setup_call(w);
        }
    }

    // Single-pass walk: emit the template body into a side writer so
    // the emitter's boundary-hoist buffer fills up. Drain hoists into
    // `w` before writing the body — the `__wireBoundary_b<N>` consts
    // must be in scope at every bindIf/bindFor call site they wrap.
    let mut emitter = TemplateEmitter::new(bank, bindings, !is_component);
    if is_component {
        let mut tpl_w = Writer::new();
        tpl_w.line("const __root = host.shadowRoot ?? host;");
        let is_shadow = opts.component_meta.as_ref().map(|m| m.shadow).unwrap_or(false);
        if is_shadow {
            tpl_w.line("if (suiTonesSheet && __root instanceof ShadowRoot) { try { __root.adoptedStyleSheets = [suiTonesSheet, ...(__root.adoptedStyleSheets ?? [])]; } catch {} }");
        }
        emitter.emit_top_level(&mut tpl_w, &program.template, "__root", "__cleanups");
        if opts.setup_url.is_some() {
            // bind:this refs are now in $scope; call mount (if any) with a
            // proxy that reads them lazily — giving the controller access to
            // e.g. `state.termHost()` even though termHost was not yet in
            // $scope when setup.setup(ctx) ran above.
            emit_component_lifecycle_wiring(&mut tpl_w);
        }
        tpl_w.line("return $scope;");

        let hoists = emitter.take_boundary_hoists();
        if !hoists.is_empty() {
            w.push_str(&hoists);
        }
        w.push_str(&tpl_w.into_string());
    } else {
        let mut mount_w = Writer::new();
        mount_w.line("const __cleanups = [];");
        emitter.emit_top_level(
            &mut mount_w,
            &program.template,
            "container",
            "__cleanups",
        );
        mount_w.line("return () => { for (const c of __cleanups) try { c(); } catch (e) {} };");

        let hoists = emitter.take_boundary_hoists();
        let attach_handlers = emitter.take_attach_handlers();
        let callbacks = emitter.take_render_callbacks();

        if !hoists.is_empty() {
            w.push_str(&hoists);
        }

        w.line("return Object.assign({}, $scope, {");
        w.enter();
        w.line("__mount(container) {");
        w.enter();
        w.push_str(&mount_w.into_string());
        w.leave();
        w.line("},");
        if !attach_handlers.is_empty() {
            w.push_str(&attach_handlers);
        }
        if !callbacks.is_empty() {
            w.push_str(&callbacks);
        }
        w.leave();
        w.line("});");
    }
}

fn emit_component_scope_seed(w: &mut Writer, program: &Program) {
    for p in &program.props {
        w.line(&format!(
            "$scope.{ident} = props.{ident};",
            ident = js_ident(&camel_of(&p.name))
        ));
    }
}

fn emit_data_decls(w: &mut Writer, program: &Program) {
    for d in &program.data {
        let src = &d.source.source;
        if d.live {
            w.line(&format!(
                "$scope.{name} = live(({{ ship, route }}) => {src});",
                name = js_ident(&d.name),
                src = src
            ));
        } else {
            w.line(&format!(
                "$scope.{name} = data({{ initial: undefined, source: ({{ ship, route }}) => {src} }});",
                name = js_ident(&d.name),
                src = src
            ));
        }
    }
}

fn emit_state_decls(w: &mut Writer, program: &Program) {
    for s in &program.state {
        w.line(&format!(
            "$scope.{name} = signal(({src}));",
            name = js_ident(&s.name),
            src = s.value.source
        ));
    }
}

fn emit_setup_call(w: &mut Writer) {
    // Author-facing paired-controller shape is `controller(def)` (and
    // `view(setup)` for views) — both wrap into `{ setup }`. No raw
    // `function setup(ctx)` author export. Compiler emits a single call
    // path; if a paired `.ts` returns a non-object, we ignore it instead
    // of letting it shadow the runtime's $scope.
    w.line("if (setup && typeof setup.setup === \"function\") {");
    w.enter();
    w.line("const __r = setup.setup(ctx);");
    w.line("if (__r && typeof __r === \"object\") Object.assign($scope, __r);");
    w.leave();
    w.line("}");
}

fn emit_component_setup_call(w: &mut Writer) {
    // Component variant: expose $scope to the controller so its on-handlers
    // can mutate <state> signals via the writable proxy. Store __r in a
    // let-binding so emit_component_lifecycle_wiring can read LIFECYCLE_KEY
    // from it after the template has rendered (bind:this refs land in $scope
    // during emit_top_level, which runs after this call).
    w.line("let __r = null;");
    w.line("if (setup && typeof setup.setup === \"function\") {");
    w.enter();
    w.line("ctx.scope = $scope;");
    w.line("__r = setup.setup(ctx);");
    w.line("if (__r && typeof __r === \"object\") Object.assign($scope, __r);");
    w.leave();
    w.line("}");
}

fn emit_component_lifecycle_wiring(w: &mut Writer) {
    // Called after emit_top_level so bind:this refs are in $scope. Reads
    // the LIFECYCLE_KEY bundle from the controller's setup result and wires
    // mount + unmount into __cleanups.
    //
    // The __sp proxy returns $scope values as-is so signal-typed refs (e.g.
    // termHost, prop signals) remain callable: `state.termHost()` works.
    // The set-trap delegates writes back to signal.set() for mutable keys.
    w.line("if (__r != null) {");
    w.enter();
    w.line("const __lc = Object.getOwnPropertyDescriptor(__r, \"__shipSetLifecycle\")?.value;");
    w.line("if (__lc) {");
    w.enter();
    w.line("const __sp = new Proxy($scope, {");
    w.enter();
    w.line("get(t, k) { return t[k]; },");
    w.line("set(t, k, v) { const s = t[k]; if (s != null && typeof s.set === \"function\") { s.set(v); return true; } return false; }");
    w.leave();
    w.line("});");
    w.line("if (__lc.mount) { const __mc = __lc.mount(__sp, host); if (typeof __mc === \"function\") __cleanups.push(__mc); }");
    w.line("if (__lc.unmount) __cleanups.push(() => __lc.unmount(__sp));");
    w.leave();
    w.line("}");
    w.leave();
    w.line("}");
}
