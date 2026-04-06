export function SessionEmptyState({ workspaceName, activeAgent }: {
  workspaceName: string | null
  activeAgent: string | null
}) {
  return (
    <div className="flex items-center justify-center h-full text-muted-foreground">
      <div className="text-center max-w-xs">
        <img src="/ship-logos/ship_logo.svg" alt="" className="size-8 mx-auto mb-4 opacity-30" />
        {workspaceName ? (
          <>
            <p className="text-sm font-medium text-foreground/80">{workspaceName}</p>
            {activeAgent && (
              <p className="text-xs text-muted-foreground/60 mt-0.5">{activeAgent}</p>
            )}
          </>
        ) : (
          <p className="text-sm font-medium text-foreground/60">No active workspace</p>
        )}
        <p className="text-xs mt-3">
          Open a file, view, or press <kbd className="px-1.5 py-0.5 rounded border border-border bg-muted/40 text-[10px] font-mono">Cmd+K</kbd>
        </p>
      </div>
    </div>
  )
}
