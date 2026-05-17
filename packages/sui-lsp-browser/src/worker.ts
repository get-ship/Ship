// Web-worker entry. Runs the structural analyser off the main thread
// so heavy parses don't stall the editor. Stateless — every request
// carries its own document text.

import { diagnose, type SuiDiagnostic } from './diagnostics'
import { completionsAt, allSources, type SuiCompletionSource } from './completions'

interface DiagnoseRequest  { id: number; kind: 'diagnose'; text: string }
interface CompleteAtRequest { id: number; kind: 'completeAt'; text: string; offset: number }
interface AllSourcesRequest { id: number; kind: 'allSources' }
type Request = DiagnoseRequest | CompleteAtRequest | AllSourcesRequest

interface OkResponse<T> { id: number; ok: true; data: T }
interface ErrResponse    { id: number; ok: false; error: string }
type Response<T> = OkResponse<T> | ErrResponse

function reply<T>(id: number, data: T): void {
  postMessage({ id, ok: true, data } satisfies Response<T>)
}
function fail(id: number, err: unknown): void {
  postMessage({ id, ok: false, error: err instanceof Error ? err.message : String(err) } satisfies Response<unknown>)
}

addEventListener('message', (event: MessageEvent<Request>) => {
  const req = event.data
  try {
    switch (req.kind) {
      case 'diagnose': {
        const diags = diagnose(req.text)
        reply<SuiDiagnostic[]>(req.id, diags)
        break
      }
      case 'completeAt': {
        const src = completionsAt(req.text, req.offset)
        reply<SuiCompletionSource | null>(req.id, src)
        break
      }
      case 'allSources': {
        reply<SuiCompletionSource[]>(req.id, allSources())
        break
      }
    }
  } catch (e) {
    fail(req.id, e)
  }
})
