import { createFileRoute } from '@tanstack/react-router'
import { DaemonGuard } from '#/features/studio/DaemonGuard'
import { GitBranch, ArrowLeft, CheckCircle2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { GitTab } from '#/features/studio/session/GitTab'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useReviewComments } from '#/features/studio/session/useReviewComments'
import { useGlobalAnnotations } from '#/features/studio/AnnotationContext'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { useState, useCallback, useMemo } from 'react'
import type { DiffComment } from '#/features/studio/session/DiffViewer'

export const Route = createFileRoute('/studio/git')({
  component: () => <DaemonGuard><GitPage /></DaemonGuard>,
  ssr: false,
})

function ReviewBanner({ jobs }: { jobs: { slug: string; branch: string }[] }) {
  if (jobs.length === 0) return null
  return (
    <div className="mx-4 mb-3 flex items-center gap-2 rounded-lg border border-emerald-500/30 bg-emerald-500/10 px-3 py-2">
      <CheckCircle2 className="size-4 text-emerald-500 shrink-0" />
      <div className="flex-1 min-w-0">
        {jobs.map((j) => (
          <p key={j.slug} className="text-xs text-foreground">
            Pipeline <span className="font-medium">{j.slug}</span> completed. Review the diff below.
          </p>
        ))}
      </div>
    </div>
  )
}

function GitPage() {
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(20)
  const { diffText } = useDiffContent()
  const { comments, addComment, removeComment } = useReviewComments()
  const { addAnnotation } = useGlobalAnnotations()
  const { jobs } = useDaemon()
  const [selectedCommit, setSelectedCommit] = useState<string | null>(null)
  const { data: commitDiff } = useGitDiff(
    selectedCommit ? `${selectedCommit}^..${selectedCommit}` : undefined,
  )
  const [showDiff, setShowDiff] = useState(false)

  const completedJobs = useMemo(
    () => jobs.filter((j) => j.status === 'completed').map((j) => ({ slug: j.slug, branch: j.branch })),
    [jobs],
  )

  const handleComment = useCallback(
    (comment: DiffComment) => {
      addComment(comment)
      addAnnotation(comment.file, {
        type: 'click',
        id: `review-${comment.timestamp}`,
        selector: `${comment.file}:${comment.lineNum}`,
        text: comment.content,
        note: comment.comment,
        timestamp: comment.timestamp,
        x: 0,
        y: 0,
      })
    },
    [addComment, addAnnotation],
  )

  const handleShowDiff = useCallback(() => setShowDiff(true), [])
  const handleSelectCommit = useCallback((hash: string) => {
    setSelectedCommit(hash)
    setShowDiff(true)
  }, [])

  const handleBack = useCallback(() => {
    if (selectedCommit) {
      setSelectedCommit(null)
    } else {
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
        <ReviewBanner jobs={completedJobs} />
        <div className="flex-1 overflow-auto">
          <DiffViewer
            diffText={activeDiff}
            comments={comments}
            onComment={handleComment}
            onRemoveComment={removeComment}
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
        <ReviewBanner jobs={completedJobs} />
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
