import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'
import { StudioRail } from '#/features/studio/StudioRail'
import { StudioShellProvider, useStudioShell } from '#/features/studio/StudioShellContext'
import { CommandPalette } from '#/features/studio/CommandPalette'
import { AgentPanel } from '#/features/studio/AgentPanel'

export const Route = createFileRoute('/studio')({
  component: StudioLayout,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function StudioLayout() {
  return (
    <PanicSaveProvider>
      <StudioShellProvider>
        <StudioShell />
      </StudioShellProvider>
    </PanicSaveProvider>
  )
}

function StudioShell() {
  const { agentPanelOpen, toggleAgentPanel } = useStudioShell()

  return (
    <div className="flex-1 overflow-hidden min-w-0 flex">
      <StudioRail />
      <main className="flex-1 min-w-0 flex flex-col min-h-0 overflow-hidden">
        <Outlet />
      </main>
      <CommandPalette />
      <AgentPanel open={agentPanelOpen} onClose={toggleAgentPanel} />
    </div>
  )
}
