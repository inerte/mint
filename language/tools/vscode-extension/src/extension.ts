/**
 * VS Code Extension for Sigil Language
 *
 * Activates the Sigil Language Server when opening .sigil files.
 * Provides syntax highlighting, diagnostics, hover, completion, and more.
 */

import * as path from 'path';
import { workspace, ExtensionContext, window } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

/**
 * Extension activation
 * Called when a .sigil file is opened
 */
export function activate(context: ExtensionContext) {
  console.log('Sigil language extension is now active');

  // Get LSP server path
  const serverModule = getServerPath(context);

  if (!serverModule) {
    window.showErrorMessage(
      'Sigil Language Server not found. Please install or configure sigil.lsp.path in settings.'
    );
    return;
  }

  // Server options - run the LSP server
  const serverOptions: ServerOptions = {
    run: {
      module: serverModule,
      transport: TransportKind.stdio,
      options: {
        env: process.env,
      },
    },
    debug: {
      module: serverModule,
      transport: TransportKind.stdio,
      options: {
        env: process.env,
        execArgv: ['--nolazy', '--inspect=6009'],
      },
    },
  };

  // Client options - configure language client
  const clientOptions: LanguageClientOptions = {
    // Register for .sigil files
    documentSelector: [
      {
        scheme: 'file',
        language: 'sigil',
      },
    ],

    // Synchronize file events
    synchronize: {
      // Watch .sigil and .sigil.map files
      fileEvents: workspace.createFileSystemWatcher('**/*.{sigil,sigil.map}'),
    },

    // Output channel for debugging
    outputChannelName: 'Sigil Language Server',
  };

  // Create and start the language client
  client = new LanguageClient(
    'sigilLanguageServer',
    'Sigil Language Server',
    serverOptions,
    clientOptions
  );

  // Start the client (this will also launch the server)
  client.start();

  console.log('Sigil Language Server started');
}

/**
 * Extension deactivation
 * Clean up resources
 */
export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

/**
 * Get the path to the LSP server
 * Checks custom path first, then bundled server
 */
function getServerPath(context: ExtensionContext): string | null {
  // Check for custom path in settings
  const config = workspace.getConfiguration('sigil');
  const customPath = config.get<string>('lsp.path');

  if (customPath) {
    return customPath;
  }

  // Use bundled server (relative to extension root)
  // In development: ../lsp/dist/server.js
  // In packaged extension: bundled in extension
  const bundledPath = context.asAbsolutePath(
    path.join('..', 'lsp', 'dist', 'server.js')
  );

  try {
    // Check if bundled server exists
    require.resolve(bundledPath);
    return bundledPath;
  } catch {
    // Server not found
    return null;
  }
}
