import { workspace, ExtensionContext, window } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext): void {
  const config = workspace.getConfiguration("ship.lsp");
  if (!config.get<boolean>("enable", true)) {
    return;
  }
  const command = config.get<string>("path", "ship");

  const serverOptions: ServerOptions = {
    run: { command, args: ["lsp"], transport: TransportKind.stdio },
    debug: { command, args: ["lsp"], transport: TransportKind.stdio },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", pattern: "**/.ship/views/**/*.html.j2" },
    ],
    diagnosticCollectionName: "ship-lint",
    outputChannel: window.createOutputChannel("Ship LSP"),
  };

  client = new LanguageClient(
    "ship-lsp",
    "Ship Lint",
    serverOptions,
    clientOptions,
  );
  client.start();
  context.subscriptions.push({ dispose: () => client?.stop() });
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
