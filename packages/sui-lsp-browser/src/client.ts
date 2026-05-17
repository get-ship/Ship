// Main-thread Promise API over the worker. Use from any editor:
//
//   import { createSuiLspClient } from '@ship/sui-lsp-browser'
//   const lsp = createSuiLspClient(new Worker('/sui-lsp.worker.js', { type: 'module' }))
//   const diags = await lsp.diagnose(text)
//   const completions = await lsp.completionsAt(text, cursorOffset)
//
// One worker per editor instance is fine — the worker is stateless,
// requests are matched by an incrementing id.

import type { SuiDiagnostic } from './diagnostics'
import type { SuiCompletionSource } from './completions'

export type { SuiDiagnostic } from './diagnostics'
export type { SuiCompletionSource } from './completions'

interface Pending {
  resolve: (value: unknown) => void
  reject: (err: Error) => void
}

interface OkResponse<T> { id: number; ok: true; data: T }
interface ErrResponse    { id: number; ok: false; error: string }
type Response<T> = OkResponse<T> | ErrResponse

export interface SuiLspClient {
  diagnose(text: string): Promise<SuiDiagnostic[]>
  completionsAt(text: string, offset: number): Promise<SuiCompletionSource | null>
  allSources(): Promise<SuiCompletionSource[]>
  dispose(): void
}

export function createSuiLspClient(worker: Worker): SuiLspClient {
  let nextId = 1
  const pending = new Map<number, Pending>()

  worker.addEventListener('message', (event: MessageEvent<Response<unknown>>) => {
    const res = event.data
    const p = pending.get(res.id)
    if (!p) return
    pending.delete(res.id)
    if (res.ok) p.resolve(res.data)
    else p.reject(new Error(res.error))
  })

  function call<T>(payload: { kind: string; [k: string]: unknown }): Promise<T> {
    const id = nextId++
    return new Promise<T>((resolve, reject) => {
      pending.set(id, { resolve: resolve as (v: unknown) => void, reject })
      worker.postMessage({ id, ...payload })
    })
  }

  return {
    diagnose: (text) => call<SuiDiagnostic[]>({ kind: 'diagnose', text }),
    completionsAt: (text, offset) => call<SuiCompletionSource | null>({ kind: 'completeAt', text, offset }),
    allSources: () => call<SuiCompletionSource[]>({ kind: 'allSources' }),
    dispose: () => {
      worker.terminate()
      for (const p of pending.values()) p.reject(new Error('worker disposed'))
      pending.clear()
    },
  }
}
