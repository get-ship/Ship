//! Skill directory inspection helpers shared by skill-listing tools.

use std::collections::HashMap;
use std::path::Path;

use compiler::ArtifactType;

pub(crate) fn collect_skill_files(skill_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if !skill_dir.is_dir() {
        return files;
    }
    collect_files_recursive(skill_dir, skill_dir, &mut files);
    files.sort();
    files
}

pub(crate) fn collect_files_recursive(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(root, &path, out);
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            let rel_str = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            out.push(rel_str);
        }
    }
}

pub(crate) fn collect_reference_docs(skill_dir: &Path) -> HashMap<String, String> {
    let docs_dir = skill_dir.join("references").join("docs");
    let mut docs = HashMap::new();
    if !docs_dir.is_dir() {
        return docs;
    }
    let mut doc_files = Vec::new();
    collect_files_recursive(skill_dir, &docs_dir, &mut doc_files);
    for rel_path in doc_files {
        let full = skill_dir.join(&rel_path);
        if let Ok(content) = std::fs::read_to_string(&full) {
            docs.insert(rel_path, content);
        }
    }
    docs
}

pub(crate) struct SkillFrontmatter {
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) stable_id: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) authors: Vec<String>,
    pub(crate) artifacts: Vec<ArtifactType>,
}

pub(crate) fn parse_skill_frontmatter(content: &str) -> SkillFrontmatter {
    let mut fm = SkillFrontmatter {
        name: None,
        description: None,
        stable_id: None,
        tags: Vec::new(),
        authors: Vec::new(),
        artifacts: Vec::new(),
    };

    if !content.starts_with("---") {
        return fm;
    }
    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(i) => i,
        None => return fm,
    };
    let block = &rest[..end];
    for line in block.lines() {
        if let Some(v) = line.strip_prefix("name:") {
            fm.name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("description:") {
            fm.description = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("stable-id:") {
            fm.stable_id = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("tags:") {
            fm.tags = parse_inline_array(v.trim());
        } else if let Some(v) = line.strip_prefix("authors:") {
            fm.authors = parse_inline_array(v.trim());
        } else if let Some(v) = line.strip_prefix("artifacts:") {
            fm.artifacts = parse_artifact_array(v.trim());
        }
    }
    fm
}

fn parse_artifact_array(s: &str) -> Vec<ArtifactType> {
    parse_inline_array(s)
        .into_iter()
        .filter_map(|v| match v.as_str() {
            "html" => Some(ArtifactType::Html),
            "markdown" => Some(ArtifactType::Markdown),
            "image" => Some(ArtifactType::Image),
            "json" => Some(ArtifactType::Json),
            "svg" => Some(ArtifactType::Svg),
            other => {
                tracing::warn!("unknown artifact type: {other}");
                None
            }
        })
        .collect()
}

fn parse_inline_array(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }
    trimmed[1..trimmed.len() - 1]
        .split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}
