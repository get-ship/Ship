import { controller } from '@ship/ui-runtime'
import type { Scope } from './sui-annotation-list.sui'
import type { AnnotationEvent } from './sui-annotation.internals'

const LIST_STYLES = `
  ol { list-style:none; margin:0; padding:0; display:flex; flex-direction:column; gap:4px; }
  li { display:flex; align-items:center; gap:8px; padding:4px 8px;
       border:1px solid var(--border); border-radius:var(--radius-sm); background:var(--background); }
  .kind { font-size:var(--text-xs); text-transform:uppercase; letter-spacing:0.04em;
          color:var(--muted-foreground); min-width:48px; }
  .note { flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .empty { color:var(--muted-foreground); padding:4px 8px; }
  button { background:transparent; border:1px solid var(--border); border-radius:var(--radius-sm);
           color:var(--muted-foreground); cursor:pointer; font-size:var(--text-xs); padding:2px 6px; }
  button:hover { color:var(--destructive); border-color:var(--destructive); }
`.trim()

function renderList(container: HTMLElement, staged: AnnotationEvent[], host: HTMLElement) {
  container.innerHTML = ''

  if (!staged.length) {
    const d = document.createElement('div')
    d.className = 'empty'
    d.setAttribute('part', 'empty')
    d.textContent = 'No staged annotations.'
    container.appendChild(d)
    return
  }

  const style = document.createElement('style')
  style.textContent = LIST_STYLES
  container.appendChild(style)

  const ol = document.createElement('ol')
  ol.setAttribute('part', 'list')
  staged.forEach((a) => {
    const li = document.createElement('li')
    li.setAttribute('part', 'item')
    li.dataset.id = a.id

    const kind = document.createElement('span')
    kind.className = 'kind'
    kind.textContent = a.kind

    const note = document.createElement('span')
    note.className = 'note'
    note.title = a.note ?? ''
    note.textContent = a.note ?? '(no note)'

    const btn = document.createElement('button')
    btn.type = 'button'
    btn.setAttribute('aria-label', 'Remove annotation')
    btn.textContent = 'Remove'
    btn.addEventListener('click', () => {
      host.dispatchEvent(
        new CustomEvent('sui-annotation-remove', {
          detail: { id: a.id },
          bubbles: true,
          composed: true,
        }),
      )
    })

    li.append(kind, note, btn)
    ol.appendChild(li)
  })
  container.appendChild(ol)
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const containerEl = scope.containerEl() as HTMLElement | null
    if (!containerEl) return

    let _staged: AnnotationEvent[] = []

    renderList(containerEl, _staged, host)

    Object.defineProperty(host, 'staged', {
      get: () => _staged,
      set: (value: AnnotationEvent[]) => {
        _staged = Array.isArray(value) ? value : []
        renderList(containerEl, _staged, host)
      },
      configurable: true,
    })
  },
})
