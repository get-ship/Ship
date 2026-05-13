/**
 * React JSX type augmentation for Ship web components.
 *
 * Without this, `<ship-button>` in TSX raises "Property … does not exist on
 * type JSX.IntrinsicElements". Each tag is typed permissively as
 * `HTMLAttributes<HTMLElement>` plus the loose Lit-component-prop pattern
 * (every prop optional, event handlers and arbitrary attrs allowed).
 *
 * Consumers that want stricter per-tag prop types can re-augment locally.
 *
 * Activate by including this file in the consuming app's tsconfig
 * `include` (e.g. `packages/ship-set/src/web-components/react.d.ts`) — it is not
 * referenced from runtime code so it has no bundle impact.
 */

import type { HTMLAttributes, DetailedHTMLProps } from 'react'

type ShipElement<T extends HTMLElement = HTMLElement, P = Record<string, unknown>> =
  DetailedHTMLProps<HTMLAttributes<T> & Partial<P>, T>

declare module 'react' {
  namespace JSX {
    interface IntrinsicElements {
      // Core primitives
      'ship-alert': ShipElement
      'ship-alert-dialog': ShipElement
      'ship-badge': ShipElement
      'ship-button': ShipElement
      'ship-card': ShipElement
      'ship-checkbox': ShipElement
      'ship-dialog': ShipElement
      'ship-dropdown-menu': ShipElement
      'ship-empty-state': ShipElement
      'ship-field': ShipElement
      'ship-input': ShipElement
      'ship-label': ShipElement
      'ship-list-tile': ShipElement
      'ship-popover': ShipElement
      'ship-select': ShipElement
      'ship-separator': ShipElement
      'ship-spinner': ShipElement
      'ship-stat': ShipElement
      'ship-status-dot': ShipElement
      'ship-switch': ShipElement
      'ship-tabs': ShipElement
      'ship-tab': ShipElement
      'ship-tab-panel': ShipElement
      'ship-textarea': ShipElement
      'ship-theme-toggle': ShipElement
      'ship-toolbar': ShipElement
      'ship-tooltip': ShipElement

      // Domain
      'ship-pipeline-step': ShipElement
      'ship-pipeline-run': ShipElement
      'ship-pipeline-definition': ShipElement
      'ship-pipeline-dag': ShipElement
      'ship-git-diff': ShipElement
      'ship-git-diff-file': ShipElement
      'ship-git-diff-hunk': ShipElement
      'ship-git-diff-line': ShipElement

      // Observability
      'ship-code': ShipElement
      'ship-copy-button': ShipElement
      'ship-diff': ShipElement
      'ship-json-view': ShipElement
      'ship-markdown': ShipElement
      'ship-table': ShipElement
      'ship-timeline': ShipElement
      'ship-timeline-item': ShipElement
      'ship-timestamp': ShipElement
      'ship-tree-view': ShipElement
      'ship-tree-item': ShipElement

      // Charts
      'ship-bar-chart': ShipElement
      'ship-gauge': ShipElement
      'ship-heatmap': ShipElement
      'ship-line-chart': ShipElement
      'ship-meter': ShipElement
      'ship-progress-ring': ShipElement
      'ship-sparkline': ShipElement

      // Utility
      'ship-avatar': ShipElement
      'ship-breadcrumb': ShipElement
      'ship-breadcrumb-item': ShipElement
      'ship-chip': ShipElement
      'ship-icon': ShipElement
      'ship-link': ShipElement
      'ship-metachip': ShipElement
      'ship-skeleton': ShipElement
      'ship-step-segments': ShipElement
      'ship-toast': ShipElement
      'ship-toast-container': ShipElement
    }
  }
}
