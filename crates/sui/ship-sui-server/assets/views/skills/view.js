// views/skills/page.sui
import { bindAttr, bindClass, bindFor, bindIf, bindProp, bindText, bindValue, bindWhileError, bindWhileLoading, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateBindValue, hydrateOn, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/skills/controller.ts
import { controller, signal, derived, live, effect } from "@ship/sui-runtime";
import { liveExpr } from "@ship/sdk-client";
var SECTION_IDS = ["identity", "description", "tags", "triggers", "variables", "references", "evals"];
var DEFAULT_OPEN = {
  identity: true,
  description: true,
  tags: false,
  triggers: false,
  variables: false,
  references: false,
  evals: false
};
var STORAGE_PREFIX = "ship.skills.metadata-open:";
function loadOpenState(id) {
  if (!id || typeof localStorage === "undefined")
    return { ...DEFAULT_OPEN };
  try {
    const raw = localStorage.getItem(STORAGE_PREFIX + id);
    if (!raw)
      return { ...DEFAULT_OPEN };
    const parsed = JSON.parse(raw);
    const out = { ...DEFAULT_OPEN };
    for (const k of SECTION_IDS)
      if (typeof parsed[k] === "boolean")
        out[k] = parsed[k];
    return out;
  } catch {
    return { ...DEFAULT_OPEN };
  }
}
function saveOpenState(id, state) {
  if (!id || typeof localStorage === "undefined")
    return;
  try {
    localStorage.setItem(STORAGE_PREFIX + id, JSON.stringify(state));
  } catch {}
}
function fileKind(name) {
  const lower = name.toLowerCase();
  if (lower.endsWith(".md"))
    return "md";
  if (lower.endsWith(".json") || lower.endsWith(".jsonc"))
    return "json";
  if (lower.endsWith(".yaml") || lower.endsWith(".yml"))
    return "yaml";
  if (lower.endsWith(".sh"))
    return "sh";
  if (lower.endsWith(".ts") || lower.endsWith(".tsx") || lower.endsWith(".js"))
    return "ts";
  if (lower.endsWith(".txt"))
    return "txt";
  return "unknown";
}
function fileMeta(name) {
  const kind = fileKind(name);
  switch (kind) {
    case "md":
      return { icon: "file-text", tone: name === "SKILL.md" ? "primary" : "success", kind };
    case "json":
      return { icon: "file-code", tone: "warning", kind };
    case "yaml":
    case "sh":
    case "ts":
      return { icon: "file-code", tone: "info", kind };
    case "txt":
      return { icon: "file-text", tone: "muted", kind };
    default:
      return { icon: "file", tone: "muted", kind };
  }
}
function decodeEntities(s) {
  return s.replace(/&#x27;/g, "'").replace(/&quot;/g, '"').replace(/&gt;/g, ">").replace(/&lt;/g, "<").replace(/&amp;/g, "&");
}
function isEditable(path, ext) {
  if (!path)
    return true;
  return ext === "md";
}
function readSkillIdFromPath(path) {
  const m = path.match(/(?:^|\/)skills\/([^/?#]+)/);
  return m ? decodeURIComponent(m[1]) : "";
}
function readFileFromSearch(search) {
  if (!search)
    return "";
  const params = new URLSearchParams(search.startsWith("?") ? search.slice(1) : search);
  return params.get("file") ?? "";
}
function sourceLabel(src) {
  if (src === "PROJECT SKILLS")
    return "Project · .ship/";
  if (src === "USER SKILLS")
    return "User · ~/.ship/";
  return src;
}
function extractTriggerHints(body) {
  if (!body)
    return [];
  const lines = body.split(/\r?\n/);
  const out = [];
  for (const raw of lines) {
    const line = raw.trim();
    if (!line)
      continue;
    if (/^TRIGGER\b/i.test(line) || /^SKIP\b/i.test(line) || /^WHEN\b/i.test(line)) {
      out.push(line);
    }
  }
  return out.slice(0, 6);
}
function formatRelative(ts) {
  const secs = Math.max(1, Math.round((Date.now() - ts) / 1000));
  if (secs < 60)
    return `${secs}s ago`;
  const mins = Math.round(secs / 60);
  if (mins < 60)
    return `${mins}m ago`;
  const hrs = Math.round(mins / 60);
  return `${hrs}h ago`;
}
var controller_default = controller({
  scope: () => {
    const pathSig = signal(typeof location !== "undefined" ? location.pathname : "");
    const searchSig = signal(typeof location !== "undefined" ? location.search : "");
    effect(() => {
      if (typeof window === "undefined")
        return;
      const onPop = () => {
        pathSig.set(location.pathname);
        searchSig.set(location.search);
      };
      window.addEventListener("popstate", onPop);
      window.addEventListener("sui:navigate", onPop);
      return () => {
        window.removeEventListener("popstate", onPop);
        window.removeEventListener("sui:navigate", onPop);
      };
    });
    const skillId = derived(() => readSkillIdFromPath(pathSig()));
    const selectedFile = derived(() => readFileFromSearch(searchSig()));
    const q = signal("");
    const kind = signal("all");
    const tag = signal("");
    const draft = signal("");
    const baseline = signal("");
    const saving = signal(false);
    const saveBanner = signal(null);
    const descDraft = signal("");
    const descBaseline = signal("");
    const descSaving = signal(false);
    const descBanner = signal(null);
    const descSavedAt = signal(null);
    const nowTick = signal(Date.now());
    effect(() => {
      if (typeof window === "undefined")
        return;
      const t = setInterval(() => nowTick.set(Date.now()), 1000);
      return () => clearInterval(t);
    });
    const tagsList = signal([]);
    const tagsBaseline = signal([]);
    const tagsInput = signal("");
    const tagsSaving = signal(false);
    const tagsBanner = signal(null);
    const openMap = signal({ ...DEFAULT_OPEN });
    effect(() => {
      const id = skillId();
      openMap.set(loadOpenState(id));
    });
    function toggleSection(sec, next) {
      const cur = openMap();
      if (cur[sec] === next)
        return;
      const updated = { ...cur, [sec]: next };
      openMap.set(updated);
      saveOpenState(skillId(), updated);
    }
    const isOpen = (sec) => derived(() => !!openMap()[sec]);
    const rail = live(() => liveExpr(`ship.skills.summaries(${JSON.stringify({ q: q(), kind: kind(), tag: tag() })})`));
    const skill = live(() => liveExpr(`ship.skills.detail(${JSON.stringify({ id: skillId() || "__none__" })})`));
    const fileContent = live(() => liveExpr(`ship.skills.file(${JSON.stringify({
      id: skillId() || "__none__",
      path: selectedFile() || "__none__"
    })})`));
    effect(() => {
      const path = selectedFile();
      if (path) {
        const fc = fileContent();
        if (fc.kind === "ok" && fc.value && fc.value.path === path) {
          const decoded = decodeEntities(fc.value.content);
          draft.set(decoded);
          baseline.set(decoded);
          saveBanner.set(null);
        }
      } else {
        const sk2 = skill();
        if (sk2.kind === "ok" && sk2.value) {
          const body = sk2.value.body ?? "";
          draft.set(body);
          baseline.set(body);
          saveBanner.set(null);
        }
      }
    });
    effect(() => {
      const sk2 = skill();
      if (sk2.kind !== "ok" || !sk2.value)
        return;
      const desc = sk2.value.description ?? "";
      descDraft.set(desc);
      descBaseline.set(desc);
      descBanner.set(null);
      descSavedAt.set(null);
      const tags = [...sk2.value.tags ?? []];
      tagsList.set(tags);
      tagsBaseline.set(tags);
      tagsInput.set("");
      tagsBanner.set(null);
    });
    const fileMetas = derived(() => {
      const sId = skillId();
      const r = rail();
      const map = {};
      if (r.kind !== "ok" || !r.value)
        return map;
      for (const g of r.value.groups || []) {
        for (const sk2 of g.skills) {
          if (sk2.id === sId) {
            for (const f of sk2.files || [])
              map[f] = fileMeta(f);
          }
        }
      }
      return map;
    });
    const dirty = derived(() => draft() !== baseline());
    const selectedFileName = derived(() => {
      const path = selectedFile();
      if (!path)
        return "";
      const idx = path.lastIndexOf("/");
      return idx >= 0 ? path.slice(idx + 1) : path;
    });
    const activeIsEditable = derived(() => {
      const path = selectedFile();
      if (!path)
        return true;
      const fc = fileContent();
      if (fc.kind === "ok" && fc.value)
        return isEditable(path, fc.value.ext);
      return isEditable(path, fileKind(path));
    });
    const sk = derived(() => {
      const s = skill();
      return s.kind === "ok" && s.value ? s.value : null;
    });
    const skName = derived(() => sk()?.name ?? "");
    const skId = derived(() => sk()?.id ?? "");
    const skKind = derived(() => sk()?.kind ?? "");
    const skSource = derived(() => sk()?.source ?? "");
    const skSourceLabel = derived(() => sourceLabel(sk()?.source ?? ""));
    const skBody = derived(() => sk()?.body ?? "");
    const skVariables = derived(() => sk()?.variables ?? []);
    const skFiles = derived(() => sk()?.files ?? []);
    const skAuthors = derived(() => sk()?.authors ?? []);
    const skAuthorsText = derived(() => skAuthors().join(", "));
    const skTriggerHints = derived(() => extractTriggerHints(sk()?.body ?? ""));
    const skReferences = derived(() => (sk()?.files ?? []).filter((f) => f.startsWith("references/") || f.startsWith("reference/")));
    const descDirty = derived(() => descDraft() !== descBaseline());
    const tagsDirty = derived(() => {
      const a = tagsList(), b = tagsBaseline();
      if (a.length !== b.length)
        return true;
      for (let i = 0;i < a.length; i++)
        if (a[i] !== b[i])
          return true;
      return false;
    });
    const descSavedAgo = derived(() => {
      const at = descSavedAt();
      if (!at)
        return "";
      nowTick();
      return formatRelative(at);
    });
    function onEditorInput(evt) {
      const detail = evt.detail;
      if (detail && typeof detail.value === "string") {
        draft.set(detail.value);
      } else {
        const el = evt.target;
        if (typeof el.value === "string")
          draft.set(el.value);
      }
    }
    function onDescInput(evt) {
      const el = evt.target;
      if (typeof el.value === "string")
        descDraft.set(el.value);
    }
    function onTagsInput(evt) {
      const el = evt.target;
      tagsInput.set(el.value);
    }
    function addTag() {
      const raw = tagsInput().trim().replace(/,+$/, "").trim();
      if (!raw)
        return;
      const parts = raw.split(",").map((t) => t.trim()).filter(Boolean);
      const current = tagsList();
      const next = [...current];
      for (const p of parts)
        if (!next.includes(p))
          next.push(p);
      tagsList.set(next);
      tagsInput.set("");
    }
    function onTagsKeydown(evt) {
      if (evt.key === "Enter" || evt.key === ",") {
        evt.preventDefault();
        addTag();
      } else if (evt.key === "Backspace" && tagsInput() === "" && tagsList().length > 0) {
        const cur = tagsList();
        const last = cur[cur.length - 1];
        tagsList.set(cur.slice(0, -1));
        tagsInput.set(last);
      }
    }
    function removeTag(t) {
      tagsList.set(tagsList().filter((x) => x !== t));
    }
    function resetDesc() {
      descDraft.set(descBaseline());
      descBanner.set(null);
    }
    function resetTags() {
      tagsList.set([...tagsBaseline()]);
      tagsInput.set("");
      tagsBanner.set(null);
    }
    async function save() {
      const id = skillId();
      if (!id || saving())
        return;
      const path = selectedFile();
      const body = draft();
      saving.set(true);
      saveBanner.set(null);
      const url = path ? `/api/v1/skills/${encodeURIComponent(id)}/files/${path.split("/").map(encodeURIComponent).join("/")}` : `/api/v1/skills/${encodeURIComponent(id)}/body`;
      const payload = path ? { content: body } : { body };
      try {
        const resp = await fetch(url, {
          method: "PUT",
          credentials: "same-origin",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload)
        });
        const text = await resp.text().catch(() => "");
        if (!resp.ok) {
          saveBanner.set({ ok: false, msg: text || `Save failed (${resp.status})` });
          return;
        }
        baseline.set(body);
        saveBanner.set({ ok: true, msg: "Saved." });
      } catch (e) {
        saveBanner.set({ ok: false, msg: e.message ?? "Save failed." });
      } finally {
        saving.set(false);
      }
    }
    async function putFrontmatter(partial) {
      const id = skillId();
      const sk2 = skill();
      if (!id || sk2.kind !== "ok" || !sk2.value)
        return { ok: false, msg: "Skill not loaded." };
      const fm = {
        name: sk2.value.name,
        description: partial.description ?? sk2.value.description ?? ""
      };
      const tags = partial.tags ?? sk2.value.tags ?? [];
      if (tags.length > 0)
        fm.tags = tags;
      if ((sk2.value.authors ?? []).length > 0)
        fm.authors = sk2.value.authors;
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/frontmatter`, {
          method: "PUT",
          credentials: "same-origin",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(fm)
        });
        const text = await resp.text().catch(() => "");
        if (!resp.ok)
          return { ok: false, msg: text || `Save failed (${resp.status})` };
        return { ok: true };
      } catch (e) {
        return { ok: false, msg: e.message ?? "Save failed." };
      }
    }
    async function saveDesc() {
      if (descSaving() || !descDirty())
        return;
      descSaving.set(true);
      descBanner.set(null);
      const draftVal = descDraft();
      const r = await putFrontmatter({ description: draftVal, tags: tagsBaseline() });
      if (r.ok) {
        descBaseline.set(draftVal);
        descSavedAt.set(Date.now());
        descBanner.set({ ok: true, msg: "Saved." });
      } else {
        descBanner.set({ ok: false, msg: r.msg });
      }
      descSaving.set(false);
    }
    async function saveTags() {
      if (tagsSaving() || !tagsDirty())
        return;
      if (tagsInput().trim())
        addTag();
      tagsSaving.set(true);
      tagsBanner.set(null);
      const next = tagsList();
      const r = await putFrontmatter({ description: descBaseline(), tags: next });
      if (r.ok) {
        tagsBaseline.set([...next]);
        tagsBanner.set({ ok: true, msg: "Saved." });
      } else {
        tagsBanner.set({ ok: false, msg: r.msg });
      }
      tagsSaving.set(false);
    }
    function copyId() {
      const id = skId();
      if (!id || typeof navigator === "undefined" || !navigator.clipboard)
        return;
      navigator.clipboard.writeText(id).catch(() => {});
    }
    return {
      skillId,
      q,
      kind,
      tag,
      selectedFile,
      rail,
      skill,
      fileContent,
      fileMetas,
      draft,
      dirty,
      saving,
      saveBanner,
      activeIsEditable,
      selectedFileName,
      sk,
      skName,
      skId,
      skKind,
      skSource,
      skSourceLabel,
      skBody,
      skVariables,
      skFiles,
      skAuthors,
      skAuthorsText,
      skTriggerHints,
      skReferences,
      descDraft,
      descDirty,
      descSaving,
      descBanner,
      descSavedAgo,
      tagsList,
      tagsInput,
      tagsDirty,
      tagsSaving,
      tagsBanner,
      openIdentity: isOpen("identity"),
      openDescription: isOpen("description"),
      openTags: isOpen("tags"),
      openTriggers: isOpen("triggers"),
      openVariables: isOpen("variables"),
      openReferences: isOpen("references"),
      openEvals: isOpen("evals"),
      openIdentityAttr: derived(() => openMap().identity ? true : null),
      openDescriptionAttr: derived(() => openMap().description ? true : null),
      openTagsAttr: derived(() => openMap().tags ? true : null),
      openTriggersAttr: derived(() => openMap().triggers ? true : null),
      openVariablesAttr: derived(() => openMap().variables ? true : null),
      openReferencesAttr: derived(() => openMap().references ? true : null),
      openEvalsAttr: derived(() => openMap().evals ? true : null),
      toggleSection,
      onEditorInput,
      save,
      onDescInput,
      resetDesc,
      saveDesc,
      onTagsInput,
      onTagsKeydown,
      addTag,
      removeTag,
      resetTags,
      saveTags,
      copyId
    };
  }
});

// views/skills/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var ISLANDS = ["sui-markdown-editor"];
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b5 = (anchor) => bindFor({
    anchor,
    items: () => $scope.rail().value.kind_counts || [],
    key: (kc, __i) => {
      const __child = Object.create($scope);
      __child.kc = kc;
      __child.__i = __i;
      return (($scope2) => kc.id)(__child);
    },
    render: (kc, __i) => {
      const __childScope14 = Object.create($scope);
      __childScope14.kc = kc;
      __childScope14.__i = __i;
      return (($scope2) => {
        const __frag15 = document.createDocumentFragment();
        const __c16 = [];
        const __e17 = document.createElement("sui-segmented-option");
        {
          const __av18 = $scope2.kc.id;
          if (__av18 == null || __av18 === false || __av18 === "")
            __e17.removeAttribute("value");
          else
            __e17.setAttribute("value", __av18 === true ? "" : String(__av18));
        }
        const __t19 = document.createTextNode("");
        __e17.appendChild(__t19);
        __t19.textContent = $scope2.kc.label;
        const __e20 = document.createElement("span");
        __e20.setAttribute("class", "se-pill-count");
        const __t21 = document.createTextNode("");
        __e20.appendChild(__t21);
        __t21.textContent = $scope2.kc.count;
        __e17.appendChild(__e20);
        __frag15.appendChild(__e17);
        return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
          for (const c of __c16)
            try {
              c();
            } catch (e) {}
        }, update: (__new22, __newI23) => {
          const __old = $scope2.kc;
          $scope2.kc = __new22;
          $scope2.__i = __newI23;
          if (__old.id !== $scope2.kc.id) {
            {
              const __av18 = $scope2.kc.id;
              if (__av18 == null || __av18 === false || __av18 === "")
                __e17.removeAttribute("value");
              else
                __e17.setAttribute("value", __av18 === true ? "" : String(__av18));
            }
          }
          if (__old.label !== $scope2.kc.label) {
            __t19.textContent = $scope2.kc.label;
          }
          if (__old.count !== $scope2.kc.count) {
            __t21.textContent = $scope2.kc.count;
          }
        } };
      })(__childScope14);
    }
  });
  const __wireBoundary_b2 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok" && $scope.rail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag10 = document.createDocumentFragment();
        const __c11 = [];
        const __e12 = document.createElement("sui-segmented");
        __c11.push(bindAttr(__e12, "value", () => $scope.kind()));
        __e12.setAttribute("size", "sm");
        __e12.setAttribute("aria-label", "Filter by kind");
        __c11.push(on(__e12, "sui-change", function() {
          const __h = (e) => $scope.kind.set(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __forAnchor13 = document.createComment("for");
        __e12.appendChild(__forAnchor13);
        __c11.push(__wireBoundary_b5(__forAnchor13));
        __frag10.appendChild(__e12);
        return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
          for (const c of __c11)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b10 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3, 4, 5],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope29 = Object.create($scope);
      __childScope29.n = n;
      __childScope29.__i = __i;
      return (($scope2) => {
        const __frag30 = document.createDocumentFragment();
        const __c31 = [];
        const __e32 = document.createElement("sui-cluster");
        __e32.setAttribute("align", "center");
        const __e33 = document.createElement("sui-skeleton");
        __e33.setAttribute("shape", "line");
        __e33.setAttribute("size", "md");
        __e32.appendChild(__e33);
        __frag30.appendChild(__e32);
        return { nodes: Array.from(__frag30.childNodes), cleanup: () => {
          for (const c of __c31)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope29);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag25 = document.createDocumentFragment();
    const __c26 = [];
    const __e27 = document.createElement("div");
    __e27.setAttribute("class", "se-skel");
    __e27.setAttribute("role", "status");
    __e27.setAttribute("aria-live", "polite");
    __e27.setAttribute("aria-busy", "true");
    __e27.setAttribute("aria-label", "Loading skills");
    const __forAnchor28 = document.createComment("for");
    __e27.appendChild(__forAnchor28);
    __c26.push(__wireBoundary_b10(__forAnchor28));
    __frag25.appendChild(__e27);
    return { nodes: Array.from(__frag25.childNodes), cleanup: () => {
      for (const c of __c26)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope35 = Object.create($scope);
    __childScope35.err = err;
    return (($scope2) => {
      const __frag36 = document.createDocumentFragment();
      const __c37 = [];
      const __e38 = document.createElement("sui-error-state");
      __e38.setAttribute("variant", "inline");
      __e38.setAttribute("size", "sm");
      __e38.setAttribute("title", "Couldn't load skills");
      __e38.setAttribute("role", "alert");
      __e38.setAttribute("aria-live", "assertive");
      const __t39 = document.createTextNode("");
      __e38.appendChild(__t39);
      __c37.push(bindText(__t39, () => $scope2.err.message));
      __frag36.appendChild(__e38);
      return { nodes: Array.from(__frag36.childNodes), cleanup: () => {
        for (const c of __c37)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope35);
  };
  const __wireBoundary_b17 = (anchor) => bindFor({
    anchor,
    items: () => $scope.rail().value.groups || [],
    key: (g, __i) => {
      const __child = Object.create($scope);
      __child.g = g;
      __child.__i = __i;
      return (($scope2) => g.source)(__child);
    },
    render: (g, __i) => {
      const __childScope55 = Object.create($scope);
      __childScope55.g = g;
      __childScope55.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b20 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.g.skills,
          key: (s, __i2) => {
            const __child = Object.create($scope2);
            __child.s = s;
            __child.__i = __i2;
            return (($scope3) => s.id)(__child);
          },
          render: (s, __i2) => {
            const __childScope68 = Object.create($scope2);
            __childScope68.s = s;
            __childScope68.__i = __i2;
            return (($scope3) => {
              __childScope68.__v_id = signal2(0);
              const __wireBoundary_b25 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.id === $scope3.skillId())
                    return 0;
                  return 1;
                },
                branches: [
                  () => {
                    const __frag77 = document.createDocumentFragment();
                    const __c78 = [];
                    const __e79 = document.createElement("sui-icon");
                    __e79.setAttribute("name", "chevron-down");
                    __e79.setAttribute("size", "xs");
                    __frag77.appendChild(__e79);
                    return { nodes: Array.from(__frag77.childNodes), cleanup: () => {
                      for (const c of __c78)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  },
                  () => {
                    const __frag80 = document.createDocumentFragment();
                    const __c81 = [];
                    const __e82 = document.createElement("sui-icon");
                    __e82.setAttribute("name", "chevron-right");
                    __e82.setAttribute("size", "xs");
                    __frag80.appendChild(__e82);
                    return { nodes: Array.from(__frag80.childNodes), cleanup: () => {
                      for (const c of __c81)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b27 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_vars)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag88 = document.createDocumentFragment();
                    const __c89 = [];
                    const __e90 = document.createElement("span");
                    __e90.setAttribute("class", "se-dot vars");
                    __e90.setAttribute("title", "Variables");
                    __frag88.appendChild(__e90);
                    return { nodes: Array.from(__frag88.childNodes), cleanup: () => {
                      for (const c of __c89)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b28 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_refs)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag92 = document.createDocumentFragment();
                    const __c93 = [];
                    const __e94 = document.createElement("span");
                    __e94.setAttribute("class", "se-dot refs");
                    __e94.setAttribute("title", "Reference docs");
                    __frag92.appendChild(__e94);
                    return { nodes: Array.from(__frag92.childNodes), cleanup: () => {
                      for (const c of __c93)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b29 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_evals)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag96 = document.createDocumentFragment();
                    const __c97 = [];
                    const __e98 = document.createElement("span");
                    __e98.setAttribute("class", "se-dot evals");
                    __e98.setAttribute("title", "Evals");
                    __frag96.appendChild(__e98);
                    return { nodes: Array.from(__frag96.childNodes), cleanup: () => {
                      for (const c of __c97)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b31 = (anchor3) => bindFor({
                anchor: anchor3,
                items: () => $scope3.s.files,
                key: (f, __i3) => {
                  const __child = Object.create($scope3);
                  __child.f = f;
                  __child.__i = __i3;
                  return (($scope4) => f)(__child);
                },
                render: (f, __i3) => {
                  const __childScope103 = Object.create($scope3);
                  __childScope103.f = f;
                  __childScope103.__i = __i3;
                  return (($scope4) => {
                    __childScope103.__v = signal2(0);
                    const __frag104 = document.createDocumentFragment();
                    const __c105 = [];
                    const __e106 = document.createElement("a");
                    __e106.setAttribute("class", "se-file");
                    __c105.push(bindClass(__e106, "is-active", () => (void $scope4.__v(), $scope4.selectedFile() === $scope4.f)));
                    __e106.setAttribute("role", "treeitem");
                    __c105.push(bindAttr(__e106, "aria-current", () => (void $scope4.__v(), $scope4.selectedFile() === $scope4.f ? "true" : null)));
                    {
                      const __av107 = "/skills/" + $scope4.s.id + "?file=" + $scope4.f;
                      if (__av107 == null || __av107 === false || __av107 === "")
                        __e106.removeAttribute("href");
                      else
                        __e106.setAttribute("href", __av107 === true ? "" : String(__av107));
                    }
                    {
                      const __av108 = $scope4.f;
                      if (__av108 == null || __av108 === false || __av108 === "")
                        __e106.removeAttribute("title");
                      else
                        __e106.setAttribute("title", __av108 === true ? "" : String(__av108));
                    }
                    const __e109 = document.createElement("sui-icon");
                    __e109.setAttribute("size", "xs");
                    __c105.push(bindAttr(__e109, "name", () => (void $scope4.__v(), $scope4.fileMetas()[$scope4.f] ? $scope4.fileMetas()[$scope4.f].icon : "file")));
                    __c105.push(bindAttr(__e109, "tone", () => (void $scope4.__v(), $scope4.fileMetas()[$scope4.f] ? $scope4.fileMetas()[$scope4.f].tone : "muted")));
                    __e106.appendChild(__e109);
                    const __e110 = document.createElement("span");
                    __e110.setAttribute("class", "se-fname");
                    const __t111 = document.createTextNode("");
                    __e110.appendChild(__t111);
                    __t111.textContent = $scope4.f;
                    __e106.appendChild(__e110);
                    const __e112 = document.createElement("span");
                    __e112.setAttribute("class", "se-fkind");
                    __e112.setAttribute("aria-hidden", "true");
                    const __t113 = document.createTextNode("");
                    __e112.appendChild(__t113);
                    __c105.push(bindText(__t113, () => (void $scope4.__v(), $scope4.fileMetas()[$scope4.f] ? $scope4.fileMetas()[$scope4.f].kind : "")));
                    __e106.appendChild(__e112);
                    __frag104.appendChild(__e106);
                    return { nodes: Array.from(__frag104.childNodes), cleanup: () => {
                      for (const c of __c105)
                        try {
                          c();
                        } catch (e) {}
                    }, update: (__new114, __newI115) => {
                      $scope4.f = __new114;
                      $scope4.__i = __newI115;
                      {
                        const __av107 = "/skills/" + $scope4.s.id + "?file=" + $scope4.f;
                        if (__av107 == null || __av107 === false || __av107 === "")
                          __e106.removeAttribute("href");
                        else
                          __e106.setAttribute("href", __av107 === true ? "" : String(__av107));
                      }
                      {
                        const __av108 = $scope4.f;
                        if (__av108 == null || __av108 === false || __av108 === "")
                          __e106.removeAttribute("title");
                        else
                          __e106.setAttribute("title", __av108 === true ? "" : String(__av108));
                      }
                      __t111.textContent = $scope4.f;
                      $scope4.__v.set((c) => c + 1);
                    } };
                  })(__childScope103);
                }
              });
              const __wireBoundary_b30 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.id === $scope3.skillId() && $scope3.s.files && $scope3.s.files.length > 0)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag100 = document.createDocumentFragment();
                    const __c101 = [];
                    const __forAnchor102 = document.createComment("for");
                    __frag100.appendChild(__forAnchor102);
                    __c101.push(__wireBoundary_b31(__forAnchor102));
                    return { nodes: Array.from(__frag100.childNodes), cleanup: () => {
                      for (const c of __c101)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag69 = document.createDocumentFragment();
              const __c70 = [];
              const __e71 = document.createElement("div");
              __e71.setAttribute("class", "se-skill-row");
              __e71.setAttribute("role", "treeitem");
              __c70.push(bindAttr(__e71, "aria-selected", () => (void $scope3.__v_id(), $scope3.s.id === $scope3.skillId() ? "true" : "false")));
              const __e72 = document.createElement("a");
              __e72.setAttribute("class", "se-skill");
              __c70.push(bindClass(__e72, "is-active", () => (void $scope3.__v_id(), $scope3.s.id === $scope3.skillId())));
              {
                const __av73 = "/skills/" + $scope3.s.id;
                if (__av73 == null || __av73 === false || __av73 === "")
                  __e72.removeAttribute("href");
                else
                  __e72.setAttribute("href", __av73 === true ? "" : String(__av73));
              }
              {
                const __av74 = $scope3.s.description;
                if (__av74 == null || __av74 === false || __av74 === "")
                  __e72.removeAttribute("title");
                else
                  __e72.setAttribute("title", __av74 === true ? "" : String(__av74));
              }
              const __e75 = document.createElement("span");
              __e75.setAttribute("class", "se-chevron");
              __e75.setAttribute("aria-hidden", "true");
              const __ifAnchor76 = document.createComment("if");
              __e75.appendChild(__ifAnchor76);
              __c70.push(__wireBoundary_b25(__ifAnchor76));
              __e72.appendChild(__e75);
              const __e83 = document.createElement("sui-icon");
              __e83.setAttribute("name", "zap");
              __e83.setAttribute("size", "xs");
              __e83.setAttribute("tone", "primary");
              __e72.appendChild(__e83);
              const __e84 = document.createElement("span");
              __e84.setAttribute("class", "se-name");
              const __t85 = document.createTextNode("");
              __e84.appendChild(__t85);
              __t85.textContent = $scope3.s.name;
              __e72.appendChild(__e84);
              const __e86 = document.createElement("span");
              __e86.setAttribute("class", "se-dots");
              const __ifAnchor87 = document.createComment("if");
              __e86.appendChild(__ifAnchor87);
              __c70.push(__wireBoundary_b27(__ifAnchor87));
              const __ifAnchor91 = document.createComment("if");
              __e86.appendChild(__ifAnchor91);
              __c70.push(__wireBoundary_b28(__ifAnchor91));
              const __ifAnchor95 = document.createComment("if");
              __e86.appendChild(__ifAnchor95);
              __c70.push(__wireBoundary_b29(__ifAnchor95));
              __e72.appendChild(__e86);
              __e71.appendChild(__e72);
              const __ifAnchor99 = document.createComment("if");
              __e71.appendChild(__ifAnchor99);
              __c70.push(__wireBoundary_b30(__ifAnchor99));
              __frag69.appendChild(__e71);
              return { nodes: Array.from(__frag69.childNodes), cleanup: () => {
                for (const c of __c70)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new116, __newI117) => {
                const __old = $scope3.s;
                $scope3.s = __new116;
                $scope3.__i = __newI117;
                if (__old.id !== $scope3.s.id) {
                  {
                    const __av73 = "/skills/" + $scope3.s.id;
                    if (__av73 == null || __av73 === false || __av73 === "")
                      __e72.removeAttribute("href");
                    else
                      __e72.setAttribute("href", __av73 === true ? "" : String(__av73));
                  }
                }
                if (__old.description !== $scope3.s.description) {
                  {
                    const __av74 = $scope3.s.description;
                    if (__av74 == null || __av74 === false || __av74 === "")
                      __e72.removeAttribute("title");
                    else
                      __e72.setAttribute("title", __av74 === true ? "" : String(__av74));
                  }
                }
                if (__old.name !== $scope3.s.name) {
                  __t85.textContent = $scope3.s.name;
                }
                if (__old.id !== $scope3.s.id) {
                  $scope3.__v_id.set((c) => c + 1);
                }
              } };
            })(__childScope68);
          }
        });
        const __frag56 = document.createDocumentFragment();
        const __c57 = [];
        const __e58 = document.createElement("div");
        __e58.setAttribute("class", "se-section");
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "se-section-head");
        const __e60 = document.createElement("span");
        __e60.setAttribute("class", "se-section-label");
        const __e61 = document.createElement("sui-icon");
        __e61.setAttribute("name", "folder");
        __e61.setAttribute("size", "xs");
        __e61.setAttribute("tone", "muted");
        __e60.appendChild(__e61);
        const __t62 = document.createTextNode("");
        __e60.appendChild(__t62);
        __t62.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
        const __e63 = document.createElement("span");
        __e63.setAttribute("class", "count");
        const __t64 = document.createTextNode("(");
        __e63.appendChild(__t64);
        const __t65 = document.createTextNode("");
        __e63.appendChild(__t65);
        __t65.textContent = $scope2.g.skills.length;
        const __t66 = document.createTextNode(")");
        __e63.appendChild(__t66);
        __e60.appendChild(__e63);
        __e59.appendChild(__e60);
        __e58.appendChild(__e59);
        const __forAnchor67 = document.createComment("for");
        __e58.appendChild(__forAnchor67);
        __c57.push(__wireBoundary_b20(__forAnchor67));
        __frag56.appendChild(__e58);
        return { nodes: Array.from(__frag56.childNodes), cleanup: () => {
          for (const c of __c57)
            try {
              c();
            } catch (e) {}
        }, update: (__new118, __newI119) => {
          const __old = $scope2.g;
          $scope2.g = __new118;
          $scope2.__i = __newI119;
          if (__old.source !== $scope2.g.source) {
            __t62.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
          }
          if (__old.skills !== $scope2.g.skills) {
            __t65.textContent = $scope2.g.skills.length;
          }
        } };
      })(__childScope55);
    }
  });
  const __wireBoundary_b15 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (($scope.rail().value.groups || []).length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag48 = document.createDocumentFragment();
        const __c49 = [];
        const __e50 = document.createElement("div");
        __e50.setAttribute("class", "se-empty");
        const __t51 = document.createTextNode("");
        __e50.appendChild(__t51);
        __c49.push(bindText(__t51, () => $scope.q() ? 'No skills match "' + $scope.q() + '".' : "No skills discovered."));
        __frag48.appendChild(__e50);
        return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
          for (const c of __c49)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag52 = document.createDocumentFragment();
        const __c53 = [];
        const __forAnchor54 = document.createComment("for");
        __frag52.appendChild(__forAnchor54);
        __c53.push(__wireBoundary_b17(__forAnchor54));
        return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
          for (const c of __c53)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b14 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok" && $scope.rail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag45 = document.createDocumentFragment();
        const __c46 = [];
        const __ifAnchor47 = document.createComment("if");
        __frag45.appendChild(__ifAnchor47);
        __c46.push(__wireBoundary_b15(__ifAnchor47));
        return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
          for (const c of __c46)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b13 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag42 = document.createDocumentFragment();
        const __c43 = [];
        const __ifAnchor44 = document.createComment("if");
        __frag42.appendChild(__ifAnchor44);
        __c43.push(__wireBoundary_b14(__ifAnchor44));
        return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
          for (const c of __c43)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b43 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selectedFile())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag157 = document.createDocumentFragment();
        const __c158 = [];
        const __e159 = document.createElement("span");
        __e159.setAttribute("class", "sep");
        const __t160 = document.createTextNode("/");
        __e159.appendChild(__t160);
        __frag157.appendChild(__e159);
        const __e161 = document.createElement("span");
        __e161.setAttribute("class", "active");
        const __t162 = document.createTextNode("");
        __e161.appendChild(__t162);
        __c158.push(bindText(__t162, () => $scope.selectedFileName()));
        __frag157.appendChild(__e161);
        return { nodes: Array.from(__frag157.childNodes), cleanup: () => {
          for (const c of __c158)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b45 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.activeIsEditable())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag165 = document.createDocumentFragment();
        const __c166 = [];
        const __e167 = document.createElement("button");
        __e167.setAttribute("class", "sd-action sd-save-btn");
        __e167.setAttribute("data-testid", "skill-save");
        __c166.push(bindProp(__e167, "disabled", () => !$scope.dirty() || $scope.saving()));
        __c166.push(on(__e167, "click", function() {
          const __h = $scope.save;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e168 = document.createElement("sui-icon");
        __e168.setAttribute("name", "save");
        __e168.setAttribute("size", "xs");
        __e167.appendChild(__e168);
        const __e169 = document.createElement("span");
        const __t170 = document.createTextNode("");
        __e169.appendChild(__t170);
        __c166.push(bindText(__t170, () => $scope.saving() ? "Saving…" : $scope.dirty() ? "Save" : "Saved"));
        __e167.appendChild(__e169);
        __frag165.appendChild(__e167);
        return { nodes: Array.from(__frag165.childNodes), cleanup: () => {
          for (const c of __c166)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b49 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.saveBanner())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag172 = document.createDocumentFragment();
        const __c173 = [];
        const __e174 = document.createElement("sui-alert");
        __c173.push(bindAttr(__e174, "tone", () => $scope.saveBanner().ok ? "success" : "destructive"));
        __e174.setAttribute("class", "sd-save-banner");
        const __t175 = document.createTextNode("");
        __e174.appendChild(__t175);
        __c173.push(bindText(__t175, () => $scope.saveBanner().msg));
        __frag172.appendChild(__e174);
        return { nodes: Array.from(__frag172.childNodes), cleanup: () => {
          for (const c of __c173)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b53 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selectedFile())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag180 = document.createDocumentFragment();
        const __c181 = [];
        const __e182 = document.createElement("div");
        __e182.setAttribute("class", "se-empty-state");
        __e182.setAttribute("role", "status");
        __e182.setAttribute("aria-live", "polite");
        __e182.setAttribute("aria-busy", "true");
        const __e183 = document.createElement("sui-spinner");
        __e183.setAttribute("size", "md");
        __e182.appendChild(__e183);
        const __e184 = document.createElement("span");
        __e184.setAttribute("class", "se-empty-desc");
        const __t185 = document.createTextNode("Loading file…");
        __e184.appendChild(__t185);
        __e182.appendChild(__e184);
        __frag180.appendChild(__e182);
        return { nodes: Array.from(__frag180.childNodes), cleanup: () => {
          for (const c of __c181)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __renderWhileLoading_1 = () => {
    const __frag177 = document.createDocumentFragment();
    const __c178 = [];
    const __ifAnchor179 = document.createComment("if");
    __frag177.appendChild(__ifAnchor179);
    __c178.push(__wireBoundary_b53(__ifAnchor179));
    return { nodes: Array.from(__frag177.childNodes), cleanup: () => {
      for (const c of __c178)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __wireBoundary_b55 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selectedFile())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag191 = document.createDocumentFragment();
        const __c192 = [];
        const __e193 = document.createElement("sui-error-state");
        __e193.setAttribute("variant", "inline");
        __e193.setAttribute("size", "sm");
        __e193.setAttribute("title", "Couldn't load file");
        __e193.setAttribute("role", "alert");
        __e193.setAttribute("aria-live", "assertive");
        const __t194 = document.createTextNode("");
        __e193.appendChild(__t194);
        __c192.push(bindText(__t194, () => $scope.err.message));
        __frag191.appendChild(__e193);
        return { nodes: Array.from(__frag191.childNodes), cleanup: () => {
          for (const c of __c192)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __renderWhileError_1 = (err) => {
    const __childScope187 = Object.create($scope);
    __childScope187.err = err;
    return (($scope2) => {
      const __frag188 = document.createDocumentFragment();
      const __c189 = [];
      const __ifAnchor190 = document.createComment("if");
      __frag188.appendChild(__ifAnchor190);
      __c189.push(__wireBoundary_b55(__ifAnchor190));
      return { nodes: Array.from(__frag188.childNodes), cleanup: () => {
        for (const c of __c189)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope187);
  };
  const __wireBoundary_b59 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.fileContent().value.ext === "md")
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag204 = document.createDocumentFragment();
        const __c205 = [];
        const __e206 = document.createElement("sui-markdown-editor");
        __e206.setAttribute("data-testid", "skill-editor");
        __e206.setAttribute("full-height", "");
        __c205.push(bindAttr(__e206, "value", () => $scope.draft()));
        __c205.push(on(__e206, "sui-change", function() {
          const __h = $scope.onEditorInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c205.push(on(__e206, "input", function() {
          const __h = $scope.onEditorInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __frag204.appendChild(__e206);
        return { nodes: Array.from(__frag204.childNodes), cleanup: () => {
          for (const c of __c205)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag207 = document.createDocumentFragment();
        const __c208 = [];
        const __e209 = document.createElement("pre");
        __e209.setAttribute("class", "se-code-view");
        const __t210 = document.createTextNode("");
        __e209.appendChild(__t210);
        __c208.push(bindText(__t210, () => $scope.draft()));
        __frag207.appendChild(__e209);
        return { nodes: Array.from(__frag207.childNodes), cleanup: () => {
          for (const c of __c208)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b64 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skBody() || $scope.dirty())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag214 = document.createDocumentFragment();
        const __c215 = [];
        const __e216 = document.createElement("div");
        __e216.setAttribute("class", "se-editor-content");
        const __e217 = document.createElement("sui-markdown-editor");
        __e217.setAttribute("data-testid", "skill-editor");
        __e217.setAttribute("full-height", "");
        __c215.push(bindAttr(__e217, "value", () => $scope.draft()));
        __c215.push(on(__e217, "sui-change", function() {
          const __h = $scope.onEditorInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c215.push(on(__e217, "input", function() {
          const __h = $scope.onEditorInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e216.appendChild(__e217);
        __frag214.appendChild(__e216);
        return { nodes: Array.from(__frag214.childNodes), cleanup: () => {
          for (const c of __c215)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag218 = document.createDocumentFragment();
        const __c219 = [];
        const __e220 = document.createElement("div");
        __e220.setAttribute("class", "se-empty-state");
        const __e221 = document.createElement("sui-icon");
        __e221.setAttribute("name", "file");
        __e221.setAttribute("size", "xl");
        __e221.setAttribute("tone", "muted");
        __e220.appendChild(__e221);
        const __e222 = document.createElement("span");
        __e222.setAttribute("class", "se-empty-desc");
        const __t223 = document.createTextNode("This skill has no SKILL.md content yet.");
        __e222.appendChild(__t223);
        __e220.appendChild(__e222);
        __frag218.appendChild(__e220);
        return { nodes: Array.from(__frag218.childNodes), cleanup: () => {
          for (const c of __c219)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b58 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selectedFile() && $scope.fileContent().kind === "ok" && $scope.fileContent().value && $scope.fileContent().value.path === $scope.selectedFile())
        return 0;
      if (!$scope.selectedFile())
        return 1;
      return -1;
    },
    branches: [
      () => {
        const __frag200 = document.createDocumentFragment();
        const __c201 = [];
        const __e202 = document.createElement("div");
        __e202.setAttribute("class", "se-editor-content");
        const __ifAnchor203 = document.createComment("if");
        __e202.appendChild(__ifAnchor203);
        __c201.push(__wireBoundary_b59(__ifAnchor203));
        __frag200.appendChild(__e202);
        return { nodes: Array.from(__frag200.childNodes), cleanup: () => {
          for (const c of __c201)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag211 = document.createDocumentFragment();
        const __c212 = [];
        const __ifAnchor213 = document.createComment("if");
        __frag211.appendChild(__ifAnchor213);
        __c212.push(__wireBoundary_b64(__ifAnchor213));
        return { nodes: Array.from(__frag211.childNodes), cleanup: () => {
          for (const c of __c212)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b57 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.fileContent().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag197 = document.createDocumentFragment();
        const __c198 = [];
        const __ifAnchor199 = document.createComment("if");
        __frag197.appendChild(__ifAnchor199);
        __c198.push(__wireBoundary_b58(__ifAnchor199));
        return { nodes: Array.from(__frag197.childNodes), cleanup: () => {
          for (const c of __c198)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b78 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skAuthors().length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag265 = document.createDocumentFragment();
        const __c266 = [];
        const __e267 = document.createElement("div");
        __e267.setAttribute("class", "si-row");
        const __e268 = document.createElement("span");
        __e268.setAttribute("class", "si-row-key");
        const __t269 = document.createTextNode("authors");
        __e268.appendChild(__t269);
        __e267.appendChild(__e268);
        const __e270 = document.createElement("span");
        __e270.setAttribute("class", "si-row-val");
        const __t271 = document.createTextNode("");
        __e270.appendChild(__t271);
        __c266.push(bindText(__t271, () => $scope.skAuthorsText()));
        __e267.appendChild(__e270);
        __frag265.appendChild(__e267);
        return { nodes: Array.from(__frag265.childNodes), cleanup: () => {
          for (const c of __c266)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b82 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.descSavedAgo())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag278 = document.createDocumentFragment();
        const __c279 = [];
        const __e280 = document.createElement("span");
        __e280.setAttribute("class", "si-coll-meta");
        const __t281 = document.createTextNode("edited ");
        __e280.appendChild(__t281);
        const __t282 = document.createTextNode("");
        __e280.appendChild(__t282);
        __c279.push(bindText(__t282, () => $scope.descSavedAgo()));
        __frag278.appendChild(__e280);
        return { nodes: Array.from(__frag278.childNodes), cleanup: () => {
          for (const c of __c279)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b93 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.descBanner())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag296 = document.createDocumentFragment();
        const __c297 = [];
        const __e298 = document.createElement("sui-alert");
        __c297.push(bindAttr(__e298, "tone", () => $scope.descBanner().ok ? "success" : "warning"));
        __e298.setAttribute("class", "si-section-banner");
        const __t299 = document.createTextNode("");
        __e298.appendChild(__t299);
        __c297.push(bindText(__t299, () => $scope.descBanner().msg));
        __frag296.appendChild(__e298);
        return { nodes: Array.from(__frag296.childNodes), cleanup: () => {
          for (const c of __c297)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b100 = (anchor) => bindFor({
    anchor,
    items: () => $scope.tagsList(),
    key: (t, __i) => {
      const __child = Object.create($scope);
      __child.t = t;
      __child.__i = __i;
      return (($scope2) => t)(__child);
    },
    render: (t, __i) => {
      const __childScope314 = Object.create($scope);
      __childScope314.t = t;
      __childScope314.__i = __i;
      return (($scope2) => {
        const __frag315 = document.createDocumentFragment();
        const __c316 = [];
        const __e317 = document.createElement("span");
        __e317.setAttribute("class", "si-chip");
        const __e318 = document.createElement("span");
        __e318.setAttribute("class", "si-chip-label");
        const __t319 = document.createTextNode("");
        __e318.appendChild(__t319);
        __t319.textContent = $scope2.t;
        __e317.appendChild(__e318);
        const __e320 = document.createElement("button");
        __e320.setAttribute("type", "button");
        __e320.setAttribute("class", "si-chip-x");
        {
          const __av321 = "Remove tag " + $scope2.t;
          if (__av321 == null || __av321 === false || __av321 === "")
            __e320.removeAttribute("aria-label");
          else
            __e320.setAttribute("aria-label", __av321 === true ? "" : String(__av321));
        }
        {
          const __av322 = $scope2.t;
          if (__av322 == null || __av322 === false || __av322 === "")
            __e320.removeAttribute("data-tag");
          else
            __e320.setAttribute("data-tag", __av322 === true ? "" : String(__av322));
        }
        __c316.push(on(__e320, "click", function() {
          const __h = () => $scope2.removeTag($scope2.t);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t323 = document.createTextNode("×");
        __e320.appendChild(__t323);
        __e317.appendChild(__e320);
        __frag315.appendChild(__e317);
        return { nodes: Array.from(__frag315.childNodes), cleanup: () => {
          for (const c of __c316)
            try {
              c();
            } catch (e) {}
        }, update: (__new324, __newI325) => {
          $scope2.t = __new324;
          $scope2.__i = __newI325;
          __t319.textContent = $scope2.t;
          {
            const __av321 = "Remove tag " + $scope2.t;
            if (__av321 == null || __av321 === false || __av321 === "")
              __e320.removeAttribute("aria-label");
            else
              __e320.setAttribute("aria-label", __av321 === true ? "" : String(__av321));
          }
          {
            const __av322 = $scope2.t;
            if (__av322 == null || __av322 === false || __av322 === "")
              __e320.removeAttribute("data-tag");
            else
              __e320.setAttribute("data-tag", __av322 === true ? "" : String(__av322));
          }
        } };
      })(__childScope314);
    }
  });
  const __wireBoundary_b99 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tagsList().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag310 = document.createDocumentFragment();
        const __c311 = [];
        const __e312 = document.createElement("div");
        __e312.setAttribute("class", "si-chips");
        __e312.setAttribute("data-testid", "si-tag-chips");
        const __forAnchor313 = document.createComment("for");
        __e312.appendChild(__forAnchor313);
        __c311.push(__wireBoundary_b100(__forAnchor313));
        __frag310.appendChild(__e312);
        return { nodes: Array.from(__frag310.childNodes), cleanup: () => {
          for (const c of __c311)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag326 = document.createDocumentFragment();
        const __c327 = [];
        const __e328 = document.createElement("div");
        __e328.setAttribute("class", "si-empty-inline");
        const __t329 = document.createTextNode("No tags yet.");
        __e328.appendChild(__t329);
        __frag326.appendChild(__e328);
        return { nodes: Array.from(__frag326.childNodes), cleanup: () => {
          for (const c of __c327)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b115 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tagsBanner())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag341 = document.createDocumentFragment();
        const __c342 = [];
        const __e343 = document.createElement("sui-alert");
        __c342.push(bindAttr(__e343, "tone", () => $scope.tagsBanner().ok ? "success" : "warning"));
        __e343.setAttribute("class", "si-section-banner");
        const __t344 = document.createTextNode("");
        __e343.appendChild(__t344);
        __c342.push(bindText(__t344, () => $scope.tagsBanner().msg));
        __frag341.appendChild(__e343);
        return { nodes: Array.from(__frag341.childNodes), cleanup: () => {
          for (const c of __c342)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b120 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skTriggerHints().length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag351 = document.createDocumentFragment();
        const __c352 = [];
        const __e353 = document.createElement("span");
        __e353.setAttribute("class", "si-coll-count");
        const __t354 = document.createTextNode("");
        __e353.appendChild(__t354);
        __c352.push(bindText(__t354, () => $scope.skTriggerHints().length));
        __frag351.appendChild(__e353);
        return { nodes: Array.from(__frag351.childNodes), cleanup: () => {
          for (const c of __c352)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b123 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skTriggerHints(),
    key: (hint, __i) => {
      const __child = Object.create($scope);
      __child.hint = hint;
      __child.__i = __i;
      return (($scope2) => hint)(__child);
    },
    render: (hint, __i) => {
      const __childScope362 = Object.create($scope);
      __childScope362.hint = hint;
      __childScope362.__i = __i;
      return (($scope2) => {
        const __frag363 = document.createDocumentFragment();
        const __c364 = [];
        const __e365 = document.createElement("li");
        __e365.setAttribute("class", "si-trigger-row");
        const __t366 = document.createTextNode("");
        __e365.appendChild(__t366);
        __t366.textContent = $scope2.hint;
        __frag363.appendChild(__e365);
        return { nodes: Array.from(__frag363.childNodes), cleanup: () => {
          for (const c of __c364)
            try {
              c();
            } catch (e) {}
        }, update: (__new367, __newI368) => {
          $scope2.hint = __new367;
          $scope2.__i = __newI368;
          __t366.textContent = $scope2.hint;
        } };
      })(__childScope362);
    }
  });
  const __wireBoundary_b122 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skTriggerHints().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag358 = document.createDocumentFragment();
        const __c359 = [];
        const __e360 = document.createElement("ul");
        __e360.setAttribute("class", "si-trigger-list");
        const __forAnchor361 = document.createComment("for");
        __e360.appendChild(__forAnchor361);
        __c359.push(__wireBoundary_b123(__forAnchor361));
        __frag358.appendChild(__e360);
        const __e369 = document.createElement("p");
        __e369.setAttribute("class", "si-help");
        const __t370 = document.createTextNode(`
                      Trigger cues live in the skill body. Edit the SKILL.md to refine them.
                    `);
        __e369.appendChild(__t370);
        __frag358.appendChild(__e369);
        return { nodes: Array.from(__frag358.childNodes), cleanup: () => {
          for (const c of __c359)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag371 = document.createDocumentFragment();
        const __c372 = [];
        const __e373 = document.createElement("div");
        __e373.setAttribute("class", "si-empty-inline");
        const __t374 = document.createTextNode(`
                      No trigger cues declared. Library skills are surfaced by description match —
                      add `);
        __e373.appendChild(__t374);
        const __e375 = document.createElement("code");
        const __t376 = document.createTextNode("TRIGGER when:");
        __e375.appendChild(__t376);
        __e373.appendChild(__e375);
        const __t377 = document.createTextNode(` lines in the body to narrow.
                    `);
        __e373.appendChild(__t377);
        __frag371.appendChild(__e373);
        return { nodes: Array.from(__frag371.childNodes), cleanup: () => {
          for (const c of __c372)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b129 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skVariables(),
    key: (v, __i) => {
      const __child = Object.create($scope);
      __child.v = v;
      __child.__i = __i;
      return (($scope2) => v)(__child);
    },
    render: (v, __i) => {
      const __childScope391 = Object.create($scope);
      __childScope391.v = v;
      __childScope391.__i = __i;
      return (($scope2) => {
        const __frag392 = document.createDocumentFragment();
        const __c393 = [];
        const __e394 = document.createElement("div");
        __e394.setAttribute("class", "si-var-row");
        const __e395 = document.createElement("sui-icon");
        __e395.setAttribute("name", "braces");
        __e395.setAttribute("size", "xs");
        __e395.setAttribute("tone", "muted");
        __e394.appendChild(__e395);
        const __e396 = document.createElement("span");
        __e396.setAttribute("class", "si-var-name");
        const __t397 = document.createTextNode("");
        __e396.appendChild(__t397);
        __t397.textContent = $scope2.v;
        __e394.appendChild(__e396);
        __frag392.appendChild(__e394);
        return { nodes: Array.from(__frag392.childNodes), cleanup: () => {
          for (const c of __c393)
            try {
              c();
            } catch (e) {}
        }, update: (__new398, __newI399) => {
          $scope2.v = __new398;
          $scope2.__i = __newI399;
          __t397.textContent = $scope2.v;
        } };
      })(__childScope391);
    }
  });
  const __wireBoundary_b128 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skVariables().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag388 = document.createDocumentFragment();
        const __c389 = [];
        const __forAnchor390 = document.createComment("for");
        __frag388.appendChild(__forAnchor390);
        __c389.push(__wireBoundary_b129(__forAnchor390));
        const __e400 = document.createElement("p");
        __e400.setAttribute("class", "si-help");
        const __t401 = document.createTextNode(`
                      Defaults declared in `);
        __e400.appendChild(__t401);
        const __e402 = document.createElement("code");
        const __t403 = document.createTextNode("assets/vars.json");
        __e402.appendChild(__t403);
        __e400.appendChild(__e402);
        const __t404 = document.createTextNode(`. Edit that file to change.
                    `);
        __e400.appendChild(__t404);
        __frag388.appendChild(__e400);
        return { nodes: Array.from(__frag388.childNodes), cleanup: () => {
          for (const c of __c389)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag405 = document.createDocumentFragment();
        const __c406 = [];
        const __e407 = document.createElement("div");
        __e407.setAttribute("class", "si-empty-inline");
        const __t408 = document.createTextNode("No variables declared.");
        __e407.appendChild(__t408);
        __frag405.appendChild(__e407);
        return { nodes: Array.from(__frag405.childNodes), cleanup: () => {
          for (const c of __c406)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b135 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skReferences(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f)(__child);
    },
    render: (f, __i) => {
      const __childScope422 = Object.create($scope);
      __childScope422.f = f;
      __childScope422.__i = __i;
      return (($scope2) => {
        __childScope422.__v = signal2(0);
        const __frag423 = document.createDocumentFragment();
        const __c424 = [];
        const __e425 = document.createElement("a");
        __e425.setAttribute("class", "si-file-row");
        __e425.setAttribute("data-testid", "si-ref-link");
        __c424.push(bindAttr(__e425, "href", () => (void $scope2.__v(), "/skills/" + $scope2.skId() + "?file=" + $scope2.f)));
        {
          const __av426 = $scope2.f;
          if (__av426 == null || __av426 === false || __av426 === "")
            __e425.removeAttribute("title");
          else
            __e425.setAttribute("title", __av426 === true ? "" : String(__av426));
        }
        const __e427 = document.createElement("sui-icon");
        __e427.setAttribute("size", "xs");
        __c424.push(bindAttr(__e427, "name", () => (void $scope2.__v(), $scope2.fileMetas()[$scope2.f] ? $scope2.fileMetas()[$scope2.f].icon : "file")));
        __c424.push(bindAttr(__e427, "tone", () => (void $scope2.__v(), $scope2.fileMetas()[$scope2.f] ? $scope2.fileMetas()[$scope2.f].tone : "muted")));
        __e425.appendChild(__e427);
        const __e428 = document.createElement("span");
        __e428.setAttribute("class", "si-file-name");
        const __t429 = document.createTextNode("");
        __e428.appendChild(__t429);
        __t429.textContent = $scope2.f;
        __e425.appendChild(__e428);
        __frag423.appendChild(__e425);
        return { nodes: Array.from(__frag423.childNodes), cleanup: () => {
          for (const c of __c424)
            try {
              c();
            } catch (e) {}
        }, update: (__new430, __newI431) => {
          $scope2.f = __new430;
          $scope2.__i = __newI431;
          {
            const __av426 = $scope2.f;
            if (__av426 == null || __av426 === false || __av426 === "")
              __e425.removeAttribute("title");
            else
              __e425.setAttribute("title", __av426 === true ? "" : String(__av426));
          }
          __t429.textContent = $scope2.f;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope422);
    }
  });
  const __wireBoundary_b134 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skReferences().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag419 = document.createDocumentFragment();
        const __c420 = [];
        const __forAnchor421 = document.createComment("for");
        __frag419.appendChild(__forAnchor421);
        __c420.push(__wireBoundary_b135(__forAnchor421));
        return { nodes: Array.from(__frag419.childNodes), cleanup: () => {
          for (const c of __c420)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag432 = document.createDocumentFragment();
        const __c433 = [];
        const __e434 = document.createElement("div");
        __e434.setAttribute("class", "si-empty-inline");
        const __t435 = document.createTextNode(`
                      No companion docs. Add markdown files under `);
        __e434.appendChild(__t435);
        const __e436 = document.createElement("code");
        const __t437 = document.createTextNode("references/");
        __e436.appendChild(__t437);
        __e434.appendChild(__e436);
        const __t438 = document.createTextNode(`.
                    `);
        __e434.appendChild(__t438);
        __frag432.appendChild(__e434);
        return { nodes: Array.from(__frag432.childNodes), cleanup: () => {
          for (const c of __c433)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b40 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.skillId())
        return 0;
      if ($scope.skill().kind === "loading")
        return 1;
      if ($scope.skill().kind === "error")
        return 2;
      if ($scope.sk())
        return 3;
      return -1;
    },
    branches: [
      () => {
        const __frag122 = document.createDocumentFragment();
        const __c123 = [];
        const __e124 = document.createElement("div");
        __e124.setAttribute("class", "se-empty-state");
        const __e125 = document.createElement("sui-icon");
        __e125.setAttribute("name", "zap");
        __e125.setAttribute("size", "xl");
        __e125.setAttribute("class", "se-empty-icon");
        __e125.setAttribute("tone", "muted");
        __e124.appendChild(__e125);
        const __e126 = document.createElement("span");
        __e126.setAttribute("class", "se-empty-title");
        const __t127 = document.createTextNode("No skill open");
        __e126.appendChild(__t127);
        __e124.appendChild(__e126);
        const __e128 = document.createElement("span");
        __e128.setAttribute("class", "se-empty-desc");
        const __t129 = document.createTextNode("Select a skill from the explorer to view its files and edit content.");
        __e128.appendChild(__t129);
        __e124.appendChild(__e128);
        __frag122.appendChild(__e124);
        return { nodes: Array.from(__frag122.childNodes), cleanup: () => {
          for (const c of __c123)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag130 = document.createDocumentFragment();
        const __c131 = [];
        const __e132 = document.createElement("div");
        __e132.setAttribute("class", "se-empty-state");
        __e132.setAttribute("role", "status");
        __e132.setAttribute("aria-live", "polite");
        __e132.setAttribute("aria-busy", "true");
        const __e133 = document.createElement("sui-spinner");
        __e133.setAttribute("size", "md");
        __e132.appendChild(__e133);
        const __e134 = document.createElement("span");
        __e134.setAttribute("class", "se-empty-desc");
        const __t135 = document.createTextNode("Loading skill…");
        __e134.appendChild(__t135);
        __e132.appendChild(__e134);
        __frag130.appendChild(__e132);
        return { nodes: Array.from(__frag130.childNodes), cleanup: () => {
          for (const c of __c131)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag136 = document.createDocumentFragment();
        const __c137 = [];
        const __e138 = document.createElement("div");
        __e138.setAttribute("class", "se-empty-state");
        const __e139 = document.createElement("sui-icon");
        __e139.setAttribute("name", "alert-triangle");
        __e139.setAttribute("size", "xl");
        __e139.setAttribute("tone", "warning");
        __e138.appendChild(__e139);
        const __e140 = document.createElement("span");
        __e140.setAttribute("class", "se-empty-title");
        const __t141 = document.createTextNode("Couldn't load skill");
        __e140.appendChild(__t141);
        __e138.appendChild(__e140);
        const __e142 = document.createElement("span");
        __e142.setAttribute("class", "se-empty-desc");
        const __t143 = document.createTextNode("");
        __e142.appendChild(__t143);
        __c137.push(bindText(__t143, () => $scope.skill().error.message));
        __e138.appendChild(__e142);
        __frag136.appendChild(__e138);
        return { nodes: Array.from(__frag136.childNodes), cleanup: () => {
          for (const c of __c137)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag144 = document.createDocumentFragment();
        const __c145 = [];
        const __e146 = document.createElement("div");
        __e146.setAttribute("class", "sd-pane");
        const __e147 = document.createElement("div");
        __e147.setAttribute("class", "sd-editor");
        const __e148 = document.createElement("div");
        __e148.setAttribute("class", "se-toolbar");
        const __e149 = document.createElement("div");
        __e149.setAttribute("class", "se-breadcrumb");
        const __e150 = document.createElement("a");
        __e150.setAttribute("href", "/skills");
        const __t151 = document.createTextNode("skills");
        __e150.appendChild(__t151);
        __e149.appendChild(__e150);
        const __e152 = document.createElement("span");
        __e152.setAttribute("class", "sep");
        const __t153 = document.createTextNode("/");
        __e152.appendChild(__t153);
        __e149.appendChild(__e152);
        const __e154 = document.createElement("span");
        __e154.setAttribute("class", "active");
        const __t155 = document.createTextNode("");
        __e154.appendChild(__t155);
        __c145.push(bindText(__t155, () => $scope.skName()));
        __e149.appendChild(__e154);
        const __ifAnchor156 = document.createComment("if");
        __e149.appendChild(__ifAnchor156);
        __c145.push(__wireBoundary_b43(__ifAnchor156));
        __e148.appendChild(__e149);
        const __e163 = document.createElement("div");
        __e163.setAttribute("class", "se-toolbar-actions");
        const __ifAnchor164 = document.createComment("if");
        __e163.appendChild(__ifAnchor164);
        __c145.push(__wireBoundary_b45(__ifAnchor164));
        __e148.appendChild(__e163);
        __e147.appendChild(__e148);
        const __ifAnchor171 = document.createComment("if");
        __e147.appendChild(__ifAnchor171);
        __c145.push(__wireBoundary_b49(__ifAnchor171));
        const __e176 = document.createElement("div");
        __e176.setAttribute("class", "se-editor-body");
        const __wlAnchor186 = document.createComment("while-loading");
        __e176.appendChild(__wlAnchor186);
        __c145.push(bindWhileLoading({ anchor: __wlAnchor186, signal: $scope.fileContent, render: __renderWhileLoading_1 }));
        const __weAnchor195 = document.createComment("while-error");
        __e176.appendChild(__weAnchor195);
        __c145.push(bindWhileError({ anchor: __weAnchor195, signal: $scope.fileContent, render: __renderWhileError_1 }));
        const __ifAnchor196 = document.createComment("if");
        __e176.appendChild(__ifAnchor196);
        __c145.push(__wireBoundary_b57(__ifAnchor196));
        __e147.appendChild(__e176);
        __e146.appendChild(__e147);
        const __e224 = document.createElement("aside");
        __e224.setAttribute("class", "skills-info");
        __e224.setAttribute("aria-label", "Skill metadata panel");
        const __e225 = document.createElement("div");
        __e225.setAttribute("class", "si-header");
        const __e226 = document.createElement("div");
        __e226.setAttribute("class", "si-header-title");
        const __e227 = document.createElement("sui-icon");
        __e227.setAttribute("name", "zap");
        __e227.setAttribute("size", "sm");
        __e227.setAttribute("tone", "primary");
        __e226.appendChild(__e227);
        const __e228 = document.createElement("div");
        __e228.setAttribute("class", "si-header-text");
        const __e229 = document.createElement("span");
        __e229.setAttribute("class", "si-name");
        const __t230 = document.createTextNode("");
        __e229.appendChild(__t230);
        __c145.push(bindText(__t230, () => $scope.skName()));
        __e228.appendChild(__e229);
        const __e231 = document.createElement("span");
        __e231.setAttribute("class", "si-skill-id");
        const __t232 = document.createTextNode("");
        __e231.appendChild(__t232);
        __c145.push(bindText(__t232, () => $scope.skId()));
        __e228.appendChild(__e231);
        __e226.appendChild(__e228);
        __e225.appendChild(__e226);
        __e224.appendChild(__e225);
        const __e233 = document.createElement("div");
        __e233.setAttribute("class", "si-body");
        const __e234 = document.createElement("sui-collapsible");
        __e234.setAttribute("class", "si-coll");
        __e234.setAttribute("data-testid", "si-section-identity");
        __c145.push(bindAttr(__e234, "open", () => $scope.openIdentityAttr()));
        __c145.push(on(__e234, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("identity", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e235 = document.createElement("div");
        __e235.setAttribute("slot", "trigger");
        __e235.setAttribute("class", "si-coll-trigger");
        const __e236 = document.createElement("sui-icon");
        __e236.setAttribute("name", "id-card");
        __e236.setAttribute("size", "xs");
        __e236.setAttribute("tone", "muted");
        __e235.appendChild(__e236);
        const __e237 = document.createElement("span");
        const __t238 = document.createTextNode("Identity");
        __e237.appendChild(__t238);
        __e235.appendChild(__e237);
        const __e239 = document.createElement("sui-icon");
        __e239.setAttribute("class", "si-coll-caret");
        __e239.setAttribute("name", "chevron-down");
        __e239.setAttribute("size", "xs");
        __e239.setAttribute("tone", "muted");
        __e235.appendChild(__e239);
        __e234.appendChild(__e235);
        const __e240 = document.createElement("div");
        __e240.setAttribute("class", "si-coll-body");
        const __e241 = document.createElement("div");
        __e241.setAttribute("class", "si-row");
        const __e242 = document.createElement("span");
        __e242.setAttribute("class", "si-row-key");
        const __t243 = document.createTextNode("name");
        __e242.appendChild(__t243);
        __e241.appendChild(__e242);
        const __e244 = document.createElement("span");
        __e244.setAttribute("class", "si-row-val");
        const __t245 = document.createTextNode("");
        __e244.appendChild(__t245);
        __c145.push(bindText(__t245, () => $scope.skName()));
        __e241.appendChild(__e244);
        __e240.appendChild(__e241);
        const __e246 = document.createElement("div");
        __e246.setAttribute("class", "si-row");
        const __e247 = document.createElement("span");
        __e247.setAttribute("class", "si-row-key");
        const __t248 = document.createTextNode("id");
        __e247.appendChild(__t248);
        __e246.appendChild(__e247);
        const __e249 = document.createElement("span");
        __e249.setAttribute("class", "si-row-val si-row-mono");
        const __e250 = document.createElement("span");
        __e250.setAttribute("class", "si-id-text");
        __c145.push(bindAttr(__e250, "title", () => $scope.skId()));
        const __t251 = document.createTextNode("");
        __e250.appendChild(__t251);
        __c145.push(bindText(__t251, () => $scope.skId()));
        __e249.appendChild(__e250);
        const __e252 = document.createElement("button");
        __e252.setAttribute("class", "si-copy-btn");
        __e252.setAttribute("type", "button");
        __c145.push(on(__e252, "click", function() {
          const __h = $scope.copyId;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e252.setAttribute("aria-label", "Copy skill id");
        const __e253 = document.createElement("sui-icon");
        __e253.setAttribute("name", "copy");
        __e253.setAttribute("size", "xs");
        __e252.appendChild(__e253);
        __e249.appendChild(__e252);
        __e246.appendChild(__e249);
        __e240.appendChild(__e246);
        const __e254 = document.createElement("div");
        __e254.setAttribute("class", "si-row");
        const __e255 = document.createElement("span");
        __e255.setAttribute("class", "si-row-key");
        const __t256 = document.createTextNode("kind");
        __e255.appendChild(__t256);
        __e254.appendChild(__e255);
        const __e257 = document.createElement("span");
        __e257.setAttribute("class", "si-row-val");
        const __t258 = document.createTextNode("");
        __e257.appendChild(__t258);
        __c145.push(bindText(__t258, () => $scope.skKind()));
        __e254.appendChild(__e257);
        __e240.appendChild(__e254);
        const __e259 = document.createElement("div");
        __e259.setAttribute("class", "si-row");
        const __e260 = document.createElement("span");
        __e260.setAttribute("class", "si-row-key");
        const __t261 = document.createTextNode("source");
        __e260.appendChild(__t261);
        __e259.appendChild(__e260);
        const __e262 = document.createElement("span");
        __e262.setAttribute("class", "si-row-val");
        const __t263 = document.createTextNode("");
        __e262.appendChild(__t263);
        __c145.push(bindText(__t263, () => $scope.skSourceLabel()));
        __e259.appendChild(__e262);
        __e240.appendChild(__e259);
        const __ifAnchor264 = document.createComment("if");
        __e240.appendChild(__ifAnchor264);
        __c145.push(__wireBoundary_b78(__ifAnchor264));
        __e234.appendChild(__e240);
        __e233.appendChild(__e234);
        const __e272 = document.createElement("sui-collapsible");
        __e272.setAttribute("class", "si-coll");
        __e272.setAttribute("data-testid", "si-section-description");
        __c145.push(bindAttr(__e272, "open", () => $scope.openDescriptionAttr()));
        __c145.push(on(__e272, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("description", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e273 = document.createElement("div");
        __e273.setAttribute("slot", "trigger");
        __e273.setAttribute("class", "si-coll-trigger");
        const __e274 = document.createElement("sui-icon");
        __e274.setAttribute("name", "info");
        __e274.setAttribute("size", "xs");
        __e274.setAttribute("tone", "muted");
        __e273.appendChild(__e274);
        const __e275 = document.createElement("span");
        const __t276 = document.createTextNode("Description");
        __e275.appendChild(__t276);
        __e273.appendChild(__e275);
        const __ifAnchor277 = document.createComment("if");
        __e273.appendChild(__ifAnchor277);
        __c145.push(__wireBoundary_b82(__ifAnchor277));
        const __e283 = document.createElement("sui-icon");
        __e283.setAttribute("class", "si-coll-caret");
        __e283.setAttribute("name", "chevron-down");
        __e283.setAttribute("size", "xs");
        __e283.setAttribute("tone", "muted");
        __e273.appendChild(__e283);
        __e272.appendChild(__e273);
        const __e284 = document.createElement("div");
        __e284.setAttribute("class", "si-coll-body");
        const __e285 = document.createElement("textarea");
        __e285.setAttribute("class", "si-textarea");
        __e285.setAttribute("data-testid", "si-desc-edit");
        __e285.setAttribute("rows", "4");
        __e285.setAttribute("placeholder", "What this skill does (one-liner shown in the rail)…");
        __c145.push(bindProp(__e285, "disabled", () => $scope.descSaving()));
        __c145.push(bindProp(__e285, "value", () => $scope.descDraft()));
        __c145.push(on(__e285, "input", function() {
          const __h = $scope.onDescInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e284.appendChild(__e285);
        const __e286 = document.createElement("div");
        __e286.setAttribute("class", "si-section-actions");
        const __e287 = document.createElement("span");
        __e287.setAttribute("class", "si-edit-hint");
        const __t288 = document.createTextNode("");
        __e287.appendChild(__t288);
        __c145.push(bindText(__t288, () => $scope.descDirty() ? "Unsaved changes" : "Synced."));
        __e286.appendChild(__e287);
        const __e289 = document.createElement("button");
        __e289.setAttribute("class", "sd-action");
        __e289.setAttribute("type", "button");
        __e289.setAttribute("data-testid", "si-desc-reset");
        __c145.push(bindProp(__e289, "disabled", () => !$scope.descDirty() || $scope.descSaving()));
        __c145.push(on(__e289, "click", function() {
          const __h = $scope.resetDesc;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t290 = document.createTextNode("Cancel");
        __e289.appendChild(__t290);
        __e286.appendChild(__e289);
        const __e291 = document.createElement("button");
        __e291.setAttribute("class", "sd-action sd-save-btn");
        __e291.setAttribute("type", "button");
        __e291.setAttribute("data-testid", "si-desc-save");
        __c145.push(bindProp(__e291, "disabled", () => !$scope.descDirty() || $scope.descSaving()));
        __c145.push(on(__e291, "click", function() {
          const __h = $scope.saveDesc;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e292 = document.createElement("sui-icon");
        __e292.setAttribute("name", "save");
        __e292.setAttribute("size", "xs");
        __e291.appendChild(__e292);
        const __e293 = document.createElement("span");
        const __t294 = document.createTextNode("");
        __e293.appendChild(__t294);
        __c145.push(bindText(__t294, () => $scope.descSaving() ? "Saving…" : "Save"));
        __e291.appendChild(__e293);
        __e286.appendChild(__e291);
        __e284.appendChild(__e286);
        const __ifAnchor295 = document.createComment("if");
        __e284.appendChild(__ifAnchor295);
        __c145.push(__wireBoundary_b93(__ifAnchor295));
        __e272.appendChild(__e284);
        __e233.appendChild(__e272);
        const __e300 = document.createElement("sui-collapsible");
        __e300.setAttribute("class", "si-coll");
        __e300.setAttribute("data-testid", "si-section-tags");
        __c145.push(bindAttr(__e300, "open", () => $scope.openTagsAttr()));
        __c145.push(on(__e300, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("tags", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e301 = document.createElement("div");
        __e301.setAttribute("slot", "trigger");
        __e301.setAttribute("class", "si-coll-trigger");
        const __e302 = document.createElement("sui-icon");
        __e302.setAttribute("name", "tag");
        __e302.setAttribute("size", "xs");
        __e302.setAttribute("tone", "muted");
        __e301.appendChild(__e302);
        const __e303 = document.createElement("span");
        const __t304 = document.createTextNode("Tags");
        __e303.appendChild(__t304);
        __e301.appendChild(__e303);
        const __e305 = document.createElement("span");
        __e305.setAttribute("class", "si-coll-count");
        const __t306 = document.createTextNode("");
        __e305.appendChild(__t306);
        __c145.push(bindText(__t306, () => $scope.tagsList().length));
        __e301.appendChild(__e305);
        const __e307 = document.createElement("sui-icon");
        __e307.setAttribute("class", "si-coll-caret");
        __e307.setAttribute("name", "chevron-down");
        __e307.setAttribute("size", "xs");
        __e307.setAttribute("tone", "muted");
        __e301.appendChild(__e307);
        __e300.appendChild(__e301);
        const __e308 = document.createElement("div");
        __e308.setAttribute("class", "si-coll-body");
        const __ifAnchor309 = document.createComment("if");
        __e308.appendChild(__ifAnchor309);
        __c145.push(__wireBoundary_b99(__ifAnchor309));
        const __e330 = document.createElement("input");
        __e330.setAttribute("class", "si-input");
        __e330.setAttribute("type", "text");
        __e330.setAttribute("data-testid", "si-tags-input");
        __e330.setAttribute("placeholder", "Add tag (Enter or , to commit)");
        __c145.push(bindProp(__e330, "disabled", () => $scope.tagsSaving()));
        __c145.push(bindProp(__e330, "value", () => $scope.tagsInput()));
        __c145.push(on(__e330, "input", function() {
          const __h = $scope.onTagsInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c145.push(on(__e330, "keydown", function() {
          const __h = $scope.onTagsKeydown;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e308.appendChild(__e330);
        const __e331 = document.createElement("div");
        __e331.setAttribute("class", "si-section-actions");
        const __e332 = document.createElement("span");
        __e332.setAttribute("class", "si-edit-hint");
        const __t333 = document.createTextNode("");
        __e332.appendChild(__t333);
        __c145.push(bindText(__t333, () => $scope.tagsDirty() ? "Unsaved changes" : "Synced."));
        __e331.appendChild(__e332);
        const __e334 = document.createElement("button");
        __e334.setAttribute("class", "sd-action");
        __e334.setAttribute("type", "button");
        __e334.setAttribute("data-testid", "si-tags-reset");
        __c145.push(bindProp(__e334, "disabled", () => !$scope.tagsDirty() || $scope.tagsSaving()));
        __c145.push(on(__e334, "click", function() {
          const __h = $scope.resetTags;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t335 = document.createTextNode("Cancel");
        __e334.appendChild(__t335);
        __e331.appendChild(__e334);
        const __e336 = document.createElement("button");
        __e336.setAttribute("class", "sd-action sd-save-btn");
        __e336.setAttribute("type", "button");
        __e336.setAttribute("data-testid", "si-tags-save");
        __c145.push(bindProp(__e336, "disabled", () => !$scope.tagsDirty() || $scope.tagsSaving()));
        __c145.push(on(__e336, "click", function() {
          const __h = $scope.saveTags;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e337 = document.createElement("sui-icon");
        __e337.setAttribute("name", "save");
        __e337.setAttribute("size", "xs");
        __e336.appendChild(__e337);
        const __e338 = document.createElement("span");
        const __t339 = document.createTextNode("");
        __e338.appendChild(__t339);
        __c145.push(bindText(__t339, () => $scope.tagsSaving() ? "Saving…" : "Save"));
        __e336.appendChild(__e338);
        __e331.appendChild(__e336);
        __e308.appendChild(__e331);
        const __ifAnchor340 = document.createComment("if");
        __e308.appendChild(__ifAnchor340);
        __c145.push(__wireBoundary_b115(__ifAnchor340));
        __e300.appendChild(__e308);
        __e233.appendChild(__e300);
        const __e345 = document.createElement("sui-collapsible");
        __e345.setAttribute("class", "si-coll");
        __e345.setAttribute("data-testid", "si-section-triggers");
        __c145.push(bindAttr(__e345, "open", () => $scope.openTriggersAttr()));
        __c145.push(on(__e345, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("triggers", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e346 = document.createElement("div");
        __e346.setAttribute("slot", "trigger");
        __e346.setAttribute("class", "si-coll-trigger");
        const __e347 = document.createElement("sui-icon");
        __e347.setAttribute("name", "zap");
        __e347.setAttribute("size", "xs");
        __e347.setAttribute("tone", "muted");
        __e346.appendChild(__e347);
        const __e348 = document.createElement("span");
        const __t349 = document.createTextNode("Triggers");
        __e348.appendChild(__t349);
        __e346.appendChild(__e348);
        const __ifAnchor350 = document.createComment("if");
        __e346.appendChild(__ifAnchor350);
        __c145.push(__wireBoundary_b120(__ifAnchor350));
        const __e355 = document.createElement("sui-icon");
        __e355.setAttribute("class", "si-coll-caret");
        __e355.setAttribute("name", "chevron-down");
        __e355.setAttribute("size", "xs");
        __e355.setAttribute("tone", "muted");
        __e346.appendChild(__e355);
        __e345.appendChild(__e346);
        const __e356 = document.createElement("div");
        __e356.setAttribute("class", "si-coll-body");
        const __ifAnchor357 = document.createComment("if");
        __e356.appendChild(__ifAnchor357);
        __c145.push(__wireBoundary_b122(__ifAnchor357));
        __e345.appendChild(__e356);
        __e233.appendChild(__e345);
        const __e378 = document.createElement("sui-collapsible");
        __e378.setAttribute("class", "si-coll");
        __e378.setAttribute("data-testid", "si-section-variables");
        __c145.push(bindAttr(__e378, "open", () => $scope.openVariablesAttr()));
        __c145.push(on(__e378, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("variables", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e379 = document.createElement("div");
        __e379.setAttribute("slot", "trigger");
        __e379.setAttribute("class", "si-coll-trigger");
        const __e380 = document.createElement("sui-icon");
        __e380.setAttribute("name", "braces");
        __e380.setAttribute("size", "xs");
        __e380.setAttribute("tone", "muted");
        __e379.appendChild(__e380);
        const __e381 = document.createElement("span");
        const __t382 = document.createTextNode("Variables");
        __e381.appendChild(__t382);
        __e379.appendChild(__e381);
        const __e383 = document.createElement("span");
        __e383.setAttribute("class", "si-coll-count");
        const __t384 = document.createTextNode("");
        __e383.appendChild(__t384);
        __c145.push(bindText(__t384, () => $scope.skVariables().length));
        __e379.appendChild(__e383);
        const __e385 = document.createElement("sui-icon");
        __e385.setAttribute("class", "si-coll-caret");
        __e385.setAttribute("name", "chevron-down");
        __e385.setAttribute("size", "xs");
        __e385.setAttribute("tone", "muted");
        __e379.appendChild(__e385);
        __e378.appendChild(__e379);
        const __e386 = document.createElement("div");
        __e386.setAttribute("class", "si-coll-body");
        const __ifAnchor387 = document.createComment("if");
        __e386.appendChild(__ifAnchor387);
        __c145.push(__wireBoundary_b128(__ifAnchor387));
        __e378.appendChild(__e386);
        __e233.appendChild(__e378);
        const __e409 = document.createElement("sui-collapsible");
        __e409.setAttribute("class", "si-coll");
        __e409.setAttribute("data-testid", "si-section-references");
        __c145.push(bindAttr(__e409, "open", () => $scope.openReferencesAttr()));
        __c145.push(on(__e409, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("references", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e410 = document.createElement("div");
        __e410.setAttribute("slot", "trigger");
        __e410.setAttribute("class", "si-coll-trigger");
        const __e411 = document.createElement("sui-icon");
        __e411.setAttribute("name", "book-open");
        __e411.setAttribute("size", "xs");
        __e411.setAttribute("tone", "muted");
        __e410.appendChild(__e411);
        const __e412 = document.createElement("span");
        const __t413 = document.createTextNode("Reference docs");
        __e412.appendChild(__t413);
        __e410.appendChild(__e412);
        const __e414 = document.createElement("span");
        __e414.setAttribute("class", "si-coll-count");
        const __t415 = document.createTextNode("");
        __e414.appendChild(__t415);
        __c145.push(bindText(__t415, () => $scope.skReferences().length));
        __e410.appendChild(__e414);
        const __e416 = document.createElement("sui-icon");
        __e416.setAttribute("class", "si-coll-caret");
        __e416.setAttribute("name", "chevron-down");
        __e416.setAttribute("size", "xs");
        __e416.setAttribute("tone", "muted");
        __e410.appendChild(__e416);
        __e409.appendChild(__e410);
        const __e417 = document.createElement("div");
        __e417.setAttribute("class", "si-coll-body");
        const __ifAnchor418 = document.createComment("if");
        __e417.appendChild(__ifAnchor418);
        __c145.push(__wireBoundary_b134(__ifAnchor418));
        __e409.appendChild(__e417);
        __e233.appendChild(__e409);
        const __e439 = document.createElement("sui-collapsible");
        __e439.setAttribute("class", "si-coll si-coll-disabled");
        __e439.setAttribute("data-testid", "si-section-evals");
        __c145.push(bindAttr(__e439, "open", () => $scope.openEvalsAttr()));
        __c145.push(on(__e439, "sui-toggle", function() {
          const __h = (e) => $scope.toggleSection("evals", e.detail.open);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e440 = document.createElement("div");
        __e440.setAttribute("slot", "trigger");
        __e440.setAttribute("class", "si-coll-trigger");
        const __e441 = document.createElement("sui-icon");
        __e441.setAttribute("name", "flask-conical");
        __e441.setAttribute("size", "xs");
        __e441.setAttribute("tone", "muted");
        __e440.appendChild(__e441);
        const __e442 = document.createElement("span");
        const __t443 = document.createTextNode("Evals");
        __e442.appendChild(__t443);
        __e440.appendChild(__e442);
        const __e444 = document.createElement("span");
        __e444.setAttribute("class", "si-coll-soon");
        const __t445 = document.createTextNode("Coming soon");
        __e444.appendChild(__t445);
        __e440.appendChild(__e444);
        const __e446 = document.createElement("sui-icon");
        __e446.setAttribute("class", "si-coll-caret");
        __e446.setAttribute("name", "chevron-down");
        __e446.setAttribute("size", "xs");
        __e446.setAttribute("tone", "muted");
        __e440.appendChild(__e446);
        __e439.appendChild(__e440);
        const __e447 = document.createElement("div");
        __e447.setAttribute("class", "si-coll-body");
        const __e448 = document.createElement("div");
        __e448.setAttribute("class", "si-empty-inline si-evals-empty");
        const __e449 = document.createElement("sui-icon");
        __e449.setAttribute("name", "flask-conical");
        __e449.setAttribute("size", "md");
        __e449.setAttribute("tone", "muted");
        __e448.appendChild(__e449);
        const __e450 = document.createElement("p");
        const __t451 = document.createTextNode(`
                      Skill evals are not yet implemented. When they are, this section will
                      show pass/fail trends and let you run a test set against the skill.
                    `);
        __e450.appendChild(__t451);
        __e448.appendChild(__e450);
        __e447.appendChild(__e448);
        __e439.appendChild(__e447);
        __e233.appendChild(__e439);
        __e224.appendChild(__e233);
        __e146.appendChild(__e224);
        __frag144.appendChild(__e146);
        return { nodes: Array.from(__frag144.childNodes), cleanup: () => {
          for (const c of __c145)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "ship-view ship-view-skills");
      const __e1 = document.createElement("h1");
      __e1.setAttribute("class", "sr-only");
      const __t2 = document.createTextNode("Skills");
      __e1.appendChild(__t2);
      __e0.appendChild(__e1);
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "skills-ide");
      const __e4 = document.createElement("aside");
      __e4.setAttribute("class", "skills-explorer");
      __e4.setAttribute("aria-label", "Skills file explorer");
      const __e5 = document.createElement("div");
      __e5.setAttribute("class", "se-head");
      const __e6 = document.createElement("div");
      __e6.setAttribute("class", "se-search");
      const __e7 = document.createElement("sui-icon");
      __e7.setAttribute("name", "search");
      __e7.setAttribute("size", "sm");
      __e7.setAttribute("tone", "muted");
      __e6.appendChild(__e7);
      const __e8 = document.createElement("input");
      __e8.setAttribute("type", "text");
      __e8.setAttribute("placeholder", "Search skills…");
      __e8.setAttribute("spellcheck", "false");
      __e8.setAttribute("aria-label", "Search skills");
      __cleanups.push(bindValue(__e8, $scope.q));
      __cleanups.push(on(__e8, "input", function() {
        const __h = (e) => $scope.q.set(e.target.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e6.appendChild(__e8);
      __e5.appendChild(__e6);
      const __ifAnchor9 = document.createComment("if");
      __e5.appendChild(__ifAnchor9);
      __cleanups.push(__wireBoundary_b2(__ifAnchor9));
      __e4.appendChild(__e5);
      const __e24 = document.createElement("div");
      __e24.setAttribute("class", "se-body");
      __e24.setAttribute("role", "tree");
      __e24.setAttribute("aria-label", "Skills tree");
      const __wlAnchor34 = document.createComment("while-loading");
      __e24.appendChild(__wlAnchor34);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor34, signal: $scope.rail, render: __renderWhileLoading_0 }));
      const __weAnchor40 = document.createComment("while-error");
      __e24.appendChild(__weAnchor40);
      __cleanups.push(bindWhileError({ anchor: __weAnchor40, signal: $scope.rail, render: __renderWhileError_0 }));
      const __ifAnchor41 = document.createComment("if");
      __e24.appendChild(__ifAnchor41);
      __cleanups.push(__wireBoundary_b13(__ifAnchor41));
      __e4.appendChild(__e24);
      __e3.appendChild(__e4);
      const __e120 = document.createElement("main");
      __e120.setAttribute("class", "skills-main");
      const __ifAnchor121 = document.createComment("if");
      __e120.appendChild(__ifAnchor121);
      __cleanups.push(__wireBoundary_b40(__ifAnchor121));
      __e3.appendChild(__e120);
      __e0.appendChild(__e3);
      const __e452 = document.createElement("div");
      __e452.setAttribute("class", "skills-mobile");
      const __e453 = document.createElement("sui-icon");
      __e453.setAttribute("name", "monitor");
      __e453.setAttribute("size", "xl");
      __e453.setAttribute("tone", "muted");
      __e452.appendChild(__e453);
      const __e454 = document.createElement("span");
      const __t455 = document.createTextNode("Best experienced on desktop");
      __e454.appendChild(__t455);
      __e452.appendChild(__e454);
      __e0.appendChild(__e452);
      container.appendChild(__e0);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateBindValue(__m["b0"], $scope.q));
      if (__m["b1"])
        __cleanups.push(hydrateOn(__m["b1"], "input", function() {
          const __h = (e) => $scope.q.set(e.target.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b2"]) {
        const __t = __m["b2"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b2(__anchor));
      }
      if (__m["b9"])
        __cleanups.push(hydrateWhileLoading(__m["b9"], "b9", () => $scope.rail(), __renderWhileLoading_0));
      if (__m["b11"])
        __cleanups.push(hydrateWhileError(__m["b11"], "b11", () => $scope.rail(), __renderWhileError_0));
      if (__m["b13"]) {
        const __t = __m["b13"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b13(__anchor));
      }
      if (__m["b40"]) {
        const __t = __m["b40"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b40(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "01329411", dirty: true, builtAt: "2026-05-19T16:54:50Z" } });
export {
  page_default as default,
  ISLANDS
};
