import type { AuthRequirement, ViewManifest } from './types'
import { parseJsonc } from './jsonc'

const AUTH_VALUES: ReadonlySet<AuthRequirement> = new Set(['public', 'user', 'admin'])

export class ManifestError extends Error {
  constructor(message: string, public readonly file?: string) {
    super(file ? `${file}: ${message}` : message)
    this.name = 'ManifestError'
  }
}

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null && !Array.isArray(v)
}

function expectString(value: unknown, field: string, file?: string): string {
  if (typeof value !== 'string' || value.length === 0) {
    throw new ManifestError(`${field} must be a non-empty string`, file)
  }
  return value
}

function optString(value: unknown, field: string, file?: string): string | undefined {
  if (value === undefined) return undefined
  if (typeof value !== 'string' || value.length === 0) {
    throw new ManifestError(`${field} must be a non-empty string when present`, file)
  }
  return value
}

function optStringArray(value: unknown, field: string, file?: string): string[] | undefined {
  if (value === undefined) return undefined
  if (!Array.isArray(value)) {
    throw new ManifestError(`${field} must be an array of strings when present`, file)
  }
  return value.map((v, i) => {
    if (typeof v !== 'string' || v.length === 0) {
      throw new ManifestError(`${field}[${i}] must be a non-empty string`, file)
    }
    return v
  })
}

export function validateManifest(raw: unknown, file?: string): ViewManifest {
  if (!isPlainObject(raw)) {
    throw new ManifestError('manifest root must be a JSON object', file)
  }

  const name = expectString(raw.name, 'name', file)
  const path = expectString(raw.path, 'path', file)
  if (!path.startsWith('/')) {
    throw new ManifestError(`path must start with "/": got "${path}"`, file)
  }

  const manifest: ViewManifest = { name, path }

  const title = optString(raw.title, 'title', file)
  if (title !== undefined) manifest.title = title

  const layout = optString(raw.layout, 'layout', file)
  if (layout !== undefined) manifest.layout = layout

  if (raw.auth !== undefined) {
    if (typeof raw.auth !== 'string' || !AUTH_VALUES.has(raw.auth as AuthRequirement)) {
      throw new ManifestError(
        `auth must be one of "public" | "user" | "admin"; got ${JSON.stringify(raw.auth)}`,
        file,
      )
    }
    manifest.auth = raw.auth as AuthRequirement
  }

  if (raw.rail !== undefined) {
    if (!isPlainObject(raw.rail)) {
      throw new ManifestError('rail must be an object when present', file)
    }
    manifest.rail = {
      label: expectString(raw.rail.label, 'rail.label', file),
      icon: expectString(raw.rail.icon, 'rail.icon', file),
    }
  }

  const preload = optStringArray(raw.preload, 'preload', file)
  if (preload !== undefined) manifest.preload = preload

  const events = optStringArray(raw.events, 'events', file)
  if (events !== undefined) manifest.events = events

  return manifest
}

export function parseManifest(text: string, file?: string): ViewManifest {
  let raw: unknown
  try {
    raw = parseJsonc(text)
  } catch (e) {
    throw new ManifestError(`invalid JSONC: ${(e as Error).message}`, file)
  }
  return validateManifest(raw, file)
}
