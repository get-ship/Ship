/**
 * Connect-RPC implementation of `ship.gateway.v1.RegistryService` — the
 * redesigned signed-bundle registry. Typed metadata + attestations ride
 * Connect; bundle BYTES ride the `download_url` the response carries (today the
 * package's source tarball; migrates to an R2/CDN object per the distribution
 * design). The attestation binds the registry `contentHash` — the canonical
 * tree hash the daemon cache also keys by — so registry ↔ attestation ↔ cache
 * are one content hash.
 *
 * `publishBundle` is deliberately Unimplemented here: the legacy publish proxy
 * (`ops/v1` RegistryService.Publish → REST) still serves publishing; the
 * redesigned signed-bundle publish (+ R2 upload) is a later slice. Returning
 * Unimplemented is the honest contract — not a stub that pretends to store.
 */
import { Code, ConnectError, type HandlerContext } from '@connectrpc/connect'
import { create } from '@bufbuild/protobuf'
import type { ServiceImpl } from '@connectrpc/connect'
import {
  RegistryService,
  type ResolveVersionRequest,
  type ResolveVersionResponse,
  ResolveVersionResponseSchema,
  type FetchBundleRequest,
  type FetchBundleResponse,
  FetchBundleResponseSchema,
  type PublishBundleRequest,
  type PublishBundleResponse,
} from '../proto/gen/ship/gateway/v1/registry_pb.js'
import { envContextKey } from '../lib/connect-context.js'
import { getRegistryDb } from '../lib/d1.js'
import { createRegistryRepositories } from '../data/registry.repository.js'
import { compareSemver } from '../lib/semver.js'
import { signBundleAttestation } from '../lib/bundle-attestation.js'

function requireEnv(ctx: HandlerContext): Env {
  const env = ctx.values.get(envContextKey)
  if (!env) throw new ConnectError('env binding not injected', Code.Internal)
  return env
}

function repos(env: Env) {
  const d1 = getRegistryDb(env)
  if (!d1) throw new ConnectError('registry database not bound', Code.Internal)
  return createRegistryRepositories(d1)
}

/** GitHub source tarball for a commit — the bundle bytes the daemon fetches +
 *  tree-hashes. Migrates to a signed R2/CDN URL when bundle storage lands. */
function sourceTarballUrl(repoUrl: string, commitSha: string): string {
  const u = new URL(repoUrl)
  if (u.hostname !== 'github.com') {
    throw new ConnectError(`unsupported source host: ${u.hostname}`, Code.FailedPrecondition)
  }
  const [, owner, repo] = u.pathname.replace(/\.git$/, '').split('/')
  return `https://api.github.com/repos/${owner}/${repo}/tarball/${commitSha}`
}

async function resolveVersion(
  req: ResolveVersionRequest,
  ctx: HandlerContext,
): Promise<ResolveVersionResponse> {
  const r = repos(requireEnv(ctx))
  const pkg = await r.getPackage(req.path)
  if (!pkg) throw new ConnectError(`no package '${req.path}'`, Code.NotFound)

  const versions = await r.getPackageVersions(pkg.id)
  if (versions.length === 0) {
    throw new ConnectError(`no published versions for '${req.path}'`, Code.NotFound)
  }
  // Pick the highest published version. A `constraint` of "" / "*" means "latest";
  // an exact "x.y.z" selects that version. Full caret/tilde range matching is a
  // follow-up — `constraint` is forwarded but only exact/latest is honored now.
  const constraint = req.constraint.trim()
  const candidates =
    constraint && constraint !== '*'
      ? versions.filter((v) => v.version === constraint)
      : versions
  const chosen = candidates.sort((a, b) => compareSemver(b.version, a.version))[0]
  if (!chosen) throw new ConnectError(`no version matching '${constraint}'`, Code.NotFound)
  if (!chosen.contentHash) {
    throw new ConnectError(`version '${chosen.version}' has no content hash`, Code.Internal)
  }

  return create(ResolveVersionResponseSchema, {
    version: chosen.version,
    sha256: chosen.contentHash,
  })
}

async function fetchBundle(
  req: FetchBundleRequest,
  ctx: HandlerContext,
): Promise<FetchBundleResponse> {
  const env = requireEnv(ctx)
  const r = repos(env)
  const pkg = await r.getPackage(req.path)
  if (!pkg) throw new ConnectError(`no package '${req.path}'`, Code.NotFound)

  const version = (await r.getPackageVersions(pkg.id)).find((v) => v.version === req.version)
  if (!version) {
    throw new ConnectError(`no version '${req.version}' of '${req.path}'`, Code.NotFound)
  }
  if (!version.contentHash) {
    throw new ConnectError(`version '${req.version}' has no content hash`, Code.Internal)
  }

  const downloadUrl = sourceTarballUrl(pkg.repoUrl, version.commitSha)
  const attestationJwt = await signBundleAttestation({
    env,
    path: req.path,
    version: req.version,
    sha256: version.contentHash,
  })

  return create(FetchBundleResponseSchema, {
    downloadUrl,
    attestationJwt,
    sha256: version.contentHash,
  })
}

async function publishBundle(
  _req: PublishBundleRequest,
  _ctx: HandlerContext,
): Promise<PublishBundleResponse> {
  // The redesigned signed-bundle publish (+ R2 upload) is a later slice; the
  // legacy publish proxy still serves publishing today.
  throw new ConnectError('signed-bundle publish not yet implemented', Code.Unimplemented)
}

export const registryServiceImpl: ServiceImpl<typeof RegistryService> = {
  resolveVersion,
  fetchBundle,
  publishBundle,
}
