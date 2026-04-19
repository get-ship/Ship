/**
 * Status & feedback primitive samples.
 */

import { matrix, row, TONES, VARIANTS_BADGE, type Sample } from './types'

export const STATUS: Record<string, Sample[]> = {
  'ship-badge': [
    matrix('ship-badge', TONES, VARIANTS_BADGE, 'tone × variant matrix'),
    row(
      ['sm', 'md', 'lg']
        .map(
          (s) => `<ship-badge size="${s}" tone="info">Size ${s}</ship-badge>`,
        )
        .concat([
          '<ship-badge pill tone="success">pill</ship-badge>',
          '<ship-badge tone="accent" variant="outline">outline</ship-badge>',
          '<ship-badge tone="warning" variant="solid"><ship-status-dot slot="leading" tone="warning"></ship-status-dot>with dot</ship-badge>',
        ])
        .join(''),
    ),
  ],
  'ship-status-dot': [
    row(
      [
        '<ship-status-dot tone="success" label="ok"></ship-status-dot>',
        '<ship-status-dot tone="info" pulse label="live"></ship-status-dot>',
        '<ship-status-dot tone="warning" label="warn"></ship-status-dot>',
        '<ship-status-dot tone="destructive" label="down"></ship-status-dot>',
        '<ship-status-dot tone="muted" size="sm" label="idle"></ship-status-dot>',
      ].join(''),
    ),
  ],
  'ship-spinner': [
    row(
      [
        '<ship-spinner size="sm"></ship-spinner>',
        '<ship-spinner></ship-spinner>',
        '<ship-spinner size="lg" tone="info"></ship-spinner>',
        '<ship-spinner tone="success"></ship-spinner>',
      ].join(''),
    ),
  ],
  'ship-progress': [
    row(
      [
        '<ship-progress value="20" max="100" tone="info" style="width:180px"></ship-progress>',
        '<ship-progress value="60" max="100" tone="success" style="width:180px"></ship-progress>',
        '<ship-progress value="92" max="100" tone="warning" style="width:180px"></ship-progress>',
        '<ship-progress tone="accent" style="width:180px"></ship-progress>',
      ].join(''),
    ),
  ],
  'ship-progress-ring': [
    row(
      [
        '<ship-progress-ring value="20" max="100"></ship-progress-ring>',
        '<ship-progress-ring value="52" max="100" tone="info"></ship-progress-ring>',
        '<ship-progress-ring value="88" max="100" tone="success"></ship-progress-ring>',
      ].join(''),
    ),
  ],
  'ship-alert': [
    {
      props: { tone: 'info' },
      innerHTML:
        '<span slot="title">Heads up</span>New version of ship is available. Run <ship-code>ship upgrade</ship-code> to get it.',
      metaAttrs: { tone: 'info' },
    },
    {
      props: { tone: 'success' },
      innerHTML:
        '<span slot="title">Pipeline merged</span>rust-lane merged fix-auth into main.',
    },
    {
      props: { tone: 'warning' },
      innerHTML:
        '<span slot="title">Slow response</span>Gate took longer than usual to complete.',
    },
    {
      props: { tone: 'destructive', dismissible: '' },
      innerHTML:
        '<span slot="title">Build failed</span>3 tests failed in crates/runtime.',
    },
  ],
}
