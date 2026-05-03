// Mount a SUI view into a shell-owned slot.
//
// Reads `mount` off `window.__suiRuntime` — set by index.html's
// importmap-resolved bootstrap script. This sidesteps Vite's
// import-analysis, which would otherwise rewrite a `import()` URL with
// a `?import` query and create a SECOND copy of the runtime module —
// distinct from the one compiled view modules import. Reactivity
// breaks across that boundary.
//
// Returns the unmount disposer so the shell's view-swap code can tear
// down the previous view before the next one mounts.

interface MountFn {
  (
    target: HTMLElement,
    payload: unknown,
    opts?: { cssScope?: 'adjacent' | 'document-head' | 'none' },
  ): Promise<{ dispose(): void }>
}

interface SuiRuntimeGlobal {
  mount: MountFn
}

async function loadMount(): Promise<MountFn> {
  const w = window as unknown as {
    __suiRuntime?: SuiRuntimeGlobal
    __suiRuntimeReady?: Promise<SuiRuntimeGlobal>
  }
  if (w.__suiRuntime) return w.__suiRuntime.mount
  if (w.__suiRuntimeReady) {
    const rt = await w.__suiRuntimeReady
    return rt.mount
  }
  throw new Error(
    '[shell/mount-sui] window.__suiRuntimeReady missing — index.html bootstrap script did not run.',
  )
}

export async function mountSuiView(
  slot: HTMLElement,
  name: string,
  subPath?: string,
): Promise<() => void> {
  const path = subPath ? `${name}/${subPath}` : name
  const url = `/_sui/views/${path}`
  const res = await fetch(url, { credentials: 'same-origin' })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(`mountSuiView(${path}) ${res.status}: ${body}`)
  }
  const payload = await res.json()
  const mount = await loadMount()
  const handle = await mount(slot, payload, { cssScope: 'adjacent' })
  return () => handle.dispose()
}
