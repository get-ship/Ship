# UI Composes Services — It Never Defines Them

The platform's capabilities are the source of truth. The UI is a consumer that
composes them. A UI component must never cause a backend surface to exist.

## The rule
- A frontend component that needs data composes **existing** platform services
  (search, discovery, the per-domain RPC services). It does not get a bespoke
  backend endpoint created to serve its convenience.
- A new backend service/RPC is justified by a **platform capability**, not by a
  UI feature wanting something to fetch. If the only consumer is one component
  and the "service" is a static catalog of UI affordances, it belongs in the
  component, not the backend.
- The RPC surface (`proto/ship/ops/v1`) is the public contract of what Ship
  *does*. It reflects platform capabilities — never UI layout, palette entries,
  or presentation state.

## The smell (how this gets violated)
A UI team ships a feature, needs an endpoint, and adds one to the daemon without
coordinating with the platform contract. Result: backend cruft that represents a
UI component, not a capability. The `/api/v1/commands` cmdk catalog was the
canonical example — deleted; the command bar composes search/discovery instead.

## Test before adding any backend surface
1. Is this a platform capability, or a UI convenience? Only the former earns a service.
2. Would more than one consumer (CLI, MCP agent, another view) use it? If only one
   component does, it's component-local.
3. Can it be composed from services that already exist? Then compose, don't create.
