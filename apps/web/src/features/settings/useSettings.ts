import { useCallback, useRef, useState } from 'react'
import { useShipQuery } from '#/features/studio/hooks/useShipQuery'
import { daemonFetch } from '#/lib/daemon-config'

export interface ProviderStatus {
  id: string
  available: boolean
  binary: string
  version: string | null
}

type SettingsMap = Record<string, string>

export function useSettings() {
  const settings = useShipQuery<SettingsMap>('/settings')
  const providers = useShipQuery<ProviderStatus[]>('/providers')

  const [saving, setSaving] = useState<string | null>(null)
  const [saved, setSaved] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  const updateSetting = useCallback(
    async (key: string, value: string) => {
      setSaving(key)
      setError(null)
      try {
        const res = await daemonFetch('/api/settings', {
          method: 'PUT',
          body: JSON.stringify({ key, value }),
        })
        if (!res.ok) {
          const body = await res.json().catch(() => ({ error: res.statusText })) as { error?: string }
          throw new Error(body.error ?? 'Failed to save')
        }
        settings.refetch()
        setSaved(key)
        clearTimeout(timerRef.current)
        timerRef.current = setTimeout(() => setSaved(null), 1500)
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to save')
      } finally {
        setSaving(null)
      }
    },
    [settings],
  )

  return { settings, providers, updateSetting, saving, saved, error }
}
