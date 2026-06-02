// Types for the vendored dagre layered-layout core (./dagre.mjs).
// MIT — see ./LICENSE. Hand-prepared 2026-06-02.

/** Layout-relevant node attributes set/read on the graph. */
export interface DagreNodeAttrs {
  width: number
  height: number
  /** Assigned by layout(): the layered rank (0 = first rank). */
  rank?: number
  /** Assigned by layout(): center coordinates in world units. */
  x?: number
  y?: number
}

/** Rank direction: left→right, right→left, top→bottom, bottom→top. */
export type RankDir = 'LR' | 'RL' | 'TB' | 'BT'

export interface DagreOptions {
  rankdir?: RankDir
  /** Gap between adjacent nodes within a rank (world units). */
  nodesep?: number
  /** Gap between adjacent ranks (world units). */
  ranksep?: number
}

/** The public graph description handed to layout(). */
export interface DagreGraph {
  nodes: Map<string, DagreNodeAttrs>
  edges: Array<{ v: string; w: string }>
  options?: DagreOptions
}

/** A laid-out node: center coords + assigned rank. */
export interface DagreResult {
  x: number
  y: number
  rank: number
}

/** A minimal directed graph used internally; exported for parity with dagre. */
export declare class Graph {
  setNode(id: string, attrs?: Partial<DagreNodeAttrs>): this
  hasNode(id: string): boolean
  node(id: string): DagreNodeAttrs
  removeNode(id: string): void
  setEdge(v: string, w: string): this
  removeEdge(v: string, w: string): void
  hasEdge(v: string, w: string): boolean
  nodes(): string[]
  successors(id: string): string[]
  predecessors(id: string): string[]
  neighbors(id: string): string[]
}

/** Run the layered (Sugiyama) pipeline; returns a map of id → laid-out node. */
export declare function layout(g: DagreGraph): Map<string, DagreResult>

declare const _default: { layout: typeof layout; Graph: typeof Graph }
export default _default
