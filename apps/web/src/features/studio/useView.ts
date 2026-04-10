import { usePlatform } from './PlatformProvider'
import type { ShipClient } from '@ship/sdk'

interface ViewContext {
  daemon: 'connected' | 'disconnected' | 'connecting'
  auth: 'anonymous' | 'authenticated'
  workspace: { branch: string } | null
  client: ShipClient
  requireDaemon: () => ShipClient | null
}

export function useView(): ViewContext {
  const platform = usePlatform()
  return {
    daemon: platform.daemon,
    auth: platform.auth,
    workspace: platform.workspace,
    client: platform.client,
    requireDaemon: () =>
      platform.daemon === 'connected' ? platform.client : null,
  }
}
