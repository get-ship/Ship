// ConnectRPC client for the Ship daemon op surface (ship.ops.v1).
//
// One source: proto/ship/ops/v1/*.proto, generated from the registry by
// `cargo xtask gen-ops`, compiled to TS by buf (`buf.gen.sdk.yaml` → ./gen).
// This is the Connect half of @ship/sdk — consumers move here off the
// fetch/SSE client (client.ts/fetch.ts) as part of the Connect-all-in
// migration (#72 flips the shell). Pure transport: the server-side gate
// (`services::gate` via `command::call`) enforces every call.
//
// The `reads`/`commands` monolith clients are the OLD generated path; they
// stay only until the GUI flip retargets every consumer onto the per-domain
// clients below, after which Phase C deletes them.

import { createClient, type Client, type Interceptor } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";

import { OpsService } from "./gen/ship/ops/v1/ops_pb.js";
import { StreamService } from "./gen/ship/ops/v1/stream_pb.js";
// Per-domain proto-as-SSOT services (each its own service, not on ReadOps).
import { AccountService } from "./gen/ship/ops/v1/account_pb.js";
import { AgentsService } from "./gen/ship/ops/v1/agents_pb.js";
import { AnalyticsService } from "./gen/ship/ops/v1/analytics_pb.js";
import { ArtifactsService } from "./gen/ship/ops/v1/artifacts_pb.js";
import { CollectionsService } from "./gen/ship/ops/v1/collections_pb.js";
import { EventsService } from "./gen/ship/ops/v1/events_pb.js";
import { IsolationService } from "./gen/ship/ops/v1/isolation_pb.js";
import {
  type IsolationCapabilities,
  type IsolationDecision,
  type IsolationProfile,
  type IsolationPreviewResponse,
  type IsolationRecommendationsResponse,
  type IsolationResolved,
  type IsolationResolutionReason,
  type IsolationStatusResponse,
  type IsolationStrategyRow,
  type IsolationTierRow,
  type IsolationVerifyResponse,
  type IsolationVerifyStep,
} from "./gen/ship/ops/v1/isolation_pb.js";
import { EntityKind, type SearchHit, type SearchResponse } from "./gen/ship/ops/v1/search_pb.js";
import { McpService } from "./gen/ship/ops/v1/mcp_pb.js";
import { PermissionsService } from "./gen/ship/ops/v1/permissions_pb.js";
import { PipelinesService } from "./gen/ship/ops/v1/pipelines_pb.js";
import { RulesService } from "./gen/ship/ops/v1/rules_pb.js";
import { RuntimeService } from "./gen/ship/ops/v1/runtime_pb.js";
import { SettingsService, SettingScope } from "./gen/ship/ops/v1/settings_pb.js";
import type { Setting, SettingList } from "./gen/ship/ops/v1/settings_pb.js";
import { SkillsService } from "./gen/ship/ops/v1/skills_pb.js";
import { SearchService } from "./gen/ship/ops/v1/search_pb.js";
import { SpaceService } from "./gen/ship/ops/v1/space_pb.js";
import { UiService } from "./gen/ship/ops/v1/ui_pb.js";

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
  /** Daemon meta (`OpsService`). */
  meta: Client<typeof OpsService>;
  /** Live data stream (`StreamService`) — backs `<data live>`. */
  stream: Client<typeof StreamService>;
  // Per-domain clients — the flip target.
  account: Client<typeof AccountService>;
  agents: Client<typeof AgentsService>;
  analytics: Client<typeof AnalyticsService>;
  artifacts: Client<typeof ArtifactsService>;
  collections: Client<typeof CollectionsService>;
  events: Client<typeof EventsService>;
  isolation: Client<typeof IsolationService>;
  mcp: Client<typeof McpService>;
  permissions: Client<typeof PermissionsService>;
  pipelines: Client<typeof PipelinesService>;
  rules: Client<typeof RulesService>;
  runtime: Client<typeof RuntimeService>;
  search: Client<typeof SearchService>;
  settings: Client<typeof SettingsService>;
  skills: Client<typeof SkillsService>;
  space: Client<typeof SpaceService>;
  /** Unified authoring service (`UiService`): component discovery (Packs, Read),
   *  SUI compiler analysis (Diagnostics, Hover, Dependencies), and view
   *  authoring writes (PatchSession, AppendSession, SignalSession, EditMode,
   *  Tool, Scaffold). Replaces the retired `SuiService` + `ViewService` pair. */
  ui: Client<typeof UiService>;
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
    ...(opts.fetch ? { fetch: opts.fetch } : {}),
    interceptors: opts.token ? [bearer(opts.token)] : [],
  });
  return {
    meta: createClient(OpsService, transport),
    stream: createClient(StreamService, transport),
    account: createClient(AccountService, transport),
    agents: createClient(AgentsService, transport),
    analytics: createClient(AnalyticsService, transport),
    artifacts: createClient(ArtifactsService, transport),
    collections: createClient(CollectionsService, transport),
    events: createClient(EventsService, transport),
    isolation: createClient(IsolationService, transport),
    mcp: createClient(McpService, transport),
    permissions: createClient(PermissionsService, transport),
    pipelines: createClient(PipelinesService, transport),
    rules: createClient(RulesService, transport),
    search: createClient(SearchService, transport),
    runtime: createClient(RuntimeService, transport),
    settings: createClient(SettingsService, transport),
    skills: createClient(SkillsService, transport),
    space: createClient(SpaceService, transport),
    ui: createClient(UiService, transport),
  };
}

export {
  OpsService,
  StreamService,
  AccountService,
  AgentsService,
  AnalyticsService,
  ArtifactsService,
  CollectionsService,
  EventsService,
  IsolationService,
  McpService,
  PermissionsService,
  PipelinesService,
  RulesService,
  SearchService,
  RuntimeService,
  SettingsService,
  SettingScope,
  SkillsService,
  SpaceService,
  UiService,
};

export {
  EntityKind,
  type IsolationCapabilities,
  type IsolationDecision,
  type IsolationProfile,
  type IsolationPreviewResponse,
  type IsolationRecommendationsResponse,
  type IsolationResolved,
  type IsolationResolutionReason,
  type IsolationStatusResponse,
  type IsolationStrategyRow,
  type IsolationTierRow,
  type IsolationVerifyResponse,
  type IsolationVerifyStep,
  type SearchHit,
  type SearchResponse as SearchResponseProto,
  type Setting,
  type SettingList,
};
