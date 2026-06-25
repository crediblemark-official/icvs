use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Rule,
    Blocklist,
    Allowlist,
    Condition,
    Action,
}

impl NodeType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rule" => Some(Self::Rule),
            "blocklist" => Some(Self::Blocklist),
            "allowlist" => Some(Self::Allowlist),
            "condition" => Some(Self::Condition),
            "action" => Some(Self::Action),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rule => "rule",
            Self::Blocklist => "blocklist",
            Self::Allowlist => "allowlist",
            Self::Condition => "condition",
            Self::Action => "action",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Must,
    Should,
    May,
}

impl Severity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "must" => Some(Self::Must),
            "should" => Some(Self::Should),
            "may" => Some(Self::May),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Must => "must",
            Self::Should => "should",
            Self::May => "may",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TriggerOn {
    Import,
    Install,
    Run,
}

impl TriggerOn {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "import" => Some(Self::Import),
            "install" => Some(Self::Install),
            "run" => Some(Self::Run),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Install => "install",
            Self::Run => "run",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub variable: String,
    pub operator: String,
    pub value: String,
    pub then_node: String,
    pub else_node: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub content: Option<String>,
    pub severity: Option<Severity>,
    pub trigger_on: Option<TriggerOn>,
    pub condition: Option<Condition>,
    pub source_line: usize,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
    pub source_line: usize,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub resolve: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub source_line: usize,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub project_name: Option<String>,
    pub includes: Vec<String>,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
    pub targets: HashMap<String, Target>,
    pub source_path: Option<PathBuf>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            project_name: None,
            includes: Vec::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            targets: HashMap::new(),
            source_path: None,
        }
    }

    pub fn get_node_ids(&self) -> Vec<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_includes(&self) -> &[String] {
        &self.includes
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
