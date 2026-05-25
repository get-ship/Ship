// <sui-pipeline-trigger> — derives the kind label so the template renders it
// as real DOM text. The glyph stays a decorative ::before symbol.
import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-trigger.sui'

const read = (v: unknown): string => (typeof v === 'function' ? (v as () => unknown)() : v) as string

export default controller<Scope>({
  derived: {
    kindLabel: ({ kind }) => read(kind),
  },
})
