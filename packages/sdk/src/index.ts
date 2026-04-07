// @ship/sdk — framework-agnostic client for the Ship daemon
//
// Usage:
//   import { ShipClient } from '@ship/sdk'
//   const ship = new ShipClient({ token: '...' })
//   const q = ship.query('/runtime/workspaces')
//   q.subscribe(state => console.log(state.data))

export { ShipClient } from './client'
export type {
  ShipClientOptions,
  Capability,
  ShipQuery,
  ShipQueryState,
  ShipEvent,
} from './client'

export type * from './types'
