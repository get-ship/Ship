import { WifiOff, Terminal, Copy, Check, RefreshCw } from 'lucide-react'
import { useState } from 'react'
import { usePlatform } from './PlatformProvider'

export function DaemonRequired() {
  const { pairing, beginPairing } = usePlatform()
  const [copied, setCopied] = useState(false)

  // Pairing in progress — show the code and instructions.
  if (pairing) {
    const cliCommand = `ship auth pair ${pairing.code}`

    const handleCopy = async () => {
      try {
        await navigator.clipboard.writeText(cliCommand)
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
      } catch {
        /* clipboard API may be blocked; fallback is visual select-all */
      }
    }

    return (
      <div role="status" className="flex flex-col items-center justify-center gap-6 py-16 px-6 text-center">
        <div className="flex size-12 items-center justify-center rounded-2xl border border-blue-500/20 bg-blue-500/10">
          <Terminal className="size-6 text-blue-500" />
        </div>
        <div className="w-full max-w-sm">
          <h2 className="text-lg font-semibold text-foreground mb-1">Pair this browser</h2>
          <p className="text-sm text-muted-foreground mb-6">
            Run this in your terminal to authorize Studio. The daemon is local — this proves you control the machine.
          </p>
          <div className="font-mono text-3xl font-semibold tracking-widest bg-muted rounded-lg py-3 px-6 mb-4 select-all">
            {pairing.code}
          </div>
          <div className="flex items-center gap-2">
            <code className="flex-1 px-3 py-2 rounded bg-muted text-xs font-mono text-left select-all overflow-x-auto">
              {cliCommand}
            </code>
            <button
              type="button"
              onClick={handleCopy}
              className="flex size-8 items-center justify-center rounded border border-border bg-muted hover:bg-muted/80 transition-colors"
              aria-label="Copy command"
            >
              {copied ? <Check className="size-4 text-green-500" /> : <Copy className="size-4" />}
            </button>
          </div>
          <p className="text-xs text-muted-foreground mt-4 flex items-center justify-center gap-1.5">
            Code expires in 5 minutes
            <button
              type="button"
              onClick={beginPairing}
              className="inline-flex items-center gap-1 hover:text-foreground transition-colors"
              aria-label="Get a new code"
            >
              <RefreshCw className="size-3" /> new code
            </button>
          </p>
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
