import { Save } from 'lucide-react'
import { MarkdownEditor } from '@ship/primitives'
import type { SessionFile } from './types'
import { parseFrontmatter, parseArray } from './parse-frontmatter'

interface AdrRendererProps {
  file: SessionFile
  content: string
  isDirty?: boolean
  onContentChange?: (path: string, content: string) => void
  onSave?: (path: string) => void
}

export function AdrRenderer({ file, content, isDirty, onContentChange, onSave }: AdrRendererProps) {
  const { meta, body } = parseFrontmatter(content)
  const title = meta.title
  const status = meta.status
  const authors = parseArray(meta.authors)
  const date = meta.date
  const supersedes = meta.supersedes
  const supersededBy = meta.superseded_by
  const tags = parseArray(meta.tags)

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="shrink-0 border-b border-border/60 px-6 py-4 space-y-3">
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-lg font-semibold text-foreground">{title}</h1>
          {isDirty && onSave && (
            <button
              onClick={() => onSave(file.path)}
              className="flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium text-primary hover:bg-primary/10 transition"
            >
              <Save className="size-3" />
              Save
            </button>
          )}
        </div>

        {status && (
          <span data-testid="status-badge" className="inline-block rounded-full bg-muted px-2 py-0.5 text-[11px] font-medium">
            {status}
          </span>
        )}

        <div className="flex flex-wrap gap-x-6 gap-y-1 text-[11px] text-muted-foreground">
          {authors.length > 0 && (
            <span>Authors: {authors.join(', ')}</span>
          )}
          {date && (
            <span data-testid="adr-date">Date: {date}</span>
          )}
          {supersedes && (
            <span>Supersedes: {supersedes}</span>
          )}
          {supersededBy && (
            <span>Superseded by: {supersededBy}</span>
          )}
        </div>

        {tags.length > 0 && (
          <div className="flex gap-1.5 flex-wrap">
            {tags.map((tag) => (
              <span key={tag} className="rounded bg-muted/60 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-auto">
        <MarkdownEditor
          value={body}
          onChange={onContentChange ? (v: string) => onContentChange(file.path, v) : undefined}
        />
      </div>
    </div>
  )
}
