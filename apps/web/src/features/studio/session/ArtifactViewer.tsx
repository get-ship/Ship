// Viewer for non-HTML artifacts: markdown (tiptap editor),
// images (click-to-zoom + annotation), JSON, text, and URL artifacts.

import { useMemo, useState, useCallback, lazy, Suspense } from 'react'
import { File, Save, ChevronDown, ChevronRight, MonitorPlay, ExternalLink } from 'lucide-react'
import type { FrontmatterEntry } from '@ship/primitives'
import { MarkdownPreview } from '#/features/studio/skills-ide/MarkdownPreview'
import { ImageViewer } from './ImageViewer'
import { SpecRenderer } from './SpecRenderer'
import { AdrRenderer } from './AdrRenderer'
import { classifyArtifact } from './classify-artifact'
import { daemonFetch } from '#/lib/daemon-config'
import type { SessionFile, ClickAnnotation } from './types'

const MarkdownEditor = lazy(() =>
  import('@ship/primitives').then((m) => ({ default: m.MarkdownEditor }))
)

interface ArtifactViewerProps {
  file: SessionFile
  content: string
  draftContent?: string
  isDirty?: boolean
  onContentChange?: (path: string, content: string) => void
  onSave?: (path: string) => void
  onComment?: (selectedText: string, comment: string) => void
  annotationMode?: boolean
  annotations?: ClickAnnotation[]
  activeAnnotationId?: string | null
  onAddAnnotation?: (x: number, y: number, note: string) => void
  onAnnotationClick?: (id: string) => void
  onRemoveAnnotation?: (id: string) => void
}

// ── Frontmatter utilities ──

/** Replace or insert a key: value line within the frontmatter block of a markdown document. */
function patchFrontmatterInContent(content: string, key: string, value: string): string {
  const FM_RE = /^[\s\uFEFF\xA0]*(\+\+\+|---)[^\n]*\n([\s\S]*?)\n[ \t]*\1[^\n]*(\n|$)/
  const m = FM_RE.exec(content)
  if (!m) return content
  const delim = m[1]
  const rawFm = m[2]
  const rest = content.slice(m[0].length)
  const lines = rawFm.split('\n')
  const keyRe = new RegExp(`^${key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*:`)
  const idx = lines.findIndex((l) => keyRe.test(l))
  if (idx >= 0) {
    if (value === '') lines.splice(idx, 1)
    else lines[idx] = `${key}: ${value}`
  } else if (value !== '') {
    lines.push(`${key}: ${value}`)
  }
  return `${delim}\n${lines.join('\n')}\n${delim}\n${rest}`
}

// ── Editable Frontmatter Bar ──

function FrontmatterBar({
  entries,
  onUpdateEntry,
}: {
  entries: FrontmatterEntry[]
  onUpdateEntry?: (key: string, value: string) => void
}) {
  const [expanded, setExpanded] = useState(false)
  if (entries.length === 0) return null
  return (
    <div className="border-b border-border/60 bg-muted/30 shrink-0">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1.5 px-4 py-1 text-[11px] text-muted-foreground hover:text-foreground transition-colors w-full text-left"
      >
        {expanded ? <ChevronDown className="size-3" /> : <ChevronRight className="size-3" />}
        <span className="font-medium">Frontmatter</span>
        <span className="text-muted-foreground/60">
          ({entries.length} {entries.length === 1 ? 'field' : 'fields'})
        </span>
      </button>
      {expanded && (
        <div className="px-4 pb-2 space-y-1">
          {entries.map((entry) => (
            <div key={entry.key} className="flex gap-2 text-[11px] items-baseline">
              <span className="text-muted-foreground font-mono shrink-0 pt-1">{entry.key}:</span>
              {onUpdateEntry ? (
                <input
                  className="flex-1 bg-transparent border-b border-border/40 focus:border-primary/60 outline-none text-foreground/80 py-0.5 min-w-0"
                  defaultValue={entry.value.replace(/^['"]|['"]$/g, '')}
                  onBlur={(e) => onUpdateEntry(entry.key, e.currentTarget.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur() }}
                />
              ) : (
                <span className="text-foreground/80 truncate">{entry.value}</span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

// ── Markdown editor (tiptap) with inline frontmatter editing ──

function MarkdownFileEditor({
  file, content, draftContent, isDirty, onContentChange, onSave, onComment,
}: {
  file: SessionFile
  content: string
  draftContent?: string
  isDirty?: boolean
  onContentChange?: (path: string, content: string) => void
  onSave?: (path: string) => void
  onComment?: (selectedText: string, comment: string) => void
}) {
  const editable = onContentChange != null
  const displayContent = draftContent ?? content
  const [fmEntries, setFmEntries] = useState<FrontmatterEntry[]>([])

  const handleUpdateFrontmatter = useCallback((key: string, value: string) => {
    if (!onContentChange) return
    onContentChange(file.path, patchFrontmatterInContent(displayContent, key, value))
  }, [displayContent, file.path, onContentChange])

  if (!editable) {
    return (
      <div className="flex flex-col h-full">
        <FrontmatterBar entries={fmEntries} />
        <div className="flex-1 overflow-auto">
          <MarkdownPreview content={displayContent} />
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {isDirty && onSave && (
        <div className="flex items-center justify-end px-4 py-1 border-b border-border/60 shrink-0 bg-card/30">
          <button
            onClick={() => onSave(file.path)}
            className="flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium text-primary hover:bg-primary/10 transition"
          >
            <Save className="size-3" />
            Save
          </button>
        </div>
      )}
      <FrontmatterBar entries={fmEntries} onUpdateEntry={handleUpdateFrontmatter} />
      <div className="flex-1 overflow-hidden">
        <Suspense fallback={<div className="p-4 text-xs text-muted-foreground">Loading editor...</div>}>
          <MarkdownEditor
            value={displayContent}
            onChange={(v) => onContentChange(file.path, v)}
            fillHeight
            hideChrome
            onComment={onComment}
            onFrontmatterParsed={(entries) => setFmEntries(entries)}
          />
        </Suspense>
      </div>
    </div>
  )
}

// ── JSON viewer ──

function JsonViewer({ content, file }: { content: string; file: SessionFile }) {
  const formatted = useMemo(() => {
    try { return JSON.stringify(JSON.parse(content), null, 2) }
    catch { return content }
  }, [content])

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <File className="size-3.5 text-muted-foreground" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
      </div>
      <pre className="flex-1 px-6 py-4 text-xs font-mono text-foreground leading-relaxed overflow-auto whitespace-pre-wrap break-words">
        {formatted}
      </pre>
    </div>
  )
}

// ── Text viewer ──

function TextViewer({ content, file }: { content: string; file: SessionFile }) {
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <File className="size-3.5 text-muted-foreground" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
      </div>
      <pre className="flex-1 px-6 py-4 text-xs font-mono text-foreground leading-relaxed overflow-auto whitespace-pre-wrap break-words">
        {content}
      </pre>
    </div>
  )
}

// ── URL viewer — attempts to launch the underlying service ──

async function tryLaunchUrl(url: string): Promise<void> {
  try {
    const res = await daemonFetch(`/api/runtime/open-url`, {
      method: 'POST',
      body: JSON.stringify({ url }),
    })
    if (res.ok) return
  } catch { /* daemon not available */ }
  window.open(url, '_blank', 'noopener,noreferrer')
}

function UrlViewer({ file, content }: { file: SessionFile; content: string }) {
  const url = content.trim()

  if (!url) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 p-8">
        <MonitorPlay className="size-12 text-muted-foreground/60" />
        <p className="text-sm text-muted-foreground">No URL</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <MonitorPlay className="size-3.5 text-indigo-500" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
        <div className="flex-1" />
        <button
          onClick={() => tryLaunchUrl(url)}
          className="flex items-center gap-1 rounded px-2 py-0.5 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted transition"
        >
          <ExternalLink className="size-3" />
          Open
        </button>
      </div>
      <iframe
        src={url}
        className="flex-1 w-full border-0"
        title={file.name}
      />
    </div>
  )
}

// ── Router ──

export function ArtifactViewer({
  file, content, draftContent, isDirty, onContentChange, onSave, onComment,
  annotationMode, annotations, activeAnnotationId,
  onAddAnnotation, onAnnotationClick, onRemoveAnnotation,
}: ArtifactViewerProps) {
  if (file.type === 'image') {
    return (
      <ImageViewer
        file={file}
        content={content}
        annotationMode={annotationMode}
        annotations={annotations}
        activeAnnotationId={activeAnnotationId}
        onAddAnnotation={onAddAnnotation}
        onAnnotationClick={onAnnotationClick}
        onRemoveAnnotation={onRemoveAnnotation}
      />
    )
  }
  if (file.type === 'url') return <UrlViewer file={file} content={content} />

  // Route typed markdown artifacts to specialized renderers
  const kind = classifyArtifact(file, content)
  if (kind === 'spec') {
    return (
      <SpecRenderer
        file={file}
        content={content}
        isDirty={isDirty}
        onContentChange={onContentChange}
        onSave={onSave}
      />
    )
  }
  if (kind === 'adr') {
    return (
      <AdrRenderer
        file={file}
        content={content}
        isDirty={isDirty}
        onContentChange={onContentChange}
        onSave={onSave}
      />
    )
  }

  if (file.type === 'markdown') {
    return (
      <MarkdownFileEditor
        file={file}
        content={content}
        draftContent={draftContent}
        isDirty={isDirty}
        onContentChange={onContentChange}
        onSave={onSave}
        onComment={onComment}
      />
    )
  }
  if (file.name.endsWith('.json')) return <JsonViewer content={content} file={file} />
  return <TextViewer content={content} file={file} />
}
