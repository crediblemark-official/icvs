# API Reference

## Rust (Core Library)

```toml
[dependencies]
icvs = { git = "https://github.com/your-org/icvs" }
```

### `parser::parse_document(input: &str) -> Result<Document>`

Parse string `.icvs` menjadi AST `Document`.

```rust
use icvs::parser;

let doc = parser::parse_document(r#"
    [node: example]
      type = rule
      content = "Example rule"
      severity = must
"#)?;
println!("Nodes: {}", doc.nodes.len());
```

### `parser::parse_file(path: &Path) -> Result<Document>`

Parse file `.icvs` dari filesystem.

### `validator::validate(doc: &Document) -> Result<ValidationReport>`

Validasi struktur DAG, deteksi cycle, orphan nodes, unresolved references.

```rust
use icvs::validator;

let doc = parser::parse_file(Path::new("rules.icvs"))?;
let report = validator::validate(&doc)?;
println!("Valid: {}", report.is_valid);
```

### `validator::topological_sort(doc: &Document) -> Result<Vec<String>>`

Kahn's algorithm. Error jika ada cycle.

### `resolver::resolve(doc: Document, base_path: Option<&Path>) -> Result<Document>`

Resolve semua `[include:]` directives secara rekursif.

### `resolver::resolve_file(path: &Path) -> Result<Document>`

Parse + resolve include dalam satu langkah.

### `exporter::export_markdown(doc: &Document, target: &str) -> Result<String>`

Export Markdown untuk target tertentu (filter resolve/ignore).

### `exporter::export_dot(doc: &Document) -> Result<String>`

Export DOT graph dengan warna per node type.

### `exporter::export_markdown_merge(doc: &Document) -> Result<String>`

Export semua node tanpa filter target.

### Struct `ast::Document`

```rust
Document {
    pub project_name: Option<String>,
    pub includes: Vec<String>,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
    pub targets: HashMap<String, Target>,
    pub source_path: Option<PathBuf>,
}
```

### Struct `ast::Node`

```rust
Node {
    pub id: String,
    pub node_type: NodeType,        // Rule | Blocklist | Allowlist | Condition | Action
    pub content: Option<String>,
    pub severity: Option<Severity>,  // Must | Should | May
    pub trigger_on: Option<TriggerOn>, // Import | Install | Run
    pub condition: Option<Condition>,
    pub source_line: usize,
}
```

---

## WASM (Browser / IDE Plugin)

### Instalasi

```bash
npm install instructcanvas-wasm
# atau
yarn add instructcanvas-wasm
```

### Usage

```js
import init, {
  parse,
  validate,
  exportMarkdown,
  exportDot,
  exportMerge
} from 'icvs-wasm';

await init();

const source = `
  [node: hello]
    type = rule
    content = "Hello WASM"
    severity = must

  [target: all]
    resolve = [hello]
`;

// Parse → JSON
const doc = parse(source);
console.log(doc.nodes, doc.edges);

// Validate → { is_valid, errors, warnings, ... }
const result = validate(source);
console.log(result.is_valid, result.errors);

// Export
const md = exportMarkdown(source, 'claude');
const dot = exportDot(source);
const merged = exportMerge(source);
```

### API

| Function | Input | Output | Description |
|---|---|---|---|
| `parse(input)` | `string` | `JSON string` | Parse .icvs → Document JSON |
| `validate(input)` | `string` | `JSON string` | Validate → report JSON |
| `exportMarkdown(input, target)` | `string, string` | `string` | Export Markdown for target |
| `exportDot(input)` | `string` | `string` | Export DOT graph |
| `exportMerge(input)` | `string` | `string` | Merge all nodes |

---

## Python

### Instalasi

```bash
pip install instructcanvas
```

### Usage

```python
import instructcanvas as icvs

# Parse
doc = icvs.parse("""
[node: hello]
  type = rule
  content = "Hello Python"
  severity = must
""")
print(f"Nodes: {doc.node_count()}, Edge: {doc.edge_count()}")
print(f"Node IDs: {doc.node_ids()}")

# File
doc = icvs.parse_file("path/to/rules.icvs")

# Validate
result = icvs.validate("""
[node: a]
  type = rule
  content = "A"
[node: b]
  type = rule
  content = "B"
[edge: a -> b]
""")
print(f"Valid: {result.is_valid}")
print(f"Errors: {result.errors}")
print(f"Warnings: {result.warnings}")

# Validate file
result = icvs.validate_file("path/to/rules.icvs")

# Export
md = icvs.export_markdown(source, "claude")
dot = icvs.export_dot(source)
merged = icvs.export_merge(source)
```

### API

| Function | Input | Returns |
|---|---|---|
| `parse(source)` | `str` | `IcvsDocument` |
| `parse_file(path)` | `str` | `IcvsDocument` |
| `validate(source)` | `str` | `ValidationResult` |
| `validate_file(path)` | `str` | `ValidationResult` |
| `export_markdown(source, target)` | `str, str` | `str` (Markdown) |
| `export_dot(source)` | `str` | `str` (DOT) |
| `export_merge(source)` | `str` | `str` (Markdown) |

### Class `IcvsDocument`

| Method | Returns |
|---|---|
| `node_count()` | `int` |
| `edge_count()` | `int` |
| `target_count()` | `int` |
| `project_name()` | `Optional[str]` |
| `node_ids()` | `List[str]` |

### Class `ValidationResult`

| Field | Type |
|---|---|
| `is_valid` | `bool` |
| `node_count` | `int` |
| `edge_count` | `int` |
| `errors` | `List[str]` |
| `warnings` | `List[str]` |
| `orphan_nodes` | `List[str]` |
