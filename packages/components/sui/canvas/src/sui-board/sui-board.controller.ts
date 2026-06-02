import { controller, signal, derived, effect, debounce } from '@ship/sui-runtime'
import type { Scope } from './sui-board.sui'
import { createCanvas } from '../engine/create-canvas'
import { createBoardModel, type Node, type Edge, type PortRef } from './engine/board-model'

// SSR fallback = the verified sui-combobox shape: plain JSON.parse, try/caught.
// The live path is prop:* identity onto the board model.
function parseBoard(raw: string): { nodes?: Node[]; edges?: Edge[] } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { nodes?: Node[]; edges?: Edge[] }
  } catch {
    return null
  }
}

// <sui-board> controller — SVG-native nodes/edges/ports on the canvas
// substrate. Geometry lives in the board model (world coords); the view maps
// to screen through the substrate transform. Node visuals are a per-node slot.
export default controller<Scope>({
  scope: ({ value }) => {
    const seed = parseBoard(value())
    const board = createBoardModel({
      nodes: seed?.nodes ?? [],
      edges: seed?.edges ?? [],
    })
    const canvas = createCanvas({
      getViewportElement: () => null, // <sui-canvas> owns the viewport thunk
    })
    // Drag bookkeeping — which node is under an active face-drag.
    const draggingNode = signal<string | null>(null)
    const lastPointer = signal<{ x: number; y: number } | null>(null)
    // Outbound debounced change source: a monotonic version bumped on mutation.
    const version = signal(0)
    const changeSrc = debounce(version, 200)
    return { board, canvas, draggingNode, lastPointer, version, changeSrc }
  },

  derived: {
    nodes: ({ board }) => board.nodes(),
    routes: ({ board }) => board.routes(),
    selectedSet: ({ board }) => new Set(board.selectedNodes()),
    incidentSet: ({ board }) => new Set(board.incidentEdges()),
    pendingPath: ({ board, canvas }) => {
      const pe = board.pendingEdge()
      if (!pe) return ''
      const a = canvas.worldToScreen(pe.start)
      const b = canvas.worldToScreen(pe.end)
      return `M ${a.x} ${a.y} L ${b.x} ${b.y}`
    },
  },

  on: {
    nodePointerDown: (s, e: PointerEvent) => {
      const el = e.currentTarget as HTMLElement
      const id = el.dataset.nodeId
      if (!id) return
      // additive select with shift; otherwise replace selection.
      if (e.shiftKey) s.board.toggleSelection(id)
      else if (!s.board.selectedNodes().includes(id)) s.board.setSelection([id])
      s.draggingNode = id
      s.lastPointer = { x: e.clientX, y: e.clientY }
      el.setPointerCapture?.(e.pointerId)
      e.stopPropagation()
    },
    portPointerDown: (s, e: PointerEvent) => {
      const el = e.currentTarget as HTMLElement
      const portId = el.dataset.portId
      const nodeEl = el.closest('[data-node-id]') as HTMLElement | null
      const nodeId = nodeEl?.dataset.nodeId
      if (!portId || !nodeId) return
      s.board.beginEdge({ node: nodeId, port: portId })
      e.stopPropagation()
    },
  },

  mount(state, host) {
    const board = state.board
    const canvas = state.canvas

    // Node-drag pump: move the dragged (or whole-selection) node by the
    // world-space delta. Listens on the host so capture survives.
    const onMove = (e: PointerEvent): void => {
      const id = state.draggingNode
      const last = state.lastPointer
      if (id && last) {
        const scale = canvas.transform().scale
        const dx = (e.clientX - last.x) / scale
        const dy = (e.clientY - last.y) / scale
        if (board.selectedNodes().includes(id)) board.moveSelectionBy(dx, dy)
        else board.moveNodeBy(id, dx, dy)
        state.lastPointer = { x: e.clientX, y: e.clientY }
        state.version = state.version + 1
        return
      }
      if (board.pendingEdge()) {
        board.updatePendingEnd(canvas.screenToWorld({ x: e.clientX, y: e.clientY }))
      }
    }
    const onUp = (e: PointerEvent): void => {
      if (state.draggingNode) {
        state.draggingNode = null
        state.lastPointer = null
        host.emit!('sui-node-select', { ids: board.selectedNodes() })
      }
      const pe = board.pendingEdge()
      if (pe) {
        const tEl = (e.target as HTMLElement)?.closest?.('[data-port-id]') as HTMLElement | null
        const nodeEl = tEl?.closest('[data-node-id]') as HTMLElement | null
        const target: PortRef | null =
          tEl && nodeEl ? { node: nodeEl.dataset.nodeId!, port: tEl.dataset.portId! } : null
        const edge = target ? board.commitEdge(target) : (board.cancelEdge(), null)
        if (edge) {
          state.version = state.version + 1
          host.emit!('sui-edge-create', { source: edge.source, target: edge.target })
        }
      }
    }
    const onKey = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') {
        board.clearSelection()
        board.cancelEdge()
        host.emit!('sui-node-select', { ids: [] })
      }
    }
    host.addEventListener('pointermove', onMove as EventListener)
    host.addEventListener('pointerup', onUp as EventListener)
    host.addEventListener('keydown', onKey as EventListener)

    // Debounced board-change event (the KEEP'd contract: value + version).
    const stopChange = effect(() => {
      const v = state.changeSrc
      if (v === 0) return
      host.emit!('sui-board-change', { value: board.serialize(), version: v })
    })
    // Announce selection changes on the host (live-region consumers read it).
    const stopSel = effect(() => {
      host.emit!('sui-node-select', { ids: board.selectedNodes() })
    })

    return () => {
      host.removeEventListener('pointermove', onMove as EventListener)
      host.removeEventListener('pointerup', onUp as EventListener)
      host.removeEventListener('keydown', onKey as EventListener)
      stopChange()
      stopSel()
      canvas.dispose()
    }
  },
})
