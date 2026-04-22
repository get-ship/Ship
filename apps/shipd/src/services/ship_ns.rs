//! The `ship.*` template namespace — minijinja Object bindings over
//! the service registry + request context. Injected into the render
//! context so templates can call services inline:
//!
//! ```jinja
//! {% set agents = ship.agents.list() %}
//! {% set agent = ship.agents.get(id=ship.route.id) %}
//! {% set diff = ship.git.diff(commit=ship.query.commit) %}
//! ```
//!
//! Calls go through `services::registry::call`. Errors raise
//! `InvalidOperation` — rendering fails with a useful message rather
//! than silently returning empty.

use minijinja::value::{Kwargs, Object, ObjectRepr, Value};
use minijinja::{Error, ErrorKind, State};
use std::sync::Arc;

use crate::rest_api::ApiState;
use crate::services::registry;

/// Root `ship` namespace. Exposes service domains + request context.
pub struct ShipNs {
    pub state: ApiState,
    /// Path params extracted from file-based routing, e.g. `{id: "foo"}`.
    pub route: Value,
    /// URL query string, e.g. `{commit: "deadbeef", limit: "50"}`.
    pub query: Value,
    /// Datastar signals sent with the request.
    pub signals: Value,
    /// Authenticated session — user id, workspace scope, etc.
    pub session: Value,
    /// Active workspace metadata.
    pub workspace: Value,
    /// Platform info — version, dev flag, scope, now timestamp.
    pub env: Value,
}

impl std::fmt::Debug for ShipNs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShipNs").finish_non_exhaustive()
    }
}

impl Object for ShipNs {
    fn repr(self: &Arc<Self>) -> ObjectRepr { ObjectRepr::Map }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let k = key.as_str()?;
        match k {
            // Service domain namespaces — each returns a Domain object
            // whose callable methods dispatch to the registry.
            "agents" | "skills" | "rules" | "mcp" | "permissions"
            | "pipelines" | "git" | "collections" | "specs" => {
                Some(Value::from_object(Domain {
                    state: self.state.clone(),
                    name: k.to_string(),
                }))
            }
            "route" => Some(self.route.clone()),
            "query" => Some(self.query.clone()),
            "signals" => Some(self.signals.clone()),
            "session" => Some(self.session.clone()),
            "workspace" => Some(self.workspace.clone()),
            "env" => Some(self.env.clone()),
            _ => None,
        }
    }
}

/// One service domain — `ship.agents`, `ship.git`, etc. Methods on the
/// domain (`list`, `get`, `diff`, …) dispatch through the registry.
pub struct Domain {
    state: ApiState,
    name: String,
}

impl std::fmt::Debug for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ship.{}", self.name)
    }
}

impl Object for Domain {
    fn call_method(
        self: &Arc<Self>,
        _state: &State<'_, '_>,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let args_json = extract_args(args).map_err(|msg| {
            Error::new(
                ErrorKind::InvalidOperation,
                format!("ship.{}.{method}: {msg}", self.name),
            )
        })?;
        let full = format!("{}.{method}", self.name);
        match registry::call(&full, &args_json, &self.state) {
            Ok(value) => Ok(Value::from_serialize(&value)),
            Err(e) => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("ship.{full}: [{code}] {msg}", code = e.code, msg = e.message),
            )),
        }
    }
}

/// Accept three calling conventions:
///   - `ship.agents.list()`                      → {}
///   - `ship.pipelines.get(name="foo")`          → {"name": "foo"}
///   - `ship.agents.get({id: "x", scope: "user"})` → the map verbatim
fn extract_args(args: &[Value]) -> Result<serde_json::Value, String> {
    let (positional, kwargs) = split_kwargs(args);
    if !positional.is_empty() {
        // A single map positional arg is accepted as the whole args object.
        if positional.len() == 1 && positional[0].kind() == minijinja::value::ValueKind::Map {
            return value_to_json(&positional[0]);
        }
        return Err(
            "pass args as kwargs (e.g. `get(id=\"x\")`) or a single map positional arg".into(),
        );
    }
    if let Some(kw) = kwargs {
        let mut obj = serde_json::Map::new();
        for key in kw.args() {
            let v = kw.get::<Value>(key).map_err(|e| e.to_string())?;
            obj.insert(key.to_string(), value_to_json(&v)?);
        }
        Ok(serde_json::Value::Object(obj))
    } else {
        Ok(serde_json::json!({}))
    }
}

fn split_kwargs(args: &[Value]) -> (Vec<Value>, Option<Kwargs>) {
    if let Some((last, rest)) = args.split_last() {
        if let Ok(kw) = Kwargs::try_from(last.clone()) {
            return (rest.to_vec(), Some(kw));
        }
    }
    (args.to_vec(), None)
}

fn value_to_json(v: &Value) -> Result<serde_json::Value, String> {
    // minijinja::Value → serde_json::Value via serde. Lossy on some
    // minijinja-specific types (Undefined, etc.) — they become null.
    serde_json::to_value(v).map_err(|e| e.to_string())
}
