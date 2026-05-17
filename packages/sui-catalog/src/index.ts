// Public API for `@ship/sui-catalog`. Importable by other workspace tools
// (apps/mcp's components_catalog could read these in-process; runtime
// shipd handlers read the on-disk JSON to avoid pulling Bun into Rust).

export type {
  CatalogDocument,
  CatalogEntry,
  ComponentOrigin,
  WorkspacePackage,
  CemAttr,
  CemSlot,
  CemEvent,
  CemMethod,
  CemCssProperty,
  CemCssPart,
  CemDeclaration,
  CemModule,
  CemRoot,
} from './types.js'
export {
  buildCatalogs,
  stringifyCatalog,
  originForNonPackagedSource,
} from './aggregator.js'
export type { AggregateOptions } from './aggregator.js'
export {
  discoverWorkspacePackages,
  workspaceGlobs,
  packIdForName,
} from './workspace.js'
