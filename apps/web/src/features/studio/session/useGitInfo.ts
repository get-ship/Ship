// Hooks for fetching git status, diff, and log via the shipd daemon.
// Project-level hooks hit /api/project/git/* (no workspace scope).
// Workspace-level hooks hit /api/workspaces/{id}/git/* (legacy, still available).

import { useQuery } from '@tanstack/react-query'
import { daemonFetch } from '#/lib/daemon-config'
import { sessionKeys } from './query-keys'

export interface GitStatusFile {
  path: string
  status: string
}

export interface GitStatusResult {
  branch: string
  clean: boolean
  staged: GitStatusFile[]
  modified: GitStatusFile[]
  untracked: GitStatusFile[]
  workingDirectory?: string
}

export interface GitLogEntry {
  hash: string
  short_hash?: string
  message?: string
  subject?: string
  author: string
  date: string
}

function parseGitStatus(raw: string): GitStatusResult {
  const lines = raw.split('\n')
  let branch = ''
  const staged: GitStatusFile[] = []
  const modified: GitStatusFile[] = []
  const untracked: GitStatusFile[] = []

  let section: 'staged' | 'unstaged' | 'untracked' | null = null
  for (const line of lines) {
    const branchMatch = line.match(/^On branch (.+)/)
    if (branchMatch) { branch = branchMatch[1]; continue }
    if (line.startsWith('Changes to be committed:')) { section = 'staged'; continue }
    if (line.startsWith('Changes not staged for commit:')) { section = 'unstaged'; continue }
    if (line.startsWith('Untracked files:')) { section = 'untracked'; continue }
    if (line === '') continue

    const fileMatch = line.match(/^\t(new file|modified|deleted|renamed|copied):\s+(.+)/)
    if (fileMatch) {
      const status = fileMatch[1]
      const path = fileMatch[2].trim()
      if (section === 'staged') staged.push({ path, status })
      else if (section === 'unstaged') modified.push({ path, status })
      continue
    }

    if (section === 'untracked') {
      const untrackedFile = line.match(/^\t(.+)/)
      if (untrackedFile) untracked.push({ path: untrackedFile[1].trim(), status: 'untracked' })
    }
  }

  const clean = staged.length === 0 && modified.length === 0 && untracked.length === 0
  return { branch, clean, staged, modified, untracked }
}

// ── Project-level hooks ─────────────────────────────────────────────────────

export function useGitStatus() {
  return useQuery({
    queryKey: sessionKeys.projectGitStatus(),
    queryFn: async (): Promise<GitStatusResult | null> => {
      const res = await daemonFetch('/api/project/git/status')
      if (!res.ok) return null
      const body = (await res.json()) as { ok: boolean; data: { output: string } }
      return parseGitStatus(body.data.output)
    },
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitDiff(range?: string) {
  return useQuery({
    queryKey: sessionKeys.projectGitDiff(range),
    queryFn: async (): Promise<string | null> => {
      const params = range ? `?range=${encodeURIComponent(range)}` : ''
      const res = await daemonFetch(`/api/project/git/diff${params}`)
      if (!res.ok) return null
      const body = (await res.json()) as { ok: boolean; data: { output: string } }
      if (!body.data.output.trim()) return null
      return body.data.output
    },
    enabled: range !== undefined,
    staleTime: 10_000,
    refetchInterval: 15_000,
  })
}

export function useGitLog(limit = 10) {
  return useQuery({
    queryKey: sessionKeys.projectGitLog(limit),
    queryFn: async (): Promise<GitLogEntry[]> => {
      const res = await daemonFetch(`/api/project/git/log?limit=${limit}`)
      if (!res.ok) return []
      const body = (await res.json()) as { ok: boolean; data: { output: string; commits: GitLogEntry[] } }
      return body.data.commits
    },
    staleTime: 15_000,
    refetchInterval: 30_000,
  })
}
