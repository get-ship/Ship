<!--
  Space route — placeholder for the file-routing launch surface.

  The Tauri host launches one window per space (`--app=<id>`); the
  placeholder's only job is to prove the routes/ convention resolves
  inside the spaces app: deck regenerates `apps/spaces/routes.gen.ts`
  from this dir, the runtime registers `/[appId]` as a dynamic
  pattern, and the spaces scaffold's main.ts reads `appId` from
  `route.params.appId()` to address the active space.
-->
<div data-test="space-view">
  <main data-test="space-content">
    <section>
      <h1>Space</h1>
      <p>
        File-routing placeholder for the launched space. The routes/
        convention is in place; a future pass wires each space id to
        its configured chrome + view set.
      </p>
    </section>
  </main>
</div>
