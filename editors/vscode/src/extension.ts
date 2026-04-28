import { ExtensionContext, window, workspace } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(_context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration('sail');
  const serverPath = config.get<string>('serverPath') ?? 'sail-lsp';

  const serverOptions: ServerOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'set' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.set'),
    },
    outputChannel: window.createOutputChannel('Sail LSP'),
  };

  client = new LanguageClient('sail-lsp', 'Sail LSP', serverOptions, clientOptions);

  try {
    await client.start();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    window.showErrorMessage(
      `Failed to start sail-lsp ("${serverPath}"). ` +
        `Set "sail.serverPath" or install sail-lsp on PATH. ${message}`,
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
