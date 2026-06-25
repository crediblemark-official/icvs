use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum IcvsError {
    Io { path: PathBuf, message: String },
    Parse { line: usize, message: String },
    Validation { message: String },
    CycleDetected { cycle: Vec<String> },
    NodeNotFound { node: String, referenced_from: String },
    TargetNotFound { target: String },
    IncludeNotFound { path: PathBuf },
    DuplicateNode { node: String, first: usize, second: usize },
    CircularInclude { path: PathBuf },
    DuplicateTarget { target: String, first: usize, second: usize },
}

impl fmt::Display for IcvsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IcvsError::Io { path, message } => {
                write!(f, "I/O error reading {}: {}", path.display(), message)
            }
            IcvsError::Parse { line, message } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }
            IcvsError::Validation { message } => {
                write!(f, "Validation error: {}", message)
            }
            IcvsError::CycleDetected { cycle } => {
                write!(f, "Cycle detected: {}", cycle.join(" -> "))
            }
            IcvsError::NodeNotFound { node, referenced_from } => {
                write!(f, "Node '{}' referenced from '{}' not found", node, referenced_from)
            }
            IcvsError::TargetNotFound { target } => {
                write!(f, "Target '{}' not defined in this document", target)
            }
            IcvsError::IncludeNotFound { path } => {
                write!(f, "Included file not found: {}", path.display())
            }
            IcvsError::DuplicateNode { node, first, second } => {
                write!(f, "Duplicate node '{}' (first defined at line {}, second at line {})",
                    node, first, second)
            }
            IcvsError::CircularInclude { path } => {
                write!(f, "Circular include detected: {}", path.display())
            }
            IcvsError::DuplicateTarget { target, first, second } => {
                write!(f, "Duplicate target '{}' (first defined at line {}, second at line {})",
                    target, first, second)
            }
        }
    }
}

impl std::error::Error for IcvsError {}

pub type Result<T> = std::result::Result<T, IcvsError>;
