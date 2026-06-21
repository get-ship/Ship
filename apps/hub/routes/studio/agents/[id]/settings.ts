// Hub shim for the shell's `readSetting` / `writeSetting` helpers
// (`apps/shell/src/ops.ts`). The shell's detail controller imports
// them from `../../../../ops`; the hub has no `ops.ts`, so they live
// here as a sibling helper file. Bodies are byte-identical to the
// shell's — only the import seam moved. Connect-target is the hub's
// `createShipConnectClient` singleton (the same `ship` every other
// hub controller uses), which is wired to the same daemon the shell
// talks to in dev. The `SettingScope` is RUNTIME because the active
// agent id is machine-local.
import { SettingScope } from '@ship/sdk/connect'
import { ship } from '../../../../src/lib/client'

/** Read one RUNTIME setting, parsed from its JSON bytes. `null` when unset. */
export async function readSetting<T = unknown>(key: string): Promise<T | null> {
  try {
    const setting = await ship.settings.get({ key, scope: SettingScope.RUNTIME })
    if (setting.valueJson.length === 0) return null
    return JSON.parse(new TextDecoder().decode(setting.valueJson)) as T
  } catch {
    return null
  }
}

/** Write one RUNTIME setting as JSON bytes. */
export async function writeSetting(key: string, value: unknown): Promise<void> {
  await ship.settings.set({
    key,
    valueJson: new TextEncoder().encode(JSON.stringify(value)),
    scope: SettingScope.RUNTIME,
  })
}
