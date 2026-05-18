// UI endpoints slice — daemon read surface the dashboard depends on.
//
// Targets the analytics endpoints added in d4fef06ba. Pure read
// contracts: shape-only assertions, no preconditions on prior
// dispatched sessions. Endpoints must be reachable, return 200, and
// emit the documented envelope ({ ok, data: {...} }).
//
// Why this slice exists: AnalyticsProjection populates rows for every
// dispatched harness session (via hook.* OR actor.created fallback).
// Without HTTP read endpoints, that projection is invisible to the
// dashboard. These tests gate the contract.

import { describe, test, expect } from "bun:test";

import { api } from "./_helpers";

type Envelope<T> = { ok: boolean; data: T };

describe("ui endpoints / analytics", () => {
  test("GET /api/v1/analytics/sessions returns sessions array", async () => {
    const res = await api<Envelope<{ count: number; sessions: unknown[] }>>(
      "/api/v1/analytics/sessions",
    );
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
    expect(Array.isArray(res.body.data.sessions)).toBe(true);
    expect(typeof res.body.data.count).toBe("number");
    expect(res.body.data.count).toBe(res.body.data.sessions.length);
  });

  test("GET /api/v1/analytics/sessions honors limit query param", async () => {
    const res = await api<Envelope<{ count: number; sessions: unknown[] }>>(
      "/api/v1/analytics/sessions?limit=1",
    );
    expect(res.status).toBe(200);
    expect(res.body.data.sessions.length).toBeLessThanOrEqual(1);
  });

  test("GET /api/v1/analytics/sessions/{key} 404s on unknown key", async () => {
    const res = await api(
      `/api/v1/analytics/sessions/nonexistent-${Date.now()}`,
    );
    expect(res.status).toBe(404);
  });

  test("GET /api/v1/analytics/tool-usage returns tools array", async () => {
    const res = await api<Envelope<{ count: number; tools: unknown[] }>>(
      "/api/v1/analytics/tool-usage",
    );
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
    expect(Array.isArray(res.body.data.tools)).toBe(true);
    expect(typeof res.body.data.count).toBe("number");
  });

  test("GET /api/v1/analytics/tool-usage with time window doesn't 500", async () => {
    // RFC3339 timestamps — projection's SQL filters on them as strings.
    const since = "2026-01-01T00:00:00+00:00";
    const until = "2027-01-01T00:00:00+00:00";
    const res = await api<Envelope<{ count: number; tools: unknown[] }>>(
      `/api/v1/analytics/tool-usage?since=${encodeURIComponent(
        since,
      )}&until=${encodeURIComponent(until)}`,
    );
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
  });
});

describe("ui endpoints / actors", () => {
  // Lightweight reachability check on the actor read surface the
  // dashboard pairs with analytics_sessions. Real shape coverage lives
  // in apps/shipd's own scope/integration tests; this slice just
  // confirms the URL is wired and authenticated.

  test("GET /api/v1/actors returns actors array", async () => {
    const res = await api<Envelope<{ actors: unknown[] }>>("/api/v1/actors");
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
    expect(Array.isArray(res.body.data.actors)).toBe(true);
  });

  test("GET /api/v1/mesh returns mesh roster", async () => {
    const res = await api<Envelope<{ count: number; actors: unknown[] }>>(
      "/api/v1/mesh",
    );
    expect(res.status).toBe(200);
    expect(res.body.ok).toBe(true);
    expect(typeof res.body.data.count).toBe("number");
  });
});
