// SUI tutorial mock-daemon service worker.
//
// Intercepts iframe requests under /tutorial/ to:
//   - Stream <data live> sources via SSE (`/_sui/data/<source>`)
//   - Accept morph patches (`POST /_sui/morph/<view>`)
//   - Return mock REST payloads (`/_sui/api/<path>`)
//
// Tutorial scenarios are tiny JS modules the parent posts to the SW
// after registration. Each scenario describes a timeline:
//   {
//     '/users': {
//       initial: { kind: 'ok', value: { items: [...] } },
//       frames: [
//         { afterMs: 1500, payload: { kind: 'ok', value: { items: [...] } } },
//         { afterMs: 3000, payload: { kind: 'error', error: { message: '…' } } },
//       ],
//     },
//   }
//
// The SW is plain JS — no bundler, no esm.sh dependency. If route
// growth ever warrants Hono's middleware, that's a one-off swap;
// the scenario contract above stays stable.

const VERSION = 'sui-tutorial-sw-v1'

// Per-scenario timeline keyed by source URL (e.g. '/users').
// Each value: { initial: any, frames: Array<{ afterMs, payload }> }
const scenarios = new Map()

self.addEventListener('install', () => {
  self.skipWaiting()
})

self.addEventListener('activate', (event) => {
  event.waitUntil(self.clients.claim())
})

// Parent posts scenario data here. The same client can register
// multiple sources at once and re-register when the user switches
// tutorial steps.
self.addEventListener('message', (event) => {
  const data = event.data
  if (!data || typeof data !== 'object') return

  if (data.kind === 'load-scenario') {
    // data.scenario is the per-source map shown in the file header.
    scenarios.clear()
    for (const [sourceUrl, timeline] of Object.entries(data.scenario)) {
      scenarios.set(sourceUrl, timeline)
    }
    event.source?.postMessage({ kind: 'scenario-loaded', count: scenarios.size })
    return
  }

  if (data.kind === 'ping') {
    event.source?.postMessage({ kind: 'pong', version: VERSION })
  }
})

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url)
  if (!url.pathname.startsWith('/_sui/')) return

  // SSE: `/_sui/data/<source>` → text/event-stream from the scenario
  if (url.pathname.startsWith('/_sui/data/')) {
    const sourceUrl = url.pathname.replace(/^\/_sui\/data/, '')
    const timeline = scenarios.get(sourceUrl)
    if (!timeline) {
      event.respondWith(new Response(`no scenario for ${sourceUrl}`, { status: 404 }))
      return
    }
    event.respondWith(streamScenario(timeline))
    return
  }

  // Morph push: `POST /_sui/morph/<view>` → echoed back as a morph
  // frame on the matching SSE stream. v1: not yet implemented (needs
  // tracking active SSE controllers per stream). For now, accept the
  // body and respond ok so authoring code that calls patch_view
  // doesn't crash.
  if (url.pathname.startsWith('/_sui/morph/')) {
    event.respondWith(new Response('{"ok":true}', {
      status: 200,
      headers: { 'content-type': 'application/json' },
    }))
    return
  }

  // Mock REST: `/_sui/api/<path>` → static payload from scenario if
  // declared as an `'api:/<path>'` key.
  if (url.pathname.startsWith('/_sui/api/')) {
    const apiKey = 'api:' + url.pathname.replace(/^\/_sui\/api/, '')
    const payload = scenarios.get(apiKey)
    if (payload) {
      event.respondWith(new Response(JSON.stringify(payload), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }))
      return
    }
    event.respondWith(new Response('null', { status: 404 }))
    return
  }
})

/**
 * Open an SSE stream whose first chunk is the scenario's `initial`
 * envelope and whose subsequent chunks fire on the declared timeline.
 */
function streamScenario(timeline) {
  const stream = new ReadableStream({
    start(controller) {
      const enc = new TextEncoder()
      const send = (payload) => controller.enqueue(enc.encode(`data: ${JSON.stringify(payload)}\n\n`))

      send(timeline.initial)

      for (const frame of timeline.frames || []) {
        setTimeout(() => {
          try { send(frame.payload) } catch { /* stream closed */ }
        }, frame.afterMs)
      }
      // Open-ended: don't close the controller. The iframe's
      // EventSource stays subscribed until the tab/page closes.
    },
  })
  return new Response(stream, {
    status: 200,
    headers: {
      'content-type': 'text/event-stream',
      'cache-control': 'no-cache',
      'connection': 'keep-alive',
    },
  })
}
