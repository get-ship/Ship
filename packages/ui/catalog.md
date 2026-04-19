# Ship element catalog

Compact reference auto-generated from `custom-elements.json`. For depth (types, defaults, descriptions, CSS custom properties), load the full CEM at `/api/runtime/assets/custom-elements.json`.

Convention: `attr=value|value` = enum, bare `attr` = boolean.


## Core primitives

- `<ship-alert>` [tone dismissible] events: ship-dismiss
- `<ship-alert-dialog>` [open tone=default|destructive confirm-label cancel-label] events: name
- `<ship-annotation-layer>` [view-id mode endpoint staging staged _pending _drag] events: ship-annotation-staged, ship-annotation-error, ship-annotation-sent
- `<ship-annotation-list>` [staged] events: ship-annotation-remove
- `<ship-annotation-marker>` [kind x y rect active index]
- `<ship-badge>` [tone variant size pill]
- `<ship-button>` [tone variant size disabled loading type=button|submit|reset label pressed=true|false expanded=true|false]
- `<ship-card>` [tone variant padding interactive accent-bar status-rail fill]
- `<ship-checkbox>` [checked indeterminate disabled required invalid tone size name value label]
- `<ship-dialog>` [open size backdrop dismissible no-close-button no-header no-footer role-override=dialog|alertdialog] events: ship-open
- `<ship-dropdown-menu>` [open]
- `<ship-empty-state>` [variant=default|connecting|placeholder]
- `<ship-field>` [invalid]
- `<ship-input>` [type value placeholder name autocomplete required disabled readonly invalid loading tone variant=default size label described-by error-message] events: ship-input
- `<ship-label>` [for size=sm|md required]
- `<ship-list-item>` [selected disabled] events: ship-select
- `<ship-popover>` [open placement=top|bottom|left|right] events: name
- `<ship-progress>` [value max tone size=sm|md label]
- `<ship-select>` [value name required disabled invalid tone size label described-by] events: ship-change
- `<ship-separator>` [orientation=horizontal|vertical variant=solid|dashed]
- `<ship-spinner>` [tone size label]
- `<ship-stat>` [accent interactive]
- `<ship-status-dot>` [tone pulse size=sm|md label]
- `<ship-switch>` [checked disabled tone size name value label]
- `<ship-tab>` [name label count disabled]
- `<ship-tab-panel>` [name active]
- `<ship-tabs>` [active variant=underline|segmented size tone sticky orientation=horizontal|vertical] events: ship-tab-change
- `<ship-textarea>` [value placeholder name rows required disabled readonly tone variant=default|code size resize=none|vertical|horizontal|both invalid label described-by] events: ship-input
- `<ship-theme-toggle>` [theme=light|dark|auto]
- `<ship-toolbar>` [justify=start|center|end|between align=center|start|end|baseline gap=sm|md|lg variant=default|bordered]
- `<ship-tooltip>` [placement=top|bottom|left|right]

## Pipeline domain

- `<ship-git-diff>` [files additions deletions empty allCollapsed]
- `<ship-git-diff-file>` [path additions deletions collapsed]
- `<ship-git-diff-hunk>` [header]
- `<ship-git-diff-line>` [type old new]
- `<ship-history-row>` [state]
- `<ship-pipeline-dag>`
- `<ship-pipeline-definition>` [name steps description]
- `<ship-pipeline-run>` [state slug agent]
- `<ship-pipeline-step>` [name step-type state agent]
- `<ship-spec-row>` [item-title status pipeline selected]

## Observability

- `<ship-code>` [lang block]
- `<ship-copy-button>` [value label copied] events: ship-copy
- `<ship-diff>` [from to value]
- `<ship-event-row>` [actor verb target at]
- `<ship-json-view>` [data expanded depth]
- `<ship-log-line>` [level timestamp source]
- `<ship-markdown>` [value]
- `<ship-table>` [variant density dense hoverable striped sticky-header label] events: ship-sort
- `<ship-timeline>`
- `<ship-timeline-item>` [tone at interactive] events: ship-select
- `<ship-timestamp>` [value format]
- `<ship-tree-node>` [label expanded selected leaf]
- `<ship-tree-view>`

## Charts

- `<ship-area-chart>` [series labels width height points grid label]
- `<ship-bar-chart>` [data labels tone orientation=vertical|horizontal width height max label]
- `<ship-donut>` [data size thickness label]
- `<ship-gauge>` [value min max tone low-threshold high-threshold size thickness label]
- `<ship-heatmap>` [data rows cols tone label]
- `<ship-line-chart>` [series labels width height points grid label]
- `<ship-meter>` [segments total label]
- `<ship-progress-ring>` [value size thickness tone label]
- `<ship-sparkline>` [data tone width height fill label]

## Utility

- `<ship-avatar>` [src name size tone _errored]
- `<ship-breadcrumb>`
- `<ship-breadcrumb-item>` [href active]
- `<ship-chip>` [tone variant size selected removable disabled] events: ship-remove
- `<ship-icon>` [name size tone label]
- `<ship-kbd>` [keys]
- `<ship-link>` [href external tone]
- `<ship-metachip>` [label value]
- `<ship-skeleton>` [variant width height lines]
- `<ship-step-segments>` [values size=sm|md]
- `<ship-toast>` [tone duration dismissible] events: ship-dismiss
- `<ship-toast-container>` [position]
