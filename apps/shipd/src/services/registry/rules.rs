//! Single-source `rules.*` ops — descriptors + dispatch via `ship_ops!`.

use super::{parse_scope, required_str, RegistryError};
use crate::services::schema::{ArgDescriptor, PROJECT_SCOPE};

use ship_platform::services::rules::state;
use ship_platform::services::rules::state::ScopeTag;
use ship_platform::services::rules::wire_types::{RuleGetResponse, RulesIndexResponse};

ship_ops! {
    RULES_OPS;
    {
        name: "rules.list",
        desc: "List markdown rules across scopes.",
        perm: "ship.rules.read",
        args: &[],
        ret: RulesIndexResponse,
        run: |_args, _state, files| Ok(RulesIndexResponse { items: state::list(files) }),
    },
    {
        name: "rules.get",
        desc: "Raw rule body at the requested scope.",
        perm: "ship.rules.read",
        args: &[ArgDescriptor { name: "id", kind: "string", required: true }, PROJECT_SCOPE],
        ret: RuleGetResponse,
        run: |args, _state, files| {
            let id = required_str(args, "id")?;
            let scope = parse_scope(args)?;
            let (resolved, body) = state::load_body(files, id, scope).map_err(|e| match e {
                state::Error::NotFound(m) => RegistryError::not_found(m),
                other => RegistryError::service(other.to_string()),
            })?;
            Ok(RuleGetResponse { id: id.to_string(), scope: ScopeTag::from(resolved), body })
        },
    },
}
