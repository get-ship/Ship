/**
 * <ship-graph-canvas>
 *
 * Generic positioned-graph primitive. Renders an SVG edge layer with bezier
 * paths plus absolutely-positioned node cards authored via per-kind
 * <template slot="node" data-kind="..."> fragments. Any view that needs a
 * typed, clickable node-and-edge graph (agents wire, provider topology,
 * permission fanout) composes it here instead of re-rolling SVG per view.
 *
 * Data model
 *   nodes: Array<{ id, kind, title?, sub?, x?, y?, w?, data? }>
 *     - x/y are numbers in the same coordinate space as width/height (the
 *       SVG viewBox). When omitted on any node, the component falls back
 *       to a radial layout (agent-ish kinds on the left, other kinds
 *       grouped by kind into vertical bands on the right).
 *     - w is an optional pixel-space hint used by CSS for node width.
 *   edges: Array<{ from, to, kind?, dashed? }>
 *     - kind picks the stroke color via KIND_TONE. `dashed` toggles
 *       a stroke-dasharray.
 *
 * Kind → edge/rail color (via CSS in ship-graph-canvas.styles.ts):
 *   agent=primary, skill=success, mcp=info, rule=warning,
 *   perm=destructive, provider=accent
 *
 * Templates
 *   <template slot="node" data-kind="agent">
 *     <div class="node-h"><span class="tag">AGENT</span></div>
 *     <div class="node-title">${title}</div>
 *     <div class="node-sub">${sub}</div>
 *   </template>
 *
 *   Placeholders ${title}, ${sub}, ${id} are literal-substituted (HTML-
 *   escaped) before innerHTML assignment. Templates are author-trusted —
 *   node data is not. If no template matches a node's kind, a minimal
 *   default chrome is rendered.
 *
 * Events
 *   ship-node-select  { id, kind }  — click / Enter / Space on a node
 *   ship-node-hover   { id, kind }  — mouseenter on a node
 *
 * Attributes / properties
 *   nodes         JSON string or array
 *   edges         JSON string or array
 *   width         SVG viewBox width  (default 1000)
 *   height        SVG viewBox height (default 600)
 *   layout        'positioned' | 'radial' (default 'positioned')
 *   selected-id   reflected; matching node gets [data-selected]
 *
 * Parts: canvas, edges, node
 */

import { LitElement, html, svg } from 'lit'
import { shipGraphCanvasStyles } from './ship-graph-canvas.styles'

export interface GraphNode {
  id: string
  kind: string
  title?: string
  sub?: string
  x?: number
  y?: number
  w?: number
  data?: Record<string, unknown>
}

export interface GraphEdge {
  from: string
  to: string
  kind?: string
  dashed?: boolean
}

function escapeHtml(s: string): string {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function coerce<T>(v: unknown, fallback: T[]): T[] {
  if (Array.isArray(v)) return v as T[]
  if (typeof v === 'string' && v.trim()) {
    try {
      const parsed = JSON.parse(v)
      return Array.isArray(parsed) ? (parsed as T[]) : fallback
    } catch {
      return fallback
    }
  }
  return fallback
}

export class ShipGraphCanvas extends LitElement {
  static styles = shipGraphCanvasStyles

  static properties = {
    nodes: { attribute: 'nodes' },
    edges: { attribute: 'edges' },
    width: { type: Number, reflect: true },
    height: { type: Number, reflect: true },
    layout: { type: String, reflect: true },
    selectedId: { type: String, reflect: true, attribute: 'selected-id' },
  }

  nodes: GraphNode[] | string = []
  edges: GraphEdge[] | string = []
  width = 1000
  height = 600
  layout: 'positioned' | 'radial' = 'positioned'
  selectedId = ''

  private get nodeList(): GraphNode[] {
    return coerce<GraphNode>(this.nodes, [])
  }
  private get edgeList(): GraphEdge[] {
    return coerce<GraphEdge>(this.edges, [])
  }

  private templateFor(kind: string): HTMLTemplateElement | null {
    const tpls = this.querySelectorAll('template[slot="node"]')
    for (const t of Array.from(tpls)) {
      if ((t as HTMLElement).dataset.kind === kind) return t as HTMLTemplateElement
    }
    return null
  }

  /** Position map in viewBox coordinates. Radial layout activates when
   *  layout='radial' or any node is missing x/y. */
  private positions(): Map<string, { x: number; y: number }> {
    const nodes = this.nodeList
    const map = new Map<string, { x: number; y: number }>()
    const missing = nodes.some((n) => typeof n.x !== 'number' || typeof n.y !== 'number')
    if (this.layout === 'positioned' && !missing) {
      for (const n of nodes) map.set(n.id, { x: n.x as number, y: n.y as number })
      return map
    }
    const cx = this.width * 0.22
    const cy = this.height * 0.5
    const centers = nodes.filter((n) => n.kind === 'agent')
    const satellites = nodes.filter((n) => n.kind !== 'agent')
    centers.forEach((n, i) => {
      const span = Math.max(1, centers.length)
      const y = cy + (i - (span - 1) / 2) * 80
      map.set(n.id, { x: cx, y })
    })
    const byKind = new Map<string, GraphNode[]>()
    for (const n of satellites) {
      const list = byKind.get(n.kind) || []
      list.push(n)
      byKind.set(n.kind, list)
    }
    const kinds = Array.from(byKind.keys())
    kinds.forEach((kind, bandIdx) => {
      const list = byKind.get(kind)!
      const bandX = this.width * (0.5 + ((bandIdx + 1) / (kinds.length + 1)) * 0.45)
      list.forEach((n, i) => {
        const y = (this.height * (i + 1)) / (list.length + 1)
        map.set(n.id, { x: bandX, y })
      })
    })
    return map
  }

  private renderNodeBody(n: GraphNode) {
    const tpl = this.templateFor(n.kind)
    if (!tpl) {
      return html`
        <div class="node-h"><span class="tag">${n.kind}</span></div>
        <div class="node-title">${n.title ?? n.id}</div>
        ${n.sub ? html`<div class="node-sub">${n.sub}</div>` : null}
      `
    }
    const filled = tpl.innerHTML
      .replace(/\$\{title\}/g, escapeHtml(n.title ?? ''))
      .replace(/\$\{sub\}/g, escapeHtml(n.sub ?? ''))
      .replace(/\$\{id\}/g, escapeHtml(n.id))
    const host = document.createElement('div')
    host.className = 'tpl-host'
    host.innerHTML = filled
    return host
  }

  private fire(name: 'ship-node-select' | 'ship-node-hover', n: GraphNode) {
    this.dispatchEvent(
      new CustomEvent(name, {
        detail: { id: n.id, kind: n.kind },
        bubbles: true,
        composed: true,
      }),
    )
  }

  private activate(n: GraphNode) {
    this.selectedId = n.id
    this.fire('ship-node-select', n)
  }

  private onKey = (e: KeyboardEvent, n: GraphNode) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      this.activate(n)
    }
  }

  private bezier(a: { x: number; y: number }, b: { x: number; y: number }): string {
    const dx = Math.max(40, Math.abs(b.x - a.x) * 0.4)
    return `M ${a.x} ${a.y} C ${a.x + dx} ${a.y}, ${b.x - dx} ${b.y}, ${b.x} ${b.y}`
  }

  render() {
    const nodes = this.nodeList
    const edges = this.edgeList
    const pos = this.positions()
    const w = this.width || 1000
    const h = this.height || 600

    const edgeEls = edges.map((e) => {
      const a = pos.get(e.from)
      const b = pos.get(e.to)
      if (!a || !b) return null
      return svg`<path
        d=${this.bezier(a, b)}
        data-kind=${e.kind || 'default'}
        ?data-dashed=${!!e.dashed}
      ></path>`
    })

    return html`
      <div class="canvas" part="canvas">
        <svg
          class="edges"
          part="edges"
          viewBox="0 0 ${w} ${h}"
          preserveAspectRatio="none"
          aria-hidden="true"
        >${edgeEls}</svg>
        ${nodes.map((n) => {
          const p = pos.get(n.id)
          if (!p) return null
          const left = (p.x / w) * 100
          const top = (p.y / h) * 100
          const selected = this.selectedId && this.selectedId === n.id
          const widthStyle = n.w ? `--node-w: ${n.w}px;` : ''
          const label = `${n.kind}: ${n.title ?? n.id}`
          return html`
            <div
              class="node"
              part="node"
              data-kind=${n.kind}
              data-id=${n.id}
              ?data-selected=${selected}
              tabindex="0"
              role="button"
              aria-label=${label}
              style="left: ${left}%; top: ${top}%; ${widthStyle}"
              @click=${() => this.activate(n)}
              @keydown=${(e: KeyboardEvent) => this.onKey(e, n)}
              @mouseenter=${() => this.fire('ship-node-hover', n)}
            >
              ${this.renderNodeBody(n)}
            </div>
          `
        })}
        <slot name="node" hidden></slot>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-graph-canvas')) {
  customElements.define('ship-graph-canvas', ShipGraphCanvas)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-graph-canvas': ShipGraphCanvas
  }
}
