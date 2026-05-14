/**
 * ProseMirror content styles for <sui-markdown-editor>.
 * Adopted into the shadow root by the controller alongside the bubble styles.
 * Kept in a separate module so the controller stays under 300 lines.
 */

const MONO = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace"

export const EDITOR_CONTENT_CSS = `
.ProseMirror {
  padding: var(--ed-pad-y, 1rem) var(--ed-pad-x, 1.25rem) calc(var(--ed-pad-y, 1rem) + 0.75rem);
  font-family: var(--font-body);
  font-size: var(--ed-fs, 0.95rem);
  line-height: var(--ed-lh, 1.7);
  color: var(--foreground);
  outline: none;
  min-height: 100%;
  caret-color: var(--foreground);
}
.ProseMirror > *:first-child { margin-top: 0 !important; }
.ProseMirror > *:last-child { margin-bottom: 0 !important; }
.ProseMirror ::selection, .ProseMirror *::selection {
  background: color-mix(in oklch, var(--tone, var(--ring-color)) 28%, transparent);
  color: inherit;
}
.ProseMirror h1 {
  font-family: var(--heading-font);
  font-size: 1.6rem; font-weight: 700;
  margin: 1.5rem 0 0.5rem; line-height: 1.25;
  letter-spacing: -0.015em;
}
.ProseMirror h2 {
  font-family: var(--heading-font);
  font-size: 1.3rem; font-weight: 650;
  margin: 1.25rem 0 0.4rem; line-height: 1.3;
  letter-spacing: -0.01em;
}
.ProseMirror h3 { font-size: 1.1rem; font-weight: 600; margin: 1rem 0 0.35rem; line-height: 1.35; }
.ProseMirror h4, .ProseMirror h5, .ProseMirror h6 { font-size: 1rem; font-weight: 600; margin: 0.75rem 0 0.25rem; }
.ProseMirror p { margin: 0 0 0.65rem; }
.ProseMirror ul { list-style: disc; padding-left: 1.4rem; margin: 0 0 0.65rem; }
.ProseMirror ol { list-style: decimal; padding-left: 1.4rem; margin: 0 0 0.65rem; }
.ProseMirror li { margin: 0.15rem 0; }
.ProseMirror li p { margin: 0; }
.ProseMirror blockquote {
  border-left: 3px solid var(--border); color: var(--muted-foreground);
  padding-left: 1rem; margin: 0.75rem 0;
}
.ProseMirror hr { border: 0; border-top: 1px solid var(--border); margin: 1.5rem 0; }
.ProseMirror strong { font-weight: var(--font-weight-semibold); }
.ProseMirror em { font-style: italic; }
.ProseMirror code {
  font-family: ${MONO};
  font-size: 0.85em;
  background: var(--muted);
  padding: 0.15em 0.35em;
  border-radius: var(--radius-sm);
}
.ProseMirror a {
  color: var(--primary); text-decoration: underline;
  text-underline-offset: 3px; text-decoration-thickness: 1px; cursor: pointer;
}
.ProseMirror pre {
  background: var(--muted); border: 1px solid var(--border);
  border-radius: var(--radius-md); padding: 0.75rem 1rem;
  margin: 0.75rem 0; overflow-x: auto; font-family: ${MONO};
}
.ProseMirror pre code { background: transparent; padding: 0; font-size: 0.85rem; line-height: 1.6; color: var(--foreground); }
.ProseMirror pre .hljs-comment, .ProseMirror pre .hljs-quote { color: var(--muted-foreground); font-style: italic; }
.ProseMirror pre .hljs-keyword, .ProseMirror pre .hljs-selector-tag, .ProseMirror pre .hljs-built_in { color: var(--status-purple); }
.ProseMirror pre .hljs-string, .ProseMirror pre .hljs-attr { color: var(--status-green); }
.ProseMirror pre .hljs-number, .ProseMirror pre .hljs-literal { color: var(--status-orange); }
.ProseMirror pre .hljs-title, .ProseMirror pre .hljs-function { color: var(--status-blue); }
.ProseMirror table { width: 100%; border-collapse: collapse; margin: 0.75rem 0; table-layout: fixed; }
.ProseMirror th, .ProseMirror td { border: 1px solid var(--border); padding: 0.4rem 0.6rem; text-align: left; vertical-align: top; }
.ProseMirror th { font-weight: 600; background: var(--muted); }
.ProseMirror ul[data-type='taskList'] { list-style: none; padding-left: 0; }
.ProseMirror ul[data-type='taskList'] li { display: flex; align-items: flex-start; gap: 0.4rem; }
.ProseMirror ul[data-type='taskList'] li > label { margin-top: 0.25rem; flex-shrink: 0; }
.ProseMirror ul[data-type='taskList'] li > label input[type='checkbox'] { accent-color: var(--primary); cursor: pointer; }
.ProseMirror ul[data-type='taskList'] li[data-checked='true'] > div > p { text-decoration: line-through; opacity: 0.6; }
.ProseMirror img { max-width: 100%; height: auto; border-radius: var(--radius-md); }
.ProseMirror p.is-editor-empty:first-child::before {
  content: attr(data-placeholder); float: left;
  color: var(--muted-foreground); opacity: 0.55; pointer-events: none; height: 0;
}
@media (forced-colors: active) {
  .ProseMirror a { text-decoration-color: LinkText; }
  .ProseMirror pre { border: 1px solid CanvasText; }
}
`

export function injectEditorStyles(shadow: ShadowRoot): void {
  try {
    const sheet = new CSSStyleSheet()
    sheet.replaceSync(EDITOR_CONTENT_CSS)
    shadow.adoptedStyleSheets = [...shadow.adoptedStyleSheets, sheet]
  } catch { /* CSSStyleSheet not available in JSDOM */ }
}
