import { afterEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-sandbox-list/sui-sandbox-list.compiled.js'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-sandbox-list>', () => {
  it('registers the custom element', () => {
    expect(customElements.get('sui-sandbox-list')).toBeDefined()
  })

  it('shows loading state before fetch settles', async () => {
    vi.spyOn(globalThis, 'fetch').mockImplementation(
      () => new Promise(() => {}),
    )
    const { el } = mount(reg, 'sui-sandbox-list')
    await flush()
    expect(shadow(el).querySelector('.loading')?.textContent ?? '').toContain(
      'Loading',
    )
  })

  it('renders the empty state when no sandboxes registered', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(JSON.stringify({ ok: true, data: { sandboxes: [] } }), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    )
    const { el } = mount(reg, 'sui-sandbox-list')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(el.getAttribute('count')).toBe('0')
    expect(shadow(el).querySelector('.empty')?.textContent ?? '').toContain(
      'No sandboxes',
    )
  })

  it('groups sandboxes by project + renders per-row state', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          ok: true,
          data: {
            sandboxes: [
              {
                id: 'sbx_abc',
                project: '/home/user/project-a',
                name: 'default',
                tier: 'container',
                strategy: 'docker',
                state: 'running',
                endpoint: { kind: 'in_process' },
                actors: ['actor-1', 'actor-2'],
                created_at: '2026-01-01T00:00:00Z',
                idle_since: null,
                has_owner: true,
              },
              {
                id: 'sbx_def',
                project: '/home/user/project-a',
                name: 'untrusted',
                tier: 'container',
                strategy: 'docker',
                state: 'idle',
                endpoint: { kind: 'in_process' },
                actors: [],
                created_at: '2026-01-01T00:00:00Z',
                idle_since: '2026-01-01T00:05:00Z',
                has_owner: true,
              },
              {
                id: 'sbx_ghi',
                project: '/home/user/other-proj',
                name: 'default',
                tier: 'none',
                strategy: 'noop',
                state: 'running',
                endpoint: { kind: 'in_process' },
                actors: ['actor-3'],
                created_at: '2026-01-01T00:00:00Z',
                idle_since: null,
                has_owner: false,
              },
            ],
          },
        }),
        { status: 200, headers: { 'content-type': 'application/json' } },
      ),
    )
    const { el } = mount(reg, 'sui-sandbox-list')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(el.getAttribute('count')).toBe('3')
    const groups = shadow(el).querySelectorAll('.project')
    expect(groups.length).toBe(2)

    // States rendered with data-state attributes for CSS keying.
    const states = Array.from(shadow(el).querySelectorAll('.state')).map((s) =>
      s.getAttribute('data-state'),
    )
    // Projects sort alphabetically (other-proj < project-a); within
    // a project, sandboxes preserve input order.
    expect(states).toEqual(['running', 'running', 'idle'])

    // Project names trim to last segment for readability.
    const projectLabels = Array.from(
      shadow(el).querySelectorAll('.project-name'),
    ).map((n) => n.textContent?.trim())
    expect(projectLabels).toContain('project-a')
    expect(projectLabels).toContain('other-proj')

    // Sandbox ids + names visible.
    const text = shadow(el).textContent ?? ''
    expect(text).toContain('sbx_abc')
    expect(text).toContain('untrusted')
    expect(text).toContain('2 actors')
  })

  it('renders error state when fetch fails', async () => {
    vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('network'))
    const { el } = mount(reg, 'sui-sandbox-list')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(shadow(el).querySelector('.error')?.textContent ?? '').toContain(
      'network',
    )
  })

  it('renders error when endpoint returns non-200', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response('forbidden', { status: 403 }),
    )
    const { el } = mount(reg, 'sui-sandbox-list')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(shadow(el).querySelector('.error')?.textContent ?? '').toContain(
      '403',
    )
  })

  it('skips auto-fetch when auto-fetch="false"', async () => {
    vi.restoreAllMocks()
    const spy = vi.spyOn(globalThis, 'fetch')
    mount(reg, 'sui-sandbox-list', { 'auto-fetch': 'false' })
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(spy).not.toHaveBeenCalled()
  })

  it('count reflects on host attribute for parent observation', async () => {
    vi.spyOn(globalThis, 'fetch').mockResolvedValue(
      new Response(
        JSON.stringify({
          ok: true,
          data: {
            sandboxes: [
              {
                id: 'a',
                project: 'p',
                name: 'default',
                tier: 'none',
                strategy: 'noop',
                state: 'running',
                endpoint: null,
                actors: [],
                created_at: '',
                idle_since: null,
                has_owner: false,
              },
            ],
          },
        }),
        { status: 200 },
      ),
    )
    const { el } = mount(reg, 'sui-sandbox-list')
    await new Promise((r) => setTimeout(r, 30))
    await flush()
    expect(el.getAttribute('count')).toBe('1')
  })
})
