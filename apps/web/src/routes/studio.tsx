import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'
import { StudioRail } from '#/features/studio/StudioRail'
import { StudioShellProvider } from '#/features/studio/StudioShellContext'
import { CommandPalette } from '#/features/studio/CommandPalette'

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
          <main className="flex-1 min-w-0 flex flex-col min-h-0 overflow-hidden">
            <Outlet />
          </main>
          <CommandPalette />
        </div>
      </StudioShellProvider>
    </PanicSaveProvider>
  )
}
