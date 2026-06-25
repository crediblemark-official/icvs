use std::path::Path;

#[test]
fn test_parse_simple_file() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).expect("Failed to parse simple.icvs");

    assert_eq!(doc.nodes.len(), 3);
    assert_eq!(doc.edges.len(), 2);
    assert_eq!(doc.targets.len(), 2);
    assert_eq!(doc.project_name.as_deref(), Some("my-api"));
    assert_eq!(doc.includes.len(), 0);

    let style = doc.nodes.get("coding_style").unwrap();
    assert_eq!(style.node_type, icvs::ast::NodeType::Rule);
    assert_eq!(style.content.as_deref(), Some("Use 2-space indentation"));
    assert_eq!(style.severity, Some(icvs::ast::Severity::Must));
}

#[test]
fn test_parse_with_condition() {
    let path = Path::new("tests/fixtures/with-condition.icvs");
    let doc = icvs::parser::parse_file(path).expect("Failed to parse with-condition.icvs");

    assert_eq!(doc.nodes.len(), 3);
    let cond = doc.nodes.get("prod_check").unwrap();
    assert_eq!(cond.node_type, icvs::ast::NodeType::Condition);
    assert!(cond.condition.is_some());

    let condition = cond.condition.as_ref().unwrap();
    assert_eq!(condition.variable, "BRANCH");
    assert_eq!(condition.operator, "==");
    assert_eq!(condition.value, "main");
    assert_eq!(condition.then_node, "strict_lint");
    assert_eq!(condition.else_node.as_deref(), Some("relaxed_lint"));
}

#[test]
fn test_validate_valid() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let report = icvs::validator::validate(&doc).unwrap();

    assert!(report.is_valid);
    assert!(report.errors.is_empty());
    assert_eq!(report.node_count, 3);
    assert_eq!(report.edge_count, 2);
}

#[test]
fn test_validate_cycle_detection() {
    let path = Path::new("tests/fixtures/invalid-cycle.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let report = icvs::validator::validate(&doc).unwrap();

    assert!(!report.is_valid);
    assert!(!report.errors.is_empty());

    let has_cycle_error = report.errors.iter().any(|e| matches!(e, icvs::error::IcvsError::CycleDetected { .. }));
    assert!(has_cycle_error, "Expected cycle detection error");
}

#[test]
fn test_topological_sort() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let sorted = icvs::validator::topological_sort(&doc).unwrap();

    assert_eq!(sorted.len(), 3);

    let coding_idx = sorted.iter().position(|n| n == "coding_style").unwrap();
    let forbidden_idx = sorted.iter().position(|n| n == "forbidden_libs").unwrap();
    let testing_idx = sorted.iter().position(|n| n == "testing_rule").unwrap();

    assert!(coding_idx < forbidden_idx, "coding_style should be before forbidden_libs");
    assert!(forbidden_idx < testing_idx, "forbidden_libs should be before testing_rule");
}

#[test]
fn test_export_markdown() {
    let path = Path::new("tests/fixtures/with-targets.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let _report = icvs::validator::validate(&doc).unwrap();

    let md = icvs::exporter::export_markdown(&doc, "claude").unwrap();
    assert!(md.contains("style"));
    assert!(md.contains("Use Prettier"));
    assert!(md.contains("claude"));
    assert!(md.contains("security"));

    let md_cursor = icvs::exporter::export_markdown(&doc, "cursor").unwrap();
    assert!(md_cursor.contains("style"));
    assert!(!md_cursor.contains("security"), "cursor should not include security node");
}

#[test]
fn test_export_dot() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();

    let dot = icvs::exporter::export_dot(&doc).unwrap();
    assert!(dot.starts_with("digraph"));
    assert!(dot.contains("coding_style"));
    assert!(dot.contains("forbidden_libs"));
    assert!(dot.contains("->"));
}

#[test]
fn test_export_merge() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let _report = icvs::validator::validate(&doc).unwrap();

    let merged = icvs::exporter::export_markdown_merge(&doc).unwrap();
    assert!(merged.contains("my-api"));
    assert!(merged.contains("Use 2-space indentation"));
    assert!(merged.contains("lodash"));
}

#[test]
fn test_parse_error_on_stray_text() {
    let path = Path::new("tests/fixtures/parse-error.icvs");
    let result = icvs::parser::parse_file(path);
    assert!(result.is_err(), "Expected parse error for stray text");
}

#[test]
fn test_duplicate_node_detection() {
    let input = r#"
[node: dup]
  type = rule
  content = "First"

[node: dup]
  type = rule
  content = "Second"
"#;
    let result = icvs::parser::parse_document(input);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, icvs::error::IcvsError::DuplicateNode { .. }));
}

#[test]
fn test_missing_node_in_edge() {
    let input = r#"
[node: a]
  type = rule
  content = "Node A"

[edge: a -> nonexistent]
"#;
    let doc = icvs::parser::parse_document(input).unwrap();
    let report = icvs::validator::validate(&doc).unwrap();
    assert!(!report.is_valid);
    assert!(report.errors.iter().any(|e| matches!(e, icvs::error::IcvsError::NodeNotFound { .. })));
}

#[test]
fn test_target_not_found() {
    let path = Path::new("tests/fixtures/simple.icvs");
    let doc = icvs::parser::parse_file(path).unwrap();
    let result = icvs::exporter::export_markdown(&doc, "nonexistent");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), icvs::error::IcvsError::TargetNotFound { .. }));
}

#[test]
fn test_empty_document() {
    let input = "# just a comment\n";
    let doc = icvs::parser::parse_document(input).unwrap();
    assert_eq!(doc.nodes.len(), 0);
    assert_eq!(doc.edges.len(), 0);
    assert_eq!(doc.targets.len(), 0);
}

#[test]
fn test_invalid_node_type() {
    let input = r#"
[node: bad]
  type = invalid_type
  content = "test"
"#;
    let result = icvs::parser::parse_document(input);
    assert!(result.is_err());
}

#[test]
fn test_resolve_includes() {
    let path = Path::new("tests/fixtures/include-root.icvs");
    let doc = icvs::resolver::resolve_file(path).expect("Failed to resolve includes");

    assert_eq!(doc.nodes.len(), 2);
    assert!(doc.nodes.contains_key("root_node"));
    assert!(doc.nodes.contains_key("child_node"));

    assert!(doc.targets.contains_key("all"));
    assert!(doc.targets.contains_key("child_only"));

    assert_eq!(doc.project_name.as_deref(), Some("included-project"));
}

#[test]
fn test_resolve_circular_include_error() {
    let path = Path::new("tests/fixtures/include-circular-a.icvs");
    let result = icvs::resolver::resolve_file(path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, icvs::error::IcvsError::CircularInclude { .. }),
        "Expected CircularInclude error, got: {:?}", err);
}

#[test]
fn test_resolve_missing_include_error() {
    let path = Path::new("tests/fixtures/include-missing.icvs");
    let result = icvs::resolver::resolve_file(path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, icvs::error::IcvsError::IncludeNotFound { .. }),
        "Expected IncludeNotFound error, got: {:?}", err);
}

#[test]
fn test_multiline_target_list() {
    let input = r#"
[node: a]
  type = rule
  content = "A"
  severity = must

[node: b]
  type = rule
  content = "B"
  severity = should

[node: c]
  type = rule
  content = "C"
  severity = may

[target: all]
  resolve = [
    a, b,
    c
  ]
  ignore = [
    c
  ]
"#;
    let doc = icvs::parser::parse_document(input).unwrap();
    let target = doc.targets.get("all").unwrap();
    assert_eq!(target.resolve.as_ref().unwrap(), &vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(target.ignore.as_ref().unwrap(), &vec!["c".to_string()]);
}

#[test]
fn test_condition_without_then() {
    let input = r#"
[node: cond]
  type = condition
  if = $ENV == "prod"
"#;
    let doc = icvs::parser::parse_document(input).unwrap();
    let cond_node = doc.nodes.get("cond").unwrap();
    assert!(cond_node.condition.is_some());
    let cond = cond_node.condition.as_ref().unwrap();
    assert_eq!(cond.then_node, "");
}
