// Thin custom-element → React bridge for Excalidraw.
//
// No prior React-in-shadow-DOM art exists in this repo (every other
// rich wrapper — graph-canvas, log-stream, code editors — renders with
// `bindHtml` or plain DOM). So this bridge owns the React-root lifecycle
// itself, and that choice is documented here:
//
//   - Excalidraw is a React component. We create ONE react-dom `Root`
//     per host, render <Excalidraw> into a container that lives inside
//     the host's shadow root, and `root.unmount()` on disconnect. That
//     unmount tears down the entire React fiber tree and every listener
//     Excalidraw attached — no leaked fibers, no leaked listeners.
//   - SUI components are always shadow-rooted, so Excalidraw's stylesheet
//     (which it does NOT auto-inject in v0.18) is adopted into the shadow
//     root as a constructable sheet. `sui.config.ts` maps `.css` → the
//     `text` loader so the import below resolves to the raw CSS string.
//
// The bridge exposes a tiny imperative surface — `update()` to push new
// props, `destroy()` to dispose — and is the ONLY module that imports
// React. Keeping it isolated means the ~1 MB Excalidraw + React tree
// only loads for consumers that actually mount <sui-canvas>.
import { createElement } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { Excalidraw, getSceneVersion } from '@excalidraw/excalidraw'
// Excalidraw's stylesheet, inlined as text (sui.config maps `.css` → the
// `text` loader). We reach the file by a RELATIVE path into node_modules
// on purpose: the package's `./index.css` export entry only declares
// `development`/`production` conditions with no `default`, and the bundler
// (Bun) doesn't pass an explicit export condition — so the bare specifier
// is unresolvable. A relative path bypasses package-exports encapsulation
// and resolves the prod stylesheet directly. @excalidraw/excalidraw is a
// direct dependency, so the path is stable for installs of this pack.
// @ts-expect-error — CSS-as-text import, no type declaration.
import excalidrawCss from '../../node_modules/@excalidraw/excalidraw/dist/prod/index.css'

export type CanvasTheme = 'light' | 'dark'

export interface CanvasProps {
  value: string
  readonly: boolean
  theme: CanvasTheme
}

export interface BridgeOptions extends CanvasProps {
  /** Fired on every Excalidraw scene edit (raw — caller debounces). */
  onChange: (value: string, sceneVersion: number) => void
}

export interface CanvasBridge {
  /** Re-render with new props (value / readonly / theme). */
  update(props: CanvasProps): void
  /** Unmount the React root and free all resources. */
  destroy(): void
}

interface ExcalidrawElementsState {
  elements: readonly unknown[]
  appState: Record<string, unknown>
  files: Record<string, unknown>
}

/** Parse a scene-JSON string into Excalidraw `initialData`. Bad JSON
 * (or an empty value) yields `null` — Excalidraw treats that as a blank
 * scene rather than throwing. */
function parseScene(value: string): unknown {
  if (!value || !value.trim()) return null
  try {
    return JSON.parse(value)
  } catch {
    return null
  }
}

/** Serialize the current Excalidraw scene to the `value` contract. */
function serializeScene(state: ExcalidrawElementsState): string {
  return JSON.stringify({
    type: 'excalidraw',
    version: 2,
    elements: state.elements,
    appState: state.appState,
    files: state.files,
  })
}

/** Adopt Excalidraw's stylesheet into the shadow root once. Document-head
 * CSS can't cross the shadow boundary, so the surface would be unstyled
 * otherwise. No-op when the CSS text is empty (e.g. test env). */
function adoptStyles(root: ShadowRoot): void {
  if (!excalidrawCss || typeof CSSStyleSheet === 'undefined') return
  const marker = '__suiCanvasStylesAdopted'
  const tagged = root as ShadowRoot & Record<string, boolean>
  if (tagged[marker]) return
  try {
    const sheet = new CSSStyleSheet()
    sheet.replaceSync(excalidrawCss as unknown as string)
    root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet]
    tagged[marker] = true
  } catch {
    // Constructable stylesheets unsupported — skip; surface degrades
    // gracefully rather than failing the mount.
  }
}

function render(root: Root, props: CanvasProps, onChange: BridgeOptions['onChange']): void {
  root.render(
    createElement(Excalidraw, {
      initialData: parseScene(props.value),
      viewModeEnabled: props.readonly,
      theme: props.theme,
      onChange: (elements: readonly unknown[], appState: Record<string, unknown>, files: Record<string, unknown>) => {
        const state: ExcalidrawElementsState = { elements, appState, files }
        onChange(serializeScene(state), getSceneVersion(elements as never))
      },
    }),
  )
}

export function mountExcalidraw(container: HTMLElement, opts: BridgeOptions): CanvasBridge {
  const rootNode = container.getRootNode()
  if (rootNode instanceof ShadowRoot) adoptStyles(rootNode)

  const root = createRoot(container)
  let current: CanvasProps = { value: opts.value, readonly: opts.readonly, theme: opts.theme }
  render(root, current, opts.onChange)

  return {
    update(props: CanvasProps) {
      current = props
      render(root, current, opts.onChange)
    },
    destroy() {
      root.unmount()
    },
  }
}
