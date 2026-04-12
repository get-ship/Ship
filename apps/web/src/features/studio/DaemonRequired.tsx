import { WifiOff, Terminal } from 'lucide-react'
import { usePlatform } from './PlatformProvider'

export function DaemonRequired() {
  const { pairing } = usePlatform()

  // Pairing in progress — show the code and instructions.
  if (pairing) {
    return (
      <div role="status" className="flex flex-col items-center justify-center gap-6 py-16 px-6 text-center">
        <div className="flex size-12 items-center justify-center rounded-2xl border border-blue-500/20 bg-blue-500/10">
          <Terminal className="size-6 text-blue-500" />
        </div>
        <div>
          <h2 className="text-lg font-semibold text-foreground mb-1">Pair this browser</h2>
          <p className="text-sm text-muted-foreground max-w-sm mb-6">
            Paste this code into your terminal to authorize Studio. The daemon is local — this proves you control the machine.
          </p>
          <div className="font-mono text-3xl font-semibold tracking-widest bg-muted rounded-lg py-3 px-6 mb-4 select-all">
            {pairing.code}
          </div>
          <code className="block px-3 py-2 rounded bg-muted text-xs font-mono">
            ship auth pair {pairing.code}
          </code>
        </div>
      </div>
    )
  }

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
