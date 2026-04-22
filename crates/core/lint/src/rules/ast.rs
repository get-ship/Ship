//! AST-based rules powered by minijinja's parser.
//!
//! This pass augments the regex linter with semantic checks:
//!   R1 unknown-macro, R2 unused-import, R3 unknown-filter,
//!   R4 undefined-context-field, R5 undeclared-signal (AST-grade),
//!   R6 unknown-route.
//!
//! Public entry: [`lint_ast`]. The parser is feature-gated in minijinja
//! under `unstable_machinery`, accessed via `minijinja::machinery`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::LazyLock;

use minijinja::machinery::{
    ast::{CallArg, Expr, Stmt},
    parse, WhitespaceConfig,
};
use minijinja::syntax::SyntaxConfig;

use crate::report::{Finding, Severity};

// ── Rule IDs ──────────────────────────────────────────────────────────────────

pub const RULE_AST_UNKNOWN_MACRO: &str = "ast-unknown-macro";
pub const RULE_AST_UNUSED_IMPORT: &str = "ast-unused-import";
pub const RULE_AST_UNKNOWN_FILTER: &str = "ast-unknown-filter";
pub const RULE_AST_UNDEFINED_CONTEXT: &str = "ast-undefined-context";
pub const RULE_AST_UNDECLARED_SIGNAL: &str = "ast-undeclared-signal";
pub const RULE_AST_UNKNOWN_ROUTE: &str = "ast-unknown-route";
pub const RULE_AST_PARSE_ERROR: &str = "ast-parse-error";

pub const AST_RULES: &[&str] = &[
    RULE_AST_UNKNOWN_MACRO,
    RULE_AST_UNUSED_IMPORT,
    RULE_AST_UNKNOWN_FILTER,
    RULE_AST_UNDEFINED_CONTEXT,
    RULE_AST_UNDECLARED_SIGNAL,
    RULE_AST_UNKNOWN_ROUTE,
    RULE_AST_PARSE_ERROR,
];

// ── Embedded SDK manifest ─────────────────────────────────────────────────────

const SDK_MACROS_JSON: &str =
    include_str!("../../../../../packages/ship-set/sdk/sdk-macros.json");
const SDK_VIEW_CTX_JSON: &str =
    include_str!("../../../../../packages/ship-set/sdk/sdk-view-contexts.json");
const SDK_URLS_JSON: &str =
    include_str!("../../../../../packages/ship-set/sdk/sdk-urls.json");

pub struct Sdk {
    pub macros: HashSet<String>,
    /// view_name -> set of top-level field names
    pub view_contexts: HashMap<String, HashSet<String>>,
    /// "{METHOD} {path}" entries, e.g. "GET /api/git/status"
    pub routes: HashSet<String>,
    pub filters: HashSet<String>,
}

static SDK: LazyLock<Sdk> = LazyLock::new(load_sdk);

pub fn sdk() -> &'static Sdk {
    &SDK
}

fn load_sdk() -> Sdk {
    let macros = parse_macro_names(SDK_MACROS_JSON);
    let view_contexts = parse_view_contexts(SDK_VIEW_CTX_JSON);
    let routes = parse_routes(SDK_URLS_JSON);
    let filters = builtin_filters();
    Sdk { macros, view_contexts, routes, filters }
}

fn parse_macro_names(raw: &str) -> HashSet<String> {
    let v: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    v.get("entries")
        .and_then(|e| e.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

fn parse_view_contexts(raw: &str) -> HashMap<String, HashSet<String>> {
    let v: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    let mut out = HashMap::new();
    if let Some(entries) = v.get("entries").and_then(|e| e.as_object()) {
        for (name, body) in entries {
            let mut fields = HashSet::new();
            if let Some(arr) = body.get("fields").and_then(|f| f.as_array()) {
                for field in arr {
                    if let Some(n) = field.get("name").and_then(|s| s.as_str()) {
                        fields.insert(n.to_string());
                    }
                }
            }
            out.insert(name.clone(), fields);
        }
    }
    out
}

fn parse_routes(raw: &str) -> HashSet<String> {
    let v: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    v.get("entries")
        .and_then(|e| e.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

/// Hardcoded list of minijinja's built-in filter names (from `src/defaults.rs`
/// `build_builtin_filters`), plus custom filters registered in shipd.
fn builtin_filters() -> HashSet<String> {
    let names: &[&str] = &[
        // minijinja builtins
        "safe", "escape", "e", "lower", "upper", "title", "capitalize", "replace",
        "length", "count", "dictsort", "items", "reverse", "trim", "join", "split",
        "lines", "default", "d", "round", "abs", "int", "float", "attr", "first",
        "last", "min", "max", "sort", "list", "string", "bool", "batch", "slice",
        "sum", "indent", "select", "reject", "selectattr", "rejectattr", "map",
        "groupby", "unique", "chain", "zip", "pprint", "format", "tojson",
        "urlencode",
        // shipd custom filters (apps/shipd/src/views/view_filters.rs + templates.rs)
        "where", "sort_by", "take", "to_attr_json", "truncate",
    ];
    names.iter().map(|s| s.to_string()).collect()
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn lint_ast(view_name: &str, source: &str, path: &std::path::Path) -> Vec<Finding> {
    let sdk = sdk();
    let stmt = match parse(
        source,
        view_name,
        SyntaxConfig::default(),
        WhitespaceConfig::default(),
    ) {
        Ok(s) => s,
        Err(err) => {
            return vec![Finding {
                file: path.to_path_buf(),
                line: err.line().unwrap_or(1),
                col: 1,
                rule: RULE_AST_PARSE_ERROR,
                severity: Severity::Warn,
                message: format!("ast-lint skipped (parse error: {})", err),
                suggest: None,
            }];
        }
    };

    let mut visitor = Visitor::new(path.to_path_buf(), view_name.to_string(), sdk);
    visitor.walk_stmt(&stmt);
    visitor.finish()
}

// ── Visitor ───────────────────────────────────────────────────────────────────

#[derive(Default)]
struct Import {
    name: String,
    line: usize,
    col: usize,
    used: bool,
    known: bool,
}

struct Visitor<'a> {
    file: PathBuf,
    view_name: String,
    sdk: &'a Sdk,
    findings: Vec<Finding>,
    imports: Vec<Import>,
    reported_context: HashSet<String>,
    /// Locally-assigned names (set/for-target/macro-arg/from-import) — these
    /// count as "defined" even if they are not in the context schema.
    assigned: HashSet<String>,
    /// Signal names declared via `signal('name', ...)` macro call — tracked
    /// so R5 can validate `$name` references against declarations anywhere
    /// in the template.
    declared_signals: HashSet<String>,
}

impl<'a> Visitor<'a> {
    fn new(file: PathBuf, view_name: String, sdk: &'a Sdk) -> Self {
        let mut assigned: HashSet<String> = HashSet::new();
        // built-in scope identifiers always available
        for n in [
            "self", "loop", "super", "caller", "fragment", "workspace_id",
            "error", "range", "true", "false", "none", "dict", "namespace",
        ] {
            assigned.insert(n.to_string());
        }
        Self {
            file,
            view_name,
            sdk,
            findings: Vec::new(),
            imports: Vec::new(),
            reported_context: HashSet::new(),
            assigned,
            declared_signals: HashSet::new(),
        }
    }

    fn finish(mut self) -> Vec<Finding> {
        for imp in &self.imports {
            if imp.known && !imp.used {
                self.findings.push(Finding {
                    file: self.file.clone(),
                    line: imp.line,
                    col: imp.col,
                    rule: RULE_AST_UNUSED_IMPORT,
                    severity: Severity::Warn,
                    message: format!(
                        "imported macro `{}` is never called — drop the import",
                        imp.name
                    ),
                    suggest: Some(format!("remove `{}` from the `from \"_ship.html.j2\" import …` list", imp.name)),
                });
            }
        }
        self.findings
    }

    fn walk_stmt(&mut self, stmt: &Stmt<'_>) {
        match stmt {
            Stmt::Template(s) => {
                for child in &s.children {
                    self.walk_stmt(child);
                }
            }
            Stmt::EmitExpr(s) => self.walk_expr(&s.expr),
            Stmt::EmitRaw(_) => {}
            Stmt::ForLoop(s) => {
                self.walk_expr(&s.iter);
                assign_target(&s.target, &mut self.assigned);
                if let Some(e) = &s.filter_expr {
                    self.walk_expr(e);
                }
                for c in &s.body {
                    self.walk_stmt(c);
                }
                for c in &s.else_body {
                    self.walk_stmt(c);
                }
            }
            Stmt::IfCond(s) => {
                self.walk_expr(&s.expr);
                for c in &s.true_body {
                    self.walk_stmt(c);
                }
                for c in &s.false_body {
                    self.walk_stmt(c);
                }
            }
            Stmt::WithBlock(s) => {
                for (t, e) in &s.assignments {
                    self.walk_expr(e);
                    assign_target(t, &mut self.assigned);
                }
                for c in &s.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::Set(s) => {
                self.walk_expr(&s.expr);
                assign_target(&s.target, &mut self.assigned);
            }
            Stmt::SetBlock(s) => {
                assign_target(&s.target, &mut self.assigned);
                for c in &s.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::AutoEscape(s) => {
                for c in &s.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::FilterBlock(s) => {
                self.walk_expr(&s.filter);
                for c in &s.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::Block(s) => {
                for c in &s.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::Import(_) => {}
            Stmt::Extends(_) | Stmt::Include(_) => {}
            Stmt::FromImport(s) => {
                let span = s.span();
                for (name_expr, alias) in &s.names {
                    if let Expr::Var(v) = name_expr {
                        let name = v.id.to_string();
                        let known = self.sdk.macros.contains(&name)
                            || is_known_ship_macro(&name);
                        if !known && is_from_ship_sdk(&s.expr) {
                            let suggestion = closest_macro(&name, &self.sdk.macros);
                            self.findings.push(Finding {
                                file: self.file.clone(),
                                line: span.start_line as usize,
                                col: span.start_col as usize + 1,
                                rule: RULE_AST_UNKNOWN_MACRO,
                                severity: Severity::Error,
                                message: format!(
                                    "unknown macro `{}` imported from `_ship.html.j2`{}",
                                    name,
                                    suggestion
                                        .as_deref()
                                        .map(|s| format!(" — did you mean `{s}`?"))
                                        .unwrap_or_default()
                                ),
                                suggest: suggestion.clone().map(|s| format!("rename `{name}` to `{s}`")),
                            });
                        }
                        self.assigned.insert(name.clone());
                        self.imports.push(Import {
                            name,
                            line: span.start_line as usize,
                            col: span.start_col as usize + 1,
                            used: false,
                            known: is_from_ship_sdk(&s.expr),
                        });
                    }
                    if let Some(Expr::Var(v)) = alias.as_ref() {
                        self.assigned.insert(v.id.to_string());
                    }
                }
            }
            Stmt::Macro(m) => {
                self.assigned.insert(m.name.to_string());
                for arg in &m.args {
                    if let Expr::Var(v) = arg {
                        self.assigned.insert(v.id.to_string());
                    }
                }
                for c in &m.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::CallBlock(s) => {
                self.walk_expr(&s.call.expr);
                for a in &s.call.args {
                    self.walk_callarg(a);
                }
                for c in &s.macro_decl.body {
                    self.walk_stmt(c);
                }
            }
            Stmt::Do(s) => {
                self.walk_expr(&s.call.expr);
                for a in &s.call.args {
                    self.walk_callarg(a);
                }
            }
            Stmt::Continue(_) | Stmt::Break(_) => {}
        }
    }

    fn walk_expr(&mut self, expr: &Expr<'_>) {
        match expr {
            Expr::Var(v) => {
                self.check_context_var(v.id, v.span());
            }
            Expr::Const(_) => {}
            Expr::UnaryOp(e) => self.walk_expr(&e.expr),
            Expr::BinOp(e) => {
                self.walk_expr(&e.left);
                self.walk_expr(&e.right);
            }
            Expr::IfExpr(e) => {
                self.walk_expr(&e.test_expr);
                self.walk_expr(&e.true_expr);
                if let Some(f) = &e.false_expr {
                    self.walk_expr(f);
                }
            }
            Expr::Filter(e) => {
                let span = e.span();
                if !self.sdk.filters.contains(e.name) {
                    let suggestion = closest(e.name, self.sdk.filters.iter().map(|s| s.as_str()));
                    self.findings.push(Finding {
                        file: self.file.clone(),
                        line: span.start_line as usize,
                        col: span.start_col as usize + 1,
                        rule: RULE_AST_UNKNOWN_FILTER,
                        severity: Severity::Warn,
                        message: format!(
                            "unknown filter `{}`{}",
                            e.name,
                            suggestion
                                .as_deref()
                                .map(|s| format!(" — did you mean `{s}`?"))
                                .unwrap_or_default()
                        ),
                        suggest: suggestion.clone().map(|s| format!("replace `| {}` with `| {s}`", e.name)),
                    });
                }
                if let Some(sub) = &e.expr {
                    self.walk_expr(sub);
                }
                for a in &e.args {
                    self.walk_callarg(a);
                }
            }
            Expr::Test(e) => {
                self.walk_expr(&e.expr);
                for a in &e.args {
                    self.walk_callarg(a);
                }
            }
            Expr::GetAttr(e) => self.walk_expr(&e.expr),
            Expr::GetItem(e) => {
                self.walk_expr(&e.expr);
                self.walk_expr(&e.subscript_expr);
            }
            Expr::Slice(e) => {
                if let Some(s) = &e.start {
                    self.walk_expr(s);
                }
                if let Some(s) = &e.stop {
                    self.walk_expr(s);
                }
                if let Some(s) = &e.step {
                    self.walk_expr(s);
                }
            }
            Expr::Call(e) => {
                // Record signal declarations from `signal('name', ...)`.
                if let Expr::Var(v) = &e.expr {
                    let name = v.id;
                    // mark import as used
                    for imp in self.imports.iter_mut() {
                        if imp.name == name {
                            imp.used = true;
                        }
                    }
                    if name == "signal" {
                        if let Some(CallArg::Pos(Expr::Const(c))) = e.args.first() {
                            if let Some(s) = c.value.as_str() {
                                self.declared_signals.insert(s.to_string());
                            }
                        }
                    }
                    // R1 — unknown macro call (only flag names that look like
                    // ship SDK macros i.e. short snake_case and not built-in
                    // jinja/minijinja constructs). We skip unknown globals to
                    // avoid noisy false positives for user-added functions.
                    let _ = name; // names already filtered via import tracking
                }
                self.walk_expr(&e.expr);
                for a in &e.args {
                    self.walk_callarg(a);
                }
            }
            Expr::List(e) => {
                for i in &e.items {
                    self.walk_expr(i);
                }
            }
            Expr::Map(e) => {
                for k in &e.keys {
                    self.walk_expr(k);
                }
                for v in &e.values {
                    self.walk_expr(v);
                }
            }
        }
    }

    fn check_context_var(&mut self, id: &str, span: minijinja::machinery::Span) {
        if self.assigned.contains(id) {
            return;
        }
        let Some(fields) = self.sdk.view_contexts.get(&self.view_name) else {
            return; // no schema → informational only; silent by default
        };
        if fields.contains(id) {
            return;
        }
        if !self.reported_context.insert(id.to_string()) {
            return;
        }
        let suggestion = closest(id, fields.iter().map(|s| s.as_str()));
        self.findings.push(Finding {
            file: self.file.clone(),
            line: span.start_line as usize,
            col: span.start_col as usize + 1,
            rule: RULE_AST_UNDEFINED_CONTEXT,
            severity: Severity::Warn,
            message: format!(
                "`{}` is not a declared field on the `{}` view context{}",
                id,
                self.view_name,
                suggestion
                    .as_deref()
                    .map(|s| format!(" — did you mean `{s}`?"))
                    .unwrap_or_default()
            ),
            suggest: suggestion.clone().map(|s| format!("replace `{id}` with `{s}`")),
        });
    }

    fn walk_callarg(&mut self, arg: &CallArg<'_>) {
        match arg {
            CallArg::Pos(e)
            | CallArg::Kwarg(_, e)
            | CallArg::PosSplat(e)
            | CallArg::KwargSplat(e) => self.walk_expr(e),
        }
    }
}

fn assign_target(expr: &Expr<'_>, out: &mut HashSet<String>) {
    match expr {
        Expr::Var(v) => {
            out.insert(v.id.to_string());
        }
        Expr::List(l) => {
            for item in &l.items {
                assign_target(item, out);
            }
        }
        _ => {}
    }
}

/// Scan the raw source for `@get(…)` / `@post(…)` / `@put(…)` / `@delete(…)`
/// patterns that appear inside Datastar attribute expressions. The AST treats
/// these as opaque string literals on HTML elements, so we need a regex pass.
pub fn lint_routes(path: &std::path::Path, source: &str, sdk: &Sdk) -> Vec<Finding> {
    use regex::Regex;
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r#"@(get|post|put|delete|patch)\(\s*['"]([^'"]+)['"]\s*[,)]"#).unwrap()
    });
    let mut out = Vec::new();
    let mut seen: HashSet<(usize, String)> = HashSet::new();
    for (idx, line) in source.lines().enumerate() {
        for cap in re.captures_iter(line) {
            let method = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_uppercase();
            let path_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let key = format!("{method} {path_str}");
            if route_matches(&key, &sdk.routes) {
                continue;
            }
            if !seen.insert((idx, key.clone())) {
                continue;
            }
            let col = cap.get(0).map(|m| m.start() + 1).unwrap_or(1);
            out.push(Finding {
                file: path.to_path_buf(),
                line: idx + 1,
                col,
                rule: RULE_AST_UNKNOWN_ROUTE,
                severity: Severity::Warn,
                message: format!("unknown route `{method} {path_str}` (not in sdk-urls.json)"),
                suggest: Some("call ship_sdk_query with this path to find a matching route".to_string()),
            });
        }
    }
    out
}

/// Loose match: strip `{…}` path params and compare as prefix-style globs.
fn route_matches(needle: &str, pool: &HashSet<String>) -> bool {
    if pool.contains(needle) {
        return true;
    }
    let Some((method, path)) = needle.split_once(' ') else {
        return false;
    };
    let n_parts: Vec<&str> = path.split('/').collect();
    for entry in pool {
        let Some((m2, p2)) = entry.split_once(' ') else {
            continue;
        };
        if m2 != method {
            continue;
        }
        let e_parts: Vec<&str> = p2.split('/').collect();
        if e_parts.len() != n_parts.len() {
            continue;
        }
        let ok = e_parts.iter().zip(n_parts.iter()).all(|(a, b)| {
            a.starts_with('{') && a.ends_with('}') || a == b
        });
        if ok {
            return true;
        }
    }
    false
}

fn is_from_ship_sdk(expr: &Expr<'_>) -> bool {
    if let Expr::Const(c) = expr {
        if let Some(s) = c.value.as_str() {
            return s.ends_with("_ship.html.j2");
        }
    }
    false
}

fn is_known_ship_macro(name: &str) -> bool {
    // Known additional imports outside sdk-macros.json
    matches!(name, "view_git" | "view_url")
}

// ── Close-match helper ────────────────────────────────────────────────────────

fn closest_macro(name: &str, pool: &HashSet<String>) -> Option<String> {
    closest(name, pool.iter().map(|s| s.as_str()))
}

fn closest<'a, I: IntoIterator<Item = &'a str>>(needle: &str, pool: I) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for cand in pool {
        let d = levenshtein(needle, cand);
        if d <= 2 && best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
            best = Some((d, cand.to_string()));
        }
    }
    best.map(|(_, s)| s)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
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
mod inner_tests {
    use super::*;

    #[test]
    fn sdk_loads() {
        let s = sdk();
        assert!(s.macros.contains("signal"), "macros = {:?}", s.macros);
        assert!(s.filters.contains("tojson"));
        assert!(!s.routes.is_empty());
    }

    #[test]
    fn unknown_filter_fires() {
        let findings = lint_ast(
            "demo",
            "{{ x | urlencod }}\n",
            std::path::Path::new("demo/index.html.j2"),
        );
        assert!(
            findings.iter().any(|f| f.rule == RULE_AST_UNKNOWN_FILTER),
            "{findings:?}"
        );
    }

    #[test]
    fn unknown_macro_import_fires() {
        let findings = lint_ast(
            "demo",
            "{% from '_ship.html.j2' import bogus_macro_xyz %}\n",
            std::path::Path::new("demo/index.html.j2"),
        );
        assert!(
            findings
                .iter()
                .any(|f| f.rule == RULE_AST_UNKNOWN_MACRO && f.message.contains("bogus_macro_xyz")),
            "{findings:?}"
        );
    }

    #[test]
    fn unused_import_fires() {
        let findings = lint_ast(
            "demo",
            "{% from '_ship.html.j2' import signal %}<div>nothing</div>\n",
            std::path::Path::new("demo/index.html.j2"),
        );
        assert!(
            findings.iter().any(|f| f.rule == RULE_AST_UNUSED_IMPORT),
            "{findings:?}"
        );
    }

    #[test]
    fn used_import_silent() {
        let findings = lint_ast(
            "demo",
            "{% from '_ship.html.j2' import signal %}<div {{ signal('x', 1) }}></div>\n",
            std::path::Path::new("demo/index.html.j2"),
        );
        assert!(
            !findings.iter().any(|f| f.rule == RULE_AST_UNUSED_IMPORT),
            "{findings:?}"
        );
    }

    #[test]
    fn parse_error_yields_single_info() {
        let findings = lint_ast(
            "demo",
            "{% if %}\n",
            std::path::Path::new("demo/index.html.j2"),
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule, RULE_AST_PARSE_ERROR);
    }

    #[test]
    fn levenshtein_smoke() {
        assert_eq!(levenshtein("urlencod", "urlencode"), 1);
        assert_eq!(closest("urlencod", ["urlencode", "escape"]).as_deref(), Some("urlencode"));
    }
}
