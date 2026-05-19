// Chrome breadcrumb slot — renders the items dispatched via
// `chrome.breadcrumb.set([...])`. The slot lives just below the top nav
// so views can stamp page-level context without taking control of the
// nav itself. Plain anchors keep shell's existing internal-link click
// delegate working (no extra routing here).

interface BreadcrumbItem {
  label: string
  href?: string
}

const HOST_ID = 'ship-breadcrumb-slot'

export function ensureBreadcrumbHost(): HTMLElement {
  let host = document.getElementById(HOST_ID)
  if (host) return host
  host = document.createElement('nav')
  host.id = HOST_ID
  host.setAttribute('aria-label', 'Breadcrumb')
  host.hidden = true
  // Insert just below the top strip so chrome breadcrumbs stamped by a
  // view appear above the canvas. Falls back to document.body when the
  // strip isn't mounted (e.g. focus mode).
  const anchor = document.getElementById('ship-shell-top')
  if (anchor?.parentNode) {
    anchor.parentNode.insertBefore(host, anchor.nextSibling)
  } else {
    document.body.appendChild(host)
  }
  return host
}

export function renderBreadcrumb(items: BreadcrumbItem[]): void {
  const host = ensureBreadcrumbHost()
  if (items.length === 0) {
    host.replaceChildren()
    host.hidden = true
    return
  }
  const frag = document.createDocumentFragment()
  items.forEach((item, i) => {
    if (i > 0) {
      const sep = document.createElement('span')
      sep.className = 'ship-breadcrumb-sep'
      sep.setAttribute('aria-hidden', 'true')
      sep.textContent = '/'
      frag.appendChild(sep)
    }
    const isLast = i === items.length - 1
    if (item.href && !isLast) {
      const a = document.createElement('a')
      a.href = item.href
      a.textContent = item.label
      a.className = 'ship-breadcrumb-link'
      frag.appendChild(a)
    } else {
      const span = document.createElement('span')
      span.textContent = item.label
      span.className = isLast ? 'ship-breadcrumb-current' : 'ship-breadcrumb-link'
      if (isLast) span.setAttribute('aria-current', 'page')
      frag.appendChild(span)
    }
  })
  host.replaceChildren(frag)
  host.hidden = false
}
