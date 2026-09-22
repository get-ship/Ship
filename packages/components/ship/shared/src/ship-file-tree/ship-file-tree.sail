<!--
  <ship-file-tree> — @ship/shared · the context navigator. A titled panel that
  composes the generic <sail-tree> primitive for ONE context (views,
  components…). RUNTIME-FREE, purely presentational: it owns the panel chrome
  (header label + scope line + a host-filled actions slot) and delegates ALL
  hierarchy behaviour — expand/collapse, keyboard nav, virtualization,
  selection — to <sail-tree>. Selection/toggle events bubble up from the tree
  unchanged (sui-select, sui-toggle, sui-node-click); the host listens on this
  element.

  Data in (the sui-tree contract, forwarded):
    • `nodes='[…JSON…]'` attribute — the declarative/SSR path. Each node is
      `{ id, label, icon?, children? }`; `icon` is a <icon> name, so a
      file-type icon rides per node (the v2 sui-tree feature this panel was
      built to use). The host may instead assign a live `Table` to the inner
      tree's `tree` IDENTITY property via `part="tree"` — that stays host-side
      (it owns the runtime), keeping this wrapper runtime-free.

  Header actions are NOT baked in. The host projects its own affordances into
  the `actions` slot (e.g. a "New" or "Collapse all" button) and wires their
  clicks — this component emits nothing and imports nothing (no controller,
  no @ship/sail-runtime / SDK / contract; enforced by `bun run guard`).

  Prop surface
  ─────────────────────────────────────────────────────────────────────────────
    label   (string, '')   panel header label — the context name (NOT a
                           generic "Explorer")
    scope   (string, '')   secondary scope line under the header
    nodes   (string, '')   JSON node list forwarded to <sail-tree nodes>
    selectable (none|single|multi, 'none')  forwarded to <sail-tree>
    show-guides (bool, true)                forwarded to <sail-tree>
    size    (sm|md|lg, 'md')                forwarded to <sail-tree>
    loading (bool, false)                   forwarded to <sail-tree>
    error   (string, '')                    forwarded to <sail-tree>
  ─────────────────────────────────────────────────────────────────────────────
  Slots:
    actions   host-supplied header affordances (New, Collapse all, …)
    empty     forwarded into <sail-tree slot="empty">
  Parts: head, scope, body, tree
  ─────────────────────────────────────────────────────────────────────────────
-->
<scope>
  props: {
    label = ''
    scope = ''
    nodes = ''
    selectable: 'none' | 'single' | 'multi' = 'none'
    showGuides = true
    size: 'sm' | 'md' | 'lg' = 'md'
    loading = false
    error = ''
  }
</scope>

<div>
  <span>{label()}</span>
  <span><slot name="actions"></slot></span>
</div>
<div>{scope()}</div>
<div>
  <sail-tree
    nodes={nodes()}
    label={label()}
    selectable={selectable()}
    show-guides={showGuides()}
    size={size()}
    loading={loading()}
    error={error()}
  >
    <slot name="empty" slot="empty"></slot>
  </sail-tree>
</div>
