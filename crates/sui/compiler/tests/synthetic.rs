//! Integration tests for the synthetic emitter (`emit::synthetic`).
//!
//! These exercise the emitter as a pure function — parse a `.sui`, run
//! `emit_synthetic`, assert structural properties of the emitted controller
//! + render. The TS-level proof (tsgo-green + typo caught + jsdom mount)
//! lives alongside the deck-conformance app; these lock the *shape* the
//! TS proof depends on, and run in plain `cargo test` (no node toolchain).

use sui_compiler::emit::synthetic::{
    emit_synthetic, emit_synthetic_client_module, AppManifest, AuthoredController,
};
use sui_compiler::parse::parse;
use sui_compiler::CompileMode;

const ROUTE_MANIFEST: &str = r#"{
  "clients": {
    "route": { "from": "@ship/deck/route", "import": "route", "construct": true }
  }
}"#;

const SHIP_MANIFEST: &str = r#"{
  "clients": {
    "ship": { "from": "./clients/demo.ts", "import": "makeShip", "construct": true }
  }
}"#;

fn synth(src: &str, manifest_src: &str, mode: CompileMode) -> (String, String) {
    let program = parse(src).expect("parse .sui");
    let manifest = AppManifest::parse(manifest_src).expect("parse manifest");
    let out = emit_synthetic(&program, &manifest, None, "./page.synthetic.controller.ts", mode)
        .expect("emit_synthetic");
    (out.controller_ts, out.render_ts)
}

#[test]
fn route_is_a_real_import_not_a_builtin() {
    let (ctrl, render) = synth(
        "<h1>Hello, {route.params.id()}</h1>",
        ROUTE_MANIFEST,
        CompileMode::View,
    );
    // The controller imports `route` as a REAL ESM import + constructs it.
    assert!(
        ctrl.contains(r#"import { route as __make_route } from "@ship/deck/route";"#),
        "controller must import route from the manifest module:\n{ctrl}"
    );
    assert!(ctrl.contains("const route = __make_route();"), "route is constructed:\n{ctrl}");
    assert!(ctrl.contains("export const scope = { route };"), "typed scope exports route:\n{ctrl}");
    // The render reads the signal by CALLING it (the closed signal-read seam).
    assert!(
        render.contains("bindText(") && render.contains("=> (scope.route.params.id())"),
        "render must read the route signal by calling it against the typed scope:\n{render}"
    );
    assert!(render.contains(r#"import { scope } from "./page.synthetic.controller.ts";"#));
}

#[test]
fn route_typo_is_a_real_member_access_for_tsgo_to_reject() {
    // `route.parms` (typo) must survive into the render verbatim as
    // `scope.route.parms.id()` — a real member access against RouteSignals,
    // which tsgo rejects. (No ambient passthrough.)
    let (_ctrl, render) = synth(
        "<h1>Hello, {route.parms.id()}</h1>",
        ROUTE_MANIFEST,
        CompileMode::View,
    );
    assert!(
        render.contains("scope.route.parms.id()"),
        "the typo must reach the typed scope read so tsgo flags it:\n{render}"
    );
}

#[test]
fn control_flow_reuses_template_emitter_with_mixed_prefix() {
    // The hard path: {#while-*}, {#if}, {#for}, class:, mutation — the full
    // production TemplateEmitter, driven with the `scope.` author prefix.
    let src = r#"<data name="list" source="ship.demo.todos()" live />
<div>
  {#while-loading of list}<p>Loading…</p>{/while-loading}
  {#while-error err of list}<p>Error: {err.message}</p>{/while-error}
  {#if list().kind === 'ok'}
  <ul>
    {#for item of list().value.items key={item.id}}
    <li class:done={item.done}>{item.text}</li>
    {/for}
  </ul>
  {/if}
</div>"#;
    let (ctrl, render) = synth(src, SHIP_MANIFEST, CompileMode::View);

    // Controller: live data lowered + typed scope.
    assert!(ctrl.contains("const list = live(() => (ship.demo.todos()));"), "live data:\n{ctrl}");
    assert!(ctrl.contains("export const scope = { ship, list };"), "scope:\n{ctrl}");

    // Render: the full binder set is reused from TemplateEmitter.
    for binder in ["bindWhileLoading", "bindWhileError", "bindIf", "bindFor", "bindClass"] {
        assert!(render.contains(binder), "render must use {binder} (TemplateEmitter reuse):\n{render}");
    }
    // Outer-scope data reads are typed (`scope.`), so tsgo checks them.
    assert!(render.contains("items: () => (scope.list().value.items)"), "typed outer read:\n{render}");
    assert!(render.contains("if (scope.list().kind === 'ok')"), "typed if test:\n{render}");
    // Loop / error binders live on the runtime `$scope` (keyed-reuse mutates
    // them) — qualified with `$scope.`, NOT the typed prefix.
    assert!(render.contains("$scope.item.done"), "loop local is machinery-prefixed:\n{render}");
    assert!(render.contains("$scope.err.message"), "error binder is machinery-prefixed:\n{render}");
    // The key fn keeps the loop var bare (it is the key fn parameter).
    assert!(render.contains("key: (item, __i) => (item.id)"), "key var stays bare:\n{render}");
    // The keyed-reuse update reassigns the runtime binder and re-reads it.
    assert!(
        render.contains("$scope.item = __new") && render.contains("$scope.item.text"),
        "update() reassigns and re-reads the runtime row binder:\n{render}"
    );
}

#[test]
fn non_live_data_lowers_to_data_handle() {
    let src = r#"<data name="cfg" source="ship.demo.config()" />
<p>{cfg().value}</p>"#;
    let (ctrl, _render) = synth(src, SHIP_MANIFEST, CompileMode::View);
    assert!(
        ctrl.contains("const cfg = data({ initial: undefined, source: () => (ship.demo.config()) });"),
        "non-live <data> lowers to the data() handle shape:\n{ctrl}"
    );
    assert!(ctrl.contains(r#"import { data } from "@ship/sui-runtime";"#), "imports data:\n{ctrl}");
}

#[test]
fn authored_controller_is_composed_via_setup_not_rewritten() {
    // The authored module default-exports a real `controller({...})`
    // (`ControllerExport`); the synthetic controller drives its `setup(ctx)`
    // (the production `resolveController`), spreads the result, and recovers
    // the contributed scope TYPE from the export — composed, never rewritten.
    let authored = AuthoredController { specifier: "./controller.ts".to_string() };
    let program = parse("<p>{greeting()} (count {count()})</p>").expect("parse");
    let manifest = AppManifest::parse("{}").expect("manifest");
    let out = emit_synthetic(
        &program,
        &manifest,
        Some(&authored),
        "./page.synthetic.controller.ts",
        CompileMode::View,
    )
    .expect("emit");
    assert!(out.controller_ts.contains(r#"import __authored from "./controller.ts";"#));
    assert!(
        out.controller_ts.contains(
            "import type { ControllerExport, ResolverContext } from \"@ship/sui-runtime\";"
        ),
        "composes the DSL via its typed export:\n{}",
        out.controller_ts
    );
    assert!(
        out.controller_ts.contains(
            "type __AuthoredScope = typeof __authored extends ControllerExport<infer S> ? S : never;"
        ),
        "recovers the contributed scope type from the ControllerExport:\n{}",
        out.controller_ts
    );
    assert!(
        out.controller_ts.contains("const __authoredScope = __authored.setup(__ctx) as __AuthoredScope;"),
        "runs the real resolveController via setup(ctx):\n{}",
        out.controller_ts
    );
    assert!(
        out.controller_ts.contains("export const scope = { ...__authoredScope };"),
        "authored scope is spread (composed), never rewritten:\n{}",
        out.controller_ts
    );
    // Render reads the composed members against the typed scope.
    assert!(out.render_ts.contains("scope.greeting()") && out.render_ts.contains("scope.count()"));
}

#[test]
fn authored_compose_maps_manifest_clients_into_resolver_ctx() {
    // A view whose template uses a well-known manifest client (`route`) AND a
    // named one (`api`) and has an authored controller: the synthetic
    // controller passes them to `setup(ctx)` in the shape `resolveController`
    // expects — well-known names as `ResolverContext` fields, the rest spread
    // into `clients`.
    let authored = AuthoredController { specifier: "./controller.ts".to_string() };
    let manifest = AppManifest::parse(
        r#"{ "clients": {
            "route": { "from": "@ship/deck/route", "import": "route", "construct": true },
            "api":   { "from": "@app/clients/demo", "import": "makeApi", "construct": true }
        } }"#,
    )
    .expect("manifest");
    let program = parse("<p>{route.params.id()} {api.demo.clock()}</p>").expect("parse");
    let out = emit_synthetic(
        &program,
        &manifest,
        Some(&authored),
        "./page.synthetic.controller.ts",
        CompileMode::View,
    )
    .expect("emit");
    assert!(out.controller_ts.contains("route: route,"), "route → ctx.route:\n{}", out.controller_ts);
    assert!(
        out.controller_ts.contains("clients: { api },"),
        "named client → ctx.clients:\n{}",
        out.controller_ts
    );
}

#[test]
fn propless_component_synthesizes_like_a_view() {
    // demo-chip shape: slot-only component, no props, no controller.
    let (ctrl, render) = synth("<span class=\"chip\"><slot></slot></span>", "{}", CompileMode::Component);
    assert!(ctrl.contains("export const scope = {  };"), "empty typed scope:\n{ctrl}");
    assert!(render.contains("export function mount(container: HTMLElement)"), "render mounts:\n{render}");
    assert!(render.contains(r#"document.createElement("span")"#), "emits the element:\n{render}");
}

#[test]
fn component_props_are_rejected_for_now() {
    let program = parse("<prop name=\"label\" type=\"string\" />\n<span>{label}</span>").expect("parse");
    let manifest = AppManifest::parse("{}").expect("manifest");
    let err = emit_synthetic(&program, &manifest, None, "./c.ts", CompileMode::Component)
        .expect_err("component props must be rejected in Phase 1");
    assert!(format!("{err:?}").contains("prop"), "error names props: {err:?}");
}

#[test]
fn async_let_is_rejected_until_phase_2() {
    let src = r#"<data name="thing" source="ship.demo.thing()" live />
{#async thing as v}{:loading}<p>…</p>{:error err}<p>{err.message}</p>{:ok}<p>{v.x}</p>{/async}"#;
    let program = parse(src).expect("parse");
    let manifest = AppManifest::parse(SHIP_MANIFEST).expect("manifest");
    let err = emit_synthetic(&program, &manifest, None, "./c.ts", CompileMode::View)
        .expect_err("{#async} must be rejected");
    assert!(format!("{err:?}").contains("async"), "error names async: {err:?}");
}

#[test]
fn client_emit_returns_mount_and_hydrate_contract() {
    // The CLIENT half: `default.setup(ctx)` returns `__mount` (CSR build) +
    // `hydrate` (adopt SSR markers) — the contract mount.ts drives. Same
    // mount body as the SSR render, plus the production hydrate buffers.
    let program = parse("<h1>Hello, {route.params.id()}</h1>").expect("parse");
    let manifest = AppManifest::parse(ROUTE_MANIFEST).expect("manifest");
    let out = emit_synthetic(
        &program,
        &manifest,
        None,
        "./page.synthetic.controller.ts",
        CompileMode::View,
    )
    .expect("emit");
    let client = out.render_client_ts;
    // The browser contract: wrapped in `view()`, default-exported.
    assert!(client.contains("export default view(() => {"), "client wraps in view():\n{client}");
    assert!(client.contains("__mount(container: HTMLElement) {"), "client has __mount:\n{client}");
    assert!(client.contains("hydrate(root: HTMLElement) {"), "client has hydrate:\n{client}");
    // It imports `view` from the runtime and the text hydrate helpers from
    // @ship/sui-dom (the production split).
    assert!(client.contains("from \"@ship/sui-dom\""), "imports hydrate helpers from sui-dom:\n{client}");
    assert!(client.contains("collectTextOpens") && client.contains("hydrateText"), "text hydrate:\n{client}");
    // __mount still reads the typed scope (the CSR path), exactly like SSR.
    assert!(client.contains("scope.route.params.id()"), "typed scope read in __mount:\n{client}");
    // The SSR render stays the bare `mount` export (unchanged contract).
    assert!(
        out.render_ts.contains("export function mount(container: HTMLElement)")
            && !out.render_ts.contains("export default view"),
        "SSR render keeps its mount-only export:\n{}",
        out.render_ts
    );
}

#[test]
fn merged_client_module_is_self_contained() {
    // The bundler-facing single module: controller + client render fused.
    // The cross-module `import { scope }` is dropped (scope is now local),
    // and the two import blocks merge without duplicate bindings.
    let src = r#"<data name="list" source="ship.demo.todos()" live />
<ul>{#for item of list().value.items key={item.id}}<li>{item.text}</li>{/for}</ul>"#;
    let program = parse(src).expect("parse");
    let manifest = AppManifest::parse(SHIP_MANIFEST).expect("manifest");
    let merged = emit_synthetic_client_module(&program, &manifest, None, CompileMode::View)
        .expect("merged client module");

    // No cross-module scope import survives the merge.
    assert!(
        !merged.contains("import { scope } from"),
        "merged module must not import scope across a boundary:\n{merged}"
    );
    // The controller's `scope` const is present as a local (its export line).
    assert!(merged.contains("export const scope = { ship, list };"), "scope is local:\n{merged}");
    // The client default export is present.
    assert!(merged.contains("export default view(() => {"), "client default export present:\n{merged}");
    // The live-data lowering (controller) and the for-binder (render) coexist.
    assert!(merged.contains("const list = live(() => (ship.demo.todos()));"), "controller body:\n{merged}");
    assert!(merged.contains("bindFor("), "render body:\n{merged}");
    // Exactly one value import per module specifier (names unioned, not
    // duplicated) — count `from "@ship/sui-runtime"` value-import lines.
    let runtime_value_imports = merged
        .lines()
        .filter(|l| l.starts_with("import {") && l.contains("from \"@ship/sui-runtime\""))
        .count();
    assert_eq!(
        runtime_value_imports, 1,
        "value imports from @ship/sui-runtime must be merged to one line:\n{merged}"
    );
}

#[test]
fn only_referenced_manifest_clients_are_imported() {
    // route is in the manifest but unused → not imported.
    let (ctrl, _render) = synth("<p>static</p>", ROUTE_MANIFEST, CompileMode::View);
    assert!(!ctrl.contains("@ship/deck/route"), "unused manifest client must not import:\n{ctrl}");
    assert!(ctrl.contains("export const scope = {  };"), "empty scope when nothing referenced:\n{ctrl}");
}
