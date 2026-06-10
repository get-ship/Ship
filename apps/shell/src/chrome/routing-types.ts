// Reserved for chrome-side route hooks. The shell routes by raw
// pathname; the runtime's `route()` provides `route.params.<name>()`
// to views. Chrome modules that need to ask "what view is active?"
// use the `getCurrentViewName()` getter main.ts passes to them —
// see `chrome/keymap.ts` for the call sites.
export type CurrentViewName = string
