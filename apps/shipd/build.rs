//! Build-time codegen for the canonical Ship route registry.
//!
//! Reads `src/routes.rs`, parses every variant's `@route` / `@returns` /
//! `@body` doc-comment annotations, validates path-param → variant-field
//! correspondence, and emits three artifacts:
//!
//! 1. `$OUT_DIR/routes_generated.rs` — Rust URL builders and the
//!    `ShipRouterExt::ship_route` trait extension.
//! 2. `packages/sdk/src/generated/routes.ts` — TypeScript route catalog.
//! 3. `apps/shipd/templates/_urls.html.j2` — jinja macros for templates.
//!
//! Any structural violation (mismatched path params, unsupported type in
//! `@returns` / `@body`, missing `@route`) is a compile-time panic that
//! names the offending variant.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{Fields, Item, Meta};

fn main() {
    // Rerun triggers: the enum, this script, and the target paths we write.
    // (Touching only the generated files must NOT trigger re-build — we use
    // content-equality skips below.)
    println!("cargo:rerun-if-changed=src/routes.rs");
    println!("cargo:rerun-if-changed=build.rs");

    // Asset rerun triggers — `include_str!`/`include_bytes!` does NOT make
    // cargo aware of the file dependency. Without these declarations,
    // regenerating bundles via `node scripts/build-bundles.mjs` doesn't
    // invalidate the shipd build cache, so the next `cargo build` reuses
    // the binary with the OLD asset embedded — silently. Every gitignored
    // asset under `assets/` that any source `include_str!`s must be listed
    // here. Keep in sync with `serve_*` handlers in `src/views/`.
    let asset_paths: &[&str] = &[
        "assets/datastar.js",
        "assets/ship-elements.js",
        "assets/ship-git.js",
        "assets/ship-code-editor.js",
        "assets/ship-markdown-editor.js",
        "assets/ship-terminal.js",
        "assets/sui-runtime.js",
        "assets/sui-dom.js",
        "assets/sui-sdk-client.js",
        "assets/sui-components.js",
        "assets/sui-islands/sui-terminal.js",
        "assets/sui-islands/sui-command-bar.js",
        "assets/sui-custom-elements.json",
        "assets/view-tokens.css",
        "assets/views/agents/view.js",
        "assets/views/git/view.js",
        "assets/views/counter/view.js",
    ];
    for p in asset_paths {
        println!("cargo:rerun-if-changed={p}");
    }

    // Build identity (BUILD_GIT_SHA, BUILD_GIT_DIRTY, BUILD_TIMESTAMP,
    // BUILD_RUSTC_VERSION, BUILD_PROFILE) — shared across shipd, ship-cli,
    // and mcp via the build-info crate.
    build_info::emit_build_env();

    let routes_rs = Path::new("src/routes.rs");
    let source = fs::read_to_string(routes_rs)
        .unwrap_or_else(|e| panic!("build.rs: read src/routes.rs: {e}"));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|e| panic!("build.rs: parse src/routes.rs: {e}"));

    let variants = extract_route_variants(&file);

    let out_dir: PathBuf = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR must be set by cargo")
        .into();
    fs::write(out_dir.join("routes_generated.rs"), emit_rust(&variants))
        .expect("write routes_generated.rs");

    // Manifest dir is `apps/shipd`; compute workspace-relative outputs.
    let manifest_dir: PathBuf = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR")
        .into();

    // Template macros.
    let jinja_path = manifest_dir.join("templates").join("_urls.html.j2");
    write_if_changed(&jinja_path, &emit_jinja(&variants));

    // Ensure `apps/shell/dist/` exists so `include_dir!` in shell_assets.rs
    // can compile when the shell hasn't been built yet. Embeds an empty dir;
    // shell_assets returns a friendly "run `pnpm -F shell build`" page in
    // that case.
    let shell_dist = manifest_dir.join("..").join("shell").join("dist");
    if let Err(e) = fs::create_dir_all(&shell_dist) {
        panic!("build.rs: mkdir {shell_dist:?}: {e}");
    }

    // Ensure every gitignored asset that `include_str!` references exists.
    // Without this, a fresh clone that runs `cargo build` before the node
    // bundle script (build-bundles.mjs) fails with "couldn't read X". The
    // bundle script overwrites these placeholders with real content when it
    // runs; until then, asset endpoints serve an empty body which is the
    // same outcome as the file being absent at runtime.
    let assets_dir = manifest_dir.join("assets");
    let placeholders: &[&str] = &[
        "datastar.js",
        "ship-elements.js",
        "ship-git.js",
        "ship-code-editor.js",
        "ship-markdown-editor.js",
        "ship-terminal.js",
        "sui-runtime.js",
        "sui-dom.js",
        "sui-sdk-client.js",
        "sui-components.js",
        "sui-islands/sui-terminal.js",
        "sui-islands/sui-command-bar.js",
        "sui-custom-elements.json",
        "view-tokens.css",
        "views/agents/view.js",
        "views/git/view.js",
        "views/counter/view.js",
    ];
    for rel in placeholders {
        let path = assets_dir.join(rel);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                panic!("build.rs: mkdir {parent:?}: {e}");
            }
        }
        let initial = if rel.ends_with(".json") { "{}" } else { "" };
        if let Err(e) = fs::write(&path, initial) {
            panic!("build.rs: write placeholder {path:?}: {e}");
        }
    }

    // SDK TypeScript catalog. apps/shipd → ../../packages/sdk/src/generated/routes.ts
    let sdk_dir = manifest_dir
        .join("..")
        .join("..")
        .join("packages")
        .join("sdk")
        .join("src")
        .join("generated");
    if let Err(e) = fs::create_dir_all(&sdk_dir) {
        panic!("build.rs: mkdir {sdk_dir:?}: {e}");
    }
    write_if_changed(&sdk_dir.join("routes.ts"), &emit_typescript(&variants));
}

// ───────────────────────── Parsing ─────────────────────────────────────────

#[derive(Clone, Debug)]
struct RouteVariant {
    name: String,
    method: String,
    path: String,
    /// Declared path params in appearance order (e.g. `["id", "name"]`).
    path_params: Vec<String>,
    /// Unit | Named.
    kind: VariantKind,
    /// Variant field names (named variants only).
    fields: Vec<String>,
    returns: Option<TypeRef>,
    body: Option<TypeRef>,
}

#[derive(Clone, Debug)]
enum VariantKind {
    Unit,
    Named,
}

#[derive(Clone, Debug)]
enum TypeRef {
    /// A single identifier path, e.g. `Spec`.
    Plain(String),
    Vec(String),
    Option(String),
}

fn extract_route_variants(file: &syn::File) -> Vec<RouteVariant> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Enum(en) = item else { continue };
        if en.ident != "Route" {
            continue;
        }
        for v in &en.variants {
            let variant_name = v.ident.to_string();
            let mut route_line: Option<String> = None;
            let mut returns_line: Option<String> = None;
            let mut body_line: Option<String> = None;

            for attr in &v.attrs {
                let Meta::NameValue(nv) = &attr.meta else { continue };
                if !nv.path.is_ident("doc") {
                    continue;
                }
                // Extract the string literal from the doc attribute.
                let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                else {
                    continue;
                };
                let line = s.value();
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("@route ") {
                    route_line = Some(rest.trim().to_string());
                } else if let Some(rest) = trimmed.strip_prefix("@returns ") {
                    returns_line = Some(rest.trim().to_string());
                } else if let Some(rest) = trimmed.strip_prefix("@body ") {
                    body_line = Some(rest.trim().to_string());
                }
            }

            let route_line = route_line.unwrap_or_else(|| {
                panic!("build.rs: variant Route::{variant_name} is missing `/// @route METHOD \"/path\"` annotation")
            });
            let (method, path) = parse_route_line(&variant_name, &route_line);
            let path_params = extract_path_params(&path);

            let (kind, fields) = match &v.fields {
                Fields::Unit => (VariantKind::Unit, Vec::new()),
                Fields::Named(named) => {
                    let field_names: Vec<String> = named
                        .named
                        .iter()
                        .map(|f| {
                            f.ident
                                .as_ref()
                                .map(|i| i.to_string())
                                .expect("named field must have an ident")
                        })
                        .collect();
                    (VariantKind::Named, field_names)
                }
                Fields::Unnamed(_) => {
                    panic!(
                        "build.rs: Route::{variant_name} uses a tuple variant; only unit or named variants are supported"
                    );
                }
            };

            validate_params(&variant_name, &path_params, &fields);

            let returns = returns_line
                .as_deref()
                .map(|s| parse_type_ref(&variant_name, "returns", s));
            let body = body_line
                .as_deref()
                .map(|s| parse_type_ref(&variant_name, "body", s));

            out.push(RouteVariant {
                name: variant_name,
                method,
                path,
                path_params,
                kind,
                fields,
                returns,
                body,
            });
        }
    }
    if out.is_empty() {
        panic!("build.rs: no variants found on `enum Route` in src/routes.rs");
    }
    out
}

fn parse_route_line(variant: &str, line: &str) -> (String, String) {
    // Expected: `METHOD "/path"`
    let mut parts = line.splitn(2, char::is_whitespace);
    let method = parts
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    let rest = parts
        .next()
        .unwrap_or_else(|| panic!("build.rs: Route::{variant} @route needs a path"))
        .trim();
    let path = rest
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or_else(|| {
            panic!("build.rs: Route::{variant} @route path must be double-quoted: got {rest}")
        })
        .to_string();
    match method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" => {}
        other => panic!("build.rs: Route::{variant} unsupported HTTP method: {other}"),
    }
    (method, path)
}

fn extract_path_params(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = path;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let end = after
            .find('}')
            .unwrap_or_else(|| panic!("build.rs: unclosed `{{` in path: {path}"));
        let name = &after[..end];
        // Axum glob params (`*path`) still project to a String field; strip
        // the leading star for our bookkeeping.
        let name = name.trim_start_matches('*');
        if !name.is_empty() {
            out.push(name.to_string());
        }
        rest = &after[end + 1..];
    }
    out
}

fn validate_params(variant: &str, path_params: &[String], fields: &[String]) {
    let path_set: HashSet<&str> = path_params.iter().map(|s| s.as_str()).collect();
    let field_set: HashSet<&str> = fields.iter().map(|s| s.as_str()).collect();
    for p in &path_set {
        if !field_set.contains(p) {
            panic!(
                "build.rs: Route::{variant} path param `{{{p}}}` has no matching field — add a String field named `{p}` to the variant"
            );
        }
    }
    for f in &field_set {
        if !path_set.contains(f) {
            panic!(
                "build.rs: Route::{variant} has field `{f}` that is not referenced in the path — remove it or add `{{{f}}}` to the path"
            );
        }
    }
}

fn parse_type_ref(variant: &str, kind: &str, src: &str) -> TypeRef {
    let s = src.trim();
    if let Some(inner) = s.strip_prefix("Vec<").and_then(|r| r.strip_suffix('>')) {
        let t = inner.trim();
        ensure_plain_ident(variant, kind, t);
        return TypeRef::Vec(t.to_string());
    }
    if let Some(inner) = s.strip_prefix("Option<").and_then(|r| r.strip_suffix('>')) {
        let t = inner.trim();
        ensure_plain_ident(variant, kind, t);
        return TypeRef::Option(t.to_string());
    }
    ensure_plain_ident(variant, kind, s);
    TypeRef::Plain(s.to_string())
}

fn ensure_plain_ident(variant: &str, kind: &str, s: &str) {
    let ok = !s.is_empty()
        && s.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false)
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !ok {
        panic!(
            "build.rs: Route::{variant} @{kind} type `{s}` is not supported — allowlist is plain identifier, Vec<Ident>, Option<Ident>"
        );
    }
}

// ───────────────────────── Rust emitter ────────────────────────────────────

fn emit_rust(variants: &[RouteVariant]) -> String {
    let method_arms = variants
        .iter()
        .map(|v| {
            let pattern = match v.kind {
                VariantKind::Unit => format!("Route::{}", v.name),
                VariantKind::Named => format!("Route::{} {{ .. }}", v.name),
            };
            format!("            {pattern} => \"{}\",", v.method)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let path_arms = variants
        .iter()
        .map(|v| match v.kind {
            VariantKind::Unit => {
                format!(
                    "            Route::{} => \"{}\".to_string(),",
                    v.name, v.path
                )
            }
            VariantKind::Named => {
                let binds = v
                    .fields
                    .iter()
                    .map(|f| f.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                // Build the format string: every `{name}` becomes `{name}`
                // in Rust's format! naturally because Rust supports named
                // captures for fields of the same name via explicit args.
                // Use positional args to avoid collision with any `*name`
                // (glob) syntax we may have stripped.
                let mut fmt = v.path.clone();
                let mut args = Vec::new();
                for (i, name) in v.path_params.iter().enumerate() {
                    let placeholder = format!("{{{name}}}");
                    let star_placeholder = format!("{{*{name}}}");
                    let replacement = format!("{{{i}}}");
                    fmt = fmt.replace(&placeholder, &replacement);
                    fmt = fmt.replace(&star_placeholder, &replacement);
                    args.push(name.clone());
                }
                let args_str = args
                    .iter()
                    .map(|a| a.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "            Route::{} {{ {binds} }} => format!(\"{fmt}\", {args_str}),",
                    v.name
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Registration pattern list for ShipRouterExt::ship_route. Each variant
    // (Unit or Named) is matched by constructor shape — for Named variants
    // we dummy the fields with `String::new()` since the registration only
    // needs the pattern's *path template* (not a specific instance).
    //
    // To keep it simple we expose a helper `Route::template_path()` that
    // returns the Axum route pattern (with `{name}` intact). Registration
    // uses that and the method.
    let template_path_arms = variants
        .iter()
        .map(|v| {
            let pattern = match v.kind {
                VariantKind::Unit => format!("Route::{}", v.name),
                VariantKind::Named => format!("Route::{} {{ .. }}", v.name),
            };
            format!("            {pattern} => \"{}\",", v.path)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Instance constructor helper so callers can pass `Route::ListJobs` (no
    // fields) or build a template instance for Named variants. For Named
    // variants we emit a `Route::<variant>_template()` associated fn that
    // returns an instance with empty-string fields; `ship_route` only reads
    // the template_path, so this is safe.
    let template_ctors = variants
        .iter()
        .filter_map(|v| match v.kind {
            VariantKind::Unit => None,
            VariantKind::Named => {
                let fields = v
                    .fields
                    .iter()
                    .map(|f| format!("{f}: String::new()"))
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!(
                    "    /// Template instance (all path params empty). Used by `ship_route` for pattern-only registration.\n    #[allow(non_snake_case)]\n    pub fn {name}_template() -> Route {{ Route::{name} {{ {fields} }} }}",
                    name = v.name
                ))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"// @generated by apps/shipd/build.rs — DO NOT EDIT.

impl Route {{
    /// HTTP method as a static str (`"GET"`, `"POST"`, ...).
    pub fn method(&self) -> &'static str {{
        match self {{
{method_arms}
        }}
    }}

    /// Full URL path with `{{param}}` placeholders substituted. Returns a
    /// relative path (cookie-authenticated; the browser supplies origin).
    pub fn full_path(&self) -> String {{
        match self {{
{path_arms}
        }}
    }}

    /// Axum route pattern (`{{param}}` placeholders left intact). Used by
    /// router registration.
    pub fn template_path(&self) -> &'static str {{
        match self {{
{template_path_arms}
        }}
    }}

{template_ctors}
}}

/// Extension trait that registers a `Route` against an `axum::Router` using
/// the route's own `method()` + `template_path()`. Handler must satisfy
/// Axum's `Handler<T, S>` — the usual combination of extractors and a
/// `Future<Output = impl IntoResponse>`.
pub trait ShipRouterExt<S> {{
    fn ship_route<H, T>(self, route: Route, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static;
}}

impl<S> ShipRouterExt<S> for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{{
    fn ship_route<H, T>(self, route: Route, handler: H) -> Self
    where
        H: axum::handler::Handler<T, S>,
        T: 'static,
    {{
        let method = route.method();
        let path = route.template_path();
        let mr = match method {{
            "GET" => axum::routing::get(handler),
            "POST" => axum::routing::post(handler),
            "PUT" => axum::routing::put(handler),
            "DELETE" => axum::routing::delete(handler),
            "PATCH" => axum::routing::patch(handler),
            other => panic!("ship_route: unsupported method {{other}}"),
        }};
        self.route(path, mr)
    }}
}}
"#
    )
}

// ───────────────────────── Jinja emitter ───────────────────────────────────

fn emit_jinja(variants: &[RouteVariant]) -> String {
    let mut out = String::new();
    out.push_str(
        "{#- @generated by apps/shipd/build.rs — DO NOT EDIT.\n   URL builders for every Ship route. Import the ones you need:\n   {% from '_urls.html.j2' import view_pipelines, list_jobs %}\n   URLs are relative so they work in all contexts: daemon-direct,\n   shell app (same-origin proxy), and Studio (iframe or fragment).\n   The shell's fetch/EventSource shims rewrite absolute daemon URLs\n   to relative, so relative-by-default is the correct choice.\n-#}\n",
    );
    for v in variants {
        let macro_name = to_snake(&v.name);
        let params = v.path_params.join(", ");
        // Jinja macro body: substitute `{name}` with `{{ name }}`.
        let mut body = v.path.clone();
        for name in &v.path_params {
            body = body.replace(&format!("{{{name}}}"), &format!("{{{{ {name} }}}}"));
            body = body.replace(&format!("{{*{name}}}"), &format!("{{{{ {name} }}}}"));
        }
        out.push_str(&format!(
            "{{%- macro {macro_name}({params}) -%}}{body}{{%- endmacro -%}}\n"
        ));
    }
    out
}

// ───────────────────────── TypeScript emitter ──────────────────────────────

fn emit_typescript(variants: &[RouteVariant]) -> String {
    let mut out = String::new();
    out.push_str(
        "// @generated by apps/shipd/build.rs — DO NOT EDIT.\n// URL builders and metadata for every Ship daemon route. All URLs are\n// relative; authentication is provided by the `ship_session` cookie.\n\nexport type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';\n\nexport interface RouteMeta {\n  method: HttpMethod;\n  template: string;\n  returns?: string;\n  body?: string;\n}\n\n",
    );

    // Individual URL-builder functions.
    for v in variants {
        let fn_name = to_camel(&v.name);
        let params_ts = v
            .path_params
            .iter()
            .map(|p| format!("{p}: string"))
            .collect::<Vec<_>>()
            .join(", ");
        // Build the URL template literal: replace `{name}` with `${encodeURIComponent(name)}`.
        let mut body = v.path.clone();
        for name in &v.path_params {
            body = body.replace(
                &format!("{{{name}}}"),
                &format!("${{encodeURIComponent({name})}}"),
            );
            body = body.replace(
                &format!("{{*{name}}}"),
                &format!("${{encodeURIComponent({name})}}"),
            );
        }
        out.push_str(&format!("export function {fn_name}({params_ts}): string {{\n  return `{body}`;\n}}\n\n"));
    }

    // Central metadata catalog.
    out.push_str("export const routes = {\n");
    for v in variants {
        let key = to_camel(&v.name);
        let method = &v.method;
        let template = &v.path;
        let returns = v
            .returns
            .as_ref()
            .map(|t| format!(", returns: '{}'", render_type(t)))
            .unwrap_or_default();
        let body = v
            .body
            .as_ref()
            .map(|t| format!(", body: '{}'", render_type(t)))
            .unwrap_or_default();
        out.push_str(&format!(
            "  {key}: {{ method: '{method}', template: '{template}'{returns}{body} }} satisfies RouteMeta,\n"
        ));
    }
    out.push_str("} as const;\n");
    out
}

fn render_type(t: &TypeRef) -> String {
    match t {
        TypeRef::Plain(s) => s.clone(),
        TypeRef::Vec(s) => format!("Vec<{s}>"),
        TypeRef::Option(s) => format!("Option<{s}>"),
    }
}

// ───────────────────────── utils ───────────────────────────────────────────

fn to_snake(camel: &str) -> String {
    let mut out = String::with_capacity(camel.len() + 4);
    for (i, c) in camel.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn to_camel(pascal: &str) -> String {
    let mut chars = pascal.chars();
    match chars.next() {
        Some(c) => c.to_ascii_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

fn write_if_changed(path: &Path, content: &str) {
    let existing = fs::read_to_string(path).ok();
    if existing.as_deref() == Some(content) {
        return;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content).unwrap_or_else(|e| panic!("build.rs: write {path:?}: {e}"));
}
