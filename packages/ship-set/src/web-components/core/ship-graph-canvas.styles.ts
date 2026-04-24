/**
 * Styles for <ship-graph-canvas>.
 *
 * Three channels:
 *
 *   Edge strokes  — --edge-<kind> custom properties mapped from tone tokens.
 *                   agent=primary, skill=success, mcp=info, rule=warning,
 *                   perm=destructive, provider=accent.
 *
 *   Node chrome   — surface tokens (--card, --border, --radius-md) plus a
 *                   3px left rail keyed to the same kind->edge color so
 *                   nodes visually echo their outgoing/incoming edges.
 *
 *   Layout        — viewBox-relative positioning: each node sits at
 *                   left/top percentages of the container. SVG uses
 *                   preserveAspectRatio="none" so edges track node
 *                   percentages across any container size.
 */

import { css } from 'lit'
import { reducedMotion } from './util'

export const shipGraphCanvasStyles = [
  reducedMotion,
  css`
    :host {
      display: block;
      position: relative;
      width: 100%;
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      overflow: hidden;
      --edge-default: var(--border);
      --edge-agent: var(--tone-primary-fg, var(--foreground));
      --edge-skill: var(--tone-success-fg);
      --edge-mcp: var(--tone-info-fg);
      --edge-rule: var(--tone-warning-fg);
      --edge-perm: var(--tone-destructive-fg);
      --edge-provider: var(--tone-accent-fg);
    }
    :host([hidden]) { display: none; }
    .canvas {
      position: relative;
      width: 100%;
      aspect-ratio: var(--graph-aspect, 5 / 3);
    }
    svg.edges {
      position: absolute; inset: 0;
      width: 100%; height: 100%;
      pointer-events: none;
    }
    svg.edges path {
      fill: none;
      stroke: var(--edge-default);
      stroke-width: 1.5;
      opacity: 0.85;
    }
    svg.edges path[data-kind='agent']    { stroke: var(--edge-agent); }
    svg.edges path[data-kind='skill']    { stroke: var(--edge-skill); }
    svg.edges path[data-kind='mcp']      { stroke: var(--edge-mcp); }
    svg.edges path[data-kind='rule']     { stroke: var(--edge-rule); }
    svg.edges path[data-kind='perm']     { stroke: var(--edge-perm); }
    svg.edges path[data-kind='provider'] { stroke: var(--edge-provider); }
    svg.edges path[data-dashed] { stroke-dasharray: 4 3; }

    .node {
      position: absolute;
      transform: translate(-50%, -50%);
      width: var(--node-w, 180px);
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      padding: var(--space-2) var(--space-3);
      box-shadow: var(--shadow-xs);
      cursor: pointer;
      color: var(--foreground);
      font-family: var(--font-body);
      font-size: var(--text-sm);
      transition: box-shadow var(--duration-fast) var(--ease-out),
                  border-color var(--duration-fast) var(--ease-out);
    }
    .node:hover { box-shadow: var(--shadow-sm); }
    .node:focus-visible {
      outline: 2px solid var(--ring);
      outline-offset: 2px;
    }
    .node[data-selected] {
      border-color: var(--ring);
      box-shadow: var(--shadow-sm);
    }
    .node[data-kind='agent']    { border-left: 3px solid var(--edge-agent); }
    .node[data-kind='skill']    { border-left: 3px solid var(--edge-skill); }
    .node[data-kind='mcp']      { border-left: 3px solid var(--edge-mcp); }
    .node[data-kind='rule']     { border-left: 3px solid var(--edge-rule); }
    .node[data-kind='perm']     { border-left: 3px solid var(--edge-perm); }
    .node[data-kind='provider'] { border-left: 3px solid var(--edge-provider); }

    .tpl-host .node-h {
      display: flex; align-items: center; gap: var(--space-2);
      margin-bottom: var(--space-1);
    }
    .tpl-host .tag {
      font-size: var(--text-xs);
      font-weight: var(--font-weight-semibold);
      letter-spacing: 0.04em;
      text-transform: uppercase;
      color: var(--muted-foreground);
    }
    .tpl-host .node-title {
      font-weight: var(--font-weight-semibold);
      color: var(--foreground);
    }
    .tpl-host .node-sub {
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }

    ::slotted(template) { display: none; }
  `,
]
