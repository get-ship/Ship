<scope>
  props: {}
</scope>
<controller>
import { controller, derived, query, unwrapOr } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { AgentSummary } from '@ship/sdk-client'
import type { Scope } from './ship-agents-table.sail'

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

export default controller((_ctx: ViewScope<Scope>) => {

    const agents = query(() => ship.agents.summaries())
    const rows = derived(() => unwrapOr(agents, EMPTY).items)
    const columnDefs = derived<ColumnDef[]>(() => DEFAULT_COLUMNS)
    return { agents, rows, columnDefs }
})
</controller>

<!--
  <ship-agents-table> — pre-bound list of all agents.

  Subscribes to ship.agents.summaries() via query() inside the controller;
  no SDK call in the consumer. Renders a sortable table with a default
  column set (name, description, source, skills, mcp).

  Tag prefix `ship-*` (vs `sui-*`) marks this component as depending on
  the Ship runtime: importing or mounting it without a host that has
  called setShipClient() will throw on the first SDK access.
-->

<match {agents()}>
  {loading => <spinner size="lg"></spinner>}
  {error as err => <error-state title="Couldn't load agents">{err.message}</error-state>}
  {ok as _ =>
    <table>
      <thead>
        <tr>
          <for {columnDefs() as col} key={col.field}>
            <th>{col.label}</th>
          </for>
        </tr>
      </thead>
      <tbody>
        <for {rows() as row} key={row.name}>
          <tr>
            <td>{row.name}</td>
            <td>{row.description}</td>
            <td>{row.source}</td>
          </tr>
        </for>
      </tbody>
    </table>
  }
</match>
