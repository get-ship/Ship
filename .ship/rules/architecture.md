# Architecture

* Four layers: L0 (substrate), L1 (platform primitives), L2 (built-in apps), L3 (user apps). The charter at `docs/ship-architecture/references/docs/charter.md` is canonical for intent; the rest of the ship-architecture skill describes the codebase as it currently stands.
* L2 has no privileged access. Built-in apps consume the same SDK as L3 — no `crate::services::*` shortcuts in L2 code.
* Event-sourced storage by default. Use the kernel + projections. Other storage requires a registered exception with a written reason.
* One contract per boundary, one source of truth. SDK from one operation descriptor; primitive schemas validated by one validator.
* Hard file cap: 400 lines. Use `// FILE_CAP_OVERRIDE: <reason>` only when genuinely warranted.
* Agents are actors. Actor lifecycle and supervision are L0; agent profiles plus compilation to provider config are L1. Don't conflate.
* No ADR convention exists yet. Don't suggest writing one. The charter is the canonical constraint document; new architectural decisions update the charter or extend a rule file.
