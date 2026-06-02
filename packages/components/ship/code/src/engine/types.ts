/** Headless code-editor contract — see build-spec Slice 1. */
export interface CodeEditorOptions {
  /** Initial document. */
  doc?: string
  /** Language id (see languages.ts). Unknown id → plain text. */
  language?: string
  /** Mount read-only (non-editable). */
  readonly?: boolean
  /** Show the line-number gutter. Default true. */
  lineNumbers?: boolean
  /** Soft-wrap long lines. Default false. */
  lineWrap?: boolean
  /** Indent width in spaces. Default 2. */
  tabSize?: number
  /** Hint shown when the document is empty. */
  placeholder?: string
  /** Use the dark token palette. Default false (light). */
  dark?: boolean
  /** Fired when the document changes (including setValue). */
  onChange?: (value: string) => void
  /** Fired on the Mod-s keybinding (Save). */
  onSave?: (value: string) => void
}

export interface CodeEditor {
  /** Construct the EditorView into `parent`. */
  mount(parent: HTMLElement): void
  /** Current document text. */
  value(): string
  /** Replace the whole document via a transaction. */
  setValue(v: string): void
  /** Reconfigure the language compartment in place. */
  setLanguage(id: string): void
  /** Reconfigure the readonly/editable compartments in place. */
  setReadonly(ro: boolean): void
  /** Current readonly state. */
  isReadonly(): boolean
  /** Reconfigure the theme compartment (light/dark) in place. */
  setDark(dark: boolean): void
  /** Reconfigure the line-numbers compartment in place. */
  setLineNumbers(on: boolean): void
  /** Reconfigure the line-wrap compartment in place. */
  setLineWrap(on: boolean): void
  /** Reconfigure the empty-doc placeholder compartment in place. */
  setPlaceholder(text: string): void
  /** Set/replace the Mod-s save callback after construction (host wiring). */
  setSave(cb: (value: string) => void): void
  /** Focus the editor. */
  focus(): void
  /** Destroy the EditorView. */
  dispose(): void
}
