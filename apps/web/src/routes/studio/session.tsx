import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { daemonFetch } from '#/lib/daemon-config'
import type { ClickAnnotation } from '#/features/studio/session/types'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { SessionSidebar } from '#/features/studio/session/SessionSidebar'
import { useSessionFiles, useSessionFileContent, useUploadSessionFile, useDeleteSessionFile } from '#/features/studio/session/useSessionFiles'
import { useSessionDrafts } from '#/features/studio/session/useSessionDrafts'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { useSessionHandlers } from '#/features/studio/session/useSessionHandlers'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'
import { DropZoneOverlay } from '#/features/studio/session/DropZoneOverlay'
import { ViewHost } from '#/features/studio/views/ViewHost'
import { usePlatform } from '#/features/studio/PlatformProvider'
import { useGlobalAnnotations } from '#/features/studio/AnnotationContext'
import { SessionEmptyState } from '#/features/studio/session/SessionEmptyState'

export const Route = createFileRoute('/studio/session')({
  component: SessionPage,
  pendingComponent: SessionSkeleton,
  ssr: false,
})

type ViewMode = 'file' | 'diff'
interface OpenTab { path: string; name: string; type: string }

function SessionPage() {
  const { workspaces } = useDaemon()
  const active = workspaces
    .filter((w) => w.status === 'active')
    .sort((a, b) => (b.last_activated_at ?? '').localeCompare(a.last_activated_at ?? ''))
  const activeWs = active[0]
  const workspaceId = activeWs?.branch ?? ''

  const { files } = useSessionFiles()
  const uploadMutation = useUploadSessionFile()
  const deleteMutation = useDeleteSessionFile()
  const { diffText } = useDiffContent()
  useGitStatus() // keep query warm for diff viewer
  useGitLog(5)
  const drafts = useSessionDrafts()

  const [openTabs, setOpenTabs] = useState<OpenTab[]>([])
  const [activeTabPath, setActiveTabPath] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('file')
  const [selectedCommitHash, setSelectedCommitHash] = useState<string | null>(null)
  const [activeView, setActiveView] = useState<string | null>(null)

  const { data: viewHtml } = useQuery({
    queryKey: ['view', activeView],
    queryFn: async () => {
      const res = await daemonFetch(`/api/runtime/view/${activeView}`)
      if (!res.ok) return null
      const body = await res.json() as { ok: boolean; data: { html: string } }
      return body.data.html
    },
    enabled: activeView != null,
    staleTime: 60_000,
  })

  const ann = useAnnotations(activeTabPath)
  const { data: commitDiff } = useGitDiff(
    selectedCommitHash ? `${selectedCommitHash}^..${selectedCommitHash}` : undefined,
  )

  const activeTab = openTabs.find((t) => t.path === activeTabPath) ?? null
  const activeFile = activeTabPath ? files.find((f) => f.path === activeTabPath) : null
  const { data: fileContent } = useSessionFileContent(activeTabPath)

  useEffect(() => {
    if (activeTabPath && fileContent != null) drafts.openFile(activeTabPath, fileContent)
  }, [activeTabPath, fileContent]) // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        if (activeTabPath && drafts.isDirty(activeTabPath)) drafts.saveFile(activeTabPath)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [activeTabPath, drafts])

  const closeTab = useCallback((path: string) => {
    setOpenTabs((prev) => {
      const next = prev.filter((t) => t.path !== path)
      if (activeTabPath === path) setActiveTabPath(next.length > 0 ? next[next.length - 1].path : null)
      return next
    })
  }, [activeTabPath])

  const openFile = useCallback((path: string) => {
    const file = files.find((f) => f.path === path)
    if (!file) return
    setSelectedCommitHash(null)
    setViewMode('file')
    setActiveView(null)
    setActiveTabPath(path)
    setOpenTabs((prev) => {
      if (prev.some((t) => t.path === path)) return prev
      if (activeTabPath) return prev.map((t) => t.path === activeTabPath ? { path, name: file.name, type: file.type } : t)
      return [...prev, { path, name: file.name, type: file.type }]
    })
  }, [files, activeTabPath])

  const selectTab = useCallback((path: string) => {
    setActiveTabPath(path)
    setSelectedCommitHash(null)
    setViewMode('file')
    setActiveView(null)
  }, [])

  // View opening handled by route navigation now
  void setActiveView // keep setter available for future use

  const {
    isDragging, handleDeleteFile, handleExport, handleComment, handleDiffComment,
    handleUploadFiles,
    handleDragOver, handleDragLeave, handleDrop,
  } = useSessionHandlers({
    workspaceId,
    ann, openFile, closeTab,
    setViewMode, setSelectedCommitHash,
    uploadMutate: uploadMutation.mutate,
    deleteMutate: deleteMutation.mutate,
  })

  const isHtml = activeTab?.type === 'html'
  const showView = activeView != null && viewHtml != null
  const showCanvas = !showView && viewMode === 'file' && isHtml
  const showArtifact = !showView && viewMode === 'file' && activeFile != null && !isHtml
  const showDiff = !showView && viewMode === 'diff'
  const activeDiffText = selectedCommitHash ? commitDiff : diffText

  const { sidebarOpen: fileSidebarOpen, drawerOpen, toggleDrawer } = usePlatform()
  const globalAnn = useGlobalAnnotations()

  return (
    <>
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40">
          <img src="/ship-logos/ship_logo.svg" alt="" className="size-5 opacity-60" />
        </div>
        <p className="font-display text-base font-semibold">Best on desktop</p>
        <p className="text-sm text-muted-foreground max-w-xs">
          Open on a wider screen for the full experience.
        </p>
      </div>

      <div
        className="hidden md:flex flex-1 h-full min-h-0 overflow-hidden"
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {/* Docked file sidebar — Cmd+B toggles */}
        {fileSidebarOpen && (
          <div className="w-60 shrink-0 border-r border-border/60 bg-card/30 flex flex-col min-h-0 overflow-hidden">
            <SessionSidebar
              files={files}
              activeFile={activeTabPath}
              onSelectFile={openFile}
              onDeleteFile={handleDeleteFile}
              onUploadFiles={handleUploadFiles}
            />
          </div>
        )}

        <div className="flex flex-1 min-w-0 min-h-0 flex-col">
          <div className="flex flex-1 min-w-0 min-h-0">
            <div className="flex-1 flex flex-col min-w-0 min-h-0">
                {showView && (
                  <ViewHost html={viewHtml} workspaceId={workspaceId} />
                )}
                {showCanvas && (
                  <SessionCanvas
                    key={activeTabPath}
                    htmlContent={fileContent ?? ''}
                    fileType={activeFile?.type}
                    annotations={ann.annotations}
                    activeId={ann.activeId}
                    annotationMode={ann.annotationMode}
                    openTabs={activeTabPath ? [activeTabPath] : []}
                    activeTab={activeTabPath}
                    onTabSelect={selectTab}
                    onTabClose={closeTab}
                    onAnnotationClick={ann.toggleActiveId}
                    onDismissActive={ann.dismissActive}
                    onRemoveAnnotation={ann.removeAnnotation}
                    onAddClick={ann.addClickAnnotation}
                    onAddBox={ann.addBoxAnnotation}
                    onAddAction={ann.addActionAnnotation}
                    onExport={handleExport}
                  />
                )}
                {showArtifact && activeFile && (
                  <ArtifactViewer
                    file={activeFile}
                    content={fileContent ?? ''}
                    draftContent={drafts.getDraft(activeFile.path)}
                    isDirty={drafts.isDirty(activeFile.path)}
                    onContentChange={drafts.updateContent}
                    onSave={drafts.saveFile}
                    onComment={handleComment}
                    annotationMode={ann.annotationMode}
                    annotations={ann.annotations.filter((a): a is ClickAnnotation => a.type === 'click')}
                    activeAnnotationId={ann.activeId}
                    onAddAnnotation={(x, y, note) => ann.addClickAnnotation('', '', note, x, y)}
                    onAnnotationClick={ann.toggleActiveId}
                    onRemoveAnnotation={ann.removeAnnotation}
                  />
                )}
                {showDiff && (
                  activeDiffText
                    ? <DiffViewer diffText={activeDiffText} onComment={handleDiffComment} />
                    : (
                      <div className="flex items-center justify-center h-full text-muted-foreground">
                        <div className="text-center">
                          <p className="text-sm font-medium">No diff available</p>
                          <p className="text-xs mt-1">
                            {selectedCommitHash ? `Loading ${selectedCommitHash.slice(0, 7)}...` : 'No changes to show'}
                          </p>
                        </div>
                      </div>
                    )
                )}
                {!showView && !showCanvas && !showArtifact && !showDiff && (
                  <SessionEmptyState
                    workspaceName={activeWs?.branch ?? null}
                    activeAgent={activeWs?.active_agent ?? null}
                    files={files}
                    onSelectFile={openFile}
                  />
                )}
            </div>
          </div>
        </div>

        {isDragging && <DropZoneOverlay />}

        {/* Annotation FAB — bottom center, visible on canvas/artifact/diff views */}
        {(showCanvas || showArtifact || showDiff) && (
          <div className="absolute bottom-4 left-1/2 -translate-x-1/2 z-20 flex items-center gap-1.5 rounded-full border border-border/60 bg-card/90 backdrop-blur-sm shadow-lg px-2 py-1">
            <button
              onClick={() => ann.setAnnotationMode(!ann.annotationMode)}
              className={`flex items-center gap-1.5 rounded-full px-3 py-1.5 text-[11px] font-medium transition ${
                ann.annotationMode
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
              }`}
            >
              {ann.annotationMode ? 'Annotating' : 'Annotate'}
            </button>
            {ann.annotations.length > 0 && (
              <>
                <span className="text-[10px] text-muted-foreground tabular-nums">
                  {ann.annotations.length}
                </span>
                <button
                  onClick={() => {
                    // Stage all current annotations into the global context for the agent chat
                    for (const a of ann.allStaged) {
                      globalAnn.addAnnotation(a.filePath, a.ann)
                    }
                    // Always open — never close when sending annotations
                    if (!drawerOpen) toggleDrawer()
                  }}
                  className="rounded-full px-2 py-1 text-[10px] text-primary hover:bg-primary/10 transition font-medium"
                >
                  Send to Agent
                </button>
                <button
                  onClick={ann.clearAnnotations}
                  className="rounded-full px-2 py-1 text-[10px] text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition"
                >
                  Clear
                </button>
              </>
            )}
          </div>
        )}
      </div>
    </>
  )
}
