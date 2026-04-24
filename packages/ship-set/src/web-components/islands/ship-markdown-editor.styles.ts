/**
 * Styles for <ship-markdown-editor>.
 *
 * Adapted from primitives/src/components/editor/editor.css for shadow DOM.
 * All design tokens pierce via CSS custom properties.
 *
 * Two style channels:
 *   --tone / --ed-ring  — focus ring color (derives from tone axis)
 *   --ed-min-h / --ed-fs / --ed-pad — size axis
 *
 * The ProseMirror `.ProseMirror` class is what TipTap puts on the
 * contenteditable root; we target it directly in shadow DOM.
 */

import { css } from 'lit'
import { MONO, reducedMotion } from '../core/util'

export const shipMarkdownEditorStyles = [reducedMotion, css`
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color);
  }
  :host([hidden]) { display: none; }

  :host(:not([tone])),
  :host([tone='default'])    { --tone: var(--ring-color); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --ed-fs: 0.95rem;
    --ed-lh: 1.7;
    --ed-pad-y: var(--space-4);
    --ed-pad-x: var(--space-5);
    --ed-tb-h: 36px;
    --ed-tb-btn: 28px;
  }
  :host([size='sm']) {
    --ed-fs: var(--text-sm);
    --ed-lh: 1.6;
    --ed-pad-y: var(--space-3);
    --ed-pad-x: var(--space-4);
    --ed-tb-h: 32px;
    --ed-tb-btn: 24px;
  }
  :host([size='lg']) {
    --ed-fs: 1rem;
    --ed-lh: 1.75;
    --ed-pad-y: var(--space-5);
    --ed-pad-x: var(--space-6);
    --ed-tb-h: 40px;
    --ed-tb-btn: 32px;
  }

  :host([full-height]) .shell {
    height: 100%;
    min-height: 0;
  }

  .shell {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--card);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-xs);
    overflow: hidden;
    min-height: var(--ed-min-h, 320px);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  :host(:not([disabled])) .shell:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }
  .shell:focus-within {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }
  :host([invalid]) .shell {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) .shell:focus-within {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }
  :host([disabled]) .shell {
    background: var(--muted);
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Toolbar */
  .toolbar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 2px;
    padding: 4px 6px;
    min-height: var(--ed-tb-h);
    background: color-mix(in oklch, var(--muted) 40%, var(--card));
    border-bottom: 1px solid var(--border);
  }
  .toolbar button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--ed-tb-btn);
    height: var(--ed-tb-btn);
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .toolbar button:hover:not(:disabled) {
    background: color-mix(in oklch, var(--muted) 80%, transparent);
    color: var(--foreground);
  }
  .toolbar button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    color: var(--foreground);
  }
  .toolbar button[aria-pressed='true'],
  .toolbar button.is-active {
    background: color-mix(in oklch, var(--primary) 18%, var(--muted));
    color: var(--foreground);
  }
  .toolbar button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .toolbar svg { width: 14px; height: 14px; }
  .toolbar .divider {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 4px;
    flex-shrink: 0;
  }

  /* Editor surface */
  .surface {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .ProseMirror {
    padding: var(--ed-pad-y) var(--ed-pad-x) calc(var(--ed-pad-y) + var(--space-3));
    font-family: var(--font-body);
    font-size: var(--ed-fs);
    line-height: var(--ed-lh);
    color: var(--foreground);
    outline: none;
    min-height: 100%;
    caret-color: var(--foreground);
  }
  .ProseMirror > *:first-child { margin-top: 0 !important; }
  .ProseMirror > *:last-child { margin-bottom: 0 !important; }

  .ProseMirror ::selection,
  .ProseMirror *::selection {
    background: color-mix(in oklch, var(--tone) 28%, transparent);
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
  .ProseMirror h3 {
    font-size: 1.1rem; font-weight: 600;
    margin: 1rem 0 0.35rem; line-height: 1.35;
  }
  .ProseMirror h4, .ProseMirror h5, .ProseMirror h6 {
    font-size: 1rem; font-weight: 600;
    margin: 0.75rem 0 0.25rem;
  }
  .ProseMirror p { margin: 0 0 0.65rem; }
  .ProseMirror ul { list-style: disc; padding-left: 1.4rem; margin: 0 0 0.65rem; }
  .ProseMirror ol { list-style: decimal; padding-left: 1.4rem; margin: 0 0 0.65rem; }
  .ProseMirror li { margin: 0.15rem 0; }
  .ProseMirror li p { margin: 0; }

  .ProseMirror blockquote {
    border-left: 3px solid var(--border);
    color: var(--muted-foreground);
    padding-left: 1rem;
    margin: 0.75rem 0;
  }
  .ProseMirror hr {
    border: 0;
    border-top: 1px solid var(--border);
    margin: 1.5rem 0;
  }
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
    color: var(--primary);
    text-decoration: underline;
    text-underline-offset: 3px;
    text-decoration-thickness: 1px;
    cursor: pointer;
  }

  .ProseMirror pre {
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 0.75rem 1rem;
    margin: 0.75rem 0;
    overflow-x: auto;
    font-family: ${MONO};
  }
  .ProseMirror pre code {
    background: transparent;
    padding: 0;
    font-size: 0.85rem;
    line-height: 1.6;
    color: var(--foreground);
  }
  .ProseMirror pre .hljs-comment,
  .ProseMirror pre .hljs-quote { color: var(--muted-foreground); font-style: italic; }
  .ProseMirror pre .hljs-keyword,
  .ProseMirror pre .hljs-selector-tag,
  .ProseMirror pre .hljs-built_in { color: var(--status-purple); }
  .ProseMirror pre .hljs-string,
  .ProseMirror pre .hljs-attr { color: var(--status-green); }
  .ProseMirror pre .hljs-number,
  .ProseMirror pre .hljs-literal { color: var(--status-orange); }
  .ProseMirror pre .hljs-title,
  .ProseMirror pre .hljs-function { color: var(--status-blue); }

  .ProseMirror table {
    width: 100%;
    border-collapse: collapse;
    margin: 0.75rem 0;
    table-layout: fixed;
  }
  .ProseMirror th, .ProseMirror td {
    border: 1px solid var(--border);
    padding: 0.4rem 0.6rem;
    text-align: left;
    vertical-align: top;
  }
  .ProseMirror th {
    font-weight: 600;
    background: var(--muted);
  }

  .ProseMirror ul[data-type='taskList'] {
    list-style: none;
    padding-left: 0;
  }
  .ProseMirror ul[data-type='taskList'] li {
    display: flex;
    align-items: flex-start;
    gap: 0.4rem;
  }
  .ProseMirror ul[data-type='taskList'] li > label {
    margin-top: 0.25rem;
    flex-shrink: 0;
  }
  .ProseMirror ul[data-type='taskList'] li > label input[type='checkbox'] {
    accent-color: var(--primary);
    cursor: pointer;
  }
  .ProseMirror ul[data-type='taskList'] li[data-checked='true'] > div > p {
    text-decoration: line-through;
    opacity: 0.6;
  }

  .ProseMirror img {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-md);
  }

  .ProseMirror p.is-editor-empty:first-child::before {
    content: attr(data-placeholder);
    float: left;
    color: var(--muted-foreground);
    opacity: 0.55;
    pointer-events: none;
    height: 0;
  }

  @media (forced-colors: active) {
    .shell:focus-within { outline: 2px solid CanvasText; outline-offset: 2px; }
    .toolbar button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`]
