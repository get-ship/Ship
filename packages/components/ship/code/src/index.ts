// @ship/code — Ship Code pack (ship-* tags for code surfaces).
//
// Import .sail so a file-linked consume compiles the tags.
//
//   - <ship-code-editor>  CodeMirror 6 wrapped as a lazy-loaded island.
//   - <ship-terminal>     stub until xterm lands on current Sail.
//   - <ship-git-diff>     unified-diff tree (+ file / hunk / line children).
import './ship-code-editor/ship-code-editor.compiled.js'
import './ship-terminal/ship-terminal.compiled.js'
import './ship-git-diff/ship-git-diff.compiled.js'
import './ship-git-diff/ship-git-diff-file.compiled.js'
import './ship-git-diff/ship-git-diff-hunk.compiled.js'
import './ship-git-diff/ship-git-diff-line.compiled.js'
