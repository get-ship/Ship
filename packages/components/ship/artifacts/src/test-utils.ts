// Test helpers for pack component tests.
//
// Each component test repeats the same dance: create a throwaway
// container, append it to body, write `<tag attrs>` HTML, query the
// shadow root. These helpers compress the boilerplate.

export interface TestContainerRegistry {
  mk(): HTMLElement
  cleanup(): void
}

export function createContainerRegistry(): TestContainerRegistry {
  const containers: HTMLElement[] = []
  return {
    mk() {
      const c = document.createElement('div')
      document.body.appendChild(c)
      containers.push(c)
      return c
    },
    cleanup() {
      for (const c of containers.splice(0)) c.remove()
    },
  }
}

export function mount(
  reg: TestContainerRegistry,
  tag: string,
  attrs: Record<string, string | boolean> = {},
  innerHTML: string = '',
): { el: HTMLElement; shadow: ShadowRoot | null; container: HTMLElement } {
  const container = reg.mk()
  const attrsHtml = Object.entries(attrs)
    .map(([k, v]) =>
      v === true ? k : v === false ? '' : `${k}="${escapeAttr(String(v))}"`,
    )
    .filter((s) => s.length > 0)
    .join(' ')
  container.innerHTML = `<${tag}${attrsHtml ? ' ' + attrsHtml : ''}>${innerHTML}</${tag}>`
  const el = container.querySelector(tag) as HTMLElement
  return { el, shadow: el.shadowRoot, container }
}

// One-tick flush into the microtask queue — mirrors the agents pack
// helper, used to wait for the SUI runtime to resolve its initial
// render + scope setup before assertions run.
export function flush(): Promise<void> {
  const { promise, resolve } = Promise.withResolvers<void>()
  setTimeout(resolve, 0)
  return promise
}

const escapeAttr = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;')
