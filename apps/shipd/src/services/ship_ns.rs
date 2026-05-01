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
            // Soft-fail namespace: every domain re-exposed but runtime
            // errors (not_found, service_error) come back as an `{error,
            // code, message}` object instead of raising. Author bugs
            // (unknown_service, bad_args) still raise.
            //
            //   {% set spec = ship.try.specs.detail(id=ship.route.id) %}
            //   {% if spec.error %} … {% else %} … {% endif %}
            "try" => Some(Value::from_object(TryNs { state: self.state.clone() })),
            // Service domain namespaces — each returns a Domain object
            // whose callable methods dispatch to the registry.
            "agents" | "skills" | "rules" | "mcp" | "permissions"
            | "pipelines" | "git" | "collections" | "specs" | "events"
            | "settings" | "dashboard" | "chat" | "sessions" | "actors" | "mesh" => {
                Some(Value::from_object(Domain {
                    state: self.state.clone(),
                    name: k.to_string(),
                    soft: false,
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
///
/// `soft = true` comes from the `ship.try.*` path: runtime errors
/// (not_found, service_error) return an `{error, code, message}` object
/// instead of raising. Author bugs (bad_args, unknown_service) still
/// raise under both modes — they're not recoverable at render time.
pub struct Domain {
    state: ApiState,
    name: String,
    soft: bool,
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
            Err(e) => {
                // Soft-fail mode wraps runtime conditions as an object
                // templates can render. Author bugs always raise.
                if self.soft && matches!(e.code, "not_found" | "service_error") {
                    Ok(Value::from_serialize(serde_json::json!({
                        "error": true,
                        "code": e.code,
                        "message": e.message,
                        "op": format!("ship.{full}"),
                    })))
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("ship.{full}: [{code}] {msg}", code = e.code, msg = e.message),
                    ))
                }
            }
        }
    }
}

/// `ship.try` — soft-fail namespace. Re-exposes every domain as a
/// `Domain { soft: true }`. Call shape is identical to `ship.<domain>`.
pub struct TryNs {
    state: ApiState,
}

impl std::fmt::Debug for TryNs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ship.try")
    }
}

impl Object for TryNs {
    fn repr(self: &Arc<Self>) -> ObjectRepr { ObjectRepr::Map }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let k = key.as_str()?;
        match k {
            "agents" | "skills" | "rules" | "mcp" | "permissions"
            | "pipelines" | "git" | "collections" | "specs" | "events"
            | "settings" | "dashboard" | "chat" | "sessions" | "actors" | "mesh" => {
                Some(Value::from_object(Domain {
                    state: self.state.clone(),
                    name: k.to_string(),
                    soft: true,
                }))
            }
            _ => None,
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
    if let Some((last, rest)) = args.split_last()
        && let Ok(kw) = Kwargs::try_from(last.clone()) {
            return (rest.to_vec(), Some(kw));
        }
    (args.to_vec(), None)
}

fn value_to_json(v: &Value) -> Result<serde_json::Value, String> {
    // minijinja::Value → serde_json::Value via serde. Lossy on some
    // minijinja-specific types (Undefined, etc.) — they become null.
    serde_json::to_value(v).map_err(|e| e.to_string())
}
