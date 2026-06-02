// Language id → CM LanguageSupport factory. Lazy: each factory pulls from the
// vendored CM6 lang packs only when a language is actually requested. Unknown
// id → null (plain text, no language extension).
import {
  javascript,
  json,
  css,
  html,
  markdown,
  rust,
  python,
  sql,
  yaml,
  type LanguageSupport,
} from '../vendor/codemirror/codemirror.mjs'

type LangFactory = () => LanguageSupport

const REGISTRY: Record<string, LangFactory> = {
  javascript: () => javascript(),
  js: () => javascript(),
  jsx: () => javascript({ jsx: true }),
  typescript: () => javascript({ typescript: true }),
  ts: () => javascript({ typescript: true }),
  tsx: () => javascript({ jsx: true, typescript: true }),
  json: () => json(),
  css: () => css(),
  html: () => html(),
  markdown: () => markdown(),
  md: () => markdown(),
  rust: () => rust(),
  rs: () => rust(),
  python: () => python(),
  py: () => python(),
  sql: () => sql(),
  yaml: () => yaml(),
  yml: () => yaml(),
}

/** Canonical language ids (and aliases) this pack understands. */
export const languageIds: readonly string[] = Object.keys(REGISTRY)

/** Resolve a language id to its CM LanguageSupport, or null for plain text. */
export function resolveLanguage(id: string | undefined): LanguageSupport | null {
  if (!id) return null
  const factory = REGISTRY[id.toLowerCase()]
  return factory ? factory() : null
}
