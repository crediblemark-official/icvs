use serde::Serialize;

use crate::ast::*;
use crate::error::Result;
use crate::validator::topological_sort;

/// Output formats for AI agent tool definitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentFormat {
    Claude,
    OpenAI,
    GenericJson,
    Anthropic,
}

impl AgentFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude" | "anthropic" => Some(Self::Claude),
            "openai" => Some(Self::OpenAI),
            "json" | "generic" => Some(Self::GenericJson),
            _ => None,
        }
    }
}

/// A single node converted to an agent-compatible tool/skill definition.
#[derive(Debug, Clone, Serialize)]
pub struct AgentTool {
    pub name: String,
    pub description: String,
    pub tool_type: String,
    pub severity: Option<String>,
    pub trigger: Option<String>,
    pub depends_on: Vec<String>,
    pub required_by: Vec<String>,
}

/// Convert an entire icvs Document to an AI agent format (JSON string).
pub fn export_agent_format(doc: &Document, format: AgentFormat) -> Result<String> {
    let tools = build_tools(doc);
    match format {
        AgentFormat::Claude => export_claude(&tools),
        AgentFormat::OpenAI => export_openai(&tools),
        AgentFormat::GenericJson => export_json(&tools),
        _ => export_claude(&tools),
    }
}

fn build_tools(doc: &Document) -> Vec<AgentTool> {
    let order = topological_sort(doc).unwrap_or_else(|_| doc.nodes.keys().cloned().collect());
    let mut tools = Vec::new();

    for node_id in &order {
        if let Some(node) = doc.nodes.get(node_id) {
            let incoming: Vec<String> = doc.edges.iter()
                .filter(|e| e.target == node.id)
                .map(|e| e.source.clone())
                .collect();
            let outgoing: Vec<String> = doc.edges.iter()
                .filter(|e| e.source == node.id)
                .map(|e| e.target.clone())
                .collect();

            let tool_type = match node.node_type {
                NodeType::Rule => "rule",
                NodeType::Blocklist => "blocklist",
                NodeType::Allowlist => "allowlist",
                NodeType::Condition => "condition",
                NodeType::Action => "action",
            };

            let description = match (&node.content, &node.condition) {
                (Some(c), _) => c.clone(),
                (None, Some(cond)) => {
                    let desc = format!("If ${} {} \"{}\" then apply node '{}'",
                        cond.variable, cond.operator, cond.value, cond.then_node);
                    if let Some(ref else_node) = cond.else_node {
                        format!("{}, otherwise apply '{}'", desc, else_node)
                    } else {
                        desc
                    }
                }
                (None, None) => format!("{} node", node.id),
            };

            tools.push(AgentTool {
                name: node.id.clone(),
                description,
                tool_type: tool_type.to_string(),
                severity: node.severity.map(|s| s.as_str().to_string()),
                trigger: node.trigger_on.map(|t| t.as_str().to_string()),
                depends_on: incoming,
                required_by: outgoing,
            });
        }
    }

    tools
}

/// Claude/Anthropic tool use format.
///
/// Each node becomes a `tool` with:
/// - name: node_id
/// - description: node content
/// - input_schema: structured parameters based on node type
fn export_claude(tools: &[AgentTool]) -> Result<String> {
    #[derive(Serialize)]
    struct ClaudeTool {
        name: String,
        description: String,
        input_schema: serde_json::Value,
    }

    let claude_tools: Vec<ClaudeTool> = tools.iter().map(|t| {
        let mut properties = serde_json::Map::new();

        properties.insert("action".to_string(), serde_json::json!({
            "type": "string",
            "enum": [t.tool_type],
            "description": "The type of instruction"
        }));

        if t.severity.is_some() {
            properties.insert("severity".to_string(), serde_json::json!({
                "type": "string",
                "enum": ["must", "should", "may"]
            }));
        }

        if !t.depends_on.is_empty() {
            properties.insert("depends_on".to_string(), serde_json::json!({
                "type": "array",
                "items": {"type": "string"},
                "description": "Nodes that must be satisfied first"
            }));
        }

        ClaudeTool {
            name: t.name.clone(),
            description: t.description.clone(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": properties,
                "required": ["action"]
            }),
        }
    }).collect();

    #[derive(Serialize)]
    struct ClaudeOutput {
        tools: Vec<ClaudeTool>,
        system: String,
    }

    let output = ClaudeOutput {
        tools: claude_tools,
        system: format!(
            "You are following a DAG-based instruction set with {} rules. Each tool represents a node in the instruction graph. Execute them in dependency order.",
            tools.len()
        ),
    };

    serde_json::to_string_pretty(&output)
        .map_err(|e| crate::error::IcvsError::Validation {
            message: format!("JSON serialization error: {}", e),
        })
}

/// OpenAI function calling format.
fn export_openai(tools: &[AgentTool]) -> Result<String> {
    #[derive(Serialize)]
    struct OpenAIFunction {
        name: String,
        description: String,
        strict: bool,
        parameters: serde_json::Value,
    }

    let functions: Vec<OpenAIFunction> = tools.iter().map(|t| {
        OpenAIFunction {
            name: t.name.clone(),
            description: t.description.clone(),
            strict: true,
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": [t.tool_type]
                    }
                },
                "required": ["action"],
                "additionalProperties": false
            }),
        }
    }).collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "tools": functions.into_iter().map(|f| serde_json::json!({
            "type": "function",
            "function": f
        })).collect::<Vec<_>>(),
        "tool_choice": "auto"
    }))
    .map_err(|e| crate::error::IcvsError::Validation {
        message: format!("JSON serialization error: {}", e),
    })
}

/// Generic JSON format (list of tool/rule objects).
fn export_json(tools: &[AgentTool]) -> Result<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "format": "instructcanvas",
        "version": "0.2.2",
        "tools": tools
    }))
    .map_err(|e| crate::error::IcvsError::Validation {
        message: format!("JSON serialization error: {}", e),
    })
}

/// Convert from an agent's tool response back to .icvs nodes.
/// Given a list of tool calls, produce a set of .icvs statements.
pub fn agent_tools_to_icvs(agent_tools: &[AgentTool]) -> String {
    let mut output = String::new();

    for tool in agent_tools {
        output.push_str(&format!("[node: {}]\n", tool.name));
        output.push_str(&format!("  type = {}\n", tool.tool_type));
        if !tool.description.is_empty() {
            output.push_str(&format!("  content = \"{}\"\n", tool.description.replace('"', "\\\"")));
        }
        if let Some(ref sev) = tool.severity {
            output.push_str(&format!("  severity = {}\n", sev));
        }
        if let Some(ref trigger) = tool.trigger {
            output.push_str(&format!("  trigger_on = {}\n", trigger));
        }
    }

    for tool in agent_tools {
        for dep in &tool.depends_on {
            output.push_str(&format!("[edge: {} -> {}]\n", tool.name, dep));
        }
        for req in &tool.required_by {
            output.push_str(&format!("[edge: {} -> {}]\n", tool.name, req));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_doc() -> Document {
        let mut doc = Document::new();
        doc.project_name = Some("test".to_string());
        doc.nodes.insert("style".to_string(), Node {
            id: "style".to_string(), node_type: NodeType::Rule,
            content: Some("Use 2-space indent".to_string()),
            severity: Some(crate::ast::Severity::Must),
            trigger_on: None, condition: None, source_line: 1,
        });
        doc.nodes.insert("lint".to_string(), Node {
            id: "lint".to_string(), node_type: NodeType::Action,
            content: Some("Run linter".to_string()),
            severity: None, trigger_on: Some(crate::ast::TriggerOn::Run), condition: None, source_line: 2,
        });
        doc.edges.push(Edge {
            source: "style".to_string(), target: "lint".to_string(),
            label: None, source_line: 3,
        });
        doc
    }

    #[test]
    fn test_export_claude_format() {
        let doc = make_test_doc();
        let json = export_agent_format(&doc, AgentFormat::Claude).unwrap();
        assert!(json.contains("tools"));
        assert!(json.contains("style"));
        assert!(json.contains("lint"));
        assert!(json.contains("2-space"));
    }

    #[test]
    fn test_export_openai_format() {
        let doc = make_test_doc();
        let json = export_agent_format(&doc, AgentFormat::OpenAI).unwrap();
        assert!(json.contains("function"));
        assert!(json.contains("style"));
    }

    #[test]
    fn test_export_json_format() {
        let doc = make_test_doc();
        let json = export_agent_format(&doc, AgentFormat::GenericJson).unwrap();
        assert!(json.contains("instructcanvas"));
        assert!(json.contains("tools"));
    }

    #[test]
    fn test_agent_tools_roundtrip() {
        let tools = vec![
            AgentTool {
                name: "style".to_string(), description: "Use 2-space".to_string(),
                tool_type: "rule".to_string(), severity: Some("must".to_string()),
                trigger: None, depends_on: vec!["base".to_string()], required_by: vec![],
            }
        ];
        let icvs = agent_tools_to_icvs(&tools);
        assert!(icvs.contains("[node: style]"));
        assert!(icvs.contains("type = rule"));
        assert!(icvs.contains("[edge: style -> base]"));
    }
}
