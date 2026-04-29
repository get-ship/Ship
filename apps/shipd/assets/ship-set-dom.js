// packages/ship-set-dom/src/template/expr.ts
function compileExpr(expr, opts = {}) {
  const trimmed = expr.trim();
  if (!trimmed) {
    throw new SyntaxError(formatErr("empty expression", opts.location));
  }
  validate(trimmed, opts);
  const body = `with($scope) { return (${trimmed}); }`;
  let fn;
  try {
    fn = new Function("$scope", body);
  } catch (e) {
    throw new SyntaxError(
      formatErr(
        `invalid expression \u2014 ${e.message}: \`${trimmed}\``,
        opts.location
      )
    );
  }
  return (scope) => fn(scope);
}
function compileSource(expr, paramNames, opts = {}) {
  validate(expr.trim(), opts);
  const body = `return (${expr.trim()});`;
  try {
    return new Function(...paramNames, body);
  } catch (e) {
    throw new SyntaxError(
      formatErr(
        `invalid source \u2014 ${e.message}: \`${expr.trim()}\``,
        opts.location
      )
    );
  }
}
var formatErr = (msg, loc) => loc ? `[ship-set-dom] ${msg} (at ${loc})` : `[ship-set-dom] ${msg}`;
function validate(expr, opts) {
  const len = expr.length;
  const state2 = {
    inString: null,
    inTemplate: 0,
    parenDepth: 0,
    braceDepth: 0
  };
  for (let i = 0; i < len; i++) {
    const c = expr[i];
    const prev = i > 0 ? expr[i - 1] : "";
    const next = i + 1 < len ? expr[i + 1] : "";
    if (state2.inString) {
      if (c === "\\") {
        i++;
        continue;
      }
      if (c === state2.inString) {
        state2.inString = null;
      } else if (state2.inString === "`" && c === "$" && next === "{") {
        state2.inTemplate++;
        state2.inString = null;
        i++;
      }
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      state2.inString = c;
      continue;
    }
    if (c === "{") {
      state2.braceDepth++;
      continue;
    }
    if (c === "}") {
      if (state2.inTemplate > 0 && state2.braceDepth === 0) {
        state2.inTemplate--;
        state2.inString = "`";
        continue;
      }
      state2.braceDepth--;
      continue;
    }
    if (c === "(") state2.parenDepth++;
    if (c === ")") state2.parenDepth--;
    if (c === ";") {
      throw new SyntaxError(
        formatErr(
          `statements are not allowed in expressions: \`${expr}\``,
          opts.location
        )
      );
    }
    if (c === "=" && next === ">") {
      if (!opts.allowArrow) {
        throw new SyntaxError(
          formatErr(
            `arrow functions are not allowed here: \`${expr}\``,
            opts.location
          )
        );
      }
      i++;
      continue;
    }
    if (c === "=") {
      const prev1 = prev;
      const next1 = next;
      const isCompare = prev1 === "=" || prev1 === "!" || prev1 === "<" || prev1 === ">" || next1 === "=";
      if (!isCompare) {
        throw new SyntaxError(
          formatErr(
            `assignments are not allowed in expressions: \`${expr}\``,
            opts.location
          )
        );
      }
    }
  }
  if (state2.inString) {
    throw new SyntaxError(
      formatErr(`unterminated string in expression: \`${expr}\``, opts.location)
    );
  }
}

// packages/ship-set-dom/src/preprocess.ts
var VOID_TAGS = /* @__PURE__ */ new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "source",
  "track",
  "wbr"
]);
function preprocess(source) {
  return expandSelfClose(quoteBraceAttrs(source));
}
function quoteBraceAttrs(source) {
  let out = "";
  let i = 0;
  const len = source.length;
  while (i < len) {
    const c = source[i];
    if (c !== "<") {
      out += c;
      i++;
      continue;
    }
    const tagStart = i;
    out += "<";
    i++;
    let inSingle = false;
    let inDouble = false;
    while (i < len) {
      const ch = source[i];
      if (ch === '"' && !inSingle) {
        inDouble = !inDouble;
        out += ch;
        i++;
        continue;
      }
      if (ch === "'" && !inDouble) {
        inSingle = !inSingle;
        out += ch;
        i++;
        continue;
      }
      if (!inSingle && !inDouble && ch === ">") {
        out += ">";
        i++;
        break;
      }
      if (!inSingle && !inDouble && ch === "=" && source[i + 1] === "{") {
        const end = findClose(source, i + 1);
        if (end === -1) {
          out += ch;
          i++;
          continue;
        }
        const expr = source.slice(i + 1, end + 1);
        out += '="' + expr.replace(/"/g, "&quot;") + '"';
        i = end + 1;
        continue;
      }
      out += ch;
      i++;
    }
    void tagStart;
  }
  return out;
}
var findClose = (s, openBraceAt) => {
  let depth = 0;
  let str = null;
  for (let i = openBraceAt; i < s.length; i++) {
    const c = s[i];
    if (str) {
      if (c === "\\") {
        i++;
        continue;
      }
      if (c === str) str = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      str = c;
      continue;
    }
    if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
};
function expandSelfClose(source) {
  let out = "";
  let i = 0;
  const len = source.length;
  while (i < len) {
    const c = source[i];
    if (c !== "<") {
      out += c;
      i++;
      continue;
    }
    if (source[i + 1] === "/" || source[i + 1] === "!") {
      const end = source.indexOf(">", i);
      if (end === -1) {
        out += source.slice(i);
        break;
      }
      out += source.slice(i, end + 1);
      i = end + 1;
      continue;
    }
    const tagMatch = source.slice(i + 1).match(/^([A-Za-z][A-Za-z0-9-]*)/);
    if (!tagMatch) {
      out += c;
      i++;
      continue;
    }
    const tag = tagMatch[1].toLowerCase();
    let j = i + 1 + tagMatch[1].length;
    let inSingle = false;
    let inDouble = false;
    while (j < len) {
      const ch = source[j];
      if (ch === '"' && !inSingle) inDouble = !inDouble;
      else if (ch === "'" && !inDouble) inSingle = !inSingle;
      else if (!inSingle && !inDouble && ch === ">") break;
      j++;
    }
    if (j === len) {
      out += source.slice(i);
      break;
    }
    const head = source.slice(i, j);
    const trimmedHead = head.replace(/\s*\/\s*$/, "");
    const isSelfClose = trimmedHead !== head;
    if (isSelfClose && !VOID_TAGS.has(tag)) {
      out += trimmedHead + `></${tag}>`;
    } else {
      out += source.slice(i, j + 1);
    }
    i = j + 1;
  }
  return out;
}

// packages/ship-set-dom/src/parse.ts
var TOP_LEVEL_TAGS = /* @__PURE__ */ new Set(["data", "state", "component"]);
function parseSetSource(source) {
  const prepared = preprocess(source);
  const wrapped = `<!doctype html><html><body>${prepared}</body></html>`;
  const doc = new DOMParser().parseFromString(wrapped, "text/html");
  const body = doc.body;
  const data = [];
  const state2 = [];
  let component2 = null;
  const toRemove = [];
  for (const child of Array.from(body.children)) {
    const tag = child.tagName.toLowerCase();
    if (!TOP_LEVEL_TAGS.has(tag)) continue;
    toRemove.push(child);
    if (tag === "data") data.push(parseData(child));
    else if (tag === "state") state2.push(parseState(child));
    else if (tag === "component") {
      if (component2) {
        throw new Error(
          "[ship-set-dom] only one <component> declaration allowed per file"
        );
      }
      component2 = parseComponent(child);
    }
  }
  for (const el of toRemove) el.remove();
  trimEdges(body);
  return {
    data,
    state: state2,
    component: component2,
    templateNodes: Array.from(body.childNodes),
    doc
  };
}
var parseData = (el) => {
  const name = required(el, "name", "<data>");
  const source = required(el, "source", "<data>");
  return {
    name,
    source: compileSource(source, ["ship", "route"], {
      location: `<data name="${name}">`
    }),
    live: el.hasAttribute("live")
  };
};
var parseState = (el) => {
  const name = required(el, "name", "<state>");
  const raw = required(el, "value", "<state>");
  const expr = stripExprBraces(raw);
  return {
    name,
    value: compileExpr(expr, { location: `<state name="${name}">` })
  };
};
var stripExprBraces = (s) => {
  const trimmed = s.trim();
  if (trimmed.startsWith("{") && trimmed.endsWith("}")) {
    return trimmed.slice(1, -1).trim();
  }
  return trimmed;
};
var parseComponent = (el) => {
  const tag = required(el, "name", "<component>");
  const props = [];
  for (const child of Array.from(el.children)) {
    if (child.tagName.toLowerCase() !== "prop") continue;
    props.push(parseProp(child));
  }
  return { tag, props };
};
var parseProp = (el) => {
  const name = required(el, "name", "<prop>");
  const valuesAttr = el.getAttribute("values");
  const typeAttr = el.getAttribute("type");
  const defaultAttr = el.getAttribute("default") ?? void 0;
  const reflect = el.hasAttribute("reflect");
  const bindable = el.hasAttribute("bindable");
  if (valuesAttr) {
    const values = valuesAttr.split("|").map((v) => v.trim()).filter(Boolean);
    return {
      name,
      type: "union",
      values,
      default: defaultAttr,
      reflect,
      bindable
    };
  }
  switch (typeAttr) {
    case "bool":
      return {
        name,
        type: "bool",
        default: defaultAttr === "true",
        reflect,
        bindable
      };
    case "number":
      return {
        name,
        type: "number",
        default: defaultAttr === void 0 ? void 0 : Number(defaultAttr),
        reflect,
        bindable
      };
    case "string":
    case null:
    case void 0:
      return {
        name,
        type: "string",
        default: defaultAttr,
        reflect,
        bindable
      };
    default:
      throw new Error(
        `[ship-set-dom] <prop name="${name}"> unknown type: ${typeAttr}`
      );
  }
};
var required = (el, attr, label) => {
  const v = el.getAttribute(attr);
  if (v === null) {
    throw new Error(`[ship-set-dom] ${label} missing required attribute: ${attr}`);
  }
  return v;
};
var trimEdges = (parent) => {
  while (parent.firstChild && isEmptyText(parent.firstChild)) {
    parent.firstChild.remove();
  }
  while (parent.lastChild && isEmptyText(parent.lastChild)) {
    parent.lastChild.remove();
  }
};
var isEmptyText = (n) => n.nodeType === 3 && (n.textContent ?? "").trim() === "";

// packages/ship-set-dom/src/bindings/attr-compile.ts
var EXPR_RE = /^\{(.*)\}$/s;
var isExprValue = (v) => EXPR_RE.test(v.trim());
var stripBraces = (v) => {
  const m = v.trim().match(EXPR_RE);
  return m ? m[1] ?? "" : v;
};
function compileAttrs(el, locPrefix) {
  const out = [];
  for (const { name, value } of Array.from(el.attributes)) {
    out.push(compileAttr(name, value, `${locPrefix} ${name}`));
  }
  return out;
}
var compileAttr = (name, value, loc) => {
  if (name.startsWith("on:")) return compileOn(name, value, loc);
  if (name.startsWith("bind:")) return compileBind(name, value, loc);
  if (name.startsWith("class:")) return compileClass(name, value, loc);
  if (name.startsWith("style:")) return compileStyle(name, value, loc);
  if (name.startsWith("attr:")) return compileDynamicAttr(name, value, loc);
  if (isExprValue(value)) {
    return {
      kind: "attr",
      name,
      expr: compileExpr(stripBraces(value), { location: loc })
    };
  }
  return { kind: "static", name, value };
};
var compileOn = (name, value, loc) => {
  const parts = name.slice(3).split("|");
  const event = parts[0] ?? "";
  if (!event) {
    throw new Error(`[ship-set-dom] empty event name in on: binding (at ${loc})`);
  }
  const modifiers = [];
  for (let i = 1; i < parts.length; i++) {
    const m = parts[i] ?? "";
    if (!m) continue;
    modifiers.push(parseModifier(m, loc));
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] on: handler must be {expression} (at ${loc})`
    );
  }
  return {
    kind: "on",
    event,
    handler: compileExpr(stripBraces(value), {
      allowArrow: true,
      location: loc
    }),
    modifiers
  };
};
var parseModifier = (raw, loc) => {
  const colonAt = raw.indexOf(":");
  const head = colonAt === -1 ? raw : raw.slice(0, colonAt);
  const arg = colonAt === -1 ? void 0 : raw.slice(colonAt + 1);
  switch (head.toLowerCase()) {
    case "preventdefault":
    case "prevent":
      return { kind: "preventDefault" };
    case "stoppropagation":
    case "stop":
      return { kind: "stopPropagation" };
    case "stopimmediatepropagation":
      return { kind: "stopImmediatePropagation" };
    case "self":
      return { kind: "self" };
    case "trusted":
      return { kind: "trusted" };
    case "key":
      return arg !== void 0 ? { kind: "key", arg } : { kind: "key" };
    case "debounce":
      return arg !== void 0 ? { kind: "debounce", arg } : { kind: "debounce" };
    case "throttle":
      return arg !== void 0 ? { kind: "throttle", arg } : { kind: "throttle" };
    default:
      throw new Error(
        `[ship-set-dom] unknown modifier "${head}" (at ${loc})`
      );
  }
};
var compileBind = (name, value, loc) => {
  const target = name.slice(5);
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] bind:${target} expects {signal} (at ${loc})`
    );
  }
  const expr = compileExpr(stripBraces(value), { location: loc });
  if (target === "value") return { kind: "bind-value", expr };
  if (target === "checked") return { kind: "bind-checked", expr };
  throw new Error(`[ship-set-dom] unknown bind:${target} (at ${loc})`);
};
var compileClass = (name, value, loc) => {
  const className = name.slice(6);
  if (!className) {
    throw new Error(`[ship-set-dom] class: requires a class name (at ${loc})`);
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] class:${className} expects {expr} (at ${loc})`
    );
  }
  return {
    kind: "class",
    name: className,
    expr: compileExpr(stripBraces(value), { location: loc })
  };
};
var compileStyle = (name, value, loc) => {
  const prop2 = name.slice(6);
  if (!prop2) {
    throw new Error(`[ship-set-dom] style: requires a property name (at ${loc})`);
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] style:${prop2} expects {expr} (at ${loc})`
    );
  }
  return {
    kind: "style",
    name: prop2,
    expr: compileExpr(stripBraces(value), { location: loc })
  };
};
var compileDynamicAttr = (name, value, loc) => {
  const rawNameExpr = name.slice(5);
  if (!isExprValue(rawNameExpr)) {
    throw new Error(
      `[ship-set-dom] attr: expects attr:{nameExpr} (at ${loc})`
    );
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] attr:{...} expects ={valueExpr} (at ${loc})`
    );
  }
  return {
    kind: "attr-dynamic",
    nameExpr: compileExpr(stripBraces(rawNameExpr), { location: loc }),
    expr: compileExpr(stripBraces(value), { location: loc })
  };
};

// packages/ship-set-dom/src/template/tokenize.ts
function tokenizeChildren(parent) {
  const out = [];
  for (const node of Array.from(parent.childNodes)) {
    if (node.nodeType === 3) {
      tokenizeText(node.data, out);
    } else if (node.nodeType === 8) {
      out.push({ kind: "comment", node });
    } else if (node.nodeType === 1) {
      out.push({ kind: "element", node });
    }
  }
  return out;
}
function tokenizeText(text, out) {
  let i = 0;
  let buf = "";
  const len = text.length;
  while (i < len) {
    if (text[i] === "{") {
      const close = findMatchingClose(text, i);
      if (close === -1) {
        buf += text[i];
        i++;
        continue;
      }
      const inner = text.slice(i + 1, close).trim();
      if (buf.length > 0) {
        out.push({ kind: "text", value: buf });
        buf = "";
      }
      out.push(classifyBrace(inner, text.slice(i, close + 1)));
      i = close + 1;
      continue;
    }
    buf += text[i];
    i++;
  }
  if (buf.length > 0) out.push({ kind: "text", value: buf });
}
function findMatchingClose(text, start) {
  let depth = 0;
  let str = null;
  for (let i = start; i < text.length; i++) {
    const c = text[i];
    if (str) {
      if (c === "\\") {
        i++;
        continue;
      }
      if (c === str) str = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      str = c;
      continue;
    }
    if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}
function classifyBrace(inner, raw) {
  const loc = raw.length > 60 ? raw.slice(0, 57) + "\u2026" : raw;
  if (inner.startsWith("#if")) {
    return { kind: "open-if", src: inner.slice(3).trim(), loc };
  }
  if (inner.startsWith("#for")) {
    return { kind: "open-for", src: inner.slice(4).trim(), loc };
  }
  if (inner.startsWith("#match")) {
    return { kind: "open-match", src: inner.slice(6).trim(), loc };
  }
  if (inner.startsWith(":else if")) {
    return { kind: "else-if", src: inner.slice(8).trim(), loc };
  }
  if (inner === ":else") {
    return { kind: "else", loc };
  }
  if (inner.startsWith(":case")) {
    return { kind: "case", src: inner.slice(5).trim(), loc };
  }
  if (inner === ":default") {
    return { kind: "default", loc };
  }
  if (inner === "/if") return { kind: "close-if", loc };
  if (inner === "/for") return { kind: "close-for", loc };
  if (inner === "/match") return { kind: "close-match", loc };
  return { kind: "expr", src: inner, loc };
}

// packages/ship-set-dom/src/template/compile-blocks.ts
var IF_TERMS = ["else-if", "else", "close-if"];
var FOR_TERMS = ["else", "close-for"];
var MATCH_TERMS = ["case", "default", "close-match"];
function parseIf(tokens, start, scope) {
  const head = tokens[start];
  const branches = [];
  let else_;
  let cur = compileExpr(head.src, { location: head.loc });
  let i = start + 1;
  while (true) {
    const body = scope.body(tokens, i, IF_TERMS);
    branches.push({ cond: cur, body: body.instrs });
    i = body.end;
    const term = tokens[i];
    if (!term) {
      throw new Error(`[ship-set-dom] missing {/if} (at ${head.loc})`);
    }
    if (term.kind === "close-if") {
      i++;
      break;
    }
    if (term.kind === "else-if") {
      cur = compileExpr(term.src, { location: term.loc });
      i++;
      continue;
    }
    if (term.kind === "else") {
      i++;
      const elseBody = scope.body(tokens, i, ["close-if"]);
      else_ = elseBody.instrs;
      i = elseBody.end;
      const closer = tokens[i];
      if (!closer || closer.kind !== "close-if") {
        throw new Error(`[ship-set-dom] missing {/if} (at ${head.loc})`);
      }
      i++;
      break;
    }
  }
  const instr = else_ !== void 0 ? { kind: "if", branches, else_ } : { kind: "if", branches };
  return { instr, end: i };
}
function parseFor(tokens, start, scope) {
  const head = tokens[start];
  const { varName, indexName, iterable, key } = parseForHeader(
    head.src,
    head.loc
  );
  let i = start + 1;
  const body = scope.body(tokens, i, FOR_TERMS);
  i = body.end;
  let else_;
  const term = tokens[i];
  if (term?.kind === "else") {
    i++;
    const elseBody = scope.body(tokens, i, ["close-for"]);
    else_ = elseBody.instrs;
    i = elseBody.end;
  }
  const closer = tokens[i];
  if (!closer || closer.kind !== "close-for") {
    throw new Error(`[ship-set-dom] missing {/for} (at ${head.loc})`);
  }
  i++;
  const instr = {
    kind: "for",
    varName,
    indexName,
    iterable,
    key,
    body: body.instrs,
    ...else_ !== void 0 ? { else_ } : {}
  };
  return { instr, end: i };
}
function parseForHeader(src, loc) {
  const ofIdx = findKeywordIndex(src, " of ");
  if (ofIdx === -1) {
    throw new Error(`[ship-set-dom] {#for} expects "X of items" (at ${loc})`);
  }
  const left = src.slice(0, ofIdx).trim();
  let right = src.slice(ofIdx + 4).trim();
  let varName = left;
  let indexName = null;
  const parenMatch = left.match(
    /^\(\s*([A-Za-z_$][\w$]*)\s*,\s*([A-Za-z_$][\w$]*)\s*\)$/
  );
  if (parenMatch) {
    varName = parenMatch[1];
    indexName = parenMatch[2];
  }
  if (!/^[A-Za-z_$][\w$]*$/.test(varName)) {
    throw new Error(`[ship-set-dom] {#for} bad variable: ${left} (at ${loc})`);
  }
  let key = null;
  const keyMatch = right.match(/\skey=\{(.+)\}\s*$/s);
  if (keyMatch) {
    key = compileExpr(keyMatch[1], { location: `${loc} key=` });
    right = right.slice(0, keyMatch.index).trim();
  }
  const iterable = compileExpr(right, { location: `${loc} of ${right}` });
  return { varName, indexName, iterable, key };
}
function findKeywordIndex(s, keyword) {
  let str = null;
  let depth = 0;
  for (let i = 0; i <= s.length - keyword.length; i++) {
    const c = s[i];
    if (str) {
      if (c === "\\") {
        i++;
        continue;
      }
      if (c === str) str = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      str = c;
      continue;
    }
    if (c === "(" || c === "[" || c === "{") depth++;
    else if (c === ")" || c === "]" || c === "}") depth--;
    if (depth === 0 && s.slice(i, i + keyword.length) === keyword) return i;
  }
  return -1;
}
function parseMatch(tokens, start, scope) {
  const head = tokens[start];
  const discriminant = compileExpr(head.src, { location: head.loc });
  let i = start + 1;
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.kind === "text" && t.value.trim() === "") {
      i++;
      continue;
    }
    break;
  }
  const cases = [];
  let default_;
  while (i < tokens.length) {
    const t = tokens[i];
    if (t.kind === "close-match") {
      i++;
      break;
    }
    if (t.kind === "case") {
      const test = compileExpr(t.src, { location: t.loc });
      i++;
      const body = scope.body(tokens, i, MATCH_TERMS);
      cases.push({ test, body: body.instrs });
      i = body.end;
      continue;
    }
    if (t.kind === "default") {
      i++;
      const body = scope.body(tokens, i, ["close-match"]);
      default_ = body.instrs;
      i = body.end;
      continue;
    }
    throw new Error(
      `[ship-set-dom] unexpected token in {#match}: ${t.kind} (at ${head.loc})`
    );
  }
  const instr = {
    kind: "match",
    discriminant,
    cases,
    ...default_ !== void 0 ? { default_ } : {}
  };
  return { instr, end: i };
}

// packages/ship-set-dom/src/template/compile.ts
function compileChildren(parent) {
  const tokens = tokenizeChildren(parent);
  return parseTokens(tokens, 0).instrs;
}
var blockScope = {
  body: (tokens, start, terms) => parseTokens(tokens, start, terms)
};
function parseTokens(tokens, start, terminators = []) {
  const instrs = [];
  let i = start;
  while (i < tokens.length) {
    const tok = tokens[i];
    if (terminators.includes(tok.kind)) return { instrs, end: i };
    if (tok.kind === "text") {
      if (tok.value.length > 0) instrs.push({ kind: "static-text", text: tok.value });
      i++;
      continue;
    }
    if (tok.kind === "comment") {
      instrs.push({ kind: "comment", data: tok.node.data });
      i++;
      continue;
    }
    if (tok.kind === "expr") {
      instrs.push({
        kind: "dynamic-text",
        expr: compileExpr(tok.src, { location: tok.loc })
      });
      i++;
      continue;
    }
    if (tok.kind === "element") {
      instrs.push(compileElement(tok.node));
      i++;
      continue;
    }
    if (tok.kind === "open-if") {
      const { instr, end } = parseIf(tokens, i, blockScope);
      instrs.push(instr);
      i = end;
      continue;
    }
    if (tok.kind === "open-for") {
      const { instr, end } = parseFor(tokens, i, blockScope);
      instrs.push(instr);
      i = end;
      continue;
    }
    if (tok.kind === "open-match") {
      const { instr, end } = parseMatch(tokens, i, blockScope);
      instrs.push(instr);
      i = end;
      continue;
    }
    throw new Error(
      `[ship-set-dom] unexpected token: ${tok.kind} (at ${getLoc(tok)})`
    );
  }
  return { instrs, end: i };
}
var getLoc = (tok) => "loc" in tok && typeof tok.loc === "string" ? tok.loc : "?";
function compileElement(el) {
  const tag = el.tagName.toLowerCase();
  const locPrefix = `<${tag}>`;
  if (tag === "while-loading") {
    const bind = requiredAttr(el, "bind", "<while-loading>");
    return {
      kind: "element",
      tag: "while-loading",
      attrs: [],
      children: compileChildren(el),
      isWhileLoading: { bind }
    };
  }
  if (tag === "while-error") {
    const bind = requiredAttr(el, "bind", "<while-error>");
    let errorVar = "error";
    for (const attr of Array.from(el.attributes)) {
      if (attr.name.startsWith("let:")) {
        errorVar = attr.name.slice(4);
        break;
      }
    }
    return {
      kind: "element",
      tag: "while-error",
      attrs: [],
      children: compileChildren(el),
      isWhileError: { bind, errorVar }
    };
  }
  return {
    kind: "element",
    tag: el.tagName,
    // preserve original casing for the renderer
    attrs: compileAttrs(el, locPrefix),
    children: compileChildren(el)
  };
}
var requiredAttr = (el, name, label) => {
  const v = el.getAttribute(name);
  if (v === null) {
    throw new Error(`[ship-set-dom] ${label} missing required attribute: ${name}`);
  }
  return v;
};

// packages/ship-set-dom/src/template/render.ts
import { bindText, effect as effect3 } from "@ship-set/runtime";

// packages/ship-set-dom/src/template/render-attrs.ts
import {
  bindAttr,
  bindClass,
  bindValue,
  bindChecked,
  effect,
  on
} from "@ship-set/runtime";
var assertSignal = (v, label) => {
  if (typeof v !== "function") {
    throw new Error(`[ship-set-dom] ${label} requires a Signal`);
  }
};
function applyAttr(el, a, scope, disposers) {
  switch (a.kind) {
    case "static":
      el.setAttribute(a.name, a.value);
      return;
    case "attr":
      disposers.push(bindAttr(el, a.name, () => a.expr(scope)));
      return;
    case "attr-dynamic": {
      let prevName = null;
      const dispose = effect(() => {
        const nameVal = a.nameExpr(scope);
        const valueVal = a.expr(scope);
        const nm = nameVal == null ? null : String(nameVal);
        if (prevName !== null && prevName !== nm) el.removeAttribute(prevName);
        if (nm !== null) {
          if (valueVal == null || valueVal === false || valueVal === "") {
            el.removeAttribute(nm);
          } else {
            el.setAttribute(nm, valueVal === true ? "" : String(valueVal));
          }
        }
        prevName = nm;
      });
      disposers.push(dispose);
      return;
    }
    case "bind-value": {
      const sig = a.expr(scope);
      assertSignal(sig, "bind:value");
      disposers.push(
        bindValue(
          el,
          sig
        )
      );
      return;
    }
    case "bind-checked": {
      const sig = a.expr(scope);
      assertSignal(sig, "bind:checked");
      disposers.push(bindChecked(el, sig));
      return;
    }
    case "on": {
      const handler = a.handler(scope);
      if (typeof handler !== "function") {
        throw new Error(
          `[ship-set-dom] on:${a.event} handler is not a function`
        );
      }
      const opts = a.modifiers.length > 0 ? { modifiers: a.modifiers } : void 0;
      disposers.push(
        on(el, a.event, handler, opts ?? {})
      );
      return;
    }
    case "class":
      disposers.push(bindClass(el, a.name, () => a.expr(scope)));
      return;
    case "style": {
      const dispose = effect(() => {
        const v = a.expr(scope);
        const html = el;
        if (v == null || v === false || v === "") {
          html.style.removeProperty(a.name);
        } else {
          html.style.setProperty(a.name, String(v));
        }
      });
      disposers.push(dispose);
      return;
    }
  }
}

// packages/ship-set-dom/src/template/render-for.ts
import {
  bindFor,
  effect as effect2
} from "@ship-set/runtime";
function renderFor(instr, scope, doc) {
  const anchor = doc.createComment("for");
  return { nodes: [anchor], mount: () => mountFor(anchor, instr, scope, doc) };
}
function mountFor(anchor, instr, scope, doc) {
  let elseActive = null;
  const tearDownElse = () => {
    if (!elseActive) return;
    for (const n of elseActive.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
    elseActive.cleanup();
    elseActive = null;
  };
  const itemsThunk = () => {
    let v;
    try {
      v = instr.iterable(scope);
    } catch {
      return [];
    }
    if (Array.isArray(v)) return v;
    if (v == null) return [];
    if (typeof v.length === "number") {
      return Array.from(v);
    }
    return [];
  };
  const wrappedRender = (item, i) => {
    const additions = { [instr.varName]: item };
    if (instr.indexName) additions[instr.indexName] = i;
    const childScope = extendScope(scope, additions);
    const r = render(instr.body, childScope, doc);
    const cleanup = r.mount();
    return { nodes: r.nodes, cleanup };
  };
  const dispose = bindFor({
    anchor,
    items: () => itemsThunk(),
    key: (item, i) => instr.key ? instr.key(
      extendScope(scope, {
        [instr.varName]: item,
        ...instr.indexName ? { [instr.indexName]: i } : {}
      })
    ) : i,
    render: wrappedRender
  });
  const elseDispose = instr.else_ ? effect2(() => {
    const empty = itemsThunk().length === 0;
    if (empty && !elseActive) {
      const r = render(instr.else_, scope, doc);
      const parent = anchor.parentNode;
      if (parent) {
        for (const n of r.nodes) parent.insertBefore(n, anchor);
      }
      const cleanup = r.mount();
      elseActive = { nodes: r.nodes, cleanup };
    } else if (!empty && elseActive) {
      tearDownElse();
    }
  }) : null;
  return () => {
    dispose();
    elseDispose?.();
    tearDownElse();
  };
}

// packages/ship-set-dom/src/template/render-while.ts
import {
  bindWhileLoading,
  bindWhileError
} from "@ship-set/runtime";
function renderWhileLoading(bindName, body, scope, doc) {
  const anchor = doc.createComment("while-loading");
  return {
    nodes: [anchor],
    mount: () => {
      const sig = scope[bindName];
      if (typeof sig !== "function") {
        throw new Error(
          `[ship-set-dom] <while-loading bind="${bindName}">: scope value is not a signal`
        );
      }
      return bindWhileLoading({
        anchor,
        signal: sig,
        render: () => {
          const r = render(body, scope, doc);
          const childCleanup = r.mount();
          return { nodes: r.nodes, cleanup: childCleanup };
        }
      });
    }
  };
}
function renderWhileError(decl, body, scope, doc) {
  const anchor = doc.createComment("while-error");
  return {
    nodes: [anchor],
    mount: () => {
      const sig = scope[decl.bind];
      if (typeof sig !== "function") {
        throw new Error(
          `[ship-set-dom] <while-error bind="${decl.bind}">: scope value is not a signal`
        );
      }
      return bindWhileError({
        anchor,
        signal: sig,
        render: (error) => {
          const child = extendScope(scope, { [decl.errorVar]: error });
          const r = render(body, child, doc);
          const childCleanup = r.mount();
          return { nodes: r.nodes, cleanup: childCleanup };
        }
      });
    }
  };
}

// packages/ship-set-dom/src/template/render.ts
var extendScope = (parent, additions) => Object.assign(Object.create(parent), additions);
var composeDisposers = (ds) => () => {
  for (const d of ds) {
    try {
      d();
    } catch (e) {
      console.error("[ship-set-dom] cleanup threw:", e);
    }
  }
};
function render(instrs, scope, doc) {
  const nodes = [];
  const mounts = [];
  for (const instr of instrs) {
    const r = renderOne(instr, scope, doc);
    for (const n of r.nodes) nodes.push(n);
    mounts.push(r.mount);
  }
  return {
    nodes,
    mount: () => composeDisposers(mounts.map((m) => m()))
  };
}
var NO_MOUNT = () => () => {
};
function renderOne(instr, scope, doc) {
  switch (instr.kind) {
    case "static-text":
      return { nodes: [doc.createTextNode(instr.text)], mount: NO_MOUNT };
    case "comment":
      return { nodes: [doc.createComment(instr.data)], mount: NO_MOUNT };
    case "dynamic-text": {
      const t = doc.createTextNode("");
      return {
        nodes: [t],
        mount: () => bindText(t, () => instr.expr(scope))
      };
    }
    case "element":
      return renderElement(instr, scope, doc);
    case "if":
      return renderIf(instr, scope, doc);
    case "for":
      return renderFor(instr, scope, doc);
    case "match":
      return renderMatch(instr, scope, doc);
  }
}
function renderElement(instr, scope, doc) {
  if (instr.isWhileLoading) {
    return renderWhileLoading(
      instr.isWhileLoading.bind,
      instr.children,
      scope,
      doc
    );
  }
  if (instr.isWhileError) {
    return renderWhileError(instr.isWhileError, instr.children, scope, doc);
  }
  const el = doc.createElement(instr.tag);
  const childRender = render(instr.children, scope, doc);
  for (const c of childRender.nodes) el.appendChild(c);
  return {
    nodes: [el],
    mount: () => {
      const disposers = [];
      for (const a of instr.attrs) applyAttr(el, a, scope, disposers);
      disposers.push(childRender.mount());
      return composeDisposers(disposers);
    }
  };
}
function renderIf(instr, scope, doc) {
  const anchor = doc.createComment("if");
  const all = instr.branches.map(
    (b) => ({
      cond: () => {
        try {
          return Boolean(b.cond(scope));
        } catch {
          return false;
        }
      },
      body: b.body
    })
  );
  return {
    nodes: [anchor],
    mount: () => mountChainedIf(anchor, all, instr.else_, scope, doc)
  };
}
function mountChainedIf(anchor, branches, else_, scope, doc) {
  let active = null;
  let lastIdx = -2;
  const tearDown = () => {
    if (!active) return;
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
    active.cleanup();
    active = null;
  };
  const dispose = effect3(() => {
    let pickIdx = -1;
    for (let i = 0; i < branches.length; i++) {
      if (branches[i].cond()) {
        pickIdx = i;
        break;
      }
    }
    if (pickIdx === lastIdx) return;
    tearDown();
    lastIdx = pickIdx;
    const body = pickIdx === -1 ? else_ ?? [] : branches[pickIdx].body;
    if (body.length === 0) {
      active = { nodes: [], cleanup: () => {
      } };
      return;
    }
    const r = render(body, scope, doc);
    const parent = anchor.parentNode;
    if (parent) {
      for (const n of r.nodes) parent.insertBefore(n, anchor);
    }
    const cleanup = r.mount();
    active = { nodes: r.nodes, cleanup };
  });
  return () => {
    dispose();
    tearDown();
  };
}
function renderMatch(instr, scope, doc) {
  const branches = instr.cases.map((c) => ({
    cond: () => {
      try {
        return instr.discriminant(scope) === c.test(scope);
      } catch {
        return false;
      }
    },
    body: c.body
  }));
  const anchor = doc.createComment("match");
  return {
    nodes: [anchor],
    mount: () => mountChainedIf(anchor, branches, instr.default_, scope, doc)
  };
}

// packages/ship-set-dom/src/directives/data.ts
import { live } from "@ship-set/runtime";
function mountData(decl, ship, route) {
  if (decl.live) {
    return live(() => decl.source(ship, route));
  }
  return decl.source(ship, route);
}

// packages/ship-set-dom/src/directives/state.ts
import { state } from "@ship-set/runtime";
function mountState(decl, scope) {
  const initial = decl.value(scope);
  return state(initial);
}

// packages/ship-set-dom/src/directives/component.ts
import { component, prop } from "@ship-set/runtime";
var camelOf = (s) => s.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
var buildPropDef = (decl) => {
  switch (decl.type) {
    case "union":
      return prop.union(decl.values ?? [], {
        ...decl.default !== void 0 ? { default: decl.default } : {},
        reflect: decl.reflect,
        bindable: decl.bindable
      });
    case "bool":
      return prop.bool({
        ...decl.default !== void 0 ? { default: Boolean(decl.default) } : {},
        reflect: decl.reflect,
        bindable: decl.bindable
      });
    case "number":
      return prop.number({
        ...decl.default !== void 0 ? { default: Number(decl.default) } : {},
        reflect: decl.reflect,
        bindable: decl.bindable
      });
    case "string":
      return prop.string({
        ...decl.default !== void 0 ? { default: String(decl.default) } : {},
        reflect: decl.reflect,
        bindable: decl.bindable
      });
  }
};
function registerComponent(decl, body, doc, userSetup) {
  const props = {};
  for (const p of decl.props) props[p.name] = buildPropDef(p);
  component(decl.tag, {
    props,
    setup: ({ props: propSignals, host, cleanups }) => {
      const scope = {};
      for (const [attr, sig] of Object.entries(propSignals)) {
        scope[attr] = sig;
        const camel = camelOf(attr);
        if (camel !== attr) scope[camel] = sig;
      }
      let extras;
      if (userSetup) {
        const ret = userSetup.call(void 0);
        if (ret && typeof ret === "object") {
          extras = ret;
        }
      }
      const finalScope = extras ? extendScope(scope, extras) : scope;
      const r = render(body, finalScope, doc);
      for (const n of r.nodes) host.appendChild(n);
      const cleanup = r.mount();
      cleanups.push(() => {
        for (const n of r.nodes) {
          try {
            n.parentNode?.removeChild(n);
          } catch {
          }
        }
        cleanup();
      });
    }
  });
}

// packages/ship-set-dom/src/hydrate.ts
function hydrate(template, setup, opts) {
  if (!opts || !opts.container) {
    throw new Error("[ship-set-dom] hydrate: opts.container is required");
  }
  const parsed = parseSetSource(template);
  const route = opts.route ?? {};
  const ship = opts.ship;
  const navigate = opts.navigate ?? (() => {
  });
  if (parsed.component) {
    const bodyInstrs = compileTopLevel(parsed.templateNodes);
    registerComponent(
      parsed.component,
      bodyInstrs,
      opts.container.ownerDocument,
      setup
    );
    return { dispose: () => {
    } };
  }
  let scope = { ship, route, navigate };
  for (const decl of parsed.data) {
    scope[decl.name] = mountData(decl, ship, route);
  }
  for (const decl of parsed.state) {
    scope[decl.name] = mountState(decl, scope);
  }
  if (setup) {
    const ctx = { ship, route, navigate };
    const ret = setup(ctx);
    if (ret && typeof ret === "object") {
      scope = extendScope(scope, ret);
    }
  }
  const instrs = compileTopLevel(parsed.templateNodes);
  const r = render(instrs, scope, opts.container.ownerDocument);
  for (const n of r.nodes) opts.container.appendChild(n);
  const cleanup = r.mount();
  return {
    dispose: () => {
      for (const n of r.nodes) {
        try {
          n.parentNode?.removeChild(n);
        } catch {
        }
      }
      cleanup();
    }
  };
}
function compileTopLevel(nodes) {
  if (nodes.length === 0) return [];
  const owner = nodes[0].ownerDocument;
  if (!owner) return [];
  const fragment = owner.createElement("div");
  for (const n of nodes) fragment.appendChild(n);
  return compileChildren(fragment);
}

// packages/ship-set-dom/src/setup-loader.ts
async function loadSetup(setUrl, loader = (p) => import(
  /* @vite-ignore */
  p
)) {
  const tsUrl = pairedTsPath(setUrl);
  let mod;
  try {
    mod = await loader(tsUrl);
  } catch {
    return void 0;
  }
  const def = mod?.default;
  if (typeof def === "function") return def;
  if (def && typeof def === "object" && "setup" in def) {
    const setup = def.setup;
    if (typeof setup === "function") return setup;
  }
  return void 0;
}
function pairedTsPath(setUrl) {
  if (setUrl.endsWith(".html.set")) {
    return setUrl.slice(0, -".html.set".length) + ".ts";
  }
  if (setUrl.endsWith(".set")) {
    return setUrl.slice(0, -".set".length) + ".ts";
  }
  return setUrl + ".ts";
}
export {
  hydrate,
  loadSetup,
  pairedTsPath,
  parseSetSource
};
