//! Phase 10 — JSON schema emission.
//!
//! ```json
//! {
//!   "tag": "my-component",
//!   "version": "1.0",
//!   "props": {
//!     "prop1": { "type": "string", "default": "", "required": false },
//!     "prop2": { "type": "number", "default": 0, "required": false }
//!   },
//!   "events": {
//!     "select": { "payload": { "id": "string" } }
//!   },
//!   "slots": [
//!     { "name": "header" },
//!     { "name": "default" }
//!   ],
//!   "dependencies": {
//!     "components": ["ship-button"],
//!     "data": ["ship.agents.list"]
//!   }
//! }
//! ```
//!
//! Consumed by:
//! - The LSP (call-site validation, completion)
//! - Devtools panels (component-aware introspection)
//! - The benchmark runner (corpus generation)
//! - Other Sail compilers (when this component is referenced elsewhere)
