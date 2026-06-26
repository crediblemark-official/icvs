# API 参考

## Rust（核心库）

```toml
[dependencies]
icvs = { git = "https://github.com/crediblemark-official/icvs" }
```

### `parser::parse_document(input: &str) -> Result<Document>`

将 `.icvs` 字符串解析为 AST `Document`。

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

从文件系统解析 `.icvs` 文件。

### `validator::validate(doc: &Document) -> Result<ValidationReport>`

验证 DAG 结构、检测循环、孤立节点和未解析引用。

```rust
use icvs::validator;

let doc = parser::parse_file(Path::new("rules.icvs"))?;
let report = validator::validate(&doc)?;
println!("Valid: {}", report.is_valid);
```

### `validator::topological_sort(doc: &Document) -> Result<Vec<String>>`

Kahn 算法。如果存在循环则返回错误。

### `resolver::resolve(doc: Document, base_path: Option<&Path>) -> Result<Document>`

递归解析所有 `[include:]` 指令。

### `resolver::resolve_file(path: &Path) -> Result<Document>`

在一个步骤中解析文件并处理包含。

### `exporter::export_markdown(doc: &Document, target: &str) -> Result<String>`

为特定目标导出 Markdown（应用 resolve/ignore 过滤）。

### `exporter::export_dot(doc: &Document) -> Result<String>`

导出带节点类型颜色的 DOT 图。

### `exporter::export_markdown_merge(doc: &Document) -> Result<String>`

导出所有节点，不进行目标过滤。

### 结构体 `ast::Document`

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

### 结构体 `ast::Node`

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

## WASM（浏览器 / IDE 插件）

### 安装

```bash
npm install icvs-wasm
# 或
yarn add icvs-wasm
```

### 使用方法

```js
import init, {
  parse,
  validate,
  exportMarkdown,
  exportMerge,
  convertAgent,
  applyTemplate
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

// 解析 → JSON
const doc = parse(source);
console.log(doc.nodes, doc.edges);

// 验证 → { is_valid, errors, warnings, ... }
const result = validate(source);
console.log(result.is_valid, result.errors);

// 导出
const md = exportMarkdown(source, 'claude');
const dot = exportDot(source);
const merged = exportMerge(source);

// 转换为代理格式
const tools = convertAgent(source, 'claude');

// 模板
const output = applyTemplate(source, '{"FRAMEWORK": "React"}');
```

### API

| 函数 | 输入 | 输出 | 描述 |
|------|------|------|------|
| `parse(input)` | `string` | `JSON string` | 解析 .icvs → Document JSON |
| `parseAndResolve(input)` | `string` | `JSON string` | 解析 + 解析包含 |
| `validate(input)` | `string` | `JSON string` | 验证 → 报告 JSON |
| `exportMarkdown(input, target)` | `string, string` | `string` | 为目标导出 Markdown |
| `exportDot(input)` | `string` | `string` | 导出 DOT 图 |
| `exportMerge(input)` | `string` | `string` | 合并所有节点 |
| `mdToIcvs(markdown)` | `string` | `string` | 将 Markdown 转换为 .icvs |
| `icvsToMd(input)` | `string` | `string` | 将 .icvs 转换为 Markdown |
| `convertAgent(input, format)` | `string, string` | `string` | 导出为 Claude/OpenAI/JSON |
| `applyTemplate(input, varsJson)` | `string, string` | `string` | 应用 `{{ VAR }}` 模板 |

---

## Python

### 安装

```bash
pip install instructcanvas
```

### 使用方法

```python
import instructcanvas as icvs

# 解析
doc = icvs.parse("""
[node: hello]
  type = rule
  content = "Hello Python"
  severity = must
""")
print(f"Nodes: {doc.node_count()}, Edges: {doc.edge_count()}")
print(f"Node IDs: {doc.node_ids()}")

# 文件
doc = icvs.parse_file("path/to/rules.icvs")

# 验证
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

# 验证文件
result = icvs.validate_file("path/to/rules.icvs")

# 导出
md = icvs.export_markdown(source, "claude")
dot = icvs.export_dot(source)
merged = icvs.export_merge(source)

# 转换
agent = icvs.convert_agent(source, "claude")

# 模板
result = icvs.apply_template(source, {"FRAMEWORK": "React"})
```

### API

| 函数 | 输入 | 返回 |
|------|------|------|
| `parse(source)` | `str` | `IcvsDocument` |
| `parse_file(path)` | `str` | `IcvsDocument` |
| `validate(source)` | `str` | `ValidationResult` |
| `validate_file(path)` | `str` | `ValidationResult` |
| `export_markdown(source, target)` | `str, str` | `str` (Markdown) |
| `export_dot(source)` | `str` | `str` (DOT) |
| `export_merge(source)` | `str` | `str` (Markdown) |
| `md_to_icvs(markdown)` | `str` | `str` (.icvs) |
| `icvs_to_md(source)` | `str` | `str` (Markdown) |
| `convert_agent(source, format)` | `str, str` | `str` (JSON) |
| `apply_template(source, vars)` | `str, dict` | `str` (.icvs) |

### 类 `IcvsDocument`

| 方法 | 返回 |
|------|------|
| `node_count()` | `int` |
| `edge_count()` | `int` |
| `target_count()` | `int` |
| `project_name()` | `Optional[str]` |
| `node_ids()` | `List[str]` |

### 类 `ValidationResult`

| 字段 | 类型 |
|------|------|
| `is_valid` | `bool` |
| `node_count` | `int` |
| `edge_count` | `int` |
| `errors` | `List[str]` |
| `warnings` | `List[str]` |
| `orphan_nodes` | `List[str]` |
