// packages/ship-sdk-client/src/index.ts
import { applyPatch } from "@ship/sui-runtime/patch";
function createClient(opts) {
  const baseUrl = (opts?.baseUrl ?? "").replace(/\/$/, "");
  const ESCtor = opts?.EventSource ?? globalThis.EventSource;
  return {
    agents: {
      summaries: () => makeSseSource(`${baseUrl}/_sui/data?expr=${encodeURIComponent("ship.agents.summaries()")}`, ESCtor)
    },
    git: {
      overview: () => makeSseSource(`${baseUrl}/_sui/data?expr=${encodeURIComponent("ship.git.overview()")}`, ESCtor),
      diff: (args) => makeSseSource(`${baseUrl}/_sui/data?expr=${encodeURIComponent(`ship.git.diff(${JSON.stringify(args)})`)}`, ESCtor)
    }
  };
}
function makeSseSource(url, ESCtor) {
  return {
    subscribe(sig) {
      if (!ESCtor) {
        sig.set({
          kind: "error",
          error: new Error("[ship-set/sdk-client] no EventSource available")
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
            error: new Error("[ship-set/sdk-client] SSE connection error")
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
export {
  makeSseSource,
  createClient
};
