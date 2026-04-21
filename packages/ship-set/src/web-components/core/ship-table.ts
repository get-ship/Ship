/**
 * <ship-table>
 *
 * Data-table primitive matching shadcn/ui Table's visual weight: thin
 * border, --radius-lg corners, uppercase tracked headers, quiet row hover,
 * optional zebra and sticky-header variants. The wrapper chrome lives in
 * Shadow DOM; cell-level styling is adopted once onto the host document
 * because Shadow DOM's `::slotted()` can't reach <th>/<td> inside slotted
 * <thead>/<tr> elements.
 *
 * Two consumption patterns are supported:
 *
 *   A) Slot pattern (preserves pre-polish API, used by tests + JS callers):
 *
 *      const t = document.createElement('ship-table')
 *      const headRow = document.createElement('tr')
 *      headRow.slot = 'head'
 *      // ... append th / td
 *      t.append(headRow, bodyRow)
 *
 *   B) Native table pattern (recommended for HTML authoring): set
 *      `variant="native"` and place a full <table> as a child. The
 *      HTML5 parser requires the <table> ancestor for <thead>/<tbody>/
 *      <tr> to survive foster parenting, so this is the only option
 *      for consumers writing HTML directly (gallery, daemon templates,
 *      Datastar-rendered views).
 *
 *      <ship-table variant="native" striped sticky-header>
 *        <table>
 *          <thead><tr><th data-sort-key="name">Name</th></tr></thead>
 *          <tbody><tr><td>Alpha</td></tr></tbody>
 *        </table>
 *      </ship-table>
 *
 * Attributes:
 *   - variant: slot | native (default: slot)
 *   - density: compact | default | comfortable (default: default)
 *     Row padding channel. `compact` matches log-table density; `comfortable`
 *     matches settings / admin tables.
 *   - dense: boolean (legacy)
 *     Alias for density="compact". Preserved for the pre-polish API.
 *   - hoverable: boolean (default: true)
 *     Subtle row hover tint. Set `hoverable="false"` to disable.
 *   - striped: boolean
 *     Zebra-stripe alternate body rows with a quiet muted tint.
 *   - sticky-header: boolean
 *     Pins the <thead> while the body scrolls. Pair with an inline
 *     `style="max-height: 400px"` on the host so the wrapper has a
 *     scroll container.
 *   - sticky-first-col: boolean
 *     Pins the first column during horizontal scroll.
 *   - gutter: boolean
 *     Prepends a narrow 40px leading column. Fill via `class="ship-table-gutter-cell"`
 *     on the first th/td of each row, or let the element inject the header cell.
 *   - label: string
 *     Forwarded to the internal table as `aria-label`.
 *
 * Events:
 *   - ship-sort — CustomEvent { detail: { key, direction } }
 *   - ship-row-click — CustomEvent { detail: { id, index } }
 *     Dispatched when a cell in a row with data-row-id is clicked (non-gutter).
 *
 * Per-cell conventions (consumer markup):
 *   - align="left|right|center" on <th>/<td>
 *   - class="mono" on a <td> for monospaced ID-style values
 *   - class="muted" on a <td> for muted-foreground text
 *   - class="null" on placeholder text (italic muted)
 *   - data-sort-key="<key>" on a <th> — renders a sort chevron and
 *     dispatches `ship-sort` on click; aria-sort cycles
 *     none -> ascending -> descending -> none.
 *   - data-selected (or aria-selected="true") on a <tr> — primary-tinted row.
 *
 * Slots:
 *   - head — legacy slot for thead <tr>s (tests rely on this)
 *   - (default) — tbody <tr>s in slot mode; the outer <table> in native mode
 *   - empty — placeholder rendered when there are no body rows
 *
 * Events:
 *   - ship-sort — CustomEvent { detail: { key, direction } }
 *     direction: 'ascending' | 'descending' | 'none' (cycles).
 */

import { LitElement, html, nothing } from 'lit'
import { shipTableStyles, shipTableLightDomCss } from './ship-table.styles'

type Density = 'compact' | 'default' | 'comfortable'
type Variant = 'slot' | 'native'
type SortDirection = 'ascending' | 'descending' | 'none'

/** Install the light-DOM stylesheet on a document once. Idempotent across
 * any number of <ship-table> instances. */
const lightDomInstalled = new WeakSet<Document>()
function installLightDomStyles(doc: Document) {
  if (lightDomInstalled.has(doc)) return
  lightDomInstalled.add(doc)
  try {
    if (
      typeof CSSStyleSheet !== 'undefined' &&
      'replaceSync' in CSSStyleSheet.prototype
    ) {
      const sheet = new CSSStyleSheet()
      sheet.replaceSync(shipTableLightDomCss)
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet]
      return
    }
  } catch {
    /* fall through to <style> fallback */
  }
  const style = doc.createElement('style')
  style.setAttribute('data-ship-table', '')
  style.textContent = shipTableLightDomCss
  doc.head.appendChild(style)
}

export class ShipTable extends LitElement {
  static styles = shipTableStyles

  static properties = {
    variant: { type: String, reflect: true },
    density: { type: String, reflect: true },
    dense: { type: Boolean, reflect: true },
    hoverable: { type: Boolean, reflect: true },
    striped: { type: Boolean, reflect: true },
    stickyHeader: { type: Boolean, reflect: true, attribute: 'sticky-header' },
    stickyFirstCol: { type: Boolean, reflect: true, attribute: 'sticky-first-col' },
    gutter: { type: Boolean, reflect: true },
    label: { type: String },
  }

  variant: Variant = 'slot'
  density: Density = 'default'
  dense = false
  hoverable = true
  striped = false
  stickyHeader = false
  stickyFirstCol = false
  gutter = false
  label = ''

  private structureObserver?: MutationObserver

  connectedCallback() {
    super.connectedCallback()
    installLightDomStyles(this.ownerDocument)
    this.addEventListener('click', this.onHeaderClick)
    this.addEventListener('click', this.onRowClick)
    this.addEventListener('keydown', this.onHeaderKey)
  }

  disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('click', this.onHeaderClick)
    this.removeEventListener('click', this.onRowClick)
    this.removeEventListener('keydown', this.onHeaderKey)
    this.structureObserver?.disconnect()
  }

  firstUpdated() {
    this.enhanceSortableHeaders()
    this.syncEmptyState()
    this.structureObserver = new MutationObserver(() => {
      this.enhanceSortableHeaders()
      this.syncEmptyState()
    })
    this.structureObserver.observe(this, { childList: true, subtree: true })
  }

  /** Give sortable headers a tabindex, role, chevron, and an initial
   * aria-sort so keyboard users can cycle the sort direction. Idempotent
   * across re-runs so Datastar-patched tables stay enhanced. */
  private enhanceSortableHeaders() {
    const ths = this.querySelectorAll<HTMLTableCellElement>(
      'th[data-sort-key]',
    )
    ths.forEach((th) => {
      if (!th.hasAttribute('aria-sort')) th.setAttribute('aria-sort', 'none')
      if (!th.hasAttribute('tabindex')) th.setAttribute('tabindex', '0')
      if (!th.hasAttribute('role')) th.setAttribute('role', 'columnheader')
      if (!th.querySelector('.ship-sort-indicator')) {
        const chev = document.createElement('span')
        chev.className = 'ship-sort-indicator'
        chev.setAttribute('aria-hidden', 'true')
        chev.style.cssText =
          'margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;'
        chev.innerHTML =
          '<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M2 4l3-2 3 2M2 6l3 2 3-2"/></svg>'
        th.appendChild(chev)
      }
    })
    this.paintSortChevrons()
  }

  private paintSortChevrons() {
    this.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach(
      (th) => {
        const dir = (th.getAttribute('aria-sort') || 'none') as SortDirection
        const ind = th.querySelector<HTMLElement>('.ship-sort-indicator')
        if (!ind) return
        if (dir === 'ascending') {
          ind.style.transform = 'rotate(180deg)'
          ind.style.opacity = '1'
        } else if (dir === 'descending') {
          ind.style.transform = 'rotate(0deg)'
          ind.style.opacity = '1'
        } else {
          ind.style.transform = 'rotate(0deg)'
          ind.style.opacity = '0.4'
        }
      },
    )
  }

  /** Toggle `[data-empty]` when the table has no body rows AND an
   * `empty` slot is provided. Covers all three structural patterns:
   *   - slot mode: <tr slot="head"> + bare <tr> default-slotted
   *   - native mode: <table><thead><tr/></thead><tbody><tr/></tbody></table>
   *   - native mode without explicit tbody */
  private syncEmptyState() {
    const bodyRow =
      this.querySelector(':scope > table tbody > tr') ||
      this.querySelector(':scope > table > tr:not(:has(th))') ||
      this.querySelector(':scope > tbody > tr') ||
      this.querySelector(':scope > tr:not([slot="head"])')
    const hasEmptySlot = this.querySelector('[slot="empty"]') != null
    if (!bodyRow && hasEmptySlot) this.setAttribute('data-empty', '')
    else this.removeAttribute('data-empty')
  }

  private onHeaderClick = (e: MouseEvent) => {
    const th = (e.target as HTMLElement).closest(
      'th[data-sort-key]',
    ) as HTMLTableCellElement | null
    if (!th || !this.contains(th)) return
    this.cycleSort(th)
  }

  private onHeaderKey = (e: KeyboardEvent) => {
    if (e.key !== 'Enter' && e.key !== ' ') return
    const th = (e.target as HTMLElement).closest(
      'th[data-sort-key]',
    ) as HTMLTableCellElement | null
    if (!th || !this.contains(th)) return
    e.preventDefault()
    this.cycleSort(th)
  }

  private onRowClick = (e: MouseEvent) => {
    const td = (e.target as HTMLElement).closest('td') as HTMLTableCellElement | null
    if (!td || !this.contains(td)) return
    // Skip gutter cells
    if (td.classList.contains('ship-table-gutter-cell')) return
    const tr = td.closest('tr') as HTMLTableRowElement | null
    if (!tr) return
    const rowId = tr.getAttribute('data-row-id')
    if (!rowId) return
    const index = tr.rowIndex
    this.dispatchEvent(new CustomEvent('ship-row-click', {
      detail: { id: rowId, index },
      bubbles: true,
      composed: true,
    }))
  }

  /** Cycle: none -> ascending -> descending -> none. Clearing peers keeps
   * the table single-sort; multi-sort consumers can listen for ship-sort
   * and manage aria-sort themselves. */
  private cycleSort(th: HTMLTableCellElement) {
    const key = th.getAttribute('data-sort-key') || ''
    const current = (th.getAttribute('aria-sort') || 'none') as SortDirection
    const next: SortDirection =
      current === 'none'
        ? 'ascending'
        : current === 'ascending'
          ? 'descending'
          : 'none'
    this.querySelectorAll<HTMLTableCellElement>('th[data-sort-key]').forEach(
      (peer) => {
        if (peer !== th) peer.setAttribute('aria-sort', 'none')
      },
    )
    th.setAttribute('aria-sort', next)
    this.paintSortChevrons()
    this.dispatchEvent(
      new CustomEvent('ship-sort', {
        detail: { key, direction: next },
        bubbles: true,
        composed: true,
      }),
    )
  }

  render() {
    // Native mode — consumer provides a full <table> in the default slot.
    // Shadow DOM is just the wrapper chrome + empty-state slot.
    if (this.variant === 'native') {
      return html`
        <div class="wrap" part="wrap">
          <slot></slot>
          <div class="empty" part="empty" role="status">
            <slot name="empty"></slot>
          </div>
        </div>
      `
    }
    // Slot mode — preserves the pre-polish test API. The shadow DOM
    // assembles the table scaffold; the consumer's <tr>s are slotted
    // into the thead/tbody.
    return html`
      <div class="wrap" part="wrap">
        <table part="table" aria-label=${this.label || nothing}>
          <thead part="thead"><slot name="head"></slot></thead>
          <tbody part="tbody"><slot></slot></tbody>
        </table>
        <div class="empty" part="empty" role="status">
          <slot name="empty"></slot>
        </div>
      </div>
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-table')) {
  customElements.define('ship-table', ShipTable)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-table': ShipTable
  }
}
