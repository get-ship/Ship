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
  style val = { type({ family: mono, weight: bold, size: lg }) }
  style lab = { type({ size: xs, case: uppercase }).fg(muted-foreground) }
  style outcome = { type({ family: mono, weight: semibold }) }
</scope>

<row gap="0">
  <column gap="1"><span data-test="m-duration" style(val)>{duration()}</span><span style(lab)>duration</span></column>
  <column gap="1"><span data-test="m-turns" style(val)>{turns()}</span><span style(lab)>turns</span></column>
  <column gap="1"><span data-test="m-tools" style(val)>{tools()}</span><span style(lab)>tool calls</span></column>
  <column gap="1" data-zero={errors() === '0' ? 'true' : null}><span data-test="m-errors" style(val)>{errors()}</span><span style(lab)>errors</span></column>
  <column gap="1"><span data-test="m-cost" style(val)>{cost()}</span><span style(lab)>cost</span></column>
  <div><span data-test="m-outcome" style(outcome)>{outcome()}</span></div>
</row>
