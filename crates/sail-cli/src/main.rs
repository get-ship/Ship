//! `sail` — command-line entry to the Sail compiler.
//!
//! See `docs/sail/10-dev-tooling.md § CLI`.
//!
//! Verbs (v1):
//! - `sail build [path]` — compile every `.sail` under `path` to `dist/`
//! - `sail check [path]` — parse + report diagnostics; no emit
//! - `sail new <name>`   — scaffold a starter project
//!
//! `dev` (watch + recompile loop) lives in the daemon for the Ship use
//! case; standalone OSS users get it next.

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use sail_compiler::{CompileOpts, Severity, compile_source, compile_source_with, tag_for};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "sail", version, about = "Sail framework CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Compile every `.sail` under <path> into <out>/.
    Build {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output directory for compiled artifacts.
        #[arg(short, long, default_value = "dist")]
        out: PathBuf,
    },
    /// Parse every `.sail` under <path> and report any errors.
    Check {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Scaffold a new Sail project directory.
    New {
        /// Project directory to create.
        name: PathBuf,
    },
    /// Migrate a `.ship` SFC to `.sail` (best-effort structural port).
    Migrate {
        /// Path to a `.ship` file.
        path: PathBuf,
        /// Output `.sail` path. Defaults to <path>.sail.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.cmd {
        Cmd::Build { path, out } => run_build(&path, &out),
        Cmd::Check { path } => run_check(&path),
        Cmd::New { name } => run_new(&name),
        Cmd::Migrate { path, out } => run_migrate(&path, out.as_deref()),
    };
    if let Err(e) = result {
        eprintln!("sail: error: {e:#}");
        std::process::exit(1);
    }
}

fn run_build(root: &Path, out: &Path) -> Result<()> {
    let files = collect_sail_files(root)?;
    if files.is_empty() {
        eprintln!("sail: no .sail files found under {}", root.display());
        return Ok(());
    }
    std::fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    let mut count = 0usize;
    for path in &files {
        let tag = tag_for(path);
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let result =
            compile_source(&source, &tag).with_context(|| format!("compile {}", path.display()))?;
        write_artifact(out, &tag, "js", &result.js)?;
        write_artifact(out, &tag, "html", &result.ssr)?;
        write_artifact(out, &tag, "d.ts", &result.dts)?;
        write_artifact(out, &tag, "schema.json", &result.schema)?;
        count += 1;
    }
    eprintln!("sail: built {count} component(s) → {}/", out.display());
    Ok(())
}

fn run_check(root: &Path) -> Result<()> {
    let files = collect_sail_files(root)?;
    if files.is_empty() {
        eprintln!("sail: no .sail files found under {}", root.display());
        return Ok(());
    }
    let mut errors = 0usize;
    let opts = CompileOpts::default();
    for path in &files {
        let source = std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?;
        let tag = tag_for(path);
        match compile_source_with(&source, &tag, &opts) {
            Ok(result) => {
                for d in &result.diagnostics {
                    let label = match d.severity {
                        Severity::Error => { errors += 1; "error" }
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    };
                    eprintln!("{label} [{}]: {}: {}", d.code, path.display(), d.message);
                    if let Some(h) = &d.help {
                        eprintln!("    help: {h}");
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {}: {e:#}", path.display());
                errors += 1;
            }
        }
    }
    if errors > 0 {
        bail!("{errors} file(s) failed to compile");
    }
    eprintln!("sail: {} file(s) ok", files.len());
    Ok(())
}

/// Best-effort structural port of a `.ship` SFC to `.sail`.
///
/// What it does:
/// - Reads the `.ship` source verbatim
/// - Detects whether each of the three blocks is present
/// - Wraps a bare-HTML body (no `<template>` block) into `<template>...</template>`
/// - Translates a small set of legacy directives:
///     - `data-text="$expr"` → `{expr}` interpolation
///     - `data-on-click="fn()"` → `on:click={fn}`
///     - `data-bind="$x"` on inputs → `bind:value={x}`
///   (more idioms left as TODO comments)
/// - Emits a `<!-- sail:migrate TODO -->` block at the top with a list
///   of things the operator should review by hand.
fn run_migrate(path: &Path, out: Option<&Path>) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let (sail, todos) = migrate_source(&source);
    let out_path = out
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| path.with_extension("sail"));
    std::fs::write(&out_path, &sail)
        .with_context(|| format!("write {}", out_path.display()))?;
    eprintln!("sail: migrated {} → {}", path.display(), out_path.display());
    if !todos.is_empty() {
        eprintln!("    {} item(s) need manual review:", todos.len());
        for t in &todos {
            eprintln!("    - {t}");
        }
    }
    Ok(())
}

fn migrate_source(source: &str) -> (String, Vec<String>) {
    let mut todos: Vec<String> = Vec::new();
    let has_script = source.contains("<script");
    let has_template = source.contains("<template");
    let has_style = source.contains("<style");

    // Translate inline directives (cheap, regex-free string passes).
    let mut translated = source.to_string();
    let mut translated_count = 0usize;

    // data-text="$expr" → {expr} interpolation in inner text
    while let Some(idx) = translated.find("data-text=\"$") {
        let rest = &translated[idx + "data-text=\"$".len()..];
        if let Some(end) = rest.find('"') {
            let expr = &rest[..end];
            // Find the enclosing tag and replace its inner text. For
            // simplicity we just delete the attribute and append the
            // interpolation as a TODO at the tag site.
            let attr_end = idx + "data-text=\"$".len() + end + 1;
            let mut new = String::new();
            new.push_str(&translated[..idx]);
            // Trim a leading space if present
            if new.ends_with(' ') {
                new.pop();
            }
            new.push_str(&format!("data-sail-migrated-text=\"{{{}}}\""  , expr));
            new.push_str(&translated[attr_end..]);
            translated = new;
            translated_count += 1;
        } else {
            break;
        }
    }
    if translated_count > 0 {
        todos.push(format!(
            "{translated_count} `data-text=\"$expr\"` directives parked as `data-sail-migrated-text` — move the value into the element's body as `{{expr}}`"
        ));
    }

    // data-on-click="fn()" → on:click={fn} (handler name extraction is
    // best-effort; complex expressions get parked as TODO).
    let mut click_count = 0usize;
    while let Some(idx) = translated.find("data-on-click=\"") {
        let rest = &translated[idx + "data-on-click=\"".len()..];
        if let Some(end) = rest.find('"') {
            let body = rest[..end].trim();
            let attr_end = idx + "data-on-click=\"".len() + end + 1;
            // Strip trailing `()` for handler name.
            let handler = body.trim_end_matches(')').trim_end_matches('(');
            let replacement = if handler.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$') {
                format!("on:click={{{handler}}}")
            } else {
                todos.push(format!("complex `data-on-click=\"{body}\"` left as TODO; rewrite as on:click={{...}}"));
                format!("data-sail-migrate-todo-on-click=\"{body}\"")
            };
            let mut new = String::new();
            new.push_str(&translated[..idx]);
            new.push_str(&replacement);
            new.push_str(&translated[attr_end..]);
            translated = new;
            click_count += 1;
        } else {
            break;
        }
    }
    if click_count > 0 {
        todos.push(format!("{click_count} `data-on-click` directive(s) translated"));
    }

    // data-bind="$x" → bind:value={x}
    let mut bind_count = 0usize;
    while let Some(idx) = translated.find("data-bind=\"$") {
        let rest = &translated[idx + "data-bind=\"$".len()..];
        if let Some(end) = rest.find('"') {
            let name = rest[..end].trim();
            let attr_end = idx + "data-bind=\"$".len() + end + 1;
            let mut new = String::new();
            new.push_str(&translated[..idx]);
            new.push_str(&format!("bind:value={{{name}}}"));
            new.push_str(&translated[attr_end..]);
            translated = new;
            bind_count += 1;
        } else {
            break;
        }
    }
    if bind_count > 0 {
        todos.push(format!("{bind_count} `data-bind` directive(s) translated to `bind:value`"));
    }

    if translated.contains("data-show") {
        todos.push("`data-show` directives need manual translation — use `{#if cond}…{/if}` or render conditionally".into());
    }
    if translated.contains("data-class") {
        todos.push("`data-class` directives need manual translation — use `class:NAME={cond}` (Sail v2)".into());
    }
    if translated.contains("data-for") {
        todos.push("`data-for` directives need manual translation — use `{#for item of list key={...}}`".into());
    }
    if translated.contains("{{") {
        todos.push("Jinja `{{ ... }}` interpolations need translation — Sail uses single-brace `{ ... }`".into());
    }
    if translated.contains("onConnect(") {
        todos.push("`onConnect(el => …)` should become a `mount { … }` block; `el` is available inside".into());
    }
    if !translated.contains("state ")
        && !translated.contains("derived ")
        && !translated.contains("prop ")
    {
        todos.push("script has no Sail keyword declarations — add `state X = …`, `derived Y = …`, or `prop Z: T` as appropriate".into());
    }

    let mut out = String::new();
    out.push_str("<!-- sail:migrate — review the items reported by the migrator before shipping -->\n");

    if has_script {
        // Pass the script block through verbatim.
        out.push_str(extract_block(&translated, "script").unwrap_or("<script></script>"));
        out.push('\n');
    } else {
        out.push_str("<script>\n  // TODO: declare props/state/derived as needed\n</script>\n");
    }

    if has_template {
        out.push_str(extract_block(&translated, "template").unwrap_or("<template></template>"));
        out.push('\n');
    } else {
        // Wrap the non-script/style body as a template.
        let body = strip_known_blocks(&translated);
        out.push_str("<template>\n");
        out.push_str(body.trim());
        out.push_str("\n</template>\n");
    }

    if has_style {
        out.push_str(extract_block(&translated, "style").unwrap_or("<style></style>"));
        out.push('\n');
    }
    (out, todos)
}

/// Pull out a `<TAG ...>...</TAG>` block from a source string.
fn extract_block<'a>(source: &'a str, tag: &str) -> Option<&'a str> {
    let open_pat = format!("<{tag}");
    let close_pat = format!("</{tag}>");
    let start = source.find(&open_pat)?;
    let end = source[start..].find(&close_pat)? + start + close_pat.len();
    Some(&source[start..end])
}

fn strip_known_blocks(source: &str) -> String {
    let mut out = source.to_string();
    for tag in ["script", "template", "style"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(i) = out.find(&open) {
            if let Some(rel) = out[i..].find(&close) {
                let end = i + rel + close.len();
                out.replace_range(i..end, "");
            } else {
                break;
            }
        }
    }
    out
}

fn run_new(dir: &Path) -> Result<()> {
    if dir.exists() {
        bail!("{} already exists", dir.display());
    }
    std::fs::create_dir_all(dir.join("sail"))?;
    std::fs::write(dir.join("sail/app-counter.sail"), STARTER_COUNTER)?;
    std::fs::write(dir.join("README.md"), STARTER_README)?;
    eprintln!("sail: created {}", dir.display());
    eprintln!("    → cd {} && sail build", dir.display());
    Ok(())
}

fn collect_sail_files(root: &Path) -> Result<Vec<PathBuf>> {
    if root.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            std::fs::read_dir(&dir).with_context(|| format!("read_dir {}", dir.display()))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                // Skip dist/, node_modules/, .git/ and similar.
                let skip = p
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| matches!(n, "dist" | "node_modules" | ".git" | "target"))
                    .unwrap_or(false);
                if !skip {
                    stack.push(p);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("sail") {
                out.push(p);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn write_artifact(out: &Path, tag: &str, ext: &str, body: &str) -> Result<()> {
    let path = out.join(format!("{tag}.{ext}"));
    std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))
}

const STARTER_COUNTER: &str = r#"<script>
state count = 0
derived doubled = count() * 2
function inc() { count.set(count() + 1) }
</script>

<template>
<div class="counter">
  <p>Count: {count}</p>
  <p>Doubled: {doubled}</p>
  <button on:click={inc}>+1</button>
</div>
</template>

<style>
.counter { padding: 1rem; font-family: system-ui; }
button { padding: 0.5rem 1rem; }
</style>
"#;

const STARTER_README: &str = r#"# New Sail project

Run `sail build` to compile every `.sail` under this directory into `dist/`.

The starter component lives at `sail/app-counter.sail`. Each `.sail` file
becomes four artifacts in `dist/`: `<tag>.js`, `<tag>.html` (SSR),
`<tag>.d.ts`, and `<tag>.schema.json`.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn collect_finds_only_sail_files() {
        let dir = tmp_dir();
        std::fs::write(dir.path().join("a.sail"), "<template><div/></template>").unwrap();
        std::fs::write(dir.path().join("b.txt"), "ignore").unwrap();
        std::fs::create_dir_all(dir.path().join("nested")).unwrap();
        std::fs::write(
            dir.path().join("nested/c.sail"),
            "<template><div/></template>",
        )
        .unwrap();
        let mut found = collect_sail_files(dir.path()).unwrap();
        found.sort();
        assert_eq!(found.len(), 2);
        assert!(found[0].ends_with("a.sail"));
        assert!(found[1].ends_with("c.sail"));
    }

    #[test]
    fn collect_skips_dist_and_node_modules() {
        let dir = tmp_dir();
        for skip in ["dist", "node_modules", ".git", "target"] {
            std::fs::create_dir_all(dir.path().join(skip)).unwrap();
            std::fs::write(
                dir.path().join(skip).join("x.sail"),
                "<template><div/></template>",
            )
            .unwrap();
        }
        std::fs::write(dir.path().join("ok.sail"), "<template><div/></template>").unwrap();
        let found = collect_sail_files(dir.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("ok.sail"));
    }

    #[test]
    fn build_emits_four_artifacts_per_file() {
        let dir = tmp_dir();
        let src = dir.path().join("counter.sail");
        let mut f = std::fs::File::create(&src).unwrap();
        f.write_all(b"<script>state c = 0</script><template><span>{c}</span></template>")
            .unwrap();
        let out = dir.path().join("dist");
        run_build(dir.path(), &out).unwrap();
        assert!(out.join("sail-counter.js").exists());
        assert!(out.join("sail-counter.html").exists());
        assert!(out.join("sail-counter.d.ts").exists());
        assert!(out.join("sail-counter.schema.json").exists());
    }

    #[test]
    fn check_passes_clean_files_and_fails_broken() {
        let dir = tmp_dir();
        std::fs::write(dir.path().join("ok.sail"), "<template><span>ok</span></template>").unwrap();
        run_check(dir.path()).unwrap();

        std::fs::write(
            dir.path().join("bad.sail"),
            "<template><div></span></template>",
        )
        .unwrap();
        let err = run_check(dir.path()).unwrap_err();
        assert!(err.to_string().contains("failed to compile"));
    }

    #[test]
    fn migrate_translates_on_click_and_bind() {
        let src = r#"<script>const x = 1</script>
<button data-on-click="inc">+</button>
<input data-bind="$draft" />
<style></style>"#;
        let (out, todos) = migrate_source(src);
        assert!(out.contains("on:click={inc}"));
        assert!(out.contains("bind:value={draft}"));
        assert!(out.contains("<template>"));
        assert!(todos.iter().any(|t| t.contains("data-on-click")));
        assert!(todos.iter().any(|t| t.contains("bind:value")));
    }

    #[test]
    fn migrate_flags_unsupported_directives() {
        let src = "<template><div data-show=\"$x\" data-for=\"item in xs\">{{ name }}</div></template>";
        let (_out, todos) = migrate_source(src);
        assert!(todos.iter().any(|t| t.contains("data-show")));
        assert!(todos.iter().any(|t| t.contains("data-for")));
        assert!(todos.iter().any(|t| t.contains("Jinja")));
    }

    #[test]
    fn migrate_warns_about_onconnect() {
        let src = "<script>onConnect(el => el.focus())</script><template><div/></template>";
        let (_out, todos) = migrate_source(src);
        assert!(todos.iter().any(|t| t.contains("onConnect")));
    }

    #[test]
    fn migrate_writes_output_file() {
        let dir = tmp_dir();
        let src = dir.path().join("c.ship");
        std::fs::write(&src, "<template><span>hi</span></template>").unwrap();
        run_migrate(&src, None).unwrap();
        let out = src.with_extension("sail");
        assert!(out.exists());
        let body = std::fs::read_to_string(&out).unwrap();
        assert!(body.contains("<template>"));
    }

    #[test]
    fn new_scaffolds_starter() {
        let dir = tmp_dir();
        let proj = dir.path().join("myapp");
        run_new(&proj).unwrap();
        assert!(proj.join("sail/app-counter.sail").exists());
        assert!(proj.join("README.md").exists());
        // Re-running fails because it already exists.
        assert!(run_new(&proj).is_err());
    }
}
