// Test environment polyfills for the @ship/code pack.
//
// <ship-terminal> wraps xterm.js, which under jsdom calls `matchMedia`,
// canvas `getContext`, and throws "WebGL2 not supported" inside its
// `requestAnimationFrame` open() path. `initJsdomPolyfills` stubs those
// minimal browser APIs; `suppressRafErrors` swallows the third-party
// rAF init throw so the smoke tests assert structure without a real
// layout/WebGL engine. <ship-code-editor> (CodeMirror) and the
// <ship-git-diff> tree also rely on ResizeObserver, which the same
// helper installs.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills({ suppressRafErrors: true })
