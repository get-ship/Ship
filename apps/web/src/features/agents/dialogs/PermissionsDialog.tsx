import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import { Lock, Wrench, FolderOpen, X } from 'lucide-react'
import type { ProfilePermissions, McpServerConfig } from '@ship/set'
import {
  BUILTIN_TOOLS,
  COMMON_BASH_PATTERNS,
  getMcpToolSuggestions,
} from '#/features/agents/permission-presets'

/** Extended permissions including additional_directories from the schema. */
type ExtendedPermissions = ProfilePermissions & { additional_directories?: string[] }

interface PermissionsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  permissions: ProfilePermissions
  onSave: (permissions: ProfilePermissions) => void
  mcpServers?: McpServerConfig[]
}

export function PermissionsDialog({ open, onOpenChange, permissions, onSave, mcpServers }: PermissionsDialogProps) {
  const [local, setLocal] = useState<ExtendedPermissions>({})

  const suggestions = useMemo(() => {
    const mcpTools = getMcpToolSuggestions((mcpServers ?? []).map((s) => s.name))
    return [...BUILTIN_TOOLS, ...mcpTools, ...COMMON_BASH_PATTERNS]
  }, [mcpServers])

  useEffect(() => {
    if (open) setLocal(structuredClone(permissions) as ExtendedPermissions)
  }, [open, permissions])

  const handleEscape = useCallback(
    (e: KeyboardEvent) => { if (e.key === 'Escape') onOpenChange(false) },
    [onOpenChange],
  )

  useEffect(() => {
    if (!open) return
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [open, handleEscape])

  if (!open) return null

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={() => onOpenChange(false)} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-lg rounded-xl border border-border/60 bg-card shadow-2xl flex flex-col max-h-[80vh]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <Lock className="size-4 text-primary" />
              <h2 className="font-display text-sm font-semibold text-foreground">Edit Permissions</h2>
            </div>
            <button
              onClick={() => onOpenChange(false)}
              aria-label="Close"
              className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition"
            >
              <X className="size-4" />
            </button>
          </div>

          {/* Scrollable body */}
          <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
            <DimensionSection icon={<Wrench className="size-3.5" />} label="Tools">
              <TagInput
                label="Allow"
                variant="allow"
                values={local.tools_allow ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_allow: v }))}
                suggestions={suggestions}
              />
              <TagInput
                label="Ask"
                variant="neutral"
                values={local.tools_ask ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_ask: v }))}
                suggestions={suggestions}
              />
              <TagInput
                label="Deny"
                variant="deny"
                values={local.tools_deny ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_deny: v }))}
                suggestions={suggestions}
              />
            </DimensionSection>
            <DimensionSection icon={<FolderOpen className="size-3.5" />} label="Additional Directories">
              <TagInput
                label="Directories"
                variant="neutral"
                values={local.additional_directories ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, additional_directories: v }))}
              />
              <p className="text-[10px] text-muted-foreground/50">
                Additional directories the agent can access outside the project root.
              </p>
            </DimensionSection>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button
              onClick={() => onOpenChange(false)}
              className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Cancel
            </button>
            <button
              onClick={() => { onSave(local as ProfilePermissions); onOpenChange(false) }}
              className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </>
  )
}

function DimensionSection({ icon, label, children }: { icon: React.ReactNode; label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2.5">
      <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
        {icon}
        {label}
      </div>
      <div className="space-y-2 pl-5">{children}</div>
    </div>
  )
}

function TagInput({
  label,
  variant,
  values,
  onChange,
  suggestions,
}: {
  label: string
  variant: 'allow' | 'deny' | 'neutral'
  values: string[]
  onChange: (values: string[]) => void
  suggestions?: readonly string[] | string[]
}) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [inputValue, setInputValue] = useState('')
  const [showSuggestions, setShowSuggestions] = useState(false)

  const tagColor =
    variant === 'allow' ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20'
    : variant === 'deny' ? 'bg-destructive/10 text-destructive border-destructive/20'
    : 'bg-muted text-muted-foreground border-border/40'

  const filteredSuggestions = useMemo(() => {
    if (!suggestions || !inputValue) return []
    const lower = inputValue.toLowerCase()
    return suggestions
      .filter((s) => s.toLowerCase().includes(lower) && !values.includes(s))
      .slice(0, 8)
  }, [suggestions, inputValue, values])

  const addValue = (v: string) => {
    const trimmed = v.trim()
    if (trimmed && !values.includes(trimmed)) {
      onChange([...values, trimmed])
    }
    setInputValue('')
    setShowSuggestions(false)
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key !== 'Enter') return
    e.preventDefault()
    addValue(inputValue)
  }

  const remove = (idx: number) => onChange(values.filter((_, i) => i !== idx))

  return (
    <div className="space-y-1.5">
      <span className="text-[11px] text-muted-foreground">{label}</span>
      {values.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {values.map((v, i) => (
            <span key={`${v}-${i}`} className={`inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px] ${tagColor}`}>
              {v}
              <button type="button" onClick={() => remove(i)} className="opacity-60 hover:opacity-100 transition-opacity">
                <X className="size-2.5" />
              </button>
            </span>
          ))}
        </div>
      )}
      <div className="relative">
        <input
          ref={inputRef}
          type="text"
          value={inputValue}
          onChange={(e) => { setInputValue(e.target.value); setShowSuggestions(true) }}
          onFocus={() => setShowSuggestions(true)}
          onBlur={() => { setTimeout(() => setShowSuggestions(false), 150) }}
          placeholder={`Add ${label.toLowerCase()}...`}
          onKeyDown={handleKeyDown}
          className="w-full rounded-md border border-border/60 bg-background px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground/40 outline-none focus:border-primary"
        />
        {showSuggestions && filteredSuggestions.length > 0 && (
          <div className="absolute top-full left-0 right-0 z-10 mt-1 rounded-md border border-border/60 bg-popover shadow-lg overflow-hidden max-h-40 overflow-y-auto">
            {filteredSuggestions.map((s) => (
              <button
                key={s}
                type="button"
                onMouseDown={(e) => { e.preventDefault(); addValue(s) }}
                className="w-full px-2.5 py-1.5 text-left text-[11px] text-foreground hover:bg-muted/60 transition-colors font-mono"
              >
                {s}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
