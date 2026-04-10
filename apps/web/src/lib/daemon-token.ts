import { readFile } from 'node:fs/promises'
import { homedir } from 'node:os'
import { join } from 'node:path'

/** Read daemon token from ~/.ship/daemon.token. Server-side only. */
export async function readDaemonToken(): Promise<string | null> {
  try {
    const tokenPath = join(homedir(), '.ship', 'daemon.token')
    const token = await readFile(tokenPath, 'utf-8')
    return token.trim()
  } catch {
    return null
  }
}
