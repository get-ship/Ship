//! Single-source `skills.*` ops — descriptors + dispatch via `ship_ops!`.

use serde_json::json;

use super::wire::{SkillFileResponse, SkillsIndexResponse, SkillsSummariesResponse};
use super::{html_escape, parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};

use ship_platform::services::skills::state;
use ship_platform::services::skills::state::SkillDetail as RawSkillDetail;
use ship_platform::services::skills::view_state::SkillDetail as ViewSkillDetail;

ship_ops! {
    SKILLS_OPS;
    {
        name: "skills.list",
        desc: "List all skills as minimal IndexEntry rows.",
        perm: "ship.skills.read",
        args: &[],
        ret: SkillsIndexResponse,
        run: |_args, _state, files| Ok(SkillsIndexResponse { items: state::list(files) }),
    },
    {
        name: "skills.summaries",
        desc: "Rail shape: filtered + grouped + kind counts + tags.",
        perm: "ship.skills.read",
        args: &[
            ArgDescriptor { name: "q", kind: "string?", required: false },
            ArgDescriptor { name: "kind", kind: "string?", required: false },
            ArgDescriptor { name: "tag", kind: "string?", required: false },
        ],
        ret: SkillsSummariesResponse,
        run: |args, state, _files| Ok(build_summaries(args, state)),
    },
    {
        name: "skills.detail",
        desc: "Full view-shaped skill detail — authors, variables, files.",
        perm: "ship.skills.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }],
        ret: ViewSkillDetail,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            crate::views::state::load_skill(state.ship_dir(), id)
                .ok_or_else(|| RegistryError::not_found(format!("skill `{id}`")))
        },
    },
    {
        name: "skills.file",
        desc: "HTML-escaped file content within a skill dir.",
        perm: "ship.skills.read",
        args: &[
            ArgDescriptor { name: "id", kind: "string", required: true },
            ArgDescriptor { name: "path", kind: "string", required: true },
        ],
        ret: SkillFileResponse,
        run: |args, state, _files| {
            let id = required_str(args, "id")?;
            let path = required_str(args, "path")?;
            let content = crate::views::state::load_skill_file(state.ship_dir(), id, path)
                .ok_or_else(|| RegistryError::not_found(format!("file `{path}` in skill `{id}`")))?;
            // HTML-escape: the user-view env runs AutoEscape::None, so untrusted
            // file content must be pre-escaped at the service boundary.
            let escaped = html_escape(&content);
            let name = path.rsplit('/').next().unwrap_or(path).to_string();
            let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
            Ok(SkillFileResponse { path: path.to_string(), name, ext, content: escaped })
        },
    },
    {
        name: "skills.get",
        desc: "Raw skill detail at the requested scope.",
        perm: "ship.skills.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        ret: RawSkillDetail,
        run: |args, _state, files| {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            state::load(files, id, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })
        },
    },
}

/// The `skills.summaries` rail shape — filtered + grouped + kind counts + tags.
/// Byte-identical to the prior anonymous `json!`.
fn build_summaries(
    args: &serde_json::Value,
    state: &crate::ship_runtime::ApiState,
) -> SkillsSummariesResponse {
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
        for t in &s.tags {
            tag_set.insert(t.clone());
        }
    }
    let filtered: Vec<_> = all
        .iter()
        .filter(|s| {
            let k = serde_json::to_value(s.kind).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
            let kind_ok = kind == "all" || kind == k;
            let hay = format!("{} {}", s.name, s.description).to_lowercase();
            let q_ok = needle.is_empty() || hay.contains(&needle);
            let tag_ok = tag.is_empty() || s.tags.iter().any(|t| t == tag);
            kind_ok && q_ok && tag_ok
        })
        .cloned()
        .collect();
    let mut by_source: std::collections::BTreeMap<String, Vec<_>> = Default::default();
    for s in &filtered {
        by_source.entry(s.source.clone()).or_default().push(s.clone());
    }
    let groups: Vec<_> = by_source
        .into_iter()
        .map(|(source, skills)| json!({ "source": source, "skills": skills }))
        .collect();
    let kind_counts = vec![
        json!({ "id": "all", "label": "All", "count": total, "selected": kind == "all" }),
        json!({ "id": "smart", "label": "Smart", "count": smart, "selected": kind == "smart" }),
        json!({ "id": "docs", "label": "Docs", "count": docs, "selected": kind == "docs" }),
    ];
    let filtered_count = filtered.len();
    SkillsSummariesResponse {
        items: all,
        filtered,
        groups,
        kind_counts,
        tags: tag_set,
        total,
        filtered_count,
        q: q.to_string(),
        active_kind: kind.to_string(),
        active_tag: tag.to_string(),
    }
}
