import { createFileRoute } from '@tanstack/react-router'
import { GitBranch } from 'lucide-react'
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

  const activeDiff = selectedCommit ? commitDiff : diffText

  if (showDiff && activeDiff) {
    return (
      <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0">
          <GitBranch className="size-4 text-muted-foreground" />
          <span className="text-xs font-medium">
            {selectedCommit ? `Commit ${selectedCommit.slice(0, 7)}` : 'Working tree diff'}
          </span>
          <div className="flex-1" />
          <button
            onClick={() => { setShowDiff(false); setSelectedCommit(null) }}
            className="text-xs text-muted-foreground hover:text-foreground transition-colors"
          >
            Back
          </button>
        </div>
        <div className="flex-1 overflow-auto">
          <DiffViewer diffText={activeDiff} onComment={() => {}} />
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
