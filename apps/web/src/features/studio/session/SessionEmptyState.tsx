import { FolderOpen, Search, FileText } from 'lucide-react'
import { usePlatform } from '#/features/studio/PlatformProvider'

interface SessionFile {
  path: string
  name: string
  type: string
}

interface SessionEmptyStateProps {
  workspaceName: string | null
  activeAgent: string | null
  files: SessionFile[]
  onSelectFile: (path: string) => void
}

export function SessionEmptyState({
  workspaceName, activeAgent, files, onSelectFile,
}: SessionEmptyStateProps) {
  const { toggleSidebar: toggleFileSidebar, toggleCommandPalette } = usePlatform()
  const recentFiles = files.slice(0, 5)

  return (
    <div className="flex items-center justify-center h-full text-muted-foreground">
      <div className="text-center max-w-sm">
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

        <div className="flex items-center justify-center gap-2 mt-5">
          <button
            onClick={toggleFileSidebar}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md border border-border bg-muted/30 hover:bg-muted/60 text-foreground/70 hover:text-foreground transition-colors"
          >
            <FolderOpen className="size-3.5" />
            <span>Open Files</span>
            <kbd className="ml-1 px-1 py-0.5 rounded border border-border/60 bg-background/40 text-[10px] font-mono leading-none">
              Cmd+B
            </kbd>
          </button>
          <button
            onClick={toggleCommandPalette}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md border border-border bg-muted/30 hover:bg-muted/60 text-foreground/70 hover:text-foreground transition-colors"
          >
            <Search className="size-3.5" />
            <span>Search</span>
            <kbd className="ml-1 px-1 py-0.5 rounded border border-border/60 bg-background/40 text-[10px] font-mono leading-none">
              Cmd+K
            </kbd>
          </button>
        </div>

        {recentFiles.length > 0 && (
          <div className="mt-5 text-left">
            <p className="text-[10px] uppercase tracking-wider text-muted-foreground/50 mb-1.5 px-2">
              Session files
            </p>
            <div className="space-y-px">
              {recentFiles.map((file) => (
                <button
                  key={file.path}
                  onClick={() => onSelectFile(file.path)}
                  className="flex items-center gap-2 w-full px-2 py-1.5 text-xs rounded hover:bg-muted/40 text-foreground/60 hover:text-foreground transition-colors"
                >
                  <FileText className="size-3.5 shrink-0 text-muted-foreground/40" />
                  <span className="truncate">{file.name}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
