# Architecture

* Four layers: L0 (substrate), L1 (platform primitives), L2 (built-in apps), L3 (user apps). The `ship-architecture` skill describes the codebase as it currently stands.
* L2 has no privileged access. Built-in apps consume the same SDK as L3 — no `crate::services::*` shortcuts in L2 code.
* Event-sourced storage by default. Use the kernel + projections. Other storage requires a registered exception with a written reason.
* One contract per boundary, one source of truth. SDK from one operation descriptor; primitive schemas validated by one validator.
* Hard file cap: 400 lines (soft cap in `engineering.md`). Use `// FILE_CAP_OVERRIDE: <reason>` only when genuinely warranted.
* Agents are actors. Actor lifecycle and supervision are L0; agent profiles plus compilation to provider config are L1. Don't conflate.
* Architectural decisions that constrain future work go in `docs/ship-adrs/` as a new ADR. Smaller decisions update a rule file in `.ship/rules/`.
