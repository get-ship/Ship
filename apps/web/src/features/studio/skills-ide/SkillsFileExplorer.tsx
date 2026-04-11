import { useState, useMemo, useCallback } from 'react'
import {
  Search, Plus, ChevronRight, ChevronDown, Zap,
  FolderOpen, Library, ChevronsDownUp,
} from 'lucide-react'
import type { Skill } from '@ship/ui'
import type { LibrarySkill } from './useSkillsLibrary'
import { SKILL_MD } from './useSkillsIDE'
import { SkillContextMenu, type ContextMenuState } from './SkillContextMenu'
import { SkillFilterBar, applyFilters, type SkillFilter } from './SkillFilters'
import { buildTree, DirTree } from './FileTree'

interface Props {
  filteredSkills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  isLoading: boolean
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onSearchChange: (q: string) => void
  onToggleFolder: (id: string) => void
  onCollapseAll: () => void
  onOpenFile: (skillId: string, filePath: string) => void
  onAddFile: (skillId: string, filePath: string, content: string) => void
  onDeleteFile: (skillId: string, filePath: string) => void
  onCreateSkill: () => void
}

export function SkillsFileExplorer({
  filteredSkills, activeTabId, expandedFolders, searchQuery,
  isLoading, getLibrarySkill,
  onSearchChange, onToggleFolder, onCollapseAll, onOpenFile, onAddFile, onDeleteFile, onCreateSkill,
}: Props) {
  const [activeFilter, setActiveFilter] = useState<SkillFilter>('all')
  const [activeTags, setActiveTags] = useState<Set<string>>(new Set())
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set())

  const handleToggleDir = useCallback((key: string) => {
    setExpandedDirs((prev) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key); else next.add(key)
      return next
    })
  }, [])

  const handleTagToggle = useCallback((tag: string) => {
    setActiveTags((prev) => {
      const next = new Set(prev)
      if (next.has(tag)) next.delete(tag); else next.add(tag)
      return next
    })
  }, [])

  const allLibrarySkills = useMemo(
    () => filteredSkills.map((s) => getLibrarySkill(s.id)).filter(Boolean) as LibrarySkill[],
    [filteredSkills, getLibrarySkill],
  )
  const finalSkills = useMemo(
    () => applyFilters(allLibrarySkills, activeFilter, activeTags),
    [allLibrarySkills, activeFilter, activeTags],
  )

  const projectSkills = finalSkills.filter((s) => s.origin !== 'library')
  const librarySkillsList = finalSkills.filter((s) => s.origin === 'library')

  return (
    <div className="flex w-60 shrink-0 flex-col border-r border-border bg-card/30">
      <div className="p-2.5 border-b border-border space-y-1.5">
        <div className="flex items-center gap-2 rounded-md border border-border bg-background/80 px-2.5 py-1.5 focus-within:border-primary/50 transition-colors">
          <Search className="size-3.5 text-muted-foreground shrink-0" />
          <input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search skills..."
            className="min-w-0 flex-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none"
            spellCheck={false}
          />
          {expandedFolders.size > 0 && (
            <button onClick={onCollapseAll} className="text-muted-foreground hover:text-foreground transition-colors" title="Collapse all">
              <ChevronsDownUp className="size-3.5" />
            </button>
          )}
        </div>
        <SkillFilterBar
          allSkills={allLibrarySkills}
          activeFilter={activeFilter}
          activeTags={activeTags}
          onFilterChange={setActiveFilter}
          onTagToggle={handleTagToggle}
        />
      </div>

      <div className="flex-1 overflow-y-auto">
        {isLoading && filteredSkills.length === 0 ? (
          <div className="p-3 space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-6 w-full animate-pulse rounded bg-muted" />
            ))}
          </div>
        ) : (
          <>
            <SkillSection
              label="Project Skills"
              icon={<FolderOpen className="size-3 text-muted-foreground" />}
              skills={projectSkills}
              activeTabId={activeTabId}
              expandedFolders={expandedFolders}
              expandedDirs={expandedDirs}
              searchQuery={searchQuery}
              getLibrarySkill={getLibrarySkill}
              onToggleFolder={onToggleFolder}
              onToggleDir={handleToggleDir}
              onOpenFile={onOpenFile}
              onContextMenu={setContextMenu}
              action={<button onClick={onCreateSkill} className="text-muted-foreground hover:text-primary transition-colors" title="New skill"><Plus className="size-3.5" /></button>}
            />

            <div className="h-px bg-border mx-3" />

            <SkillSection
              label="Library"
              icon={<Library className="size-3 text-muted-foreground" />}
              skills={librarySkillsList}
              activeTabId={activeTabId}
              expandedFolders={expandedFolders}
              expandedDirs={expandedDirs}
              searchQuery={searchQuery}
              getLibrarySkill={getLibrarySkill}
              onToggleFolder={onToggleFolder}
              onToggleDir={handleToggleDir}
              onOpenFile={onOpenFile}
              onContextMenu={setContextMenu}
            />
          </>
        )}
      </div>

      {contextMenu && (
        <SkillContextMenu
          menu={contextMenu}
          onAddFile={onAddFile}
          onDeleteFile={onDeleteFile}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  )
}

interface SectionProps {
  label: string; icon: React.ReactNode; skills: Skill[]; activeTabId: string | null
  expandedFolders: Set<string>; expandedDirs: Set<string>; searchQuery: string; action?: React.ReactNode
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onToggleFolder: (id: string) => void
  onToggleDir: (key: string) => void
  onOpenFile: (skillId: string, filePath: string) => void
  onContextMenu: (state: ContextMenuState) => void
}

function SkillSection({
  label, icon, skills, activeTabId, expandedFolders, expandedDirs, searchQuery,
  getLibrarySkill, onToggleFolder, onToggleDir, onOpenFile, onContextMenu, action,
}: SectionProps) {
  return (
    <div className="py-1.5">
      <div className="flex items-center justify-between px-3.5 py-1">
        <span className="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground">
          {icon}
          {label}
          <span className="text-muted-foreground font-normal normal-case">
            ({skills.length})
          </span>
        </span>
        {action}
      </div>

      {skills.length === 0 && (
        <p className="px-4 py-3 text-[11px] italic text-muted-foreground">
          {searchQuery ? 'No matches.' : 'No skills yet.'}
        </p>
      )}

      {skills.map((skill) => {
        const expanded = expandedFolders.has(skill.id)
        const ls = getLibrarySkill(skill.id)
        const originLabel = ls?.origin === 'library' ? 'library' : null
        const files = ls?.files ?? [SKILL_MD]
        const hasVars = ls?.varsSchema != null
        const hasEvals = ls?.evals != null
        const hasRefs = Object.keys(ls?.referenceDocs ?? {}).length > 0
        const tree = buildTree(files)

        return (
          <div key={skill.id}>
            <button
              onClick={() => onToggleFolder(skill.id)}
              onContextMenu={(e) => {
                if (!ls) return
                e.preventDefault()
                onContextMenu({ mode: 'folder', skill: ls, x: e.clientX, y: e.clientY })
              }}
              className="flex w-full items-center gap-1.5 px-3 py-1 text-xs text-foreground/80 hover:text-foreground transition-colors"
            >
              {expanded ? <ChevronDown className="size-3 shrink-0" /> : <ChevronRight className="size-3 shrink-0" />}
              <Zap className="size-3 shrink-0 text-primary/80" />
              <span className="truncate font-medium">{skill.name || skill.id}</span>
              <span className="ml-auto flex items-center gap-1">
                {hasVars && <span className="size-1.5 rounded-full bg-amber-400" title="Has variables" />}
                {hasRefs && <span className="size-1.5 rounded-full bg-emerald-400" title="Has docs" />}
                {hasEvals && <span className="size-1.5 rounded-full bg-violet-400" title="Has evals" />}
              </span>
              {originLabel && (
                <span className="text-[9px] text-violet-400 bg-violet-500/15 px-1.5 rounded">{originLabel}</span>
              )}
            </button>
            {expanded && ls && (
              <div className="pb-1">
                <DirTree
                  dirPath="" node={tree} skill={ls} skillId={skill.id}
                  activeTabId={activeTabId} depth={0}
                  expandedDirs={expandedDirs} onToggleDir={onToggleDir}
                  onOpenFile={onOpenFile} onContextMenu={onContextMenu}
                />
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}
