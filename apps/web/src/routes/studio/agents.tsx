import { createFileRoute, Outlet } from '@tanstack/react-router'
import { DaemonGuard } from '#/features/studio/DaemonGuard'

export const Route = createFileRoute('/studio/agents')({
  component: () => <DaemonGuard><AgentsLayout /></DaemonGuard>,
  ssr: false,
})

function AgentsLayout() {
  return <Outlet />
}
