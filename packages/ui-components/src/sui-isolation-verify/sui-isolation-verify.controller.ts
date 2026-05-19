import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-isolation-verify.sui'

interface Step {
  name: string
  ok: boolean
  detail: string
}

interface VerifyResult {
  ok: boolean
  steps: Step[]
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const body = host.shadowRoot?.getElementById('body')
    if (!body) return

    if ((scope.autoFetch() ?? 'true') === 'false') {
      // Views supplying data via slot/childlist do their own render;
      // we just observe.
      return
    }

    renderLoading(body)

    void fetchVerify()
      .then((result) => {
        if (!result) {
          host.setAttribute('ok', 'false')
          renderError(body, 'verify endpoint returned no data')
          return
        }
        host.setAttribute('ok', result.ok ? 'true' : 'false')
        renderSteps(body, result.steps)
      })
      .catch((err) => {
        host.setAttribute('ok', 'false')
        renderError(body, err?.message ?? 'verify request failed')
      })
  },
})

function renderLoading(target: HTMLElement) {
  target.replaceChildren(makeNode('div', { class: 'loading' }, 'Running verification…'))
}

function renderError(target: HTMLElement, message: string) {
  target.replaceChildren(
    makeNode(
      'ul',
      {},
      makeNode(
        'li',
        {},
        makeNode('span', { class: 'dot fail' }),
        makeNode(
          'span',
          { class: 'check' },
          makeNode('span', { class: 'name' }, 'Verify failed'),
          makeNode('span', { class: 'detail' }, message),
        ),
      ),
    ),
  )
}

function renderSteps(target: HTMLElement, steps: Step[]) {
  if (!steps.length) {
    target.replaceChildren(
      makeNode('div', { class: 'loading' }, 'No checks reported'),
    )
    return
  }
  const ul = makeNode(
    'ul',
    {},
    ...steps.map((s) =>
      makeNode(
        'li',
        {},
        makeNode('span', { class: `dot ${s.ok ? 'pass' : 'fail'}`, 'data-pass': s.ok ? 'true' : 'false' }),
        makeNode(
          'span',
          { class: 'check' },
          makeNode('span', { class: 'name' }, s.name),
          s.detail
            ? makeNode('span', { class: 'detail' }, s.detail)
            : null,
        ),
      ),
    ),
  )
  target.replaceChildren(ul)
}

function makeNode(
  tag: string,
  attrs: Record<string, string> = {},
  ...children: (Node | string | null)[]
): HTMLElement {
  const el = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) {
    el.setAttribute(k, v)
  }
  for (const c of children) {
    if (c == null) continue
    el.append(typeof c === 'string' ? document.createTextNode(c) : c)
  }
  return el
}

async function fetchVerify(): Promise<VerifyResult | null> {
  try {
    const resp = await fetch('/api/v1/isolation/verify', {
      method: 'POST',
      headers: { accept: 'application/json' },
    })
    if (!resp.ok) return null
    const json = await resp.json()
    const data = json?.data
    if (!data) return null
    return {
      ok: Boolean(data.ok),
      steps: Array.isArray(data.steps) ? data.steps.map(coerceStep) : [],
    }
  } catch {
    return null
  }
}

function coerceStep(raw: unknown): Step {
  const r = (raw ?? {}) as Record<string, unknown>
  return {
    name: String(r.name ?? ''),
    ok: Boolean(r.ok),
    detail: String(r.detail ?? ''),
  }
}
