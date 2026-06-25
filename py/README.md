# instructcanvas

Python bindings for **InstructCanvas** (.icvs) — a DAG-based instruction format for agentic AI tools.

## Install

```bash
pip install instructcanvas
```

## Usage

```python
import instructcanvas as icvs

source = """[node: rule1]
  type = rule
  content = "Use 2-space indentation"
  severity = must

[edge: rule1 -> rule2]

[target: claude]
  resolve = [rule1]
"""

# Parse
doc = icvs.parse(source)
print(f"Nodes: {doc.node_count()}, Edges: {doc.edge_count()}")

# Validate
result = icvs.validate(source)
print("Valid" if result.is_valid else "Invalid")

# Export Markdown
md = icvs.export_markdown(source, "claude")
print(md)

# Export DOT graph
dot = icvs.export_dot(source)

# .icvs ↔ Markdown
icvs_back = icvs.md_to_icvs("# Hello")
md_back = icvs.icvs_to_md(source)

# Convert to AI agent format
agent_json = icvs.convert_agent(source, "claude")

# Apply template variables
result = icvs.apply_template(source, {"FRAMEWORK": "React"})
```

## API

| Function | Description |
|----------|-------------|
| `parse(input)` | Parse .icvs → Document |
| `validate(input)` | Validate → `ValidationReport` |
| `export_markdown(input, target)` | Export per-target Markdown |
| `export_dot(input)` | Export DOT graph |
| `export_merge(input)` | Merge all nodes → Markdown |
| `md_to_icvs(markdown)` | Convert Markdown → .icvs |
| `icvs_to_md(input)` | Convert .icvs → Markdown |
| `convert_agent(input, format)` | Export as Claude/OpenAI/JSON |
| `apply_template(input, vars_dict)` | Apply `{{ VAR }}` template |

## CLI (Rust)

```bash
cargo install icvs
icvs validate rules.icvs
icvs convert --target claude rules.icvs
```

## License

MIT
