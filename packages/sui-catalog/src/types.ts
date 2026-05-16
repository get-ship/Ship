// Canonical catalog entry — one per `<sui-*>` custom-element declaration.
//
// The aggregator walks per-component `.custom-elements.json` files emitted
// by sui-build for every component in the workspace. Each entry carries
// the standard CEM declaration fields PLUS Ship-specific routing metadata
// (`package`, `private`, `sourcePath`, `bundleUrl`, `manifestUrl`) so the
// picker / MCP / future library-consumer UIs can resolve "where does this
// tag live and how do I load it" without re-reading the file tree.

export interface CemAttr {
  name: string
  description?: string
  type?: { text?: string }
  default?: string
}

export interface CemSlot {
  name: string
  description?: string
}

export interface CemEvent {
  name: string
  description?: string
  type?: { text?: string }
}

export interface CemMethod {
  name: string
  description?: string
  parameters?: Array<{ name: string; type?: { text?: string } }>
}

export interface CemCssProperty {
  name: string
  description?: string
  default?: string
}

export interface CemCssPart {
  name: string
  description?: string
}

export interface CemDeclaration {
  kind?: string
  name: string
  tagName?: string
  customElement?: boolean
  description?: string
  attributes?: CemAttr[]
  slots?: CemSlot[]
  events?: CemEvent[]
  members?: CemMethod[]
  cssProperties?: CemCssProperty[]
  cssParts?: CemCssPart[]
}

export interface CemModule {
  kind?: string
  path?: string
  declarations?: CemDeclaration[]
}

export interface CemRoot {
  schemaVersion?: string
  readme?: string
  modules?: CemModule[]
}

/** Origin of a component in the monorepo. `"@scope/name"` for a published
 *  pack; `"app:shell"` for shell-local components; `"view:<name>"` for
 *  view-local components. Used by the picker for grouping and by the
 *  daemon's pack-bundle handler to route bundle URLs. */
export type ComponentOrigin = string

export interface CatalogEntry {
  /** Canonical custom-element name, e.g. `sui-line-chart`. */
  tagName: string
  /** Origin package — npm name for packaged components, `app:shell` or
   *  `view:<name>` for non-packaged. Picker groups by this directly. */
  package: ComponentOrigin
  /** `true` if the origin package declares `"private": true` (or is a
   *  non-packaged origin). Used by the Published/All filter chip. */
  private: boolean
  /** Repo-relative path of the `.sui` source file. Used by deep-link
   *  "view source" affordances and by lint/scaffold tooling. */
  sourcePath: string
  /** URL the daemon serves the component bundle at. For packaged
   *  components this is `/_sui/packs/<pack-id>/<bundle-name>.js`; for
   *  non-packaged it's the existing `/_sui/sui-components/<tag>.js`. */
  bundleUrl: string
  /** URL the daemon serves the package's aggregate manifest at. Empty
   *  string for non-packaged origins. */
  manifestUrl: string
  /** Human description from the .sui's `<!-- comment -->` header, when the
   *  compiler captured one. May be empty. */
  description: string
  /** First line of description — convenience for grid tile labels. */
  firstDesc: string
  /** Standard CEM fields (passed through verbatim). */
  attributes: CemAttr[]
  slots: CemSlot[]
  events: CemEvent[]
  methods: CemMethod[]
  cssProperties: CemCssProperty[]
  cssParts: CemCssPart[]
}

export interface CatalogDocument {
  /** Stable schema version for downstream consumers. Bump when the entry
   *  shape changes incompatibly. */
  schemaVersion: '1'
  /** Number of entries — convenience header for sanity-checking. */
  count: number
  /** Sorted by `tagName` for byte-stable output. */
  entries: CatalogEntry[]
}

export interface WorkspacePackage {
  /** npm name (`@ship/charts`). */
  name: string
  /** Absolute path to the package root. */
  root: string
  /** Whether `package.json.private === true`. */
  private: boolean
  /** Value of `package.json.customElements` (if any), relative to root. */
  customElements?: string
}
