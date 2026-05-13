import { controller, signal, live, derived, ref } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: ({ ship }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const systems = live(() => (ship as any).collections.items({ collection: 'systems' }))
    const depsJson = derived(() => {
      const env = systems()
      if (env.kind !== 'ok') return {} as Record<string, string>
      const out: Record<string, string> = {}
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      for (const item of env.value.items as Array<{ id: string; data?: { depends_on?: unknown } }>) {
        out[item.id] = JSON.stringify(item.data?.depends_on ?? [])
      }
      return out
    })

    // Live kernel-event feed driving the summary band — `events.list` returns
    // most-recent-first groups keyed by entity. We flatten and sort
    // chronologically so the open/close counting below applies the latest
    // verdict per session / agent.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const events = live(() => (ship as any).events.list({ limit: 50 }))
    type EventRow = {
      event_type: string; actor: string; session_id: string; timestamp: string
    }
    const sortedRows = derived<EventRow[]>(() => {
      const env = events()
      if (env.kind !== 'ok') return []
      const groups = ((env.value as { groups?: Array<{ rows?: EventRow[] }> }).groups ?? [])
      const rows: EventRow[] = groups.flatMap((g) => g.rows ?? [])
      rows.sort((a, b) => a.timestamp.localeCompare(b.timestamp))
      return rows
    })
    const activeSessions = derived(() => {
      const open = new Set<string>()
      for (const r of sortedRows()) {
        if (!r.session_id) continue
        if (r.event_type === 'session.started') open.add(r.session_id)
        else if (r.event_type === 'session.ended') open.delete(r.session_id)
      }
      return open.size
    })
    const runningAgents = derived(() => {
      const open = new Set<string>()
      for (const r of sortedRows()) {
        if (!r.actor) continue
        if (r.event_type === 'agent.started') open.add(r.actor)
        else if (
          r.event_type === 'agent.completed' ||
          r.event_type === 'agent.error' ||
          r.event_type === 'agent.errored' ||
          r.event_type === 'agent.failed'
        ) open.delete(r.actor)
      }
      return open.size
    })
    const lastEventLabel = derived(() => {
      const rows = sortedRows()
      if (rows.length === 0) return 'no events yet'
      const last = rows[rows.length - 1]
      const ts = new Date(last.timestamp)
      if (Number.isNaN(ts.getTime())) return last.event_type
      return `${last.event_type} · ${ts.toLocaleTimeString()}`
    })
    return {
      systems,
      depsJson,
      events,
      activeSessions,
      runningAgents,
      lastEventLabel,
      search: signal(''),
      filterStatus: signal(''),
      filterLayer: signal(''),
      drawerOpen: signal(false),
      drawerId: signal<string | null>(null),
      drawerTitle: signal('New system'),
      fldName: signal(''),
      fldLayer: signal(''),
      fldStatus: signal('spec'),
      fldSummary: signal(''),
      fldDocSkill: signal(''),
      fldGaps: signal<string[]>([]),
      fldCodePaths: signal<string[]>([]),
      fldDependsOn: signal<string[]>([]),
      // DOM refs wired via bind:this in the template.
      wrapRef: ref<HTMLElement>(null),
      canvasRef: ref<HTMLElement>(null),
      edgesRef: ref<SVGElement>(null),
    }
  },

  on: {
    openNewDrawer: (s) => {
      s.drawerId = null
      s.drawerTitle = 'New system'
      s.fldName = ''; s.fldLayer = ''; s.fldStatus = 'spec'
      s.fldSummary = ''; s.fldDocSkill = ''
      s.fldGaps = []; s.fldCodePaths = []; s.fldDependsOn = []
      s.drawerOpen = true
    },
    closeDrawer: (s) => { s.drawerOpen = false },
    fitView: () => (window as Window & { __ovFit?: () => void }).__ovFit?.(),
    askAgent: (s) => {
      fetch('/api/v1/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          kind: 'system_dispatch',
          system: {
            name: s.fldName,
            summary: s.fldSummary,
            status: s.fldStatus,
            layer: s.fldLayer,
          },
        }),
      }).catch(() => {})
    },
    addGap: (s) => { s.fldGaps = [...s.fldGaps, ''] },
    addDependsOn: (s) => { s.fldDependsOn = [...s.fldDependsOn, ''] },
    saveDrawer: (s) => {
      const data: Record<string, unknown> = {}
      if (s.fldName)    data.name = s.fldName
      if (s.fldLayer)   data.layer = s.fldLayer
      if (s.fldStatus)  data.status = s.fldStatus
      if (s.fldSummary) data.summary = s.fldSummary
      if (s.fldDocSkill) data.doc_skill = s.fldDocSkill
      data.gaps = s.fldGaps
      data.code_paths = s.fldCodePaths
      data.depends_on = s.fldDependsOn
      const id = s.drawerId as string | null
      const url = id
        ? `/api/v1/collections/systems/items/${encodeURIComponent(id)}`
        : '/api/v1/collections/systems/items'
      fetch(url, {
        method: id ? 'PUT' : 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ data }),
      }).then(r => { if (r.ok) s.drawerOpen = false }).catch(() => {})
    },
  },

  mount: ({ state }, _el) => {
    const canvas = state.canvasRef()
    const edges = state.edgesRef()
    const wrap = state.wrapRef()
    if (!canvas || !edges || !wrap) return

    let scale = 1, tx = 0, ty = 0
    function applyTransform() {
      canvas.style.transform = `translate(${tx}px,${ty}px) scale(${scale})`
    }

    function nodeCenter(node: HTMLElement) {
      return {
        x: (parseFloat(node.style.left) || 0) + node.offsetWidth / 2,
        y: (parseFloat(node.style.top)  || 0) + node.offsetHeight / 2,
      }
    }
    function drawEdges() {
      const ns = 'http://www.w3.org/2000/svg'
      edges.replaceChildren()
      const defs = document.createElementNS(ns, 'defs')
      const marker = document.createElementNS(ns, 'marker')
      marker.setAttribute('id', 'ov-arrow')
      marker.setAttribute('viewBox', '0 0 10 10')
      marker.setAttribute('refX', '9'); marker.setAttribute('refY', '5')
      marker.setAttribute('markerWidth', '6'); marker.setAttribute('markerHeight', '6')
      marker.setAttribute('orient', 'auto')
      const arrow = document.createElementNS(ns, 'path')
      arrow.setAttribute('d', 'M0,0 L10,5 L0,10 z')
      arrow.setAttribute('fill', 'var(--muted-foreground)')
      marker.appendChild(arrow); defs.appendChild(marker); edges.appendChild(defs)
      canvas.querySelectorAll<HTMLElement>('.ov-node').forEach(src => {
        let deps: string[] = []
        try { deps = JSON.parse(src.dataset.depends ?? '[]') } catch (_) {}
        const a = nodeCenter(src)
        deps.forEach(name => {
          const tgt = canvas.querySelector<HTMLElement>(`.ov-node[data-name="${CSS.escape(name)}"]`)
          if (!tgt) return
          const b = nodeCenter(tgt)
          const p = document.createElementNS(ns, 'path')
          p.setAttribute('d', `M ${a.x} ${a.y} L ${b.x} ${b.y}`)
          p.setAttribute('marker-end', 'url(#ov-arrow)')
          edges.appendChild(p)
        })
      })
    }

    // Pan
    let panning = false, panStart = { x: 0, y: 0 }
    wrap.addEventListener('pointerdown', e => {
      if ((e.target as Element).closest('.ov-node, .ov-drawer')) return
      panning = true; panStart = { x: e.clientX - tx, y: e.clientY - ty }
      canvas.classList.add('panning'); wrap.setPointerCapture(e.pointerId)
    })
    wrap.addEventListener('pointermove', e => {
      if (!panning) return
      tx = e.clientX - panStart.x; ty = e.clientY - panStart.y; applyTransform()
    })
    wrap.addEventListener('pointerup', e => {
      panning = false; canvas.classList.remove('panning')
      try { wrap.releasePointerCapture(e.pointerId) } catch (_) {}
    })

    // Zoom
    wrap.addEventListener('wheel', e => {
      e.preventDefault()
      const rect = wrap.getBoundingClientRect()
      const cx = e.clientX - rect.left, cy = e.clientY - rect.top
      const dz = Math.exp(-e.deltaY * 0.0015)
      const ns2 = Math.max(0.2, Math.min(3, scale * dz))
      tx = cx - (cx - tx) * (ns2 / scale)
      ty = cy - (cy - ty) * (ns2 / scale)
      scale = ns2; applyTransform()
    }, { passive: false })

    // Node drag + click → drawer
    function openDrawer(id: string) {
      fetch(`/api/v1/collections/systems/items/${encodeURIComponent(id)}`)
        .then(r => r.json())
        .then(env => {
          const item = env.data || env
          const data = item.data || item
          state.drawerId.set(id)
          state.drawerTitle.set(data.name || 'System')
          state.fldName.set(data.name || '')
          state.fldLayer.set(data.layer || '')
          state.fldStatus.set(data.status || 'spec')
          state.fldSummary.set(data.summary || '')
          state.fldDocSkill.set(data.doc_skill || '')
          state.fldGaps.set(data.gaps || [])
          state.fldCodePaths.set(data.code_paths || [])
          state.fldDependsOn.set(data.depends_on || [])
          state.drawerOpen.set(true)
        })
        .catch(() => {})
    }

    canvas.querySelectorAll<HTMLElement>('.ov-node').forEach(node => {
      let startX = 0, startY = 0, origX = 0, origY = 0, moved = false
      node.addEventListener('pointerdown', e => {
        e.stopPropagation()
        moved = false; startX = e.clientX; startY = e.clientY
        origX = parseFloat(node.style.left) || 0
        origY = parseFloat(node.style.top) || 0
        node.classList.add('dragging'); node.setPointerCapture(e.pointerId)
      })
      node.addEventListener('pointermove', e => {
        const dx = (e.clientX - startX) / scale
        const dy = (e.clientY - startY) / scale
        if (Math.hypot(dx, dy) > 3) moved = true
        if (!moved) return
        node.style.left = (origX + dx) + 'px'
        node.style.top  = (origY + dy) + 'px'
        drawEdges()
      })
      node.addEventListener('pointerup', e => {
        node.classList.remove('dragging')
        try { node.releasePointerCapture(e.pointerId) } catch (_) {}
        if (moved) {
          fetch(`/api/v1/collections/systems/items/${encodeURIComponent(node.dataset.id ?? '')}`, {
            method: 'PUT', headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ data: { position_x: parseFloat(node.style.left), position_y: parseFloat(node.style.top) } }),
          }).catch(() => {})
        } else {
          openDrawer(node.dataset.id ?? '')
        }
      })
    })

    // Fit to view
    ;(window as Window & { __ovFit?: () => void }).__ovFit = () => {
      const nodes = [...canvas.querySelectorAll<HTMLElement>('.ov-node')]
      if (!nodes.length) return
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
      nodes.forEach(n => {
        const x = parseFloat(n.style.left), y = parseFloat(n.style.top)
        if (x < minX) minX = x; if (y < minY) minY = y
        if (x + n.offsetWidth > maxX) maxX = x + n.offsetWidth
        if (y + n.offsetHeight > maxY) maxY = y + n.offsetHeight
      })
      const pad = 40, rect = wrap.getBoundingClientRect()
      scale = Math.max(0.3, Math.min(1.5, Math.min((rect.width - pad * 2) / (maxX - minX), (rect.height - pad * 2) / (maxY - minY))))
      tx = pad - minX * scale; ty = pad - minY * scale; applyTransform()
    }

    const onKeydown = (e: KeyboardEvent) => { if (e.key === 'Escape') state.drawerOpen.set(false) }
    document.addEventListener('keydown', onKeydown)
    ;(state as { _ovKeydown?: (e: KeyboardEvent) => void })._ovKeydown = onKeydown
    drawEdges()
    requestAnimationFrame(() => (window as Window & { __ovFit?: () => void }).__ovFit?.())
  },

  unmount: (state) => {
    delete (window as Window & { __ovFit?: () => void }).__ovFit
    const fn = (state as { _ovKeydown?: (e: KeyboardEvent) => void })._ovKeydown
    if (fn) document.removeEventListener('keydown', fn)
  },
})
