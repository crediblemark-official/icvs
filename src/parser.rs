use std::path::Path;

use crate::ast::*;
use crate::error::{IcvsError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
enum BlockKind {
    None,
    Node,
    Target,
}

pub struct Parser {
    lines: Vec<String>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let lines: Vec<String> = input.lines().enumerate()
            .map(|(i, line)| format!("{}:{}", i + 1, line))
            .collect();
        Self { lines }
    }

    pub fn parse(&mut self) -> Result<Document> {
        let mut doc = Document::new();
        let mut current_block: BlockKind = BlockKind::None;
        let mut current_node_id = String::new();
        let mut current_target_name = String::new();
        let mut idx = 0;

        while idx < self.lines.len() {
            let raw_line = &self.lines[idx];
            let colon_pos = raw_line.find(':').ok_or_else(|| {
                IcvsError::Parse { line: 0, message: "internal: malformed line".into() }
            })?;
            let line_num: usize = raw_line[..colon_pos].parse().unwrap_or(0);
            let line = raw_line[colon_pos + 1..].to_string();

            let trimmed = line.trim();

            if trimmed.is_empty() || (trimmed.starts_with('#') && !trimmed.starts_with("#project:")) {
                idx += 1;
                continue;
            }

            let indent = line.len() - line.trim_start().len();
            let is_block_start = trimmed.starts_with('[') && trimmed.ends_with(']');

            if trimmed.starts_with("#project:") {
                let value = trimmed["#project:".len()..].trim();
                doc.project_name = Some(self.parse_possible_quoted(value));
                idx += 1;
                continue;
            }

            if indent == 0 && !is_block_start {
                return Err(IcvsError::Parse {
                    line: line_num,
                    message: "Top-level lines must be block definitions ([...]) or empty/comments".into(),
                });
            }

            if is_block_start {
                current_block = BlockKind::None;
                self.parse_block_header(trimmed, line_num, &mut doc,
                    &mut current_block, &mut current_node_id, &mut current_target_name)?;
                idx += 1;
                continue;
            }

            match current_block {
                BlockKind::Node => {
                    self.parse_node_attr(line.trim(), line_num, &mut doc, &current_node_id)?;
                    idx += 1;
                }
                BlockKind::Target => {
                    let line_trimmed = line.trim();
                    let consumed = self.parse_target_attr_multi(
                        line_trimmed, line_num, &mut doc, &current_target_name, &self.lines, idx
                    )?;
                    idx += consumed;
                }
                BlockKind::None => {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: "Attribute line without a parent block".into(),
                    });
                }
            }
        }

        Ok(doc)
    }

    fn parse_block_header(&self, line: &str, line_num: usize, doc: &mut Document,
        current_block: &mut BlockKind, current_node_id: &mut String,
        current_target_name: &mut String) -> Result<()>
    {
        let inner = &line[1..line.len() - 1];
        let colon_pos = inner.find(':').ok_or_else(|| {
            IcvsError::Parse {
                line: line_num,
                message: format!("Invalid block header '{}': expected 'keyword: arg' format", line),
            }
        })?;

        let keyword = inner[..colon_pos].trim();
        let arg = inner[colon_pos + 1..].trim();

        match keyword {
            "include" => {
                let path = self.parse_quoted_string(arg, line_num)?;
                doc.includes.push(path);
            }
            "node" => {
                let node_id = arg.to_string();
                if !self.is_valid_id(&node_id) {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: format!("Invalid node ID '{}': must be alphanumeric with underscores/hyphens", node_id),
                    });
                }
                if doc.nodes.contains_key(&node_id) {
                    let existing = &doc.nodes[&node_id];
                    return Err(IcvsError::DuplicateNode {
                        node: node_id.clone(),
                        first: existing.source_line,
                        second: line_num,
                    });
                }
                doc.nodes.insert(node_id.clone(), Node {
                    id: node_id.clone(),
                    node_type: NodeType::Rule,
                    content: None,
                    severity: None,
                    trigger_on: None,
                    condition: None,
                    source_line: line_num,
                });
                *current_block = BlockKind::Node;
                *current_node_id = node_id;
            }
            "edge" => {
                let (source, target) = self.parse_edge_arg(arg, line_num)?;
                doc.edges.push(Edge {
                    source,
                    target,
                    label: None,
                    source_line: line_num,
                });
            }
            "target" => {
                let target_name = arg.to_string();
                if !self.is_valid_id(&target_name) {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: format!("Invalid target name '{}'", target_name),
                    });
                }
                if doc.targets.contains_key(&target_name) {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: format!("Duplicate target '{}'", target_name),
                    });
                }
                doc.targets.insert(target_name.clone(), Target {
                    name: target_name.clone(),
                    resolve: None,
                    ignore: None,
                    source_line: line_num,
                });
                *current_block = BlockKind::Target;
                *current_target_name = target_name;
            }
            "project" => {
                let name = self.parse_quoted_string(arg, line_num)?;
                doc.project_name = Some(name);
            }
            _ => {
                return Err(IcvsError::Parse {
                    line: line_num,
                    message: format!("Unknown block type '{}'", keyword),
                });
            }
        }

        Ok(())
    }

    fn parse_node_attr(&self, line: &str, line_num: usize, doc: &mut Document, node_id: &str) -> Result<()> {
        let node = doc.nodes.get_mut(node_id).ok_or_else(|| {
            IcvsError::NodeNotFound { node: node_id.into(), referenced_from: "self".into() }
        })?;

        let eq_pos = line.find('=').ok_or_else(|| {
            IcvsError::Parse {
                line: line_num,
                message: format!("Expected 'key = value' format, got '{}'", line),
            }
        })?;

        let key = line[..eq_pos].trim();
        let value = line[eq_pos + 1..].trim();

        match key {
            "type" => {
                let nt = NodeType::from_str(value).ok_or_else(|| {
                    IcvsError::Parse {
                        line: line_num,
                        message: format!("Unknown node type '{}'. Valid: rule, blocklist, allowlist, condition, action", value),
                    }
                })?;
                node.node_type = nt;
            }
            "content" => {
                let content = self.parse_possible_quoted(value);
                node.content = Some(content);
            }
            "severity" => {
                let sev = Severity::from_str(value).ok_or_else(|| {
                    IcvsError::Parse {
                        line: line_num,
                        message: format!("Unknown severity '{}'. Valid: must, should, may", value),
                    }
                })?;
                node.severity = Some(sev);
            }
            "trigger_on" => {
                let trig = TriggerOn::from_str(value).ok_or_else(|| {
                    IcvsError::Parse {
                        line: line_num,
                        message: format!("Unknown trigger '{}'. Valid: import, install, run", value),
                    }
                })?;
                node.trigger_on = Some(trig);
            }
            "if" => {
                let cond = self.parse_condition(value, line_num)?;
                if node.condition.is_some() {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: "Duplicate 'if' condition for this node".into(),
                    });
                }
                node.condition = Some(cond);
            }
            "then" | "else" => {
                if node.condition.is_none() {
                    return Err(IcvsError::Parse {
                        line: line_num,
                        message: format!("'{}' attribute without a preceding 'if' condition", key),
                    });
                }
                let target_node = value.strip_prefix("-> ").or_else(|| value.strip_prefix("->"))
                    .ok_or_else(|| {
                        IcvsError::Parse {
                            line: line_num,
                            message: format!("Expected '-> node_id' format for '{}', got '{}'", key, value),
                        }
                    })?;
                let target_node = target_node.trim();
                let cond = node.condition.as_mut().unwrap();
                match key {
                    "then" => cond.then_node = target_node.to_string(),
                    "else" => cond.else_node = Some(target_node.to_string()),
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(IcvsError::Parse {
                    line: line_num,
                    message: format!("Unknown attribute '{}' for node. Valid: type, content, severity, trigger_on, if, then, else", key),
                });
            }
        }

        Ok(())
    }

    fn parse_target_attr_multi(&self, line: &str, line_num: usize, doc: &mut Document,
        target_name: &str, all_lines: &[String], current_idx: usize) -> Result<usize> {
        let trimmed = line.trim();
        let eq_pos = trimmed.find('=').ok_or_else(|| {
            IcvsError::Parse {
                line: line_num,
                message: format!("Expected 'key = value' in target, got '{}'", trimmed),
            }
        })?;

        let key = trimmed[..eq_pos].trim();
        let mut value = trimmed[eq_pos + 1..].trim().to_string();
        let mut consumed = 1;

        if value.starts_with('[') && !value.ends_with(']') {
            let mut rest_lines = String::new();
            let mut j = current_idx + 1;
            while j < all_lines.len() {
                let next = &all_lines[j];
                let n_colon = next.find(':').unwrap_or(0);
                let next_line = next[n_colon + 1..].trim().to_string();
                rest_lines.push_str(&next_line);
                consumed += 1;
                if next_line.contains(']') {
                    break;
                }
                j += 1;
            }
            value.push_str(&rest_lines);
        }

        match key {
            "resolve" | "ignore" => {
                let list = self.parse_bracket_list(&value, line_num)?;
                let target = doc.targets.get_mut(target_name).ok_or_else(|| {
                    IcvsError::Parse { line: line_num, message: "internal: target not found".into() }
                })?;
                match key {
                    "resolve" => target.resolve = Some(list),
                    "ignore" => target.ignore = Some(list),
                    _ => unreachable!(),
                }
            }
            _ => {
                return Err(IcvsError::Parse {
                    line: line_num,
                    message: format!("Unknown target attribute '{}'. Valid: resolve, ignore", key),
                });
            }
        }

        Ok(consumed)
    }

    fn parse_quoted_string(&self, s: &str, line_num: usize) -> Result<String> {
        let trimmed = s.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            Ok(trimmed[1..trimmed.len() - 1].to_string())
        } else {
            Err(IcvsError::Parse {
                line: line_num,
                message: format!("Expected quoted string, got '{}'", trimmed),
            })
        }
    }

    fn parse_possible_quoted(&self, s: &str) -> String {
        let trimmed = s.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            trimmed[1..trimmed.len() - 1].to_string()
        } else {
            trimmed.to_string()
        }
    }

    fn parse_edge_arg(&self, arg: &str, line_num: usize) -> Result<(String, String)> {
        let parts: Vec<&str> = arg.splitn(2, "->").collect();
        if parts.len() != 2 {
            return Err(IcvsError::Parse {
                line: line_num,
                message: format!("Invalid edge format '{}'. Expected 'source -> target'", arg),
            });
        }
        let source = parts[0].trim().to_string();
        let target = parts[1].trim().to_string();
        if source.is_empty() || target.is_empty() {
            return Err(IcvsError::Parse {
                line: line_num,
                message: "Edge source and target must not be empty".into(),
            });
        }
        Ok((source, target))
    }

    fn parse_condition(&self, value: &str, line_num: usize) -> Result<Condition> {
        let trimmed = value.trim();
        let dollar_pos = trimmed.find('$');
        if dollar_pos != Some(0) {
            return Err(IcvsError::Parse {
                line: line_num,
                message: format!("Condition must start with $VARIABLE, got '{}'", trimmed),
            });
        }

        let after_dollar = &trimmed[1..];
        let var_end = after_dollar.find(|c: char| !c.is_alphanumeric() && c != '_').unwrap_or(after_dollar.len());
        let variable = after_dollar[..var_end].to_string();
        if variable.is_empty() {
            return Err(IcvsError::Parse {
                line: line_num,
                message: "Empty variable name in condition".into(),
            });
        }

        let rest = after_dollar[var_end..].trim();
        let operators = ["==", "!=", ">=", "<=", ">", "<"];
        let op = operators.iter().find(|op| rest.starts_with(*op))
            .ok_or_else(|| IcvsError::Parse {
                line: line_num,
                message: format!("Expected operator (==, !=, >=, <=, >, <) after variable, got '{}'", rest),
            })?;
        let after_op = rest[op.len()..].trim();
        let value = self.parse_possible_quoted(after_op);
        return Ok(Condition {
            variable,
            operator: op.to_string(),
            value,
            then_node: String::new(),
            else_node: None,
        });
    }

    fn parse_bracket_list(&self, value: &str, line_num: usize) -> Result<Vec<String>> {
        let trimmed = value.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            return Err(IcvsError::Parse {
                line: line_num,
                message: format!("Expected bracketed list [...], got '{}'", trimmed),
            });
        }
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Ok(Vec::new());
        }
        let items: Vec<String> = inner.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(items)
    }

    fn is_valid_id(&self, id: &str) -> bool {
        if id.is_empty() { return false; }
        id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    }
}

pub fn parse_document(input: &str) -> Result<Document> {
    let mut parser = Parser::new(input);
    parser.parse()
}

pub fn parse_document_with_path(input: &str, path: Option<&Path>) -> Result<Document> {
    let mut doc = parse_document(input)?;
    doc.source_path = path.map(|p| p.to_path_buf());
    Ok(doc)
}

pub fn parse_file(path: &Path) -> Result<Document> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| IcvsError::Io {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
    let mut doc = parse_document(&content)?;
    doc.source_path = Some(path.to_path_buf());
    Ok(doc)
}
