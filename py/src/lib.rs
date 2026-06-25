use std::collections::HashMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use icvs::agent_format;
use icvs::ast::Document;
use icvs::exporter;
use icvs::md_converter;
use icvs::parser;
use icvs::resolver;
use icvs::template;
use icvs::validator;

#[pyclass]
#[derive(Clone)]
struct IcvsDocument {
    doc: Document,
}

#[pymethods]
impl IcvsDocument {
    fn node_count(&self) -> usize {
        self.doc.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.doc.edges.len()
    }

    fn target_count(&self) -> usize {
        self.doc.targets.len()
    }

    fn project_name(&self) -> Option<String> {
        self.doc.project_name.clone()
    }

    fn node_ids(&self) -> Vec<String> {
        self.doc.nodes.keys().cloned().collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "IcvsDocument(nodes={}, edges={}, targets={})",
            self.doc.nodes.len(),
            self.doc.edges.len(),
            self.doc.targets.len(),
        )
    }
}

#[pyclass]
struct ValidationResult {
    #[pyo3(get)]
    is_valid: bool,
    #[pyo3(get)]
    node_count: usize,
    #[pyo3(get)]
    edge_count: usize,
    #[pyo3(get)]
    errors: Vec<String>,
    #[pyo3(get)]
    warnings: Vec<String>,
    #[pyo3(get)]
    orphan_nodes: Vec<String>,
}

#[pymethods]
impl ValidationResult {
    fn __repr__(&self) -> String {
        format!(
            "ValidationResult(valid={}, nodes={}, edges={}, errors={})",
            self.is_valid, self.node_count, self.edge_count, self.errors.len()
        )
    }
}

#[pyfunction]
fn parse(source: &str) -> PyResult<IcvsDocument> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(IcvsDocument { doc })
}

#[pyfunction]
fn parse_file(path: &str) -> PyResult<IcvsDocument> {
    let path = std::path::Path::new(path);
    let doc = resolver::resolve_file(path)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(IcvsDocument { doc })
}

#[pyfunction]
fn validate(source: &str) -> PyResult<ValidationResult> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let doc = resolver::resolve(doc, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let report = validator::validate(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(ValidationResult {
        is_valid: report.is_valid,
        node_count: report.node_count,
        edge_count: report.edge_count,
        errors: report.errors.iter().map(|e| e.to_string()).collect(),
        warnings: report.warnings,
        orphan_nodes: report.orphan_nodes,
    })
}

#[pyfunction]
fn validate_file(path: &str) -> PyResult<ValidationResult> {
    let doc = resolver::resolve_file(std::path::Path::new(path))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let report = validator::validate(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(ValidationResult {
        is_valid: report.is_valid,
        node_count: report.node_count,
        edge_count: report.edge_count,
        errors: report.errors.iter().map(|e| e.to_string()).collect(),
        warnings: report.warnings,
        orphan_nodes: report.orphan_nodes,
    })
}

#[pyfunction]
fn export_markdown(source: &str, target: &str) -> PyResult<String> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let doc = resolver::resolve(doc, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let report = validator::validate(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    if !report.is_valid {
        return Err(PyValueError::new_err(format!(
            "Document is invalid: {} errors", report.errors.len()
        )));
    }

    exporter::export_markdown(&doc, target)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn export_dot(source: &str) -> PyResult<String> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let doc = resolver::resolve(doc, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    exporter::export_dot(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn export_merge(source: &str) -> PyResult<String> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let doc = resolver::resolve(doc, None)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    let report = validator::validate(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    if !report.is_valid {
        return Err(PyValueError::new_err(format!(
            "Document is invalid: {} errors", report.errors.len()
        )));
    }

    exporter::export_markdown_merge(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn md_to_icvs(markdown: &str) -> PyResult<String> {
    md_converter::md_to_icvs(markdown)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn icvs_to_md(source: &str) -> PyResult<String> {
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    md_converter::icvs_to_md(&doc)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn convert_agent(source: &str, format: &str) -> PyResult<String> {
    let fmt = agent_format::AgentFormat::from_str(format).ok_or_else(|| {
        PyValueError::new_err(format!("Unknown format '{}'. Valid: claude, openai, json", format))
    })?;
    let doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    agent_format::export_agent_format(&doc, fmt)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn apply_template(source: &str, vars: HashMap<String, String>) -> PyResult<String> {
    let mut doc = parser::parse_document(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    template::apply_template(&mut doc, &vars, true)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let mut out = String::new();
    if let Some(ref name) = doc.project_name {
        out.push_str(&format!("#project: \"{}\"\n", name));
    }
    for node in doc.nodes.values() {
        out.push_str(&format!("\n[node: {}]\n", node.id));
        out.push_str(&format!("  type = {}\n", node.node_type.as_str()));
        if let Some(ref c) = node.content {
            out.push_str(&format!("  content = \"{}\"\n", c));
        }
        if let Some(ref s) = node.severity {
            out.push_str(&format!("  severity = {}\n", s.as_str()));
        }
        if let Some(ref t) = node.trigger_on {
            out.push_str(&format!("  trigger_on = {}\n", t.as_str()));
        }
        if let Some(ref cond) = node.condition {
            out.push_str(&format!("  if = ${} {} \"{}\"\n", cond.variable, cond.operator, cond.value));
            if !cond.then_node.is_empty() {
                out.push_str(&format!("  then = -> {}\n", cond.then_node));
            }
            if let Some(ref else_node) = cond.else_node {
                out.push_str(&format!("  else = -> {}\n", else_node));
            }
        }
    }
    for edge in &doc.edges {
        out.push_str(&format!("\n[edge: {} -> {}]\n", edge.source, edge.target));
    }
    for target in doc.targets.values() {
        out.push_str(&format!("\n[target: {}]\n", target.name));
        if let Some(ref resolve) = target.resolve {
            let list: Vec<String> = resolve.iter().map(|s| format!("\"{}\"", s)).collect();
            out.push_str(&format!("  resolve = [{}]\n", list.join(", ")));
        }
        if let Some(ref ignore) = target.ignore {
            let list: Vec<String> = ignore.iter().map(|s| format!("\"{}\"", s)).collect();
            out.push_str(&format!("  ignore = [{}]\n", list.join(", ")));
        }
    }
    Ok(out)
}

#[pymodule]
fn instructcanvas(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_file, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(validate_file, m)?)?;
    m.add_function(wrap_pyfunction!(export_markdown, m)?)?;
    m.add_function(wrap_pyfunction!(export_dot, m)?)?;
    m.add_function(wrap_pyfunction!(export_merge, m)?)?;
    m.add_function(wrap_pyfunction!(md_to_icvs, m)?)?;
    m.add_function(wrap_pyfunction!(icvs_to_md, m)?)?;
    m.add_function(wrap_pyfunction!(convert_agent, m)?)?;
    m.add_function(wrap_pyfunction!(apply_template, m)?)?;
    m.add_class::<IcvsDocument>()?;
    m.add_class::<ValidationResult>()?;
    Ok(())
}
