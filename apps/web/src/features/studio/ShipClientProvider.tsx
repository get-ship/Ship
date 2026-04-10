// Compatibility layer — delegates to PlatformProvider.
// TODO: migrate all consumers to usePlatform().client, then delete this file.

import { usePlatform } from './PlatformProvider'
import type { ShipClient } from '@ship/sdk'

export function useShipClient(): ShipClient {
  return usePlatform().client
}

// ShipClientProvider is no longer needed — PlatformProvider handles client lifecycle.
// This export exists only for imports that haven't been migrated yet.
export function ShipClientProvider({ children }: { children: React.ReactNode }) {
  return <>{children}</>
}
