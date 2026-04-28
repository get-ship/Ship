// Vitest setup — polyfills for jsdom gaps that ship-set-sail tests
// rely on but jsdom doesn't ship.

// jsdom doesn't implement DragEvent or DataTransfer. The drag/drop +
// file-input tests for ship-file-upload need both. Polyfill as thin
// shapes — enough surface for `setData`/`getData`/`files` access plus
// a DragEvent that carries `dataTransfer` on the instance.

if (typeof globalThis.DataTransfer === 'undefined') {
  class DataTransferPolyfill {
    public dropEffect: string = 'none'
    public effectAllowed: string = 'all'
    public readonly items: { add: (f: File) => void }
    public readonly types: string[] = []
    private _files: File[] = []
    private _store = new Map<string, string>()

    constructor() {
      const self = this
      this.items = { add: (f: File) => { self._files.push(f) } }
    }

    get files(): FileList {
      const arr = this._files
      const out = arr as unknown as FileList & { item: (i: number) => File | null }
      out.item = (i: number) => arr[i] ?? null
      return out
    }
    setData(format: string, data: string): void { this._store.set(format, data) }
    getData(format: string): string { return this._store.get(format) ?? '' }
    clearData(): void { this._store.clear(); this._files = [] }
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(globalThis as any).DataTransfer = DataTransferPolyfill
}

if (typeof globalThis.DragEvent === 'undefined') {
  class DragEventPolyfill extends Event {
    public readonly dataTransfer: DataTransfer | null
    constructor(type: string, init: EventInit & { dataTransfer?: DataTransfer | null } = {}) {
      super(type, init)
      this.dataTransfer = init.dataTransfer ?? null
    }
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(globalThis as any).DragEvent = DragEventPolyfill
}
