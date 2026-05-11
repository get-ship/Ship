import * as path from 'path';
import { ExtensionContext, window, workspace } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export async function activate(context: ExtensionContext): Promise<void> {
  const config = workspace.getConfiguration('sui');
  const serverPath = config.get<string>('serverPath') ?? '';

  const serverOptions: ServerOptions = serverPath
    ? {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio },
      }
    : nodeServer(context);

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
      `Failed to start SUI LSP. Set "sui.serverPath" to override. ${message}`,
    );
  }
}

function nodeServer(context: ExtensionContext): ServerOptions {
  const module = context.asAbsolutePath(
    path.join('..', '..', 'packages', 'ui-lsp', 'dist', 'server.js'),
  );
  return {
    run: { module, transport: TransportKind.ipc },
    debug: {
      module,
      transport: TransportKind.ipc,
      options: { execArgv: ['--nolazy', '--inspect=6009'] },
    },
  };
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
