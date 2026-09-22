<!--
  <sui-sfx> — declarative one-shot Web Audio binding.

  Plays `src` whenever the stage's action bus emits `on`. `pitch-range` like
  `"0.9..1.1"` samples a random playback rate inside that range each play.
  Buses: sfx (game effects), music (looped beds — out of scope for v0.1
  playback semantics, but the bus exists for volume control), ui (HUD clicks).
-->
<scope>
  props: {
    src = ''
    on = ''
    volume = 1
    'pitch-range' = '1..1'
    bus: 'sfx' | 'music' | 'ui' = 'sfx'
  }
</scope>
