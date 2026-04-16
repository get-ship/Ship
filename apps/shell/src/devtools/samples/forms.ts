/**
 * Form primitive samples: input / textarea / select / checkbox / switch /
 * field / label.
 */

import { row, type Sample } from './types'

export const FORMS: Record<string, Sample[]> = {
  'ship-input': [
    row(
      [
        '<ship-input placeholder="default"></ship-input>',
        '<ship-input tone="info" placeholder="info tone"></ship-input>',
        '<ship-input invalid placeholder="invalid"></ship-input>',
        '<ship-input loading placeholder="loading"></ship-input>',
        '<ship-input disabled value="read-only"></ship-input>',
      ].join(''),
    ),
    row(
      [
        '<ship-input size="sm" placeholder="sm"></ship-input>',
        '<ship-input size="md" placeholder="md"></ship-input>',
        '<ship-input size="lg" placeholder="lg"></ship-input>',
      ].join(''),
    ),
  ],
  'ship-textarea': [
    {
      props: { placeholder: 'Write a short note…', rows: '3' },
      metaAttrs: { placeholder: '…', rows: '3' },
    },
    {
      props: {
        variant: 'code',
        rows: '3',
        value: 'function ok() {\n  return true\n}',
      },
      metaAttrs: { variant: 'code' },
    },
  ],
  'ship-select': [
    {
      innerHTML:
        '<option value="a">Alpha</option>' +
        '<option value="b">Bravo</option>' +
        '<option value="c">Charlie</option>',
    },
    {
      props: { tone: 'info', size: 'sm' },
      innerHTML:
        '<option value="1">Small · info</option>' +
        '<option value="2">Second</option>',
    },
    {
      props: { invalid: '', size: 'lg' },
      innerHTML:
        '<option value="x">Invalid · large</option>' +
        '<option value="y">Second</option>',
    },
  ],
  'ship-checkbox': [
    row(
      [
        '<ship-checkbox label="unchecked"></ship-checkbox>',
        '<ship-checkbox checked label="checked"></ship-checkbox>',
        '<ship-checkbox indeterminate label="mixed"></ship-checkbox>',
        '<ship-checkbox checked tone="success" label="success"></ship-checkbox>',
        '<ship-checkbox checked tone="destructive" label="destructive"></ship-checkbox>',
        '<ship-checkbox disabled checked label="disabled"></ship-checkbox>',
      ].join(''),
    ),
  ],
  'ship-switch': [
    row(
      [
        '<ship-switch label="off"></ship-switch>',
        '<ship-switch checked label="on"></ship-switch>',
        '<ship-switch checked tone="info" label="info"></ship-switch>',
        '<ship-switch checked tone="success" label="success"></ship-switch>',
        '<ship-switch checked tone="warning" label="warning"></ship-switch>',
        '<ship-switch disabled checked label="disabled"></ship-switch>',
      ].join(''),
    ),
  ],
  'ship-field': [
    {
      innerHTML:
        '<ship-label slot="label" for="gallery-email">Email address</ship-label>' +
        '<ship-input id="gallery-email" placeholder="you@ship.dev"></ship-input>' +
        '<span slot="hint">We never share your address.</span>',
      metaAttrs: { composition: 'label + input + hint' },
    },
    {
      props: { invalid: '' },
      innerHTML:
        '<ship-label slot="label" required>API token</ship-label>' +
        '<ship-input invalid placeholder="ship_…"></ship-input>' +
        '<span slot="error">Token must start with ship_.</span>',
    },
  ],
  'ship-label': [
    { innerHTML: 'Label text' },
    { props: { required: '' }, innerHTML: 'Required label' },
    { props: { size: 'sm' }, innerHTML: 'Small label' },
  ],
}
