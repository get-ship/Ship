// @ship/sdk — framework-agnostic client for the Ship daemon
//
// For Studio (same-origin): direct fetch, full access
// For extensions (iframe): postMessage bridge, scoped access
// For third-party apps: direct fetch with registered token
//
// Usage:
//   import { ShipClient } from '@ship/sdk'
//   const ship = new ShipClient({ token: '...' })
//   const ws = ship.workspaces()
//   ws.subscribe(state => console.log(state.data))

export { ShipClient } from './client'
export type {
  ShipClientOptions,
  Capability,
  QueryState,
  QueryStatus,
  ShipEvent,
} from './client'

export { ShipBridge } from './bridge'
export type { BridgeOptions, BridgeResponse } from './bridge'

export type * from './types'
