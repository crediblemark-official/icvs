use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use icvs::ast::Document;
use icvs::agent_format;
use icvs::exporter;
use icvs::md_converter;
use icvs::parser;
use icvs::resolver;
use icvs::template;
use icvs::validator;

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<JsValue, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serialize_doc(&doc)
}

#[wasm_bindgen]
pub fn parse_and_resolve(input: &str) -> Result<JsValue, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let doc = resolver::resolve(doc, None).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serialize_doc(&doc)
}

#[wasm_bindgen]
pub fn validate(input: &str) -> Result<JsValue, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let doc = resolver::resolve(doc, None).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let report = validator::validate(&doc).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let json = serde_json::json!({
        "is_valid": report.is_valid,
        "node_count": report.node_count,
        "edge_count": report.edge_count,
        "errors": report.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        "warnings": report.warnings,
        "orphan_nodes": report.orphan_nodes,
    });

    Ok(JsValue::from_str(&json.to_string()))
}

#[wasm_bindgen]
pub fn export_markdown(input: &str, target: &str) -> Result<String, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let doc = resolver::resolve(doc, None).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let report = validator::validate(&doc).map_err(|e| JsValue::from_str(&e.to_string()))?;

    if !report.is_valid {
        return Err(JsValue::from_str(&format!("Validation failed: {:?}", report.errors)));
    }

    exporter::export_markdown(&doc, target)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn export_dot(input: &str) -> Result<String, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let doc = resolver::resolve(doc, None).map_err(|e| JsValue::from_str(&e.to_string()))?;

    exporter::export_dot(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn export_merge(input: &str) -> Result<String, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let doc = resolver::resolve(doc, None).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let report = validator::validate(&doc).map_err(|e| JsValue::from_str(&e.to_string()))?;

    if !report.is_valid {
        return Err(JsValue::from_str(&format!("Validation failed: {:?}", report.errors)));
    }

    exporter::export_markdown_merge(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

fn serialize_doc(doc: &Document) -> Result<JsValue, JsValue> {
    let nodes: Vec<serde_json::Value> = doc.nodes.values().map(|node| {
        serde_json::json!({
            "id": node.id,
            "type": node.node_type.as_str(),
            "content": node.content,
            "severity": node.severity.map(|s| s.as_str()),
            "trigger_on": node.trigger_on.map(|t| t.as_str()),
            "condition": node.condition.as_ref().map(|c| {
                serde_json::json!({
                    "variable": c.variable,
                    "operator": c.operator,
                    "value": c.value,
                    "then_node": c.then_node,
                    "else_node": c.else_node,
                })
            }),
            "source_line": node.source_line,
        })
    }).collect();

    let edges: Vec<serde_json::Value> = doc.edges.iter().map(|edge| {
        serde_json::json!({
            "source": edge.source,
            "target": edge.target,
            "label": edge.label,
        })
    }).collect();

    let targets: Vec<serde_json::Value> = doc.targets.values().map(|t| {
        serde_json::json!({
            "name": t.name,
            "resolve": t.resolve,
            "ignore": t.ignore,
        })
    }).collect();

    let json = serde_json::json!({
        "project_name": doc.project_name,
        "nodes": nodes,
        "edges": edges,
        "targets": targets,
    });

    Ok(JsValue::from_str(&json.to_string()))
}

#[wasm_bindgen]
pub fn md_to_icvs(markdown: &str) -> Result<String, JsValue> {
    md_converter::md_to_icvs(markdown)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn icvs_to_md(input: &str) -> Result<String, JsValue> {
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    md_converter::icvs_to_md(&doc)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn convert_agent(input: &str, format: &str) -> Result<String, JsValue> {
    let fmt = agent_format::AgentFormat::from_str(format).ok_or_else(|| {
        JsValue::from_str(&format!("Unknown format '{}'. Valid: claude, openai, json", format))
    })?;
    let doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    agent_format::export_agent_format(&doc, fmt)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn apply_template(input: &str, vars_json: &str) -> Result<JsValue, JsValue> {
    let mut doc = parser::parse_document(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let vars: HashMap<String, String> = serde_json::from_str(vars_json).map_err(|e| {
        JsValue::from_str(&format!("Invalid vars JSON: {}", e))
    })?;
    template::apply_template(&mut doc, &vars, true)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    serialize_doc(&doc)
}
