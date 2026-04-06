import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'
import { StudioRail } from '#/features/studio/StudioRail'
import { StudioShellProvider } from '#/features/studio/StudioShellContext'
import { CommandPalette } from '#/features/studio/CommandPalette'
import { RightDrawer } from '#/features/studio/session/RightDrawer'
import { WorkspaceIndicator } from '#/features/studio/WorkspaceIndicator'

export const Route = createFileRoute('/studio')({
  component: StudioLayout,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function StudioLayout() {
  return (
    <PanicSaveProvider>
      <StudioShellProvider>
        <div className="flex-1 overflow-hidden min-w-0 flex">
          <StudioRail />
          <div className="flex-1 min-w-0 flex min-h-0 overflow-hidden relative">
            <main className="flex-1 min-w-0 flex flex-col min-h-0 overflow-hidden relative">
              <WorkspaceIndicator />
              <Outlet />
            </main>
            <RightDrawer />
          </div>
          <CommandPalette />
        </div>
      </StudioShellProvider>
    </PanicSaveProvider>
  )
}
