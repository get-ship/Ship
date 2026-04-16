/**
 * Selection-bubble and comment-composer styles for <ship-markdown-editor>.
 *
 * Kept separate from the main stylesheet so both files stay well under the
 * 300-line cap. The component composes both in its `static styles` array.
 *
 * The bubble lives in the component's shadow DOM (see `.bubble` anchor on
 * the shell). Positioning is set inline by the component from ProseMirror
 * selection coordinates, relative to the shell box.
 */

import { css } from 'lit'

export const shipMarkdownEditorBubbleStyles = css`
  .bubble {
    position: absolute;
    z-index: 40;
    display: inline-flex;
    align-items: center;
    gap: 0;
    padding: 2px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md, 0 4px 12px rgba(0, 0, 0, 0.14));
    transform: translate(-50%, -8px);
    opacity: 0;
    pointer-events: none;
    transform-origin: 50% 100%;
    transition:
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .bubble[data-open='true'] {
    opacity: 1;
    pointer-events: auto;
    transform: translate(-50%, -12px);
  }
  .bubble button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-family: var(--font-body);
    font-size: 12px;
    line-height: 1;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .bubble button:hover:not(:disabled) {
    background: color-mix(in oklch, var(--muted) 80%, transparent);
    color: var(--foreground);
  }
  .bubble button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    color: var(--foreground);
  }
  .bubble button.is-active {
    background: color-mix(in oklch, var(--primary) 18%, var(--muted));
    color: var(--foreground);
  }
  .bubble .sep {
    width: 1px;
    height: 18px;
    background: var(--border);
    margin: 0 2px;
    flex-shrink: 0;
  }
  .bubble svg { width: 14px; height: 14px; }
  .bubble .glyph-bold   { font-weight: 800; font-size: 13px; }
  .bubble .glyph-italic { font-weight: 600; font-size: 13px; font-style: italic; }
  .bubble .glyph-strike { font-weight: 600; font-size: 13px; text-decoration: line-through; }
  .bubble .glyph-code   { font-weight: 600; font-size: 11px; letter-spacing: -0.03em; }
  @media (prefers-reduced-motion: reduce) {
    .bubble { transition-duration: 1ms; }
  }

  /* Inline comment composer */
  .composer {
    position: absolute;
    z-index: 50;
    width: 320px;
    max-width: calc(100% - 16px);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg, 0 12px 32px rgba(0, 0, 0, 0.22));
    overflow: hidden;
    transform: translate(-50%, 0);
    transform-origin: 50% 0;
    animation: composer-in var(--duration-fast) var(--ease-out);
  }
  @keyframes composer-in {
    from { opacity: 0; transform: translate(-50%, -4px); }
    to   { opacity: 1; transform: translate(-50%, 0); }
  }
  .composer-quote {
    padding: 8px 12px;
    font-size: 11px;
    font-style: italic;
    color: var(--muted-foreground);
    border-bottom: 1px solid var(--border);
    max-height: 60px;
    overflow: hidden;
    line-height: 1.4;
    border-left: 3px solid color-mix(in oklch, var(--primary) 40%, var(--border));
    background: color-mix(in oklch, var(--muted) 35%, transparent);
  }
  .composer textarea {
    display: block;
    width: 100%;
    padding: 10px 12px;
    border: 0;
    background: transparent;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: 13px;
    line-height: 1.5;
    resize: none;
    outline: none;
    min-height: 52px;
  }
  .composer textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 0.6;
  }
  .composer-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    padding: 6px 8px;
    border-top: 1px solid var(--border);
    background: color-mix(in oklch, var(--muted) 30%, transparent);
  }
  .composer-actions button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 0;
    border-radius: var(--radius-sm);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  .composer-actions .cancel {
    background: transparent;
    color: var(--muted-foreground);
  }
  .composer-actions .cancel:hover { color: var(--foreground); }
  .composer-actions .submit {
    background: var(--primary);
    color: var(--primary-foreground);
  }
  .composer-actions .submit:hover:not(:disabled) { opacity: 0.92; }
  .composer-actions .submit:disabled { opacity: 0.4; cursor: default; }
  .composer-actions kbd {
    font-family: var(--font-body);
    font-size: 9px;
    opacity: 0.75;
  }

  @media (forced-colors: active) {
    .bubble, .composer { border: 1px solid CanvasText; }
  }
`
