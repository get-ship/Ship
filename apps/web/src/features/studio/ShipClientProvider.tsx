// React provider for @ship/sdk's ShipClient.
// Creates one client instance, connects events, provides via context.

import { createContext, useContext, useEffect, useRef, type ReactNode } from 'react'
import { ShipClient } from '@ship/sdk'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'

const ShipClientContext = createContext<ShipClient | null>(null)

export function ShipClientProvider({ children }: { children: ReactNode }) {
  const clientRef = useRef<ShipClient | null>(null)

  if (!clientRef.current) {
    clientRef.current = new ShipClient({
      baseUrl: DAEMON_BASE_URL,
      capabilities: ['*'],
      staleTime: 5000,
    })
  }

  useEffect(() => {
    const client = clientRef.current!
    client.connectEvents()
    return () => { client.destroy() }
  }, [])

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
