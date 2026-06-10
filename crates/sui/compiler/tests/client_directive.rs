//! `<client>` directive — the per-view, host-injected data-source declaration
//! that replaces the hardcoded ambient `ship` (multiple `<client>`s = multiple
//! data sources). The emitted setup must destructure each declared client from
//! `ctx.clients` and seed `$scope` so a `<data source="X.op()">` (and inline
//! handlers) resolve bare `X` at runtime.

use sui_compiler::{compile, CompileOpts};

fn js(source: &str) -> String {
    compile(source, CompileOpts::default()).expect("compile").js
}

fn dts(source: &str) -> String {
    compile(source, CompileOpts::default()).expect("compile").dts.expect("dts emitted")
}

#[test]
fn data_source_typed_from_declared_client() {
    let out = dts(
        "<client name=\"feed\" />\n\
         <data name=\"t\" source=\"feed.clock()\" live />\n\
         <data name=\"o\" source=\"feed.once()\" />\n\
         <div>{t().kind}{o()}</div>",
    );
    assert!(
        out.contains("t: LiveSignal<LiveValueOf<ReturnType<Clients['feed']['clock']>>>"),
        "live data must type from the client return; got:\n{out}"
    );
    assert!(
        out.contains("o: Signal<Awaited<ReturnType<Clients['feed']['once']>>>"),
        "one-shot data must type from the client return; got:\n{out}"
    );
}

#[test]
fn data_type_falls_back_to_unknown_for_ship_or_chained() {
    // Grandfathered `ship` (not a declared client) and a chained call both keep
    // the conservative `unknown` — no false inference.
    let out = dts(
        "<client name=\"feed\" />\n\
         <data name=\"s\" source=\"ship.git.overview()\" live />\n\
         <data name=\"c\" source=\"feed.a().b()\" live />\n\
         <div>{s().kind}{c().kind}</div>",
    );
    assert!(out.contains("s: Signal<LiveState<unknown>>"), "ship source stays unknown");
    assert!(out.contains("c: Signal<LiveState<unknown>>"), "chained call stays unknown");
}

#[test]
fn declared_client_is_destructured_from_ctx_clients() {
    let out = js(
        "<client name=\"feed\" />\n\
         <data name=\"t\" source=\"feed.clock()\" live />\n\
         <div>{t().kind}</div>",
    );
    assert!(
        out.contains("const { feed } = ctx.clients"),
        "setup must destructure the declared client from ctx.clients; got:\n{out}"
    );
    assert!(out.contains("$scope.feed = feed"), "setup must seed $scope.<client>");
    assert!(out.contains("feed.clock()"), "the data-source thunk calls the client verbatim");
}

#[test]
fn multiple_clients_each_destructured() {
    let out = js(
        "<client name=\"crm\" />\n\
         <client name=\"analytics\" />\n\
         <data name=\"a\" source=\"crm.leads()\" live />\n\
         <data name=\"b\" source=\"analytics.traffic()\" live />\n\
         <div>{a().kind}{b().kind}</div>",
    );
    assert!(out.contains("const { crm } = ctx.clients"), "first client destructured");
    assert!(
        out.contains("const { analytics } = ctx.clients"),
        "second client destructured — multiple data sources in one view"
    );
}

#[test]
fn ship_grandfathered_not_routed_through_ctx_clients() {
    // A `ship.*` view with no `<client>` still emits the hardcoded destructure
    // (existing views are untouched through the migration); it is NOT routed
    // through ctx.clients.
    let out = js(
        "<data name=\"t\" source=\"ship.git.overview()\" live />\n<div>{t().kind}</div>",
    );
    assert!(out.contains("ship.git.overview()"), "ship source still resolves");
    assert!(
        !out.contains("const { ship } = ctx.clients"),
        "grandfathered ship must NOT be routed through ctx.clients"
    );
}
