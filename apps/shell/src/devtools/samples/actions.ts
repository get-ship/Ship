/**
 * Buttons, chips, and copy-button samples.
 */

import { matrix, row, TONES, VARIANTS_BADGE, VARIANTS_BTN, type Sample } from './types'

export const ACTIONS: Record<string, Sample[]> = {
  'ship-button': [
    matrix('ship-button', TONES, VARIANTS_BTN, 'tone × variant matrix'),
    row(
      ['sm', 'md', 'lg']
        .map(
          (s) => `<ship-button size="${s}" tone="info">Size ${s}</ship-button>`,
        )
        .join(''),
    ),
    row(
      [
        '<ship-button loading>Loading</ship-button>',
        '<ship-button disabled>Disabled</ship-button>',
        '<ship-button tone="destructive" variant="soft">Delete</ship-button>',
        '<ship-button tone="success" variant="solid"><ship-icon slot="leading" name="check"></ship-icon>Confirm</ship-button>',
      ].join(''),
    ),
  ],
  'ship-chip': [
    matrix('ship-chip', TONES, VARIANTS_BADGE, 'tone × variant matrix'),
    row(
      [
        '<ship-chip tone="info" selected>selected</ship-chip>',
        '<ship-chip tone="success" removable>removable</ship-chip>',
        '<ship-chip disabled>disabled</ship-chip>',
        '<ship-chip size="sm" tone="accent">sm · accent</ship-chip>',
        '<ship-chip size="lg" tone="warning" variant="outline">lg · outline</ship-chip>',
      ].join(''),
    ),
  ],
  'ship-copy-button': [
    { props: { value: 'npm i @ship/ui', label: 'Copy install command' } },
    { props: { value: 'ship run polish' } },
  ],
}
