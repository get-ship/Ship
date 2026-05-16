// View-authoring MCP/SSE stubs.
//
// This whole `mocks/` subtree is a build-time crutch — delete it (and
// the `initViewMocks()` call in main.ts) once the real MCP tools land
// from `job/view-authoring-backend`. See view-mcp.ts for the surface
// being faked.

export { initViewMocks, stopAnnotationTimer } from './view-mcp'
export type { ViewMcpStubs } from './view-mcp'
