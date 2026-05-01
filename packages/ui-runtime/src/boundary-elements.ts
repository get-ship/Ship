// Register <sui-if>, <sui-for>, <sui-match> as inert custom elements.
//
// These are SSR boundary markers emitted by the Rust walker. Registering them
// ensures consistent parsing and allows stylesheets to target them reliably.
// Each element is an inert HTMLElement — the attach runtime does all the work.

if (typeof customElements !== 'undefined') {
  if (!customElements.get('sui-if')) {
    customElements.define('sui-if', class extends HTMLElement {})
  }
  if (!customElements.get('sui-for')) {
    customElements.define('sui-for', class extends HTMLElement {})
  }
  if (!customElements.get('sui-match')) {
    customElements.define('sui-match', class extends HTMLElement {})
  }
}
