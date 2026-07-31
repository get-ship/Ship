<!--
  <ship-error-state> — @ship/shared · the canonical app-level error and
  not-found surface.

  RUNTIME-FREE, presentational. A token-themed card with a variant-driven
  icon, title + body (prop or slot), and a recovery affordance that the
  HOST supplies: set `href` to render an SPA link to a known surface, or
  project a button/link into the `action` slot. The component owns NO
  navigation or data behaviour — it has no controller and imports neither
  @ship/sail-runtime nor the SDK. The consumer (which has runtime access)
  decides how the user recovers.

  The hub routes its route-level boundary fallback through this one
  component — one SSOT, one chrome, one way for the user to recover.
  Token-themed: every paint is driven by design-system tokens
  (var(--space-*), var(--foreground), var(--muted-foreground),
  var(--destructive), var(--border), var(--shadow-*), var(--radius-*)).

  Prop surface
  ─────────────────────────────────────────────────────────────────────
  title  (string, '')        Reflected. Defaults to "Not found" or
                             "Something went wrong" by variant.
  body   (string, '')        Default-slot fallback when no body slot
                             content is projected.
  variant  ('not-found' | 'error', 'not-found', reflect)
                             - 'not-found'  → icon "map-pin-off", muted
                               chrome
                             - 'error'      → icon "alert-octagon",
                               destructive halo
  href   (string, '')        Reflected. When set, the primary action is
                             an SPA link to that path. Empty → no built-in
                             primary action (host supplies one via slot).
  action-label (string, 'Go back')
                             Reflected. Copy for the `href` link.

  Slots
  ─────────────────────────────────────────────────────────────────────
  title         Replaces the default title.
  (default)     Replaces the default body.
  action        Host-supplied recovery action(s). Rendered alongside the
                `href` link when both are present, or on their own when
                `href` is empty. Behaviour (retry/back/reload) lives with
                the host, which owns the runtime.

  Parts: card, icon, title, body, primary-action, actions
-->
<!-- <prop name="title" type="string" default="" reflect /> -->
<!-- <prop name="body" type="string" default="" /> -->
<!-- <prop name="variant" values="not-found|error" default="not-found" reflect /> -->
<!-- <prop name="href" type="string" default="" reflect /> -->
<!-- <prop name="action-label" type="string" default="Go back" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<div role="alert" aria-live="polite">
  <div aria-hidden="true">
    <icon
      name={variant() === 'error' ? 'alert-octagon' : 'map-pin-off'}
      size="md"
    ></icon>
  </div>

  <h1>
    <slot name="title">{title() || (variant() === 'error' ? 'Something went wrong' : 'Not found')}</slot>
  </h1>

  <p>
    <slot>{body() || (variant() === 'not-found'
      ? "We couldn't find that page. Check the URL or head back to a known surface."
      : 'The page failed to load. Try again, or head back.')}</slot>
  </p>

  <div>
    <if cond={href()}>
      <a
         href={href()}
         data-test="primary-action">{actionLabel()}</a>
    </if>

    <slot name="action"></slot>
  </div>
</div>
