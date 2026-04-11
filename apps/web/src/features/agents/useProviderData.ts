import { useShipQuery } from '#/features/studio/hooks/useShipQuery'

export interface ProviderStatus {
  id: string
  available: boolean
  binary: string
  version: string | null
}

interface SettingsMap {
  [key: string]: unknown
}

export function useProviderData() {
  const providers = useShipQuery<ProviderStatus[]>('/providers')
  const settings = useShipQuery<SettingsMap>('/settings')

  return {
    providers: providers.data ?? [],
    defaultProvider: (settings.data?.['defaults.provider'] as string) ?? 'claude',
    defaultModel: (settings.data?.['defaults.model'] as string) ?? 'sonnet',
    isLoading: providers.isLoading || settings.isLoading,
  }
}
