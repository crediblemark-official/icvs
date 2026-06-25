use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::*;
use crate::error::{IcvsError, Result};

#[derive(Debug)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<IcvsError>,
    pub warnings: Vec<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub orphan_nodes: Vec<String>,
    pub unresolved_refs: Vec<(String, String)>,
}

pub fn validate(doc: &Document) -> Result<ValidationReport> {
    let mut errors: Vec<IcvsError> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let node_ids: HashSet<&str> = doc.nodes.keys().map(|s| s.as_str()).collect();

    check_cycle(doc, &mut errors);
    check_orphans(doc, &node_ids, &mut warnings);
    check_unresolved_refs(doc, &node_ids, &mut errors);
    check_condition_refs(doc, &node_ids, &mut errors);
    check_target_refs(doc, &node_ids, &mut errors);

    let orphan_nodes = find_orphan_ids(doc);

    Ok(ValidationReport {
        is_valid: errors.is_empty(),
        errors,
        warnings,
        node_count: doc.nodes.len(),
        edge_count: doc.edges.len(),
        orphan_nodes,
        unresolved_refs: Vec::new(),
    })
}

fn check_cycle<'a>(doc: &'a Document, errors: &mut Vec<IcvsError>) {
    let adj = build_adjacency_list(doc);
    let node_ids: Vec<&'a str> = doc.nodes.keys().map(|s| s.as_str()).collect();

    let mut visited: HashSet<&'a str> = HashSet::new();
    let mut in_stack: HashSet<&'a str> = HashSet::new();
    let mut path: Vec<String> = Vec::new();

    for node_id in node_ids {
        if !visited.contains(node_id) {
            if dfs_cycle(node_id, &adj, &mut visited, &mut in_stack, &mut path) {
                let cycle_path: Vec<String> = path.iter().cloned().collect();
                errors.push(IcvsError::CycleDetected { cycle: cycle_path });
                return;
            }
        }
    }
}

fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
    path: &mut Vec<String>,
) -> bool {
    visited.insert(node);
    in_stack.insert(node);
    path.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for &next in neighbors {
            if !visited.contains(next) {
                if dfs_cycle(next, adj, visited, in_stack, path) {
                    return true;
                }
            } else if in_stack.contains(next) {
                let cycle_start = path.iter().position(|n| n == next).unwrap_or(0);
                path.drain(0..cycle_start);
                path.push(next.to_string());
                return true;
            }
        }
    }

    path.pop();
    in_stack.remove(node);
    false
}

fn build_adjacency_list<'a>(doc: &'a Document) -> HashMap<&'a str, Vec<&'a str>> {
    let mut adj: HashMap<&'a str, Vec<&'a str>> = HashMap::new();
    for node_id in doc.nodes.keys() {
        adj.entry(node_id.as_str()).or_default();
    }
    for edge in &doc.edges {
        if doc.nodes.contains_key(&edge.source) && doc.nodes.contains_key(&edge.target) {
            adj.entry(edge.source.as_str()).or_default().push(edge.target.as_str());
        }
    }
    for node in doc.nodes.values() {
        if let Some(cond) = &node.condition {
            if doc.nodes.contains_key(&cond.then_node) {
                adj.entry(node.id.as_str()).or_default().push(cond.then_node.as_str());
            }
            if let Some(ref else_node) = cond.else_node {
                if doc.nodes.contains_key(else_node) {
                    adj.entry(node.id.as_str()).or_default().push(else_node.as_str());
                }
            }
        }
    }
    adj
}

fn check_orphans(doc: &Document, node_ids: &HashSet<&str>, warnings: &mut Vec<String>) {
    let mut referenced: HashSet<&str> = HashSet::new();
    for edge in &doc.edges {
        if node_ids.contains(edge.source.as_str()) {
            referenced.insert(edge.source.as_str());
        }
        if node_ids.contains(edge.target.as_str()) {
            referenced.insert(edge.target.as_str());
        }
    }
    for node in doc.nodes.values() {
        if let Some(cond) = &node.condition {
            if node_ids.contains(cond.then_node.as_str()) {
                referenced.insert(cond.then_node.as_str());
            }
            if let Some(ref else_node) = cond.else_node {
                if node_ids.contains(else_node.as_str()) {
                    referenced.insert(else_node.as_str());
                }
            }
        }
    }
    for node_id in doc.nodes.keys() {
        if !referenced.contains(node_id.as_str()) {
            for edge in &doc.edges {
                if edge.source == *node_id {
                    referenced.insert(node_id.as_str());
                    break;
                }
            }
        }
    }
    'outer: for node_id in doc.nodes.keys() {
        if referenced.contains(node_id.as_str()) {
            continue;
        }
        for edge in &doc.edges {
            if edge.source == *node_id || edge.target == *node_id {
                continue 'outer;
            }
        }
        warnings.push(format!("Node '{}' is not connected to any edge", node_id));
    }
}

fn find_orphan_ids(doc: &Document) -> Vec<String> {
    let mut referenced: HashSet<&str> = HashSet::new();
    for edge in &doc.edges {
        referenced.insert(edge.source.as_str());
        referenced.insert(edge.target.as_str());
    }
    for node in doc.nodes.values() {
        if let Some(cond) = &node.condition {
            referenced.insert(cond.then_node.as_str());
            if let Some(ref else_node) = cond.else_node {
                referenced.insert(else_node.as_str());
            }
        }
    }
    let mut orphans: Vec<String> = Vec::new();
    for node_id in doc.nodes.keys() {
        let id_str: &str = node_id.as_str();
        if !referenced.contains(id_str) {
            let has_outgoing = doc.edges.iter().any(|e| e.source == *node_id);
            if !has_outgoing {
                orphans.push(node_id.clone());
            }
        }
    }
    orphans.sort();
    orphans
}

fn check_unresolved_refs(doc: &Document, node_ids: &HashSet<&str>, errors: &mut Vec<IcvsError>) {
    for edge in &doc.edges {
        if !node_ids.contains(edge.source.as_str()) {
            errors.push(IcvsError::NodeNotFound {
                node: edge.source.clone(),
                referenced_from: format!("edge -> {}", edge.target),
            });
        }
        if !node_ids.contains(edge.target.as_str()) {
            errors.push(IcvsError::NodeNotFound {
                node: edge.target.clone(),
                referenced_from: format!("edge from {}", edge.source),
            });
        }
    }
}

fn check_condition_refs(doc: &Document, node_ids: &HashSet<&str>, errors: &mut Vec<IcvsError>) {
    for node in doc.nodes.values() {
        if let Some(cond) = &node.condition {
            if !node_ids.contains(cond.then_node.as_str()) {
                errors.push(IcvsError::NodeNotFound {
                    node: cond.then_node.clone(),
                    referenced_from: format!("condition 'then' in node '{}'", node.id),
                });
            }
            if let Some(ref else_node) = cond.else_node {
                if !else_node.is_empty() && !node_ids.contains(else_node.as_str()) {
                    errors.push(IcvsError::NodeNotFound {
                        node: else_node.clone(),
                        referenced_from: format!("condition 'else' in node '{}'", node.id),
                    });
                }
            }
        }
    }
}

fn check_target_refs(doc: &Document, node_ids: &HashSet<&str>, errors: &mut Vec<IcvsError>) {
    for target in doc.targets.values() {
        if let Some(ref resolve) = target.resolve {
            for node_id in resolve {
                if !node_ids.contains(node_id.as_str()) {
                    errors.push(IcvsError::NodeNotFound {
                        node: node_id.clone(),
                        referenced_from: format!("target '{}' resolve list", target.name),
                    });
                }
            }
        }
        if let Some(ref ignore) = target.ignore {
            for node_id in ignore {
                if !node_ids.contains(node_id.as_str()) {
                    errors.push(IcvsError::NodeNotFound {
                        node: node_id.clone(),
                        referenced_from: format!("target '{}' ignore list", target.name),
                    });
                }
            }
        }
    }
}

pub fn topological_sort(doc: &Document) -> Result<Vec<String>> {
    let adj = build_adjacency_list(doc);

    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for node_id in doc.nodes.keys() {
        in_degree.insert(node_id.as_str(), 0);
    }
    for (_, neighbors) in &adj {
        for &next in neighbors {
            *in_degree.entry(next).or_insert(0) += 1;
        }
    }

    let mut queue: VecDeque<&str> = VecDeque::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node);
        }
    }

    let mut result: Vec<String> = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.to_string());
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                if let Some(deg) = in_degree.get_mut(next) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    if result.len() != doc.nodes.len() {
        return Err(IcvsError::CycleDetected {
            cycle: vec!["graph contains a cycle — cannot topological sort".to_string()],
        });
    }

    Ok(result)
}
