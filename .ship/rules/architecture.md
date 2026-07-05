# Architecture

* Four layers: L0 (substrate), L1 (platform primitives), L2 (built-in apps), L3 (user apps). (Current code map: the `ship-architecture` skill — descriptive, not a source of intent.)
* L2 has no privileged access. Built-in apps consume the same SDK as L3 — no `crate::services::*` shortcuts in L2 code.
* Event-sourced storage by default. Use the kernel + projections. Other storage requires a registered exception with a written reason.
* One contract per boundary, one source of truth. SDK from one operation descriptor; primitive schemas validated by one validator.
* Agents are actors. Actor lifecycle and supervision are L0; agent profiles plus compilation to provider config are L1. Don't conflate.
* Capability, not mechanism: L1 declares results; L0 owns how. A port/method names a domain need, not an IO verb (`config.get`, not `fs.read`).
* L1 reaching for a primitive (fs/process/broker) with specific paths = driving mechanism — name the result, move the procedure behind a capability.
* Group capabilities by what callers want, not what things are.
* ConnectRPC is the only surface Ship exposes — a new capability is a Connect op or it doesn't exist; no new REST/CLI/MCP endpoints. (Integrating an external surface, e.g. an MCP server, is a capability; exposing a second one is not.)
* Consumers (UI, CLI, agents) compose capabilities; they never cause a backend surface to exist. A new op is justified by a capability, not a consumer's convenience.
* No ADR convention exists — don't suggest writing one. Architectural decisions that constrain future work live in the Helm blueprint (system/subsystem charters); smaller decisions update a rule file in `.ship/rules/`.
