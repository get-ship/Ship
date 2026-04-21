// ViewHost — iframe container that bridges ship-sdk postMessage to the
// daemon via a parent-side ShipClient. Handles request/response proxying,
// SSE event forwarding, and theme sync.
//
// NOTE: Phase 6 punts on formalizing `BridgeTransport` — the wire protocol
// used between the iframe and this host is still the ad-hoc one defined
// here (`{__ship, type:'request', method, params, seq}`). A follow-up
// can move both sides onto a `BridgeTransport`-shaped client. For now
// this component owns a real ShipClient (DirectTransport) and maps
// postMessage methods onto its surface — so the daemon HTTP calls are
// no longer duplicated between ViewMount and ViewHost.

import { useRef, useEffect, useCallback, useMemo } from 'react'
import { ShipClient } from '../index'
import { DAEMON_BASE_URL } from './daemon-config.js'
import { getResolvedTheme } from './canvas-helpers.js'

interface ViewHostProps {
  /** Raw HTML content of the view (loaded by the parent). */
  html: string
  /** Active workspace branch ID for file operations. */
  workspaceId?: string
}

export function ViewHost({ html, workspaceId }: ViewHostProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const client = useMemo(
    () => new ShipClient({ baseUrl: DAEMON_BASE_URL, activeWorkspace: workspaceId }),
    [workspaceId],
  )

  useEffect(() => () => client.destroy(), [client])

  // Bridge postMessage requests from the view to the parent ShipClient.
  const handleMessage = useCallback(async (e: MessageEvent) => {
    const msg = e.data as
      | { __ship?: boolean; type?: string; method?: string; params?: Record<string, unknown>; seq?: number }
      | undefined
    if (!msg || !msg.__ship || msg.type !== 'request') return

    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    if (e.source !== iframe.contentWindow) return

    const { method, params, seq } = msg
    try {
      const data = await routeRequest(client, method ?? '', params, workspaceId)
      iframe.contentWindow.postMessage({ __ship: true, type: 'response', seq, data }, '*')
    } catch (err) {
      const error = err instanceof Error ? err.message : String(err)
      iframe.contentWindow.postMessage({ __ship: true, type: 'response', seq, error }, '*')
    }
  }, [client, workspaceId])

  useEffect(() => {
    window.addEventListener('message', handleMessage)
    return () => window.removeEventListener('message', handleMessage)
  }, [handleMessage])

  // Forward SSE events from the ShipClient to the view iframe.
  useEffect(() => {
    const iframe = iframeRef.current
    if (!iframe) return
    return client.onEvent((envelope) => {
      iframe.contentWindow?.postMessage({ __ship: true, type: 'event', envelope }, '*')
    })
  }, [client])

  // Sync theme to iframe on load and on host theme changes.
  const postTheme = useCallback(() => {
    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    const theme = getResolvedTheme()
    iframe.contentWindow.postMessage({ type: 'theme', theme }, '*')
    iframe.contentWindow.postMessage({ __ship: true, type: 'theme', theme }, '*')
  }, [])

  useEffect(() => {
    const observer = new MutationObserver(() => postTheme())
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] })
    return () => observer.disconnect()
  }, [postTheme])

  return (
    <iframe
      ref={iframeRef}
      srcDoc={html}
      title="Ship View"
      // allow-same-origin needed: runtime views make fetch calls to the daemon.
      // These are trusted views served by the daemon, not user-generated content.
      // Third-party extensions should use a stricter sandbox.
      sandbox="allow-same-origin allow-scripts"
      className="w-full h-full border-0"
      onLoad={postTheme}
    />
  )
}

// Route SDK method calls to the parent-side ShipClient.
async function routeRequest(
  client: ShipClient,
  method: string,
  params: Record<string, unknown> | undefined,
  activeWsId?: string,
): Promise<unknown> {
  switch (method) {
    case 'workspace.active':
      return activeWsId ?? null
    case 'jobs.list':
      return client.getJobs()
    case 'jobs.create':
      return client.createJob(params ?? {})
    case 'events.emit': {
      const p = params as { event_type: string; entity_id: string; payload: unknown }
      return client.emitEvent(p.event_type, p.entity_id, p.payload)
    }
    case 'files.list': {
      const p = params as { workspace_id: string }
      return client.getSessionFiles(p.workspace_id)
    }
    case 'files.read': {
      const p = params as { workspace_id: string; path: string }
      return client.readFile(p.workspace_id, p.path)
    }
    default:
      throw new Error(`unknown method: ${method}`)
  }
}
