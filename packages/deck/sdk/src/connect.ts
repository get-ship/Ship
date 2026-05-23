// ConnectRPC client for the Ship daemon op surface (ship.ops.v1).
//
// One source: proto/ship/ops/v1/*.proto, generated from the registry by
// `cargo xtask gen-ops`, compiled to TS by buf (`buf.gen.sdk.yaml` → ./gen).
// This is the Connect half of @ship/sdk — consumers move here off the
// fetch/SSE client (client.ts/fetch.ts) as part of the Connect-all-in
// migration (P3 flips the shell). Pure transport: the server-side gate
// (`services::gate` via `command::call`) enforces every call.

import { createClient, type Client, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";

import { OpsService } from "./gen/ship/ops/v1/ops_pb.js";
import { ReadOpsService } from "./gen/ship/ops/v1/reads.gen_pb.js";
import { CommandOpsService } from "./gen/ship/ops/v1/commands.gen_pb.js";
import { StreamService } from "./gen/ship/ops/v1/stream_pb.js";

export interface ShipConnectOptions {
  /** Daemon base URL, e.g. `http://127.0.0.1:9315`. */
  baseUrl: string;
  /** Bearer token: daemon token for the operator, scoped session token for an agent. */
  token?: string;
  /** Override fetch (tests / non-browser hosts). */
  fetch?: typeof globalThis.fetch;
}

/** The daemon op surface, grouped by service. */
export interface ShipConnectClient {
  meta: Client<typeof OpsService>;
  reads: Client<typeof ReadOpsService>;
  commands: Client<typeof CommandOpsService>;
  stream: Client<typeof StreamService>;
}

/** Attach `Authorization: Bearer <token>` to every request. */
function bearer(token: string): Interceptor {
  return (next) => (req) => {
    req.header.set("Authorization", `Bearer ${token}`);
    return next(req);
  };
}

/**
 * Build a typed Connect client for the daemon op surface.
 * Wire paths: `/ship.ops.v1.<Service>/<Method>`.
 */
export function createShipConnectClient(opts: ShipConnectOptions): ShipConnectClient {
  const transport = createConnectTransport({
    baseUrl: opts.baseUrl,
    fetch: opts.fetch,
    interceptors: opts.token ? [bearer(opts.token)] : [],
  });
  return {
    meta: createClient(OpsService, transport),
    reads: createClient(ReadOpsService, transport),
    commands: createClient(CommandOpsService, transport),
    stream: createClient(StreamService, transport),
  };
}

export { OpsService, ReadOpsService, CommandOpsService, StreamService };
