import { createFileRoute, useNavigate, Link } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { ArrowLeft, SearchX } from 'lucide-react'
import { useAgents } from '#/features/agents/useAgents'
import { useAgentDrafts } from '#/features/agents/useAgentDrafts'
import { AgentStickyHeader } from '#/features/agents/AgentStickyHeader'
import { SkillsSection } from '#/features/agents/sections/SkillsSection'
import { McpSection } from '#/features/agents/sections/McpSection'
import { PermissionsSection } from '#/features/agents/sections/PermissionsSection'
import { ProvidersSection } from '#/features/agents/sections/ProvidersSection'
import { RulesSection } from '#/features/agents/sections/RulesSection'
import { AddSkillDialog } from '#/features/agents/dialogs/AddSkillDialog'
import { AddMcpDialog } from '#/features/agents/dialogs/AddMcpDialog'
import { EditAgentDialog } from '#/features/agents/dialogs/EditAgentDialog'
import { PermissionsDialog } from '#/features/agents/dialogs/PermissionsDialog'
import { RuleEditorDialog } from '#/features/agents/dialogs/RuleEditorDialog'
import type { ResolvedAgentProfile, ToolPermission } from '#/features/agents/types'
import type { Skill, Rule, HookConfig, ProfilePermissions } from '@ship/set'

import { AgentDetailSkeleton } from '#/features/studio/StudioSkeleton'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'

export const Route = createFileRoute('/studio/agents/$id')({
  component: AgentDetailPage,
  pendingComponent: AgentDetailSkeleton,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function AgentNotFound({ id }: { id: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-24 text-center">
      <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40 mb-4">
        <SearchX className="size-5 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">Agent not found</p>
      <p className="mt-1 text-xs text-muted-foreground max-w-xs">
        No agent with ID "{id}" exists in this project.
      </p>
      <Link
        to="/studio/agents"
        className="mt-4 inline-flex items-center gap-1.5 rounded-lg border border-border px-4 py-2 text-xs font-medium text-muted-foreground hover:text-foreground hover:border-primary/30 transition no-underline"
      >
        <ArrowLeft className="size-3" />
        Back to agents
      </Link>
    </div>
  )
}

function AgentDetailPage() {
  const { id } = Route.useParams()
  const navigate = useNavigate()
  const { getAgent, isLoading } = useAgents()
  const { setDraft, hasDraft, clearDraft } = useAgentDrafts()
  const profile = getAgent(id)
  const isDraft = hasDraft(id)

  if (isLoading && !profile) {
    return <AgentDetailSkeleton />
  }

  if (!profile) {
    return <AgentNotFound id={id} />
  }

  const handleDelete = () => {
    if (!confirm(`Delete "${profile.profile.name}"?`)) return
    void navigate({ to: '/studio/agents', replace: true })
  }

  const update = (patch: Partial<ResolvedAgentProfile>) => setDraft(id, patch)
  const handleDiscard = () => clearDraft(id)

  return (
    <AgentDetailView
      profile={profile}
      isDraft={isDraft}
      onUpdate={update}
      onDelete={handleDelete}
      onDiscard={handleDiscard}
    />
  )
}

// -- Inner view (split to keep AgentDetailPage under 300 lines) -------------

interface ViewProps {
  profile: ResolvedAgentProfile
  isDraft: boolean
  onUpdate: (patch: Partial<ResolvedAgentProfile>) => void
  onDelete: () => void
  onDiscard: () => void
}

function AgentDetailView({ profile, isDraft, onUpdate, onDelete, onDiscard }: ViewProps) {
  const removeSkill = useCallback(
    (skillId: string) => onUpdate({ skills: profile.skills.filter((s) => s.id !== skillId) }),
    [onUpdate, profile.skills],
  )
  const addSkill = useCallback(
    (skill: Skill) => {
      if (profile.skills.some((s) => s.id === skill.id)) return
      onUpdate({ skills: [...profile.skills, skill] })
    },
    [onUpdate, profile.skills],
  )
  const removeServer = useCallback(
    (name: string) => onUpdate({ mcpServers: profile.mcpServers.filter((s) => s.name !== name) }),
    [onUpdate, profile.mcpServers],
  )
  const mcpToolStates = profile.toolPermissions ?? {}
  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = prev[serverName] ?? {}
      onUpdate({ toolPermissions: { ...prev, [serverName]: { ...serverTools, [toolName]: permission } } })
    },
    [onUpdate, profile.toolPermissions],
  )
  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      const prev = profile.toolPermissions ?? {}
      const serverTools = { ...(prev[serverName] ?? {}) }
      for (const name of toolNames) serverTools[name] = permission
      onUpdate({ toolPermissions: { ...prev, [serverName]: serverTools } })
    },
    [onUpdate, profile.toolPermissions],
  )
  const setPermissionPreset = useCallback(
    (preset: string) => onUpdate({ permissions: { ...profile.permissions, preset } }),
    [onUpdate, profile.permissions],
  )
  const updatePermissions = useCallback(
    (permissions: ProfilePermissions) => onUpdate({ permissions: { ...permissions, preset: 'custom' } }),
    [onUpdate],
  )
  const addRule = useCallback((rule: Rule) => onUpdate({ rules: [...profile.rules, rule] }), [onUpdate, profile.rules])
  const updateRule = useCallback(
    (index: number, rule: Rule) => onUpdate({ rules: profile.rules.map((r, i) => (i === index ? rule : r)) }),
    [onUpdate, profile.rules],
  )
  const removeRule = useCallback((index: number) => onUpdate({ rules: profile.rules.filter((_, i) => i !== index) }), [onUpdate, profile.rules])
  const setModel = useCallback((model: string | null) => onUpdate({ model }), [onUpdate])
  const setHooks = useCallback((hooks: HookConfig[]) => onUpdate({ hooks }), [onUpdate])
  const setProviderSettings = useCallback(
    (settings: Record<string, Record<string, unknown>>) => onUpdate({ providerSettings: settings }),
    [onUpdate],
  )

  const [skillOpen, setSkillOpen] = useState(false)
  const [mcpOpen, setMcpOpen] = useState(false)
  const [editOpen, setEditOpen] = useState(false)
  const [permsOpen, setPermsOpen] = useState(false)
  const [ruleOpen, setRuleOpen] = useState(false)
  const [ruleEdit, setRuleEdit] = useState<{ index: number; rule: Rule } | null>(null)

  return (
    <>
      <div className="flex flex-col h-full min-h-0 overflow-hidden">
        <AgentStickyHeader profile={profile} onEdit={() => setEditOpen(true)} onDelete={onDelete} onDiscard={isDraft ? onDiscard : undefined} isDraft={isDraft} />
        <div className="flex-1 overflow-y-auto">
          <SkillsSection skills={profile.skills} onRemove={removeSkill} onAdd={() => setSkillOpen(true)} />
          <McpSection servers={profile.mcpServers} toolStates={mcpToolStates} onRemove={removeServer} onSetToolPermission={setToolPermission} onSetGroupPermission={setGroupPermission} onAdd={() => setMcpOpen(true)} />
          <PermissionsSection
            permissions={profile.permissions ?? {}}
            activePreset={profile.permissions?.preset ?? 'ship-standard'}
            onPresetChange={setPermissionPreset}
            onEdit={() => setPermsOpen(true)}
            mcpServers={profile.mcpServers}
          />
          <RulesSection
            rules={profile.rules}
            onAdd={() => { setRuleEdit(null); setRuleOpen(true) }}
            onEdit={(i) => { setRuleEdit({ index: i, rule: profile.rules[i] }); setRuleOpen(true) }}
            onRemove={removeRule}
          />
          <ProvidersSection
            providers={profile.profile.providers ?? []}
            model={profile.model}
            hooks={profile.hooks}
            providerSettings={profile.providerSettings ?? {}}
            onChangeModel={setModel}
            onChangeHooks={setHooks}
            onChangeProviderSettings={setProviderSettings}
          />
          <div className="h-24" />
        </div>
      </div>
      <AddSkillDialog open={skillOpen} onOpenChange={setSkillOpen} existingIds={profile.skills.map((s) => s.id)} onAdd={addSkill} />
      <AddMcpDialog open={mcpOpen} onOpenChange={setMcpOpen} existingNames={profile.mcpServers.map((s) => s.name)} onAdd={(server) => onUpdate({ mcpServers: [...profile.mcpServers, server] })} />
      <EditAgentDialog open={editOpen} onOpenChange={setEditOpen} profile={profile} onSave={(patch) => onUpdate(patch)} />
      <PermissionsDialog open={permsOpen} onOpenChange={setPermsOpen} permissions={profile.permissions ?? {}} onSave={updatePermissions} mcpServers={profile.mcpServers} />
      <RuleEditorDialog open={ruleOpen} onOpenChange={setRuleOpen} rule={ruleEdit?.rule ?? null} onSave={(rule) => { if (ruleEdit) updateRule(ruleEdit.index, rule); else addRule(rule) }} onDelete={ruleEdit ? () => removeRule(ruleEdit.index) : undefined} />
    </>
  )
}
