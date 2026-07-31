<controller>
import { DECK_PREFIX } from '@ship/sdk-client'
import { controller, signal, effect, derived } from '@ship/sail-runtime'
// `defineRoutes` is intentionally NOT called here. The shell's
// /studio/components/:name route is registered by the codegen at boot
// (apps/shell/routes.gen.ts); the view's defineRoutes would land in a
// separate runtime instance and never seed the shell's shared `route`
// proxy's `params.name`. The view-host standalone pattern
// /_deck/view/studio/components/:name is documented here but is
// registered by a future host (TODO: not yet wired).

// Shared across mounts: one iframe + one preview node at module scope. Holding
// them caps DOM iframe count at 1 (the SSE connection survives navigations) and
// lets the sui-kind preview reuse the same host element across same-tag mounts.
let sharedFrame: HTMLIFrameElement | null = null
let sharedNode: HTMLElement | null = null
let sharedNodeTag: string | null = null

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const allCatalog = signal<CatalogEntry[]>([])
    const loading = signal(true)
    const propValues = signal<Record<string, unknown>>({})
    const _dispose = signal<(() => void) | null>(null)
    const copyFeedback = signal(false)
    // Authoritative subroute slug — seeded from `location.pathname` at
    // mount() rather than `route.params.name()`. The shell pre-registers
    // /studio/components/:name at boot (see apps/shell/routes.gen.ts),
    // so `route.params.name()` resolves on first paint. Reading the
    // pathname is defensive — if the codegen ever drifts, this still
    // works.
    const currentName = signal<string>('')

    const entry = derived(() => {
      const name = currentName()
      if (!name) return null
      return allCatalog().find(e => e.id === name) ?? null
    })
    const hasProps = derived(() => {
      const e = entry()
      return e !== null && (e.props ?? []).length > 0
    })
    // Knobs for the playground, classified into enum / bool / string.
    const knobs = derived(() => {
      const e = entry()
      if (!e) return [] as Knob[]
      return (e.props ?? []).map(classifyKnob)
    })
    const slotList = derived(() => {
      const e = entry()
      return (e?.slots ?? []) as { name?: string; description?: string }[]
    })
    const metaLabel = derived(() => {
      const e = entry()
      if (!e) return ''
      const p = (e.props ?? []).length
      const s = (e.slots ?? []).length
      const parts: string[] = [`${p} prop${p === 1 ? '' : 's'}`]
      if (s > 0) parts.push(`${s} slot${s === 1 ? '' : 's'}`)
      return parts.join(' · ')
    })
    // Live source snippet, recomputed as the knobs change.
    const snippet = derived(() => {
      const e = entry()
      if (!e) return ''
      return buildSnippet(e.tag, propValues(), e.props ?? [], e.examples?.[0]?.slots)
    })
    const matrixAxes = derived(() => {
      const e = entry()
      if (!e || e.kind !== 'sui') return [] as MatrixAxis[]
      return deriveAxes(e.props ?? [])
    })
    const matrixCells = derived(() => buildCells(matrixAxes()))
    const matrixAxesLabel = derived(() => {
      const axes = matrixAxes()
      if (axes.length === 0) return ''
      return axes.map(a => a.name).join(' × ')
    })
    return {
      allCatalog, loading, propValues, _dispose, copyFeedback, currentName,
      entry, hasProps, knobs, slotList, metaLabel, snippet, matrixAxes, matrixCells, matrixAxesLabel,
    }
  },

  on: {
    setProp: (state, propName: string, value: unknown) => {
      state.propValues = { ...state.propValues, [propName]: value }
    },
    copyCode: (state, entry: CatalogEntry | null) => {
      if (!entry) return
      const snippet = buildSnippet(
        entry.tag,
        state.propValues,
        entry.props ?? [],
        entry.examples?.[0]?.slots,
      )
      void navigator.clipboard.writeText(snippet)
      state.copyFeedback = true
      setTimeout(() => { state.copyFeedback = false }, 1500)
    },
  },

  // mount may be async; the runtime awaits it. Mount's return type is
  // sync-only; cast through unknown to satisfy the inferred section contract.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  mount: (async (state: any, el: HTMLElement) => {
    const all = await loadCatalog()
    state.allCatalog = all

    const segments = location.pathname.split('/').filter(Boolean)
    const name = segments[segments.length - 1] ?? ''
    state.currentName = name
    const entry = all.find(e => e.id === name)

    if (!entry) {
      state.loading = false
      return
    }

    // Seed prop defaults. For size props, prefer the largest enum variant
    // so small components (badge, kbd, status-dot) are legible.
    const initVals: Record<string, unknown> = {}
    for (const prop of entry.props ?? []) {
      if (prop.name === 'size' && prop.values && prop.values.length > 0) {
        initVals[prop.name] = prop.values[prop.values.length - 1]
      } else if (prop.default !== undefined) {
        initVals[prop.name] = prop.default
      }
    }
    const fromUrl = readSearchParams(location.search, entry.props ?? [])
    Object.assign(initVals, fromUrl)
    state.propValues = initVals
    state.loading = false

    // Wait one tick for the template to re-render with loading=false.
    await new Promise<void>(r => setTimeout(r, 0))

    const slot = el.querySelector('[data-render-slot]') as HTMLElement | null
    if (!slot) return

    if (entry.kind === 'view') {
      if (sharedNode?.parentElement) sharedNode.parentElement.removeChild(sharedNode)
      if (!sharedFrame) {
        sharedFrame = document.createElement('iframe')
        sharedFrame.className = 'detail-view-frame'
        sharedFrame.loading = 'eager'
      }
      if (sharedFrame.parentElement !== slot) {
        sharedFrame.parentElement?.removeChild(sharedFrame)
        slot.appendChild(sharedFrame)
      }
      sharedFrame.title = `${entry.id} preview`
      const nextHref = new URL(`${DECK_PREFIX}/view/${entry.id}`, location.href).href
      if (sharedFrame.src !== nextHref) sharedFrame.src = nextHref
      state._dispose = () => {}
      return
    }

    if (sharedFrame?.parentElement) sharedFrame.parentElement.removeChild(sharedFrame)

    if (sharedNode && sharedNodeTag !== entry.tag) {
      sharedNode.remove()
      sharedNode = null
      sharedNodeTag = null
    }
    if (!sharedNode) {
      sharedNode = document.createElement(entry.tag)
      sharedNode.className = 'detail-rendered-component'
      sharedNodeTag = entry.tag
    }
    const node = sharedNode
    node.style.transform = ''
    node.style.transformOrigin = ''
    const compose = entry.examples?.[0]?.slots
    if (compose) {
      const range = document.createRange()
      range.selectNodeContents(node)
      const frag = range.createContextualFragment(compose)
      node.replaceChildren(frag)
    } else {
      node.replaceChildren(document.createTextNode(slotLabel(entry.tag)))
    }
    if (node.parentElement !== slot) {
      node.parentElement?.removeChild(node)
      slot.appendChild(node)
    }

    // Components with intrinsic size < 64px are scaled up for legibility.
    const ZOOM_TAGS = new Set(['sui-status-dot', 'sui-badge', 'sui-chip', 'sui-spinner'])
    if (ZOOM_TAGS.has(entry.tag)) {
      node.style.transform = 'scale(3)'
      node.style.transformOrigin = 'center'
    }

    // Apply prop values to the rendered host. Boolean attrs follow HTML
    // semantics (presence ≡ true) — strip entirely when falsy. String
    // props: remove on null/empty so the component keeps its declared
    // default.
    const props = entry.props ?? []
    const propTypes = new Map(props.map(p => [p.name, p.type]))
    const applyProps = (vals: Record<string, unknown>) => {
      for (const p of props) {
        const v = vals[p.name]
        const t = propTypes.get(p.name)
        if (isBoolType(t)) {
          const truthy = v === true || v === 'true'
          if (truthy) node.setAttribute(p.name, '')
          else node.removeAttribute(p.name)
        } else {
          if (v == null || v === '') node.removeAttribute(p.name)
          else node.setAttribute(p.name, String(v))
        }
      }
    }
    applyProps(initVals)

    // Reactively sync propValues → element attributes + URL search params,
    // and re-highlight the matching matrix cell. The render-slot carries
    // `morph` so controller-driven re-renders patch in place.
    const stopEffect = effect(() => {
      const vals = state.propValues
      applyProps(vals)
      writeSearchParams(vals, props)
      highlightActiveCell(el, vals)
    })

    mountMatrixCells(el, entry.tag, () => entry.examples?.[0]?.slots)
    highlightActiveCell(el, state.propValues)

    state._dispose = () => { stopEffect() }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  }) as unknown as any,

  unmount: (state) => {
    state._dispose?.()
  },
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <div>
      <if cond={loading()}>
        <div><spinner size="md" centered></spinner></div>
      <else-if cond={entry()}>
        <article data-test="component-detail">

          <a href="/studio/components" data-test="detail-back">
            <icon name="chevron-left" size="xs"></icon>
            Components
          </a>

          <header>
            <span aria-hidden="true">&lt;/&gt;</span>
            <div>
              <h1>{entry().tag}</h1>
              <div>
                <code>{entry().category}</code>
                <span aria-hidden="true"></span>
                <span>{metaLabel()}</span>
                <if cond={entry().pkg}>
                  <span aria-hidden="true"></span>
                  <code>{entry().pkg}</code>
                </if>
              </div>
              <div aria-hidden="true"></div>
            </div>
            <div>
              <if cond={entry().sourcePath}>
                <button variant="outline" size="sm"
                  href={'/_deck/source/' + entry().tag}
                  data-test="source-link">
                  <icon slot="leading" name="file-code" size="xs"></icon>
                  Open source
                </button>
              </if>
            </div>
          </header>

          <if cond={entry().description}>
            <p>{entry().description}</p>
          </if>

          <!-- Matrix -->
          <if cond={matrixCells().length > 0}>
            <section data-test="state-matrix">
              <div>
                <h2>Matrix <span>· {matrixAxesLabel()}</span></h2>
                <span>{matrixCells().length} cells</span>
              </div>
              <div>
                <for as="cell" of={matrixCells()} key={cell.id}>
                  <div data-test="matrix-cell" data-label={cell.label}>
                    <div
                      data-matrix-tag={entry().tag}
                      data-matrix-cell={cell.id}
                      data-props={cell.propsJson}
                      morph></div>
                    <div>{cell.label}</div>
                  </div>
                </for>
              </div>
            </section>
          </if>

          <!-- Playground -->
          <section data-test="playground">
            <div>
              <h2>Playground</h2>
              <button type="button"
                data-test="copy-code" data-copied={copyFeedback()}
                on:click={() => copyCode(entry())}>
                {copyFeedback() ? 'Copied' : 'Copy code'}
              </button>
            </div>

            <if cond={knobs().length > 0}>
              <div data-test="knobs">
                <for as="knob" of={knobs()} key={knob.name}>
                  <div data-test="knob" data-prop={knob.name}>
                    <span>{knob.name}</span>
                    <if cond={knob.kind === 'enum'}>
                      <div role="group">
                        <for as="v" of={knob.values} key={v}>
                          <button type="button" data-test="seg"
                            data-active={(propValues()[knob.name] ?? '') + '' === v}
                            on:click={() => setProp(knob.name, v)}>{v}</button>
                        </for>
                      </div>
                    <else-if cond={knob.kind === 'bool'}>
                      <label>
                        <input type="checkbox" data-test="knob-bool"
                          checked={propValues()[knob.name] === true || propValues()[knob.name] === 'true'}
                          on:change={(e) => setProp(knob.name, e.target.checked ? 'true' : 'false')} />
                        <span>{(propValues()[knob.name] === true || propValues()[knob.name] === 'true') ? 'on' : 'off'}</span>
                      </label>
                    </else-if><else>
                      <input type="text" data-test="knob-text"
                        value={(propValues()[knob.name] ?? '') + ''}
                        placeholder={knob.name}
                        on:input={(e) => setProp(knob.name, e.target.value)} />
                    </else></if>
                  </div>
                </for>
              </div>
            </if>

            <div data-stage-kind={entry().kind}>
              <div data-render-slot data-test="render-slot" morph></div>
            </div>

            <if cond={entry().kind === 'sui'}>
              <pre data-test="code-output">{snippet()}</pre>
            <else>
              <div>Live view · <code>/_deck/view/{entry().id}</code></div>
            </else></if>
          </section>

          <!-- Props -->
          <if cond={hasProps()}>
            <section data-test="props-table">
              <div>
                <h2>Props <span>· {entry().props.length}</span></h2>
              </div>
              <table>
                <thead>
                  <tr><th>name</th><th>type</th><th>default</th></tr>
                </thead>
                <tbody>
                  <for as="prop" of={entry().props} key={prop.name}>
                    <tr data-test="prop-row">
                      <td><code>{prop.name}</code></td>
                      <td><code>{prop.type}</code></td>
                      <td><if cond={prop.default}><code>{prop.default}</code><else><span>—</span></else></if></td>
                    </tr>
                  </for>
                </tbody>
              </table>
            </section>
          </if>

          <!-- Slots -->
          <if cond={slotList().length > 0}>
            <section data-test="slots">
              <div>
                <h2>Slots <span>· {slotList().length}</span></h2>
              </div>
              <div>
                <for as="s" of={slotList()} key={s.name}>
                  <div>
                    <span>{s.name ? 'slot="' + s.name + '"' : '(default)'}</span>
                    <if cond={s.description}><span>— {s.description}</span></if>
                  </div>
                </for>
              </div>
            </section>
          </if>

          <!-- Source -->
          <if cond={entry().sourcePath}>
            <section data-test="source">
              <div>
                <h2>Source</h2>
              </div>
              <div>{entry().sourcePath}</div>
            </section>
          </if>

        </article>
      </else-if><else>
        <div>
          <empty-state icon="search-x" variant="default" size="md" data-test="not-found">
            <span slot="title">Component not found</span>
            <span>No component matched this path.</span>
            <button slot="action" variant="outline" size="sm" href="/studio/components">Back to components</button>
          </empty-state>
        </div>
      </else></if>
    </div>
  </main>
</div>
