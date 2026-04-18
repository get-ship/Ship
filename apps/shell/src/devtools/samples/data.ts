/**
 * Data-display samples: stats / tables / charts / heatmap / etc.
 */

import { row, type Sample } from './types'

export const DATA: Record<string, Sample[]> = {
  'ship-stat': [
    row(
      [
        '<ship-stat><span slot="label">Uptime</span>99.9%<span slot="hint">Past 30 days</span></ship-stat>',
        '<ship-stat accent="success"><span slot="label">Passed</span>142<ship-badge slot="trend" tone="success" size="sm">+12</ship-badge></ship-stat>',
        '<ship-stat accent="destructive"><span slot="label">Failed</span>3<ship-badge slot="trend" tone="destructive" size="sm">-1</ship-badge></ship-stat>',
        '<ship-stat accent="info"><span slot="label">Active agents</span>7<span slot="hint">rust, web, qa…</span></ship-stat>',
      ].join(''),
    ),
  ],
  'ship-table': [
    {
      props: { variant: 'native', striped: '', hoverable: '' },
      innerHTML:
        '<table>' +
        '<thead><tr><th>Agent</th><th>Lane</th><th>State</th><th align="right">Runs</th></tr></thead>' +
        '<tbody>' +
        '<tr><td class="mono">rust-lane</td><td>rust</td><td>idle</td><td align="right">142</td></tr>' +
        '<tr><td class="mono">web-lane</td><td>web</td><td>busy</td><td align="right">87</td></tr>' +
        '<tr><td class="mono">qa</td><td>qa</td><td>idle</td><td align="right">31</td></tr>' +
        '<tr><td class="mono">reviewer</td><td>gate</td><td>idle</td><td align="right">64</td></tr>' +
        '</tbody>' +
        '</table>',
      metaAttrs: { variant: 'native', striped: '' },
    },
  ],
  'ship-meter': [
    {
      props: {
        segments: JSON.stringify([
          { tone: 'success', count: 18, label: 'passed' },
          { tone: 'warning', count: 3, label: 'warn' },
          { tone: 'destructive', count: 1, label: 'failed' },
        ]),
        total: '24',
      },
      metaAttrs: { segments: 'json', total: '24' },
    },
  ],
  'ship-gauge': [
    row(
      [
        '<ship-gauge value="32" max="100" label="Low">32</ship-gauge>',
        '<ship-gauge value="68" tone="info" label="Medium">68</ship-gauge>',
        '<ship-gauge value="92" tone="success" label="High">92</ship-gauge>',
      ].join(''),
    ),
  ],
  'ship-donut': [
    row(
      [
        `<ship-donut data='${JSON.stringify([
          { label: 'passed', value: 18, tone: 'success' },
          { label: 'warn', value: 3, tone: 'warning' },
          { label: 'fail', value: 1, tone: 'destructive' },
        ])}'>22</ship-donut>`,
        `<ship-donut data='${JSON.stringify([
          { label: 'a', value: 40, tone: 'info' },
          { label: 'b', value: 60, tone: 'accent' },
        ])}'>60%</ship-donut>`,
      ].join(''),
    ),
  ],
  'ship-sparkline': [
    row(
      [
        '<ship-sparkline data="3,5,4,7,6,9,8,11,10,13" tone="success" fill></ship-sparkline>',
        '<ship-sparkline data="13,10,11,9,7,9,5,8,6,4" tone="destructive" fill></ship-sparkline>',
        '<ship-sparkline data="5,5,7,6,8,7,9,8,10,9" tone="info"></ship-sparkline>',
      ].join(''),
    ),
  ],
  'ship-bar-chart': [
    {
      props: {
        data: '12,19,7,24,15,22,30,26',
        labels: 'M,T,W,T,F,S,S,+',
        tone: 'accent',
      },
      metaAttrs: { data: '8 bars', tone: 'accent' },
    },
  ],
  'ship-line-chart': [
    {
      props: {
        series: JSON.stringify([
          {
            name: 'passed',
            data: [1, 4, 3, 7, 9, 8, 11, 14, 12, 16],
            tone: 'success',
          },
          {
            name: 'failed',
            data: [3, 2, 4, 1, 2, 1, 0, 2, 1, 0],
            tone: 'destructive',
          },
        ]),
        grid: '',
        points: '',
      },
      innerHTML:
        '<span><ship-status-dot tone="success"></ship-status-dot> passed</span>' +
        '<span><ship-status-dot tone="destructive"></ship-status-dot> failed</span>',
      metaAttrs: { series: '2 series', grid: '', points: '' },
    },
  ],
  'ship-area-chart': [
    {
      props: {
        series: JSON.stringify([
          {
            name: 'total',
            data: [5, 9, 7, 14, 12, 18, 20, 24, 22, 28],
            tone: 'info',
          },
        ]),
        grid: '',
      },
      metaAttrs: { series: '1 series', grid: '' },
    },
  ],
  'ship-pagination': [
    {
      props: { page: '2', total: '127', 'per-page': '25', tone: 'info' },
      metaAttrs: { page: '2', total: '127', 'per-page': '25', tone: 'info' },
    },
  ],
  'ship-heatmap': [
    {
      props: {
        data: JSON.stringify(
          Array.from({ length: 14 }, (_, x) =>
            Array.from({ length: 5 }, (_, y) => ({
              x: `d${x}`,
              y: ['mon', 'tue', 'wed', 'thu', 'fri'][y],
              value: Math.max(
                0,
                Math.min(
                  1,
                  (Math.sin(x * 0.6) + Math.cos(y * 0.8)) * 0.4 + 0.55,
                ),
              ),
            })),
          ).flat(),
        ),
        rows: 'mon,tue,wed,thu,fri',
        cols: 'd0,d1,d2,d3,d4,d5,d6,d7,d8,d9,d10,d11,d12,d13',
        tone: 'accent',
      },
      metaAttrs: { data: '14×5 cells', tone: 'accent' },
    },
  ],
}
