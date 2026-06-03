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
import { AgentService } from "./gen/ship/ops/v1/agent_pb.js";
import { AgentsService } from "./gen/ship/ops/v1/agents_pb.js";
import { AnalyticsService } from "./gen/ship/ops/v1/analytics_pb.js";
import { AnnotationService } from "./gen/ship/ops/v1/annotation_pb.js";
import { ArtifactsService } from "./gen/ship/ops/v1/artifacts_pb.js";
import { ChatService } from "./gen/ship/ops/v1/chat_pb.js";
import { CollectionsService } from "./gen/ship/ops/v1/collections_pb.js";
import { CreateService } from "./gen/ship/ops/v1/create_pb.js";
import { DashboardService } from "./gen/ship/ops/v1/dashboard_pb.js";
import { DiscoveryService } from "./gen/ship/ops/v1/discovery_pb.js";
import { EventsService } from "./gen/ship/ops/v1/events_pb.js";
import { FeedbackService } from "./gen/ship/ops/v1/feedback_pb.js";
import { GitService } from "./gen/ship/ops/v1/git_pb.js";
import { McpService } from "./gen/ship/ops/v1/mcp_pb.js";
import { PermissionsService } from "./gen/ship/ops/v1/permissions_pb.js";
import { PipelinesService } from "./gen/ship/ops/v1/pipelines_pb.js";
import { ProjectService } from "./gen/ship/ops/v1/project_pb.js";
import { RulesService } from "./gen/ship/ops/v1/rules_pb.js";
import { RuntimeService } from "./gen/ship/ops/v1/runtime_pb.js";
import { SessionsService } from "./gen/ship/ops/v1/sessions_pb.js";
import { SettingsService, SettingScope } from "./gen/ship/ops/v1/settings_pb.js";
import type { Setting, SettingList } from "./gen/ship/ops/v1/settings_pb.js";
import { SkillsService } from "./gen/ship/ops/v1/skills_pb.js";
import { SpecsService } from "./gen/ship/ops/v1/specs_pb.js";
import { ViewService } from "./gen/ship/ops/v1/view_pb.js";
import { ViewErrorsService } from "./gen/ship/ops/v1/view_errors_pb.js";
import { WorkspacesService } from "./gen/ship/ops/v1/workspaces_pb.js";

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
  agent: Client<typeof AgentService>;
  agents: Client<typeof AgentsService>;
  analytics: Client<typeof AnalyticsService>;
  annotation: Client<typeof AnnotationService>;
  artifacts: Client<typeof ArtifactsService>;
  chat: Client<typeof ChatService>;
  collections: Client<typeof CollectionsService>;
  create: Client<typeof CreateService>;
  dashboard: Client<typeof DashboardService>;
  discovery: Client<typeof DiscoveryService>;
  events: Client<typeof EventsService>;
  feedback: Client<typeof FeedbackService>;
  git: Client<typeof GitService>;
  mcp: Client<typeof McpService>;
  permissions: Client<typeof PermissionsService>;
  pipelines: Client<typeof PipelinesService>;
  project: Client<typeof ProjectService>;
  rules: Client<typeof RulesService>;
  runtime: Client<typeof RuntimeService>;
  sessions: Client<typeof SessionsService>;
  settings: Client<typeof SettingsService>;
  skills: Client<typeof SkillsService>;
  specs: Client<typeof SpecsService>;
  view: Client<typeof ViewService>;
  viewErrors: Client<typeof ViewErrorsService>;
  workspaces: Client<typeof WorkspacesService>;
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
    agent: createClient(AgentService, transport),
    agents: createClient(AgentsService, transport),
    analytics: createClient(AnalyticsService, transport),
    annotation: createClient(AnnotationService, transport),
    artifacts: createClient(ArtifactsService, transport),
    chat: createClient(ChatService, transport),
    collections: createClient(CollectionsService, transport),
    create: createClient(CreateService, transport),
    dashboard: createClient(DashboardService, transport),
    discovery: createClient(DiscoveryService, transport),
    events: createClient(EventsService, transport),
    feedback: createClient(FeedbackService, transport),
    git: createClient(GitService, transport),
    mcp: createClient(McpService, transport),
    permissions: createClient(PermissionsService, transport),
    pipelines: createClient(PipelinesService, transport),
    project: createClient(ProjectService, transport),
    rules: createClient(RulesService, transport),
    runtime: createClient(RuntimeService, transport),
    sessions: createClient(SessionsService, transport),
    settings: createClient(SettingsService, transport),
    skills: createClient(SkillsService, transport),
    specs: createClient(SpecsService, transport),
    view: createClient(ViewService, transport),
    viewErrors: createClient(ViewErrorsService, transport),
    workspaces: createClient(WorkspacesService, transport),
  };
}

export {
  OpsService,
  StreamService,
  AccountService,
  AgentService,
  AgentsService,
  AnalyticsService,
  AnnotationService,
  ArtifactsService,
  ChatService,
  CollectionsService,
  CreateService,
  DashboardService,
  DiscoveryService,
  EventsService,
  FeedbackService,
  GitService,
  McpService,
  PermissionsService,
  PipelinesService,
  ProjectService,
  RulesService,
  RuntimeService,
  SessionsService,
  SettingsService,
  SettingScope,
  SkillsService,
  SpecsService,
  ViewService,
  ViewErrorsService,
  WorkspacesService,
};

export type { Setting, SettingList };
