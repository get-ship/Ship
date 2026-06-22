// Hub home controller — the index route's view scope.
//
// The home is minimal, static chrome: a time-of-day greeting and navigation
// into the hub's surfaces (plain `<a href>` cards — the router intercepts
// same-origin clicks for SPA nav, so no handler is needed). It makes NO SDK
// calls — the spaces list is a backend gap (see .artifacts/hub-buildout-
// flags.md) so the page shows an empty-state; keeping it SDK-free also means
// the home can't trigger the live-read retry path.
import { controller, derived } from '@ship/sui-runtime'

export default controller({
  scope: () => {
    // Time-of-day greeting (pure derived; client-side clock).
    const greeting = derived<string>(() => {
      const h = new Date().getHours()
      return h < 5 ? 'Good night'
        : h < 12 ? 'Good morning'
          : h < 18 ? 'Good afternoon'
            : 'Good evening'
    })
    return { greeting }
  },
})
