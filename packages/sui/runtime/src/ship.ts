// ship.ts — the framework's host seam.
//
// The SUI runtime is host-agnostic: it names no Ship concepts. `ShipHost` is
// an empty interface that the app (deck / `@ship/sdk-client`) augments via
// declaration merging to the concrete `ShipSDK`:
//
//   declare module '@ship/sui-runtime' {
//     interface ShipHost extends ShipSDK {}
//   }
//
// `ViewScope.ship` / `ResolverContext.ship` are typed as `ShipHost`, so
// authoring stays fully typed while the runtime stays free of any host shape.
// The concrete `ShipSDK` + its data types live in `@ship/sdk-client`
// (`ship-sdk.ts`) — the framework deliberately knows nothing about them.

// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ShipHost {}
