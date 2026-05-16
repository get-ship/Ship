import { controller, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface Room {
  key: string
  label: string
  desc: string
  icon: string
  path: string
}

const TRELLIS: readonly Room[] = [
  { key: 'board', label: 'Board',
    desc: 'Kanban with drag-and-drop, keyed lists, enter/leave transitions, card drawer.',
    icon: 'columns', path: '/harness/board' },
  { key: 'inbox', label: 'Inbox',
    desc: 'Virtualized feed, simulated live patches, filter chips, mark-read animation.',
    icon: 'inbox', path: '/harness/inbox' },
  { key: 'sync', label: 'Sync',
    desc: 'Real shipd live data. Subscribe, mutate, race patches, kill the socket.',
    icon: 'radio', path: '/harness/sync' },
]

const LAB: readonly Room[] = [
  { key: 'motion', label: 'Motion',
    desc: 'Chained enter/leave, layout thrash, reduced-motion respect, transition stagger.',
    icon: 'sparkles', path: '/harness/lab/motion' },
  { key: 'forms', label: 'Forms cathedral',
    desc: 'Every input × every validation state. Async validators, paste-fuzz.',
    icon: 'edit', path: '/harness/lab/forms' },
  { key: 'virt', label: 'Virtualization',
    desc: '100k rows. Jump-scroll. Dynamic row height. Live mutation mid-scroll.',
    icon: 'list', path: '/harness/lab/virt' },
  { key: 'modals', label: 'Modal labyrinth',
    desc: '3-deep dialogs + drawer + hover-card + context menu. Focus/escape ordering.',
    icon: 'layers', path: '/harness/lab/modals' },
  { key: 'charts', label: 'Chart zoo',
    desc: 'All chart types, 60Hz mutation source. Streaming line / live bars / spark.',
    icon: 'bar-chart', path: '/harness/lab/charts' },
  { key: 'theme', label: 'Theme hot-swap',
    desc: 'Switch theme tokens mid-animation. Density toggle. Tone overrides.',
    icon: 'palette', path: '/harness/lab/theme' },
]

export default controller<Scope>({
  scope: () => ({
    goNav: (event: Event, path: string): void => {
      event.preventDefault()
      chrome.navigate(path)
    },
  }),
  derived: {
    trellisRooms: () => TRELLIS,
    labRooms: () => LAB,
  },
})
