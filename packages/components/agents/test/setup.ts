// Vitest setup for `@ship/agents` — JSDOM polyfills only.
//
// <ship-agent-thread> composes pure `@sui/chat-elements` primitives and
// owns its own state; the tested surface touches no SDK client. The
// shared polyfills (ResizeObserver, matchMedia, Popover, Dialog,
// ToggleEvent, PointerEvent) cover what the composed components use.
import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()
