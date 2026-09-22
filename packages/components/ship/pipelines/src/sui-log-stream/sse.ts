export type SseHandlers = {
  onOpen: () => void
  onError: () => void
  onChunk: (chunk: string) => void
}

/** Same-origin SSE client. Lives outside the controller so EventSource
 *  is not ambient authority inside the .sail controller region. */
export function openEventSource(
  url: string,
  eventName: string,
  handlers: SseHandlers,
): () => void {
  let source: EventSource | null = null
  try {
    source = new EventSource(url, { withCredentials: true })
  } catch {
    handlers.onError()
    return () => {}
  }
  source.addEventListener(eventName, (ev) => {
    handlers.onChunk((ev as MessageEvent).data ?? '')
  })
  source.onopen = () => handlers.onOpen()
  source.onerror = () => handlers.onError()
  return () => {
    if (source) {
      source.close()
      source = null
    }
  }
}
