<!--
  ship-usage-chip — per-turn / per-session token consumption + cost.
-->
<scope>
  props: {
    input = '0'
    output = '0'
    reasoning = ''
    cache = ''
    cost = ''
  }
</scope>

<span>
  <span><b data-test="usage-input">{input()}</b> in</span>
  <span><b data-test="usage-output">{output()}</b> out</span>
  <if cond={reasoning()}>
    <span data-test="usage-reasoning"><b>{reasoning()}</b> reas</span>
  </if>
  <if cond={cache()}>
    <span data-test="usage-cache"><b>{cache()}</b> cache</span>
  </if>
  <if cond={cost()}>
    <span data-test="usage-cost">{cost()}</span>
  </if>
</span>
