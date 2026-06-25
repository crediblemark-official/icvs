use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::error::{IcvsError, Result};
use crate::parser;

pub fn resolve(doc: Document, base_path: Option<&Path>) -> Result<Document> {
    let mut resolver = Resolver::new();
    resolver.resolve_document(doc, base_path)
}

struct Resolver {
    visited: HashSet<PathBuf>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    fn resolve_document(&mut self, mut doc: Document, base_path: Option<&Path>) -> Result<Document> {
        let includes = std::mem::take(&mut doc.includes);

        if base_path.is_none() && !includes.is_empty() {
            return Err(IcvsError::Validation {
                message: format!(
                    "Include directives require a file path context. Includes found: {}",
                    includes.join(", ")
                ),
            });
        }

        for include_path in &includes {
            let resolved_path = resolve_path(base_path, include_path);
            let canonical = resolved_path.canonicalize()
                .unwrap_or_else(|_| resolved_path.clone());

            if !self.visited.insert(canonical.clone()) {
                return Err(IcvsError::CircularInclude { path: include_path.clone().into() });
            }

            if !canonical.exists() {
                return Err(IcvsError::IncludeNotFound { path: resolved_path.clone() });
            }

            let content = std::fs::read_to_string(&canonical)
                .map_err(|e| IcvsError::Io {
                    path: canonical.clone(),
                    message: e.to_string(),
                })?;

            let included_doc = parser::parse_document_with_path(&content, Some(&canonical))?;

            let resolved_included = self.resolve_document(included_doc, canonical.parent())?;

            merge_documents(&mut doc, resolved_included)?;
        }

        Ok(doc)
    }
}

fn resolve_path(base: Option<&Path>, include: &str) -> PathBuf {
    match base {
        Some(base_dir) => {
            if Path::new(include).is_absolute() {
                PathBuf::from(include)
            } else {
                base_dir.join(include)
            }
        }
        None => PathBuf::from(include),
    }
}

fn merge_documents(target: &mut Document, source: Document) -> Result<()> {
    if target.project_name.is_none() {
        target.project_name = source.project_name;
    }

    for (id, node) in source.nodes {
        if target.nodes.contains_key(&id) {
            return Err(IcvsError::DuplicateNode {
                node: id.clone(),
                first: target.nodes[&id].source_line,
                second: node.source_line,
            });
        }
        target.nodes.insert(id, node);
    }

    target.edges.extend(source.edges);

    for (name, tgt) in source.targets {
        if target.targets.contains_key(&name) {
            let existing = &target.targets[&name];
            return Err(IcvsError::DuplicateTarget {
                target: name.clone(),
                first: existing.source_line,
                second: tgt.source_line,
            });
        }
        target.targets.insert(name, tgt);
    }

    target.includes.extend(source.includes);

    Ok(())
}

pub fn resolve_file(path: &Path) -> Result<Document> {
    let doc = parser::parse_file(path)?;
    let parent = path.parent();
    resolve(doc, parent)
}
