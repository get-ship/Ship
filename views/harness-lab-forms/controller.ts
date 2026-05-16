import { controller, signal, derived, chrome } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type SubmitState = 'idle' | 'submitting' | 'ok' | 'error'

const EMAIL = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

export default controller<Scope>({
  scope: () => {
    const text     = signal('')
    const email    = signal('')
    const num      = signal('')
    const pw       = signal('')
    const notes    = signal('')
    const choice   = signal('a')
    const notify   = signal(true)
    const terms    = signal(false)
    const priority = signal('normal')
    const vol      = signal(50)
    const submitState = signal<SubmitState>('idle')
    const submitError = signal('')
    const asyncMsg    = signal('')
    let pwTimer: ReturnType<typeof setTimeout> | null = null

    const setVal = (field: string, value: unknown): void => {
      const sigMap: Record<string, { set: (v: unknown) => void } | undefined> = {
        text, email, num, pw, notes, choice, notify, terms, priority, vol,
      } as never
      sigMap[field]?.set(value)
    }

    const onPw = (value: string): void => {
      pw.set(value)
      asyncMsg.set('checking…')
      if (pwTimer) clearTimeout(pwTimer)
      pwTimer = setTimeout(() => {
        if (value.length < 8) asyncMsg.set('too short')
        else if (/^password/i.test(value)) asyncMsg.set('seen 10k+ times')
        else asyncMsg.set('strong')
      }, 380)
    }

    const emailError = derived(() => {
      const v = email()
      if (!v) return ''
      return EMAIL.test(v) ? '' : 'invalid email'
    })

    return {
      text, email, num, pw, notes, choice, notify, terms, priority, vol,
      submitState, submitError, asyncMsg, emailError,

      goNav: (e: Event, path: string): void => { e.preventDefault(); chrome.navigate(path) },
      set: setVal,
      onPw,

      submit: (): void => {
        submitState.set('submitting')
        submitError.set('')
        setTimeout(() => {
          if (Math.random() < 0.3) {
            submitState.set('error')
            submitError.set('upstream rejected — retry')
          } else {
            submitState.set('ok')
          }
        }, 700)
      },

      reset: (): void => {
        text.set(''); email.set(''); num.set(''); pw.set('')
        notes.set(''); choice.set('a'); notify.set(true); terms.set(false)
        priority.set('normal'); vol.set(50)
        submitState.set('idle'); submitError.set(''); asyncMsg.set('')
      },
    }
  },

  derived: {
    canSubmit: ({ terms, emailError, submitState }) =>
      terms() && !emailError() && submitState() !== 'submitting',
    snapshot: ({ text, email, num, pw, notes, choice, notify, terms, priority, vol }) =>
      JSON.stringify({
        text: text(), email: email(), num: num(), pw: pw() ? '••••' : '',
        notes: notes(), choice: choice(),
        notify: notify(), terms: terms(), priority: priority(), vol: vol(),
      }, null, 2),
  },
})
