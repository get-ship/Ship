<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <section>
      <h1>Studio</h1>
      <p>
        Configure this workspace — agents, skills, components, themes and more.
        Pick a section from the rail. Agents is live today; the rest are
        landing as deck views.
      </p>
      <empty-state variant="placeholder" size="md" data-test="studio-landing-empty">
        <icon slot="icon" name="bot"></icon>
        <span slot="title">Start with Agents</span>
        Define and wire the agents available to this project.
        <a slot="action" href="/studio/agents" data-test="studio-landing-agents">
          <icon name="arrow-right" size="xs"></icon>
          Open Agents
        </a>
      </empty-state>
    </section>
  </main>
</div>
