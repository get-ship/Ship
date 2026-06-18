// app.jsx — composes DesignCanvas with all artboards

const { useState, useEffect } = React;

function App() {
  return (
    <DesignCanvas
      title="Spaces · interaction model"
      subtitle="Ship hub + space-per-window. Customization vectors and component surface."
    >
      <DCSection
        id="hub"
        title="Ship Hub"
        subtitle="The new Ship.app. System hub for spaces, studio, marketplace, account, settings, activity."
      >
        <DCArtboard id="hub-spaces" label="A · Spaces (default)" width={HUB_W} height={HUB_H}>
          <HubSpaces />
        </DCArtboard>
        <DCArtboard id="hub-activity" label="B · Activity stream" width={HUB_W} height={HUB_H}>
          <HubActivity />
        </DCArtboard>
      </DCSection>

      <DCSection
        id="onboarding"
        title="First-run + create-space flow"
        subtitle="Wizard sequence. Ship handles host-OS registration automatically."
      >
        <DCArtboard id="onb-welcome" label="0 · Welcome" width={WIZ_W} height={WIZ_H}>
          <OnbWelcome />
        </DCArtboard>
        <DCArtboard id="onb-step1" label="1 · Name + icon" width={WIZ_W} height={WIZ_H}>
          <OnbStep1 />
        </DCArtboard>
        <DCArtboard id="onb-step2" label="2 · Theme + chrome prefs" width={WIZ_W} height={WIZ_H}>
          <OnbStep2 />
        </DCArtboard>
        <DCArtboard id="onb-step3" label="3 · Registered" width={WIZ_W} height={WIZ_H}>
          <OnbStep3 />
        </DCArtboard>
      </DCSection>

      <DCSection
        id="windows"
        title="Spaces, running"
        subtitle="Two example spaces in the wild — each its own OS window, its own theme & chrome."
      >
        <DCArtboard id="space-ship" label="Ship · dev (ship-dark)" width={SPACE_W} height={SPACE_H}>
          <SpaceShip />
        </DCArtboard>
        <DCArtboard id="space-paper" label="Paperback · publishing (paper)" width={SPACE_W} height={SPACE_H}>
          <SpacePaper />
        </DCArtboard>
      </DCSection>

      <DCSection
        id="surface"
        title="The component surface"
        subtitle="How a space is declared, what it can customize, and where users tune it."
      >
        <DCArtboard id="surface-vectors" label="Reference · customization vectors" width={920} height={560}>
          <VectorsArt />
        </DCArtboard>
        <DCArtboard id="surface-manifest" label="space.jsonc · declarative model" width={SURF_W} height={SURF_H}>
          <ManifestArt />
        </DCArtboard>
        <DCArtboard id="surface-editor" label="Studio · live editor (single-window, v1)" width={SURF_W} height={SURF_H}>
          <StudioEditor />
        </DCArtboard>
      </DCSection>

      <DCSection
        id="paired"
        title="Editing surface · the two-window pattern"
        subtitle="The hub holds the IDE; the space holds the runtime. Edits flow into the live window."
      >
        <DCArtboard id="paired-hero" label="0 · Pattern" width={HERO_W} height={HERO_H}>
          <PairedHero />
        </DCArtboard>
        <DCArtboard id="paired-spaces" label="A · Space chrome · paperback" width={PAIRED_W} height={PAIRED_H}>
          <PairedSpaces />
        </DCArtboard>
        <DCArtboard id="paired-pipelines" label="B · Pipelines · overnight run" width={PAIRED_W} height={PAIRED_H}>
          <PairedPipelines />
        </DCArtboard>
        <DCArtboard id="paired-skills" label="C · Agents &amp; skills · doc lane" width={PAIRED_W} height={PAIRED_H}>
          <PairedSkills />
        </DCArtboard>
      </DCSection>

      <DCSection
        id="authority"
        title="The hub as authority"
        subtitle="Spaces ask; the hub decides. Elevated commands, capability grants, and identity all live here — never inside a running space."
      >
        <DCArtboard id="auth-queue" label="A · Authority queue" width={AUTH_W} height={AUTH_H}>
          <HubAuthority />
        </DCArtboard>
        <DCArtboard id="auth-capability" label="B · Capability request" width={AUTH_W} height={AUTH_H}>
          <HubCapabilityRequest />
        </DCArtboard>
        <DCArtboard id="auth-command" label="C · Command approval (high-risk)" width={AUTH_W} height={AUTH_H}>
          <HubCommandApproval />
        </DCArtboard>
        <DCArtboard id="auth-identity" label="D · Hub locked · linked identities" width={AUTH_W} height={AUTH_H}>
          <HubIdentity />
        </DCArtboard>
      </DCSection>
    </DesignCanvas>
  );
}

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(<App />);
