// Curated token catalog for the Theme Studio editor.
//
// Sourced from crates/sui/compiler/tokens.json (the canonical
// machine-readable spec). Re-stating the relevant subset inline avoids a
// runtime JSON fetch or a cross-package import that the view shell doesn't
// resolve. The four layers mirror packages/components/ui-components/src/tokens/tokens.css
// and the ship-design-system skill's layer table; "Palette" sits between
// Layer 1 (Surfaces) and Layer 4 (Primitives) per that skill.

export type TokenType = 'color' | 'length' | 'easing' | 'font' | 'shadow' | 'number' | 'duration'

export interface TokenDef {
  name: string
  type: TokenType
  intent: string
}

export interface TokenLayer {
  id: string
  label: string
  blurb: string
  tokens: TokenDef[]
}

export const LAYERS: TokenLayer[] = [
  {
    id: 'palette',
    label: 'Palette',
    blurb: 'Brand colors. Tone ramps rebuild from these bases.',
    tokens: [
      { name: '--background',          type: 'color', intent: 'App background' },
      { name: '--foreground',          type: 'color', intent: 'App text' },
      { name: '--card',                type: 'color', intent: 'Card background' },
      { name: '--card-foreground',     type: 'color', intent: 'Card text' },
      { name: '--primary',             type: 'color', intent: 'Brand primary' },
      { name: '--primary-foreground',  type: 'color', intent: 'On primary' },
      { name: '--accent',              type: 'color', intent: 'Brand accent' },
      { name: '--accent-foreground',   type: 'color', intent: 'On accent' },
      { name: '--muted',               type: 'color', intent: 'Muted surface' },
      { name: '--muted-foreground',    type: 'color', intent: 'Muted text' },
      { name: '--border',              type: 'color', intent: 'Borders' },
      { name: '--ring',                type: 'color', intent: 'Focus ring' },
      { name: '--success',             type: 'color', intent: 'Success base' },
      { name: '--warning',             type: 'color', intent: 'Warning base' },
      { name: '--info',                type: 'color', intent: 'Info base' },
      { name: '--destructive',         type: 'color', intent: 'Destructive base' },
    ],
  },
  {
    id: 'surface',
    label: 'Semantic surfaces & intents',
    blurb: 'Depth and interaction surfaces. Cards read --surface-1, popovers --surface-3.',
    tokens: [
      { name: '--surface-0',     type: 'color', intent: 'App-level surface' },
      { name: '--surface-1',     type: 'color', intent: 'Resting card surface' },
      { name: '--surface-2',     type: 'color', intent: 'Nested panel surface' },
      { name: '--surface-3',     type: 'color', intent: 'Floating/popover surface' },
      { name: '--backdrop',      type: 'color', intent: 'Modal backdrop scrim' },
      { name: '--scrim',         type: 'color', intent: 'Lighter scrim overlay' },
      { name: '--hover-bg',      type: 'color', intent: 'Hover tint' },
      { name: '--selected-bg',   type: 'color', intent: 'Selected tint' },
      { name: '--divider',       type: 'color', intent: 'Divider line' },
    ],
  },
  {
    id: 'tone',
    label: 'Tone ramps',
    blurb: 'Status tones. Read by chips, badges, alerts. Ramps derive from palette bases above.',
    tokens: [
      { name: '--tone-success-fg',     type: 'color', intent: 'Success fg' },
      { name: '--tone-success-bg',     type: 'color', intent: 'Success soft bg' },
      { name: '--tone-success-border', type: 'color', intent: 'Success border' },
      { name: '--tone-warning-fg',     type: 'color', intent: 'Warning fg' },
      { name: '--tone-warning-bg',     type: 'color', intent: 'Warning soft bg' },
      { name: '--tone-destructive-fg', type: 'color', intent: 'Destructive fg' },
      { name: '--tone-destructive-bg', type: 'color', intent: 'Destructive soft bg' },
      { name: '--tone-info-fg',        type: 'color', intent: 'Info fg' },
      { name: '--tone-info-bg',        type: 'color', intent: 'Info soft bg' },
    ],
  },
  {
    id: 'role',
    label: 'Role tokens',
    blurb: 'Typography, motion, density, shadow intent.',
    tokens: [
      { name: '--heading-font',     type: 'font',     intent: 'Heading font stack' },
      { name: '--heading-weight',   type: 'number',   intent: 'Heading weight' },
      { name: '--body-font',        type: 'font',     intent: 'Body font stack' },
      { name: '--label-font',       type: 'font',     intent: 'Label font stack' },
      { name: '--ease-out',         type: 'easing',   intent: 'Default ease' },
      { name: '--ease-out-soft',    type: 'easing',   intent: 'Soft ease' },
      { name: '--density-row-py',   type: 'length',   intent: 'Row vertical padding' },
      { name: '--density-row-px',   type: 'length',   intent: 'Row horizontal padding' },
      { name: '--shadow-hover',     type: 'shadow',   intent: 'Hover shadow' },
      { name: '--shadow-accent',    type: 'shadow',   intent: 'Primary-tinted focus glow' },
    ],
  },
  {
    id: 'primitive',
    label: 'Primitives',
    blurb: 'Raw scale. Rarely the right override point.',
    tokens: [
      { name: '--space-1',         type: 'length',   intent: '4px' },
      { name: '--space-2',         type: 'length',   intent: '8px' },
      { name: '--space-3',         type: 'length',   intent: '12px' },
      { name: '--space-4',         type: 'length',   intent: '16px' },
      { name: '--radius-sm',       type: 'length',   intent: 'Small radius' },
      { name: '--radius-md',       type: 'length',   intent: 'Medium radius' },
      { name: '--radius-lg',       type: 'length',   intent: 'Large radius' },
      { name: '--radius-xl',       type: 'length',   intent: 'XL radius' },
      { name: '--duration-fast',   type: 'duration', intent: 'Fast duration' },
      { name: '--duration-medium', type: 'duration', intent: 'Medium duration' },
      { name: '--duration-slow',   type: 'duration', intent: 'Slow duration' },
    ],
  },
]

export const THEME_PACKS: ReadonlyArray<{ id: string; label: string }> = [
  { id: 'ship-light', label: 'SHIP Light' },
  { id: 'ship-dark',  label: 'SHIP Dark' },
  { id: 'paper',      label: 'Paper' },
  { id: 'halo',       label: 'Halo (Soft)' },
  { id: 'console',    label: 'Console' },
  { id: 'contrast',   label: 'High Contrast' },
]

export const ALL_TOKENS: TokenDef[] = LAYERS.flatMap((l) => l.tokens)
