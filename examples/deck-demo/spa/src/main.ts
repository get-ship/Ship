// SPA entry — deck's `next export` analog: the client renders everything, no
// view-host, no SSR. A counter built on the real SUI runtime, which the build
// leaves EXTERNAL (resolved to /_sui/runtime.js via the importmap) so there is
// exactly one runtime instance whether a daemon serves /_sui/* or a static
// deploy ships it.
import { effect, signal } from '@ship/sui-runtime'

const root = document.getElementById('root')
if (root) {
  const count = signal(0)
  const value = document.createElement('strong')
  const button = document.createElement('button')
  button.textContent = 'increment'
  button.addEventListener('click', () => count.set(count() + 1))
  effect(() => {
    value.textContent = String(count())
  })
  const line = document.createElement('p')
  line.append('Count: ', value)
  const heading = document.createElement('h1')
  heading.textContent = 'deck SPA'
  root.append(heading, line, button)
}
