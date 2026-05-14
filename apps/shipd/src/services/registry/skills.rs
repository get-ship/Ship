//! Registry dispatcher for `skills.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, html_escape, parse_scope, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "skills.list" => Ok(json!({
            "items": ship_platform::services::skills::state::list(files),
        })),
        "skills.summaries" => {
            let all = crate::views::state::list_skills(state.ship_dir());
            let q = args.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("all");
            let tag = args.get("tag").and_then(|v| v.as_str()).unwrap_or("");
            let needle = q.to_lowercase();
            let total = all.len();
            let mut smart = 0usize;
            let mut docs = 0usize;
            let mut tag_set: std::collections::BTreeSet<String> = Default::default();
            for s in &all {
                match serde_json::to_value(s.kind).ok().and_then(|v| v.as_str().map(String::from)).as_deref() {
                    Some("smart") => smart += 1,
                    Some("docs") => docs += 1,
                    _ => {}
                }
                for t in &s.tags { tag_set.insert(t.clone()); }
            }
            let filtered: Vec<_> = all.iter().filter(|s| {
                let k = serde_json::to_value(s.kind).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
                let kind_ok = kind == "all" || kind == k;
                let hay = format!("{} {}", s.name, s.description).to_lowercase();
                let q_ok = needle.is_empty() || hay.contains(&needle);
                let tag_ok = tag.is_empty() || s.tags.iter().any(|t| t == tag);
                kind_ok && q_ok && tag_ok
            }).cloned().collect();
            let mut by_source: std::collections::BTreeMap<String, Vec<_>> = Default::default();
            for s in &filtered { by_source.entry(s.source.clone()).or_default().push(s.clone()); }
            let groups: Vec<_> = by_source.into_iter()
                .map(|(source, skills)| json!({ "source": source, "skills": skills }))
                .collect();
            Ok(json!({
                "items": all,
                "filtered": filtered,
                "groups": groups,
                "kind_counts": [
                    { "id": "all", "label": "All", "count": total, "selected": kind == "all" },
                    { "id": "smart", "label": "Smart", "count": smart, "selected": kind == "smart" },
                    { "id": "docs", "label": "Docs", "count": docs, "selected": kind == "docs" },
                ],
                "tags": tag_set,
                "total": total,
                "filtered_count": filtered.len(),
                "q": q,
                "active_kind": kind,
                "active_tag": tag,
            }))
        }
        "skills.file" => {
            let id = required_str(args, "id")?;
            let path = required_str(args, "path")?;
            let content = crate::views::state::load_skill_file(state.ship_dir(), id, path)
                .ok_or_else(|| RegistryError::not_found(format!("file `{path}` in skill `{id}`")))?;
            // HTML-escape: user-view env runs AutoEscape::None so any
            // <script>/<style>/<link> in user content would otherwise
            // leak into the document. Pre-escape at the service boundary.
            let escaped = html_escape(&content);
            let name = path.rsplit('/').next().unwrap_or(path).to_string();
            let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
            Ok(json!({
                "path": path,
                "name": name,
                "ext": ext,
                "content": escaped,
            }))
        }
        "skills.detail" => {
            let id = required_str(args, "id")?;
            let detail = crate::views::state::load_skill(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("skill `{id}`")))?;
            serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))
        }
        "skills.get" => {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let detail = ship_platform::services::skills::state::load(files, id, scope)
                .map_err(|e| match e {
                    ship_platform::services::skills::state::Error::NotFound(m) => RegistryError::not_found(m),
                    other => RegistryError::service(other.to_string()),
                })?;
            Ok(serde_json::to_value(&detail).map_err(|e| RegistryError::service(e.to_string()))?)
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
