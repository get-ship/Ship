// ViewMount — React wrapper that loads a trusted web component view bundle,
// creates the custom element, and injects the Ship API handle.

import { useRef, useEffect, useState, useMemo } from 'react'
import { createShipApi, type ShipApi } from './createShipApi'
import { daemonFetch } from '#/lib/daemon-config'
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

/** Custom element with Ship injected properties. */
interface ShipElement extends HTMLElement {
  shipApi?: ShipApi
  shipData?: Record<string, unknown>
  shipTheme?: string
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

  const shipApi = useMemo(
    () => createShipApi(workspaceId),
    [workspaceId],
  )

  // Load the bundle script and create the custom element.
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    setStatus('loading')
    let cancelled = false
    let blobUrl: string | null = null

    // Fetch bundle with auth, create blob URL, load as module script.
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
            el.shipApi = shipApi
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
  }, [bundleUrl, elementName, shipApi, data])

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
