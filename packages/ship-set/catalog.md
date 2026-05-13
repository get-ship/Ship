# Ship element catalog

Compact reference auto-generated from `custom-elements.json`. For depth (types, defaults, descriptions, CSS custom properties), load the full CEM at `/api/assets/custom-elements.json`.

Convention: `attr=value|value` = enum, bare `attr` = boolean.


## Core primitives

- `<ship-accordion>` [type=single|multiple value] events: ship-change
- `<ship-accordion-item>` [value disabled open]
- `<ship-alert>` [tone dismissible] events: ship-dismiss
- `<ship-alert-dialog>` [open tone=default|destructive confirm-label cancel-label] events: name
- `<ship-annotation-layer>` [view-id mode endpoint staging staged _pending _drag] events: ship-annotation-staged, ship-annotation-error, ship-annotation-sent
- `<ship-annotation-list>` [staged] events: ship-annotation-remove
- `<ship-annotation-marker>` [kind x y rect active index]
- `<ship-app-icon>` [name variant size radius= | | | | | | |  label]
- `<ship-avatar>` [src name initials size shape tone icon-slug identity-hue _errored]
- `<ship-badge>` [tone variant size pill]
- `<ship-breadcrumb>`
- `<ship-breadcrumb-item>` [href active]
- `<ship-button>` [tone variant size disabled loading type=button|submit|reset label pressed=true|false expanded=true|false]
- `<ship-card>` [tone variant padding interactive accent-bar status-rail fill]
- `<ship-checkbox>` [checked indeterminate disabled required invalid tone size name value label]
- `<ship-chip>` [tone variant size selected removable disabled] events: ship-remove
- `<ship-code>` [lang block]
- `<ship-code-editor>` [value language= placeholder readonly line-numbers size=sm|md|lg tone theme=light|dark|auto] events: ship-save, ship-cancel, input, ship-change
- `<ship-collapsible>` [open disabled]
- `<ship-context-menu>` [open] events: name
- `<ship-dialog>` [open size backdrop dismissible no-close-button no-header no-footer role-override=dialog|alertdialog] events: ship-open
- `<ship-diff>` [from to value]
- `<ship-drawer>` [open side size dismissible] events: ship-open
- `<ship-dropdown-menu>` [open]
- `<ship-empty-state>` [variant size tone icon]
- `<ship-error-state>` [title body size variant icon _open _hasDetails]
- `<ship-field>` [invalid]
- `<ship-file-upload>` [accept multiple max-size disabled tone _files _over _error] events: ship-change
- `<ship-hover-card>` [open placement=top|bottom|left|right show-delay hide-delay] events: name
- `<ship-icon>` [name size tone label]
- `<ship-input>` [type value placeholder name autocomplete required disabled readonly invalid loading tone variant=default size label described-by error-message] events: ship-input
- `<ship-json-view>` [data expanded depth]
- `<ship-label>` [for size=sm|md required]
- `<ship-link>` [href external tone]
- `<ship-list-tile>` [variant interactive navigable selected disabled density tone]
- `<ship-mark>` [size]
- `<ship-markdown>` [value]
- `<ship-markdown-editor>` [value placeholder size tone disabled invalid rows full-height name label show-selection-toolbar allow-comment allow-generate] events: input, ship-change
- `<ship-navigation-item>` [value active disabled href]
- `<ship-navigation-menu>` [value]
- `<ship-pagination>` [page total per-page tone] events: ship-change
- `<ship-popover>` [open placement=top|bottom|left|right] events: name
- `<ship-radio>` [value disabled checked tone size]
- `<ship-radio-group>` [value disabled tone size name label]
- `<ship-scroll-area>` [orientation=vertical|horizontal|both size=sm|md|lg]
- `<ship-segmented>` [value size disabled] events: ship-change
- `<ship-segmented-option>` [value icon disabled]
- `<ship-select>` [value name required disabled invalid tone size label described-by] events: ship-change
- `<ship-separator>` [orientation=horizontal|vertical variant=solid|dashed]
- `<ship-skeleton>` [variant width height lines animate]
- `<ship-slider>` [value min max step disabled tone size label] events: ship-change
- `<ship-spinner>` [tone size label]
- `<ship-stat>` [accent interactive]
- `<ship-status-dot>` [tone pulse size=sm|md label]
- `<ship-stepper>` [total current tone size label]
- `<ship-switch>` [checked disabled tone size name value label]
- `<ship-tab>` [name label count disabled]
- `<ship-tab-panel>` [name active]
- `<ship-table>` [variant density dense hoverable striped sticky-header sticky-first-col gutter label] events: ship-sort
- `<ship-tabs>` [active variant=underline|segmented size tone sticky orientation=horizontal|vertical] events: ship-tab-change
- `<ship-textarea>` [value placeholder name rows required disabled readonly tone variant=default|code size resize=none|vertical|horizontal|both invalid label described-by] events: ship-input
- `<ship-theme-toggle>` [theme=light|dark|auto]
- `<ship-timestamp>` [value format]
- `<ship-toast>` [tone size duration dismissible show-progress] events: ship-dismiss
- `<ship-toast-container>` [position max-visible]
- `<ship-toggle>` [pressed disabled value size=sm|md|lg variant=outline|ghost]
- `<ship-toggle-group>` [type=single|multiple value size=sm|md|lg variant=outline|ghost] events: ship-change
- `<ship-toolbar>` [justify=start|center|end|between align=center|start|end|baseline gap=sm|md|lg variant=default|bordered]
- `<ship-tooltip>` [placement=top|bottom|left|right]
- `<ship-tree-node>` [label expanded selected leaf]
- `<ship-tree-view>`
- `<ship-view-chrome>` [title branch ahead behind tone=default|success|warning|destructive|accent|muted busy]

## Pipeline domain

- `<ship-auth-dialog>` [open mode loading backdrop=default|blur|solid] events: ship-mode-change
- `<ship-git-diff>` [files additions deletions empty allCollapsed]
- `<ship-git-diff-file>` [path additions deletions collapsed]
- `<ship-git-diff-hunk>` [header]
- `<ship-git-diff-line>` [type old new comment-count] events: ship-comment
- `<ship-history-row>` [sha subject author timestamp additions deletions active human] events: ship-select
- `<ship-pipeline-dag>`
- `<ship-pipeline-definition>` [name steps description]
- `<ship-pipeline-run>` [state slug agent]
- `<ship-pipeline-step>` [name step-type state agent]
- `<ship-provider-blob>` [provider size variant]
