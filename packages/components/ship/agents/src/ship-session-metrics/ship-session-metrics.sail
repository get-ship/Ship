<!--
  ship-session-metrics — session rollup strip.
-->
<scope>
  props: {
    duration = '—'
    turns = '0'
    tools = '0'
    errors = '0'
    cost = '—'
    outcome: 'completed' | 'failed' | 'cancelled' = 'completed'
  }
</scope>

<row gap="0">
  <column gap="1"><span data-test="m-duration">{duration()}</span><span>duration</span></column>
  <column gap="1"><span data-test="m-turns">{turns()}</span><span>turns</span></column>
  <column gap="1"><span data-test="m-tools">{tools()}</span><span>tool calls</span></column>
  <column gap="1"><span data-test="m-errors">{errors()}</span><span>errors</span></column>
  <column gap="1"><span data-test="m-cost">{cost()}</span><span>cost</span></column>
  <div><span data-test="m-outcome">{outcome()}</span></div>
</row>
