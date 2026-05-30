// @sui/terminal — PTY rendering primitive.
//
// Pure `sui-*` tag: no `ship` Proxy, no SDK reads. Consumers bind their
// own PTY transport and data. A single `import '@sui/terminal'` registers
// <sui-terminal> globally — the compiled tag calls customElements.define()
// on import.
import './sui-terminal/sui-terminal.compiled.js'
