//! Single-source `mcp.*` ops — descriptors + dispatch via `ship_ops!`.

use super::{parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};

use ship_platform::services::mcp::state;
use ship_platform::services::mcp::state::ScopeTag;
use ship_platform::services::mcp::wire_types::{McpGetResponse, McpIndexResponse};

ship_ops! {
    MCP_OPS;
    {
        name: "mcp.list",
        desc: "List MCP server entries across scopes.",
        perm: "ship.mcp.read",
        args: &[],
        ret: McpIndexResponse,
        run: |_args, _state, files| Ok(McpIndexResponse { items: state::list(files) }),
    },
    {
        name: "mcp.get",
        desc: "Raw MCP entry at the requested scope.",
        perm: "ship.mcp.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        ret: McpGetResponse,
        run: |args, _state, files| {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, entry) = state::load_entry(files, id, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })?;
            Ok(McpGetResponse { id: id.to_string(), scope: ScopeTag::from(resolved), entry })
        },
    },
}
