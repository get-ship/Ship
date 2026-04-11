import type { SessionFile } from './types'

export type ArtifactKind = 'spec' | 'adr' | 'generic'

const YAML_FM = /^[\s\uFEFF\xA0]*---[^\n]*\n([\s\S]*?)\n[ \t]*---/
const TOML_FM = /^[\s\uFEFF\xA0]*\+\+\+[^\n]*\n([\s\S]*?)\n[ \t]*\+\+\+/

function extractTypeFromFrontmatter(content: string): string | null {
  const yamlMatch = YAML_FM.exec(content)
  if (yamlMatch) {
    const m = /^type:\s*["']?(\w+)["']?\s*$/im.exec(yamlMatch[1])
    return m ? m[1].toLowerCase() : null
  }
  const tomlMatch = TOML_FM.exec(content)
  if (tomlMatch) {
    const m = /^type\s*=\s*["'](\w+)["']\s*$/im.exec(tomlMatch[1])
    return m ? m[1].toLowerCase() : null
  }
  return null
}

function inferTypeFromFilename(name: string): ArtifactKind | null {
  if (/\S+-spec\.md$/i.test(name)) return 'spec'
  if (/^adr-\S+\.md$/i.test(name)) return 'adr'
  return null
}

export function classifyArtifact(file: SessionFile, content: string): ArtifactKind {
  const fmType = extractTypeFromFrontmatter(content)
  if (fmType === 'spec') return 'spec'
  if (fmType === 'adr') return 'adr'
  if (fmType) return 'generic'
  return inferTypeFromFilename(file.name) ?? 'generic'
}
