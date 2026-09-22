<!--
  <sui-stage-input> — declarative key→action binding for the enclosing <sui-stage>.

  Multiple inputs per stage; each maps a comma-separated set of keys to one
  logical action. The stage queries inputs on init and registers them with
  its InputBus.

  Key names follow KeyboardEvent.key (case-insensitive for letters):
    "ArrowLeft", "A", "Space", "Enter", "Escape", or "GamepadButton0..".
  Convenience aliases recognized: Up/Down/Left/Right/Esc/Space.
-->
<scope>
  props: {
    map = ''
    action = ''
    repeat = true
  }
</scope>
