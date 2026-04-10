// React provider for @ship/sdk's ShipClient.
// Creates one client instance, authenticates with daemon token, provides via context.

import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react'
import { ShipClient } from '@ship/sdk'
import { DAEMON_BASE_URL, setDaemonToken } from '#/lib/daemon-config'

const ShipClientContext = createContext<ShipClient | null>(null)

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
      try {
        const res = await fetch('/api/daemon-token')
        if (res.ok) {
          const { token } = await res.json() as { token: string }
          if (token && !cancelled) {
            client.setToken(token)
            setDaemonToken(token)
          }
        }
      } catch {
        // Token endpoint unavailable — continue without auth
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
