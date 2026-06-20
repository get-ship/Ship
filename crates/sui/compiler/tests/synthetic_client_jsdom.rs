//! jsdom proof for the synthetic CLIENT contract (slice 1, acceptance #1/#3).
//!
//! Two things the SSR sweep can't show:
//!   * **CSR** — the client `default.setup().__mount(container)` path, driven
//!     through the REAL `mount.ts`, builds reactive DOM in a browser (jsdom):
//!     hub `/` renders, and a propless component (badge) re-renders on a signal
//!     write. (acceptance #1: "render CSR (the __mount path)".)
//!   * **Isomorphic-render parity / clean hydration** — the SAME render run
//!     server-side (the SSR module against the shared server-DOM, i.e. exactly
//!     what ship-js serializes) produces a tree the client agrees with: the
//!     client `__mount` build is structurally identical, and `mount.ts`
//!     adopting the SSR HTML completes with NO mismatch / teardown.
//!     (acceptance #3.)
//!
//! Opt-in: requires `bun install` so `node_modules/.bin/vitest` exists.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use sui_compiler::emit::synthetic::{
    emit_synthetic, emit_synthetic_client_module, AppManifest, AuthoredController,
};
use sui_compiler::parse::parse;
use sui_compiler::CompileMode;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap()
}

const CONF_MANIFEST: &str = "crates/deck/deck-conformance/app/app.manifest.jsonc";
const HUB_MANIFEST: &str = "apps/hub/app.manifest.jsonc";
const VIEWS_MANIFEST: &str = "views/app.manifest.jsonc";

struct Target {
    name: &'static str,
    source_rel: &'static str,
    manifest_rel: &'static str,
    mode: CompileMode,
    controller_rel: Option<&'static str>,
}

/// Synthesize, for one target: the merged CLIENT module (`<name>.client.ts`,
/// what `mount.ts` drives) AND the split SSR render + controller
/// (`<name>.ssr.ts` + `<name>.controller.ts`, what reproduces the ship-js
/// server-DOM render). Both compose the same authored controller (abs path).
fn synth(repo: &Path, work: &Path, t: &Target) {
    let src = fs::read_to_string(repo.join(t.source_rel)).expect("read source");
    let manifest_src = fs::read_to_string(repo.join(t.manifest_rel)).expect("read manifest");
    let manifest = AppManifest::parse(&manifest_src).expect("manifest");
    let program = parse(&src).unwrap_or_else(|e| panic!("parse {}: {}", t.name, e.render(&src)));
    let authored = t
        .controller_rel
        .map(|c| AuthoredController { specifier: repo.join(c).to_string_lossy().into_owned() });

    let merged = emit_synthetic_client_module(&program, &manifest, authored.as_ref(), t.mode)
        .unwrap_or_else(|e| panic!("merged {}: {}", t.name, e.render(&src)));
    fs::write(work.join(format!("{}.client.ts", t.name)), merged).unwrap();

    let split = emit_synthetic(
        &program,
        &manifest,
        authored.as_ref(),
        &format!("./{}.controller.ts", t.name),
        t.mode,
    )
    .unwrap_or_else(|e| panic!("split {}: {}", t.name, e.render(&src)));
    fs::write(work.join(format!("{}.controller.ts", t.name)), split.controller_ts).unwrap();
    fs::write(work.join(format!("{}.ssr.ts", t.name)), split.render_ts).unwrap();
}

fn write_harness(repo: &Path, work: &Path) {
    let r = repo.display();
    // @ship/sui-runtime → a shim (the SSR server surface + `view` + the types
    // @ship/sui-dom imports). @ship/sui-dom → its real source. @ship/deck/route
    // → the server route factory. mount.ts is imported by absolute path.
    fs::write(
        work.join("runtime-shim.ts"),
        format!(
            "export * from \"{r}/packages/sui/runtime/src/server.ts\";\n\
             export {{ view }} from \"{r}/packages/sui/runtime/src/view.ts\";\n\
             export type {{ Modifier, OnOptions }} from \"{r}/packages/sui/runtime/src/on.ts\";\n"
        ),
    )
    .unwrap();

    let cfg = format!(
        r#"import {{ defineConfig }} from 'vitest/config'
export default defineConfig({{
  test: {{ environment: 'jsdom', include: ['client.test.ts'] }},
  resolve: {{ alias: {{
    '@ship/sui-runtime': '{w}/runtime-shim.ts',
    '@ship/sui-dom': '{r}/packages/sui/dom/src/index.ts',
    '@ship/deck/route': '{r}/packages/deck/route/src/server.ts',
  }} }},
}})
"#,
        w = work.display(),
        r = r,
    );
    fs::write(work.join("vitest.config.ts"), cfg).unwrap();

    let mount_path = format!("{r}/packages/sui/runtime/src/mount.ts");
    let server_dom = format!("{r}/packages/sui/runtime/src/server-dom.ts");
    let test = format!(
        r#"import {{ describe, it, expect }} from 'vitest'
import {{ mount }} from '{mount}'
import {{ installServerDom }} from '{server_dom}'

// A minimal ViewPayload for a pre-resolved client module + given SSR html.
function payload(name: string, html: string) {{
  return {{ name, html, css: '', scope: {{}}, moduleUrl: '', params: {{}} }} as any
}}

// Reproduce the ship-js SSR HTML by running the SSR render against the shared
// server-DOM — the SAME code ship-js runs, just in node. Swaps globalThis.document
// for the server-DOM during the render, then restores jsdom's document.
async function ssrHtml(name: string): Promise<string> {{
  const saved = (globalThis as any).document
  const srv = installServerDom()
  try {{
    const mod = await import(`./${{name}}.ssr.ts`)
    const container = srv.container()
    const dispose = mod.mount(container)
    const html = srv.html(container)
    dispose()
    return html
  }} finally {{
    ;(globalThis as any).document = saved
  }}
}}

// Strip insignificant whitespace BETWEEN tags so server-DOM vs jsdom
// serialization differences don't mask structural equality.
function norm(html: string): string {{
  return html.replace(/>\s+</g, '><').replace(/\s+/g, ' ').trim()
}}

describe('synthetic CLIENT contract — CSR (__mount via mount.ts)', () => {{
  it('hub / renders its text binding client-side', async () => {{
    const mod = await import('./hub.client.ts')
    const el = document.createElement('div')
    const handle = await mount(el, payload('hub', ''), {{ module: mod as any }})
    expect(el.querySelector('h1')?.textContent).toBe('Hub')
    expect(el.textContent).toContain("Ship's primary app")
    handle.dispose()
  }})

  it('badge (a real component) mounts and stays reactive', async () => {{
    const mod = await import('./badge.client.ts')
    const el = document.createElement('div')
    const handle = await mount(el, payload('badge', ''), {{ module: mod as any }})
    const btn = el.querySelector('button') as HTMLButtonElement
    expect(btn).toBeTruthy()
    expect(btn.textContent).toBe('0')
    btn.click()
    expect(btn.textContent).toBe('1')  // signal write → re-render (live reactivity)
    handle.dispose()
  }})

  it('board control-flow builds + reacts client-side', async () => {{
    const mod = await import('./board.client.ts')
    const el = document.createElement('div')
    const handle = await mount(el, payload('board', ''), {{ module: mod as any }})
    expect(el.querySelectorAll('li').length).toBe(2)
    expect(el.textContent).toContain('Alpha')
    expect(el.querySelector('demo-chip')).toBeTruthy()
    expect(el.querySelectorAll('li.flagged').length).toBe(1)
    handle.dispose()
  }})
}})

describe('isomorphic-render parity / clean hydration (acceptance #3)', () => {{
  it('board: client __mount build is structurally identical to the ship-js SSR tree', async () => {{
    const html_ssr = await ssrHtml('board')
    const mod = await import('./board.client.ts')
    const el = document.createElement('div')
    const handle = await mount(el, payload('board', ''), {{ module: mod as any }})
    const html_csr = el.innerHTML
    // The SAME render server-side and client-side → no hydration mismatch.
    expect(norm(html_csr)).toBe(norm(html_ssr))
    handle.dispose()
  }})

  it('showcase (a real view-host view): mount.ts adopts the ship-js SSR HTML with no mismatch', async () => {{
    const html_ssr = await ssrHtml('showcase')
    const mod = await import('./showcase.client.ts')
    // Seed the container with the SSR HTML, as the view-host shell would.
    const el = document.createElement('div')
    el.innerHTML = html_ssr
    const before = norm(el.innerHTML)
    const handle = await mount(el, payload('showcase', html_ssr), {{ module: mod as any }})
    // Static view: morph adopts in place, no __mount rebuild, no teardown —
    // the adopted DOM is unchanged (clean hydration).
    expect(norm(el.innerHTML)).toBe(before)
    expect(el.textContent).toContain('sui-area-chart')
    handle.dispose()
  }})
}})
"#,
        mount = mount_path,
        server_dom = server_dom,
    );
    fs::write(work.join("client.test.ts"), test).unwrap();
}

#[test]
fn client_contract_csr_and_hydration_in_jsdom() {
    let repo = repo_root();
    let vitest = repo.join("node_modules/.bin/vitest");
    if !vitest.exists() {
        eprintln!(
            "SKIP synthetic_client_jsdom: {} not found — run `bun install` to enable the jsdom proof.",
            vitest.display()
        );
        return;
    }

    let work = repo.join("target/synthetic-client-jsdom");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).unwrap();

    let targets = [
        Target {
            name: "hub",
            source_rel: "apps/hub/routes/page.sui",
            manifest_rel: HUB_MANIFEST,
            mode: CompileMode::View,
            controller_rel: Some("apps/hub/routes/controller.ts"),
        },
        Target {
            name: "badge",
            source_rel: "crates/deck/deck-conformance/app/components/badge/badge.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::Component,
            controller_rel: Some("crates/deck/deck-conformance/app/components/badge/badge.controller.ts"),
        },
        Target {
            name: "board",
            source_rel: "crates/deck/deck-conformance/app/views/board/page.sui",
            manifest_rel: CONF_MANIFEST,
            mode: CompileMode::View,
            controller_rel: Some("crates/deck/deck-conformance/app/views/board/controller.ts"),
        },
        Target {
            name: "showcase",
            source_rel: "views/showcase/page.sui",
            manifest_rel: VIEWS_MANIFEST,
            mode: CompileMode::View,
            controller_rel: None,
        },
    ];
    for t in &targets {
        synth(&repo, &work, t);
    }
    write_harness(&repo, &work);

    let status = Command::new(&vitest)
        .args(["run", "-c", "vitest.config.ts"])
        .current_dir(&work)
        .status()
        .expect("run vitest");
    assert!(
        status.success(),
        "the synthetic CLIENT contract must CSR + hydrate cleanly in jsdom"
    );
}
