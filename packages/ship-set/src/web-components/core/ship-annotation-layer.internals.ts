/**
 * Internals for `<ship-annotation-layer>`: shadow-DOM styles plus the
 * popover template. Split out to keep the main element file under the
 * 300-line cap.
 */

import { css, html, type TemplateResult } from 'lit'

export const layerStyles = css`
  :host {
    position: relative;
    display: block;
    font-family: var(--font-body);
  }
  .surface { position: relative; width: 100%; height: 100%; }
  .overlay { position: absolute; inset: 0; z-index: 10; }
  :host([mode='point']) .overlay,
  :host([mode='region']) .overlay { cursor: crosshair; }
  :host(:not([mode='point']):not([mode='region'])) .overlay { pointer-events: none; }

  .drag-rect {
    position: absolute;
    border: 2px dashed var(--primary);
    background: var(--tone-primary-bg);
    pointer-events: none;
    border-radius: var(--radius-sm);
  }
  .popover {
    position: absolute;
    z-index: 20;
    min-width: 220px;
    padding: 8px;
    background: var(--popover, var(--background));
    color: var(--popover-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .popover textarea {
    font: inherit;
    width: 100%;
    min-height: 60px;
    resize: vertical;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
    color: var(--foreground);
    box-sizing: border-box;
  }
  .popover-actions { display: flex; justify-content: flex-end; gap: 6px; }
  .popover button {
    font: inherit;
    font-size: var(--text-sm);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    cursor: pointer;
  }
  .popover button[data-primary] {
    background: var(--primary);
    color: var(--primary-foreground);
    border-color: var(--primary);
  }
`

export interface PopoverHost {
  confirmNote(value: string): void
  cancelNote(): void
}

export function renderPopover(
  host: PopoverHost,
  at: { x: number; y: number },
): TemplateResult {
  const left = Math.min(at.x + 12, 10_000)
  const top = Math.min(at.y + 12, 10_000)
  return html`
    <div
      class="popover"
      data-annotation-popover
      style=${`left:${left}px;top:${top}px`}
      @pointerdown=${(e: Event) => e.stopPropagation()}
    >
      <textarea
        part="note-input"
        placeholder="Add a note…"
        @keydown=${(e: KeyboardEvent) => {
          if (e.key === 'Escape') host.cancelNote()
          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
            host.confirmNote((e.currentTarget as HTMLTextAreaElement).value)
          }
        }}
      ></textarea>
      <div class="popover-actions">
        <button type="button" @click=${() => host.cancelNote()}>Cancel</button>
        <button
          type="button"
          data-primary
          @click=${(e: Event) => {
            const ta = (e.currentTarget as HTMLElement)
              .closest('.popover')!
              .querySelector('textarea') as HTMLTextAreaElement
            host.confirmNote(ta.value)
          }}
        >Send</button>
      </div>
    </div>
  `
}
