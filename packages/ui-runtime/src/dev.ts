// `@ship/ui-runtime/dev` — dev-only entry point.
//
// Imports under this subpath stay out of the production runtime
// bundle (sui-runtime.js externalizes only `@ship/ui-runtime`, so
// `/dev` references would error if shipped). The shell + tests
// import from here when they want the leak detector wired in.

export { enableLeakWarn, enableLeakWarnInScope } from './scope-resource-warn'
