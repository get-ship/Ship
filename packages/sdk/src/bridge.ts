/**
 * Ship Bridge — connects to the local daemon via a hidden iframe.
 *
 * Framework-agnostic. No React, no dependencies. Just postMessage.
 *
 * Usage:
 *   const bridge = new ShipBridge()
 *   await bridge.connect()
 *   const workspaces = await bridge.get('/runtime/workspaces')
 *   await bridge.post('/runtime/jobs', { slug: 'my-job', ... })
 *   bridge.disconnect()
 */

export interface BridgeOptions {
  /** Daemon URL. Default: http://localhost:9315 */
  daemonUrl?: string
  /** Connection timeout in ms. Default: 3000 */
  timeout?: number
}

export interface BridgeResponse<T = unknown> {
  ok: boolean
  status: number
  data: T
  error?: string
}

type PendingRequest = {
  resolve: (value: BridgeResponse) => void
  reject: (reason: Error) => void
  timer: ReturnType<typeof setTimeout>
}

export class ShipBridge {
  private iframe: HTMLIFrameElement | null = null
  private pending = new Map<number, PendingRequest>()
  private seq = 0
  private ready = false
  private readyPromise: Promise<void> | null = null
  private readyResolve: (() => void) | null = null
  private daemonUrl: string
  private timeout: number
  private listener: ((e: MessageEvent) => void) | null = null

  constructor(options?: BridgeOptions) {
    this.daemonUrl = options?.daemonUrl ?? 'http://localhost:9315'
    this.timeout = options?.timeout ?? 3000
  }

  /** Connect to the daemon by creating the bridge iframe. */
  async connect(): Promise<boolean> {
    if (this.ready) return true
    if (this.readyPromise) return this.readyPromise.then(() => true)

    this.readyPromise = new Promise<void>((resolve, reject) => {
      this.readyResolve = resolve

      // Listen for messages from the bridge iframe
      this.listener = (e: MessageEvent) => {
        if (e.origin !== this.daemonUrl) return
        const msg = e.data
        if (!msg || !msg.__ship_bridge) return

        if (msg.type === 'ready') {
          this.ready = true
          this.readyResolve?.()
          return
        }

        if (msg.type === 'response' && typeof msg.seq === 'number') {
          const pending = this.pending.get(msg.seq)
          if (pending) {
            this.pending.delete(msg.seq)
            clearTimeout(pending.timer)
            pending.resolve({
              ok: msg.ok,
              status: msg.status,
              data: msg.data,
              error: msg.error,
            })
          }
        }
      }
      window.addEventListener('message', this.listener)

      // Create hidden iframe
      this.iframe = document.createElement('iframe')
      this.iframe.src = `${this.daemonUrl}/bridge`
      this.iframe.style.display = 'none'
      this.iframe.setAttribute('sandbox', 'allow-scripts allow-same-origin')
      document.body.appendChild(this.iframe)

      // Timeout
      setTimeout(() => {
        if (!this.ready) {
          reject(new Error('Bridge connection timeout'))
          this.disconnect()
        }
      }, this.timeout)
    })

    try {
      await this.readyPromise
      return true
    } catch {
      return false
    }
  }

  /** Check if the bridge is connected. */
  get connected(): boolean {
    return this.ready
  }

  /** Disconnect and clean up. */
  disconnect(): void {
    this.ready = false
    this.readyPromise = null
    this.readyResolve = null
    if (this.listener) {
      window.removeEventListener('message', this.listener)
      this.listener = null
    }
    if (this.iframe) {
      this.iframe.remove()
      this.iframe = null
    }
    for (const [, pending] of this.pending) {
      clearTimeout(pending.timer)
      pending.reject(new Error('Bridge disconnected'))
    }
    this.pending.clear()
  }

  /** Send a request through the bridge. */
  private request<T = unknown>(
    method: string,
    path: string,
    body?: unknown,
  ): Promise<BridgeResponse<T>> {
    if (!this.ready || !this.iframe?.contentWindow) {
      return Promise.reject(new Error('Bridge not connected'))
    }

    const seq = ++this.seq

    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(seq)
        reject(new Error(`Bridge request timeout: ${method} ${path}`))
      }, this.timeout)

      this.pending.set(seq, {
        resolve: resolve as (v: BridgeResponse) => void,
        reject,
        timer,
      })

      this.iframe!.contentWindow!.postMessage(
        {
          __ship_bridge: true,
          type: 'request',
          seq,
          method,
          path,
          body,
        },
        this.daemonUrl,
      )
    })
  }

  /** GET request. */
  get<T = unknown>(path: string): Promise<BridgeResponse<T>> {
    return this.request<T>('GET', path)
  }

  /** POST request. */
  post<T = unknown>(path: string, body?: unknown): Promise<BridgeResponse<T>> {
    return this.request<T>('POST', path, body)
  }

  /** PUT request. */
  put<T = unknown>(path: string, body?: unknown): Promise<BridgeResponse<T>> {
    return this.request<T>('PUT', path, body)
  }

  /** DELETE request. */
  delete<T = unknown>(path: string): Promise<BridgeResponse<T>> {
    return this.request<T>('DELETE', path)
  }
}
