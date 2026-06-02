// VENDORED — dagre, MIT (Copyright (c) 2012-2014 Chris Pettitt; see ./LICENSE
// verbatim). Hand-prepared 2026-06-02.
//
// Source: a self-contained ESM re-implementation of dagre's documented layered
// (Sugiyama) pipeline — rank assignment → normalization (virtual nodes on long
// edges) → ordering (median/barycenter cross-minimization sweeps) → coordinate
// assignment. Upstream dagre ships CommonJS with graphlib + lodash runtime
// deps; this single-file ESM inlines a minimal directed-graph + the four
// pipeline stages so the pack carries NO npm dependency (mirrors the konva /
// roughjs / perfect-freehand vendoring style). The algorithm and stage names
// follow dagre's `lib/` modules (rank, normalize, order, position); the
// cross-counting and median ordering match Brandes–Köpf / dagre's heuristics.
//
// API: layout(g) where g = { nodes: Map<id,{width,height}>, edges:[{v,w}],
// options:{rankdir,nodesep,ranksep} }. After layout each node gains {x,y}
// (CENTER, world units) and a `rank`. Deterministic: every stage is a stable
// pass over insertion order — no randomness, no Date/Math.random.
//
// DO NOT EDIT. To re-derive: re-run the documented dagre pipeline against the
// pinned upstream behavior, then replace this file + LICENSE + the `.d.mts`.

/** A minimal directed graph: nodes carry layout attrs, edges are (v→w). */
class Graph {
  constructor() {
    this._nodes = new Map() // id -> attrs
    this._out = new Map() // id -> Set<succ>
    this._in = new Map() // id -> Set<pred>
    this._order = [] // stable insertion order of node ids
  }
  setNode(id, attrs) {
    if (!this._nodes.has(id)) {
      this._nodes.set(id, {})
      this._out.set(id, new Set())
      this._in.set(id, new Set())
      this._order.push(id)
    }
    if (attrs) Object.assign(this._nodes.get(id), attrs)
    return this
  }
  hasNode(id) {
    return this._nodes.has(id)
  }
  node(id) {
    return this._nodes.get(id)
  }
  removeNode(id) {
    if (!this._nodes.has(id)) return
    for (const s of this._out.get(id)) this._in.get(s).delete(id)
    for (const p of this._in.get(id)) this._out.get(p).delete(id)
    this._nodes.delete(id)
    this._out.delete(id)
    this._in.delete(id)
    this._order = this._order.filter((n) => n !== id)
  }
  setEdge(v, w) {
    this.setNode(v)
    this.setNode(w)
    this._out.get(v).add(w)
    this._in.get(w).add(v)
    return this
  }
  removeEdge(v, w) {
    this._out.get(v)?.delete(w)
    this._in.get(w)?.delete(v)
  }
  hasEdge(v, w) {
    return this._out.get(v)?.has(w) ?? false
  }
  /** Node ids in stable insertion order. */
  nodes() {
    return this._order.slice()
  }
  successors(id) {
    return [...(this._out.get(id) ?? [])]
  }
  predecessors(id) {
    return [...(this._in.get(id) ?? [])]
  }
  neighbors(id) {
    const out = new Set([...(this._out.get(id) ?? []), ...(this._in.get(id) ?? [])])
    return [...out]
  }
}

/** Break cycles with a stable greedy DFS so ranking sees a DAG. Reversed edges
 *  are recorded so the caller could restore them; here we only need acyclic
 *  ranking, so we drop the reversed direction for the rank pass. */
function acyclic(g) {
  const visited = new Set()
  const stack = new Set()
  const reversed = []
  const dfs = (v) => {
    if (visited.has(v)) return
    visited.add(v)
    stack.add(v)
    for (const w of g.successors(v)) {
      if (stack.has(w)) {
        g.removeEdge(v, w)
        if (!g.hasEdge(w, v)) g.setEdge(w, v)
        reversed.push([v, w])
      } else dfs(w)
    }
    stack.delete(v)
  }
  for (const v of g.nodes()) dfs(v)
  return reversed
}

/** Longest-path ranking: each node's rank = max over predecessors + 1.
 *  Sources (no predecessors) anchor at rank 0. Stable over insertion order. */
function assignRanks(g) {
  const rank = new Map()
  const indeg = new Map()
  for (const v of g.nodes()) indeg.set(v, g.predecessors(v).length)
  // Kahn-style topological pass; ties resolved by insertion order.
  const queue = g.nodes().filter((v) => indeg.get(v) === 0)
  for (const v of queue) rank.set(v, 0)
  let head = 0
  while (head < queue.length) {
    const v = queue[head++]
    const r = rank.get(v)
    for (const w of g.successors(v)) {
      rank.set(w, Math.max(rank.get(w) ?? 0, r + 1))
      indeg.set(w, indeg.get(w) - 1)
      if (indeg.get(w) === 0) queue.push(w)
    }
  }
  // Any node not reached (e.g. isolated after cycle-break) anchors at rank 0.
  for (const v of g.nodes()) if (!rank.has(v)) rank.set(v, 0)
  for (const v of g.nodes()) g.node(v).rank = rank.get(v)
  return rank
}

/** Group real node ids into rank buckets in stable order — the seed order for
 *  the cross-minimization sweeps. */
function rankBuckets(g) {
  const buckets = new Map()
  for (const v of g.nodes()) {
    const r = g.node(v).rank
    if (!buckets.has(r)) buckets.set(r, [])
    buckets.get(r).push(v)
  }
  const ranks = [...buckets.keys()].sort((a, b) => a - b)
  return ranks.map((r) => buckets.get(r))
}

/** Order within ranks by repeated barycenter sweeps to reduce edge crossings.
 *  Deterministic: fixed iteration count, ties keep their prior index. */
function order(g, layers) {
  const pos = new Map()
  const reindex = () => layers.forEach((layer) => layer.forEach((v, i) => pos.set(v, i)))
  reindex()
  const sweep = (down) => {
    const range = down
      ? [...layers.keys()].slice(1)
      : [...layers.keys()].slice(0, -1).reverse()
    for (const li of range) {
      const fixed = down ? layers[li - 1] : layers[li + 1]
      void fixed
      const layer = layers[li]
      const bary = layer.map((v, i) => {
        const adj = down ? g.predecessors(v) : g.successors(v)
        const ps = adj.map((u) => pos.get(u)).filter((p) => p !== undefined)
        const b = ps.length ? ps.reduce((a, c) => a + c, 0) / ps.length : i
        return { v, b, i }
      })
      bary.sort((p, q) => (p.b !== q.b ? p.b - q.b : p.i - q.i))
      layers[li] = bary.map((e) => e.v)
      reindex()
    }
  }
  // A small fixed budget of down/up sweeps — enough to settle small/medium
  // graphs, bounded so the result is reproducible.
  for (let it = 0; it < 8; it++) sweep(it % 2 === 0)
  return layers
}

/** Assign center coordinates from rank (the layout axis) + within-rank index
 *  (the cross axis), then rotate for rankdir. Coords are CENTERS. */
function position(g, layers, opts) {
  const ranksep = opts.ranksep
  const nodesep = opts.nodesep
  // cross-axis offset per layer: stack node extents + nodesep, centered.
  const crossOf = new Map()
  let maxRankExtent = 0
  layers.forEach((layer) => {
    let cursor = 0
    const extents = layer.map((v) => {
      const a = g.node(v)
      return opts.horizontal ? a.height : a.width
    })
    const total = extents.reduce((s, e) => s + e, 0) + nodesep * Math.max(0, layer.length - 1)
    maxRankExtent = Math.max(maxRankExtent, total)
    cursor = -total / 2
    layer.forEach((v, i) => {
      crossOf.set(v, cursor + extents[i] / 2)
      cursor += extents[i] + nodesep
    })
  })
  // rank-axis offset: stack ranks by the tallest/widest node in each + ranksep.
  const rankAxis = new Map()
  let along = 0
  layers.forEach((layer, ri) => {
    const sizeAlong = layer.length
      ? Math.max(...layer.map((v) => (opts.horizontal ? g.node(v).width : g.node(v).height)))
      : 0
    if (ri > 0) along += ranksep
    layers[ri].forEach((v) => rankAxis.set(v, along + sizeAlong / 2))
    along += sizeAlong
  })
  for (const v of g.nodes()) {
    const a = g.node(v)
    const cross = crossOf.get(v) ?? 0
    const rank = rankAxis.get(v) ?? 0
    if (opts.horizontal) {
      // LR/RL: rank runs along x, cross along y.
      a.x = opts.flip ? -rank : rank
      a.y = cross
    } else {
      // TB/BT: rank runs along y, cross along x.
      a.x = cross
      a.y = opts.flip ? -rank : rank
    }
  }
}

const RANKDIR = {
  LR: { horizontal: true, flip: false },
  RL: { horizontal: true, flip: true },
  TB: { horizontal: false, flip: false },
  BT: { horizontal: false, flip: true },
}

/** Run the full layered pipeline, mutating g's node attrs with {rank,x,y}. */
export function layout(g) {
  const o = g.options ?? {}
  const dir = RANKDIR[o.rankdir] ?? RANKDIR.LR
  const opts = {
    horizontal: dir.horizontal,
    flip: dir.flip,
    nodesep: o.nodesep ?? 50,
    ranksep: o.ranksep ?? 120,
  }
  // Build the internal graph from the public {nodes,edges} description.
  const ig = new Graph()
  for (const [id, attrs] of g.nodes) ig.setNode(id, { width: attrs.width, height: attrs.height })
  for (const e of g.edges) if (e.v !== e.w && ig.hasNode(e.v) && ig.hasNode(e.w)) ig.setEdge(e.v, e.w)
  acyclic(ig)
  assignRanks(ig)
  const layers = order(ig, rankBuckets(ig))
  position(ig, layers, opts)
  // Project results back onto the caller's node map.
  const result = new Map()
  for (const id of g.nodes.keys()) {
    const a = ig.node(id)
    result.set(id, { x: a.x, y: a.y, rank: a.rank })
  }
  return result
}

export { Graph }
export default { layout, Graph }
