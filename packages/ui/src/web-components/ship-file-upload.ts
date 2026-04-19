/**
 * <ship-file-upload>
 *
 * Drag-and-drop file upload zone with removable file list.
 * Attrs: accept, multiple, max-size (bytes, default 10 MB), disabled, tone.
 * Emits `ship-change` with `{ files: File[] }`.
 */

import { LitElement, html, css, nothing } from 'lit'
import { toneVars, reducedMotion } from './util'
import type { Tone } from './util'

function fmtSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

export class ShipFileUpload extends LitElement {
  static styles = [
    toneVars,
    reducedMotion,
    css`
      :host { display: block; }
      :host([disabled]) { opacity: 0.5; pointer-events: none; }

      .dropzone {
        display:flex;flex-direction:column;align-items:center;justify-content:center;
        gap:8px;padding:24px 16px;border:2px dashed var(--border);
        border-radius:var(--radius);background:transparent;cursor:pointer;
        transition:border-color .15s,background .15s;
        color:var(--muted-foreground);font-size:.875rem;
      }
      .dropzone:focus-visible { outline:2px solid var(--ring);outline-offset:2px; }
      .dropzone.over { border-style:solid;border-color:var(--tone-border);background:var(--tone-bg); }
      .icon { font-size:1.5rem;line-height:1; }
      input[type="file"] { display:none; }
      .file-list { display:flex;flex-direction:column;gap:4px;margin-top:8px; }
      .file-item {
        display:flex;align-items:center;gap:8px;padding:6px 10px;
        border-radius:var(--radius-sm);background:var(--muted);
        font-size:.8125rem;color:var(--foreground);
      }
      .file-name { flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
      .file-size { color:var(--muted-foreground);flex-shrink:0; }
      .remove {
        appearance:none;border:none;background:none;cursor:pointer;
        color:var(--muted-foreground);padding:2px;line-height:1;
        border-radius:var(--radius-sm);font-size:1rem;
      }
      .remove:hover { color:var(--foreground);background:var(--border); }
      .error { font-size:.75rem;color:var(--destructive);margin-top:4px;padding:0 2px; }
    `,
  ]

  static properties = {
    accept: { type: String },
    multiple: { type: Boolean },
    maxSize: { type: Number, attribute: 'max-size' },
    disabled: { type: Boolean, reflect: true },
    tone: { type: String, reflect: true },
    _files: { state: true },
    _over: { state: true },
    _error: { state: true },
  }

  accept = ''
  multiple = false
  maxSize = 10485760
  disabled = false
  tone?: Tone
  private _files: File[] = []
  private _over = false
  private _error = ''

  private get _inputEl(): HTMLInputElement | null {
    return this.renderRoot.querySelector('input[type="file"]')
  }

  private _open() {
    if (this.disabled) return
    this._inputEl?.click()
  }

  private _onKey(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this._open() }
  }

  private _onDragOver(e: DragEvent) { e.preventDefault(); this._over = true }
  private _onDragLeave() { this._over = false }

  private _onDrop(e: DragEvent) {
    e.preventDefault(); this._over = false
    if (e.dataTransfer?.files) this._addFiles(Array.from(e.dataTransfer.files))
  }

  private _onInputChange(e: Event) {
    const input = e.target as HTMLInputElement
    if (input.files) this._addFiles(Array.from(input.files))
    input.value = ''
  }

  private _addFiles(incoming: File[]) {
    this._error = ''
    const valid: File[] = []
    for (const f of incoming) {
      if (f.size > this.maxSize) {
        this._error = `"${f.name}" exceeds max size (${fmtSize(this.maxSize)}).`
        continue
      }
      if (this.accept && !this._matchesAccept(f)) {
        this._error = `"${f.name}" is not an accepted file type.`
        continue
      }
      valid.push(f)
    }
    this._files = this.multiple ? [...this._files, ...valid] : valid.slice(0, 1)
    this._emit()
  }

  private _matchesAccept(file: File): boolean {
    const tokens = this.accept.split(',').map((t) => t.trim().toLowerCase())
    const ext = '.' + file.name.split('.').pop()?.toLowerCase()
    return tokens.some((t) => {
      if (t.startsWith('.')) return ext === t
      if (t.endsWith('/*')) return file.type.startsWith(t.replace('/*', '/'))
      return file.type === t
    })
  }

  private _remove(idx: number) {
    this._files = this._files.filter((_, i) => i !== idx)
    this._error = ''
    this._emit()
  }

  private _emit() {
    this.dispatchEvent(new CustomEvent('ship-change', {
      detail: { files: [...this._files] }, bubbles: true, composed: true,
    }))
  }

  render() {
    return html`
      <div
        class="dropzone ${this._over ? 'over' : ''}"
        role="button"
        tabindex=${this.disabled ? '-1' : '0'}
        aria-label=${this.accept
          ? `Upload files (${this.accept})`
          : 'Upload files'}
        @click=${this._open}
        @keydown=${this._onKey}
        @dragover=${this._onDragOver}
        @dragleave=${this._onDragLeave}
        @drop=${this._onDrop}
      >
        <span class="icon" aria-hidden="true">\u{1F4C4}</span>
        <span>Drop files here or click to browse</span>
      </div>
      <input
        type="file"
        accept=${this.accept || nothing}
        ?multiple=${this.multiple}
        @change=${this._onInputChange}
      />
      ${this._error ? html`<div class="error" role="alert">${this._error}</div>` : nothing}
      ${this._files.length
        ? html`<div class="file-list" aria-live="polite">
            ${this._files.map(
              (f, i) => html`
                <div class="file-item">
                  <span class="file-name">${f.name}</span>
                  <span class="file-size">${fmtSize(f.size)}</span>
                  <button
                    class="remove"
                    aria-label="Remove ${f.name}"
                    @click=${() => this._remove(i)}
                  >&times;</button>
                </div>`,
            )}
          </div>`
        : nothing}
    `
  }
}

if (typeof customElements !== 'undefined' && !customElements.get('ship-file-upload')) {
  customElements.define('ship-file-upload', ShipFileUpload)
}

declare global {
  interface HTMLElementTagNameMap {
    'ship-file-upload': ShipFileUpload
  }
}
