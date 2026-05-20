import { ExtensionContext, window, workspace } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(_context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration('sui');
  // The server is the native `sui-lsp` Rust binary (one grammar, shared with
  // the compiler + agent tooling). `sui.serverPath` overrides the location;
  // otherwise we rely on `sui-lsp` being on PATH (`just install-dev`).
  const command = config.get<string>('serverPath')?.trim() || 'sui-lsp';
  const serverOptions: ServerOptions = {
    run: { command, transport: TransportKind.stdio },
    debug: { command, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'sui' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.sui'),
    },
    outputChannel: window.createOutputChannel('SUI LSP'),
  };

  client = new LanguageClient('sui-lsp', 'SUI LSP', serverOptions, clientOptions);

  try {
    await client.start();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    window.showErrorMessage(
      `Failed to start SUI LSP (\`${command}\`). Install it with \`just install-dev\` or set "sui.serverPath". ${message}`,
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
