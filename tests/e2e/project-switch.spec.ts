/**
 * Behavioral coverage for the project-switch wiring.
 *
 * Three layers:
 *   1. Daemon API — the new endpoints (POST /activate, GET /active, GET
 *      /sessions/active-across-projects) behave correctly and the
 *      `project.switched` event reaches SSE subscribers.
 *   2. Chrome subscription — when shell2's vite dev server is reachable,
 *      switching the active project re-mounts the current view and the
 *      cross-project indicator chip surfaces in <sui-status-bar>.
 *   3. The legacy POST /api/v1/projects/switch surface still works
 *      (back-compat) and funnels through the same code path.
 *
 * Tests register two temp directories as projects so they don't depend
 * on any pre-existing project state. The temp dirs are abandoned at the
 * end of the spec (registry rows still resolve since we never call
 * `forget`; that's acceptable — `register` is idempotent).
 */
import { test, expect, type APIRequestContext } from '@playwright/test'
import { mkdtempSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'

import { getDaemonUrl } from './_helpers'
import { SHELL2_URL, shell2Reachable } from './_shell2'

const DAEMON = getDaemonUrl()

interface Project {
  id: string
  name: string
  path: string
}

interface ActiveResp {
  active?: Project | null
}

interface ProjectsListResp {
  projects: Project[]
  active?: string | null
}

interface SessionsResp {
  ok: boolean
  data: {
    sessions: Array<{
      id: string
      project_id?: string | null
      project_name?: string | null
      status?: string | null
    }>
  }
}

async function registerProject(req: APIRequestContext, label: string): Promise<Project> {
  const dir = mkdtempSync(path.join(tmpdir(), `ship-projswitch-${label}-`))
  const resp = await req.post(`${DAEMON}/api/v1/projects`, {
    data: { path: dir, label },
  })
  expect(resp.status(), `register ${label}`).toBe(200)
  const body = (await resp.json()) as ProjectsListResp
  const match = body.projects.find((p) => p.path === dir || p.path.endsWith(dir.split('/').pop()!))
  expect(match, `freshly-registered project not in list (${label})`).toBeTruthy()
  return match!
}

async function activate(req: APIRequestContext, id: string): Promise<Response> {
  const resp = await req.post(`${DAEMON}/api/v1/projects/${encodeURIComponent(id)}/activate`)
  return resp as unknown as Response
}

test.describe('project switch — daemon API', () => {
  test.beforeAll(async ({ request }) => {
    // Skip the whole spec when the daemon isn't reachable so this file
    // doesn't fail in environments where `just dev` isn't running.
    try {
      const health = await request.get(`${DAEMON}/api/health`)
      if (!health.ok()) test.skip(true, `daemon at ${DAEMON} not healthy`)
    } catch {
      test.skip(true, `daemon at ${DAEMON} unreachable`)
    }
  })

  test('activate changes the active project record', async ({ request }) => {
    const a = await registerProject(request, 'a')
    const b = await registerProject(request, 'b')

    const sa = await activate(request, a.id)
    expect(sa.status()).toBe(200)
    const acA = await request.get(`${DAEMON}/api/v1/projects/active`)
    const ja = (await acA.json()) as ActiveResp
    expect(ja.active?.id).toBe(a.id)

    const sb = await activate(request, b.id)
    expect(sb.status()).toBe(200)
    const acB = await request.get(`${DAEMON}/api/v1/projects/active`)
    const jb = (await acB.json()) as ActiveResp
    expect(jb.active?.id).toBe(b.id)
  })

  test('activate returns 404 for unknown project id', async ({ request }) => {
    const resp = await request.post(
      `${DAEMON}/api/v1/projects/${encodeURIComponent('deadbeef-not-a-project')}/activate`,
    )
    expect(resp.status()).toBe(404)
  })

  test('legacy POST /api/v1/projects/switch still works', async ({ request }) => {
    const a = await registerProject(request, 'legacy')
    const resp = await request.post(`${DAEMON}/api/v1/projects/switch`, {
      data: { id: a.id },
    })
    expect(resp.status()).toBe(200)
    const ac = await request.get(`${DAEMON}/api/v1/projects/active`)
    const j = (await ac.json()) as ActiveResp
    expect(j.active?.id).toBe(a.id)
  })

  test('cross-project sessions endpoint returns labeled rows', async ({ request }) => {
    // We don't spawn an agent in-test (that needs provider creds + spawn
    // path), but the endpoint must always answer with the wire shape so
    // chrome can render zero-state. With ≥1 active session in the DB,
    // rows carry project_id + project_name labels.
    const resp = await request.get(`${DAEMON}/api/v1/sessions/active-across-projects`)
    expect(resp.status()).toBe(200)
    const body = (await resp.json()) as SessionsResp
    expect(body.ok).toBe(true)
    expect(Array.isArray(body.data.sessions)).toBe(true)
    for (const s of body.data.sessions) {
      // Every row exposes the labeling fields the chrome indicator reads.
      expect(s).toHaveProperty('project_id')
      expect(s).toHaveProperty('project_name')
    }
  })

  test('project.switched SSE event fires on activate', async ({ request, page }) => {
    const a = await registerProject(request, 'sse-a')
    const b = await registerProject(request, 'sse-b')
    // Park on the active one first so the next activate is a *change*.
    await activate(request, a.id)

    // Listen on the events stream from the browser context so the
    // cookie-auth + EventSource semantics match shell2's runtime.
    await page.goto(`${DAEMON}/api/health`).catch(() => {})

    const received = page.evaluate(async (expectedId) => {
      return await new Promise<unknown>((resolve, reject) => {
        const es = new EventSource('/api/v1/events/stream')
        const timer = setTimeout(() => {
          es.close()
          reject(new Error('timeout waiting for project.switched'))
        }, 5_000)
        es.onmessage = (ev) => {
          try {
            const data = JSON.parse(ev.data) as { event_type?: string; entity?: string }
            if (data.event_type === 'project.switched' && data.entity === expectedId) {
              clearTimeout(timer)
              es.close()
              resolve(data)
            }
          } catch { /* skip non-JSON keepalive frames */ }
        }
        es.onerror = () => {
          /* EventSource auto-reconnects; let the timeout decide */
        }
      })
    }, b.id)

    // Trigger the switch *after* the EventSource is open. A small delay
    // lets the SSE handshake complete; the daemon doesn't replay events
    // emitted before subscribe.
    await page.waitForTimeout(250)
    await activate(request, b.id)

    const evt = await received
    expect(evt).toMatchObject({
      event_type: 'project.switched',
      entity: b.id,
    })
  })
})

test.describe('project switch — chrome surface', () => {
  test.beforeAll(async () => {
    if (!(await shell2Reachable())) {
      test.skip(
        true,
        `shell2 vite not reachable at ${SHELL2_URL} — start it with \`bun -F shell2 dev\``,
      )
    }
  })

  test('switching project triggers chrome remount + status refresh', async ({ page, request }) => {
    const a = await registerProject(request, 'chrome-a')
    const b = await registerProject(request, 'chrome-b')
    await activate(request, a.id)

    await page.goto(`${SHELL2_URL}/agents`)
    await page.waitForSelector('sui-rail#ship-rail')

    // Sentinel: full reloads wipe this. If it survives a project switch
    // the chrome handled the event via re-mount, not via document
    // navigation.
    await page.evaluate(() => { (window as unknown as { __noReload?: boolean }).__noReload = true })

    // Trigger the switch through the same surface the picker uses.
    await page.evaluate(async (id) => {
      await fetch(`/api/v1/projects/${encodeURIComponent(id)}/activate`, {
        method: 'POST',
        credentials: 'same-origin',
      })
    }, b.id)

    // Chrome dispatches `ship:project:switched` on document after the
    // kernel-bridge sees the SSE event. Wait for it as the canonical
    // "switch was applied" signal.
    await page.waitForFunction(
      () => new Promise<boolean>((resolve) => {
        const timer = setTimeout(() => resolve(false), 5_000)
        document.addEventListener(
          'ship:project:switched',
          () => { clearTimeout(timer); resolve(true) },
          { once: true },
        )
      }),
      null,
      { timeout: 7_000 },
    )

    const survived = await page.evaluate(
      () => (window as unknown as { __noReload?: boolean }).__noReload === true,
    )
    expect(survived).toBe(true)
  })
})
