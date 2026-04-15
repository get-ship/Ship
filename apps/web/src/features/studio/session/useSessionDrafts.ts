// Draft state management for session file editing.
// In-memory only — drafts do not survive reload. Save writes to daemon.

import { useState, useCallback, useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { daemonFetch } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { usePlatform } from '#/features/studio/PlatformProvider'
import { sessionKeys } from './query-keys'

export interface SessionDraft {
  content: string
  originalContent: string
}

export interface UseSessionDraftsReturn {
  /** Get draft content for a path, or undefined if no draft exists */
  getDraft: (path: string) => string | undefined
  /** Initialize a draft from server content (no-op if draft already exists) */
  openFile: (path: string, serverContent: string) => void
  /** Update draft content */
  updateContent: (path: string, content: string) => void
  /** Whether the draft for this path differs from original */
  isDirty: (path: string) => boolean
  /** Set of all paths with unsaved changes */
  unsavedPaths: Set<string>
  /** Save a single file to daemon */
  saveFile: (path: string) => void
  /** Whether a save is in progress */
  isSaving: boolean
}

export function useSessionDrafts(): UseSessionDraftsReturn {
  const { workspace } = usePlatform()
  const viewedWorkspace = workspace?.name ?? null
  const { workspaces } = useDaemon()
  const wsId = viewedWorkspace
    ?? (workspaces.find((w) => String(w.status).toLowerCase() === 'active') ?? workspaces[0])?.name
    ?? ''
  const queryClient = useQueryClient()
  const [drafts, setDrafts] = useState<Record<string, SessionDraft>>({})

  const getDraft = useCallback((path: string): string | undefined => {
    return drafts[path]?.content
  }, [drafts])

  const openFile = useCallback((path: string, serverContent: string) => {
    setDrafts((prev) => {
      if (prev[path]) return prev
      return { ...prev, [path]: { content: serverContent, originalContent: serverContent } }
    })
  }, [])

  const updateContent = useCallback((path: string, content: string) => {
    setDrafts((prev) => {
      const existing = prev[path]
      if (!existing) {
        return { ...prev, [path]: { content, originalContent: content } }
      }
      return { ...prev, [path]: { ...existing, content } }
    })
  }, [])

  const isDirty = useCallback((path: string): boolean => {
    const draft = drafts[path]
    if (!draft) return false
    return draft.content !== draft.originalContent
  }, [drafts])

  const unsavedPaths = useMemo(() => {
    const paths = new Set<string>()
    for (const [path, draft] of Object.entries(drafts)) {
      if (draft.content !== draft.originalContent) paths.add(path)
    }
    return paths
  }, [drafts])

  // Save to daemon
  const saveMutation = useMutation({
    mutationFn: async ({ path, content }: { path: string; content: string }) => {
      const res = await daemonFetch(`/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(path)}`, {
        method: 'PUT',
        body: JSON.stringify({ content }),
      })
      if (!res.ok) throw new Error(`daemon: write session file ${res.status}`)
    },
    onSuccess: (_, { path }) => {
      setDrafts((prev) => {
        const draft = prev[path]
        if (!draft) return prev
        return { ...prev, [path]: { ...draft, originalContent: draft.content } }
      })
      void queryClient.invalidateQueries({ queryKey: sessionKeys.fileContent(wsId, path) })
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files(wsId) })
    },
  })

  const saveFile = useCallback((path: string) => {
    const draft = drafts[path]
    if (!draft) return
    saveMutation.mutate({ path, content: draft.content })
  }, [drafts, saveMutation])

  return {
    getDraft,
    openFile,
    updateContent,
    isDirty,
    unsavedPaths,
    saveFile,
    isSaving: saveMutation.isPending,
  }
}
