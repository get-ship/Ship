import { ChevronRight, ChevronDown, FileText, FileJson, BookOpen, FolderOpen, FlaskConical, Settings2, Terminal } from 'lucide-react'
import type { LibrarySkill } from './useSkillsLibrary'
import { makeTabId, SKILL_MD } from './useSkillsIDE'
import type { ContextMenuState } from './SkillContextMenu'

export interface DirNode {
  dirs: Record<string, DirNode>
  files: string[]
}

export function buildTree(paths: string[]): DirNode {
  const root: DirNode = { dirs: {}, files: [] }
  for (const p of paths) {
    const parts = p.split('/')
    if (parts.length === 1) {
      root.files.push(p)
    } else {
      let node = root
      for (let i = 0; i < parts.length - 1; i++) {
        node.dirs[parts[i]] ??= { dirs: {}, files: [] }
        node = node.dirs[parts[i]]
      }
      node.files.push(p)
    }
  }
  return root
}

function fileIcon(path: string) {
  if (path === SKILL_MD) return <FileText className="size-3 shrink-0 text-sky-400" />
  if (path.endsWith('.json')) return <FileJson className="size-3 shrink-0 text-amber-400" />
  if (path.endsWith('.md')) return <BookOpen className="size-3 shrink-0 text-emerald-400" />
  return <FileText className="size-3 shrink-0 text-muted-foreground" />
}

function dirIcon(name: string) {
  switch (name) {
    case 'assets': return <Settings2 className="size-2.5 text-muted-foreground" />
    case 'references': return <BookOpen className="size-2.5 text-muted-foreground" />
    case 'evals': return <FlaskConical className="size-2.5 text-muted-foreground" />
    case 'scripts': return <Terminal className="size-2.5 text-muted-foreground" />
    default: return <FolderOpen className="size-2.5 text-muted-foreground" />
  }
}

export function DirTree({ dirPath, node, skill, skillId, activeTabId, depth, expandedDirs, onToggleDir, onOpenFile, onContextMenu }: {
  dirPath: string; node: DirNode; skill: LibrarySkill; skillId: string
  activeTabId: string | null; depth: number
  expandedDirs: Set<string>; onToggleDir: (key: string) => void
  onOpenFile: (sid: string, fp: string) => void
  onContextMenu: (state: ContextMenuState) => void
}) {
  const pad = 36 + depth * 12
  const sortedDirs = Object.keys(node.dirs).sort()
  return (
    <>
      {node.files.map((f) => (
        <FileEntry key={f} skill={skill} skillId={skillId} filePath={f}
          activeTabId={activeTabId} padLeft={pad}
          onOpenFile={onOpenFile} onContextMenu={onContextMenu} />
      ))}
      {sortedDirs.map((name) => {
        const subPath = dirPath ? `${dirPath}/${name}` : name
        const key = `${skillId}:${subPath}`
        const isOpen = expandedDirs.has(key)
        return (
          <div key={name}>
            <button
              onClick={() => onToggleDir(key)}
              style={{ paddingLeft: pad }}
              className="flex w-full items-center gap-1.5 pr-3 py-0.5 text-[10px] text-muted-foreground font-medium hover:text-foreground transition-colors"
            >
              {isOpen ? <ChevronDown className="size-2.5 shrink-0" /> : <ChevronRight className="size-2.5 shrink-0" />}
              {dirIcon(name)}
              {name}/
            </button>
            {isOpen && (
              <DirTree dirPath={subPath} node={node.dirs[name]} skill={skill} skillId={skillId}
                activeTabId={activeTabId} depth={depth + 1}
                expandedDirs={expandedDirs} onToggleDir={onToggleDir}
                onOpenFile={onOpenFile} onContextMenu={onContextMenu} />
            )}
          </div>
        )
      })}
    </>
  )
}

function FileEntry({ skill, skillId, filePath, activeTabId, padLeft, onOpenFile, onContextMenu }: {
  skill: LibrarySkill; skillId: string; filePath: string; activeTabId: string | null; padLeft: number
  onOpenFile: (sid: string, fp: string) => void
  onContextMenu: (state: ContextMenuState) => void
}) {
  const tabId = makeTabId(skillId, filePath)
  const isActive = activeTabId === tabId
  return (
    <button
      onClick={() => onOpenFile(skillId, filePath)}
      onContextMenu={(e) => {
        e.preventDefault()
        onContextMenu({ mode: 'file', skill, filePath, x: e.clientX, y: e.clientY })
      }}
      style={{ paddingLeft: padLeft }}
      className={`flex w-full items-center gap-1.5 pr-3 py-1 text-xs transition-colors border-l-2 ${isActive ? 'border-primary bg-primary/5 text-primary' : 'border-transparent text-foreground/80 hover:text-foreground hover:bg-muted/30'}`}
    >
      {fileIcon(filePath)}
      <span className="truncate">{filePath.split('/').pop() ?? filePath}</span>
    </button>
  )
}
