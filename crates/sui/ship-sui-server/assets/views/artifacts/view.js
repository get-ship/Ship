// views/artifacts/page.sui
import { bindAttr, bindFor, bindIf, bindText, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateOn, hydrateText } from "@ship/sui-dom";

// views/artifacts/controller.ts
import { controller, derived, live, signal, unwrap } from "@ship/sui-runtime";

// views/artifacts/viewers.ts
var EXACT = {
  "text/html": (id) => ({ label: "canvas:read", url: `/canvas/${id}` }),
  "text/markdown": (id) => ({ label: "canvas:doc", url: `/canvas/${id}` }),
  "application/x-sui-scaffold": (id) => ({ label: "canvas:read", url: `/canvas/${id}` }),
  "application/x-excalidraw": (id) => ({ label: "canvas:sketch", url: `/canvas/${id}?mode=sketch` }),
  "application/x-git-diff": () => ({ label: "view/git", url: "/git" }),
  "text/x-diff": () => ({ label: "view/git", url: "/git" })
};
function viewerFor(row) {
  const m = (row.mime || "").toLowerCase().split(";")[0].trim();
  const hit = EXACT[m];
  if (hit)
    return hit(row.id);
  if (m.startsWith("image/"))
    return { label: "canvas:image", url: `/canvas/${row.id}` };
  return null;
}

// views/artifacts/decorate.ts
var MIME_BUCKETS = [
  { value: "image", prefix: "image/" },
  { value: "text", prefix: "text/" },
  { value: "json", prefix: "application/json" }
];
function bucketFor(mime) {
  for (const b of MIME_BUCKETS)
    if (mime.startsWith(b.prefix))
      return b.value;
  return "";
}
function previewKindFor(mime) {
  if (mime.startsWith("image/"))
    return "image";
  if (mime === "text/html")
    return "html";
  if (mime.startsWith("text/") || mime === "application/json" || mime === "application/xml")
    return "text";
  return "binary";
}
function formatSize(bytes) {
  if (!Number.isFinite(bytes) || bytes <= 0)
    return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let n = bytes;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  const unit = units[i] ?? "B";
  return `${n.toFixed(n < 10 && i > 0 ? 1 : 0)} ${unit}`;
}
function formatShortDate(iso) {
  if (!iso)
    return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime()))
    return iso;
  return d.toLocaleString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
}
function decorate(rows) {
  for (const r of rows) {
    r.size_label = formatSize(r.size);
    r.id_short = (r.id ?? "").slice(0, 12);
    r.created_short = formatShortDate(r.created_at);
    const hit = viewerFor(r);
    r.viewer_label = hit?.label ?? "download only";
    r.viewer_url = hit?.url ?? null;
  }
  return rows;
}
var AGE_BUCKETS = {
  "24h": 24 * 60 * 60 * 1000,
  "7d": 7 * 24 * 60 * 60 * 1000,
  "30d": 30 * 24 * 60 * 60 * 1000
};

// views/artifacts/controller.ts
var controller_default = controller({
  scope: (ctx) => {
    const { ship } = ctx;
    const navigate = ctx.navigate ?? ((p) => {
      if (typeof window !== "undefined")
        window.location.href = p;
    });
    const items = signal([]);
    const loading = signal(true);
    const loadError = signal("");
    const mimeFilter = signal("");
    const collectionFilter = signal("");
    const actorFilter = signal("");
    const ageFilter = signal("");
    const searchQuery = signal("");
    const detailItem = signal(null);
    const detailOpen = signal(false);
    const previewText = signal("");
    const previewLoading = signal(false);
    const previewError = signal("");
    const uploadOpen = signal(false);
    const uploadFiles = signal([]);
    const uploadCollection = signal("");
    const uploadError = signal("");
    const uploading = signal(false);
    const dragActive = signal(false);
    const collectionsLive = live(() => ship.collections.list());
    const collectionOptions = derived(() => {
      const unwrapped = unwrap(collectionsLive);
      return unwrapped?.items ?? [];
    });
    const actorOptions = derived(() => {
      const set = new Set;
      for (const r of items())
        if (r.owner_actor)
          set.add(r.owner_actor);
      return Array.from(set).sort();
    });
    const visibleItems = derived(() => {
      const q = searchQuery().trim().toLowerCase();
      const mb = mimeFilter();
      const cf = collectionFilter();
      const af = actorFilter();
      const age = ageFilter();
      const ageMs = age ? AGE_BUCKETS[age] ?? 0 : 0;
      const cutoff = ageMs > 0 ? Date.now() - ageMs : 0;
      return items().filter((r) => {
        if (mb && bucketFor(r.mime) !== mb)
          return false;
        if (cf && (r.collection_id ?? "") !== cf)
          return false;
        if (af && (r.owner_actor ?? "") !== af)
          return false;
        if (cutoff > 0) {
          const t = Date.parse(r.created_at);
          if (!Number.isFinite(t) || t < cutoff)
            return false;
        }
        if (q) {
          const hay = `${r.name ?? ""} ${r.id} ${r.mime} ${r.owner_actor ?? ""}`.toLowerCase();
          if (!hay.includes(q))
            return false;
        }
        return true;
      });
    });
    const totalCount = derived(() => items().length);
    const visibleCount = derived(() => visibleItems().length);
    const hasActiveFilters = derived(() => mimeFilter() !== "" || collectionFilter() !== "" || actorFilter() !== "" || ageFilter() !== "" || searchQuery().trim() !== "");
    const detailUrl = derived(() => detailItem() ? `/api/v1/artifacts/${detailItem().id}` : "");
    const previewKind = derived(() => detailItem() ? previewKindFor(detailItem().mime) : "binary");
    async function refresh() {
      loading.set(true);
      loadError.set("");
      try {
        const resp = await fetch("/api/v1/artifacts?limit=200", { credentials: "same-origin" });
        if (!resp.ok) {
          loadError.set(`HTTP ${resp.status}`);
          items.set([]);
          return;
        }
        const body = await resp.json();
        const rows = Array.isArray(body.data) ? body.data : [];
        items.set(decorate(rows));
      } catch (e) {
        loadError.set(e.message);
        items.set([]);
      } finally {
        loading.set(false);
      }
    }
    async function loadPreview(item) {
      const kind = previewKindFor(item.mime);
      if (kind !== "text")
        return;
      previewLoading.set(true);
      previewError.set("");
      previewText.set("");
      try {
        const text = await ship.artifacts.get({ id: item.id });
        previewText.set(text.length > 50000 ? text.slice(0, 50000) + `
…(truncated)` : text);
      } catch (e) {
        previewError.set(e.message);
      } finally {
        previewLoading.set(false);
      }
    }
    refresh();
    const uploadArtifact = (args) => ship.artifacts.upload(args);
    async function activateRow(id) {
      const row = items().find((r) => r.id === id) ?? null;
      if (!row)
        return;
      if (row.viewer_url) {
        navigate(row.viewer_url);
        return;
      }
      detailItem.set(row);
      detailOpen.set(true);
      previewText.set("");
      previewError.set("");
      await loadPreview(row);
    }
    async function uploadDirect(file) {
      uploading.set(true);
      uploadError.set("");
      try {
        await uploadArtifact({ file });
        await refresh();
      } catch (e) {
        uploadError.set(e.message);
        uploadOpen.set(true);
      } finally {
        uploading.set(false);
      }
    }
    return {
      items,
      loading,
      loadError,
      mimeFilter,
      collectionFilter,
      actorFilter,
      ageFilter,
      searchQuery,
      detailItem,
      detailOpen,
      previewText,
      previewLoading,
      previewError,
      uploadOpen,
      uploadFiles,
      uploadCollection,
      uploadError,
      uploading,
      dragActive,
      collectionsLive,
      collectionOptions,
      actorOptions,
      visibleItems,
      totalCount,
      visibleCount,
      hasActiveFilters,
      detailUrl,
      previewKind,
      refresh,
      loadPreview,
      uploadArtifact,
      uploadDirect,
      activateRow
    };
  },
  on: {
    setMimeFilter: (s, v) => {
      s.mimeFilter = v;
    },
    setCollectionFilter: (s, v) => {
      s.collectionFilter = v;
    },
    setActorFilter: (s, v) => {
      s.actorFilter = v;
    },
    setAgeFilter: (s, v) => {
      s.ageFilter = v;
    },
    setSearchQuery: (s, v) => {
      s.searchQuery = v;
    },
    clearFilters: (s) => {
      s.mimeFilter = "";
      s.collectionFilter = "";
      s.actorFilter = "";
      s.ageFilter = "";
      s.searchQuery = "";
    },
    closeDetail: (s) => {
      s.detailOpen = false;
      s.detailItem = null;
      s.previewText = "";
      s.previewError = "";
    },
    openUpload: (s) => {
      s.uploadOpen = true;
      s.uploadError = "";
      s.uploadFiles = [];
      s.uploadCollection = "";
    },
    closeUpload: (s) => {
      if (s.uploading)
        return;
      s.uploadOpen = false;
      s.uploadError = "";
      s.uploadFiles = [];
    },
    onUploadFiles: (s, files) => {
      s.uploadFiles = Array.isArray(files) ? files : [];
      s.uploadError = "";
    },
    onUploadCollection: (s, v) => {
      s.uploadCollection = v;
    },
    submitUpload: async (s) => {
      const files = s.uploadFiles;
      if (!files.length) {
        s.uploadError = "Choose a file to upload.";
        return;
      }
      s.uploading = true;
      s.uploadError = "";
      try {
        const col = s.uploadCollection.trim();
        const args = { file: files[0] };
        if (col)
          args.collection_id = col;
        await s.uploadArtifact(args);
        s.uploadOpen = false;
        s.uploadFiles = [];
        s.uploadCollection = "";
        await s.refresh();
      } catch (e) {
        s.uploadError = e.message;
      } finally {
        s.uploading = false;
      }
    },
    onDragOver: (s, e) => {
      const dt = e.dataTransfer;
      if (!dt || !Array.from(dt.types || []).includes("Files"))
        return;
      e.preventDefault();
      dt.dropEffect = "copy";
      s.dragActive = true;
    },
    onDragLeave: (s, e) => {
      const t = e.currentTarget;
      const related = e.relatedTarget;
      if (t && related && t.contains(related))
        return;
      s.dragActive = false;
    },
    onDrop: async (s, e) => {
      e.preventDefault();
      s.dragActive = false;
      const files = e.dataTransfer?.files;
      if (!files || files.length === 0)
        return;
      const first = files[0];
      if (!first)
        return;
      await s.uploadDirect(first);
    }
  }
});

// views/artifacts/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
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
  const __wireBoundary_b12 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope27 = Object.create($scope);
      __childScope27.n = n;
      __childScope27.__i = __i;
      return (($scope2) => {
        const __frag28 = document.createDocumentFragment();
        const __c29 = [];
        const __e30 = document.createElement("li");
        __e30.setAttribute("class", "art-skel-row");
        __e30.setAttribute("aria-hidden", "true");
        const __e31 = document.createElement("sui-skeleton");
        __e31.setAttribute("shape", "block");
        __e31.setAttribute("size", "sm");
        __e31.setAttribute("class", "art-skel-icon");
        __e30.appendChild(__e31);
        const __e32 = document.createElement("div");
        __e32.setAttribute("class", "art-skel-text");
        const __e33 = document.createElement("sui-skeleton");
        __e33.setAttribute("shape", "line");
        __e33.setAttribute("size", "md");
        __e33.setAttribute("style", "width:40%");
        __e32.appendChild(__e33);
        const __e34 = document.createElement("sui-skeleton");
        __e34.setAttribute("shape", "line");
        __e34.setAttribute("size", "sm");
        __e34.setAttribute("style", "width:60%");
        __e32.appendChild(__e34);
        __e30.appendChild(__e32);
        __frag28.appendChild(__e30);
        return { nodes: Array.from(__frag28.childNodes), cleanup: () => {
          for (const c of __c29)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope27);
    }
  });
  const __wireBoundary_b20 = (anchor) => bindFor({
    anchor,
    items: () => $scope.collectionOptions(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.id)(__child);
    },
    render: (c, __i) => {
      const __childScope63 = Object.create($scope);
      __childScope63.c = c;
      __childScope63.__i = __i;
      return (($scope2) => {
        const __frag64 = document.createDocumentFragment();
        const __c65 = [];
        const __e66 = document.createElement("option");
        {
          const __av67 = $scope2.c.id;
          if (__av67 == null || __av67 === false || __av67 === "")
            __e66.removeAttribute("value");
          else
            __e66.setAttribute("value", __av67 === true ? "" : String(__av67));
        }
        const __t68 = document.createTextNode("");
        __e66.appendChild(__t68);
        __t68.textContent = $scope2.c.name || $scope2.c.id;
        __frag64.appendChild(__e66);
        return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
          for (const c2 of __c65)
            try {
              c2();
            } catch (e) {}
        }, update: (__new69, __newI70) => {
          const __old = $scope2.c;
          $scope2.c = __new69;
          $scope2.__i = __newI70;
          if (__old.id !== $scope2.c.id) {
            {
              const __av67 = $scope2.c.id;
              if (__av67 == null || __av67 === false || __av67 === "")
                __e66.removeAttribute("value");
              else
                __e66.setAttribute("value", __av67 === true ? "" : String(__av67));
            }
          }
          if (__old.id !== $scope2.c.id || __old.name !== $scope2.c.name) {
            __t68.textContent = $scope2.c.name || $scope2.c.id;
          }
        } };
      })(__childScope63);
    }
  });
  const __wireBoundary_b25 = (anchor) => bindFor({
    anchor,
    items: () => $scope.actorOptions(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a)(__child);
    },
    render: (a, __i) => {
      const __childScope75 = Object.create($scope);
      __childScope75.a = a;
      __childScope75.__i = __i;
      return (($scope2) => {
        const __frag76 = document.createDocumentFragment();
        const __c77 = [];
        const __e78 = document.createElement("option");
        {
          const __av79 = $scope2.a;
          if (__av79 == null || __av79 === false || __av79 === "")
            __e78.removeAttribute("value");
          else
            __e78.setAttribute("value", __av79 === true ? "" : String(__av79));
        }
        const __t80 = document.createTextNode("");
        __e78.appendChild(__t80);
        __t80.textContent = $scope2.a;
        __frag76.appendChild(__e78);
        return { nodes: Array.from(__frag76.childNodes), cleanup: () => {
          for (const c of __c77)
            try {
              c();
            } catch (e) {}
        }, update: (__new81, __newI82) => {
          $scope2.a = __new81;
          $scope2.__i = __newI82;
          {
            const __av79 = $scope2.a;
            if (__av79 == null || __av79 === false || __av79 === "")
              __e78.removeAttribute("value");
            else
              __e78.setAttribute("value", __av79 === true ? "" : String(__av79));
          }
          __t80.textContent = $scope2.a;
        } };
      })(__childScope75);
    }
  });
  const __wireBoundary_b30 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.hasActiveFilters())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag93 = document.createDocumentFragment();
        const __c94 = [];
        const __e95 = document.createElement("sui-button");
        __e95.setAttribute("size", "sm");
        __e95.setAttribute("variant", "ghost");
        __c94.push(on(__e95, "click", function() {
          const __h = $scope.clearFilters;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e95.setAttribute("data-test", "clear-filters");
        const __t96 = document.createTextNode("Clear");
        __e95.appendChild(__t96);
        __frag93.appendChild(__e95);
        return { nodes: Array.from(__frag93.childNodes), cleanup: () => {
          for (const c of __c94)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b37 = (anchor) => bindFor({
    anchor,
    items: () => $scope.visibleItems(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope129 = Object.create($scope);
      __childScope129.a = a;
      __childScope129.__i = __i;
      return (($scope2) => {
        __childScope129.__v_id = signal2(0);
        __childScope129.__v_name = signal2(0);
        const __wireBoundary_b41 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.viewer_url)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag143 = document.createDocumentFragment();
              const __c144 = [];
              const __e145 = document.createElement("sui-chip");
              __e145.setAttribute("tone", "accent");
              __e145.setAttribute("variant", "soft");
              __e145.setAttribute("size", "sm");
              __e145.setAttribute("data-test", "viewer-badge");
              const __t146 = document.createTextNode(`
                    ↗ `);
              __e145.appendChild(__t146);
              const __t147 = document.createTextNode("");
              __e145.appendChild(__t147);
              __t147.textContent = $scope2.a.viewer_label;
              __frag143.appendChild(__e145);
              return { nodes: Array.from(__frag143.childNodes), cleanup: () => {
                for (const c of __c144)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag148 = document.createDocumentFragment();
              const __c149 = [];
              const __e150 = document.createElement("sui-chip");
              __e150.setAttribute("tone", "muted");
              __e150.setAttribute("variant", "soft");
              __e150.setAttribute("size", "sm");
              __e150.setAttribute("data-test", "viewer-badge");
              const __t151 = document.createTextNode(`
                    ⊘ `);
              __e150.appendChild(__t151);
              const __t152 = document.createTextNode("");
              __e150.appendChild(__t152);
              __t152.textContent = $scope2.a.viewer_label;
              __frag148.appendChild(__e150);
              return { nodes: Array.from(__frag148.childNodes), cleanup: () => {
                for (const c of __c149)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b44 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.owner_actor)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag155 = document.createDocumentFragment();
              const __c156 = [];
              const __e157 = document.createElement("span");
              __e157.setAttribute("class", "mono muted");
              const __t158 = document.createTextNode("");
              __e157.appendChild(__t158);
              __t158.textContent = $scope2.a.owner_actor;
              __frag155.appendChild(__e157);
              return { nodes: Array.from(__frag155.childNodes), cleanup: () => {
                for (const c of __c156)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag159 = document.createDocumentFragment();
              const __c160 = [];
              const __e161 = document.createElement("span");
              __e161.setAttribute("class", "muted");
              const __t162 = document.createTextNode("—");
              __e161.appendChild(__t162);
              __frag159.appendChild(__e161);
              return { nodes: Array.from(__frag159.childNodes), cleanup: () => {
                for (const c of __c160)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag130 = document.createDocumentFragment();
        const __c131 = [];
        const __e132 = document.createElement("tr");
        {
          const __av133 = $scope2.a.id;
          if (__av133 == null || __av133 === false || __av133 === "")
            __e132.removeAttribute("data-row-id");
          else
            __e132.setAttribute("data-row-id", __av133 === true ? "" : String(__av133));
        }
        __e132.setAttribute("data-test", "row");
        const __e134 = document.createElement("td");
        __e134.setAttribute("class", "art-cell-name");
        const __e135 = document.createElement("sui-icon");
        __e135.setAttribute("name", "file");
        __e135.setAttribute("size", "sm");
        __e135.setAttribute("tone", "muted");
        __e134.appendChild(__e135);
        const __e136 = document.createElement("span");
        __e136.setAttribute("class", "art-cell-name-text");
        const __t137 = document.createTextNode("");
        __e136.appendChild(__t137);
        __t137.textContent = $scope2.a.name || $scope2.a.id;
        __e134.appendChild(__e136);
        __e132.appendChild(__e134);
        const __e138 = document.createElement("td");
        const __e139 = document.createElement("span");
        __e139.setAttribute("class", "mono muted");
        const __t140 = document.createTextNode("");
        __e139.appendChild(__t140);
        __t140.textContent = $scope2.a.mime;
        __e138.appendChild(__e139);
        __e132.appendChild(__e138);
        const __e141 = document.createElement("td");
        const __ifAnchor142 = document.createComment("if");
        __e141.appendChild(__ifAnchor142);
        __c131.push(__wireBoundary_b41(__ifAnchor142));
        __e132.appendChild(__e141);
        const __e153 = document.createElement("td");
        const __ifAnchor154 = document.createComment("if");
        __e153.appendChild(__ifAnchor154);
        __c131.push(__wireBoundary_b44(__ifAnchor154));
        __e132.appendChild(__e153);
        const __e163 = document.createElement("td");
        __e163.setAttribute("class", "muted");
        const __t164 = document.createTextNode("");
        __e163.appendChild(__t164);
        __t164.textContent = $scope2.a.created_short;
        __e132.appendChild(__e163);
        const __e165 = document.createElement("td");
        const __t166 = document.createTextNode("");
        __e165.appendChild(__t166);
        __t166.textContent = $scope2.a.size_label;
        __e132.appendChild(__e165);
        const __e167 = document.createElement("td");
        __e167.setAttribute("class", "art-cell-action");
        const __e168 = document.createElement("a");
        __e168.setAttribute("class", "art-row-link");
        {
          const __av169 = "/api/v1/artifacts/" + $scope2.a.id;
          if (__av169 == null || __av169 === false || __av169 === "")
            __e168.removeAttribute("href");
          else
            __e168.setAttribute("href", __av169 === true ? "" : String(__av169));
        }
        __e168.setAttribute("data-no-router", "");
        __c131.push(on(__e168, "click", function() {
          const __h = (e) => e.stopPropagation();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c131.push(bindAttr(__e168, "title", () => (void $scope2.__v_name(), void $scope2.__v_id(), "Download " + ($scope2.a.name || $scope2.a.id))));
        __c131.push(bindAttr(__e168, "aria-label", () => (void $scope2.__v_name(), void $scope2.__v_id(), "Download " + ($scope2.a.name || $scope2.a.id))));
        const __e170 = document.createElement("sui-icon");
        __e170.setAttribute("name", "download");
        __e170.setAttribute("size", "sm");
        __e168.appendChild(__e170);
        __e167.appendChild(__e168);
        __e132.appendChild(__e167);
        __frag130.appendChild(__e132);
        return { nodes: Array.from(__frag130.childNodes), cleanup: () => {
          for (const c of __c131)
            try {
              c();
            } catch (e) {}
        }, update: (__new171, __newI172) => {
          const __old = $scope2.a;
          $scope2.a = __new171;
          $scope2.__i = __newI172;
          if (__old.id !== $scope2.a.id) {
            {
              const __av133 = $scope2.a.id;
              if (__av133 == null || __av133 === false || __av133 === "")
                __e132.removeAttribute("data-row-id");
              else
                __e132.setAttribute("data-row-id", __av133 === true ? "" : String(__av133));
            }
            {
              const __av169 = "/api/v1/artifacts/" + $scope2.a.id;
              if (__av169 == null || __av169 === false || __av169 === "")
                __e168.removeAttribute("href");
              else
                __e168.setAttribute("href", __av169 === true ? "" : String(__av169));
            }
          }
          if (__old.id !== $scope2.a.id || __old.name !== $scope2.a.name) {
            __t137.textContent = $scope2.a.name || $scope2.a.id;
          }
          if (__old.mime !== $scope2.a.mime) {
            __t140.textContent = $scope2.a.mime;
          }
          if (__old.created_short !== $scope2.a.created_short) {
            __t164.textContent = $scope2.a.created_short;
          }
          if (__old.size_label !== $scope2.a.size_label) {
            __t166.textContent = $scope2.a.size_label;
          }
          if (__old.id !== $scope2.a.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
          if (__old.name !== $scope2.a.name) {
            $scope2.__v_name.set((c) => c + 1);
          }
        } };
      })(__childScope129);
    }
  });
  const __wireBoundary_b32 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.visibleCount() === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag98 = document.createDocumentFragment();
        const __c99 = [];
        const __e100 = document.createElement("sui-empty-state");
        __e100.setAttribute("variant", "default");
        __e100.setAttribute("size", "sm");
        __e100.setAttribute("data-test", "empty-filtered");
        __e100.setAttribute("role", "status");
        __e100.setAttribute("aria-live", "polite");
        const __e101 = document.createElement("sui-icon");
        __e101.setAttribute("slot", "icon");
        __e101.setAttribute("name", "search");
        __e100.appendChild(__e101);
        const __e102 = document.createElement("span");
        __e102.setAttribute("slot", "title");
        const __t103 = document.createTextNode("No matches");
        __e102.appendChild(__t103);
        __e100.appendChild(__e102);
        const __e104 = document.createElement("span");
        __e104.setAttribute("slot", "description");
        const __t105 = document.createTextNode("");
        __e104.appendChild(__t105);
        __c99.push(bindText(__t105, () => $scope.totalCount()));
        const __t106 = document.createTextNode(" artifact");
        __e104.appendChild(__t106);
        const __t107 = document.createTextNode("");
        __e104.appendChild(__t107);
        __c99.push(bindText(__t107, () => $scope.totalCount() === 1 ? "" : "s"));
        const __t108 = document.createTextNode(` stored, but none match the current filters.
          `);
        __e104.appendChild(__t108);
        __e100.appendChild(__e104);
        const __e109 = document.createElement("sui-button");
        __e109.setAttribute("slot", "action");
        __e109.setAttribute("size", "sm");
        __e109.setAttribute("variant", "ghost");
        __c99.push(on(__e109, "click", function() {
          const __h = $scope.clearFilters;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t110 = document.createTextNode(`
            Clear filters
          `);
        __e109.appendChild(__t110);
        __e100.appendChild(__e109);
        __frag98.appendChild(__e100);
        return { nodes: Array.from(__frag98.childNodes), cleanup: () => {
          for (const c of __c99)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag111 = document.createDocumentFragment();
        const __c112 = [];
        const __e113 = document.createElement("sui-table");
        __e113.setAttribute("variant", "slot");
        __e113.setAttribute("label", "Artifacts");
        __e113.setAttribute("data-test", "table");
        __c112.push(on(__e113, "sui-row-click", function() {
          const __h = (e) => $scope.activateRow(e.detail.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e114 = document.createElement("tr");
        __e114.setAttribute("slot", "head");
        const __e115 = document.createElement("th");
        const __t116 = document.createTextNode("Name");
        __e115.appendChild(__t116);
        __e114.appendChild(__e115);
        const __e117 = document.createElement("th");
        const __t118 = document.createTextNode("Type");
        __e117.appendChild(__t118);
        __e114.appendChild(__e117);
        const __e119 = document.createElement("th");
        const __t120 = document.createTextNode("Viewer");
        __e119.appendChild(__t120);
        __e114.appendChild(__e119);
        const __e121 = document.createElement("th");
        const __t122 = document.createTextNode("Actor");
        __e121.appendChild(__t122);
        __e114.appendChild(__e121);
        const __e123 = document.createElement("th");
        const __t124 = document.createTextNode("Age");
        __e123.appendChild(__t124);
        __e114.appendChild(__e123);
        const __e125 = document.createElement("th");
        const __t126 = document.createTextNode("Size");
        __e125.appendChild(__t126);
        __e114.appendChild(__e125);
        const __e127 = document.createElement("th");
        __e127.setAttribute("class", "art-th-action");
        __e127.setAttribute("aria-label", "Actions");
        __e114.appendChild(__e127);
        __e113.appendChild(__e114);
        const __forAnchor128 = document.createComment("for");
        __e113.appendChild(__forAnchor128);
        __c112.push(__wireBoundary_b37(__forAnchor128));
        __frag111.appendChild(__e113);
        return { nodes: Array.from(__frag111.childNodes), cleanup: () => {
          for (const c of __c112)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.loadError())
        return 0;
      if ($scope.loading())
        return 1;
      if ($scope.totalCount() === 0)
        return 2;
      return 3;
    },
    branches: [
      () => {
        const __frag17 = document.createDocumentFragment();
        const __c18 = [];
        const __e19 = document.createElement("sui-error-state");
        __e19.setAttribute("title", "Couldn't load artifacts");
        __e19.setAttribute("data-test", "error");
        __e19.setAttribute("role", "alert");
        __e19.setAttribute("aria-live", "assertive");
        const __t20 = document.createTextNode("");
        __e19.appendChild(__t20);
        __c18.push(bindText(__t20, () => $scope.loadError()));
        const __e21 = document.createElement("sui-button");
        __e21.setAttribute("slot", "action");
        __e21.setAttribute("tone", "primary");
        __e21.setAttribute("variant", "solid");
        __c18.push(on(__e21, "click", function() {
          const __h = () => $scope.refresh();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t22 = document.createTextNode("Retry");
        __e21.appendChild(__t22);
        __e19.appendChild(__e21);
        __frag17.appendChild(__e19);
        return { nodes: Array.from(__frag17.childNodes), cleanup: () => {
          for (const c of __c18)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag23 = document.createDocumentFragment();
        const __c24 = [];
        const __e25 = document.createElement("ul");
        __e25.setAttribute("class", "art-list");
        __e25.setAttribute("data-test", "loading");
        __e25.setAttribute("role", "status");
        __e25.setAttribute("aria-live", "polite");
        __e25.setAttribute("aria-busy", "true");
        __e25.setAttribute("aria-label", "Loading artifacts");
        const __forAnchor26 = document.createComment("for");
        __e25.appendChild(__forAnchor26);
        __c24.push(__wireBoundary_b12(__forAnchor26));
        __frag23.appendChild(__e25);
        return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
          for (const c of __c24)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag35 = document.createDocumentFragment();
        const __c36 = [];
        const __e37 = document.createElement("sui-empty-state");
        __e37.setAttribute("variant", "default");
        __e37.setAttribute("size", "sm");
        __e37.setAttribute("data-test", "empty");
        __e37.setAttribute("role", "status");
        const __e38 = document.createElement("sui-icon");
        __e38.setAttribute("slot", "icon");
        __e38.setAttribute("name", "file");
        __e37.appendChild(__e38);
        const __e39 = document.createElement("span");
        __e39.setAttribute("slot", "title");
        const __t40 = document.createTextNode("No artifacts yet");
        __e39.appendChild(__t40);
        __e37.appendChild(__e39);
        const __e41 = document.createElement("span");
        __e41.setAttribute("slot", "description");
        const __t42 = document.createTextNode(`
          Run a workflow, or drop a file onto this page to upload one.
        `);
        __e41.appendChild(__t42);
        __e37.appendChild(__e41);
        const __e43 = document.createElement("sui-button");
        __e43.setAttribute("slot", "action");
        __e43.setAttribute("tone", "primary");
        __e43.setAttribute("variant", "solid");
        __c36.push(on(__e43, "click", function() {
          const __h = $scope.openUpload;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e44 = document.createElement("sui-icon");
        __e44.setAttribute("slot", "leading");
        __e44.setAttribute("name", "upload");
        __e43.appendChild(__e44);
        const __t45 = document.createTextNode(`Upload an artifact
        `);
        __e43.appendChild(__t45);
        __e37.appendChild(__e43);
        __frag35.appendChild(__e37);
        return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
          for (const c of __c36)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag46 = document.createDocumentFragment();
        const __c47 = [];
        const __e48 = document.createElement("div");
        __e48.setAttribute("class", "art-filters");
        __e48.setAttribute("data-test", "filters");
        const __e49 = document.createElement("sui-input");
        __e49.setAttribute("type", "search");
        __e49.setAttribute("size", "sm");
        __e49.setAttribute("placeholder", "Search by name, id, actor…");
        __c47.push(bindAttr(__e49, "value", () => $scope.searchQuery()));
        __c47.push(on(__e49, "sui-input", function() {
          const __h = (e) => $scope.setSearchQuery(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e49.setAttribute("class", "art-filter-search");
        __e49.setAttribute("aria-label", "Search artifacts");
        __e49.setAttribute("data-test", "filter-search");
        __e48.appendChild(__e49);
        const __e50 = document.createElement("sui-select");
        __e50.setAttribute("size", "sm");
        __c47.push(bindAttr(__e50, "value", () => $scope.mimeFilter()));
        __c47.push(on(__e50, "sui-change", function() {
          const __h = (e) => $scope.setMimeFilter(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e50.setAttribute("aria-label", "Filter by type");
        __e50.setAttribute("data-test", "filter-mime");
        const __e51 = document.createElement("option");
        __e51.setAttribute("value", "");
        const __t52 = document.createTextNode("All types");
        __e51.appendChild(__t52);
        __e50.appendChild(__e51);
        const __e53 = document.createElement("option");
        __e53.setAttribute("value", "image");
        const __t54 = document.createTextNode("Images");
        __e53.appendChild(__t54);
        __e50.appendChild(__e53);
        const __e55 = document.createElement("option");
        __e55.setAttribute("value", "text");
        const __t56 = document.createTextNode("Text");
        __e55.appendChild(__t56);
        __e50.appendChild(__e55);
        const __e57 = document.createElement("option");
        __e57.setAttribute("value", "json");
        const __t58 = document.createTextNode("JSON");
        __e57.appendChild(__t58);
        __e50.appendChild(__e57);
        __e48.appendChild(__e50);
        const __e59 = document.createElement("sui-select");
        __e59.setAttribute("size", "sm");
        __c47.push(bindAttr(__e59, "value", () => $scope.collectionFilter()));
        __c47.push(on(__e59, "sui-change", function() {
          const __h = (e) => $scope.setCollectionFilter(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e59.setAttribute("aria-label", "Filter by collection");
        __e59.setAttribute("data-test", "filter-collection");
        const __e60 = document.createElement("option");
        __e60.setAttribute("value", "");
        const __t61 = document.createTextNode("All collections");
        __e60.appendChild(__t61);
        __e59.appendChild(__e60);
        const __forAnchor62 = document.createComment("for");
        __e59.appendChild(__forAnchor62);
        __c47.push(__wireBoundary_b20(__forAnchor62));
        __e48.appendChild(__e59);
        const __e71 = document.createElement("sui-select");
        __e71.setAttribute("size", "sm");
        __c47.push(bindAttr(__e71, "value", () => $scope.actorFilter()));
        __c47.push(on(__e71, "sui-change", function() {
          const __h = (e) => $scope.setActorFilter(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e71.setAttribute("aria-label", "Filter by actor");
        __e71.setAttribute("data-test", "filter-actor");
        const __e72 = document.createElement("option");
        __e72.setAttribute("value", "");
        const __t73 = document.createTextNode("All actors");
        __e72.appendChild(__t73);
        __e71.appendChild(__e72);
        const __forAnchor74 = document.createComment("for");
        __e71.appendChild(__forAnchor74);
        __c47.push(__wireBoundary_b25(__forAnchor74));
        __e48.appendChild(__e71);
        const __e83 = document.createElement("sui-select");
        __e83.setAttribute("size", "sm");
        __c47.push(bindAttr(__e83, "value", () => $scope.ageFilter()));
        __c47.push(on(__e83, "sui-change", function() {
          const __h = (e) => $scope.setAgeFilter(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e83.setAttribute("aria-label", "Filter by age");
        __e83.setAttribute("data-test", "filter-age");
        const __e84 = document.createElement("option");
        __e84.setAttribute("value", "");
        const __t85 = document.createTextNode("Any age");
        __e84.appendChild(__t85);
        __e83.appendChild(__e84);
        const __e86 = document.createElement("option");
        __e86.setAttribute("value", "24h");
        const __t87 = document.createTextNode("Last 24h");
        __e86.appendChild(__t87);
        __e83.appendChild(__e86);
        const __e88 = document.createElement("option");
        __e88.setAttribute("value", "7d");
        const __t89 = document.createTextNode("Last 7 days");
        __e88.appendChild(__t89);
        __e83.appendChild(__e88);
        const __e90 = document.createElement("option");
        __e90.setAttribute("value", "30d");
        const __t91 = document.createTextNode("Last 30 days");
        __e90.appendChild(__t91);
        __e83.appendChild(__e90);
        __e48.appendChild(__e83);
        const __ifAnchor92 = document.createComment("if");
        __e48.appendChild(__ifAnchor92);
        __c47.push(__wireBoundary_b30(__ifAnchor92));
        __frag46.appendChild(__e48);
        const __ifAnchor97 = document.createComment("if");
        __frag46.appendChild(__ifAnchor97);
        __c47.push(__wireBoundary_b32(__ifAnchor97));
        return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
          for (const c of __c47)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b52 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.dragActive())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag174 = document.createDocumentFragment();
        const __c175 = [];
        const __e176 = document.createElement("div");
        __e176.setAttribute("class", "art-drop-overlay");
        __e176.setAttribute("data-test", "drop-overlay");
        __e176.setAttribute("aria-hidden", "true");
        const __e177 = document.createElement("sui-icon");
        __e177.setAttribute("name", "upload");
        __e177.setAttribute("size", "lg");
        __e176.appendChild(__e177);
        const __e178 = document.createElement("span");
        const __t179 = document.createTextNode("Drop to upload");
        __e178.appendChild(__t179);
        __e176.appendChild(__e178);
        __frag174.appendChild(__e176);
        return { nodes: Array.from(__frag174.childNodes), cleanup: () => {
          for (const c of __c175)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b61 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.detailItem().collection_id)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag204 = document.createDocumentFragment();
        const __c205 = [];
        const __e206 = document.createElement("div");
        __e206.setAttribute("class", "art-detail-row");
        const __e207 = document.createElement("dt");
        const __t208 = document.createTextNode("Collection");
        __e207.appendChild(__t208);
        __e206.appendChild(__e207);
        const __e209 = document.createElement("dd");
        const __e210 = document.createElement("sui-chip");
        __e210.setAttribute("tone", "muted");
        __e210.setAttribute("variant", "soft");
        __e210.setAttribute("size", "sm");
        const __t211 = document.createTextNode("");
        __e210.appendChild(__t211);
        __c205.push(bindText(__t211, () => $scope.detailItem().collection_id));
        __e209.appendChild(__e210);
        __e206.appendChild(__e209);
        __frag204.appendChild(__e206);
        return { nodes: Array.from(__frag204.childNodes), cleanup: () => {
          for (const c of __c205)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b63 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.detailItem().scope)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag213 = document.createDocumentFragment();
        const __c214 = [];
        const __e215 = document.createElement("div");
        __e215.setAttribute("class", "art-detail-row");
        const __e216 = document.createElement("dt");
        const __t217 = document.createTextNode("Scope");
        __e216.appendChild(__t217);
        __e215.appendChild(__e216);
        const __e218 = document.createElement("dd");
        const __e219 = document.createElement("span");
        __e219.setAttribute("class", "mono-snip");
        const __t220 = document.createTextNode("");
        __e219.appendChild(__t220);
        __c214.push(bindText(__t220, () => $scope.detailItem().scope));
        __e218.appendChild(__e219);
        __e215.appendChild(__e218);
        __frag213.appendChild(__e215);
        return { nodes: Array.from(__frag213.childNodes), cleanup: () => {
          for (const c of __c214)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b69 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.previewLoading())
        return 0;
      if ($scope.previewError())
        return 1;
      return 2;
    },
    branches: [
      () => {
        const __frag232 = document.createDocumentFragment();
        const __c233 = [];
        const __e234 = document.createElement("sui-skeleton");
        __e234.setAttribute("shape", "block");
        __e234.setAttribute("size", "lg");
        __frag232.appendChild(__e234);
        return { nodes: Array.from(__frag232.childNodes), cleanup: () => {
          for (const c of __c233)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag235 = document.createDocumentFragment();
        const __c236 = [];
        const __e237 = document.createElement("div");
        __e237.setAttribute("class", "art-detail-error");
        const __t238 = document.createTextNode("");
        __e237.appendChild(__t238);
        __c236.push(bindText(__t238, () => $scope.previewError()));
        __frag235.appendChild(__e237);
        return { nodes: Array.from(__frag235.childNodes), cleanup: () => {
          for (const c of __c236)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag239 = document.createDocumentFragment();
        const __c240 = [];
        const __e241 = document.createElement("pre");
        __e241.setAttribute("class", "art-detail-pre");
        const __e242 = document.createElement("code");
        const __t243 = document.createTextNode("");
        __e242.appendChild(__t243);
        __c240.push(bindText(__t243, () => $scope.previewText()));
        __e241.appendChild(__e242);
        __frag239.appendChild(__e241);
        return { nodes: Array.from(__frag239.childNodes), cleanup: () => {
          for (const c of __c240)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b65 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.previewKind() === "image")
        return 0;
      if ($scope.previewKind() === "html")
        return 1;
      if ($scope.previewKind() === "text")
        return 2;
      return 3;
    },
    branches: [
      () => {
        const __frag223 = document.createDocumentFragment();
        const __c224 = [];
        const __e225 = document.createElement("img");
        __e225.setAttribute("class", "art-detail-img");
        __c224.push(bindAttr(__e225, "src", () => $scope.detailUrl()));
        __c224.push(bindAttr(__e225, "alt", () => $scope.detailItem().name || $scope.detailItem().id));
        __frag223.appendChild(__e225);
        return { nodes: Array.from(__frag223.childNodes), cleanup: () => {
          for (const c of __c224)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag226 = document.createDocumentFragment();
        const __c227 = [];
        const __e228 = document.createElement("iframe");
        __e228.setAttribute("class", "art-detail-frame");
        __c227.push(bindAttr(__e228, "src", () => $scope.detailUrl()));
        __e228.setAttribute("title", "Artifact preview");
        __e228.setAttribute("sandbox", "");
        __frag226.appendChild(__e228);
        return { nodes: Array.from(__frag226.childNodes), cleanup: () => {
          for (const c of __c227)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag229 = document.createDocumentFragment();
        const __c230 = [];
        const __ifAnchor231 = document.createComment("if");
        __frag229.appendChild(__ifAnchor231);
        __c230.push(__wireBoundary_b69(__ifAnchor231));
        return { nodes: Array.from(__frag229.childNodes), cleanup: () => {
          for (const c of __c230)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag244 = document.createDocumentFragment();
        const __c245 = [];
        const __e246 = document.createElement("div");
        __e246.setAttribute("class", "art-detail-binary");
        const __e247 = document.createElement("sui-icon");
        __e247.setAttribute("name", "file");
        __e247.setAttribute("size", "lg");
        __e247.setAttribute("tone", "muted");
        __e246.appendChild(__e247);
        const __e248 = document.createElement("p");
        const __t249 = document.createTextNode("Binary file — download to view.");
        __e248.appendChild(__t249);
        __e246.appendChild(__e248);
        __frag244.appendChild(__e246);
        return { nodes: Array.from(__frag244.childNodes), cleanup: () => {
          for (const c of __c245)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b55 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.detailItem())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag182 = document.createDocumentFragment();
        const __c183 = [];
        const __e184 = document.createElement("span");
        __e184.setAttribute("slot", "title");
        const __t185 = document.createTextNode("");
        __e184.appendChild(__t185);
        __c183.push(bindText(__t185, () => $scope.detailItem().name || $scope.detailItem().id));
        __frag182.appendChild(__e184);
        const __e186 = document.createElement("span");
        __e186.setAttribute("slot", "description");
        const __t187 = document.createTextNode("");
        __e186.appendChild(__t187);
        __c183.push(bindText(__t187, () => $scope.detailItem().mime));
        const __t188 = document.createTextNode(" · ");
        __e186.appendChild(__t188);
        const __t189 = document.createTextNode("");
        __e186.appendChild(__t189);
        __c183.push(bindText(__t189, () => $scope.detailItem().size_label));
        __frag182.appendChild(__e186);
        const __e190 = document.createElement("sui-stack");
        __e190.setAttribute("gap", "4");
        const __e191 = document.createElement("dl");
        __e191.setAttribute("class", "art-detail-meta");
        const __e192 = document.createElement("div");
        __e192.setAttribute("class", "art-detail-row");
        const __e193 = document.createElement("dt");
        const __t194 = document.createTextNode("ID");
        __e193.appendChild(__t194);
        __e192.appendChild(__e193);
        const __e195 = document.createElement("dd");
        const __e196 = document.createElement("span");
        __e196.setAttribute("class", "mono");
        const __t197 = document.createTextNode("");
        __e196.appendChild(__t197);
        __c183.push(bindText(__t197, () => $scope.detailItem().id));
        __e195.appendChild(__e196);
        __e192.appendChild(__e195);
        __e191.appendChild(__e192);
        const __e198 = document.createElement("div");
        __e198.setAttribute("class", "art-detail-row");
        const __e199 = document.createElement("dt");
        const __t200 = document.createTextNode("Created");
        __e199.appendChild(__t200);
        __e198.appendChild(__e199);
        const __e201 = document.createElement("dd");
        __e201.setAttribute("class", "muted");
        const __t202 = document.createTextNode("");
        __e201.appendChild(__t202);
        __c183.push(bindText(__t202, () => $scope.detailItem().created_at));
        __e198.appendChild(__e201);
        __e191.appendChild(__e198);
        const __ifAnchor203 = document.createComment("if");
        __e191.appendChild(__ifAnchor203);
        __c183.push(__wireBoundary_b61(__ifAnchor203));
        const __ifAnchor212 = document.createComment("if");
        __e191.appendChild(__ifAnchor212);
        __c183.push(__wireBoundary_b63(__ifAnchor212));
        __e190.appendChild(__e191);
        const __e221 = document.createElement("div");
        __e221.setAttribute("class", "art-detail-preview");
        __e221.setAttribute("data-test", "detail-preview");
        const __ifAnchor222 = document.createComment("if");
        __e221.appendChild(__ifAnchor222);
        __c183.push(__wireBoundary_b65(__ifAnchor222));
        __e190.appendChild(__e221);
        __frag182.appendChild(__e190);
        const __e250 = document.createElement("div");
        __e250.setAttribute("slot", "footer");
        const __e251 = document.createElement("sui-button");
        __e251.setAttribute("variant", "ghost");
        __c183.push(on(__e251, "click", function() {
          const __h = $scope.closeDetail;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t252 = document.createTextNode("Close");
        __e251.appendChild(__t252);
        __e250.appendChild(__e251);
        const __e253 = document.createElement("a");
        __e253.setAttribute("class", "art-detail-download");
        __c183.push(bindAttr(__e253, "href", () => $scope.detailUrl()));
        __e253.setAttribute("data-no-router", "");
        __c183.push(bindAttr(__e253, "download", () => $scope.detailItem().name || $scope.detailItem().id));
        const __e254 = document.createElement("sui-button");
        __e254.setAttribute("tone", "primary");
        __e254.setAttribute("variant", "solid");
        const __e255 = document.createElement("sui-icon");
        __e255.setAttribute("slot", "leading");
        __e255.setAttribute("name", "download");
        __e254.appendChild(__e255);
        const __t256 = document.createTextNode(`Download
            `);
        __e254.appendChild(__t256);
        __e253.appendChild(__e254);
        __e250.appendChild(__e253);
        __frag182.appendChild(__e250);
        return { nodes: Array.from(__frag182.childNodes), cleanup: () => {
          for (const c of __c183)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b82 = (anchor) => bindFor({
    anchor,
    items: () => $scope.collectionOptions(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.id)(__child);
    },
    render: (c, __i) => {
      const __childScope271 = Object.create($scope);
      __childScope271.c = c;
      __childScope271.__i = __i;
      return (($scope2) => {
        const __frag272 = document.createDocumentFragment();
        const __c273 = [];
        const __e274 = document.createElement("option");
        {
          const __av275 = $scope2.c.id;
          if (__av275 == null || __av275 === false || __av275 === "")
            __e274.removeAttribute("value");
          else
            __e274.setAttribute("value", __av275 === true ? "" : String(__av275));
        }
        const __t276 = document.createTextNode("");
        __e274.appendChild(__t276);
        __t276.textContent = $scope2.c.name || $scope2.c.id;
        __frag272.appendChild(__e274);
        return { nodes: Array.from(__frag272.childNodes), cleanup: () => {
          for (const c2 of __c273)
            try {
              c2();
            } catch (e) {}
        }, update: (__new277, __newI278) => {
          const __old = $scope2.c;
          $scope2.c = __new277;
          $scope2.__i = __newI278;
          if (__old.id !== $scope2.c.id) {
            {
              const __av275 = $scope2.c.id;
              if (__av275 == null || __av275 === false || __av275 === "")
                __e274.removeAttribute("value");
              else
                __e274.setAttribute("value", __av275 === true ? "" : String(__av275));
            }
          }
          if (__old.id !== $scope2.c.id || __old.name !== $scope2.c.name) {
            __t276.textContent = $scope2.c.name || $scope2.c.id;
          }
        } };
      })(__childScope271);
    }
  });
  const __wireBoundary_b85 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.uploadError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag280 = document.createDocumentFragment();
        const __c281 = [];
        const __e282 = document.createElement("div");
        __e282.setAttribute("class", "art-upload-error");
        __e282.setAttribute("data-test", "upload-error");
        const __t283 = document.createTextNode("");
        __e282.appendChild(__t283);
        __c281.push(bindText(__t283, () => $scope.uploadError()));
        __frag280.appendChild(__e282);
        return { nodes: Array.from(__frag280.childNodes), cleanup: () => {
          for (const c of __c281)
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
      __e0.setAttribute("class", "ship-view ship-view-artifacts");
      __cleanups.push(on(__e0, "dragenter", function() {
        const __h = $scope.onDragOver;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(on(__e0, "dragover", function() {
        const __h = $scope.onDragOver;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(on(__e0, "dragleave", function() {
        const __h = $scope.onDragLeave;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(on(__e0, "drop", function() {
        const __h = $scope.onDrop;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "art-wrap");
      __cleanups.push(bindAttr(__e1, "data-drag-active", () => $scope.dragActive() ? "" : null));
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "art-head");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "art-head-titles");
      const __e4 = document.createElement("h1");
      __e4.setAttribute("class", "art-title");
      const __t5 = document.createTextNode("Artifacts");
      __e4.appendChild(__t5);
      __e3.appendChild(__e4);
      const __e6 = document.createElement("p");
      __e6.setAttribute("class", "art-sub");
      const __t7 = document.createTextNode("");
      __e6.appendChild(__t7);
      __cleanups.push(bindText(__t7, () => $scope.visibleCount()));
      const __t8 = document.createTextNode(" of ");
      __e6.appendChild(__t8);
      const __t9 = document.createTextNode("");
      __e6.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.totalCount()));
      const __t10 = document.createTextNode(" artifact");
      __e6.appendChild(__t10);
      const __t11 = document.createTextNode("");
      __e6.appendChild(__t11);
      __cleanups.push(bindText(__t11, () => $scope.totalCount() === 1 ? "" : "s"));
      __e3.appendChild(__e6);
      __e2.appendChild(__e3);
      const __e12 = document.createElement("sui-cluster");
      __e12.setAttribute("gap", "2");
      __e12.setAttribute("align", "center");
      const __e13 = document.createElement("sui-button");
      __e13.setAttribute("tone", "primary");
      __e13.setAttribute("variant", "solid");
      __e13.setAttribute("size", "sm");
      __cleanups.push(on(__e13, "click", function() {
        const __h = $scope.openUpload;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e13.setAttribute("data-test", "upload-button");
      const __e14 = document.createElement("sui-icon");
      __e14.setAttribute("slot", "leading");
      __e14.setAttribute("name", "upload");
      __e13.appendChild(__e14);
      const __t15 = document.createTextNode(`Upload
        `);
      __e13.appendChild(__t15);
      __e12.appendChild(__e13);
      __e2.appendChild(__e12);
      __e1.appendChild(__e2);
      const __ifAnchor16 = document.createComment("if");
      __e1.appendChild(__ifAnchor16);
      __cleanups.push(__wireBoundary_b9(__ifAnchor16));
      const __ifAnchor173 = document.createComment("if");
      __e1.appendChild(__ifAnchor173);
      __cleanups.push(__wireBoundary_b52(__ifAnchor173));
      const __e180 = document.createElement("sui-drawer");
      __cleanups.push(bindAttr(__e180, "open", () => $scope.detailOpen() ? "" : null));
      __e180.setAttribute("size", "lg");
      __cleanups.push(on(__e180, "sui-close", function() {
        const __h = $scope.closeDetail;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e180.setAttribute("data-test", "detail-drawer");
      const __ifAnchor181 = document.createComment("if");
      __e180.appendChild(__ifAnchor181);
      __cleanups.push(__wireBoundary_b55(__ifAnchor181));
      __e1.appendChild(__e180);
      const __e257 = document.createElement("sui-dialog");
      __cleanups.push(bindAttr(__e257, "open", () => $scope.uploadOpen() ? "" : null));
      __e257.setAttribute("size", "md");
      __cleanups.push(on(__e257, "sui-close", function() {
        const __h = $scope.closeUpload;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e257.setAttribute("data-test", "upload-dialog");
      const __e258 = document.createElement("span");
      __e258.setAttribute("slot", "header");
      const __t259 = document.createTextNode("Upload artifact");
      __e258.appendChild(__t259);
      __e257.appendChild(__e258);
      const __e260 = document.createElement("sui-stack");
      __e260.setAttribute("gap", "3");
      const __e261 = document.createElement("sui-file-upload");
      __e261.setAttribute("accept", "");
      __cleanups.push(bindAttr(__e261, "disabled", () => $scope.uploading() ? "" : null));
      __cleanups.push(on(__e261, "sui-change", function() {
        const __h = (e) => $scope.onUploadFiles(e.detail.files);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e261.setAttribute("data-test", "upload-files");
      __e260.appendChild(__e261);
      const __e262 = document.createElement("sui-stack");
      __e262.setAttribute("gap", "1");
      const __e263 = document.createElement("label");
      __e263.setAttribute("class", "art-upload-label");
      __e263.setAttribute("for", "art-upload-col");
      const __t264 = document.createTextNode(`
            Collection `);
      __e263.appendChild(__t264);
      const __e265 = document.createElement("span");
      __e265.setAttribute("class", "art-upload-hint");
      const __t266 = document.createTextNode("(optional)");
      __e265.appendChild(__t266);
      __e263.appendChild(__e265);
      __e262.appendChild(__e263);
      const __e267 = document.createElement("sui-select");
      __e267.setAttribute("size", "sm");
      __cleanups.push(bindAttr(__e267, "value", () => $scope.uploadCollection()));
      __cleanups.push(on(__e267, "sui-change", function() {
        const __h = (e) => $scope.onUploadCollection(e.detail.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(bindAttr(__e267, "disabled", () => $scope.uploading() ? "" : null));
      __e267.setAttribute("data-test", "upload-collection");
      const __e268 = document.createElement("option");
      __e268.setAttribute("value", "");
      const __t269 = document.createTextNode("No collection");
      __e268.appendChild(__t269);
      __e267.appendChild(__e268);
      const __forAnchor270 = document.createComment("for");
      __e267.appendChild(__forAnchor270);
      __cleanups.push(__wireBoundary_b82(__forAnchor270));
      __e262.appendChild(__e267);
      __e260.appendChild(__e262);
      const __ifAnchor279 = document.createComment("if");
      __e260.appendChild(__ifAnchor279);
      __cleanups.push(__wireBoundary_b85(__ifAnchor279));
      __e257.appendChild(__e260);
      const __e284 = document.createElement("div");
      __e284.setAttribute("slot", "footer");
      const __e285 = document.createElement("sui-button");
      __e285.setAttribute("variant", "ghost");
      __cleanups.push(on(__e285, "click", function() {
        const __h = $scope.closeUpload;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(bindAttr(__e285, "disabled", () => $scope.uploading() ? "" : null));
      const __t286 = document.createTextNode("Cancel");
      __e285.appendChild(__t286);
      __e284.appendChild(__e285);
      const __e287 = document.createElement("sui-button");
      __e287.setAttribute("tone", "primary");
      __e287.setAttribute("variant", "solid");
      __cleanups.push(on(__e287, "click", function() {
        const __h = $scope.submitUpload;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(bindAttr(__e287, "disabled", () => $scope.uploadFiles().length === 0 || $scope.uploading() ? "" : null));
      __cleanups.push(bindAttr(__e287, "loading", () => $scope.uploading() ? "" : null));
      __e287.setAttribute("data-test", "upload-submit");
      const __t288 = document.createTextNode("Upload");
      __e287.appendChild(__t288);
      __e284.appendChild(__e287);
      __e257.appendChild(__e284);
      __e1.appendChild(__e257);
      __e0.appendChild(__e1);
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
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateOn(__m["b0"], "dragenter", function() {
          const __h = $scope.onDragOver;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b1"])
        __cleanups.push(hydrateOn(__m["b1"], "dragover", function() {
          const __h = $scope.onDragOver;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b2"])
        __cleanups.push(hydrateOn(__m["b2"], "dragleave", function() {
          const __h = $scope.onDragLeave;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b3"])
        __cleanups.push(hydrateOn(__m["b3"], "drop", function() {
          const __h = $scope.onDrop;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b4"])
        __cleanups.push(hydrateAttr(__m["b4"], "data-drag-active", () => $scope.dragActive() ? "" : null));
      if (__t["b5"])
        __cleanups.push(hydrateText(__t["b5"], () => $scope.visibleCount()));
      if (__t["b6"])
        __cleanups.push(hydrateText(__t["b6"], () => $scope.totalCount()));
      if (__t["b7"])
        __cleanups.push(hydrateText(__t["b7"], () => $scope.totalCount() === 1 ? "" : "s"));
      if (__m["b8"])
        __cleanups.push(hydrateOn(__m["b8"], "click", function() {
          const __h = $scope.openUpload;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b9"]) {
        const __t2 = __m["b9"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b9(__anchor));
      }
      if (__m["b52"]) {
        const __t2 = __m["b52"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b52(__anchor));
      }
      if (__m["b53"])
        __cleanups.push(hydrateAttr(__m["b53"], "open", () => $scope.detailOpen() ? "" : null));
      if (__m["b54"])
        __cleanups.push(hydrateOn(__m["b54"], "sui-close", function() {
          const __h = $scope.closeDetail;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b55"]) {
        const __t2 = __m["b55"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b55(__anchor));
      }
      if (__m["b75"])
        __cleanups.push(hydrateAttr(__m["b75"], "open", () => $scope.uploadOpen() ? "" : null));
      if (__m["b76"])
        __cleanups.push(hydrateOn(__m["b76"], "sui-close", function() {
          const __h = $scope.closeUpload;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b77"])
        __cleanups.push(hydrateAttr(__m["b77"], "disabled", () => $scope.uploading() ? "" : null));
      if (__m["b78"])
        __cleanups.push(hydrateOn(__m["b78"], "sui-change", function() {
          const __h = (e) => $scope.onUploadFiles(e.detail.files);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b79"])
        __cleanups.push(hydrateAttr(__m["b79"], "value", () => $scope.uploadCollection()));
      if (__m["b80"])
        __cleanups.push(hydrateOn(__m["b80"], "sui-change", function() {
          const __h = (e) => $scope.onUploadCollection(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b81"])
        __cleanups.push(hydrateAttr(__m["b81"], "disabled", () => $scope.uploading() ? "" : null));
      if (__m["b82"]) {
        const __t2 = __m["b82"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("for") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b82(__anchor));
      }
      if (__m["b85"]) {
        const __t2 = __m["b85"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b85(__anchor));
      }
      if (__m["b87"])
        __cleanups.push(hydrateOn(__m["b87"], "click", function() {
          const __h = $scope.closeUpload;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b88"])
        __cleanups.push(hydrateAttr(__m["b88"], "disabled", () => $scope.uploading() ? "" : null));
      if (__m["b89"])
        __cleanups.push(hydrateOn(__m["b89"], "click", function() {
          const __h = $scope.submitUpload;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b90"])
        __cleanups.push(hydrateAttr(__m["b90"], "disabled", () => $scope.uploadFiles().length === 0 || $scope.uploading() ? "" : null));
      if (__m["b91"])
        __cleanups.push(hydrateAttr(__m["b91"], "loading", () => $scope.uploading() ? "" : null));
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
  page_default as default
};
