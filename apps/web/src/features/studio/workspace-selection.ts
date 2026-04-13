type MinimalWorkspace = { name: string; status?: string }

/**
 * Determine if the workspace selection needs updating, and if so, what the
 * new value should be. Returns undefined when no change is needed.
 */
export function resolveWorkspaceSelection(
  viewedWorkspace: string | null,
  workspaces: MinimalWorkspace[],
): string | null | undefined {
  if (workspaces.length === 0) return undefined // no change
  const isValid = viewedWorkspace && workspaces.some((w) => w.name === viewedWorkspace)
  if (isValid) return undefined // selection is good
  const fallback = workspaces.find((w) => String(w.status).toLowerCase() === 'active')
  return fallback?.name ?? workspaces[0]?.name ?? null
}
