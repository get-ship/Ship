import { createFileRoute } from '@tanstack/react-router'
import { GitBranch, ArrowLeft } from 'lucide-react'
import { Button } from '@ship/primitives'
import { GitTab } from '#/features/studio/session/GitTab'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useState, useCallback } from 'react'

export const Route = createFileRoute('/studio/git')({
  component: GitPage,
  ssr: false,
})

function GitPage() {
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(20)
  const { diffText } = useDiffContent()
  const [selectedCommit, setSelectedCommit] = useState<string | null>(null)
  const { data: commitDiff } = useGitDiff(
    selectedCommit ? `${selectedCommit}^..${selectedCommit}` : undefined,
  )
  const [showDiff, setShowDiff] = useState(false)

  const handleShowDiff = useCallback(() => setShowDiff(true), [])
  const handleSelectCommit = useCallback((hash: string) => {
    setSelectedCommit(hash)
    setShowDiff(true)
  }, [])

  const handleBack = useCallback(() => {
    if (selectedCommit) {
      // Go back to working tree diff if we were viewing a commit
      setSelectedCommit(null)
    } else {
      // Go back to status view
      setShowDiff(false)
    }
  }, [selectedCommit])

  const activeDiff = selectedCommit ? commitDiff : diffText

  if (showDiff && activeDiff) {
    return (
      <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0">
          <Button variant="ghost" size="icon" className="size-7" onClick={handleBack}>
            <ArrowLeft className="size-3.5" />
          </Button>
          <GitBranch className="size-3.5 text-muted-foreground" />
          <span className="text-xs font-medium">
            {selectedCommit ? `Commit ${selectedCommit.slice(0, 7)}` : 'Working tree diff'}
          </span>
          {selectedCommit && (
            <button
              onClick={() => setSelectedCommit(null)}
              className="text-[10px] text-muted-foreground hover:text-foreground transition-colors ml-2"
            >
              Show working tree
            </button>
          )}
        </div>
        <div className="flex-1 overflow-auto">
          <DiffViewer
            diffText={activeDiff}
            onComment={(c) => {
              // TODO: wire to annotation system
              console.log('diff comment:', c)
            }}
          />
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-2xl mx-auto py-6">
        <div className="flex items-center gap-2 px-3 mb-4">
          <GitBranch className="size-5 text-muted-foreground" />
          <h1 className="text-lg font-semibold">Git</h1>
          {gitStatus && (
            <span className={`text-xs ${gitStatus.clean ? 'text-emerald-500' : 'text-amber-500'}`}>
              {gitStatus.clean ? 'Clean' : 'Changes'}
            </span>
          )}
        </div>
        <GitTab
          gitStatus={gitStatus}
          gitLog={gitLog}
          onShowDiff={handleShowDiff}
          onSelectCommit={handleSelectCommit}
        />
      </div>
    </div>
  )
}
