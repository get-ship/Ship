import { useState, useMemo } from 'react'
import { PROVIDER_MODELS } from '#/features/agents/provider-models'

// Static model list is sufficient for autocomplete at launch.
// TODO: Replace with dynamic model discovery from a provider API endpoint.

interface ProviderGeneralTabProps {
  provider: string
  model: string
  defaultModel: string
  onModelChange: (model: string) => void
}

export function ProviderGeneralTab({
  provider,
  model,
  defaultModel,
  onModelChange,
}: ProviderGeneralTabProps) {
  const [showSuggestions, setShowSuggestions] = useState(false)

  const knownModels = useMemo(
    () => PROVIDER_MODELS[provider] ?? [],
    [provider],
  )

  const filteredModels = useMemo(() => {
    if (!model) return knownModels
    const lower = model.toLowerCase()
    return knownModels.filter((m) => m.toLowerCase().includes(lower))
  }, [model, knownModels])

  const handleSelect = (m: string) => {
    onModelChange(m)
    setShowSuggestions(false)
  }

  return (
    <div className="space-y-5">
      {/* Model override with autocomplete */}
      <div className="space-y-1.5">
        <label className="text-xs font-medium text-foreground">Model override</label>
        <div className="relative">
          <input
            type="text"
            value={model}
            onChange={(e) => { onModelChange(e.target.value); setShowSuggestions(true) }}
            onFocus={() => setShowSuggestions(true)}
            onBlur={() => { setTimeout(() => setShowSuggestions(false), 150) }}
            placeholder={`Using default: ${defaultModel}`}
            className="w-full rounded-lg border border-border/60 bg-background px-3 py-2 font-mono text-xs text-foreground placeholder:text-muted-foreground/40 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/25 transition"
          />
          {showSuggestions && filteredModels.length > 0 && (
            <div className="absolute top-full left-0 right-0 z-10 mt-1 rounded-md border border-border/60 bg-popover shadow-lg overflow-hidden max-h-48 overflow-y-auto">
              {filteredModels.map((m) => (
                <button
                  key={m}
                  type="button"
                  onMouseDown={(e) => { e.preventDefault(); handleSelect(m) }}
                  className={`w-full px-3 py-1.5 text-left text-[11px] font-mono transition-colors ${
                    m === model
                      ? 'bg-primary/10 text-primary'
                      : 'text-foreground hover:bg-muted/60'
                  }`}
                >
                  {m}
                </button>
              ))}
            </div>
          )}
        </div>
        <p className="text-[10px] text-muted-foreground/50">
          Overrides the default model for this provider.
        </p>
      </div>
    </div>
  )
}
