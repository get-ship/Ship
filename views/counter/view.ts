// Counter view — tier-1 demo with no controller. State lives entirely
// in the template (`<state name="count" value="0" />`) and inline arrow
// handlers mutate it directly. The daemon shell still imports a setup
// module from `view.js`, so we ship an explicit null default that
// `hydrate()` recognizes as "no controller".

export default null
