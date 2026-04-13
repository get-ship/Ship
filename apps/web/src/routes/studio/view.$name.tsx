import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { MonitorPlay } from 'lucide-react'
import { daemonFetch } from '#/lib/daemon-config'
import { usePlatform } from '#/features/studio/PlatformProvider'

export const Route = createFileRoute('/studio/view/$name')({
  component: ExtensionViewPage,
  ssr: false,
})

function ExtensionViewPage() {
  const { name } = Route.useParams()
  const { client, daemon } = usePlatform()
  const hostRef = useRef<HTMLDivElement>(null)
  const [status, setStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const [errorMsg, setErrorMsg] = useState('')

  useEffect(() => {
    if (daemon !== 'connected') return
    const token = client.getToken()
    if (!token) return

    const host = hostRef.current
    if (!host) return

    let cancelled = false
    let ownedScripts: HTMLScriptElement[] = []
    let ownedStyleLinks: HTMLLinkElement[] = []

    ;(async () => {
      try {
        const res = await daemonFetch(`/api/runtime/view/${encodeURIComponent(name)}/raw?token=${encodeURIComponent(token)}`)
        if (!res.ok) throw new Error(`${res.status}`)
        const html = await res.text()
        if (cancelled) return

        // Parse the view document.
        const doc = new DOMParser().parseFromString(html, 'text/html')

        // Link stylesheets — inject into host head, tagged so we can clean up.
        doc.querySelectorAll('link[rel="stylesheet"]').forEach((l) => {
          const href = l.getAttribute('href')
          if (!href) return
          const clone = document.createElement('link')
          clone.rel = 'stylesheet'
          clone.href = href
          clone.dataset.shipViewOwned = name
          document.head.appendChild(clone)
          ownedStyleLinks.push(clone)
        })

        // Scoped <style> — inject into host container itself, not head, so
        // browser still applies selectors but we can isolate by wrapping in
        // the .ship-view container class which is the only element the
        // shared view-tokens.css global rules target.
        host.className = 'ship-view flex-1 min-h-0 overflow-auto'
        doc.querySelectorAll('style').forEach((s) => {
          const clone = document.createElement('style')
          clone.textContent = s.textContent
          host.appendChild(clone)
        })

        // Body content.
        host.insertAdjacentHTML('beforeend', doc.body.innerHTML)

        // Scripts — load into document.head (required for module execution).
        doc.querySelectorAll('script').forEach((old) => {
          const s = document.createElement('script')
          if (old.type) s.type = old.type
          const src = old.getAttribute('src')
          if (src) {
            s.src = src
          } else {
            s.textContent = old.textContent
          }
          s.dataset.shipViewOwned = name
          document.head.appendChild(s)
          ownedScripts.push(s)
        })

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
      ownedScripts.forEach((s) => s.remove())
      ownedStyleLinks.forEach((l) => l.remove())
      if (host) {
        host.innerHTML = ''
        host.className = 'flex-1 min-h-0 overflow-auto'
      }
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

  return <div ref={hostRef} className="flex-1 min-h-0 overflow-auto" />
}
