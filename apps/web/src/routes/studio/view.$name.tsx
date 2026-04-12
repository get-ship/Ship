import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { MonitorPlay } from 'lucide-react'
import { DAEMON_BASE_URL, daemonFetch } from '#/lib/daemon-config'
import { usePlatform } from '#/features/studio/PlatformProvider'

export const Route = createFileRoute('/studio/view/$name')({
  component: ExtensionViewPage,
  ssr: false,
})

function ExtensionViewPage() {
  const { name } = Route.useParams()
  const { client, daemon } = usePlatform()
  const containerRef = useRef<HTMLDivElement>(null)
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const [errorMsg, setErrorMsg] = useState('')

  useEffect(() => {
    if (daemon !== 'connected') return
    const token = client.getToken()
    if (!token) return

    let cancelled = false
    let loadedScripts: HTMLScriptElement[] = []

    ;(async () => {
      try {
        const res = await daemonFetch(`/api/runtime/view/${encodeURIComponent(name)}/raw?token=${encodeURIComponent(token)}`)
        if (!res.ok) throw new Error(`${res.status}`)
        const html = await res.text()
        if (cancelled) return

        const container = containerRef.current
        if (!container) return

        // Parse the HTML and extract body + scripts
        const doc = new DOMParser().parseFromString(html, 'text/html')

        // Inject styles
        doc.querySelectorAll('style').forEach((s) => {
          const clone = document.createElement('style')
          clone.textContent = s.textContent
          container.appendChild(clone)
        })

        // Inject body content (with data-* attributes intact) BEFORE scripts
        // so Datastar finds the attributes when it initializes.
        const bodyClone = document.createElement('div')
        bodyClone.innerHTML = doc.body.innerHTML
        while (bodyClone.firstChild) container.appendChild(bodyClone.firstChild)

        // Inject stylesheet links (for /api/runtime/assets/view-tokens.css etc.)
        doc.querySelectorAll('link[rel="stylesheet"]').forEach((l) => {
          const href = l.getAttribute('href')
          if (!href) return
          const clone = document.createElement('link')
          clone.rel = 'stylesheet'
          clone.href = href
          document.head.appendChild(clone)
        })

        // Now execute scripts — Datastar will scan the DOM on startup.
        // Copy src when present, otherwise inline textContent.
        const scripts = Array.from(doc.querySelectorAll('script'))
        for (const src of scripts) {
          const s = document.createElement('script')
          if (src.type) s.type = src.type
          const srcAttr = src.getAttribute('src')
          if (srcAttr) {
            s.src = srcAttr
          } else {
            s.textContent = src.textContent
          }
          document.head.appendChild(s)
          loadedScripts.push(s)
        }

        setStatus('ready')
      } catch (err) {
        if (!cancelled) {
          setErrorMsg(err instanceof Error ? err.message : String(err))
          setStatus('error')
        }
      }
    })()

    return () => {
      cancelled = true
      loadedScripts.forEach((s) => s.remove())
      if (containerRef.current) containerRef.current.innerHTML = ''
    }
  }, [name, client, daemon])

  if (status === 'error') {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <div className="text-center">
          <MonitorPlay className="size-6 mx-auto mb-2 opacity-40" />
          <p className="text-sm font-medium">Failed to load view</p>
          <p className="text-xs mt-1">{errorMsg}</p>
        </div>
      </div>
    )
  }

  return <div ref={containerRef} className="flex-1 overflow-auto" />
}
