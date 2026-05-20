// views/collections/page.sui
import { bindAttr, bindClass, bindFor, bindIf, bindText, bindValue, bindWhileError, bindWhileLoading, data, live as live2, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateAttr, hydrateBindValue, hydrateOn } from "@ship/sui-dom";

// views/collections/controller.ts
import { controller, signal, derived, live, unwrap } from "@ship/sui-runtime";
import { route as routeFactory } from "@ship/sui-runtime/route";
var EXAMPLES = [
  {
    id: "reading_list",
    name: "Reading list",
    icon: "book-open",
    description: "Track articles, papers, books to read.",
    fields: {
      title: { type: "string", required: true },
      url: { type: "string" },
      status: { type: "enum", values: ["todo", "in-progress", "done"] },
      notes: { type: "markdown" }
    }
  },
  {
    id: "onboarding_plan",
    name: "Onboarding plan",
    icon: "list-check",
    description: "Checklist for ramping into a new project.",
    fields: {
      step: { type: "string", required: true },
      owner: { type: "string" },
      status: { type: "enum", values: ["todo", "in-progress", "blocked", "done"] },
      due: { type: "date" }
    }
  },
  {
    id: "experiments",
    name: "Experiments",
    icon: "beaker",
    description: "Track hypotheses, outcomes, follow-ups.",
    fields: {
      hypothesis: { type: "string", required: true },
      status: { type: "enum", values: ["todo", "in-progress", "blocked", "done", "cancelled"] },
      result: { type: "markdown" }
    }
  }
];
function refIconFor(value) {
  if (value.startsWith("agent:"))
    return "cpu";
  if (value.startsWith("skill:"))
    return "sparkles";
  if (value.startsWith("pipeline:"))
    return "git-branch";
  if (value.startsWith("artifact:"))
    return "file";
  if (value.startsWith("job:"))
    return "play";
  if (value.startsWith("http"))
    return "external-link";
  return "link";
}
function refLabelFor(value) {
  return value.length > 64 ? value.slice(0, 64) + "…" : value;
}
function isRefValue(fieldName, value) {
  if (typeof value !== "string" || !value)
    return false;
  if (fieldName === "url" || fieldName === "href")
    return true;
  if (fieldName === "ref" || fieldName === "item" || fieldName === "target")
    return true;
  return /^(https?:|agent:|skill:|pipeline:|artifact:|job:)/.test(value);
}
function buildCell(field, item) {
  const raw = item.data?.[field.name];
  if (field.name === "status" && field.field_type === "enum") {
    const value = typeof raw === "string" ? raw : "";
    return {
      kind: "status",
      value: value || "—",
      class_name: "status-chip status-chip-" + (value || "unset"),
      raw: item
    };
  }
  if (isRefValue(field.name, raw)) {
    return { kind: "ref", value: raw, icon: refIconFor(raw), label: refLabelFor(raw) };
  }
  if (raw === null || raw === undefined)
    return { kind: "text", value: "" };
  if (typeof raw === "object")
    return { kind: "text", value: JSON.stringify(raw) };
  return { kind: "text", value: String(raw) };
}
function shortDate(iso) {
  if (!iso)
    return "";
  const then = Date.parse(iso);
  if (Number.isNaN(then))
    return iso.slice(0, 10);
  const delta = (Date.now() - then) / 1000;
  if (delta < 60)
    return "just now";
  if (delta < 3600)
    return `${Math.round(delta / 60)}m ago`;
  if (delta < 86400)
    return `${Math.round(delta / 3600)}h ago`;
  if (delta < 604800)
    return `${Math.round(delta / 86400)}d ago`;
  return new Date(then).toISOString().slice(0, 10);
}
var controller_default = controller({
  scope: ({ ship }) => {
    const route = routeFactory();
    const activeId = derived(() => route.query.collection() || "");
    const collections = live(() => ship.collections.list());
    const colDetail = live(() => ship.collections.detail({ id: activeId() }));
    const indexQuery = signal("");
    const statusFilter = signal("");
    const activeCollection = derived(() => {
      const v = collections();
      if (v.kind !== "ok" || !v.value)
        return null;
      return v.value.items.find((c) => c.id === activeId()) ?? null;
    });
    const hasStatusField = derived(() => {
      const detail = unwrap(colDetail);
      const f = detail?.schema?.fields?.find((x) => x.name === "status");
      return !!(f && f.field_type === "enum" && (f.values?.length ?? 0) > 0);
    });
    const statusValues = derived(() => {
      const detail = unwrap(colDetail);
      return detail?.schema?.fields?.find((x) => x.name === "status")?.values ?? [];
    });
    const filteredIndex = derived(() => {
      const v = collections();
      if (v.kind !== "ok" || !v.value)
        return [];
      const q = indexQuery().trim().toLowerCase();
      const rows = q ? v.value.items.filter((c) => (c.name ?? "").toLowerCase().includes(q) || c.id.toLowerCase().includes(q)) : v.value.items;
      return rows.map((c) => ({
        ...c,
        activity_label: c.last_activity ? shortDate(c.last_activity) : "no activity",
        status_summary: (c.status_values ?? []).map((value) => ({
          value,
          count: (c.status_counts ?? {})[value] ?? 0
        }))
      }));
    });
    const filteredRows = derived(() => {
      const detail = unwrap(colDetail);
      const items = detail?.items ?? [];
      const fields = detail?.schema?.fields ?? [];
      const filter = statusFilter();
      const filtered = filter ? items.filter((r) => r.data?.["status"] === filter) : items;
      return filtered.map((item) => ({
        id: item.id,
        raw: item,
        cells: fields.map((f) => buildCell(f, item))
      }));
    });
    const schemaFieldsView = derived(() => {
      const d = unwrap(colDetail);
      const fields = d?.schema?.fields ?? [];
      return fields.map((f) => ({ ...f, values_pretty: (f.values ?? []).join(" | ") }));
    });
    const examples = derived(() => EXAMPLES);
    const tab = signal("rows");
    const newRowOpen = signal(false);
    const editOpen = signal(false);
    const editId = signal("");
    const newColOpen = signal(false);
    const rowDraft = signal({});
    const editDraft = signal({});
    const newColDraftId = signal("");
    const newColDraftName = signal("");
    const drawerError = signal("");
    const createItem = (a) => ship.collections.create_item(a);
    const updateItem = (a) => ship.collections.update_item(a);
    const deleteItem = (a) => ship.collections.delete_item(a);
    const updateSchema = (a) => ship.collections.update_schema(a);
    const createCol = (a) => ship.collections.create(a);
    return {
      activeId,
      activeCollection,
      collections,
      colDetail,
      schemaFieldsView,
      tab,
      indexQuery,
      filteredIndex,
      statusFilter,
      statusValues,
      hasStatusField,
      filteredRows,
      examples,
      newRowOpen,
      editOpen,
      editId,
      newColOpen,
      rowDraft,
      editDraft,
      newColDraftId,
      newColDraftName,
      drawerError,
      createItem,
      updateItem,
      deleteItem,
      updateSchema,
      createCol
    };
  },
  on: {
    setDraftField: (s, which, name, value) => {
      const key = which === "row" ? "rowDraft" : "editDraft";
      s[key] = { ...s[key], [name]: value };
    },
    openEditRow: (s, item) => {
      s.editId = item.id;
      s.editDraft = { ...item.data ?? {} };
      s.drawerError = "";
      s.editOpen = true;
    },
    saveNewRow: async (s) => {
      const id = s.activeId;
      try {
        await s.createItem({ collection: id, data: s.rowDraft });
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : "Request failed";
        return;
      }
      s.newRowOpen = false;
      s.rowDraft = {};
      s.drawerError = "";
    },
    saveEdit: async (s) => {
      const colId = s.activeId;
      const rowId = s.editId;
      try {
        await s.updateItem({ collection: colId, item_id: rowId, data: s.editDraft });
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : "Request failed";
        return;
      }
      s.editOpen = false;
      s.editDraft = {};
      s.drawerError = "";
    },
    deleteRow: async (s) => {
      const colId = s.activeId;
      const rowId = s.editId;
      try {
        await s.deleteItem({
          collection: colId,
          item_id: rowId
        });
      } catch {}
      s.editOpen = false;
      s.editDraft = {};
      s.drawerError = "";
    },
    removeSchemaField: async (s, fieldName) => {
      const detail = unwrap(s.colDetail);
      const fields = detail?.schema?.fields ?? [];
      const fs = fields.filter((x) => x.name !== fieldName).map((x) => ({
        name: x.name,
        type: x.field_type,
        required: x.required,
        values: x.values ?? []
      }));
      const colId = s.activeId;
      try {
        await s.updateSchema({ id: colId, fields: fs });
      } catch {}
    },
    cycleStatus: async (s, item) => {
      const values = s.statusValues;
      if (!values || values.length === 0)
        return;
      const current = item.data?.["status"] ?? "";
      const idx = values.indexOf(current);
      const next = values[(idx + 1) % values.length];
      const colId = s.activeId;
      try {
        await s.updateItem({
          collection: colId,
          item_id: item.id,
          data: { ...item.data ?? {}, status: next }
        });
      } catch {}
    },
    createCollection: async (s) => {
      const id = s.newColDraftId.trim();
      if (!id) {
        s.drawerError = "Collection ID is required";
        return;
      }
      try {
        await s.createCol({
          id,
          name: s.newColDraftName.trim() || id,
          fields: {}
        });
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : "Request failed";
        return;
      }
      s.newColOpen = false;
      s.newColDraftId = "";
      s.newColDraftName = "";
      s.drawerError = "";
      s.navigate("/collections?collection=" + id);
    },
    createFromExample: async (s, exampleId) => {
      const ex = EXAMPLES.find((e) => e.id === exampleId);
      if (!ex)
        return;
      try {
        await s.createCol({
          id: ex.id,
          name: ex.name,
          fields: ex.fields
        });
      } catch (err) {
        s.drawerError = err instanceof Error ? err.message : "Request failed";
        return;
      }
      s.navigate("/collections?collection=" + ex.id);
    }
  }
});

// views/collections/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.collections = live2(() => ship.collections.list());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b3 = (anchor) => bindFor({
    anchor,
    items: () => $scope.examples(),
    key: (ex, __i) => {
      const __child = Object.create($scope);
      __child.ex = ex;
      __child.__i = __i;
      return (($scope2) => ex.id)(__child);
    },
    render: (ex, __i) => {
      const __childScope28 = Object.create($scope);
      __childScope28.ex = ex;
      __childScope28.__i = __i;
      return (($scope2) => {
        const __frag29 = document.createDocumentFragment();
        const __c30 = [];
        const __e31 = document.createElement("button");
        __e31.setAttribute("type", "button");
        __e31.setAttribute("class", "col-example");
        {
          const __av32 = "Create " + $scope2.ex.name + " collection";
          if (__av32 == null || __av32 === false || __av32 === "")
            __e31.removeAttribute("aria-label");
          else
            __e31.setAttribute("aria-label", __av32 === true ? "" : String(__av32));
        }
        __c30.push(on(__e31, "click", function() {
          const __h = () => $scope2.createFromExample($scope2.ex.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e33 = document.createElement("span");
        __e33.setAttribute("class", "col-example-icon");
        const __e34 = document.createElement("sui-icon");
        {
          const __av35 = $scope2.ex.icon;
          if (__av35 == null || __av35 === false || __av35 === "")
            __e34.removeAttribute("name");
          else
            __e34.setAttribute("name", __av35 === true ? "" : String(__av35));
        }
        __e33.appendChild(__e34);
        __e31.appendChild(__e33);
        const __e36 = document.createElement("span");
        __e36.setAttribute("class", "col-example-name");
        const __t37 = document.createTextNode("");
        __e36.appendChild(__t37);
        __t37.textContent = $scope2.ex.name;
        __e31.appendChild(__e36);
        const __e38 = document.createElement("span");
        __e38.setAttribute("class", "col-example-desc");
        const __t39 = document.createTextNode("");
        __e38.appendChild(__t39);
        __t39.textContent = $scope2.ex.description;
        __e31.appendChild(__e38);
        __frag29.appendChild(__e31);
        return { nodes: Array.from(__frag29.childNodes), cleanup: () => {
          for (const c of __c30)
            try {
              c();
            } catch (e) {}
        }, update: (__new40, __newI41) => {
          const __old = $scope2.ex;
          $scope2.ex = __new40;
          $scope2.__i = __newI41;
          if (__old.name !== $scope2.ex.name) {
            {
              const __av32 = "Create " + $scope2.ex.name + " collection";
              if (__av32 == null || __av32 === false || __av32 === "")
                __e31.removeAttribute("aria-label");
              else
                __e31.setAttribute("aria-label", __av32 === true ? "" : String(__av32));
            }
            __t37.textContent = $scope2.ex.name;
          }
          if (__old.icon !== $scope2.ex.icon) {
            {
              const __av35 = $scope2.ex.icon;
              if (__av35 == null || __av35 === false || __av35 === "")
                __e34.removeAttribute("name");
              else
                __e34.setAttribute("name", __av35 === true ? "" : String(__av35));
            }
          }
          if (__old.description !== $scope2.ex.description) {
            __t39.textContent = $scope2.ex.description;
          }
        } };
      })(__childScope28);
    }
  });
  const __wireBoundary_b13 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.filteredIndex().length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag52 = document.createDocumentFragment();
        const __c53 = [];
        const __e54 = document.createElement("div");
        __e54.setAttribute("class", "col-index-empty");
        __e54.setAttribute("role", "status");
        const __t55 = document.createTextNode("No collections match “");
        __e54.appendChild(__t55);
        const __t56 = document.createTextNode("");
        __e54.appendChild(__t56);
        __c53.push(bindText(__t56, () => $scope.indexQuery()));
        const __t57 = document.createTextNode("”.");
        __e54.appendChild(__t57);
        __frag52.appendChild(__e54);
        return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
          for (const c of __c53)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b15 = (anchor) => bindFor({
    anchor,
    items: () => $scope.filteredIndex(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.id)(__child);
    },
    render: (c, __i) => {
      const __childScope60 = Object.create($scope);
      __childScope60.c = c;
      __childScope60.__i = __i;
      return (($scope2) => {
        __childScope60.__v_item_count = signal2(0);
        const __wireBoundary_b23 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.c.status_summary,
          key: (sv, __i2) => {
            const __child = Object.create($scope2);
            __child.sv = sv;
            __child.__i = __i2;
            return (($scope3) => sv.value)(__child);
          },
          render: (sv, __i2) => {
            const __childScope82 = Object.create($scope2);
            __childScope82.sv = sv;
            __childScope82.__i = __i2;
            return (($scope3) => {
              const __frag83 = document.createDocumentFragment();
              const __c84 = [];
              const __e85 = document.createElement("span");
              {
                const __av86 = "status-chip status-chip-" + $scope3.sv.value;
                if (__av86 == null || __av86 === false || __av86 === "")
                  __e85.removeAttribute("class");
                else
                  __e85.setAttribute("class", __av86 === true ? "" : String(__av86));
              }
              const __e87 = document.createElement("span");
              __e87.setAttribute("class", "status-chip-dot");
              __e87.setAttribute("aria-hidden", "true");
              __e85.appendChild(__e87);
              const __e88 = document.createElement("span");
              __e88.setAttribute("class", "status-chip-label");
              const __t89 = document.createTextNode("");
              __e88.appendChild(__t89);
              __t89.textContent = $scope3.sv.value;
              __e85.appendChild(__e88);
              const __e90 = document.createElement("span");
              __e90.setAttribute("class", "status-chip-count");
              const __t91 = document.createTextNode("");
              __e90.appendChild(__t91);
              __t91.textContent = $scope3.sv.count;
              __e85.appendChild(__e90);
              __frag83.appendChild(__e85);
              return { nodes: Array.from(__frag83.childNodes), cleanup: () => {
                for (const c2 of __c84)
                  try {
                    c2();
                  } catch (e) {}
              }, update: (__new92, __newI93) => {
                const __old = $scope3.sv;
                $scope3.sv = __new92;
                $scope3.__i = __newI93;
                if (__old.value !== $scope3.sv.value) {
                  {
                    const __av86 = "status-chip status-chip-" + $scope3.sv.value;
                    if (__av86 == null || __av86 === false || __av86 === "")
                      __e85.removeAttribute("class");
                    else
                      __e85.setAttribute("class", __av86 === true ? "" : String(__av86));
                  }
                  __t89.textContent = $scope3.sv.value;
                }
                if (__old.count !== $scope3.sv.count) {
                  __t91.textContent = $scope3.sv.count;
                }
              } };
            })(__childScope82);
          }
        });
        const __wireBoundary_b27 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.c.fields.slice(0, 5),
          key: (f, __i2) => {
            const __child = Object.create($scope2);
            __child.f = f;
            __child.__i = __i2;
            return (($scope3) => f.name)(__child);
          },
          render: (f, __i2) => {
            const __childScope98 = Object.create($scope2);
            __childScope98.f = f;
            __childScope98.__i = __i2;
            return (($scope3) => {
              const __wireBoundary_b30 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.f.required)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag107 = document.createDocumentFragment();
                    const __c108 = [];
                    const __e109 = document.createElement("span");
                    __e109.setAttribute("class", "data-card-req");
                    const __t110 = document.createTextNode("*");
                    __e109.appendChild(__t110);
                    __frag107.appendChild(__e109);
                    return { nodes: Array.from(__frag107.childNodes), cleanup: () => {
                      for (const c2 of __c108)
                        try {
                          c2();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag99 = document.createDocumentFragment();
              const __c100 = [];
              const __e101 = document.createElement("span");
              __e101.setAttribute("class", "data-card-field");
              const __e102 = document.createElement("sui-badge");
              __e102.setAttribute("variant", "soft");
              __e102.setAttribute("size", "sm");
              const __t103 = document.createTextNode("");
              __e102.appendChild(__t103);
              __t103.textContent = $scope3.f.type;
              __e101.appendChild(__e102);
              const __e104 = document.createElement("span");
              __e104.setAttribute("class", "mono-snip");
              const __t105 = document.createTextNode("");
              __e104.appendChild(__t105);
              __t105.textContent = $scope3.f.name;
              __e101.appendChild(__e104);
              const __ifAnchor106 = document.createComment("if");
              __e101.appendChild(__ifAnchor106);
              __c100.push(__wireBoundary_b30(__ifAnchor106));
              __frag99.appendChild(__e101);
              return { nodes: Array.from(__frag99.childNodes), cleanup: () => {
                for (const c2 of __c100)
                  try {
                    c2();
                  } catch (e) {}
              }, update: (__new111, __newI112) => {
                const __old = $scope3.f;
                $scope3.f = __new111;
                $scope3.__i = __newI112;
                if (__old.type !== $scope3.f.type) {
                  __t103.textContent = $scope3.f.type;
                }
                if (__old.name !== $scope3.f.name) {
                  __t105.textContent = $scope3.f.name;
                }
              } };
            })(__childScope98);
          }
        });
        const __wireBoundary_b31 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.c.fields.length > 5)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag114 = document.createDocumentFragment();
              const __c115 = [];
              const __e116 = document.createElement("span");
              __e116.setAttribute("class", "data-card-field data-card-field-more");
              const __t117 = document.createTextNode("+");
              __e116.appendChild(__t117);
              const __t118 = document.createTextNode("");
              __e116.appendChild(__t118);
              __t118.textContent = $scope2.c.fields.length - 5;
              const __t119 = document.createTextNode(" more");
              __e116.appendChild(__t119);
              __frag114.appendChild(__e116);
              return { nodes: Array.from(__frag114.childNodes), cleanup: () => {
                for (const c2 of __c115)
                  try {
                    c2();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b22 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.c.status_summary && $scope2.c.status_summary.length > 0)
              return 0;
            if ($scope2.c.fields && $scope2.c.fields.length > 0)
              return 1;
            return -1;
          },
          branches: [
            () => {
              const __frag78 = document.createDocumentFragment();
              const __c79 = [];
              const __e80 = document.createElement("sui-cluster");
              __e80.setAttribute("gap", "1");
              const __forAnchor81 = document.createComment("for");
              __e80.appendChild(__forAnchor81);
              __c79.push(__wireBoundary_b23(__forAnchor81));
              __frag78.appendChild(__e80);
              return { nodes: Array.from(__frag78.childNodes), cleanup: () => {
                for (const c2 of __c79)
                  try {
                    c2();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag94 = document.createDocumentFragment();
              const __c95 = [];
              const __e96 = document.createElement("sui-cluster");
              __e96.setAttribute("gap", "1");
              const __forAnchor97 = document.createComment("for");
              __e96.appendChild(__forAnchor97);
              __c95.push(__wireBoundary_b27(__forAnchor97));
              const __ifAnchor113 = document.createComment("if");
              __e96.appendChild(__ifAnchor113);
              __c95.push(__wireBoundary_b31(__ifAnchor113));
              __frag94.appendChild(__e96);
              return { nodes: Array.from(__frag94.childNodes), cleanup: () => {
                for (const c2 of __c95)
                  try {
                    c2();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag61 = document.createDocumentFragment();
        const __c62 = [];
        const __e63 = document.createElement("a");
        __e63.setAttribute("class", "data-card");
        {
          const __av64 = "/collections?collection=" + $scope2.c.id;
          if (__av64 == null || __av64 === false || __av64 === "")
            __e63.removeAttribute("href");
          else
            __e63.setAttribute("href", __av64 === true ? "" : String(__av64));
        }
        __c62.push(on(__e63, "click", function() {
          const __h = (e) => {
            e.preventDefault();
            $scope2.navigate("/collections?collection=" + $scope2.c.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e65 = document.createElement("sui-cluster");
        __e65.setAttribute("gap", "2");
        __e65.setAttribute("align", "center");
        const __e66 = document.createElement("span");
        __e66.setAttribute("class", "data-card-icon");
        const __e67 = document.createElement("sui-icon");
        __e67.setAttribute("name", "database");
        __e66.appendChild(__e67);
        __e65.appendChild(__e66);
        const __e68 = document.createElement("div");
        __e68.setAttribute("class", "data-card-identity");
        const __e69 = document.createElement("div");
        __e69.setAttribute("class", "data-card-name");
        const __t70 = document.createTextNode("");
        __e69.appendChild(__t70);
        __t70.textContent = $scope2.c.name || $scope2.c.id;
        __e68.appendChild(__e69);
        const __e71 = document.createElement("div");
        __e71.setAttribute("class", "data-card-id mono-snip");
        const __t72 = document.createTextNode("");
        __e71.appendChild(__t72);
        __t72.textContent = $scope2.c.builtin ? "built-in" : ".ship/collections/" + $scope2.c.id + ".jsonc";
        __e68.appendChild(__e71);
        __e65.appendChild(__e68);
        const __e73 = document.createElement("span");
        __e73.setAttribute("class", "data-card-count");
        const __t74 = document.createTextNode("");
        __e73.appendChild(__t74);
        __c62.push(bindText(__t74, () => (void $scope2.__v_item_count(), $scope2.c.item_count ?? 0)));
        const __t75 = document.createTextNode(" item");
        __e73.appendChild(__t75);
        const __t76 = document.createTextNode("");
        __e73.appendChild(__t76);
        __c62.push(bindText(__t76, () => (void $scope2.__v_item_count(), ($scope2.c.item_count ?? 0) !== 1 ? "s" : "")));
        __e65.appendChild(__e73);
        __e63.appendChild(__e65);
        const __ifAnchor77 = document.createComment("if");
        __e63.appendChild(__ifAnchor77);
        __c62.push(__wireBoundary_b22(__ifAnchor77));
        const __e120 = document.createElement("div");
        __e120.setAttribute("class", "data-card-footer");
        const __e121 = document.createElement("span");
        const __e122 = document.createElement("sui-icon");
        __e122.setAttribute("name", "columns");
        __e122.setAttribute("size", "sm");
        __e121.appendChild(__e122);
        const __t123 = document.createTextNode("");
        __e121.appendChild(__t123);
        __t123.textContent = $scope2.c.field_count;
        const __t124 = document.createTextNode(" field");
        __e121.appendChild(__t124);
        const __t125 = document.createTextNode("");
        __e121.appendChild(__t125);
        __t125.textContent = $scope2.c.field_count !== 1 ? "s" : "";
        __e120.appendChild(__e121);
        const __e126 = document.createElement("span");
        __e126.setAttribute("class", "data-card-footer-sep");
        __e126.setAttribute("aria-hidden", "true");
        const __t127 = document.createTextNode("·");
        __e126.appendChild(__t127);
        __e120.appendChild(__e126);
        const __e128 = document.createElement("span");
        const __e129 = document.createElement("sui-icon");
        __e129.setAttribute("name", "clock");
        __e129.setAttribute("size", "sm");
        __e128.appendChild(__e129);
        const __t130 = document.createTextNode("");
        __e128.appendChild(__t130);
        __t130.textContent = $scope2.c.activity_label;
        __e120.appendChild(__e128);
        __e63.appendChild(__e120);
        __frag61.appendChild(__e63);
        return { nodes: Array.from(__frag61.childNodes), cleanup: () => {
          for (const c2 of __c62)
            try {
              c2();
            } catch (e) {}
        }, update: (__new131, __newI132) => {
          const __old = $scope2.c;
          $scope2.c = __new131;
          $scope2.__i = __newI132;
          if (__old.id !== $scope2.c.id) {
            {
              const __av64 = "/collections?collection=" + $scope2.c.id;
              if (__av64 == null || __av64 === false || __av64 === "")
                __e63.removeAttribute("href");
              else
                __e63.setAttribute("href", __av64 === true ? "" : String(__av64));
            }
          }
          if (__old.id !== $scope2.c.id || __old.name !== $scope2.c.name) {
            __t70.textContent = $scope2.c.name || $scope2.c.id;
          }
          if (__old.builtin !== $scope2.c.builtin || __old.id !== $scope2.c.id) {
            __t72.textContent = $scope2.c.builtin ? "built-in" : ".ship/collections/" + $scope2.c.id + ".jsonc";
          }
          if (__old.field_count !== $scope2.c.field_count) {
            __t123.textContent = $scope2.c.field_count;
            __t125.textContent = $scope2.c.field_count !== 1 ? "s" : "";
          }
          if (__old.activity_label !== $scope2.c.activity_label) {
            __t130.textContent = $scope2.c.activity_label;
          }
          if (__old.item_count !== $scope2.c.item_count) {
            $scope2.__v_item_count.set((c2) => c2 + 1);
          }
        } };
      })(__childScope60);
    }
  });
  const __wireBoundary_b2 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.collections().kind === "ok" && $scope.collections().value && $scope.collections().value.items.length === 0)
        return 0;
      if ($scope.collections().kind === "ok" && $scope.collections().value)
        return 1;
      return -1;
    },
    branches: [
      () => {
        const __frag19 = document.createDocumentFragment();
        const __c20 = [];
        const __e21 = document.createElement("sui-empty-state");
        __e21.setAttribute("role", "status");
        const __e22 = document.createElement("span");
        __e22.setAttribute("slot", "title");
        const __t23 = document.createTextNode("No collections yet");
        __e22.appendChild(__t23);
        __e21.appendChild(__e22);
        const __e24 = document.createElement("span");
        __e24.setAttribute("slot", "description");
        const __t25 = document.createTextNode("Pick a template to start, or click “New collection” for a blank one.");
        __e24.appendChild(__t25);
        __e21.appendChild(__e24);
        __frag19.appendChild(__e21);
        const __e26 = document.createElement("div");
        __e26.setAttribute("class", "col-examples");
        __e26.setAttribute("aria-label", "Example collections");
        const __forAnchor27 = document.createComment("for");
        __e26.appendChild(__forAnchor27);
        __c20.push(__wireBoundary_b3(__forAnchor27));
        __frag19.appendChild(__e26);
        return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
          for (const c of __c20)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag42 = document.createDocumentFragment();
        const __c43 = [];
        const __e44 = document.createElement("div");
        __e44.setAttribute("class", "col-index-toolbar");
        const __e45 = document.createElement("sui-input");
        __c43.push(bindAttr(__e45, "value", () => $scope.indexQuery()));
        __e45.setAttribute("placeholder", "Filter collections by name…");
        __c43.push(on(__e45, "input", function() {
          const __h = (e) => $scope.indexQuery.set(e.target.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e46 = document.createElement("sui-icon");
        __e46.setAttribute("slot", "leading");
        __e46.setAttribute("name", "search");
        __e46.setAttribute("size", "sm");
        __e45.appendChild(__e46);
        __e44.appendChild(__e45);
        const __e47 = document.createElement("span");
        __e47.setAttribute("class", "col-index-count");
        const __t48 = document.createTextNode("");
        __e47.appendChild(__t48);
        __c43.push(bindText(__t48, () => $scope.filteredIndex().length));
        const __t49 = document.createTextNode(" of ");
        __e47.appendChild(__t49);
        const __t50 = document.createTextNode("");
        __e47.appendChild(__t50);
        __c43.push(bindText(__t50, () => $scope.collections().value.items.length));
        __e44.appendChild(__e47);
        __frag42.appendChild(__e44);
        const __ifAnchor51 = document.createComment("if");
        __frag42.appendChild(__ifAnchor51);
        __c43.push(__wireBoundary_b13(__ifAnchor51));
        const __e58 = document.createElement("div");
        __e58.setAttribute("class", "data-card-grid");
        const __forAnchor59 = document.createComment("for");
        __e58.appendChild(__forAnchor59);
        __c43.push(__wireBoundary_b15(__forAnchor59));
        __frag42.appendChild(__e58);
        return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
          for (const c of __c43)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b38 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.collections().value.items.length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag146 = document.createDocumentFragment();
        const __c147 = [];
        const __e148 = document.createElement("div");
        __e148.setAttribute("class", "col-nav-empty");
        __e148.setAttribute("role", "status");
        const __e149 = document.createElement("span");
        __e149.setAttribute("slot", "title");
        const __t150 = document.createTextNode("No collections defined");
        __e149.appendChild(__t150);
        __e148.appendChild(__e149);
        __frag146.appendChild(__e148);
        return { nodes: Array.from(__frag146.childNodes), cleanup: () => {
          for (const c of __c147)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b39 = (anchor) => bindFor({
    anchor,
    items: () => $scope.collections().value.items,
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.id)(__child);
    },
    render: (c, __i) => {
      const __childScope152 = Object.create($scope);
      __childScope152.c = c;
      __childScope152.__i = __i;
      return (($scope2) => {
        __childScope152.__v_id = signal2(0);
        __childScope152.__v_item_count = signal2(0);
        const __wireBoundary_b45 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.c.id === $scope2.activeId())
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag164 = document.createDocumentFragment();
              const __c165 = [];
              const __e166 = document.createElement("div");
              __e166.setAttribute("class", "col-nav-sub");
              const __e167 = document.createElement("span");
              __e167.setAttribute("class", "col-nav-sub-dot");
              __e166.appendChild(__e167);
              const __e168 = document.createElement("span");
              const __t169 = document.createTextNode("Default view");
              __e168.appendChild(__t169);
              __e166.appendChild(__e168);
              __frag164.appendChild(__e166);
              return { nodes: Array.from(__frag164.childNodes), cleanup: () => {
                for (const c2 of __c165)
                  try {
                    c2();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag153 = document.createDocumentFragment();
        const __c154 = [];
        const __e155 = document.createElement("a");
        __e155.setAttribute("class", "col-nav-row");
        __c154.push(bindClass(__e155, "is-active", () => (void $scope2.__v_id(), $scope2.c.id === $scope2.activeId())));
        {
          const __av156 = "/collections?collection=" + $scope2.c.id;
          if (__av156 == null || __av156 === false || __av156 === "")
            __e155.removeAttribute("href");
          else
            __e155.setAttribute("href", __av156 === true ? "" : String(__av156));
        }
        __c154.push(on(__e155, "click", function() {
          const __h = (e) => {
            e.preventDefault();
            $scope2.navigate("/collections?collection=" + $scope2.c.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e157 = document.createElement("span");
        __e157.setAttribute("class", "col-nav-icon");
        const __e158 = document.createElement("sui-icon");
        __e158.setAttribute("name", "database");
        __e158.setAttribute("size", "sm");
        __e157.appendChild(__e158);
        __e155.appendChild(__e157);
        const __e159 = document.createElement("span");
        __e159.setAttribute("class", "col-nav-label");
        const __t160 = document.createTextNode("");
        __e159.appendChild(__t160);
        __t160.textContent = $scope2.c.name || $scope2.c.id;
        __e155.appendChild(__e159);
        const __e161 = document.createElement("span");
        __e161.setAttribute("class", "col-nav-count");
        const __t162 = document.createTextNode("");
        __e161.appendChild(__t162);
        __c154.push(bindText(__t162, () => (void $scope2.__v_item_count(), $scope2.c.item_count ?? 0)));
        __e155.appendChild(__e161);
        __frag153.appendChild(__e155);
        const __ifAnchor163 = document.createComment("if");
        __frag153.appendChild(__ifAnchor163);
        __c154.push(__wireBoundary_b45(__ifAnchor163));
        return { nodes: Array.from(__frag153.childNodes), cleanup: () => {
          for (const c2 of __c154)
            try {
              c2();
            } catch (e) {}
        }, update: (__new170, __newI171) => {
          const __old = $scope2.c;
          $scope2.c = __new170;
          $scope2.__i = __newI171;
          if (__old.id !== $scope2.c.id) {
            {
              const __av156 = "/collections?collection=" + $scope2.c.id;
              if (__av156 == null || __av156 === false || __av156 === "")
                __e155.removeAttribute("href");
              else
                __e155.setAttribute("href", __av156 === true ? "" : String(__av156));
            }
          }
          if (__old.id !== $scope2.c.id || __old.name !== $scope2.c.name) {
            __t160.textContent = $scope2.c.name || $scope2.c.id;
          }
          if (__old.id !== $scope2.c.id) {
            $scope2.__v_id.set((c2) => c2 + 1);
          }
          if (__old.item_count !== $scope2.c.item_count) {
            $scope2.__v_item_count.set((c2) => c2 + 1);
          }
        } };
      })(__childScope152);
    }
  });
  const __wireBoundary_b37 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.collections().kind === "ok" && $scope.collections().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag143 = document.createDocumentFragment();
        const __c144 = [];
        const __ifAnchor145 = document.createComment("if");
        __frag143.appendChild(__ifAnchor145);
        __c144.push(__wireBoundary_b38(__ifAnchor145));
        const __forAnchor151 = document.createComment("for");
        __frag143.appendChild(__forAnchor151);
        __c144.push(__wireBoundary_b39(__forAnchor151));
        return { nodes: Array.from(__frag143.childNodes), cleanup: () => {
          for (const c of __c144)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b46 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.colDetail().kind === "ok" && $scope.colDetail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag178 = document.createDocumentFragment();
        const __c179 = [];
        const __e180 = document.createElement("span");
        __e180.setAttribute("class", "col-head-name");
        const __t181 = document.createTextNode("");
        __e180.appendChild(__t181);
        __c179.push(bindText(__t181, () => $scope.colDetail().value.schema.name || $scope.activeId()));
        __frag178.appendChild(__e180);
        const __e182 = document.createElement("span");
        __e182.setAttribute("class", "col-head-id mono-snip");
        const __t183 = document.createTextNode("");
        __e182.appendChild(__t183);
        __c179.push(bindText(__t183, () => $scope.activeId()));
        __frag178.appendChild(__e182);
        const __e184 = document.createElement("span");
        __e184.setAttribute("class", "col-head-count");
        const __t185 = document.createTextNode("");
        __e184.appendChild(__t185);
        __c179.push(bindText(__t185, () => $scope.colDetail().value.items.length));
        const __t186 = document.createTextNode(" of ");
        __e184.appendChild(__t186);
        const __t187 = document.createTextNode("");
        __e184.appendChild(__t187);
        __c179.push(bindText(__t187, () => $scope.colDetail().value.total_items));
        __frag178.appendChild(__e184);
        return { nodes: Array.from(__frag178.childNodes), cleanup: () => {
          for (const c of __c179)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b54 = (anchor) => bindFor({
    anchor,
    items: () => $scope.statusValues(),
    key: (sv, __i) => {
      const __child = Object.create($scope);
      __child.sv = sv;
      __child.__i = __i;
      return (($scope2) => sv)(__child);
    },
    render: (sv, __i) => {
      const __childScope196 = Object.create($scope);
      __childScope196.sv = sv;
      __childScope196.__i = __i;
      return (($scope2) => {
        const __frag197 = document.createDocumentFragment();
        const __c198 = [];
        const __e199 = document.createElement("sui-segmented-option");
        {
          const __av200 = $scope2.sv;
          if (__av200 == null || __av200 === false || __av200 === "")
            __e199.removeAttribute("value");
          else
            __e199.setAttribute("value", __av200 === true ? "" : String(__av200));
        }
        const __t201 = document.createTextNode("");
        __e199.appendChild(__t201);
        __t201.textContent = $scope2.sv;
        __frag197.appendChild(__e199);
        return { nodes: Array.from(__frag197.childNodes), cleanup: () => {
          for (const c of __c198)
            try {
              c();
            } catch (e) {}
        }, update: (__new202, __newI203) => {
          $scope2.sv = __new202;
          $scope2.__i = __newI203;
          {
            const __av200 = $scope2.sv;
            if (__av200 == null || __av200 === false || __av200 === "")
              __e199.removeAttribute("value");
            else
              __e199.setAttribute("value", __av200 === true ? "" : String(__av200));
          }
          __t201.textContent = $scope2.sv;
        } };
      })(__childScope196);
    }
  });
  const __wireBoundary_b51 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.hasStatusField())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag190 = document.createDocumentFragment();
        const __c191 = [];
        const __e192 = document.createElement("sui-segmented");
        __e192.setAttribute("size", "sm");
        __c191.push(bindAttr(__e192, "value", () => $scope.statusFilter()));
        __c191.push(on(__e192, "sui-change", function() {
          const __h = (e) => $scope.statusFilter.set(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e193 = document.createElement("sui-segmented-option");
        __e193.setAttribute("value", "");
        const __t194 = document.createTextNode("All");
        __e193.appendChild(__t194);
        __e192.appendChild(__e193);
        const __forAnchor195 = document.createComment("for");
        __e192.appendChild(__forAnchor195);
        __c191.push(__wireBoundary_b54(__forAnchor195));
        __frag190.appendChild(__e192);
        const __e204 = document.createElement("span");
        __e204.setAttribute("class", "col-head-sep");
        __e204.setAttribute("aria-hidden", "true");
        __frag190.appendChild(__e204);
        return { nodes: Array.from(__frag190.childNodes), cleanup: () => {
          for (const c of __c191)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __renderWhileLoading_0 = () => {
    const __frag218 = document.createDocumentFragment();
    const __c219 = [];
    const __e220 = document.createElement("div");
    __e220.setAttribute("role", "status");
    __e220.setAttribute("aria-live", "polite");
    __e220.setAttribute("aria-busy", "true");
    __e220.setAttribute("style", "padding:var(--space-4);color:var(--muted-foreground)");
    const __t221 = document.createTextNode("Loading…");
    __e220.appendChild(__t221);
    __frag218.appendChild(__e220);
    return { nodes: Array.from(__frag218.childNodes), cleanup: () => {
      for (const c of __c219)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (error) => {
    const __childScope223 = Object.create($scope);
    __childScope223.error = error;
    return (($scope2) => {
      const __frag224 = document.createDocumentFragment();
      const __c225 = [];
      const __e226 = document.createElement("sui-error-state");
      const __t227 = document.createTextNode("");
      __e226.appendChild(__t227);
      __c225.push(bindText(__t227, () => $scope2.error.message));
      __frag224.appendChild(__e226);
      return { nodes: Array.from(__frag224.childNodes), cleanup: () => {
        for (const c of __c225)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope223);
  };
  const __wireBoundary_b69 = (anchor) => bindFor({
    anchor,
    items: () => $scope.colDetail().value.schema.fields,
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.name)(__child);
    },
    render: (f, __i) => {
      const __childScope260 = Object.create($scope);
      __childScope260.f = f;
      __childScope260.__i = __i;
      return (($scope2) => {
        const __frag261 = document.createDocumentFragment();
        const __c262 = [];
        const __e263 = document.createElement("th");
        const __t264 = document.createTextNode("");
        __e263.appendChild(__t264);
        __t264.textContent = $scope2.f.name;
        __frag261.appendChild(__e263);
        return { nodes: Array.from(__frag261.childNodes), cleanup: () => {
          for (const c of __c262)
            try {
              c();
            } catch (e) {}
        }, update: (__new265, __newI266) => {
          const __old = $scope2.f;
          $scope2.f = __new265;
          $scope2.__i = __newI266;
          if (__old.name !== $scope2.f.name) {
            __t264.textContent = $scope2.f.name;
          }
        } };
      })(__childScope260);
    }
  });
  const __wireBoundary_b71 = (anchor) => bindFor({
    anchor,
    items: () => $scope.filteredRows(),
    key: (row, __i) => {
      const __child = Object.create($scope);
      __child.row = row;
      __child.__i = __i;
      return (($scope2) => row.id)(__child);
    },
    render: (row, __i) => {
      const __childScope270 = Object.create($scope);
      __childScope270.row = row;
      __childScope270.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b73 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.row.cells,
          key: (cell, __i2) => {
            const __child = Object.create($scope2);
            __child.cell = cell;
            __child.__i = __i2;
            return (($scope3) => cell.kind + ":" + cell.value)(__child);
          },
          render: (cell, __i2) => {
            const __childScope277 = Object.create($scope2);
            __childScope277.cell = cell;
            __childScope277.__i = __i2;
            return (($scope3) => {
              const __wireBoundary_b74 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.cell.kind === "status")
                    return 0;
                  if ($scope3.cell.kind === "ref")
                    return 1;
                  return 2;
                },
                branches: [
                  () => {
                    const __frag281 = document.createDocumentFragment();
                    const __c282 = [];
                    const __e283 = document.createElement("td");
                    __e283.setAttribute("class", "col-cell col-cell-status");
                    const __e284 = document.createElement("button");
                    __e284.setAttribute("type", "button");
                    {
                      const __av285 = $scope3.cell.class_name;
                      if (__av285 == null || __av285 === false || __av285 === "")
                        __e284.removeAttribute("class");
                      else
                        __e284.setAttribute("class", __av285 === true ? "" : String(__av285));
                    }
                    {
                      const __av286 = "Cycle status for " + $scope3.row.id;
                      if (__av286 == null || __av286 === false || __av286 === "")
                        __e284.removeAttribute("aria-label");
                      else
                        __e284.setAttribute("aria-label", __av286 === true ? "" : String(__av286));
                    }
                    __c282.push(on(__e284, "click", function() {
                      const __h = (e) => {
                        e.stopPropagation();
                        $scope3.cycleStatus($scope3.row.raw);
                      };
                      return typeof __h === "function" ? __h.bind(undefined) : () => {};
                    }(), { modifiers: [] }));
                    const __e287 = document.createElement("span");
                    __e287.setAttribute("class", "status-chip-dot");
                    __e287.setAttribute("aria-hidden", "true");
                    __e284.appendChild(__e287);
                    const __e288 = document.createElement("span");
                    __e288.setAttribute("class", "status-chip-label");
                    const __t289 = document.createTextNode("");
                    __e288.appendChild(__t289);
                    __t289.textContent = $scope3.cell.value;
                    __e284.appendChild(__e288);
                    __e283.appendChild(__e284);
                    __frag281.appendChild(__e283);
                    return { nodes: Array.from(__frag281.childNodes), cleanup: () => {
                      for (const c of __c282)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  },
                  () => {
                    const __frag290 = document.createDocumentFragment();
                    const __c291 = [];
                    const __e292 = document.createElement("td");
                    __e292.setAttribute("class", "col-cell");
                    const __e293 = document.createElement("span");
                    __e293.setAttribute("class", "ref-pill");
                    const __e294 = document.createElement("sui-icon");
                    {
                      const __av295 = $scope3.cell.icon;
                      if (__av295 == null || __av295 === false || __av295 === "")
                        __e294.removeAttribute("name");
                      else
                        __e294.setAttribute("name", __av295 === true ? "" : String(__av295));
                    }
                    __e294.setAttribute("size", "xs");
                    __e293.appendChild(__e294);
                    const __e296 = document.createElement("span");
                    __e296.setAttribute("class", "mono-snip");
                    const __t297 = document.createTextNode("");
                    __e296.appendChild(__t297);
                    __t297.textContent = $scope3.cell.label;
                    __e293.appendChild(__e296);
                    __e292.appendChild(__e293);
                    __frag290.appendChild(__e292);
                    return { nodes: Array.from(__frag290.childNodes), cleanup: () => {
                      for (const c of __c291)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  },
                  () => {
                    const __frag298 = document.createDocumentFragment();
                    const __c299 = [];
                    const __e300 = document.createElement("td");
                    __e300.setAttribute("class", "col-cell");
                    const __t301 = document.createTextNode("");
                    __e300.appendChild(__t301);
                    __t301.textContent = $scope3.cell.value;
                    __frag298.appendChild(__e300);
                    return { nodes: Array.from(__frag298.childNodes), cleanup: () => {
                      for (const c of __c299)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag278 = document.createDocumentFragment();
              const __c279 = [];
              const __ifAnchor280 = document.createComment("if");
              __frag278.appendChild(__ifAnchor280);
              __c279.push(__wireBoundary_b74(__ifAnchor280));
              return { nodes: Array.from(__frag278.childNodes), cleanup: () => {
                for (const c of __c279)
                  try {
                    c();
                  } catch (e) {}
              } };
            })(__childScope277);
          }
        });
        const __frag271 = document.createDocumentFragment();
        const __c272 = [];
        const __e273 = document.createElement("tr");
        const __e274 = document.createElement("td");
        __e274.setAttribute("class", "col-cell mono-snip");
        const __t275 = document.createTextNode("");
        __e274.appendChild(__t275);
        __t275.textContent = $scope2.row.id;
        __e273.appendChild(__e274);
        const __forAnchor276 = document.createComment("for");
        __e273.appendChild(__forAnchor276);
        __c272.push(__wireBoundary_b73(__forAnchor276));
        const __e302 = document.createElement("td");
        const __e303 = document.createElement("sui-button");
        __e303.setAttribute("size", "sm");
        __e303.setAttribute("variant", "ghost");
        {
          const __av304 = "Edit row " + $scope2.row.id;
          if (__av304 == null || __av304 === false || __av304 === "")
            __e303.removeAttribute("aria-label");
          else
            __e303.setAttribute("aria-label", __av304 === true ? "" : String(__av304));
        }
        __c272.push(on(__e303, "click", function() {
          const __h = () => $scope2.openEditRow($scope2.row.raw);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e305 = document.createElement("sui-icon");
        __e305.setAttribute("name", "edit");
        __e305.setAttribute("size", "xs");
        __e303.appendChild(__e305);
        __e302.appendChild(__e303);
        __e273.appendChild(__e302);
        __frag271.appendChild(__e273);
        return { nodes: Array.from(__frag271.childNodes), cleanup: () => {
          for (const c of __c272)
            try {
              c();
            } catch (e) {}
        }, update: (__new306, __newI307) => {
          $scope2.row = __new306;
          $scope2.__i = __newI307;
          __t275.textContent = $scope2.row.id;
          {
            const __av304 = "Edit row " + $scope2.row.id;
            if (__av304 == null || __av304 === false || __av304 === "")
              __e303.removeAttribute("aria-label");
            else
              __e303.setAttribute("aria-label", __av304 === true ? "" : String(__av304));
          }
        } };
      })(__childScope270);
    }
  });
  const __wireBoundary_b66 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.filteredRows().length === 0 && $scope.colDetail().value.items.length === 0)
        return 0;
      if ($scope.filteredRows().length === 0)
        return 1;
      return 2;
    },
    branches: [
      () => {
        const __frag236 = document.createDocumentFragment();
        const __c237 = [];
        const __e238 = document.createElement("sui-empty-state");
        __e238.setAttribute("role", "status");
        __e238.setAttribute("style", "padding:var(--space-8)");
        const __e239 = document.createElement("span");
        __e239.setAttribute("slot", "title");
        const __t240 = document.createTextNode("No items in ");
        __e239.appendChild(__t240);
        const __t241 = document.createTextNode("");
        __e239.appendChild(__t241);
        __c237.push(bindText(__t241, () => $scope.colDetail().value.schema.name || $scope.activeId()));
        __e238.appendChild(__e239);
        const __e242 = document.createElement("span");
        __e242.setAttribute("slot", "description");
        const __t243 = document.createTextNode("Click “New row” to add the first item.");
        __e242.appendChild(__t243);
        __e238.appendChild(__e242);
        __frag236.appendChild(__e238);
        return { nodes: Array.from(__frag236.childNodes), cleanup: () => {
          for (const c of __c237)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag244 = document.createDocumentFragment();
        const __c245 = [];
        const __e246 = document.createElement("div");
        __e246.setAttribute("class", "col-empty-filter");
        __e246.setAttribute("role", "status");
        const __t247 = document.createTextNode(`
                  No items with status “`);
        __e246.appendChild(__t247);
        const __t248 = document.createTextNode("");
        __e246.appendChild(__t248);
        __c245.push(bindText(__t248, () => $scope.statusFilter()));
        const __t249 = document.createTextNode(`”.
                `);
        __e246.appendChild(__t249);
        __frag244.appendChild(__e246);
        return { nodes: Array.from(__frag244.childNodes), cleanup: () => {
          for (const c of __c245)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag250 = document.createDocumentFragment();
        const __c251 = [];
        const __e252 = document.createElement("table");
        __e252.setAttribute("class", "col-table");
        const __e253 = document.createElement("thead");
        const __e254 = document.createElement("tr");
        const __e255 = document.createElement("th");
        const __t256 = document.createTextNode("id ");
        __e255.appendChild(__t256);
        const __e257 = document.createElement("span");
        __e257.setAttribute("aria-label", "sorted ascending");
        const __t258 = document.createTextNode("↑");
        __e257.appendChild(__t258);
        __e255.appendChild(__e257);
        __e254.appendChild(__e255);
        const __forAnchor259 = document.createComment("for");
        __e254.appendChild(__forAnchor259);
        __c251.push(__wireBoundary_b69(__forAnchor259));
        const __e267 = document.createElement("th");
        __e254.appendChild(__e267);
        __e253.appendChild(__e254);
        __e252.appendChild(__e253);
        const __e268 = document.createElement("tbody");
        const __forAnchor269 = document.createComment("for");
        __e268.appendChild(__forAnchor269);
        __c251.push(__wireBoundary_b71(__forAnchor269));
        __e252.appendChild(__e268);
        __frag250.appendChild(__e252);
        return { nodes: Array.from(__frag250.childNodes), cleanup: () => {
          for (const c of __c251)
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
      if ($scope.colDetail().kind === "ok" && $scope.colDetail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag233 = document.createDocumentFragment();
        const __c234 = [];
        const __ifAnchor235 = document.createComment("if");
        __frag233.appendChild(__ifAnchor235);
        __c234.push(__wireBoundary_b66(__ifAnchor235));
        return { nodes: Array.from(__frag233.childNodes), cleanup: () => {
          for (const c of __c234)
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
      if ($scope.colDetail().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag230 = document.createDocumentFragment();
        const __c231 = [];
        const __ifAnchor232 = document.createComment("if");
        __frag230.appendChild(__ifAnchor232);
        __c231.push(__wireBoundary_b65(__ifAnchor232));
        return { nodes: Array.from(__frag230.childNodes), cleanup: () => {
          for (const c of __c231)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b60 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "rows")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag215 = document.createDocumentFragment();
        const __c216 = [];
        const __e217 = document.createElement("div");
        __e217.setAttribute("class", "col-tab-panel rows-panel");
        const __wlAnchor222 = document.createComment("while-loading");
        __e217.appendChild(__wlAnchor222);
        __c216.push(bindWhileLoading({ anchor: __wlAnchor222, signal: $scope.colDetail, render: __renderWhileLoading_0 }));
        const __weAnchor228 = document.createComment("while-error");
        __e217.appendChild(__weAnchor228);
        __c216.push(bindWhileError({ anchor: __weAnchor228, signal: $scope.colDetail, render: __renderWhileError_0 }));
        const __ifAnchor229 = document.createComment("if");
        __e217.appendChild(__ifAnchor229);
        __c216.push(__wireBoundary_b64(__ifAnchor229));
        __frag215.appendChild(__e217);
        return { nodes: Array.from(__frag215.childNodes), cleanup: () => {
          for (const c of __c216)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b90 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.colDetail().value.schema.builtin)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag327 = document.createDocumentFragment();
        const __c328 = [];
        const __e329 = document.createElement("div");
        __e329.setAttribute("class", "schema-builtin-note");
        const __e330 = document.createElement("sui-icon");
        __e330.setAttribute("name", "lock");
        __e330.setAttribute("size", "sm");
        __e329.appendChild(__e330);
        const __t331 = document.createTextNode(`
                    Built-in collections ship with the runtime. Fields are listed for reference only.
                  `);
        __e329.appendChild(__t331);
        __frag327.appendChild(__e329);
        return { nodes: Array.from(__frag327.childNodes), cleanup: () => {
          for (const c of __c328)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b91 = (anchor) => bindFor({
    anchor,
    items: () => $scope.schemaFieldsView(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.name)(__child);
    },
    render: (f, __i) => {
      const __childScope334 = Object.create($scope);
      __childScope334.f = f;
      __childScope334.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b95 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.required)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag344 = document.createDocumentFragment();
              const __c345 = [];
              const __e346 = document.createElement("span");
              __e346.setAttribute("class", "schema-field-req");
              const __t347 = document.createTextNode("required");
              __e346.appendChild(__t347);
              __frag344.appendChild(__e346);
              return { nodes: Array.from(__frag344.childNodes), cleanup: () => {
                for (const c of __c345)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b96 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.values && $scope2.f.values.length > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag349 = document.createDocumentFragment();
              const __c350 = [];
              const __e351 = document.createElement("span");
              __e351.setAttribute("class", "schema-field-values mono-snip");
              const __t352 = document.createTextNode("");
              __e351.appendChild(__t352);
              __t352.textContent = $scope2.f.values_pretty;
              __frag349.appendChild(__e351);
              return { nodes: Array.from(__frag349.childNodes), cleanup: () => {
                for (const c of __c350)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b98 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if (!$scope2.colDetail().value.schema.builtin)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag354 = document.createDocumentFragment();
              const __c355 = [];
              const __e356 = document.createElement("sui-button");
              __e356.setAttribute("size", "sm");
              __e356.setAttribute("variant", "ghost");
              __e356.setAttribute("aria-label", "Remove field");
              __c355.push(on(__e356, "click", function() {
                const __h = () => $scope2.removeSchemaField($scope2.f.name);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              const __e357 = document.createElement("sui-icon");
              __e357.setAttribute("name", "trash");
              __e357.setAttribute("size", "xs");
              __e356.appendChild(__e357);
              __frag354.appendChild(__e356);
              return { nodes: Array.from(__frag354.childNodes), cleanup: () => {
                for (const c of __c355)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag335 = document.createDocumentFragment();
        const __c336 = [];
        const __e337 = document.createElement("div");
        __e337.setAttribute("class", "schema-field");
        const __e338 = document.createElement("span");
        {
          const __av339 = "ft-badge ft-" + $scope2.f.field_type;
          if (__av339 == null || __av339 === false || __av339 === "")
            __e338.removeAttribute("class");
          else
            __e338.setAttribute("class", __av339 === true ? "" : String(__av339));
        }
        const __t340 = document.createTextNode("");
        __e338.appendChild(__t340);
        __t340.textContent = $scope2.f.field_type;
        __e337.appendChild(__e338);
        const __e341 = document.createElement("span");
        __e341.setAttribute("class", "schema-field-name mono-snip");
        const __t342 = document.createTextNode("");
        __e341.appendChild(__t342);
        __t342.textContent = $scope2.f.name;
        __e337.appendChild(__e341);
        const __ifAnchor343 = document.createComment("if");
        __e337.appendChild(__ifAnchor343);
        __c336.push(__wireBoundary_b95(__ifAnchor343));
        const __ifAnchor348 = document.createComment("if");
        __e337.appendChild(__ifAnchor348);
        __c336.push(__wireBoundary_b96(__ifAnchor348));
        const __ifAnchor353 = document.createComment("if");
        __e337.appendChild(__ifAnchor353);
        __c336.push(__wireBoundary_b98(__ifAnchor353));
        __frag335.appendChild(__e337);
        return { nodes: Array.from(__frag335.childNodes), cleanup: () => {
          for (const c of __c336)
            try {
              c();
            } catch (e) {}
        }, update: (__new358, __newI359) => {
          const __old = $scope2.f;
          $scope2.f = __new358;
          $scope2.__i = __newI359;
          if (__old.field_type !== $scope2.f.field_type) {
            {
              const __av339 = "ft-badge ft-" + $scope2.f.field_type;
              if (__av339 == null || __av339 === false || __av339 === "")
                __e338.removeAttribute("class");
              else
                __e338.setAttribute("class", __av339 === true ? "" : String(__av339));
            }
            __t340.textContent = $scope2.f.field_type;
          }
          if (__old.name !== $scope2.f.name) {
            __t342.textContent = $scope2.f.name;
          }
        } };
      })(__childScope334);
    }
  });
  const __wireBoundary_b85 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.colDetail().kind === "ok" && $scope.colDetail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag313 = document.createDocumentFragment();
        const __c314 = [];
        const __e315 = document.createElement("div");
        __e315.setAttribute("class", "schema-pane");
        const __e316 = document.createElement("div");
        __e316.setAttribute("class", "schema-pane-head");
        const __e317 = document.createElement("div");
        const __e318 = document.createElement("h3");
        __e318.setAttribute("class", "schema-pane-title");
        const __t319 = document.createTextNode("");
        __e318.appendChild(__t319);
        __c314.push(bindText(__t319, () => $scope.colDetail().value.schema.name || $scope.activeId()));
        __e317.appendChild(__e318);
        const __e320 = document.createElement("div");
        __e320.setAttribute("class", "schema-pane-sub mono-snip");
        const __t321 = document.createTextNode("");
        __e320.appendChild(__t321);
        __c314.push(bindText(__t321, () => $scope.colDetail().value.schema.builtin ? "built-in" : ".ship/collections/" + $scope.activeId() + ".jsonc"));
        __e317.appendChild(__e320);
        __e316.appendChild(__e317);
        const __e322 = document.createElement("span");
        __e322.setAttribute("class", "schema-pane-count");
        const __t323 = document.createTextNode("");
        __e322.appendChild(__t323);
        __c314.push(bindText(__t323, () => $scope.colDetail().value.schema.fields.length));
        const __t324 = document.createTextNode(" field");
        __e322.appendChild(__t324);
        const __t325 = document.createTextNode("");
        __e322.appendChild(__t325);
        __c314.push(bindText(__t325, () => $scope.colDetail().value.schema.fields.length !== 1 ? "s" : ""));
        __e316.appendChild(__e322);
        __e315.appendChild(__e316);
        const __ifAnchor326 = document.createComment("if");
        __e315.appendChild(__ifAnchor326);
        __c314.push(__wireBoundary_b90(__ifAnchor326));
        const __e332 = document.createElement("sui-stack");
        __e332.setAttribute("gap", "1");
        const __forAnchor333 = document.createComment("for");
        __e332.appendChild(__forAnchor333);
        __c314.push(__wireBoundary_b91(__forAnchor333));
        __e315.appendChild(__e332);
        __frag313.appendChild(__e315);
        return { nodes: Array.from(__frag313.childNodes), cleanup: () => {
          for (const c of __c314)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b84 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "schema")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag309 = document.createDocumentFragment();
        const __c310 = [];
        const __e311 = document.createElement("div");
        __e311.setAttribute("class", "col-tab-panel schema-panel");
        const __ifAnchor312 = document.createComment("if");
        __e311.appendChild(__ifAnchor312);
        __c310.push(__wireBoundary_b85(__ifAnchor312));
        __frag309.appendChild(__e311);
        return { nodes: Array.from(__frag309.childNodes), cleanup: () => {
          for (const c of __c310)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b0 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.activeId())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag2 = document.createDocumentFragment();
        const __c3 = [];
        const __e4 = document.createElement("div");
        __e4.setAttribute("class", "data-index");
        const __e5 = document.createElement("div");
        __e5.setAttribute("class", "data-index-head");
        const __e6 = document.createElement("div");
        const __e7 = document.createElement("h1");
        __e7.setAttribute("class", "data-index-title");
        const __t8 = document.createTextNode("Collections");
        __e7.appendChild(__t8);
        __e6.appendChild(__e7);
        const __e9 = document.createElement("div");
        __e9.setAttribute("class", "data-index-sub");
        const __t10 = document.createTextNode(`
            Typed buckets for tracking specs, skills, artifacts, jobs, or anything else — stored in `);
        __e9.appendChild(__t10);
        const __e11 = document.createElement("span");
        __e11.setAttribute("class", "mono-snip");
        const __t12 = document.createTextNode("~/.ship/data.db");
        __e11.appendChild(__t12);
        __e9.appendChild(__e11);
        const __t13 = document.createTextNode(`.
          `);
        __e9.appendChild(__t13);
        __e6.appendChild(__e9);
        __e5.appendChild(__e6);
        const __e14 = document.createElement("div");
        __e14.setAttribute("class", "data-index-actions");
        const __e15 = document.createElement("sui-button");
        __e15.setAttribute("size", "sm");
        __e15.setAttribute("tone", "primary");
        __e15.setAttribute("variant", "solid");
        __c3.push(on(__e15, "click", function() {
          const __h = () => $scope.newColOpen.set(true);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e16 = document.createElement("sui-icon");
        __e16.setAttribute("slot", "leading");
        __e16.setAttribute("name", "plus");
        __e15.appendChild(__e16);
        const __t17 = document.createTextNode(`New collection
          `);
        __e15.appendChild(__t17);
        __e14.appendChild(__e15);
        __e5.appendChild(__e14);
        __e4.appendChild(__e5);
        const __ifAnchor18 = document.createComment("if");
        __e4.appendChild(__ifAnchor18);
        __c3.push(__wireBoundary_b2(__ifAnchor18));
        __frag2.appendChild(__e4);
        return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
          for (const c of __c3)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag133 = document.createDocumentFragment();
        const __c134 = [];
        const __e135 = document.createElement("div");
        __e135.setAttribute("class", "col-workspace");
        const __e136 = document.createElement("aside");
        __e136.setAttribute("class", "col-nav");
        const __e137 = document.createElement("div");
        __e137.setAttribute("class", "col-nav-section");
        const __e138 = document.createElement("span");
        const __t139 = document.createTextNode("Collections");
        __e138.appendChild(__t139);
        __e137.appendChild(__e138);
        const __e140 = document.createElement("span");
        __e140.setAttribute("class", "col-nav-add");
        __e140.setAttribute("title", "New collection");
        __e140.setAttribute("style", "cursor:pointer");
        __c134.push(on(__e140, "click", function() {
          const __h = () => $scope.newColOpen.set(true);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e141 = document.createElement("sui-icon");
        __e141.setAttribute("name", "plus");
        __e141.setAttribute("size", "sm");
        __e140.appendChild(__e141);
        __e137.appendChild(__e140);
        __e136.appendChild(__e137);
        const __ifAnchor142 = document.createComment("if");
        __e136.appendChild(__ifAnchor142);
        __c134.push(__wireBoundary_b37(__ifAnchor142));
        __e135.appendChild(__e136);
        const __e172 = document.createElement("div");
        __e172.setAttribute("class", "col-work-main");
        const __e173 = document.createElement("div");
        __e173.setAttribute("class", "col-head");
        const __e174 = document.createElement("sui-cluster");
        __e174.setAttribute("gap", "2");
        __e174.setAttribute("align", "center");
        const __e175 = document.createElement("span");
        __e175.setAttribute("class", "col-head-icon");
        const __e176 = document.createElement("sui-icon");
        __e176.setAttribute("name", "database");
        __e176.setAttribute("size", "sm");
        __e175.appendChild(__e176);
        __e174.appendChild(__e175);
        const __ifAnchor177 = document.createComment("if");
        __e174.appendChild(__ifAnchor177);
        __c134.push(__wireBoundary_b46(__ifAnchor177));
        __e173.appendChild(__e174);
        const __e188 = document.createElement("div");
        __e188.setAttribute("class", "col-head-right");
        const __ifAnchor189 = document.createComment("if");
        __e188.appendChild(__ifAnchor189);
        __c134.push(__wireBoundary_b51(__ifAnchor189));
        const __e205 = document.createElement("sui-segmented");
        __e205.setAttribute("size", "sm");
        __c134.push(bindAttr(__e205, "value", () => $scope.tab()));
        __c134.push(on(__e205, "change", function() {
          const __h = (e) => $scope.tab.set(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e206 = document.createElement("sui-segmented-option");
        __e206.setAttribute("value", "rows");
        __e206.setAttribute("icon", "list-check");
        const __t207 = document.createTextNode("Rows");
        __e206.appendChild(__t207);
        __e205.appendChild(__e206);
        const __e208 = document.createElement("sui-segmented-option");
        __e208.setAttribute("value", "schema");
        __e208.setAttribute("icon", "columns");
        const __t209 = document.createTextNode("Schema");
        __e208.appendChild(__t209);
        __e205.appendChild(__e208);
        __e188.appendChild(__e205);
        const __e210 = document.createElement("span");
        __e210.setAttribute("class", "col-head-sep");
        __e210.setAttribute("aria-hidden", "true");
        __e188.appendChild(__e210);
        const __e211 = document.createElement("sui-button");
        __e211.setAttribute("size", "sm");
        __e211.setAttribute("tone", "primary");
        __e211.setAttribute("variant", "solid");
        __c134.push(on(__e211, "click", function() {
          const __h = () => {
            $scope.rowDraft.set({});
            $scope.newRowOpen.set(true);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e212 = document.createElement("sui-icon");
        __e212.setAttribute("slot", "leading");
        __e212.setAttribute("name", "plus");
        __e211.appendChild(__e212);
        const __t213 = document.createTextNode(`New row
            `);
        __e211.appendChild(__t213);
        __e188.appendChild(__e211);
        __e173.appendChild(__e188);
        __e172.appendChild(__e173);
        const __ifAnchor214 = document.createComment("if");
        __e172.appendChild(__ifAnchor214);
        __c134.push(__wireBoundary_b60(__ifAnchor214));
        const __ifAnchor308 = document.createComment("if");
        __e172.appendChild(__ifAnchor308);
        __c134.push(__wireBoundary_b84(__ifAnchor308));
        __e135.appendChild(__e172);
        __frag133.appendChild(__e135);
        return { nodes: Array.from(__frag133.childNodes), cleanup: () => {
          for (const c of __c134)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b104 = (anchor) => bindFor({
    anchor,
    items: () => $scope.colDetail().value.schema.fields,
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.name)(__child);
    },
    render: (f, __i) => {
      const __childScope371 = Object.create($scope);
      __childScope371.f = f;
      __childScope371.__i = __i;
      return (($scope2) => {
        __childScope371.__v_name = signal2(0);
        const __wireBoundary_b106 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.required)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag378 = document.createDocumentFragment();
              const __c379 = [];
              const __e380 = document.createElement("span");
              __e380.setAttribute("class", "drawer-req");
              const __t381 = document.createTextNode("*");
              __e380.appendChild(__t381);
              __frag378.appendChild(__e380);
              return { nodes: Array.from(__frag378.childNodes), cleanup: () => {
                for (const c of __c379)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b108 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.field_type === "boolean")
              return 0;
            if ($scope2.f.field_type === "markdown")
              return 1;
            return 2;
          },
          branches: [
            () => {
              const __frag385 = document.createDocumentFragment();
              const __c386 = [];
              const __e387 = document.createElement("sui-switch");
              __c386.push(bindAttr(__e387, "checked", () => (void $scope2.__v_name(), $scope2.rowDraft()[$scope2.f.name] ? "" : null)));
              __c386.push(on(__e387, "change", function() {
                const __h = (e) => $scope2.setDraftField("row", $scope2.f.name, e.target.checked);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag385.appendChild(__e387);
              return { nodes: Array.from(__frag385.childNodes), cleanup: () => {
                for (const c of __c386)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag388 = document.createDocumentFragment();
              const __c389 = [];
              const __e390 = document.createElement("sui-textarea");
              __e390.setAttribute("rows", "4");
              {
                const __av391 = $scope2.f.name + "…";
                if (__av391 == null || __av391 === false || __av391 === "")
                  __e390.removeAttribute("placeholder");
                else
                  __e390.setAttribute("placeholder", __av391 === true ? "" : String(__av391));
              }
              __c389.push(bindAttr(__e390, "value", () => (void $scope2.__v_name(), $scope2.rowDraft()[$scope2.f.name] ?? "")));
              __c389.push(on(__e390, "input", function() {
                const __h = (e) => $scope2.setDraftField("row", $scope2.f.name, e.target.value);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag388.appendChild(__e390);
              return { nodes: Array.from(__frag388.childNodes), cleanup: () => {
                for (const c of __c389)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag392 = document.createDocumentFragment();
              const __c393 = [];
              const __e394 = document.createElement("sui-input");
              {
                const __av395 = $scope2.f.field_type === "number" ? "number" : "text";
                if (__av395 == null || __av395 === false || __av395 === "")
                  __e394.removeAttribute("type");
                else
                  __e394.setAttribute("type", __av395 === true ? "" : String(__av395));
              }
              __c393.push(bindAttr(__e394, "value", () => (void $scope2.__v_name(), $scope2.rowDraft()[$scope2.f.name] ?? "")));
              {
                const __av396 = $scope2.f.name;
                if (__av396 == null || __av396 === false || __av396 === "")
                  __e394.removeAttribute("placeholder");
                else
                  __e394.setAttribute("placeholder", __av396 === true ? "" : String(__av396));
              }
              __c393.push(on(__e394, "input", function() {
                const __h = (e) => $scope2.setDraftField("row", $scope2.f.name, e.target.value);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag392.appendChild(__e394);
              return { nodes: Array.from(__frag392.childNodes), cleanup: () => {
                for (const c of __c393)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag372 = document.createDocumentFragment();
        const __c373 = [];
        const __e374 = document.createElement("sui-stack");
        __e374.setAttribute("gap", "1");
        const __e375 = document.createElement("label");
        __e375.setAttribute("class", "drawer-label");
        const __t376 = document.createTextNode("");
        __e375.appendChild(__t376);
        __t376.textContent = $scope2.f.name;
        const __ifAnchor377 = document.createComment("if");
        __e375.appendChild(__ifAnchor377);
        __c373.push(__wireBoundary_b106(__ifAnchor377));
        const __e382 = document.createElement("span");
        __e382.setAttribute("class", "drawer-type");
        const __t383 = document.createTextNode("");
        __e382.appendChild(__t383);
        __t383.textContent = $scope2.f.field_type;
        __e375.appendChild(__e382);
        __e374.appendChild(__e375);
        const __ifAnchor384 = document.createComment("if");
        __e374.appendChild(__ifAnchor384);
        __c373.push(__wireBoundary_b108(__ifAnchor384));
        __frag372.appendChild(__e374);
        return { nodes: Array.from(__frag372.childNodes), cleanup: () => {
          for (const c of __c373)
            try {
              c();
            } catch (e) {}
        }, update: (__new397, __newI398) => {
          const __old = $scope2.f;
          $scope2.f = __new397;
          $scope2.__i = __newI398;
          if (__old.name !== $scope2.f.name) {
            __t376.textContent = $scope2.f.name;
          }
          if (__old.field_type !== $scope2.f.field_type) {
            __t383.textContent = $scope2.f.field_type;
          }
          if (__old.name !== $scope2.f.name) {
            $scope2.__v_name.set((c) => c + 1);
          }
        } };
      })(__childScope371);
    }
  });
  const __wireBoundary_b118 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.drawerError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag400 = document.createDocumentFragment();
        const __c401 = [];
        const __e402 = document.createElement("div");
        __e402.setAttribute("class", "drawer-error");
        const __t403 = document.createTextNode("");
        __e402.appendChild(__t403);
        __c401.push(bindText(__t403, () => $scope.drawerError()));
        __frag400.appendChild(__e402);
        return { nodes: Array.from(__frag400.childNodes), cleanup: () => {
          for (const c of __c401)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b102 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.colDetail().kind === "ok" && $scope.colDetail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag364 = document.createDocumentFragment();
        const __c365 = [];
        const __e366 = document.createElement("span");
        __e366.setAttribute("slot", "description");
        const __t367 = document.createTextNode("Add a new item to ");
        __e366.appendChild(__t367);
        const __t368 = document.createTextNode("");
        __e366.appendChild(__t368);
        __c365.push(bindText(__t368, () => $scope.colDetail().value.schema.name || $scope.activeId()));
        __frag364.appendChild(__e366);
        const __e369 = document.createElement("sui-stack");
        __e369.setAttribute("gap", "3");
        const __forAnchor370 = document.createComment("for");
        __e369.appendChild(__forAnchor370);
        __c365.push(__wireBoundary_b104(__forAnchor370));
        const __ifAnchor399 = document.createComment("if");
        __e369.appendChild(__ifAnchor399);
        __c365.push(__wireBoundary_b118(__ifAnchor399));
        __frag364.appendChild(__e369);
        const __e404 = document.createElement("div");
        __e404.setAttribute("slot", "footer");
        const __e405 = document.createElement("sui-button");
        __e405.setAttribute("variant", "ghost");
        __c365.push(on(__e405, "click", function() {
          const __h = () => {
            $scope.newRowOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t406 = document.createTextNode("Cancel");
        __e405.appendChild(__t406);
        __e404.appendChild(__e405);
        const __e407 = document.createElement("sui-button");
        __e407.setAttribute("tone", "primary");
        __e407.setAttribute("variant", "solid");
        __c365.push(on(__e407, "click", function() {
          const __h = $scope.saveNewRow;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t408 = document.createTextNode("Save");
        __e407.appendChild(__t408);
        __e404.appendChild(__e407);
        __frag364.appendChild(__e404);
        return { nodes: Array.from(__frag364.childNodes), cleanup: () => {
          for (const c of __c365)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b126 = (anchor) => bindFor({
    anchor,
    items: () => $scope.colDetail().value.schema.fields,
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.name)(__child);
    },
    render: (f, __i) => {
      const __childScope420 = Object.create($scope);
      __childScope420.f = f;
      __childScope420.__i = __i;
      return (($scope2) => {
        __childScope420.__v_name = signal2(0);
        const __wireBoundary_b128 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.required)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag427 = document.createDocumentFragment();
              const __c428 = [];
              const __e429 = document.createElement("span");
              __e429.setAttribute("class", "drawer-req");
              const __t430 = document.createTextNode("*");
              __e429.appendChild(__t430);
              __frag427.appendChild(__e429);
              return { nodes: Array.from(__frag427.childNodes), cleanup: () => {
                for (const c of __c428)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b130 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.f.field_type === "boolean")
              return 0;
            if ($scope2.f.field_type === "markdown")
              return 1;
            return 2;
          },
          branches: [
            () => {
              const __frag434 = document.createDocumentFragment();
              const __c435 = [];
              const __e436 = document.createElement("sui-switch");
              __c435.push(bindAttr(__e436, "checked", () => (void $scope2.__v_name(), $scope2.editDraft()[$scope2.f.name] ? "" : null)));
              __c435.push(on(__e436, "change", function() {
                const __h = (e) => $scope2.setDraftField("edit", $scope2.f.name, e.target.checked);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag434.appendChild(__e436);
              return { nodes: Array.from(__frag434.childNodes), cleanup: () => {
                for (const c of __c435)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag437 = document.createDocumentFragment();
              const __c438 = [];
              const __e439 = document.createElement("sui-textarea");
              __e439.setAttribute("rows", "4");
              {
                const __av440 = $scope2.f.name + "…";
                if (__av440 == null || __av440 === false || __av440 === "")
                  __e439.removeAttribute("placeholder");
                else
                  __e439.setAttribute("placeholder", __av440 === true ? "" : String(__av440));
              }
              __c438.push(bindAttr(__e439, "value", () => (void $scope2.__v_name(), $scope2.editDraft()[$scope2.f.name] ?? "")));
              __c438.push(on(__e439, "input", function() {
                const __h = (e) => $scope2.setDraftField("edit", $scope2.f.name, e.target.value);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag437.appendChild(__e439);
              return { nodes: Array.from(__frag437.childNodes), cleanup: () => {
                for (const c of __c438)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag441 = document.createDocumentFragment();
              const __c442 = [];
              const __e443 = document.createElement("sui-input");
              {
                const __av444 = $scope2.f.field_type === "number" ? "number" : "text";
                if (__av444 == null || __av444 === false || __av444 === "")
                  __e443.removeAttribute("type");
                else
                  __e443.setAttribute("type", __av444 === true ? "" : String(__av444));
              }
              __c442.push(bindAttr(__e443, "value", () => (void $scope2.__v_name(), $scope2.editDraft()[$scope2.f.name] ?? "")));
              {
                const __av445 = $scope2.f.name;
                if (__av445 == null || __av445 === false || __av445 === "")
                  __e443.removeAttribute("placeholder");
                else
                  __e443.setAttribute("placeholder", __av445 === true ? "" : String(__av445));
              }
              __c442.push(on(__e443, "input", function() {
                const __h = (e) => $scope2.setDraftField("edit", $scope2.f.name, e.target.value);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __frag441.appendChild(__e443);
              return { nodes: Array.from(__frag441.childNodes), cleanup: () => {
                for (const c of __c442)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag421 = document.createDocumentFragment();
        const __c422 = [];
        const __e423 = document.createElement("sui-stack");
        __e423.setAttribute("gap", "1");
        const __e424 = document.createElement("label");
        __e424.setAttribute("class", "drawer-label");
        const __t425 = document.createTextNode("");
        __e424.appendChild(__t425);
        __t425.textContent = $scope2.f.name;
        const __ifAnchor426 = document.createComment("if");
        __e424.appendChild(__ifAnchor426);
        __c422.push(__wireBoundary_b128(__ifAnchor426));
        const __e431 = document.createElement("span");
        __e431.setAttribute("class", "drawer-type");
        const __t432 = document.createTextNode("");
        __e431.appendChild(__t432);
        __t432.textContent = $scope2.f.field_type;
        __e424.appendChild(__e431);
        __e423.appendChild(__e424);
        const __ifAnchor433 = document.createComment("if");
        __e423.appendChild(__ifAnchor433);
        __c422.push(__wireBoundary_b130(__ifAnchor433));
        __frag421.appendChild(__e423);
        return { nodes: Array.from(__frag421.childNodes), cleanup: () => {
          for (const c of __c422)
            try {
              c();
            } catch (e) {}
        }, update: (__new446, __newI447) => {
          const __old = $scope2.f;
          $scope2.f = __new446;
          $scope2.__i = __newI447;
          if (__old.name !== $scope2.f.name) {
            __t425.textContent = $scope2.f.name;
          }
          if (__old.field_type !== $scope2.f.field_type) {
            __t432.textContent = $scope2.f.field_type;
          }
          if (__old.name !== $scope2.f.name) {
            $scope2.__v_name.set((c) => c + 1);
          }
        } };
      })(__childScope420);
    }
  });
  const __wireBoundary_b140 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.drawerError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag449 = document.createDocumentFragment();
        const __c450 = [];
        const __e451 = document.createElement("div");
        __e451.setAttribute("class", "drawer-error");
        const __t452 = document.createTextNode("");
        __e451.appendChild(__t452);
        __c450.push(bindText(__t452, () => $scope.drawerError()));
        __frag449.appendChild(__e451);
        return { nodes: Array.from(__frag449.childNodes), cleanup: () => {
          for (const c of __c450)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b124 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.colDetail().kind === "ok" && $scope.colDetail().value)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag413 = document.createDocumentFragment();
        const __c414 = [];
        const __e415 = document.createElement("span");
        __e415.setAttribute("slot", "description");
        const __t416 = document.createTextNode("Editing ");
        __e415.appendChild(__t416);
        const __t417 = document.createTextNode("");
        __e415.appendChild(__t417);
        __c414.push(bindText(__t417, () => $scope.editId()));
        __frag413.appendChild(__e415);
        const __e418 = document.createElement("sui-stack");
        __e418.setAttribute("gap", "3");
        const __forAnchor419 = document.createComment("for");
        __e418.appendChild(__forAnchor419);
        __c414.push(__wireBoundary_b126(__forAnchor419));
        const __ifAnchor448 = document.createComment("if");
        __e418.appendChild(__ifAnchor448);
        __c414.push(__wireBoundary_b140(__ifAnchor448));
        __frag413.appendChild(__e418);
        const __e453 = document.createElement("div");
        __e453.setAttribute("slot", "footer");
        const __e454 = document.createElement("sui-button");
        __e454.setAttribute("tone", "destructive");
        __e454.setAttribute("variant", "soft");
        __e454.setAttribute("style", "margin-right:auto");
        __c414.push(on(__e454, "click", function() {
          const __h = $scope.deleteRow;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t455 = document.createTextNode("Delete");
        __e454.appendChild(__t455);
        __e453.appendChild(__e454);
        const __e456 = document.createElement("sui-button");
        __e456.setAttribute("variant", "ghost");
        __c414.push(on(__e456, "click", function() {
          const __h = () => {
            $scope.editOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t457 = document.createTextNode("Cancel");
        __e456.appendChild(__t457);
        __e453.appendChild(__e456);
        const __e458 = document.createElement("sui-button");
        __e458.setAttribute("tone", "primary");
        __e458.setAttribute("variant", "solid");
        __c414.push(on(__e458, "click", function() {
          const __h = $scope.saveEdit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t459 = document.createTextNode("Save");
        __e458.appendChild(__t459);
        __e453.appendChild(__e458);
        __frag413.appendChild(__e453);
        return { nodes: Array.from(__frag413.childNodes), cleanup: () => {
          for (const c of __c414)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b149 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.drawerError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag479 = document.createDocumentFragment();
        const __c480 = [];
        const __e481 = document.createElement("div");
        __e481.setAttribute("class", "drawer-error");
        const __t482 = document.createTextNode("");
        __e481.appendChild(__t482);
        __c480.push(bindText(__t482, () => $scope.drawerError()));
        __frag479.appendChild(__e481);
        return { nodes: Array.from(__frag479.childNodes), cleanup: () => {
          for (const c of __c480)
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
      __e0.setAttribute("class", "ship-view data-view");
      const __ifAnchor1 = document.createComment("if");
      __e0.appendChild(__ifAnchor1);
      __cleanups.push(__wireBoundary_b0(__ifAnchor1));
      const __e360 = document.createElement("sui-drawer");
      __cleanups.push(bindAttr(__e360, "open", () => $scope.newRowOpen() ? "" : null));
      __e360.setAttribute("size", "md");
      __cleanups.push(on(__e360, "sui-close", function() {
        const __h = () => {
          $scope.newRowOpen.set(false);
          $scope.drawerError.set("");
        };
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e361 = document.createElement("span");
      __e361.setAttribute("slot", "title");
      const __t362 = document.createTextNode("New row");
      __e361.appendChild(__t362);
      __e360.appendChild(__e361);
      const __ifAnchor363 = document.createComment("if");
      __e360.appendChild(__ifAnchor363);
      __cleanups.push(__wireBoundary_b102(__ifAnchor363));
      __e0.appendChild(__e360);
      const __e409 = document.createElement("sui-drawer");
      __cleanups.push(bindAttr(__e409, "open", () => $scope.editOpen() ? "" : null));
      __e409.setAttribute("size", "md");
      __cleanups.push(on(__e409, "sui-close", function() {
        const __h = () => {
          $scope.editOpen.set(false);
          $scope.drawerError.set("");
        };
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e410 = document.createElement("span");
      __e410.setAttribute("slot", "title");
      const __t411 = document.createTextNode("Edit row");
      __e410.appendChild(__t411);
      __e409.appendChild(__e410);
      const __ifAnchor412 = document.createComment("if");
      __e409.appendChild(__ifAnchor412);
      __cleanups.push(__wireBoundary_b124(__ifAnchor412));
      __e0.appendChild(__e409);
      const __e460 = document.createElement("sui-drawer");
      __cleanups.push(bindAttr(__e460, "open", () => $scope.newColOpen() ? "" : null));
      __e460.setAttribute("size", "md");
      __cleanups.push(on(__e460, "sui-close", function() {
        const __h = () => {
          $scope.newColOpen.set(false);
          $scope.drawerError.set("");
        };
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e461 = document.createElement("span");
      __e461.setAttribute("slot", "title");
      const __t462 = document.createTextNode("New collection");
      __e461.appendChild(__t462);
      __e460.appendChild(__e461);
      const __e463 = document.createElement("span");
      __e463.setAttribute("slot", "description");
      const __t464 = document.createTextNode("Define a typed dataset stored in data.db");
      __e463.appendChild(__t464);
      __e460.appendChild(__e463);
      const __e465 = document.createElement("sui-stack");
      __e465.setAttribute("gap", "3");
      const __e466 = document.createElement("sui-stack");
      __e466.setAttribute("gap", "1");
      const __e467 = document.createElement("label");
      __e467.setAttribute("class", "drawer-label");
      const __t468 = document.createTextNode(`
          Collection ID `);
      __e467.appendChild(__t468);
      const __e469 = document.createElement("span");
      __e469.setAttribute("class", "drawer-req");
      const __t470 = document.createTextNode("*");
      __e469.appendChild(__t470);
      __e467.appendChild(__e469);
      const __e471 = document.createElement("span");
      __e471.setAttribute("class", "drawer-type");
      const __t472 = document.createTextNode("snake_case");
      __e471.appendChild(__t472);
      __e467.appendChild(__e471);
      __e466.appendChild(__e467);
      const __e473 = document.createElement("sui-input");
      __cleanups.push(bindValue(__e473, $scope.newColDraftId));
      __e473.setAttribute("placeholder", "my_collection");
      __e473.setAttribute("required", "");
      __e473.setAttribute("style", "font-family:var(--font-mono)");
      __e466.appendChild(__e473);
      __e465.appendChild(__e466);
      const __e474 = document.createElement("sui-stack");
      __e474.setAttribute("gap", "1");
      const __e475 = document.createElement("label");
      __e475.setAttribute("class", "drawer-label");
      const __t476 = document.createTextNode("Display name");
      __e475.appendChild(__t476);
      __e474.appendChild(__e475);
      const __e477 = document.createElement("sui-input");
      __cleanups.push(bindValue(__e477, $scope.newColDraftName));
      __e477.setAttribute("placeholder", "My Collection");
      __e474.appendChild(__e477);
      __e465.appendChild(__e474);
      const __ifAnchor478 = document.createComment("if");
      __e465.appendChild(__ifAnchor478);
      __cleanups.push(__wireBoundary_b149(__ifAnchor478));
      __e460.appendChild(__e465);
      const __e483 = document.createElement("div");
      __e483.setAttribute("slot", "footer");
      const __e484 = document.createElement("sui-button");
      __e484.setAttribute("variant", "ghost");
      __cleanups.push(on(__e484, "click", function() {
        const __h = () => {
          $scope.newColOpen.set(false);
          $scope.drawerError.set("");
        };
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t485 = document.createTextNode("Cancel");
      __e484.appendChild(__t485);
      __e483.appendChild(__e484);
      const __e486 = document.createElement("sui-button");
      __e486.setAttribute("tone", "primary");
      __e486.setAttribute("variant", "solid");
      __cleanups.push(on(__e486, "click", function() {
        const __h = $scope.createCollection;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t487 = document.createTextNode("Create collection");
      __e486.appendChild(__t487);
      __e483.appendChild(__e486);
      __e460.appendChild(__e483);
      __e0.appendChild(__e460);
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
      if (__m["b0"]) {
        const __t = __m["b0"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b0(__anchor));
      }
      if (__m["b100"])
        __cleanups.push(hydrateAttr(__m["b100"], "open", () => $scope.newRowOpen() ? "" : null));
      if (__m["b101"])
        __cleanups.push(hydrateOn(__m["b101"], "sui-close", function() {
          const __h = () => {
            $scope.newRowOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b102"]) {
        const __t = __m["b102"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b102(__anchor));
      }
      if (__m["b122"])
        __cleanups.push(hydrateAttr(__m["b122"], "open", () => $scope.editOpen() ? "" : null));
      if (__m["b123"])
        __cleanups.push(hydrateOn(__m["b123"], "sui-close", function() {
          const __h = () => {
            $scope.editOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b124"]) {
        const __t = __m["b124"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b124(__anchor));
      }
      if (__m["b145"])
        __cleanups.push(hydrateAttr(__m["b145"], "open", () => $scope.newColOpen() ? "" : null));
      if (__m["b146"])
        __cleanups.push(hydrateOn(__m["b146"], "sui-close", function() {
          const __h = () => {
            $scope.newColOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b147"])
        __cleanups.push(hydrateBindValue(__m["b147"], $scope.newColDraftId));
      if (__m["b148"])
        __cleanups.push(hydrateBindValue(__m["b148"], $scope.newColDraftName));
      if (__m["b149"]) {
        const __t = __m["b149"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b149(__anchor));
      }
      if (__m["b151"])
        __cleanups.push(hydrateOn(__m["b151"], "click", function() {
          const __h = () => {
            $scope.newColOpen.set(false);
            $scope.drawerError.set("");
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b152"])
        __cleanups.push(hydrateOn(__m["b152"], "click", function() {
          const __h = $scope.createCollection;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
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
