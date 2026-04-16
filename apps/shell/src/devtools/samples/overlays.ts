/**
 * Overlay samples — all use the TriggerSample shape so the gallery
 * renders a button and only opens the overlay on click. Never auto-open.
 */

import type { Sample, TriggerSample } from './types'

function dialogSample(): TriggerSample {
  return {
    label: 'Open dialog',
    buttonTone: 'info',
    caption: 'click → .show()',
    build: () => {
      const dlg = document.createElement('ship-dialog') as HTMLElement & {
        show?: () => void
        close?: () => void
      }
      dlg.innerHTML =
        '<span slot="header">Dialog example</span>' +
        '<p style="margin:0">Dialogs render above the content and trap focus. Escape or backdrop click closes.</p>' +
        '<span slot="footer">' +
        '<ship-button variant="outline" data-close>Cancel</ship-button>' +
        '<ship-button tone="info" data-close>Continue</ship-button>' +
        '</span>'
      dlg.addEventListener('click', (e) => {
        const t = e.target as HTMLElement | null
        if (t?.closest('[data-close]')) dlg.close?.()
      })
      return {
        overlay: dlg,
        portal: true,
        activate: () => dlg.show?.(),
      }
    },
  }
}

function alertDialogSample(): TriggerSample {
  return {
    label: 'Delete…',
    buttonTone: 'destructive',
    buttonVariant: 'soft',
    caption: 'click → open = true',
    build: () => {
      const dlg = document.createElement('ship-alert-dialog') as HTMLElement & {
        open?: boolean
      }
      dlg.setAttribute('tone', 'destructive')
      dlg.setAttribute('confirm-label', 'Delete')
      dlg.innerHTML =
        '<span slot="title">Delete this run?</span>' +
        'This action permanently removes the pipeline run and its logs. It cannot be undone.'
      dlg.addEventListener('ship-confirm', () => (dlg.open = false))
      dlg.addEventListener('ship-cancel', () => (dlg.open = false))
      return {
        overlay: dlg,
        portal: true,
        activate: () => {
          dlg.open = true
        },
      }
    },
  }
}

function popoverSample(): TriggerSample {
  return {
    label: 'Open popover',
    caption: 'click → .toggle()',
    build: () => {
      const pop = document.createElement('ship-popover') as HTMLElement & {
        toggle?: () => void
      }
      pop.innerHTML =
        '<div style="padding:8px;min-width:220px;font-size:var(--text-sm);line-height:var(--leading-normal)">' +
        '<strong style="display:block;margin-bottom:4px">Popover</strong>' +
        'Anchored panel with a fade-in. Click outside or press Escape to close.' +
        '</div>'
      return { overlay: pop, activate: () => pop.toggle?.() }
    },
  }
}

function tooltipSample(): TriggerSample {
  return {
    label: 'Hover me',
    caption: 'hover / focus → tooltip',
    build: () => {
      const tip = document.createElement('ship-tooltip')
      tip.innerHTML =
        '<span slot="content">Tooltips show on hover and keyboard focus.</span>' +
        '<span tabindex="0" style="padding:4px 10px;border:1px dashed var(--border);border-radius:var(--radius-sm);color:var(--muted-foreground);font-size:var(--text-sm);cursor:default">hover here</span>'
      return {
        overlay: tip,
        activate: () => {
          ;(tip.querySelector('[tabindex]') as HTMLElement | null)?.focus()
        },
      }
    },
  }
}

function dropdownSample(): TriggerSample {
  return {
    label: 'Open menu',
    caption: 'click → toggle open',
    build: () => {
      const menu = document.createElement('ship-dropdown-menu') as HTMLElement & {
        open?: boolean
      }
      menu.innerHTML =
        '<ship-button slot="trigger" variant="outline">Menu</ship-button>' +
        '<button role="menuitem" data-value="save">Save</button>' +
        '<button role="menuitem" data-value="rename">Rename…</button>' +
        '<button role="menuitem" data-value="delete">Delete</button>'
      return {
        overlay: menu,
        activate: () => {
          menu.open = !menu.open
        },
      }
    },
  }
}

function toastContainerSample(): TriggerSample {
  return {
    label: 'Show toast',
    buttonTone: 'accent',
    caption: 'click → container.show()',
    build: () => {
      // Single shared portal container keeps toasts stacking. Reuse it
      // across every gallery click so we never spawn duplicate stacks.
      let container = document.body.querySelector<HTMLElement>(
        'ship-toast-container[data-gallery]',
      )
      if (!container) {
        container = document.createElement('ship-toast-container')
        container.setAttribute('data-gallery', '')
        container.setAttribute('position', 'bottom-right')
        document.body.appendChild(container)
      }
      const tones = ['info', 'success', 'warning', 'destructive'] as const
      let i = 0
      return {
        overlay: container,
        portal: true,
        activate: () => {
          const tone = tones[i++ % tones.length]
          const api = container as HTMLElement & {
            show?: (opts: Record<string, unknown>) => void
          }
          api.show?.({
            tone,
            title: `${tone[0].toUpperCase()}${tone.slice(1)} toast`,
            description: 'Programmatic toast fired from the gallery.',
            duration: 3800,
          })
        },
      }
    },
  }
}

export const OVERLAYS: Record<string, Sample[]> = {
  'ship-dialog': [dialogSample()],
  'ship-alert-dialog': [alertDialogSample()],
  'ship-popover': [popoverSample()],
  'ship-tooltip': [tooltipSample()],
  'ship-dropdown-menu': [dropdownSample()],
  'ship-toast-container': [toastContainerSample()],
  // One full-width card with all four tones stacked — toasts have an
  // internal min-width that would overflow cramped grid cells, so show
  // them as a "container-like" stack instead.
  'ship-toast': [
    {
      raw: true,
      innerHTML: [
        '<div class="toast-stack">',
        '<ship-toast tone="info" dismissible><span slot="title">Info toast</span>Short descriptive text for the reader.</ship-toast>',
        '<ship-toast tone="success" dismissible><span slot="title">Saved</span>Changes were persisted successfully.</ship-toast>',
        '<ship-toast tone="warning" dismissible><span slot="title">Quota low</span>Less than 10% disk space remaining on the build host.</ship-toast>',
        '<ship-toast tone="destructive" dismissible><span slot="title">Error</span>Could not contact the daemon at 127.0.0.1:9315.</ship-toast>',
        '</div>',
      ].join(''),
      metaAttrs: { tone: 'info|success|warning|destructive' },
    },
  ],
}
