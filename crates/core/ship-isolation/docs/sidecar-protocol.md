# Sidecar Protocol — v1 RFC

**Status:** schema frozen, scaffold landed, sidecar handler rewrite
pending.

**Schema:** [`proto/ship/sidecar/v1/sidecar.proto`](../../../../proto/ship/sidecar/v1/sidecar.proto)
(canonical). Rust + TS bindings generated via ConnectRPC + Buf.

**Wire path:** `/ship.sidecar.v1.<Service>/<Method>` per Connect convention.

## Why ConnectRPC

The host ↔ sidecar channel is bidirectional and long-lived. PTY needs
binary streaming both ways, permission requests need request/reply
correlation, hook callbacks are server-initiated, credential fetches
are client-initiated. Three real options surveyed:

| | HTTP+JSON+SSE+WS | tarpc | **ConnectRPC** |
|---|---|---|---|
| Bidi streaming | WS only (separate transport) | Native | First-class |
| Schema | Hand-typed JSON | Rust types | Protobuf |
| Polyglot SDK | Yes (multi-codebase types) | Rust only | Yes (any language) |
| Already in repo? | Partial | No | **Yes** (gateway-client) |
| Versioning hygiene | Manual | Manual | Protobuf field-number rules |

The polyglot story matters because tier-3 integration ("ADD
`ship-sidecar` to my Dockerfile") implies users will drive sidecars
from non-Rust stacks — TS/Python/Go cloud functions, CI workflows,
internal PaaSes. Connect gives that for free via Buf-generated
clients in any language. Hand-typed JSON would re-type every shape
per language; tarpc would lock the wire format to Rust.

The codegen cost is amortized because we already pay it on the
gateway-client side.

## Tier coverage

```
Tier 1  HarnessNative / SrtWrapper    → in-process; no sidecar
Tier 2  DockerEngine / YoukiRootless  → sidecar in container; unix socket
Tier 3  LimaVm                        → sidecar in microVM; vsock
Tier 4  FlyMachines + other vendors   → sidecar in vendor sandbox; TCP+TLS
```

One protocol; the host adapts the transport. Same generated Rust
client wraps each.

## Transport per tier

| Tier | Transport | Auth |
|---|---|---|
| 2 (local container) | Unix socket bind-mounted at `/run/ship-sidecar.sock` | per-socket — the socket itself is the auth |
| 3 (local microVM) | vsock listener; host connects on the VM's CID | per-CID |
| 4 (cloud) | TCP + TLS on the sandbox's public address | `Authorization: Bearer $SHIP_SIDECAR_TOKEN` |

The HTTP layer and the protobuf payloads are identical across all
three.

## Versioning discipline

Sidecars MUST report their `protocol_version` and `capabilities` in
the `Handshake.Hello` response. Hosts:

- read the version; refuse to connect if the major mismatches
- read capabilities; never call a method whose capability isn't listed
- ignore unknown fields in any response (forward-compatible reads)

Protocol changes:

- **Additive** — new fields with new field numbers, new methods, new
  enum values (with `_UNSPECIFIED = 0` sentinel maintained). v1 stays
  v1; old hosts/sidecars work unchanged.
- **Breaking** — only under a new package (`ship.sidecar.v2`).
  Sidecars supporting both versions advertise both in `capabilities`;
  hosts negotiate down. Overlap window lasts at least one minor
  release cycle.

This means **we do not need dynamic protocol fetch at handshake.** The
schema is stable by discipline, not by runtime indirection.

## What's intentionally NOT in v1

These were considered and excluded:

- **`/fs/*` endpoints.** Workspace + creds bind-mount from host to
  container/VM. Cloud Tier 4 uses the vendor's equivalent volume
  primitive. No filesystem-over-the-wire surface.
- **Agent install.** The container image bakes harness CLIs. "BYO
  image" power users add `ship-sidecar` to their Dockerfile via
  `ADD https://github.com/ship/.../releases/.../ship-sidecar-...`;
  the sidecar doesn't install anything at runtime.
- **Desktop / X11 / WebRTC.** Out of scope.
- **`/metrics/*`.** Process state is `Processes.Get`/`List`. Richer
  metrics belong in the daemon's analytics path, not the sidecar.

## Surfaces in v1

See the .proto for canonical signatures. Summary:

| Service | RPC | Direction | Stream | Purpose |
|---|---|---|---|---|
| `Handshake` | `Hello` | host→sc | unary | capability negotiation |
| `Processes` | `Create` | host→sc | unary | spawn long-lived child |
| `Processes` | `Run` | host→sc | unary | one-shot exec with stdout/stderr/exit |
| `Processes` | `List` / `Get` | host→sc | unary | enumerate / inspect |
| `Processes` | `Stop` / `Kill` | host→sc | unary | lifecycle |
| `Processes` | `WriteInput` | host→sc | unary | stdin / PTY master writes |
| `Processes` | `ResizeTerminal` | host→sc | unary | PTY resize |
| `Processes` | `AttachPty` | host→sc | server-stream | PTY output as `PtyChunk`s |
| `Events` | `Stream` | both | bidi | universal envelope channel (stdout, exit, permission, secret, settings, hooks, log_progress) |

## The `Envelope` channel

Replaces what would have been 5+ separate endpoints
(`/v1/permission/*`, `/v1/secret/*`, `/v1/hook/*`,
`/v1/log_progress`, `/v1/settings/*`). Single bidi stream multiplexed
via `Envelope.request_id` for request/reply pairs. Sequence numbers
for replay/audit.

Sender-initiated payload types (sidecar → host):
- `process_output` — non-PTY stdout/stderr chunks
- `process_exited` — exit code or signal
- `permission_request` — agent asks before running a risky tool
- `secret_request` — agent needs a credential (broker resolves via
  shipd's keychain bridge; secret never lands in container disk)
- `setting_request` — agent reads a daemon setting via shipd
- `hook_fired` — sidecar dispatches a configured hook back to shipd
- `log_progress` — structured progress note (via MCP today)

Host-initiated payload types (host → sidecar):
- `permission_reply` — `allow` / `allow_once` / `deny`
- `secret_reply` — resolved value, never logged
- `setting_reply` — JSON value
- `hook_return` — shipd's hook handler response

The `request_id` field correlates pairs without requiring per-concern
endpoints. Adding a new request/reply pair is a `oneof` variant in
the proto, not a new HTTP route.

## Implementation map

| Piece | Status |
|---|---|
| `proto/ship/sidecar/v1/sidecar.proto` | landed |
| Rust client + server bindings (codegen) | landed; 7 smoke tests in `tests/proto_v1.rs` |
| TS bindings | wire via Buf when the SDK consumer lands |
| Sidecar binary handlers (`bin/ship-sidecar/`) | uses legacy hand-typed JSON; rewrite to Connect routing is the next chunk |
| Host-side ConnectRPC client in backends | not yet — pending sidecar handler rewrite |
| Versioning policy | this document |
