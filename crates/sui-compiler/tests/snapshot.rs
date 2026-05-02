//! Snapshot tests. For each fixture, compile and compare against a
//! checked-in `<fixture>.expected.js`. Set `UPDATE_SNAPSHOTS=1` in
//! the environment to overwrite the expectations.

use std::fs;
use std::path::{Path, PathBuf};

use sui_compiler::{compile, CompileMode, ComponentMeta, CompileOpts};

struct Fixture {
    name: &'static str,
    setup_url: Option<&'static str>,
    mode: CompileMode,
    component_meta: Option<ComponentMeta>,
}

fn fixtures() -> Vec<Fixture> {
    vec![
        Fixture { name: "hello",       setup_url: None,               mode: CompileMode::View,      component_meta: None },
        Fixture { name: "counter",     setup_url: Some("./counter.ts"), mode: CompileMode::View,     component_meta: None },
        Fixture { name: "agents",      setup_url: Some("./agents.ts"),  mode: CompileMode::View,     component_meta: None },
        Fixture { name: "ship-button", setup_url: None,               mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "ship-button-test".into(), shadow: false }) },
        Fixture { name: "bind-this-button", setup_url: None,           mode: CompileMode::View,      component_meta: None },
        Fixture { name: "reflect-empty-slot", setup_url: None,         mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "reflect-empty-slot-test".into(), shadow: true }) },
        Fixture { name: "shadow-tones", setup_url: None,               mode: CompileMode::Component,
            component_meta: Some(ComponentMeta { name: "shadow-tones-test".into(), shadow: true }) },
    ]
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn compile_fixture(f: &Fixture) -> String {
    let path = fixtures_dir().join(format!("{}.sui", f.name));
    let source = fs::read_to_string(&path).expect("read fixture");
    let opts = CompileOpts {
        setup_url: f.setup_url.map(|s| s.to_string()),
        mode: f.mode,
        source_path: Some(path),
        styles: None,
        component_meta: f.component_meta.clone(),
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    out.js
}

fn assert_snapshot(name: &str, actual: &str, expected_path: &Path) {
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(expected_path, actual).expect("write snapshot");
        return;
    }
    let expected = match fs::read_to_string(expected_path) {
        Ok(s) => s,
        Err(_) => {
            fs::write(expected_path, actual).expect("write snapshot");
            panic!(
                "snapshot {} did not exist; wrote it now — re-run to confirm",
                expected_path.display()
            );
        }
    };
    assert_eq!(
        expected, actual,
        "snapshot mismatch for {name}. Set UPDATE_SNAPSHOTS=1 to refresh."
    );
}

#[test]
fn hello_snapshot() {
    let fx = &fixtures()[0];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn counter_snapshot() {
    let fx = &fixtures()[1];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn agents_snapshot() {
    let fx = &fixtures()[2];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn ship_button_snapshot() {
    let fx = &fixtures()[3];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn bind_this_button_snapshot() {
    let fx = &fixtures()[4];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn bind_this_button_manifest_carries_ref_kind() {
    use sui_compiler::compile;
    let path = fixtures_dir().join("bind-this-button.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path: Some(path),
        styles: None,
        component_meta: None,
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    // The view bundle exports `bindings` — the bind:this entry must
    // surface as kind "bind:this" with the bare scope variable name.
    assert!(
        out.js.contains("kind: \"bind:this\""),
        "expected bind:this kind in manifest, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("expr: \"btnEl\""),
        "expected btnEl scope-var in manifest, got:\n{}",
        out.js
    );
}

#[test]
fn reflect_empty_slot_snapshot() {
    let fx = &fixtures()[5];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn shadow_tones_snapshot() {
    let fx = &fixtures()[6];
    let js = compile_fixture(fx);
    let path = fixtures_dir().join(format!("{}.expected.js", fx.name));
    assert_snapshot(fx.name, &js, &path);
}

#[test]
fn shadow_tones_emits_adopted_style_sheets_call() {
    let path = fixtures_dir().join("shadow-tones.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "shadow-tones-test".into(),
            shadow: true,
        }),
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("suiTonesSheet"),
        "expected suiTonesSheet import:\n{}",
        out.js
    );
    assert!(
        out.js.contains("adoptedStyleSheets"),
        "expected adoptedStyleSheets call:\n{}",
        out.js
    );
    // Light-DOM (shadow: false) components must NOT get the tone sheet.
    let light_opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: None,
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "shadow-tones-test".into(),
            shadow: false,
        }),
        emit_dts: false,
    };
    let light_out = compile(
        &std::fs::read_to_string(fixtures_dir().join("shadow-tones.sui")).unwrap(),
        light_opts,
    ).expect("compile light");
    assert!(
        !light_out.js.contains("suiTonesSheet"),
        "light-DOM component must not import suiTonesSheet:\n{}",
        light_out.js
    );
}

#[test]
fn reflect_empty_slot_emits_bind_slot_empty_call() {
    use sui_compiler::compile;
    let path = fixtures_dir().join("reflect-empty-slot.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: Some(ComponentMeta {
            name: "reflect-empty-slot-test".into(),
            shadow: true,
        }),
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("bindSlotEmpty(") && out.js.contains("\"data-icon-only\""),
        "expected bindSlotEmpty(...) call referencing data-icon-only:\n{}",
        out.js
    );
    // Imports must include bindSlotEmpty.
    assert!(
        out.js.contains("bindSlotEmpty"),
        "expected bindSlotEmpty import, got:\n{}",
        out.js
    );
}

#[test]
fn ship_button_emits_cem() {
    let path = fixtures_dir().join("ship-button.sui");
    let source = fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::Component,
        source_path: Some(path),
        styles: None,
        component_meta: Some(ComponentMeta { name: "ship-button-test".into(), shadow: false }),
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    let cem = out.cem.expect("CEM emitted for component-mode");
    let cem_str = serde_json::to_string_pretty(&cem).unwrap();
    let expected_path = fixtures_dir().join("ship-button.expected.cem.json");
    assert_snapshot("ship-button.cem", &cem_str, &expected_path);
}

#[test]
fn with_island_emits_islands_export() {
    let path = fixtures_dir().join("with-island.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path: Some(path),
        styles: None,
        component_meta: None,
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    assert!(
        out.js.contains("export const ISLANDS"),
        "expected ISLANDS export, got:\n{}",
        out.js
    );
    assert!(
        out.js.contains("\"sui-terminal\""),
        "expected sui-terminal in ISLANDS, got:\n{}",
        out.js
    );
    // ISLANDS must appear before the import statements
    let islands_pos = out.js.find("export const ISLANDS").unwrap();
    let import_pos = out.js.find("import {").or_else(|| out.js.find("import \"")).unwrap();
    assert!(
        islands_pos < import_pos,
        "ISLANDS export must precede imports"
    );
}

#[test]
fn without_island_omits_islands_export() {
    let path = fixtures_dir().join("without-island.sui");
    let source = std::fs::read_to_string(&path).unwrap();
    let opts = CompileOpts {
        setup_url: None,
        mode: CompileMode::View,
        source_path: Some(path),
        styles: None,
        component_meta: None,
        emit_dts: false,
    };
    let out = compile(&source, opts).expect("compile");
    assert!(
        !out.js.contains("ISLANDS"),
        "expected no ISLANDS export for view without island tags, got:\n{}",
        out.js
    );
}

#[test]
fn schema_emitted_for_each_fixture() {
    for fx in &fixtures() {
        let path = fixtures_dir().join(format!("{}.sui", fx.name));
        let source = fs::read_to_string(&path).unwrap();
        let opts = CompileOpts {
            setup_url: None,
            mode: fx.mode,
            source_path: Some(path),
            styles: None,
            component_meta: fx.component_meta.clone(),
            emit_dts: false,
        };
        let out = compile(&source, opts).expect("compile");
        let schema = out.schema.expect("schema emitted");
        let s = serde_json::to_string_pretty(&schema).unwrap();
        let expected = fixtures_dir().join(format!("{}.expected.schema.json", fx.name));
        assert_snapshot(&format!("{}.schema", fx.name), &s, &expected);
    }
}
