// ship-data.ts — hand-owned data shapes for SDK ops whose Rust structs no
// longer carry specta derives (the registrations were retired when the
// write paths moved to the Connect/proto wire; see crates/xtask/src/types.rs
// and the `gen-types` registry comments). These are the SSE/mux payload
// shapes (serde snake_case) — NOT the proto messages in @ship/sdk's
// `gen/ship/ops/v1`, whose JSON casing differs. Shapes mirror the daemon's
// serde output byte-for-byte; update alongside the Rust structs:
//   GitOverview/GitDiff…   crates/core/ship-git/src/state.rs
//   ArtifactRecord         ship-platform artifacts
//   Collection*            ship-collection registry rows

export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key in string]?: JsonValue }

/** Metadata record returned by all artifact service methods. */
export type ArtifactRecord = {
  id: string
  name: string
  mime: string
  size: number
  path: string
  scope: string
  space_id: string | null
  run_id: string | null
  user_id: string | null
  content_hash: string | null
  expires_at: string | null
  owner_actor: string | null
  collection_id: string | null
  item_id: string | null
  created_at: string
}

/** One field row in a collection summary. */
export type CollectionFieldRecord = {
  name: string
  type: string
  field_type: string
  required: boolean
  values: string[] | null
  default: JsonValue | null
}

/** One row of `ship.collections.list()` — schema metadata + stats. */
export type CollectionSummary = {
  id: string
  name: string | null
  field_count: number
  fields: CollectionFieldRecord[]
  builtin: boolean
  item_count: number
  status_field: string | null
  status_values: string[]
  status_counts: Partial<{ [key in string]: number }>
  last_activity: string | null
}

export type BranchSummary = {
  name: string
  /** This branch is HEAD. */
  active: boolean
  upstream: string | null
  ahead: number
  behind: number
}

export type CommitSummary = {
  sha: string
  short_sha: string
  author: string
  date: string
  subject: string
  additions: number
  deletions: number
}

export type StatusFile = {
  path: string
  basename: string
  /** Single-char porcelain status: M A D R ? */
  status: string
  /** Design-system tone name fed to `ship-badge`. */
  tone: string
  label: string
}

export type DiffLine = {
  /** "add" | "del" | "context" — matches the component `type` attribute. */
  kind: string
  content: string
  old_num: number | null
  new_num: number | null
}

export type DiffHunk = {
  header: string
  lines: DiffLine[]
}

export type DiffFileView = {
  path: string
  additions: number
  deletions: number
  hunks: DiffHunk[]
}

export type ParsedDiff = {
  files: DiffFileView[]
  total_additions: number
  total_deletions: number
}

/** Diff payload returned by the `git.diff` registry op. */
export type GitDiff = {
  sha: string
  short_sha: string
  is_working: boolean
  diff_text: string
  parsed: ParsedDiff
}

/** Args for the `git.diff` registry op. */
export type GitDiffArgs = {
  /** Commit SHA, or `WORKING` for the working-tree diff. */
  commit: string
  /** Optional workspace scope (defaults to the daemon's project root). */
  workspace_id: string | null
}

/** Repo overview payload returned by the `git.overview` registry op. */
export type GitOverview = {
  workspace_id: string | null
  branch: string | null
  head_short: string | null
  clean: boolean
  total_changed: number
  staged: StatusFile[]
  modified: StatusFile[]
  untracked: StatusFile[]
  commits: CommitSummary[]
  branches: BranchSummary[]
  ahead: number
  behind: number
}
