// Setup for @ship/test-setup's own tests + the behavior-grade generated tests.
// initJsdomPolyfills() makes jsdom tolerate the modern CSS components adopt
// (constructable stylesheets / tokens) — without it, mounting a real component
// throws "Could not parse CSS stylesheet".
import { initJsdomPolyfills } from '../src/jsdom-polyfills'

initJsdomPolyfills()
