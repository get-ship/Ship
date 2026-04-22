// File retired: the only consumer was project_api::pipelines (now empty).
// PipelineService does schema validation via `jsonc::strip_jsonc_comments`
// + serde, without line/column mapping. If editor-grade diagnostics return
// for pipelines, reintroduce the resolver alongside that work.
