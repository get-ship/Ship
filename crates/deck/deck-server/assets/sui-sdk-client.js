// packages/deck/sdk-client/src/http.ts
async function mutateJson(baseUrl, method, path, body) {
  const init = { method, credentials: "same-origin" };
  if (body !== undefined) {
    init.headers = { "content-type": "application/json" };
    init.body = JSON.stringify(body);
  }
  return throwOnError(await fetch(`${baseUrl}${path}`, init), method, path);
}
async function mutateForm(baseUrl, path, body) {
  const resp = await fetch(`${baseUrl}${path}`, {
    method: "POST",
    credentials: "same-origin",
    body
  });
  return throwOnError(resp, "POST", path);
}
async function fetchBody(baseUrl, path, as) {
  const resp = await fetch(`${baseUrl}${path}`, { credentials: "same-origin" });
  if (!resp.ok) {
    let detail = `${resp.status} ${resp.statusText}`;
    try {
      const t = await resp.text();
      if (t)
        detail = t;
    } catch {}
    throw new Error(`[ship-sdk-client] GET ${path} failed: ${detail}`);
  }
  if (as === "text")
    return await resp.text();
  if (as === "blob")
    return await resp.blob();
  return await resp.json();
}
async function throwOnError(resp, method, path) {
  if (!resp.ok) {
    let detail = `${resp.status} ${resp.statusText}`;
    try {
      const text = await resp.text();
      if (text)
        detail = text;
    } catch {}
    throw new Error(`[ship-sdk-client] ${method} ${path} failed: ${detail}`);
  }
  if (resp.status === 204)
    return;
  const ct = resp.headers.get("content-type") ?? "";
  if (ct.includes("application/json"))
    return await resp.json();
  return;
}

// packages/deck/sdk-client/src/sse.ts
import {
  applyPatch
} from "@ship/sui-runtime";
function liveExpr(expr, ESCtor = globalThis.EventSource) {
  return makeSseSource(`/_sui/data?expr=${encodeURIComponent(expr)}`, ESCtor);
}
function makeSseSource(url, ESCtor) {
  return {
    subscribe(sig) {
      if (!ESCtor) {
        sig.set({
          kind: "error",
          error: new Error("[@ship/sdk-client] no EventSource available")
        });
        return () => {};
      }
      const facade = makeFacade(sig);
      const es = new ESCtor(url);
      let firstSeen = false;
      const onPatch = (ev) => {
        let patch;
        try {
          patch = JSON.parse(ev.data);
        } catch (err) {
          sig.set({ kind: "error", error: toError(err) });
          return;
        }
        try {
          if (!firstSeen) {
            firstSeen = true;
            if (patch.op === "set" && (!patch.path || patch.path === "")) {
              sig.set({ kind: "ok", value: patch.value });
            } else {
              applyPatch(facade, patch);
            }
          } else {
            applyPatch(facade, patch);
          }
        } catch (err) {
          sig.set({ kind: "error", error: toError(err) });
        }
      };
      const onError = () => {
        if (!firstSeen) {
          sig.set({
            kind: "error",
            error: new Error("[@ship/sdk-client] SSE connection error")
          });
        }
      };
      es.addEventListener("patch", onPatch);
      es.addEventListener("error", onError);
      return () => {
        es.removeEventListener("patch", onPatch);
        es.removeEventListener("error", onError);
        es.close();
      };
    }
  };
}
function makeFacade(sig) {
  const reader = () => {
    const v = sig();
    return v.kind === "ok" ? v.value : undefined;
  };
  reader.set = (next) => {
    const cur = sig.__peek();
    const prev = cur.kind === "ok" ? cur.value : undefined;
    const v = typeof next === "function" ? next(prev) : next;
    sig.set({ kind: "ok", value: v });
  };
  reader.__subs = sig.__subs;
  reader.__peek = () => {
    const v = sig.__peek();
    return v.kind === "ok" ? v.value : undefined;
  };
  return reader;
}
var toError = (err) => err instanceof Error ? err : new Error(String(err));
// packages/deck/sdk-client/src/account.ts
function readIdentity(state) {
  if (!state)
    return { status: "logged_out", user_id: null, me: null, state_id: null };
  switch (state.state) {
    case "logged_in":
      return { status: "logged_in", user_id: state.user_id, me: state.me ?? null, state_id: null };
    case "pending":
      return { status: "pending", user_id: null, me: null, state_id: state.state_id };
    default:
      return { status: "logged_out", user_id: null, me: null, state_id: null };
  }
}
function displayName(id) {
  if (id.status !== "logged_in")
    return "";
  const name = id.me?.name?.trim();
  if (name)
    return name;
  const email = id.me?.email?.trim();
  if (email)
    return email;
  return id.user_id ?? "";
}
function initials(id) {
  if (id.status !== "logged_in")
    return "";
  const source = id.me?.name?.trim() || id.me?.email?.split("@")[0] || id.user_id || "";
  const parts = source.split(/[\s._-]+/).filter(Boolean);
  if (parts.length === 0)
    return "";
  if (parts.length === 1)
    return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[1][0]).toUpperCase();
}
function planLabel(_id) {
  return "";
}
async function signIn(redirectUri) {
  const body = JSON.stringify({
    redirect_uri: redirectUri ?? "http://127.0.0.1:0/callback"
  });
  const res = await fetch("/api/v1/account/login", {
    method: "POST",
    credentials: "same-origin",
    headers: { "Content-Type": "application/json" },
    body
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`account.login failed (${res.status}): ${text}`);
  }
  const data = await res.json();
  if (data.browser_url)
    window.open(data.browser_url, "_blank", "noopener");
  return data;
}
async function signOut() {
  const res = await fetch("/api/v1/account/logout", {
    method: "POST",
    credentials: "same-origin"
  });
  if (!res.ok && res.status !== 204) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`account.logout failed (${res.status}): ${text}`);
  }
}
function openBilling() {
  window.open("https://getship.dev/billing", "_blank", "noopener");
}

// packages/deck/sdk-client/src/index.ts
function createClient(opts) {
  const baseUrl = (opts?.baseUrl ?? "").replace(/\/$/, "");
  const ESCtor = opts?.EventSource ?? globalThis.EventSource;
  const sse = (domain, op, args) => {
    const argsLit = args === undefined ? "" : JSON.stringify(args);
    const expr = `ship.${domain}.${op}(${argsLit})`;
    return makeSseSource(`${baseUrl}/_sui/data?expr=${encodeURIComponent(expr)}`, ESCtor);
  };
  const json = (method, path, body2) => mutateJson(baseUrl, method, path, body2);
  const form = (path, body2) => mutateForm(baseUrl, path, body2);
  const body = (path, as) => fetchBody(baseUrl, path, as);
  return {
    account: {
      state: () => sse("account", "state")
    },
    agents: {
      summaries: () => sse("agents", "summaries"),
      create_ctx: () => sse("agents", "create_ctx"),
      wire: (args) => sse("agents", "wire", args),
      detail: (args) => sse("agents", "detail", args),
      settings: (args) => sse("agents", "settings", args),
      catalogs: () => sse("agents", "catalogs"),
      create: (args) => json("POST", "/api/v1/agents", {
        id: args.id,
        doc: args.doc
      }),
      update: (args) => json("PUT", `/api/v1/agents/${encodeURIComponent(args.id)}`, args.doc)
    },
    canvas: {
      doc: (args) => sse("canvas", "doc", args),
      list: () => sse("canvas", "list")
    },
    chat: {
      state: (args) => sse("chat", "state", args)
    },
    collections: {
      state: (args) => sse("collections", "state", args),
      detail: (args) => sse("collections", "detail", args),
      items: (args) => sse("collections", "items", args),
      list: () => sse("collections", "list"),
      create: (args) => json("POST", "/api/v1/collections", {
        id: args.id,
        name: args.name ?? args.id,
        fields: args.fields ?? {}
      }),
      update_schema: (args) => json("PUT", `/api/v1/collections/${encodeURIComponent(args.id)}/schema`, { id: args.id, fields: args.fields }),
      get_item: (args) => body(`/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`, "json"),
      create_item: (args) => {
        const b = { data: args.data };
        if (args.id !== undefined)
          b.id = args.id;
        return json("POST", `/api/v1/collections/${encodeURIComponent(args.collection)}/items`, b);
      },
      update_item: (args) => json("PUT", `/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`, { data: args.data }),
      delete_item: (args) => json("DELETE", `/api/v1/collections/${encodeURIComponent(args.collection)}/items/${encodeURIComponent(args.item_id)}`)
    },
    dashboard: {
      state: () => sse("dashboard", "state")
    },
    events: {
      list: (args) => sse("events", "list", args)
    },
    git: {
      overview: (args) => sse("git", "overview", args),
      diff: (args) => sse("git", "diff", args),
      annotations: (args) => sse("git", "annotations", args),
      stage: (args) => json("POST", "/api/v1/git/stage", args),
      unstage: (args) => json("POST", "/api/v1/git/unstage", args),
      commit: (args) => json("POST", "/api/v1/git/commit", args),
      push: (args) => json("POST", "/api/v1/git/push", args ?? {}),
      fetch: (args) => json("POST", "/api/v1/git/fetch", args ?? {}),
      pull: (args) => json("POST", "/api/v1/git/pull", args ?? {}),
      checkout: (args) => json("POST", "/api/v1/git/checkout", { ref: args.ref, workspace_id: args.workspace_id }),
      branch_create: (args) => json("POST", "/api/v1/git/branch", args),
      branch_delete: (args) => json("DELETE", "/api/v1/git/branch", args),
      init: (args) => json("POST", "/api/v1/git/init", args ?? {})
    },
    mesh: {
      snapshot: () => sse("mesh", "snapshot")
    },
    settings: {
      state: () => sse("settings", "state")
    },
    sessions: {
      list: () => sse("sessions", "list"),
      start: (args) => json("POST", "/api/v1/sessions/spawn", args),
      end: (args) => json("DELETE", `/api/v1/sessions/${encodeURIComponent(args.id)}`)
    },
    workspaces: {
      list: () => sse("workspaces", "list"),
      active: () => sse("workspaces", "active"),
      activate: (args) => json("POST", "/api/v1/projects/switch", { id: args.id })
    },
    artifacts: {
      list: (args) => sse("artifacts", "list", args),
      get: (args) => body(`/api/v1/artifacts/${encodeURIComponent(args.id)}`, args.as ?? "text"),
      upload: (args) => {
        const fd = new FormData;
        const name = args.filename ?? (args.file instanceof File ? args.file.name : "upload.bin");
        fd.append("file", args.file, name);
        if (args.collection_id)
          fd.append("collection_id", args.collection_id);
        if (args.item_id)
          fd.append("item_id", args.item_id);
        return form("/api/v1/artifacts", fd);
      }
    },
    skills: {
      list: () => sse("skills", "list"),
      summaries: (args) => sse("skills", "summaries", args),
      detail: (args) => sse("skills", "detail", args),
      invoke: (args) => json("POST", `/api/v1/skills/${encodeURIComponent(args.id)}/invoke`, {
        vars: args.vars,
        workspace_id: args.workspace_id,
        session_id: args.session_id
      })
    }
  };
}
export {
  readIdentity as readAccountIdentity,
  openBilling as openAccountBilling,
  makeSseSource,
  liveExpr,
  createClient,
  signOut as accountSignOut,
  signIn as accountSignIn,
  planLabel as accountPlanLabel,
  initials as accountInitials,
  displayName as accountDisplayName
};
