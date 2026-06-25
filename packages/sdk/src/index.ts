// @ship/sdk — internal Connect transport for the Ship daemon op surface.
//
// NOT a public surface. Callers go through `@ship/sdk-client`, which owns the
// browser-side SDK handle (`createClient`, `ship`, `setShipClient`) and
// re-exports the transport symbols below so consumers needing the raw
// Connect client (custom baseUrl, server-streaming reads, interceptors)
// don't have to reach across the package boundary.
//
// The op surface is generated from `proto/ship/ops/v1/*.proto` by buf
// (see buf.gen.sdk.yaml). Per-domain Connect clients are the public
// wire — `createShipConnectClient({ baseUrl })` returns the typed
// `ShipConnectClient` that the SDK handle uses internally.

export {
  createShipConnectClient,
  resolveDaemonBase,
} from './connect'
export type { ShipConnectClient, ShipConnectOptions } from './connect'
export { SettingScope } from './connect'
