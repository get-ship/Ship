/**
 * Form primitive samples: input / textarea / select / checkbox / switch /
 * radio-group / slider / toggle-group / field / label.
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
  'ship-radio-group': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-row" style="gap:32px">' +
        '<ship-radio-group value="prod">' +
        '<ship-radio value="dev">Development</ship-radio>' +
        '<ship-radio value="staging">Staging</ship-radio>' +
        '<ship-radio value="prod">Production</ship-radio>' +
        '</ship-radio-group>' +
        '<ship-radio-group value="fast" tone="accent">' +
        '<ship-radio value="fast">Fast</ship-radio>' +
        '<ship-radio value="balanced">Balanced</ship-radio>' +
        '<ship-radio value="thorough">Thorough</ship-radio>' +
        '</ship-radio-group>' +
        '</div>',
      metaAttrs: { tone: 'default|accent' },
    },
  ],
  'ship-slider': [
    row(
      [
        '<ship-slider value="40" style="width:200px" label="Volume"></ship-slider>',
        '<ship-slider value="70" tone="info" style="width:200px" label="Brightness"></ship-slider>',
        '<ship-slider value="50" tone="success" style="width:200px" label="Quality"></ship-slider>',
        '<ship-slider value="30" disabled style="width:200px" label="Disabled"></ship-slider>',
      ].join(''),
    ),
  ],
  'ship-toggle-group': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-row" style="gap:24px">' +
        '<ship-toggle-group type="single" value="grid">' +
        '<ship-toggle value="list">List</ship-toggle>' +
        '<ship-toggle value="grid">Grid</ship-toggle>' +
        '<ship-toggle value="board">Board</ship-toggle>' +
        '</ship-toggle-group>' +
        '<ship-toggle-group type="multiple" value="bold,italic" variant="ghost">' +
        '<ship-toggle value="bold">B</ship-toggle>' +
        '<ship-toggle value="italic">I</ship-toggle>' +
        '<ship-toggle value="underline">U</ship-toggle>' +
        '</ship-toggle-group>' +
        '</div>',
      metaAttrs: { type: 'single|multiple' },
    },
  ],
}
