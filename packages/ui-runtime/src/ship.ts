// ship.ts — typed handle for the Ship SDK.
//
// The SDK is injected by the runtime (and the daemon's hydrate flow)
// into:
//   - the `source=` attribute of `<data>` declarations
//   - the function form of a controller (`({ ship }) => ...`)
//   - each controller section's destructure
//   - controller hook factories that accept scope
//
// Authors do not import the runtime value of `ship`; only this type.
//
// TODO: tighten to the generated SDK manifest once the per-project SDK
// type is reachable from this package. Today the type is `unknown`
// so the runtime doesn't depend on the SDK package shape.

/** Live SDK handle exposed to controllers, hooks, and `<data>` sources. */
export type ShipSDK = unknown
