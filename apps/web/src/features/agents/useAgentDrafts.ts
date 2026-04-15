// Draft overlay store: holds unsaved edits keyed by agent ID.
// Only stores deltas -- not full agent objects.
// Persisted to localStorage so in-progress edits survive reloads.

import { useSyncExternalStore, useCallback } from 'react'
import type { ResolvedAgentProfile } from './types'

const STORAGE_KEY = 'ship-agent-drafts-v1'

type AgentPatch = Partial<ResolvedAgentProfile>

interface DraftState {
  drafts: Record<string, AgentPatch>
}

function loadInitial(): DraftState {
  if (typeof window === 'undefined') return { drafts: {} }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return { drafts: {} }
    const parsed = JSON.parse(raw) as DraftState
    return parsed?.drafts ? parsed : { drafts: {} }
  } catch {
    return { drafts: {} }
  }
}

let state: DraftState = loadInitial()
const listeners = new Set<() => void>()

function emit() {
  for (const fn of listeners) fn()
}

function persist() {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch { /* quota or unavailable — in-memory state is still valid */ }
}

export function setDraft(agentId: string, patch: AgentPatch) {
  const existing = state.drafts[agentId] ?? {}
  state = { drafts: { ...state.drafts, [agentId]: { ...existing, ...patch } } }
  persist()
  emit()
}

export function clearDraft(agentId: string) {
  const { [agentId]: _, ...rest } = state.drafts
  state = { drafts: rest }
  persist()
  emit()
}

export function clearAllDrafts() {
  state = { drafts: {} }
  persist()
  emit()
}

export function getDrafts(): Record<string, AgentPatch> {
  return state.drafts
}

export function hasDraft(agentId: string): boolean {
  return agentId in state.drafts
}

function subscribe(cb: () => void) {
  listeners.add(cb)
  return () => { listeners.delete(cb) }
}

function getSnapshot(): DraftState {
  return state
}

function getServerSnapshot(): DraftState {
  return { drafts: {} }
}

export function useAgentDrafts() {
  const current = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)

  return {
    drafts: current.drafts,
    setDraft: useCallback((agentId: string, patch: AgentPatch) => setDraft(agentId, patch), []),
    clearDraft: useCallback((agentId: string) => clearDraft(agentId), []),
    clearAllDrafts: useCallback(() => clearAllDrafts(), []),
    hasDraft: useCallback((agentId: string) => agentId in current.drafts, [current.drafts]),
  }
}
