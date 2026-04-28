import { ExtensionContext, window, workspace } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(_context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration('ship-set');
  const serverPath = config.get<string>('serverPath') ?? 'ship-set-lsp';

  const serverOptions: ServerOptions = {
    run: { command: serverPath, transport: TransportKind.stdio },
    debug: { command: serverPath, transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'set' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.set'),
    },
    outputChannel: window.createOutputChannel('ship-set LSP'),
  };

  client = new LanguageClient('ship-set-lsp', 'ship-set LSP', serverOptions, clientOptions);

  try {
    await client.start();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    window.showErrorMessage(
      `Failed to start ship-set-lsp ("${serverPath}"). ` +
        `Set "ship-set.serverPath" or install ship-set-lsp on PATH. ${message}`,
    );
  }
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
