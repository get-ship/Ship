import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { AgentSummary } from '@ship/sdk-client'
import type { Scope } from './ship-agents-table.sui'

interface ColumnDef {
  field: string
  label: string
}

const DEFAULT_COLUMNS: ColumnDef[] = [
  { field: 'name', label: 'Name' },
  { field: 'description', label: 'Description' },
  { field: 'source', label: 'Source' },
  { field: 'skills_count', label: 'Skills' },
  { field: 'mcp_count', label: 'MCP' },
]

const EMPTY = { items: [] as AgentSummary[] }

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const agents = live(() => ship.agents.summaries())
    const rows = derived(() => unwrapOr(agents, EMPTY).items)
    const columnDefs = derived<ColumnDef[]>(() => DEFAULT_COLUMNS)
    return { agents, rows, columnDefs }
  },
})
