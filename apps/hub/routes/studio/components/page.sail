<!--
  /studio/components — the component catalog index.

  The surface is the <ship-component-catalog> component (@ship/studio,
  src/studio/): it reads the build-time CEM catalog (`components.gen.ts`),
  groups every pack's components, shows each component's CEM metadata, and
  live-mounts a bare instance of any tag the bundle registered. The route is
  thin transport — chrome + the component; all catalog logic lives in the
  component.
-->
<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <ship-component-catalog></ship-component-catalog>
  </main>
</div>
