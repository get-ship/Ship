/**
 * Utility primitive samples: icons / avatars / metachips / skeletons /
 * theme toggle / empty state / step segments.
 */

import { row, type Sample } from './types'

export const UTILITY: Record<string, Sample[]> = {
  'ship-icon': [
    row(
      [
        'check',
        'x',
        'chevron-down',
        'search',
        'star',
        'folder',
        'sparkles',
        'copy',
      ]
        .map((n) => `<ship-icon name="${n}" size="lg"></ship-icon>`)
        .join(''),
    ),
  ],
  'ship-avatar': [
    row(
      [
        '<ship-avatar name="Ada Lovelace" size="sm"></ship-avatar>',
        '<ship-avatar name="Grace Hopper"></ship-avatar>',
        '<ship-avatar name="Alan Turing" size="lg" tone="info"></ship-avatar>',
        '<ship-avatar name="Linus Torvalds" size="xl" tone="accent"></ship-avatar>',
      ].join(''),
    ),
  ],
  'ship-theme-toggle': [{}],
  'ship-metachip': [
    {
      innerHTML:
        '<div style="display:flex;gap:0">' +
        '<ship-metachip label="agent" value="rust-lane"></ship-metachip>' +
        '<ship-metachip label="branch" value="fix-auth"></ship-metachip>' +
        '<ship-metachip label="run" value="142"></ship-metachip>' +
        '</div>',
    },
  ],
  'ship-skeleton': [
    row(
      [
        '<div style="width:220px"><ship-skeleton variant="text" lines="3"></ship-skeleton></div>',
        '<ship-skeleton variant="circle" width="40px" height="40px"></ship-skeleton>',
        '<ship-skeleton variant="rect" width="120px" height="40px"></ship-skeleton>',
      ].join(''),
    ),
  ],
  'ship-step-segments': [
    row(
      [
        '<ship-step-segments values="done,done,done,done,done"></ship-step-segments>',
        '<ship-step-segments values="done,done,running,pending,pending"></ship-step-segments>',
        '<ship-step-segments values="done,failed,pending,pending,pending" size="md"></ship-step-segments>',
      ].join(''),
    ),
  ],
  'ship-empty-state': [
    {
      raw: true,
      innerHTML:
        '<div class="sample-triptych">' +
        '<ship-empty-state icon="folder"><span slot="title">No items yet</span>Create your first pipeline to see it here.<ship-button slot="action" tone="info" size="sm">Create pipeline</ship-button></ship-empty-state>' +
        '<ship-empty-state variant="placeholder" icon="sparkles" tone="accent"><span slot="title">Nothing selected</span>Pick an item from the rail to inspect it.</ship-empty-state>' +
        '<ship-empty-state variant="inline" icon="search"><span slot="title">No results</span>Try a different query.</ship-empty-state>' +
        '</div>',
      metaAttrs: { variant: 'default|placeholder|inline' },
    },
  ],
}
