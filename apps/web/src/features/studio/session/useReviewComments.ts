// Persists diff review comments to the daemon session-files API.
// Comments are stored as JSON in a session file so they survive page reloads.

import { useCallback } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { daemonFetch } from '#/lib/daemon-config'
import { useActiveWorkspaceId } from '#/features/studio/hooks/useWorkspaceConfig'
import type { DiffComment } from './DiffViewer'

const COMMENTS_PATH = 'review-comments.json'

const queryKey = (wsId: string) => ['session', 'reviewComments', wsId] as const

async function loadComments(wsId: string): Promise<DiffComment[]> {
  const res = await daemonFetch(
    `/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(COMMENTS_PATH)}`,
  )
  if (!res.ok) return []
  const body = (await res.json()) as { ok: boolean; data: { content: string } }
  try {
    return JSON.parse(body.data.content) as DiffComment[]
  } catch {
    return []
  }
}

async function saveComments(wsId: string, comments: DiffComment[]): Promise<void> {
  await daemonFetch(
    `/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(COMMENTS_PATH)}`,
    {
      method: 'PUT',
      body: JSON.stringify({ content: JSON.stringify(comments, null, 2) }),
    },
  )
}

export function useReviewComments() {
  const wsId = useActiveWorkspaceId()
  const qc = useQueryClient()

  const { data: comments = [] } = useQuery({
    queryKey: queryKey(wsId),
    queryFn: () => loadComments(wsId),
    enabled: wsId.length > 0,
    staleTime: 30_000,
  })

  const addComment = useCallback(
    async (comment: DiffComment) => {
      const next = [...comments, comment]
      qc.setQueryData(queryKey(wsId), next)
      await saveComments(wsId, next)
    },
    [wsId, comments, qc],
  )

  const removeComment = useCallback(
    async (timestamp: string) => {
      const next = comments.filter((c) => c.timestamp !== timestamp)
      qc.setQueryData(queryKey(wsId), next)
      await saveComments(wsId, next)
    },
    [wsId, comments, qc],
  )

  return { comments, addComment, removeComment }
}
