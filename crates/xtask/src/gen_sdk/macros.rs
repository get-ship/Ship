//! Parse `apps/shipd/templates/_ship.html.j2` — the canonical macro source —
//! into a structured per-macro manifest. Uses section dividers
//! (`{#- === Name === -#}`) to categorise each macro; uses the preceding
//! `{#- … -#}` doc block for description / args / example.

use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{json, Map, Value};

const SOURCE: &str = "apps/shipd/templates/_ship.html.j2";
const SINCE: &str = "0.1.0";

pub fn extract(repo_root: &Path) -> Result<Value> {
    let path = repo_root.join(SOURCE);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;

    // Locate all section dividers so we can tag each macro with its category.
    let divider = Regex::new(r"(?m)^\s*\{#-?\s*===\s*(.+?)\s*===\s*-?#\}").unwrap();
    let sections: Vec<(usize, String)> = divider
        .captures_iter(&text)
        .map(|c| {
            let m = c.get(0).unwrap();
            (m.start(), normalize_category(c.get(1).unwrap().as_str()))
        })
        .collect();

    // Collect every doc comment block so each macro can be paired to the
    // nearest preceding one separated only by whitespace.
    let comment = Regex::new(r"\{#-?([\s\S]*?)-?#\}").unwrap();
    let mut comments: Vec<(usize, usize, String)> = Vec::new();
    for m in comment.find_iter(&text) {
        let body = &text[m.start()..m.end()];
        // Skip section dividers — they're not doc blocks.
        if divider.is_match(body) {
            continue;
        }
        // Strip leading `{#-` and trailing `-#}` / `#}` to get inner body.
        let inner = body
            .trim_start_matches("{#-")
            .trim_start_matches("{#")
            .trim_end_matches("-#}")
            .trim_end_matches("#}")
            .to_string();
        comments.push((m.start(), m.end(), inner));
    }

    let macro_open = Regex::new(r"\{%-?\s*macro\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap();
    let mut entries = Vec::new();
    for m in macro_open.captures_iter(&text) {
        let whole = m.get(0).unwrap();
        let name = m.get(1).unwrap().as_str().to_string();
        let arg_start = whole.end();
        let Some(arg_end) = scan_balanced_paren(&text, arg_start) else {
            continue;
        };
        let raw_args = &text[arg_start..arg_end];
        let line = text[..whole.start()].bytes().filter(|&b| b == b'\n').count() + 1;

        // Doc = nearest preceding comment whose gap to macro start is whitespace.
        let mut doc: Option<&str> = None;
        for (_s, e, body) in comments.iter().rev() {
            if *e > whole.start() {
                continue;
            }
            let between = &text[*e..whole.start()];
            if between.chars().all(|c| c.is_whitespace()) {
                doc = Some(body);
            }
            break;
        }

        let category = sections
            .iter()
            .rev()
            .find(|(off, _)| *off < whole.start())
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| "other".into());

        let parsed = parse_doc(doc.unwrap_or(""));
        let params = parse_params(raw_args);
        let merged_params: Vec<Value> = params
            .iter()
            .map(|p| {
                let desc = parsed.arg_desc(&p.name);
                json!({
                    "name": p.name,
                    "type": infer_type(p.default.as_deref()),
                    "required": p.required,
                    "default": p.default.clone().map(Value::String).unwrap_or(Value::Null),
                    "description": desc,
                })
            })
            .collect();

        let signature = format!("{name}({})", raw_args.trim());

        let mut entry = Map::new();
        entry.insert("name".into(), Value::String(name.clone()));
        entry.insert("category".into(), Value::String(category));
        entry.insert("signature".into(), Value::String(signature));
        entry.insert("params".into(), Value::Array(merged_params));
        entry.insert("returns".into(), Value::String("html-attributes".into()));
        entry.insert(
            "description".into(),
            parsed
                .description
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        entry.insert(
            "summary".into(),
            parsed.summary.map(Value::String).unwrap_or(Value::Null),
        );
        entry.insert(
            "example".into(),
            parsed.example.map(Value::String).unwrap_or(Value::Null),
        );
        entry.insert("since".into(), Value::String(SINCE.into()));
        entry.insert("documented".into(), Value::Bool(doc.is_some()));
        entry.insert(
            "source".into(),
            Value::String(format!("{SOURCE}:{line}")),
        );
        entries.push((name, Value::Object(entry)));
    }

    let mut out = Map::new();
    for (name, e) in entries {
        out.insert(name, e);
    }
    Ok(Value::Object(out))
}

fn normalize_category(header: &str) -> String {
    header
        .to_lowercase()
        .replace('&', "and")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

fn scan_balanced_paren(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut i = start;
    while i < bytes.len() {
        let ch = bytes[i];
        if in_single {
            if ch == b'\'' && (i == 0 || bytes[i - 1] != b'\\') {
                in_single = false;
            }
        } else if in_double {
            if ch == b'"' && (i == 0 || bytes[i - 1] != b'\\') {
                in_double = false;
            }
        } else {
            match ch {
                b'\'' => in_single = true,
                b'"' => in_double = true,
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

struct Param {
    name: String,
    required: bool,
    default: Option<String>,
}

fn parse_params(raw: &str) -> Vec<Param> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    for ch in raw.chars() {
        if in_single {
            buf.push(ch);
            if ch == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            buf.push(ch);
            if ch == '"' {
                in_double = false;
            }
            continue;
        }
        match ch {
            '\'' => {
                in_single = true;
                buf.push(ch);
            }
            '"' => {
                in_double = true;
                buf.push(ch);
            }
            '(' | '[' | '{' => {
                depth += 1;
                buf.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                buf.push(ch);
            }
            ',' if depth == 0 => {
                push_param(&mut out, &buf);
                buf.clear();
            }
            _ => buf.push(ch),
        }
    }
    if !buf.trim().is_empty() {
        push_param(&mut out, &buf);
    }
    out
}

fn push_param(out: &mut Vec<Param>, raw: &str) {
    let text = raw.trim();
    if text.is_empty() {
        return;
    }
    if let Some((name, default)) = text.split_once('=') {
        out.push(Param {
            name: name.trim().to_string(),
            required: false,
            default: Some(default.trim().to_string()),
        });
    } else {
        out.push(Param {
            name: text.to_string(),
            required: true,
            default: None,
        });
    }
}

fn infer_type(default: Option<&str>) -> &'static str {
    let Some(d) = default else { return "any" };
    let d = d.trim();
    if d.starts_with('\'') || d.starts_with('"') {
        "string"
    } else if d == "true" || d == "false" {
        "boolean"
    } else if d == "none" {
        "null"
    } else if d.starts_with('{') {
        "object"
    } else if d.starts_with('[') {
        "array"
    } else if d.parse::<f64>().is_ok() {
        "number"
    } else {
        "any"
    }
}

struct ParsedDoc {
    summary: Option<String>,
    description: Option<String>,
    args: Vec<(String, String)>,
    example: Option<String>,
}

impl ParsedDoc {
    fn arg_desc(&self, name: &str) -> Value {
        self.args
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, d)| Value::String(d.clone()))
            .unwrap_or(Value::Null)
    }
}

fn parse_doc(raw: &str) -> ParsedDoc {
    if raw.trim().is_empty() {
        return ParsedDoc {
            summary: None,
            description: None,
            args: Vec::new(),
            example: None,
        };
    }
    // Dedent.
    let lines: Vec<&str> = raw.split('\n').collect();
    let base = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut dedented: Vec<String> = lines
        .iter()
        .map(|l| if l.len() >= base { l[base..].to_string() } else { l.to_string() })
        .collect();
    // Drop leading/trailing blanks.
    while dedented.first().map(|s| s.trim().is_empty()).unwrap_or(false) {
        dedented.remove(0);
    }
    while dedented.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        dedented.pop();
    }

    // Skip the signature header line.
    if !dedented.is_empty() {
        dedented.remove(0);
    }

    let mut mode = "body";
    let mut body = Vec::new();
    let mut args_lines: Vec<String> = Vec::new();
    let mut example_lines: Vec<String> = Vec::new();
    for line in dedented {
        let t = line.trim();
        if t.eq_ignore_ascii_case("args:") || t.eq_ignore_ascii_case("arg:") {
            mode = "args";
            continue;
        }
        if t.eq_ignore_ascii_case("example:") {
            mode = "example";
            continue;
        }
        match mode {
            "body" => body.push(line),
            "args" => args_lines.push(line),
            "example" => example_lines.push(line),
            _ => {}
        }
    }
    let description = {
        let joined = body.join("\n").trim().to_string();
        if joined.is_empty() { None } else { Some(joined) }
    };
    let summary = description.as_ref().map(|d| {
        d.split("\n\n")
            .next()
            .unwrap_or("")
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    });

    let args = parse_args_block(&args_lines);
    let example = {
        let joined = example_lines.join("\n").trim().to_string();
        if joined.is_empty() { None } else { Some(dedent(&joined)) }
    };
    ParsedDoc {
        summary,
        description,
        args,
        example,
    }
}

fn parse_args_block(lines: &[String]) -> Vec<(String, String)> {
    let non_empty: Vec<&String> = lines.iter().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.is_empty() {
        return Vec::new();
    }
    let base = non_empty
        .iter()
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out: Vec<(String, String)> = Vec::new();
    for raw in lines {
        if raw.trim().is_empty() {
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        let content = if raw.len() >= base { &raw[base..] } else { raw.as_str() };
        if indent == base
            && let Some((name, desc)) = content.split_once(':') {
                out.push((name.trim().to_string(), desc.trim().to_string()));
                continue;
            }
        if let Some(last) = out.last_mut() {
            last.1 = format!("{} {}", last.1, content.trim()).trim().to_string();
        }
    }
    out
}

fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let base = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| if l.len() >= base { &l[base..] } else { l })
        .collect::<Vec<_>>()
        .join("\n")
}
