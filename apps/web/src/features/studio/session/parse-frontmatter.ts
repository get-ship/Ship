const YAML_FM = /^[\s\uFEFF\xA0]*---[^\n]*\n([\s\S]*?)\n[ \t]*---[^\n]*(\n|$)/
const TOML_FM = /^[\s\uFEFF\xA0]*\+\+\+[^\n]*\n([\s\S]*?)\n[ \t]*\+\+\+[^\n]*(\n|$)/

/** Minimal frontmatter parser — extracts key/value pairs and body. */
export function parseFrontmatter(content: string): {
  meta: Record<string, string>
  body: string
} {
  const meta: Record<string, string> = {}
  let body = content

  const yamlMatch = YAML_FM.exec(content)
  if (yamlMatch) {
    parseYamlLines(yamlMatch[1], meta)
    body = content.slice(yamlMatch[0].length)
  } else {
    const tomlMatch = TOML_FM.exec(content)
    if (tomlMatch) {
      parseTomlLines(tomlMatch[1], meta)
      body = content.slice(tomlMatch[0].length)
    }
  }

  return { meta, body: body.trimStart() }
}

function parseYamlLines(raw: string, out: Record<string, string>) {
  for (const line of raw.split('\n')) {
    const m = /^(\w[\w_]*):\s*(.*)$/.exec(line.trim())
    if (m) out[m[1]] = m[2].replace(/^["']|["']$/g, '')
  }
}

function parseTomlLines(raw: string, out: Record<string, string>) {
  for (const line of raw.split('\n')) {
    const m = /^(\w[\w_]*)\s*=\s*(.*)$/.exec(line.trim())
    if (m) out[m[1]] = m[2].replace(/^["']|["']$/g, '')
  }
}

/** Parse a YAML-style array value like ["a", "b"] into string[]. */
export function parseArray(value: string | undefined): string[] {
  if (!value) return []
  const m = value.match(/\[([^\]]*)\]/)
  if (!m) return value ? [value] : []
  return m[1]
    .split(',')
    .map((s) => s.trim().replace(/^["']|["']$/g, ''))
    .filter(Boolean)
}
