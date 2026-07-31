// Hub shim for the shell's `readSetting` / `writeSetting` helpers
// (`apps/shell/src/ops.ts`). The shell's detail controller imports
// them from `../../../../ops`; the hub has no `ops.ts`, so they live
// here as a sibling helper file. Bodies are byte-identical to the
// shell's — only the import seam moved.
//
// The helpers take `ops` (the unary Connect client) as an explicit
// argument so the file itself needs no relative `src/lib/client`
// import. The detail controller passes `ctx.ops` — the same
// ambient client every other hub view reads from its mount context.
// The `SettingScope` is RUNTIME because the active agent id is
// machine-local.
import { SettingScope, type ShipConnectClient } from '@ship/sdk/connect'

/** Read one RUNTIME setting, parsed from its JSON bytes. `null` when unset. */
export async function readSetting<T = unknown>(
  ops: ShipConnectClient,
  key: string,
): Promise<T | null> {
  try {
    const setting = await ops.settings.get({ key, scope: SettingScope.RUNTIME })
    if (setting.valueJson.length === 0) return null
    return JSON.parse(new TextDecoder().decode(setting.valueJson)) as T
  } catch {
    return null
  }
}

/** Write one RUNTIME setting as JSON bytes. */
export async function writeSetting(
  ops: ShipConnectClient,
  key: string,
  value: unknown,
): Promise<void> {
  await ops.settings.set({
    key,
    valueJson: new TextEncoder().encode(JSON.stringify(value)),
    scope: SettingScope.RUNTIME,
  })
}
