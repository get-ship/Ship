//! Phase 9 — TypeScript declaration emission.
//!
//! Generates a `.d.ts` that lets TS consumers see the custom element's
//! prop types and event payloads:
//!
//! ```ts
//! declare module '@sail/components' {
//!   export interface MyComponent extends HTMLElement {
//!     prop1: string
//!     prop2: number
//!     addEventListener(
//!       type: 'select',
//!       listener: (e: CustomEvent<{ id: string }>) => void
//!     ): void
//!   }
//! }
//!
//! declare global {
//!   interface HTMLElementTagNameMap {
//!     'my-component': MyComponent
//!   }
//! }
//! ```
//!
//! Used by JSX consumers and direct DOM API users for type-checked
//! tag references.
