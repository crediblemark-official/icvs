# InstructCanvas (.icvs)

[![crates.io](https://img.shields.io/crates/v/icvs)](https://crates.io/crates/icvs)
[![npm](https://img.shields.io/npm/v/icvs-wasm)](https://www.npmjs.com/package/icvs-wasm)
[![PyPI](https://img.shields.io/pypi/v/instructcanvas)](https://pypi.org/project/instructcanvas/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**A Directed Acyclic Graph (DAG) based instruction format for agentic AI tools.**

InstructCanvas is a new instruction file format (`.icvs`) that addresses fundamental shortcomings of Markdown for instructing AI coding agents. It represents instructions as a **DAG** in plain text syntax — human-readable, machine-precise.

```bash
npm install -g icvs-wasm            # WASM (browser/IDE)
pip install instructcanvas          # Python
cargo install icvs                  # CLI (Rust)
```

## Quick Start

```bash
# Install CLI
cargo install icvs

# Validate a file
icvs validate rules.icvs

# Export for a specific target
icvs export --target claude rules.icvs

# View dependency graph
icvs visualize rules.icvs
```

Minimal `.icvs` file example:

```plaintext
[node: coding_style]
  type = rule
  content = "Use 2-space indentation"
  severity = must

[edge: coding_style -> testing_rule]

[node: testing_rule]
  type = rule
  content = "All functions must have unit tests"
  severity = must

[target: claude]
  resolve = [coding_style, testing_rule]
```

## Why InstructCanvas?

| Markdown Problem | .icvs Solution |
|-----------------|----------------|
| Rules written in ambiguous prose | Precise, well-defined DAG structure |
| Files bloat to thousands of lines | Modular with `[include:]` |
| Conditional logic impossible | `[node: type = condition]` with `$VARIABLE` |
| Must rewrite per tool | One file for all targets (`claude`, `copilot`, `cursor`) |
| Prone to prompt injection | Strict parser, no hidden content |

## Basic Syntax

```plaintext
# Comments

# Project metadata
#project: "my-api"

# Include another file
[include: "./style.icvs"]

# Node definition
[node: <id>]
  type = rule|blocklist|allowlist|condition|action
  content = "Instructions..."
  severity = must|should|may          # for rule
  trigger_on = import|install|run     # for blocklist/allowlist
  if = $VARIABLE == "value"           # for condition
    then = -> target_node
    else = -> fallback_node

# Edge / dependency
[edge: <source> -> <target>]
[edge: <source> -> <target> with "label"]   # Edge with label

# Template include (EXPERIMENTAL)
[include: <path> @ <template>]

# Template variables (replaced at runtime)
  content = "Use {{ FRAMEWORK }} for this"

# Target tool
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

## CLI

```bash
icvs validate <file>                    # Validate + cycle detection
icvs export --target <name> <file>      # Export Markdown per target
icvs visualize <file>                   # Output DOT graph
icvs merge <file>                       # Merge all nodes into one document

icvs md-to-icvs <file.md>               # Markdown → .icvs
icvs icvs-to-md <file.icvs>             # .icvs → Markdown

icvs convert --target claude <file>     # → Claude tool JSON
icvs convert --target openai <file>     # → OpenAI function calling
icvs convert --target json <file>       # → Generic JSON

icvs template -D FRAMEWORK=React <file> # Apply {{ VAR }} template

icvs benchmark <file>                   # Performance & DAG metrics
```

## Library

### Python
```python
import instructcanvas as icvs

# Parse
doc = icvs.parse(source)
print(doc.node_count(), doc.edge_count())

# Validate
result = icvs.validate(source)
print(result.is_valid, result.errors)

# Export
md = icvs.export_markdown(source, 'claude')
dot = icvs.export_dot(source)
merged = icvs.export_merge(source)

# Convert
icvs_raw = icvs.md_to_icvs(markdown_source)
md_back = icvs.icvs_to_md(source)
agent_json = icvs.convert_agent(source, 'claude')
result = icvs.apply_template(source, {"FRAMEWORK": "React"})
```

### WASM (Browser/IDE)
```js
import init, { parse, validate, exportMarkdown, exportDot, mdToIcvs, icvsToMd, convertAgent, applyTemplate } from 'icvs-wasm';

await init();
const doc = parse(source);
const result = validate(source);
const md = exportMarkdown(source, 'claude');
const dot = exportDot(source);
const icvs = mdToIcvs(markdownSource);
const agentTools = convertAgent(source, 'claude');

// Template
const output = applyTemplate(source, '{"FRAMEWORK": "React"}');
```

## Full Documentation

- [Syntax Reference](docs/syntax.md) — .icvs grammar details
- [CLI Guide](docs/cli.md) — All commands and options
- [API Reference](docs/api.md) — Rust, WASM, Python
- [Examples](docs/examples.md) — Real-world usage examples

## Development

```bash
git clone https://github.com/crediblemark-official/icvs.git
cd icvs

# Build & test
cargo build
cargo test

# WASM
wasm-pack build wasm/

# Python
cd py && maturin build --release
```

## License

MIT
