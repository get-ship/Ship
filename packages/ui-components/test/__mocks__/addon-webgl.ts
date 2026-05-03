// No-op stub for @xterm/addon-webgl in JSDOM (no WebGL2 support).
export class WebglAddon {
  onContextLoss(_cb: () => void) { return { dispose() {} } }
  activate() {}
  dispose() {}
}
