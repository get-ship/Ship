// Hover responses for `.sui`.
//
// Phase 5 surface area:
//   • identifier inside `{…}` → resolves to declaration source +
//     setup-return type (paired `.ts` analysis from `setup.ts`)
//   • directive element tag (`<data>`, `<state>`, …) → contract
//     reminder ("requires `name` + `source`")
//   • primitive tag (`<ship-button>`) → CEM-driven prop list TODO
//     (Phase 6 emits CEM for ship-set primitives)

import type { Node, Range } from './parse-types';
import { buildScope, VIEW_GLOBALS } from './bindings';
import type { SetupInfo, SetupMember, UseNamespace, UseNamespaceMember } from './setup';

export interface Hover {
  range: Range;
  contents: string;
}

const DIRECTIVE_DOCS: Record<string, string> = {
  data: 'directive `<data name="…" source={…}>` — declares a data binding from the SDK.',
  state: 'directive `<state name="…" value={…}>` — declares a local state signal.',
  'while-loading': 'block `{#while-loading of …}…{/while-loading}` — shown while a `<data>` binding is pending.',
  'while-error': 'block `{#while-error err of …}…{/while-error}` — shown when a `<data>` binding errors.',
  component: 'directive `<component name="…">` — declares a sub-component contract.',
  prop: 'directive `<prop name="…" type="…">` — declares a component prop.',
  slot: 'directive `<slot name="…">` — opens a named slot.',
};

export function hover(
  source: string, offset: number, nodes: Node[], setup: SetupInfo | null,
): Hover | null {
  // Tag hover
  for (const n of nodes) {
    if (n.kind !== 'element-open') continue;
    if (offset < n.tagRange.start || offset > n.tagRange.end) continue;
    const doc = DIRECTIVE_DOCS[n.tag];
    if (doc) return { range: n.tagRange, contents: doc };
    if (n.tag.startsWith('ship-')) {
      return {
        range: n.tagRange,
        contents: `**${n.tag}**\n\nship-set primitive. Prop list comes from CEM (Phase 6 — TODO).`,
      };
    }
    return { range: n.tagRange, contents: `<${n.tag}>` };
  }
  // Identifier hover (inside an expression)
  return identifierHover(source, offset, nodes, setup);
}

function identifierHover(
  source: string, offset: number, nodes: Node[], setup: SetupInfo | null,
): Hover | null {
  const word = wordAt(source, offset);
  if (!word) return null;
  const scope = buildScope(nodes, source);
  // Namespace member hover (D-2): cursor sits on `filtered` in
  // `list.filtered`. Match the reference's `member` range, not the
  // bare `name` range.
  for (const r of scope.references) {
    if (!r.member) continue;
    if (offset < r.member.range.start || offset > r.member.range.end) continue;
    if (r.member.name !== word.text) continue;
    const ns = setup?.namespaces?.find(n => n.alias === r.name);
    const m = ns?.members.find(mm => mm.name === r.member!.name);
    if (ns && m) return { range: word.range, contents: formatNamespaceMember(ns, m) };
  }
  for (const r of scope.references) {
    if (offset < r.range.start || offset > r.range.end) continue;
    if (r.name !== word.text) continue;
    const decl = scope.declarations.find(d => d.name === r.name && offset >= d.extent.start && offset <= d.extent.end);
    if (decl) {
      return {
        range: word.range,
        contents: formatDecl(r.name, decl.source),
      };
    }
    const handler = setup?.onHandlers?.find(h => h.name === r.name);
    if (handler) {
      return { range: word.range, contents: formatHandler(handler.name, handler.signature) };
    }
    const ns = setup?.namespaces?.find(n => n.alias === r.name);
    if (ns) return { range: word.range, contents: formatNamespace(ns) };
    const member = setup?.members.find(m => m.name === r.name);
    if (member) return { range: word.range, contents: formatSetupMember(member) };
    if (VIEW_GLOBALS.has(r.name)) return { range: word.range, contents: `**${r.name}** — view global` };
    return { range: word.range, contents: `**${r.name}** — unresolved` };
  }
  return null;
}

function formatNamespaceMember(ns: UseNamespace, m: UseNamespaceMember): string {
  const factory = ns.hookFactory ? ` (from \`${ns.hookFactory}\`)` : '';
  return [
    '```ts',
    `${ns.alias}.${m.name}: ${m.signature}`,
    '```',
    '',
    `*\`${m.kind}\` contribution${factory}*`,
  ].join('\n');
}

function formatNamespace(ns: UseNamespace): string {
  const list = ns.members.map(m => `- \`${m.name}\` — ${m.kind}`).join('\n');
  const factory = ns.hookFactory ? ` (from \`${ns.hookFactory}\`)` : '';
  return [`**${ns.alias}** — \`use:\` namespace${factory}`, '', list || '*(no contributions)*'].join('\n');
}

function formatHandler(name: string, sig: string): string {
  return ['```ts', `${name}: ${sig}`, '```', '', '*from controller `on:` section*'].join('\n');
}

function formatSetupMember(m: SetupMember): string {
  return ['```ts', `${m.name}: ${m.type}`, '```', '', '*from paired setup `.ts`*'].join('\n');
}

function formatDecl(name: string, src: string): string {
  return `**${name}** — declared by \`<${src}>\` directive`;
}

function wordAt(source: string, offset: number): { text: string; range: Range } | null {
  const idChar = (c: string | undefined) => !!c && /[a-zA-Z0-9_$]/.test(c);
  let s = offset;
  while (s > 0 && idChar(source[s - 1])) s--;
  let e = offset;
  while (e < source.length && idChar(source[e])) e++;
  if (s === e) return null;
  return { text: source.slice(s, e), range: { start: s, end: e } };
}
