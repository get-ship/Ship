import { WifiOff } from 'lucide-react'

export function DaemonRequired() {
  return (
    <div role="status" className="flex flex-col items-center justify-center gap-4 py-16 px-6 text-center">
      <div className="flex size-12 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10">
        <WifiOff className="size-6 text-amber-500" />
      </div>
      <div>
        <h2 className="text-lg font-semibold text-foreground mb-1">Daemon not connected</h2>
        <p className="text-sm text-muted-foreground max-w-sm">
          Start the Ship daemon to connect. Run <code className="px-1.5 py-0.5 rounded bg-muted text-xs font-mono">ship daemon start</code> in your terminal, or install Ship if you haven't already.
        </p>
      </div>
    </div>
  )
}
