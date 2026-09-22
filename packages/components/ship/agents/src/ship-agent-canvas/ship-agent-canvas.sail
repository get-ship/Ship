<!--
  ship-agent-canvas.sail — @ship/agents · the session-scoped Agent Canvas (chrome).
  A field of ship-agent-slots the agent morphs, under a thin control bar that answers
  "what's happening": activity (idle · thinking · morphing) · session identity ·
  undo · stop · save. Scratch by default; Save graduates a slot to a durable artifact.
  Launches from the persistent chat; lives and dies with the session. Slots are slotted.
  Controller emits ship-undo · ship-stop · ship-save.  Props: agent · session · activity.
-->
<scope>
  props: {
    agent = 'agent'
    session = ''
    activity: 'idle' | 'thinking' | 'morphing' = 'idle'
  }
</scope>
<controller>
import type { ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './ship-agent-canvas.sail'

// The canvas chrome is reflected-attribute driven; the control bar emits the
// three session verbs (undo · stop · save) with the agent + session in the
// detail so production can route them to the Stop / Save RPCs.
export default controller((ctx: ControllerScope<Scope>, host?: Element) => {
    const emit = (name: string): void => {
      host?.dispatchEvent(
        new CustomEvent(name, {
          detail: { agent: ctx.agent(), session: ctx.session() },
          bubbles: true,
          composed: true,
        }),
      )
    }
    return {
      undo: () => emit('ship-undo'),
      stop: () => emit('ship-stop'),
      save: () => emit('ship-save'),
    }
})
</controller>

<row gap="3" align="center">
  <span><span data-test="canvas-agent">{agent()}</span><span>{session()}</span></span>
  <span>scratch</span>
  <span>
    <span></span>
    <span>idle</span><span>thinking…</span><span>morphing canvas…</span>
  </span>
  <span>
    <button type="button" label="Undo" data-test="canvas-undo" on:click={undo}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7v6h6M3 13a9 9 0 1 0 3-7.7L3 8"></path></svg></button>
    <button type="button" data-test="canvas-stop" on:click={stop}><svg viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="6" width="12" height="12" rx="1.5"></rect></svg>Stop</button>
    <button type="button" data-test="canvas-save" on:click={save}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path><path d="M17 21v-8H7v8M7 3v5h8"></path></svg>Save</button>
    <span>⌘K chat</span>
  </span>
</row>
<div><slot></slot></div>
