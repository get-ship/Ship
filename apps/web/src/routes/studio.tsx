import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'
import { StudioRail } from '#/features/studio/StudioRail'
import { StudioShellProvider, useStudioShell } from '#/features/studio/StudioShellContext'
import { CommandPalette } from '#/features/studio/CommandPalette'
import { RightDrawer } from '#/features/studio/session/RightDrawer'
import { WorkspaceIndicator } from '#/features/studio/WorkspaceIndicator'
import { AnnotationProvider, useGlobalAnnotations } from '#/features/studio/AnnotationContext'

export const Route = createFileRoute('/studio')({
  component: StudioLayout,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function StudioLayout() {
  return (
    <PanicSaveProvider>
      <StudioShellProvider>
        <AnnotationProvider>
          <StudioShell />
        </AnnotationProvider>
      </StudioShellProvider>
    </PanicSaveProvider>
  )
}

function StudioShell() {
  const { drawerOpen } = useStudioShell()
  const { staged, removeAnnotation, sendToAgent } = useGlobalAnnotations()

  return (
    <div className="flex-1 overflow-hidden min-w-0 flex">
      <StudioRail />
      <main className="flex-1 min-w-0 flex flex-col min-h-0 overflow-hidden">
        <WorkspaceIndicator />
        <Outlet />
      </main>
      {drawerOpen && (
        <RightDrawer
          stagedAnnotations={staged}
          onSend={sendToAgent}
          onRemoveAnnotation={removeAnnotation}
        />
      )}
      <CommandPalette />
    </div>
  )
}
