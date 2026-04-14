import { DAEMON_BASE_URL } from './daemon-config'

export type ViewMessage = { source: 'ship-view'; type: 'cmdk' }

function isViewMessage(data: unknown): data is ViewMessage {
  if (!data || typeof data !== 'object') return false
  const d = data as Record<string, unknown>
  return d.source === 'ship-view' && typeof d.type === 'string'
}

export function installViewBridge(handlers: {
  onCmdK: () => void
}): () => void {
  const expectedOrigin = new URL(DAEMON_BASE_URL).origin
  function onMessage(e: MessageEvent) {
    if (e.origin !== expectedOrigin) return
    if (!isViewMessage(e.data)) return
    switch (e.data.type) {
      case 'cmdk':
        handlers.onCmdK()
        return
    }
  }
  window.addEventListener('message', onMessage)
  return () => window.removeEventListener('message', onMessage)
}
