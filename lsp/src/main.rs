use std::collections::HashMap;
use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tokio::sync::Mutex;

use icvs::ast::Document;
use icvs::error::IcvsError;

struct Backend {
    client: Client,
    docs: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

struct DocumentState {
    text: String,
    version: i32,
    doc: Option<Document>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["[".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "icvs-lsp: initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        let version = params.text_document.version;

        let state = self.validate(&uri, &text, version).await;
        self.docs.lock().await.insert(uri, state);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        let mut docs = self.docs.lock().await;
        let state = docs.get_mut(&uri);

        match state {
            Some(s) => {
                for change in &params.content_changes {
                    if let Some(range) = change.range {
                        let start = offset_to_pos(&s.text, range.start);
                        let end = offset_to_pos(&s.text, range.end);
                        let mut new_text = s.text[..start].to_string();
                        new_text.push_str(&change.text);
                        new_text.push_str(&s.text[end..]);
                        s.text = new_text;
                    } else {
                        s.text = change.text.clone();
                    }
                    s.version = version;
                }
                let uri = uri.clone();
                let text = s.text.clone();
                let v = s.version;
                drop(docs);
                let new_state = self.validate(&uri, &text, v).await;
                self.docs.lock().await.insert(uri, new_state);
            }
            None => {
                drop(docs);
                let text = params.content_changes.into_iter()
                    .map(|c| c.text).collect::<Vec<_>>().join("");
                let state = self.validate(&uri, &text, version).await;
                self.docs.lock().await.insert(uri, state);
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            let diags = build_diagnostics(&state.doc);
            self.client.publish_diagnostics(uri, diags, None).await;
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        let items = vec![
            c_item("node", "Node", "Define a node block", "node: "),
            c_item("edge", "Edge", "Define an edge", "edge: source -> target"),
            c_item("target", "Target", "Define a target platform", "target: "),
            c_item("include", "Include", "Include another .icvs file", "include: \"\""),
            c_item("type", "Type attr", "Set node type", "type = "),
            c_simple("rule", "Type: rule", "Rule node type: an instruction or guideline"),
            c_simple("blocklist", "Type: blocklist", "Blocklist node type: forbidden patterns"),
            c_simple("allowlist", "Type: allowlist", "Allowlist node type: approved patterns"),
            c_simple("condition", "Type: condition", "Condition node type: branching logic"),
            c_simple("action", "Type: action", "Action node type: executable step"),
            c_item("content", "Content attr", "Set node content", "content = \"\""),
            c_item("severity", "Severity attr", "Set severity level", "severity = "),
            c_simple("must", "Severity: must", "Required: must be followed"),
            c_simple("should", "Severity: should", "Recommended: should be followed"),
            c_simple("may", "Severity: may", "Optional: may be followed"),
            c_item("trigger_on", "Trigger attr", "Set trigger event", "trigger_on = "),
            c_simple("import", "Trigger: import", "Trigger on import"),
            c_simple("install", "Trigger: install", "Trigger on install"),
            c_simple("run", "Trigger: run", "Trigger on run"),
        ];
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            if let Some(ref doc) = state.doc {
                if let Some(hover) = build_hover(doc, &state.text, pos) {
                    return Ok(Some(hover));
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            if let Some(ref doc) = state.doc {
                if let Some(location) = find_definition(doc, &state.text, pos) {
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = params.new_name;

        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            if let Some(ref doc) = state.doc {
                if let Some(edit) = build_rename(doc, &state.text, pos, &new_name) {
                    return Ok(Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(uri.clone(), edit)])),
                        ..Default::default()
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            if let Some(ref doc) = state.doc {
                let symbols = build_symbols(doc);
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
        }
        Ok(None)
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        let uri = params.text_document.uri;
        let docs = self.docs.lock().await;
        if let Some(state) = docs.get(&uri) {
            let ranges = build_folding_ranges(&state.text);
            return Ok(Some(ranges));
        }
        Ok(None)
    }
}

impl Backend {
    async fn validate(&self, uri: &Url, text: &str, version: i32) -> DocumentState {
        let doc_result = icvs::parser::parse_document(text);
        let (doc, diags) = match doc_result {
            Ok(d) => {
                let mut diags = build_diagnostics(&Some(d.clone()));

                if let Ok(report) = icvs::validator::validate(&d) {
                    for err in &report.errors {
                        let (line, message) = match err {
                            IcvsError::Parse { line, message } => (*line, message.clone()),
                            IcvsError::Validation { message } => (0, message.clone()),
                            IcvsError::CycleDetected { cycle } => (0, format!("Cycle detected: {}", cycle.join(" → "))),
                            IcvsError::NodeNotFound { node, referenced_from } => (0, format!("Node '{}' not found (from {})", node, referenced_from)),
                            IcvsError::IncludeNotFound { path } => (0, format!("Include not found: {}", path.display())),
                            IcvsError::DuplicateNode { node, first, second } => (*second as usize, format!("Duplicate node '{}' (first at line {})", node, first)),
                            _ => (0, err.to_string()),
                        };
                        let sev = match err {
                            IcvsError::CycleDetected { .. } | IcvsError::DuplicateNode { .. } | IcvsError::NodeNotFound { .. } => DiagnosticSeverity::ERROR,
                            IcvsError::Validation { .. } => DiagnosticSeverity::WARNING,
                            _ => DiagnosticSeverity::ERROR,
                        };
                        diags.push(Diagnostic {
                            range: Range { start: Position { line: line.max(1) as u32 - 1, character: 0 }, end: Position { line: line.max(1) as u32 - 1, character: 0 } },
                            severity: Some(sev),
                            source: Some("icvs".to_string()),
                            message,
                            ..Default::default()
                        });
                    }
                }

                (Some(d), diags)
            }
            Err(e) => {
                let (line, message) = match &e {
                    IcvsError::Parse { line, message } => (*line, message.clone()),
                    _ => (0, e.to_string()),
                };
                let diags = vec![Diagnostic {
                    range: Range { start: Position { line: line.max(1) as u32 - 1, character: 0 }, end: Position { line: line.max(1) as u32 - 1, character: 0 } },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("icvs".to_string()),
                    message,
                    ..Default::default()
                }];
                (None, diags)
            }
        };

        self.client.publish_diagnostics(uri.clone(), diags, Some(version)).await;

        DocumentState {
            text: text.to_string(),
            version,
            doc,
        }
    }
}

// ── Helpers ──

fn c_item(label: &str, detail: &str, doc: &str, insert: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::String(doc.to_string())),
        insert_text: Some(insert.to_string()),
        ..Default::default()
    }
}

fn c_simple(label: &str, detail: &str, doc: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        detail: Some(detail.to_string()),
        documentation: Some(Documentation::String(doc.to_string())),
        ..Default::default()
    }
}

fn build_diagnostics(doc: &Option<Document>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if let Some(ref doc) = doc {
        for node in doc.nodes.values() {
            if node.content.is_none() && node.condition.is_none() {
                diags.push(Diagnostic {
                    range: Range {
                        start: Position { line: node.source_line as u32 - 1, character: 0 },
                        end: Position { line: node.source_line as u32 - 1, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::HINT),
                    source: Some("icvs".to_string()),
                    message: format!("Node '{}' has no content or condition", node.id),
                    ..Default::default()
                });
            }
        }
    }
    diags
}

fn offset_to_pos(text: &str, pos: Position) -> usize {
    let mut offset = 0;
    let mut line = 0u32;
    for (i, c) in text.char_indices() {
        if line >= pos.line {
            offset = i + pos.character as usize;
            break;
        }
        if c == '\n' {
            line += 1;
        }
    }
    offset.min(text.len())
}

fn build_hover(doc: &Document, text: &str, pos: Position) -> Option<Hover> {
    let line_idx = pos.line as usize;
    let lines: Vec<&str> = text.lines().collect();
    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];

    for node in doc.nodes.values() {
        let header = format!("[node: {}]", node.id);
        if line.contains(&header) || line.contains(&node.id) {
            let mut content = format!("**Node: {}**  \nType: `{}`  \n", node.id, node.node_type.as_str());
            if let Some(ref sev) = node.severity {
                content.push_str(&format!("Severity: `{}`  \n", sev.as_str()));
            }
            if let Some(ref c) = node.content {
                content.push_str(&format!("\n_{}_\n", c));
            }
            if let Some(ref cond) = node.condition {
                content.push_str(&format!("\nCondition: `${}` {} \"{}\"", cond.variable, cond.operator, cond.value));
                if !cond.then_node.is_empty() {
                    content.push_str(&format!("\nThen → `{}`", cond.then_node));
                }
                if let Some(ref else_node) = cond.else_node {
                    content.push_str(&format!("\nElse → `{}`", else_node));
                }
            }
            let edges_in: Vec<&str> = doc.edges.iter().filter(|e| e.target == node.id).map(|e| e.source.as_str()).collect();
            let edges_out: Vec<&str> = doc.edges.iter().filter(|e| e.source == node.id).map(|e| e.target.as_str()).collect();
            if !edges_in.is_empty() {
                content.push_str(&format!("\n\nRequired by: {}", edges_in.join(", ")));
            }
            if !edges_out.is_empty() {
                content.push_str(&format!("\nRequires: {}", edges_out.join(", ")));
            }
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(Range {
                    start: Position { line: pos.line, character: 0 },
                    end: Position { line: pos.line, character: line.len() as u32 },
                }),
            });
        }
    }

    None
}

fn find_definition(doc: &Document, text: &str, pos: Position) -> Option<Location> {
    let line_idx = pos.line as usize;
    let lines: Vec<&str> = text.lines().collect();
    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];

    let matched = doc.nodes.iter().find(|(_, node)| line.contains(node.id.as_str()));
    if let Some((_, node)) = matched {
        return Some(Location {
            uri: Url::parse("file:///document.icvs").ok()?,
            range: Range {
                start: Position { line: node.source_line as u32 - 1, character: 0 },
                end: Position { line: node.source_line as u32 - 1, character: 100 },
            },
        });
    }

    None
}

fn build_rename(doc: &Document, text: &str, pos: Position, new_name: &str) -> Option<Vec<TextEdit>> {
    let line_idx = pos.line as usize;
    let lines: Vec<&str> = text.lines().collect();
    if line_idx >= lines.len() {
        return None;
    }
    let line = lines[line_idx];

    let old_id = doc.nodes.keys().find(|id| line.contains(id.as_str()))?;
    let mut edits = Vec::new();

    for (i, l) in lines.iter().enumerate() {
        if let Some(start_col) = l.find(old_id.as_str()) {
            edits.push(TextEdit {
                range: Range {
                    start: Position { line: i as u32, character: start_col as u32 },
                    end: Position { line: i as u32, character: (start_col + old_id.len()) as u32 },
                },
                new_text: new_name.to_string(),
            });
        }
    }

    Some(edits)
}

#[allow(deprecated)]
fn build_symbols(doc: &Document) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    for node in doc.nodes.values() {
        let kind = match node.node_type {
            icvs::ast::NodeType::Rule => SymbolKind::FUNCTION,
            icvs::ast::NodeType::Blocklist => SymbolKind::CONSTANT,
            icvs::ast::NodeType::Allowlist => SymbolKind::CONSTANT,
            icvs::ast::NodeType::Condition => SymbolKind::EVENT,
            icvs::ast::NodeType::Action => SymbolKind::METHOD,
        };
        symbols.push(DocumentSymbol {
            name: node.id.clone(),
            detail: Some(node.node_type.as_str().to_string()),
            kind,
            range: Range {
                start: Position { line: node.source_line as u32 - 1, character: 0 },
                end: Position { line: node.source_line as u32, character: 0 },
            },
            selection_range: Range {
                start: Position { line: node.source_line as u32 - 1, character: 0 },
                end: Position { line: node.source_line as u32 - 1, character: node.id.len() as u32 },
            },
            children: None,
            tags: None,
            deprecated: None,
        });
    }
    symbols
}

fn build_folding_ranges(text: &str) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with("[node:") || line.starts_with("[target:") {
            let start = i;
            let mut end = i;
            i += 1;
            while i < lines.len() {
                let next = lines[i].trim();
                if next.starts_with('[') || next.is_empty() {
                    break;
                }
                end = i;
                i += 1;
            }
            if end > start {
                ranges.push(FoldingRange {
                    start_line: start as u32,
                    end_line: end as u32,
                    start_character: None,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: None,
                });
            }
        } else {
            i += 1;
        }
    }

    ranges
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let docs: Arc<Mutex<HashMap<Url, DocumentState>>> = Arc::new(Mutex::new(HashMap::new()));

    let (service, socket) = LspService::new(|client| Backend {
        client,
        docs: docs.clone(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
