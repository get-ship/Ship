// React provider for @ship/sdk's ShipClient.
// Two auth paths:
// 1. Fragment token: ship studio → getship.dev/studio#token=xxx
// 2. Session handshake: authenticated user → daemon returns scoped token

import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react'
import { ShipClient } from '@ship/sdk'
import { DAEMON_BASE_URL, setDaemonToken } from '#/lib/daemon-config'

const ShipClientContext = createContext<ShipClient | null>(null)

/** Read token from URL fragment and clear it from the URL. */
function extractFragmentToken(): string | null {
  if (typeof window === 'undefined') return null
  const hash = window.location.hash
  const match = hash.match(/[#&]token=([^&]+)/)
  if (!match) return null
  const token = decodeURIComponent(match[1])
  // Clear fragment from URL without triggering navigation
  const url = new URL(window.location.href)
  url.hash = ''
  window.history.replaceState(null, '', url.toString())
  return token
}

export function ShipClientProvider({ children }: { children: ReactNode }) {
  const clientRef = useRef<ShipClient | null>(null)
  const [ready, setReady] = useState(false)

  if (!clientRef.current) {
    clientRef.current = new ShipClient({
      baseUrl: DAEMON_BASE_URL,
      capabilities: ['*'],
      staleTime: 5000,
    })
  }

  useEffect(() => {
    const client = clientRef.current!
    let cancelled = false

    async function init() {
      // 1. Check URL fragment (anonymous path: ship studio)
      const fragmentToken = extractFragmentToken()
      if (fragmentToken) {
        client.setToken(fragmentToken)
        setDaemonToken(fragmentToken)
        client.connectEvents()
        if (!cancelled) setReady(true)
        return
      }

      // 2. Try daemon handshake (authenticated path: ship login)
      const authenticated = await client.authenticate()
      if (authenticated) {
        setDaemonToken(client.getToken())
      }

      if (!cancelled) {
        client.connectEvents()
        setReady(true)
      }
    }

    void init()
    return () => {
      cancelled = true
      client.destroy()
    }
  }, [])

  if (!ready) return null

  return (
    <ShipClientContext value={clientRef.current}>
      {children}
    </ShipClientContext>
  )
}

export function useShipClient(): ShipClient {
  const client = useContext(ShipClientContext)
  if (!client) throw new Error('useShipClient must be used within ShipClientProvider')
  return client
}
