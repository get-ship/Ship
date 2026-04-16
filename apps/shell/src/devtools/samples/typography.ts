/**
 * Typography, text, and code samples.
 */

import { row, type Sample } from './types'

export const TYPOGRAPHY: Record<string, Sample[]> = {
  'ship-markdown': [
    {
      props: {
        value:
          '# A heading\n\nA paragraph with **bold**, *italic*, and `inline code`.\n\n- first item\n- second item\n\n```js\nconst greet = (name) => `hello, ${name}`\n```',
      },
      metaAttrs: { value: 'markdown source' },
    },
  ],
  'ship-code': [
    {
      props: { block: '', lang: 'rust' },
      innerHTML: 'fn main() {\n    println!("hello from ship");\n}',
    },
    {
      props: {},
      innerHTML: 'inline <ship-code>--count</ship-code> sample',
    },
  ],
  'ship-kbd': [
    { props: { keys: 'Cmd+K' } },
    { props: { keys: 'Ctrl+Shift+P' } },
    { props: { keys: 'Esc' } },
  ],
  'ship-link': [
    { props: { href: '#' }, innerHTML: 'Primary link' },
    { props: { href: '#', tone: 'muted' }, innerHTML: 'Muted link' },
    {
      props: { href: 'https://ship.dev', external: '' },
      innerHTML: 'External link',
    },
  ],
  'ship-timestamp': [
    { props: { value: new Date(Date.now() - 45_000).toISOString() } },
    {
      props: {
        value: new Date(Date.now() - 3_600_000).toISOString(),
        format: 'both',
      },
    },
  ],
  'ship-json-view': [
    {
      props: {
        data: JSON.stringify(
          {
            name: 'ship',
            version: '0.2.0',
            ok: true,
            agents: ['rust-lane', 'web-lane', 'qa'],
            meta: { built: '2026-04-16', commit: 'a7831cfa' },
          },
          null,
          2,
        ),
        depth: '2',
      },
      metaAttrs: { data: 'json', depth: '2' },
    },
  ],
}
