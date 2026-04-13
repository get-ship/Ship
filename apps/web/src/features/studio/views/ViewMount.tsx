// ViewMount — React wrapper that loads a trusted web component view bundle,
// creates the custom element, and injects a ShipClient instance built on
// DirectTransport (loopback fetch + EventSource).
//
// The injected client carries the same surface that the old `createShipApi`
// handle exposed — getJobs/getJob/createJob/emitEvent/getSessionFiles/
// readFile/onEvent plus an `activeWorkspace` property — so view-author code
// continues to run unchanged.

import { useRef, useEffect, useState, useMemo } from 'react'
import { ShipClient, DirectTransport } from '@ship/sdk'
import { DAEMON_BASE_URL, daemonFetch, daemonHeaders } from '#/lib/daemon-config'
import { getResolvedTheme } from '#/features/studio/session/canvas-helpers'

export interface ViewMountProps {
  /** URL to the view's JS bundle (served by daemon or registry). */
  bundleUrl: string
  /** Custom element tag name (e.g. "ship-pipeline-monitor"). */
  elementName: string
  /** Data props to pass to the web component. */
  data?: Record<string, unknown>
  /** Active workspace branch ID. */
  workspaceId?: string
}

/** Custom element with Ship-injected properties. */
interface ShipElement extends HTMLElement {
  shipApi?: ShipClient
  shipData?: Record<string, unknown>
  shipTheme?: string
}

/**
 * Build a DirectTransport that honors the app's daemon-config auth token.
 * We forward through `daemonFetch` so the shared cached token is used and
 * stays in sync when `setDaemonToken` is called by PlatformProvider.
 */
function buildTransport(): DirectTransport {
  const transport = new DirectTransport({ baseUrl: DAEMON_BASE_URL })
  // Monkey-patch the request method to route through daemonFetch so we
  // pick up the cached auth header. This keeps a single auth source of
  // truth without lifting the token into SDK construction.
  const original = transport.request.bind(transport)
  transport.request = async <T,>(path: string, init?: Parameters<typeof original>[1]) => {
    const url = path.startsWith('http')
      ? path
      : `${path.startsWith('/api') ? '' : '/api'}${path}`
    const res = await daemonFetch(url, {
      method: init?.method ?? 'GET',
      headers: daemonHeaders(),
      body: init?.body !== undefined ? JSON.stringify(init.body) : undefined,
      signal: init?.signal,
    })
    if (!res.ok) {
      const text = await res.text().catch(() => res.statusText)
      throw new Error(`Ship API ${res.status}: ${text}`)
    }
    if (res.status === 204) return undefined as T
    const json = (await res.json()) as unknown
    if (init?.raw) return json as T
    if (
      json &&
      typeof json === 'object' &&
      'ok' in (json as Record<string, unknown>) &&
      'data' in (json as Record<string, unknown>)
    ) {
      return (json as { data: T }).data
    }
    return json as T
  }
  return transport
}

export function ViewMount({
  bundleUrl,
  elementName,
  data,
  workspaceId,
}: ViewMountProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const elementRef = useRef<ShipElement | null>(null)
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>(
    'loading',
  )
  const [errorMsg, setErrorMsg] = useState<string>('')

  const shipClient = useMemo(
    () => new ShipClient({
      baseUrl: DAEMON_BASE_URL,
      activeWorkspace: workspaceId,
      transport: buildTransport(),
    }),
    [workspaceId],
  )

  // Tear down the client when it changes (workspace switch) or on unmount.
  useEffect(() => {
    return () => shipClient.destroy()
  }, [shipClient])

  // Load the bundle script and create the custom element.
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    setStatus('loading')
    let cancelled = false
    let blobUrl: string | null = null

    ;(async () => {
      try {
        const res = await daemonFetch(bundleUrl.replace(/^https?:\/\/[^/]+/, ''))
        if (!res.ok) throw new Error(`${res.status} ${res.statusText}`)
        const text = await res.text()
        if (cancelled) return

        const blob = new Blob([text], { type: 'application/javascript' })
        blobUrl = URL.createObjectURL(blob)

        const script = document.createElement('script')
        script.src = blobUrl
        script.type = 'module'
        script.onload = () => {
          if (cancelled) return
          try {
            const el = document.createElement(elementName) as ShipElement
            el.shipApi = shipClient
            el.shipData = data
            el.shipTheme = getResolvedTheme()
            container.appendChild(el)
            elementRef.current = el
            setStatus('ready')
          } catch (err) {
            setErrorMsg(err instanceof Error ? err.message : 'Failed to create element')
            setStatus('error')
          }
        }
        script.onerror = () => {
          if (cancelled) return
          setErrorMsg(`Failed to execute bundle: ${bundleUrl}`)
          setStatus('error')
        }
        document.head.appendChild(script)
      } catch (err) {
        if (cancelled) return
        setErrorMsg(err instanceof Error ? err.message : `Failed to load bundle: ${bundleUrl}`)
        setStatus('error')
      }
    })()

    return () => {
      cancelled = true
      if (blobUrl) URL.revokeObjectURL(blobUrl)
      if (elementRef.current) {
        elementRef.current.remove()
        elementRef.current = null
      }
    }
  }, [bundleUrl, elementName, shipClient, data])

  // Keep shipData in sync when data prop changes.
  useEffect(() => {
    if (elementRef.current) {
      elementRef.current.shipData = data
    }
  }, [data])

  // Sync theme changes to the web component.
  useEffect(() => {
    const syncTheme = () => {
      if (elementRef.current) {
        elementRef.current.shipTheme = getResolvedTheme()
      }
    }
    const observer = new MutationObserver(syncTheme)
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class', 'data-theme'],
    })
    return () => observer.disconnect()
  }, [])

  if (status === 'error') {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <div className="text-center">
          <p className="text-sm font-medium">Failed to mount view</p>
          <p className="text-xs mt-1">{errorMsg}</p>
        </div>
      </div>
    )
  }

  return (
    <>
      {status === 'loading' && (
        <div className="flex items-center justify-center h-full text-muted-foreground">
          <p className="text-xs animate-pulse">Loading component...</p>
        </div>
      )}
      <div
        ref={containerRef}
        className="w-full h-full"
        style={{ display: status === 'loading' ? 'none' : undefined }}
      />
    </>
  )
}
