// @ship/code — Ship Code pack (ship-* tags for code surfaces).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/code'` registers every tag in the pack globally.
//
// Three groups:
//   - <ship-code-editor>  CodeMirror 6 wrapped as a lazy-loaded island.
//   - <ship-terminal>     xterm.js PTY rendering primitive.
//   - <ship-git-diff>     unified-diff tree (+ -file / -hunk / -line children).
//
// Pure presentational tags: no `ship` Proxy, no SDK reads. Consumers bind
// their own data + transport.
import './ship-code-editor/ship-code-editor.compiled.js'
import './ship-terminal/ship-terminal.compiled.js'
import './ship-git-diff/ship-git-diff.compiled.js'
import './ship-git-diff/ship-git-diff-file.compiled.js'
import './ship-git-diff/ship-git-diff-hunk.compiled.js'
import './ship-git-diff/ship-git-diff-line.compiled.js'
