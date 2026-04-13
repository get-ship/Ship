// createShipApi — builds the API handle injected into trusted web component views.
// Direct function calls instead of the postMessage bridge used by ViewHost iframes.

import { DAEMON_BASE_URL, daemonFetch } from '#/lib/daemon-config'

/** Public contract that web component view authors program against. */
export interface ShipApi {
  getJobs(): Promise<unknown[]>
  getJob(jobId: string): Promise<unknown>
  createJob(params: Record<string, unknown>): Promise<unknown>
  emitEvent(
    eventType: string,
    entityId: string,
    payload: unknown,
  ): Promise<unknown>
  getSessionFiles(workspaceId: string): Promise<unknown>
  readFile(workspaceId: string, path: string): Promise<unknown>

  /** Subscribe to daemon SSE events. Returns an unsubscribe function. */
  onEvent(callback: (envelope: unknown) => void): () => void

  /** Currently active workspace branch ID, if any. */
  activeWorkspace: string | undefined
}

export function createShipApi(workspaceId: string | undefined): ShipApi {
  return {
    activeWorkspace: workspaceId,

    async getJobs() {
      const res = await daemonFetch('/api/runtime/jobs')
      if (!res.ok) throw new Error(`getJobs: ${res.status}`)
      const body = (await res.json()) as {
        ok: boolean
        data: { jobs: unknown[] }
      }
      return body.data.jobs
    },

    async getJob(jobId: string) {
      const res = await daemonFetch(
        `/api/runtime/jobs/${encodeURIComponent(jobId)}`,
      )
      if (!res.ok) throw new Error(`getJob: ${res.status}`)
      return await res.json()
    },

    async createJob(params: Record<string, unknown>) {
      const res = await daemonFetch('/api/runtime/jobs', {
        method: 'POST',
        body: JSON.stringify(params),
      })
      if (!res.ok) throw new Error(`createJob: ${res.status}`)
      return await res.json()
    },

    async emitEvent(
      eventType: string,
      entityId: string,
      payload: unknown,
    ) {
      const res = await daemonFetch('/api/events/emit', {
        method: 'POST',
        body: JSON.stringify({
          event_type: eventType,
          entity_id: entityId,
          payload,
        }),
      })
      if (!res.ok) throw new Error(`emitEvent: ${res.status}`)
      return await res.json()
    },

    async getSessionFiles(wsId: string) {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/session-files`,
      )
      if (!res.ok) throw new Error(`getSessionFiles: ${res.status}`)
      return await res.json()
    },

    async readFile(wsId: string, path: string) {
      const res = await daemonFetch(
        `/api/workspaces/${encodeURIComponent(wsId)}/session-files/${path}`,
      )
      if (!res.ok) throw new Error(`readFile: ${res.status}`)
      return await res.json()
    },

    onEvent(callback: (envelope: unknown) => void) {
      const es = new EventSource(`${DAEMON_BASE_URL}/api/runtime/events`)
      es.addEventListener('ship.event', (e: MessageEvent) => {
        try {
          const envelope = JSON.parse(e.data as string)
          callback(envelope)
        } catch {
          /* ignore malformed */
        }
      })
      return () => es.close()
    },
  }
}
