//! Single-source `pipelines.*` ops — descriptors + dispatch via `ship_ops!`.

use super::wire::{PipelineGetResponse, PipelinesIndexResponse};
use super::{optional_usize, parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};
use crate::services::pipelines::focus::FocusCtx as PipelineFocusCtx;
use crate::services::pipelines::history::HistoryCtx;
use crate::services::pipelines::run_detail::RunDetailCtx;
use crate::services::pipelines::summaries::SummariesCtx as PipelinesSummariesCtx;

use ship_platform::services::pipelines::state;
use ship_platform::services::pipelines::state::ScopeTag;

ship_ops! {
    PIPELINES_OPS;
    {
        name: "pipelines.list",
        desc: "List pipeline definitions across scopes.",
        perm: "ship.pipelines.read",
        args: &[],
        ret: PipelinesIndexResponse,
        run: |_args, _state, files| Ok(PipelinesIndexResponse { items: state::list(files) }),
    },
    {
        name: "pipelines.get",
        desc: "Raw pipeline doc at the requested scope.",
        perm: "ship.pipelines.read",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }, PROJECT_SCOPE],
        ret: PipelineGetResponse,
        run: |args, _state, files| {
            let n = required_str(args, "name")?;
            let scope = parse_scope(args)?;
            let (resolved, ext, doc) = state::load_doc(files, n, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })?;
            Ok(PipelineGetResponse {
                name: n.to_string(),
                scope: ScopeTag::from(resolved),
                format: ext,
                doc,
            })
        },
    },
    {
        name: "pipelines.summaries",
        desc: "Index-view shape: defs + active/history + pills + agents.",
        perm: "ship.pipelines.read",
        args: &[],
        ret: PipelinesSummariesCtx,
        run: |args, state, _files| {
            let filter = crate::views::state::Filter {
                q: args.get("q").and_then(|v| v.as_str()).map(String::from),
                agent: args.get("agent").and_then(|v| v.as_str()).map(String::from),
                pipeline: args.get("pipeline").and_then(|v| v.as_str()).map(String::from),
                status: args.get("status").and_then(|v| v.as_str()).map(String::from),
            };
            let tab = args.get("tab").and_then(|v| v.as_str()).map(String::from);
            Ok(crate::services::pipelines::summaries::build(state.ship_dir(), filter, tab))
        },
    },
    {
        name: "pipelines.focus",
        desc: "Per-pipeline detail with active + other runs precomputed.",
        perm: "ship.pipelines.read",
        args: &[ArgDescriptor { name: "name", kind: "string", required: true }],
        ret: PipelineFocusCtx,
        run: |args, state, _files| {
            let n = required_str(args, "name")?;
            crate::services::pipelines::focus::build(n, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("pipeline `{n}`")))
        },
    },
    {
        name: "pipelines.run",
        desc: "Run detail with steps + timeline.",
        perm: "ship.pipelines.read",
        args: &[ArgDescriptor { name: "slug", kind: "string", required: true }],
        ret: RunDetailCtx,
        run: |args, state, _files| {
            let slug = required_str(args, "slug")?;
            crate::services::pipelines::run_detail::build(slug, state.ship_dir())
                .ok_or_else(|| RegistryError::not_found(format!("run `{slug}`")))
        },
    },
    {
        name: "pipelines.history",
        desc: "Paginated run history with pipeline filter options.",
        perm: "ship.pipelines.read",
        args: &[
            ArgDescriptor { name: "limit", kind: "number?", required: false },
            ArgDescriptor { name: "offset", kind: "number?", required: false },
            ArgDescriptor { name: "pipeline", kind: "string?", required: false },
        ],
        ret: HistoryCtx,
        run: |args, state, _files| {
            let q = args.get("q").and_then(|v| v.as_str()).map(String::from);
            let pipeline = args.get("pipeline").and_then(|v| v.as_str()).map(String::from);
            let offset = optional_usize(args, "offset", 0);
            let limit = optional_usize(args, "limit", 50);
            Ok(crate::services::pipelines::history::build(
                state.ship_dir(),
                q,
                pipeline,
                offset,
                limit,
            ))
        },
    },
}
