import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

let diagnosticCollection: vscode.DiagnosticCollection;

export function activate(context: vscode.ExtensionContext) {
    diagnosticCollection = vscode.languages.createDiagnosticCollection('icvs');
    context.subscriptions.push(diagnosticCollection);

    context.subscriptions.push(
        vscode.commands.registerCommand('icvs.validate', () => validateActiveDocument())
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('icvs.showGraph', () => showGraphPreview(context))
    );

    context.subscriptions.push(
        vscode.workspace.onDidSaveTextDocument((doc) => {
            if (doc.languageId === 'icvs') validateDocument(doc);
        })
    );

    context.subscriptions.push(
        vscode.workspace.onDidOpenTextDocument((doc) => {
            if (doc.languageId === 'icvs') validateDocument(doc);
        })
    );

    if (vscode.window.activeTextEditor?.document.languageId === 'icvs') {
        validateDocument(vscode.window.activeTextEditor.document);
    }

    console.log('InstructCanvas extension activated');
}

export function deactivate() {
    diagnosticCollection?.dispose();
}

// ── Validation ──

async function validateActiveDocument() {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;
    await validateDocument(editor.document);
}

async function validateDocument(document: vscode.TextDocument) {
    const diagnostics: vscode.Diagnostic[] = [];
    const text = document.getText();
    const lines = text.split('\n');

    for (let i = 0; i < lines.length; i++) {
        const trimmed = lines[i].trim();
        if (trimmed === '' || trimmed.startsWith('#')) continue;

        if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
            if (!/^\[(include|node|edge|target|project):/.test(trimmed)) {
                diagnostics.push(diagnostic(i, lines[i], 'Unknown block type'));
            }
            if (!trimmed.slice(1, -1).includes(':')) {
                diagnostics.push(diagnostic(i, lines[i], "Block must use 'keyword: arg' format"));
            }
            continue;
        }

        const indent = lines[i].search(/\S/);
        if (indent === 0 && trimmed.length > 0) {
            diagnostics.push(diagnostic(i, lines[i], 'Unexpected top-level text. Content must be inside [...]'));
        }
    }

    diagnosticCollection.set(document.uri, diagnostics);
}

function diagnostic(line: number, text: string, msg: string): vscode.Diagnostic {
    const range = new vscode.Range(line, 0, line, text.length);
    const d = new vscode.Diagnostic(range, msg, vscode.DiagnosticSeverity.Error);
    d.source = 'icvs';
    return d;
}

// ── Graph Preview ──

let previewPanel: vscode.WebviewPanel | undefined;

async function showGraphPreview(context: vscode.ExtensionContext) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) return;

    if (previewPanel) {
        previewPanel.reveal(vscode.ViewColumn.Beside);
        previewPanel.webview.postMessage({
            type: 'sync',
            source: editor.document.getText()
        });
        return;
    }

    previewPanel = vscode.window.createWebviewPanel(
        'icvsGraphPreview',
        'InstructCanvas Graph',
        vscode.ViewColumn.Beside,
        {
            enableScripts: true,
            retainContextWhenHidden: true,
            localResourceRoots: [
                vscode.Uri.file(path.join(context.extensionPath, '..', 'wasm', 'pkg')),
                vscode.Uri.file(context.extensionPath),
            ]
        }
    );

    previewPanel.onDidDispose(() => { previewPanel = undefined; });

    previewPanel.webview.onDidReceiveMessage(async (msg) => {
        switch (msg.type) {
            case 'ready':
                previewPanel?.webview.postMessage({
                    type: 'sync',
                    source: editor.document.getText()
                });
                break;
            case 'updateSource':
                replaceEditorContent(editor.document, msg.source);
                break;
            case 'exportMarkdown':
                vscode.env.clipboard.writeText(msg.content);
                vscode.window.showInformationMessage('Markdown copied to clipboard');
                break;
            case 'error':
                console.error('Preview error:', msg.message);
                break;
        }
    });

    const htmlPath = path.join(context.extensionPath, '..', 'preview', 'index.html');
    const wasmDir = vscode.Uri.file(
        path.join(context.extensionPath, '..', 'wasm', 'pkg')
    );

    let html = fs.readFileSync(htmlPath, 'utf-8');

    const wasmUri = previewPanel.webview.asWebviewUri(
        vscode.Uri.joinPath(wasmDir, 'icvs_wasm.js')
    );
    const bgWasmUri = previewPanel.webview.asWebviewUri(
        vscode.Uri.joinPath(wasmDir, 'icvs_wasm_bg.wasm')
    );

    html = html.replace('../wasm/pkg/icvs_wasm.js', wasmUri.toString());
    html = html.replace('../wasm/pkg/icvs_wasm_bg.wasm', bgWasmUri.toString());

    html = html.replace('</head>', `
    <script>
    const vscode = acquireVsCodeApi();
    window.addEventListener('message', event => {
        if (event.data.type === 'sync') {
            document.getElementById('source').value = event.data.source;
            document.getElementById('source').dispatchEvent(new Event('input'));
        }
    });
    const origParse = parseDoc;
    parseDoc = function(text) {
        const result = origParse(text);
        if (result && result.nodes) {
            vscode.postMessage({ type: 'updateSource', source: text });
        }
        return result;
    };
    </script>
    </head>`);

    previewPanel.webview.html = html;
}

async function replaceEditorContent(doc: vscode.TextDocument, newContent: string) {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document !== doc) return;

    const fullRange = new vscode.Range(
        doc.positionAt(0),
        doc.positionAt(doc.getText().length)
    );
    await editor.edit(edit => edit.replace(fullRange, newContent));
}
