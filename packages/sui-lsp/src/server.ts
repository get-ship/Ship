// LSP server entry — connects vscode-languageserver to the
// `.sui` analyzers in this package.

import {
  CompletionItem as LspCompletionItem,
  CompletionItemKind,
  Diagnostic as LspDiagnostic,
  DiagnosticSeverity,
  Hover as LspHover,
  MarkupKind,
  ProposedFeatures,
  TextDocumentSyncKind,
  TextDocuments,
  createConnection,
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';
import * as url from 'node:url';

import { parse, type Node, type ParseError, type ParseResult, type Range } from './parse';
import { diagnose, type Diagnostic } from './diagnostics';
import { complete, type CompletionItem } from './completions';
import { hover } from './hover';
import { loadSetup, pairedSetupPath, type SetupInfo } from './setup';
import { diagnoseSetup } from './setup-diagnostics';
import * as fs from 'node:fs';

interface Cached {
  parsed: ParseResult;
  setup: SetupInfo | null;
  setupLoadedFor: string;
}
const cache = new Map<string, Cached>();

export function startServer() {
  const connection = createConnection(ProposedFeatures.all);
  const documents = new TextDocuments(TextDocument);

  connection.onInitialize(() => ({
    capabilities: {
      textDocumentSync: TextDocumentSyncKind.Incremental,
      completionProvider: {
        resolveProvider: false,
        triggerCharacters: ['<', ':', '|', '{', '.'],
      },
      hoverProvider: true,
    },
  }));

  documents.onDidChangeContent(change => {
    const doc = change.document;
    refresh(doc);
    const cached = cache.get(doc.uri)!;
    const diags = diagnose(doc.getText(), cached.parsed.nodes, cached.parsed.errors, cached.setup);
    void connection.sendDiagnostics({
      uri: doc.uri,
      diagnostics: diags.map(d => toLsp(d, doc)),
    });
    publishSetupDiagnostics(connection, doc, cached.setup);
  });

  connection.onCompletion(params => {
    const doc = documents.get(params.textDocument.uri);
    if (!doc) return [];
    const cached = ensureFresh(doc);
    const items = complete({
      source: doc.getText(),
      offset: doc.offsetAt(params.position),
      nodes: cached.parsed.nodes,
      setup: cached.setup,
    });
    return items.map(toLspCompletion);
  });

  connection.onHover(params => {
    const doc = documents.get(params.textDocument.uri);
    if (!doc) return null;
    const cached = ensureFresh(doc);
    const h = hover(doc.getText(), doc.offsetAt(params.position), cached.parsed.nodes, cached.setup);
    if (!h) return null;
    const lsp: LspHover = {
      contents: { kind: MarkupKind.Markdown, value: h.contents },
      range: { start: doc.positionAt(h.range.start), end: doc.positionAt(h.range.end) },
    };
    return lsp;
  });

  documents.listen(connection);
  connection.listen();
  return { connection, documents };
}

function refresh(doc: TextDocument): void {
  const fsPath = uriToFsPath(doc.uri);
  const parsed = parse(doc.getText());
  const setup = fsPath ? loadSetup(fsPath) : null;
  cache.set(doc.uri, { parsed, setup, setupLoadedFor: fsPath ?? '' });
}

function ensureFresh(doc: TextDocument): Cached {
  let c = cache.get(doc.uri);
  if (!c) { refresh(doc); c = cache.get(doc.uri)!; }
  return c;
}

function uriToFsPath(uri: string): string | null {
  if (!uri.startsWith('file://')) return null;
  try { return url.fileURLToPath(uri); } catch { return null; }
}

function publishSetupDiagnostics(
  connection: ReturnType<typeof createConnection>,
  doc: TextDocument,
  setup: SetupInfo | null,
): void {
  const fsPath = uriToFsPath(doc.uri);
  if (!fsPath) return;
  const setupPath = pairedSetupPath(fsPath);
  if (!setupPath) return;
  const setupSource = setup?.source ?? safeRead(setupPath);
  if (setupSource == null) return;
  const diags = diagnoseSetup(setupPath, setupSource);
  const setupUri = url.pathToFileURL(setupPath).href;
  const setupDoc = TextDocument.create(setupUri, 'typescript', 0, setupSource);
  void connection.sendDiagnostics({
    uri: setupUri,
    diagnostics: diags.map(d => toLsp(d, setupDoc)),
  });
}

function safeRead(p: string): string | null {
  try { return fs.readFileSync(p, 'utf8'); } catch { return null; }
}

function toLsp(d: Diagnostic, doc: TextDocument): LspDiagnostic {
  return {
    range: { start: doc.positionAt(d.range.start), end: doc.positionAt(d.range.end) },
    message: d.message,
    severity:
      d.severity === 'error' ? DiagnosticSeverity.Error :
      d.severity === 'warning' ? DiagnosticSeverity.Warning :
      DiagnosticSeverity.Information,
    source: d.source,
    code: d.code,
  };
}

function toLspCompletion(c: CompletionItem): LspCompletionItem {
  return {
    label: c.label,
    kind: lspKind(c.kind),
    detail: c.detail,
    insertText: c.insertText,
  };
}

function lspKind(k: CompletionItem['kind']): CompletionItemKind {
  switch (k) {
    case 'tag': return CompletionItemKind.Class;
    case 'directive': return CompletionItemKind.Keyword;
    case 'attr-local': return CompletionItemKind.Property;
    case 'attr-name': return CompletionItemKind.Property;
    case 'event-modifier': return CompletionItemKind.EnumMember;
    case 'identifier': return CompletionItemKind.Variable;
  }
}

// Helpers exported for tests / programmatic invocation.
export function analyze(uri: string, source: string): {
  parsed: ParseResult;
  diagnostics: Diagnostic[];
  setup: SetupInfo | null;
} {
  const fsPath = uriToFsPath(uri);
  const parsed = parse(source);
  const setup = fsPath ? loadSetup(fsPath) : null;
  const diagnostics = diagnose(source, parsed.nodes, parsed.errors, setup);
  return { parsed, diagnostics, setup };
}

export function analyzeWithSetup(
  source: string, setup: SetupInfo | null,
): { parsed: ParseResult; diagnostics: Diagnostic[] } {
  const parsed = parse(source);
  const diagnostics = diagnose(source, parsed.nodes, parsed.errors, setup);
  return { parsed, diagnostics };
}

// Re-exports for tests / consumers.
export { parse, diagnose, complete, hover, loadSetup, diagnoseSetup };
export type { Node, ParseError, ParseResult, Range, Diagnostic, CompletionItem, SetupInfo };

if (require.main === module) {
  startServer();
}
