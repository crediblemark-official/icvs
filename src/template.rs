use regex::Regex;
use std::collections::HashMap;

use crate::ast::*;
use crate::error::{IcvsError, Result};

/// Resolve template variables in a string with the pattern `{{ var_name }}`.
/// If a variable is not found, leaves the placeholder as-is (strict mode returns error).
pub fn resolve_template_vars(content: &str, vars: &HashMap<String, String>, strict: bool) -> Result<String> {
    let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();
    let mut result = content.to_string();

    for cap in re.captures_iter(content) {
        let var_name = cap.get(1).unwrap().as_str();
        let full_match = cap.get(0).unwrap().as_str();

        match vars.get(var_name) {
            Some(val) => {
                result = result.replace(full_match, val);
            }
            None => {
                if strict {
                    return Err(IcvsError::Validation {
                        message: format!("Template variable '{}' not provided", var_name),
                    });
                }
            }
        }
    }

    Ok(result)
}

/// Apply template variables to all node contents in a document.
pub fn apply_template(doc: &mut Document, vars: &HashMap<String, String>, strict: bool) -> Result<()> {
    for node in doc.nodes.values_mut() {
        if let Some(ref content) = node.content {
            let resolved = resolve_template_vars(content, vars, strict)?;
            if resolved != *content {
                node.content = Some(resolved);
            }
        }
    }

    for edge in &mut doc.edges {
        if let Some(ref label) = edge.label.clone() {
            let resolved = resolve_template_vars(label, vars, strict)?;
            if resolved != *label {
                edge.label = Some(resolved);
            }
        }
    }

    Ok(())
}

/// Parse a template include directive: `[include: base_node @ template_name ]`
/// Returns (base_node_id, template_name) if it's a template include.
pub fn parse_template_include(line: &str) -> Option<(String, String)> {
    let inner = line.trim();
    if !inner.starts_with("[include:") || !inner.ends_with(']') {
        return None;
    }
    let content = inner["[include:".len()..inner.len() - 1].trim();

    if let Some(at_pos) = content.find('@') {
        let base = content[..at_pos].trim().to_string();
        let tmpl = content[at_pos + 1..].trim().to_string();
        if !base.is_empty() && !tmpl.is_empty() {
            return Some((base, tmpl));
        }
    }

    None
}

/// Represents a template definition in an .icvs file.
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub params: Vec<String>,
    pub content: String,
    pub source_line: usize,
}

/// Extract template definitions from a document's includes/merged context.
/// A template is defined as: `[template: name]` block with nodes that contain `{{ param }}` variables.
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    pub templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tmpl: Template) -> Result<()> {
        if self.templates.contains_key(&tmpl.name) {
            return Err(IcvsError::Validation {
                message: format!("Duplicate template '{}'", tmpl.name),
            });
        }
        self.templates.insert(tmpl.name.clone(), tmpl);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Instantiate a template with given variable values.
    pub fn instantiate(&self, name: &str, vars: &HashMap<String, String>) -> Result<String> {
        let tmpl = self.templates.get(name).ok_or_else(|| {
            IcvsError::Validation {
                message: format!("Template '{}' not found", name),
            }
        })?;

        let mut result = tmpl.content.clone();
        let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

        for cap in re.captures_iter(&tmpl.content) {
            let var_name = cap.get(1).unwrap().as_str();
            let full_match = cap.get(0).unwrap().as_str();
            match vars.get(var_name) {
                Some(val) => {
                    result = result.replace(full_match, val);
                }
                None => {
                    return Err(IcvsError::Validation {
                        message: format!("Template '{}' requires variable '{}' but it was not provided", name, var_name),
                    });
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_simple_vars() {
        let content = "Use {{ FRAMEWORK }} for this project";
        let mut vars = HashMap::new();
        vars.insert("FRAMEWORK".to_string(), "React".to_string());
        let result = resolve_template_vars(content, &vars, true).unwrap();
        assert_eq!(result, "Use React for this project");
    }

    #[test]
    fn test_resolve_missing_var_not_strict() {
        let content = "Hello {{ NAME }}";
        let vars = HashMap::new();
        let result = resolve_template_vars(content, &vars, false).unwrap();
        assert_eq!(result, "Hello {{ NAME }}");
    }

    #[test]
    fn test_resolve_missing_var_strict() {
        let content = "Hello {{ NAME }}";
        let vars = HashMap::new();
        let result = resolve_template_vars(content, &vars, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_template_include_simple() {
        let line = "[include: base_node @ my_template]";
        let result = parse_template_include(line);
        assert!(result.is_some());
        let (base, tmpl) = result.unwrap();
        assert_eq!(base, "base_node");
        assert_eq!(tmpl, "my_template");
    }

    #[test]
    fn test_parse_template_include_no_at() {
        let line = "[include: some_node]";
        let result = parse_template_include(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_template_registry() {
        let mut reg = TemplateRegistry::new();
        reg.register(Template {
            name: "greeting".to_string(),
            params: vec!["NAME".to_string()],
            content: "Hello {{ NAME }}!".to_string(),
            source_line: 1,
        }).unwrap();

        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "World".to_string());
        let result = reg.instantiate("greeting", &vars).unwrap();
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_template_duplicate_error() {
        let mut reg = TemplateRegistry::new();
        reg.register(Template {
            name: "dup".to_string(), params: vec![], content: "a".to_string(), source_line: 1,
        }).unwrap();
        let result = reg.register(Template {
            name: "dup".to_string(), params: vec![], content: "b".to_string(), source_line: 2,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_template_to_document() {
        let mut doc = Document::new();
        doc.nodes.insert("a".to_string(), Node {
            id: "a".to_string(),
            node_type: NodeType::Rule,
            content: Some("Use {{ FRAMEWORK }}".to_string()),
            severity: None,
            trigger_on: None,
            condition: None,
            source_line: 1,
        });

        let mut vars = HashMap::new();
        vars.insert("FRAMEWORK".to_string(), "Vue".to_string());
        apply_template(&mut doc, &vars, true).unwrap();

        assert_eq!(doc.nodes["a"].content.as_deref(), Some("Use Vue"));
    }
}
