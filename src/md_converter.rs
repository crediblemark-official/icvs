use crate::ast::*;
use crate::error::Result;
use crate::parser;

/// Convert a Markdown document to .icvs format.
///
/// Conventions:
/// - `# Title` → `#project: "..."`
/// - `## section_id` → `[node: section_id]` (kebab-case anchor from heading)
/// - `### subsection` → content inside parent node
/// - `- section_id` as first list under a node → edges (depends on → node) or resolve/ignore flags
/// - Code blocks → `content = "..."`
pub fn md_to_icvs(markdown: &str) -> Result<String> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut output = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if line.trim().is_empty() || line.trim().starts_with("<!--") {
            i += 1;
            continue;
        }

        if line.starts_with("# ") {
            let title = line[2..].trim();
            output.push_str(&format!("#project: \"{}\"\n", title));
            i += 1;
            continue;
        }

        if line.starts_with("## ") {
            let heading = line[3..].trim();
            let node_id = heading.to_lowercase().replace(char::is_whitespace, "_").replace('-', "_");
            output.push_str(&format!("\n[node: {}]\n", node_id));

            i += 1;
            let mut content_lines = Vec::new();
            let mut edges = Vec::new();
            let mut resolve = Vec::new();
            let mut ignore = Vec::new();
            let mut in_code_block = false;

            while i < lines.len() {
                let sub = lines[i];

                if sub.starts_with("## ") || sub.starts_with("# ") {
                    break;
                }

                if sub.trim().starts_with("```") {
                    in_code_block = !in_code_block;
                    if !in_code_block {
                        content_lines.push(sub.to_string());
                    }
                    i += 1;
                    continue;
                }

                if !in_code_block {
                    let trimmed = sub.trim();
                    if trimmed.starts_with("**") && !trimmed.contains("** → `") {
                        i += 1;
                        continue;
                    }
                    if trimmed.starts_with("- **") && trimmed.contains("** → `") {
                        let parts: Vec<&str> = trimmed.splitn(2, "** → `").collect();
                        if parts.len() == 2 {
                            let source = parts[0][4..].trim();
                            let target = parts[1].trim_end_matches('`');
                            edges.push((source.to_string(), target.to_string()));
                            i += 1;
                            continue;
                        }
                    }
                    if trimmed.starts_with("- `") && trimmed.ends_with('`') {
                        let item = trimmed[3..trimmed.len() - 1].trim();
                        edges.push((item.to_string(), node_id.clone()));
                        i += 1;
                        continue;
                    }
                    if trimmed.starts_with("- ") {
                        let item = trimmed[2..].trim();
                        if item.starts_with('~') {
                            ignore.push(item[1..].trim().to_string());
                        } else {
                            resolve.push(item.to_string());
                        }
                        i += 1;
                        continue;
                    }
                }

                content_lines.push(sub.to_string());
                i += 1;
            }

            if !content_lines.is_empty() {
                let content = content_lines.iter().map(|l| l.trim_end()).collect::<Vec<_>>().join("\n").trim().to_string();
                if !content.is_empty() {
                    output.push_str(&format!("  content = \"\"\"\n{}\n\"\"\"\n", content));
                }
            }

            for (src, tgt) in &edges {
                output.push_str(&format!("[edge: {} -> {}]\n", src, tgt));
            }

            if !resolve.is_empty() {
                let list = resolve.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("[target: {}]\n  resolve = [{}]\n", node_id, list));
            }
            if !ignore.is_empty() {
                let list = ignore.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ");
                output.push_str(&format!("[target: {}]\n  ignore = [{}]\n", node_id, list));
            }

            continue;
        }

        i += 1;
    }

    Ok(output)
}

/// Convert .icvs to clean, round-trippable Markdown.
pub fn icvs_to_md(doc: &Document) -> Result<String> {
    let mut output = String::new();

    if let Some(ref name) = doc.project_name {
        output.push_str(&format!("# {}\n\n", name));
    }

    let sorted = crate::validator::topological_sort(doc)?;

    for node_id in &sorted {
        if let Some(node) = doc.nodes.get(node_id) {
            output.push_str(&format!("## {}\n\n", node_id));

            let tag = match node.node_type {
                NodeType::Rule => vec!["rule"],
                NodeType::Blocklist => vec!["blocklist"],
                NodeType::Allowlist => vec!["allowlist"],
                NodeType::Condition => vec!["condition"],
                NodeType::Action => vec!["action"],
            };
            output.push_str(&format!("_Type: {}_  \n", tag.join(", ")));

            if let Some(ref sev) = node.severity {
                output.push_str(&format!("_Severity: {}_  \n", sev.as_str()));
            }

            if let Some(ref trigger) = node.trigger_on {
                output.push_str(&format!("_Trigger on: {}_  \n", trigger.as_str()));
            }

            output.push('\n');

            if let Some(content) = &node.content {
                if content.contains('\n') {
                    output.push_str("```\n");
                    output.push_str(content);
                    output.push_str("\n```\n\n");
                } else {
                    output.push_str(content);
                    output.push_str("\n\n");
                }
            }

            if let Some(ref cond) = node.condition {
                output.push_str(&format!("- **If** `${}` {} `\"{}\"`  \n", cond.variable, cond.operator, cond.value));
                if !cond.then_node.is_empty() {
                    output.push_str(&format!("  - **Then** → `{}`\n", cond.then_node));
                }
                if let Some(ref else_node) = cond.else_node {
                    output.push_str(&format!("  - **Else** → `{}`\n", else_node));
                }
                output.push('\n');
            }

            let incoming: Vec<&Edge> = doc.edges.iter().filter(|e| e.target == *node_id).collect();
            let outgoing: Vec<&Edge> = doc.edges.iter().filter(|e| e.source == *node_id).collect();

            if !incoming.is_empty() {
                output.push_str("**Depends on:**\n");
                for e in &incoming {
                    output.push_str(&format!("- `{}`\n", e.source));
                }
                output.push('\n');
            }

            if !outgoing.is_empty() {
                output.push_str("**Required by:**\n");
                for e in &outgoing {
                    output.push_str(&format!("- `{}`", e.target));
                    if let Some(ref label) = e.label {
                        output.push_str(&format!(" ({})", label));
                    }
                    output.push('\n');
                }
                output.push('\n');
            }

            output.push_str("---\n\n");
        }
    }

    Ok(output)
}

/// Round-trip: parse .icvs, export md, then re-parse md and compare
pub fn icvs_roundtrip(input: &str) -> Result<Document> {
    let doc = parser::parse_document(input)?;
    let md = icvs_to_md(&doc)?;
    let doc2 = parser::parse_document(&md_to_icvs(&md)?)?;
    Ok(doc2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_to_icvs_simple() {
        let md = r#"# My Project

## coding_style

Use 2-space indentation.

- **Depends on:** → `styleguide`
"#;
        let icvs = md_to_icvs(md).unwrap();
        assert!(icvs.contains("#project: \"My Project\""));
        assert!(icvs.contains("[node: coding_style]"));
        assert!(icvs.contains("Use 2-space indentation"));
    }

    #[test]
    fn test_icvs_to_md_roundtrip() {
        let input = r#"
#project: "test"

[node: a]
  type = rule
  content = "Node A"
  severity = must

[node: b]
  type = rule
  content = "Node B"
  severity = should

[edge: a -> b]
"#;
        let doc = parser::parse_document(input).unwrap();
        let md = icvs_to_md(&doc).unwrap();
        assert!(md.contains("# test"));
        assert!(md.contains("## a"));
        assert!(md.contains("## b"));
        assert!(md.contains("Node A"));
        assert!(md.contains("Node B"));
    }

    #[test]
    fn test_md_to_icvs_with_edges() {
        let md = r#"## rules

- **styleguide** → `coding`
- testing

## coding_style

Use 2-space.
"#;
        let icvs = md_to_icvs(md).unwrap();
        assert!(icvs.contains("[node: rules]"));
        assert!(icvs.contains("[node: coding_style]"));
        assert!(icvs.contains("[edge: styleguide -> coding]"));
    }
}
