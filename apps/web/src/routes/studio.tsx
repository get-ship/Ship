import { createFileRoute, Outlet, useMatches, useRouterState } from '@tanstack/react-router'
import { useState, useEffect, useRef, useMemo } from 'react'
import { PublishPanel } from '#/features/studio/PublishPanel'
import { useCompiler } from '#/features/compiler/useCompiler'
import { useLibrary } from '#/features/compiler/useLibrary'
import { useAgents } from '#/features/agents/useAgents'
import { agentToLibrary } from '#/features/agents/agent-to-library'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'
import { StudioRail } from '#/features/studio/StudioRail'
import { StudioShellProvider } from '#/features/studio/StudioShellContext'
import { FloatingEvents } from '#/features/studio/FloatingEvents'
import { FloatingChat } from '#/features/studio/FloatingChat'

export const Route = createFileRoute('/studio')({
  component: StudioLayout,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function StudioLayout() {
  return (
    <PanicSaveProvider>
      <StudioShellProvider>
        <StudioSyncShell />
      </StudioShellProvider>
    </PanicSaveProvider>
  )
}

function StudioSyncShell() {
  const { library } = useLibrary()
  const { state: compileState, compile } = useCompiler()
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  const matches = useMatches()
  const activeAgentId = useMemo(() => {
    for (const match of matches) {
      const params = match.params as Record<string, string> | undefined
      if (params?.id && match.routeId === '/studio/agents/$id') {
        return params.id
      }
    }
    return null
  }, [matches])

  const [panelOpen, setPanelOpen] = useState(() =>
    typeof window !== 'undefined' ? window.innerWidth >= 768 : true
  )

  const { getAgent } = useAgents()
  const activeAgent = activeAgentId ? getAgent(activeAgentId) : undefined

  const effectiveLibrary = useMemo(() => {
    if (!library) return library
    if (!activeAgent) return library
    return agentToLibrary(activeAgent, library)
  }, [library, activeAgent])

  useEffect(() => {
    if (!panelOpen || !effectiveLibrary) return
    clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => compile(effectiveLibrary), 600)
    return () => clearTimeout(debounceRef.current)
  }, [effectiveLibrary, panelOpen])

  useEffect(() => {
    if (panelOpen && effectiveLibrary) {
      compile(effectiveLibrary)
    }
  }, [panelOpen])

  const showCompilerPanel = Boolean(activeAgentId)
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const isSession = pathname.startsWith('/studio/session')

  return (
    <div className="flex-1 overflow-hidden min-w-0 flex relative">
      <StudioRail />

      {/* Content — full bleed, no docked sidebars */}
      <main className="flex-1 min-w-0 flex min-h-0 overflow-hidden">
        <div className={`flex-1 min-w-0 ${isSession ? 'flex flex-col overflow-hidden' : 'overflow-auto'}`}>
          <Outlet />
        </div>
        {showCompilerPanel && panelOpen && !isSession && (
          <div className="hidden md:block">
            <PublishPanel
              library={effectiveLibrary}
              compileState={compileState}
              onClose={() => setPanelOpen(false)}
            />
          </div>
        )}
      </main>

      {/* Floating overlays — always accessible */}
      <FloatingEvents />
      <FloatingChat />
    </div>
  )
}
