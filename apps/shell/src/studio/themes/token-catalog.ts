// Curated, categorized token catalog for the theme detail editor.
//
// Mirrors the editor categories in theme-detail.html (Foundation, Tone,
// Typography, Motion, Radii, Effects). Token names match the canonical
// spec in crates/sui/compiler/tokens.json + tokens.css. The `type`
// drives the row layout: `color` rows show a swatch; `length` rows are
// compact numeric inputs; everything else is a wide text input. Shadow
// rows additionally render a rendered box-shadow preview.

export type TokenType = 'color' | 'length' | 'duration' | 'number' | 'font' | 'easing' | 'shadow'

export interface TokenDef {
  name: string
  type: TokenType
}

export interface TokenCategory {
  id: string
  label: string
  tokens: TokenDef[]
}

export const CATEGORIES: ReadonlyArray<TokenCategory> = [
  {
    id: 'foundation',
    label: 'Foundation',
    tokens: [
      { name: '--background', type: 'color' },
      { name: '--foreground', type: 'color' },
      { name: '--card', type: 'color' },
      { name: '--card-foreground', type: 'color' },
      { name: '--muted', type: 'color' },
      { name: '--muted-foreground', type: 'color' },
      { name: '--border', type: 'color' },
      { name: '--border-subtle', type: 'color' },
      { name: '--primary', type: 'color' },
      { name: '--primary-foreground', type: 'color' },
      { name: '--accent', type: 'color' },
      { name: '--accent-foreground', type: 'color' },
      { name: '--ring', type: 'color' },
    ],
  },
  {
    id: 'tone',
    label: 'Tone',
    tokens: [
      { name: '--info', type: 'color' },
      { name: '--success', type: 'color' },
      { name: '--warning', type: 'color' },
      { name: '--destructive', type: 'color' },
    ],
  },
  {
    id: 'typography',
    label: 'Typography',
    tokens: [
      { name: '--font-body', type: 'font' },
      { name: '--heading-font', type: 'font' },
      { name: '--font-mono', type: 'font' },
      { name: '--heading-weight', type: 'number' },
      { name: '--heading-tracking', type: 'length' },
    ],
  },
  {
    id: 'motion',
    label: 'Motion',
    tokens: [
      { name: '--duration-fast', type: 'duration' },
      { name: '--duration-medium', type: 'duration' },
      { name: '--duration-slow', type: 'duration' },
      { name: '--ease-out', type: 'easing' },
      { name: '--ease-out-soft', type: 'easing' },
    ],
  },
  {
    id: 'radii',
    label: 'Radii',
    tokens: [
      { name: '--radius-sm', type: 'length' },
      { name: '--radius-md', type: 'length' },
      { name: '--radius-lg', type: 'length' },
      { name: '--radius-xl', type: 'length' },
    ],
  },
  {
    id: 'effects',
    label: 'Effects',
    tokens: [
      { name: '--shadow-sm', type: 'shadow' },
      { name: '--shadow-md', type: 'shadow' },
      { name: '--shadow-hover', type: 'shadow' },
      { name: '--shadow-accent', type: 'shadow' },
    ],
  },
]

export const TAB_IDS: ReadonlyArray<string> = ['all', ...CATEGORIES.map((c) => c.id)]

export const ALL_TOKENS: ReadonlyArray<TokenDef> = CATEGORIES.flatMap((c) => c.tokens)

export const TOKEN_NAMES: ReadonlyArray<string> = ALL_TOKENS.map((t) => t.name)

const TYPE_BY_NAME: Record<string, TokenType> = Object.fromEntries(
  ALL_TOKENS.map((t) => [t.name, t.type]),
)

export const tokenType = (name: string): TokenType => TYPE_BY_NAME[name] ?? 'font'
