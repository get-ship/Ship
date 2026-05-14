//! Pack-declaration rules (missing-pack-declaration, stale-pack-declaration).

use std::collections::HashSet;
use std::path::PathBuf;

use crate::report::{Finding, Severity};
use crate::source::Source;
use crate::{
    RULE_MISSING_PACK_DECLARATION, RULE_STALE_PACK_DECLARATION, ViewTarget,
};

pub(super) const PACKS: &[(&str, &[&str])] = &[
    ("editor", &["ship-code-editor", "ship-markdown-editor"]),
    ("terminal", &["ship-terminal"]),
];

pub(super) fn rule_pack_declarations(
    view: &ViewTarget,
    sources: &[Source],
    enabled: &[&'static str],
) -> Vec<Finding> {
    let mut out = Vec::new();

    let tag_open_re = regex::Regex::new(r"(?i)<(ship-[a-z0-9-]+)(?:[\s/>])").unwrap();
    let mut used_tags: HashSet<String> = HashSet::new();
    let mut first_tag_loc: std::collections::HashMap<&'static str, (PathBuf, usize, usize)> =
        std::collections::HashMap::new();

    for src in sources {
        for (idx, line) in src.lines.iter().enumerate() {
            for cap in tag_open_re.captures_iter(line) {
                let Some(tag_match) = cap.get(1) else { continue };
                let tag = tag_match.as_str();
                used_tags.insert(tag.to_string());
                if let Some(pack) = pack_for_tag(tag) {
                    first_tag_loc
                        .entry(pack)
                        .or_insert_with(|| (src.path.clone(), idx + 1, tag_match.start() + 1));
                }
            }
        }
    }

    let declared = read_manifest_packs(&view.dir).unwrap_or_default();

    if enabled.contains(&RULE_MISSING_PACK_DECLARATION) {
        for &(pack, tags) in PACKS {
            let view_uses_pack = tags.iter().any(|t| used_tags.contains(*t));
            if view_uses_pack && !declared.iter().any(|d| d == pack) {
                let (file, line, col) = first_tag_loc
                    .get(pack)
                    .cloned()
                    .unwrap_or_else(|| (view.dir.clone(), 1, 1));
                let used_list = tags
                    .iter()
                    .filter(|t| used_tags.contains(**t))
                    .map(|t| format!("<{t}>"))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push(Finding {
                    file,
                    line,
                    col,
                    rule: RULE_MISSING_PACK_DECLARATION,
                    severity: Severity::Error,
                    message: format!(
                        "view '{}' uses {} but manifest does not declare `packs: [{}]` — element will not upgrade without the bundle",
                        view.name, used_list, pack,
                    ),
                    suggest: Some(format!("add `\"packs\": [\"{pack}\"]` to the view manifest")),
                });
            }
        }
    }

    if enabled.contains(&RULE_STALE_PACK_DECLARATION) {
        for pack in &declared {
            let Some(tags) = PACKS
                .iter()
                .find(|(name, _)| *name == pack.as_str())
                .map(|(_, tags)| *tags)
            else {
                continue;
            };
            let view_uses_pack = tags.iter().any(|t| used_tags.contains(*t));
            if !view_uses_pack {
                out.push(Finding {
                    file: view.dir.join(manifest_filename(&view.dir)),
                    line: 1,
                    col: 1,
                    rule: RULE_STALE_PACK_DECLARATION,
                    severity: Severity::Warn,
                    message: format!(
                        "view '{}' declares `packs: [{}]` but no template uses its tags — drop the declaration or the view loads unused bundles",
                        view.name, pack,
                    ),
                    suggest: Some(format!("remove `\"{pack}\"` from `packs` in the manifest")),
                });
            }
        }
    }

    out
}

fn pack_for_tag(tag: &str) -> Option<&'static str> {
    for &(pack, tags) in PACKS {
        if tags.contains(&tag) {
            return Some(pack);
        }
    }
    None
}

fn read_manifest_packs(view_dir: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let yaml = view_dir.join("manifest.yaml");
    let jsonc = view_dir.join("manifest.jsonc");
    let json = view_dir.join("manifest.json");

    if yaml.exists() {
        let raw = std::fs::read_to_string(&yaml)?;
        let parsed: serde_yaml::Value = serde_yaml::from_str(&raw)?;
        return Ok(extract_packs(&value_from_yaml(&parsed)));
    }
    if jsonc.exists() {
        let raw = std::fs::read_to_string(&jsonc)?;
        let stripped = ship_substrate::fs_util::strip_jsonc_comments(&raw);
        let parsed: serde_json::Value = serde_json::from_str(&stripped)?;
        return Ok(extract_packs(&parsed));
    }
    if json.exists() {
        let raw = std::fs::read_to_string(&json)?;
        let parsed: serde_json::Value = serde_json::from_str(&raw)?;
        return Ok(extract_packs(&parsed));
    }
    Ok(Vec::new())
}

fn extract_packs(value: &serde_json::Value) -> Vec<String> {
    value
        .get("packs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn value_from_yaml(v: &serde_yaml::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

fn manifest_filename(view_dir: &std::path::Path) -> &'static str {
    if view_dir.join("manifest.yaml").exists() {
        "manifest.yaml"
    } else if view_dir.join("manifest.jsonc").exists() {
        "manifest.jsonc"
    } else {
        "manifest.json"
    }
}

#[cfg(test)]
mod tests {
    use super::read_manifest_packs;
    use tempfile::tempdir;

    /// JSONC manifests round-trip through the shared
    /// `ship_substrate::fs_util::strip_jsonc_comments` path: line and block
    /// comments are erased, trailing commas are dropped, and string contents
    /// (including URLs that contain `//`) survive untouched.
    #[test]
    fn jsonc_manifest_with_comments_and_trailing_commas() {
        let dir = tempdir().unwrap();
        let manifest = dir.path().join("manifest.jsonc");
        std::fs::write(
            &manifest,
            r#"{
  // line comment with // inside
  /* block
     comment */
  "docs": "https://example.com/manifest", // trailing line comment
  "packs": [
    "ship-code-editor",
    "ship-terminal", // trailing comma below
  ],
}
"#,
        )
        .unwrap();

        let packs = read_manifest_packs(dir.path()).expect("parses");
        assert_eq!(packs, vec!["ship-code-editor", "ship-terminal"]);
    }
}
