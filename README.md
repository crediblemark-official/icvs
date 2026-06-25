# InstructCanvas (.icvs)

[![crates.io](https://img.shields.io/crates/v/icvs)](https://crates.io/crates/icvs)
[![npm](https://img.shields.io/npm/v/icvs-wasm)](https://www.npmjs.com/package/icvs-wasm)
[![PyPI](https://img.shields.io/pypi/v/instructcanvas)](https://pypi.org/project/instructcanvas/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**Format instruksi agentic AI berbasis Directed Acyclic Graph (DAG).**

InstructCanvas adalah format berkas instruksi baru (`.icvs`) yang menggantikan kelemahan fundamental Markdown untuk menginstruksikan AI coding agent. Ia merepresentasikan instruksi sebagai **DAG** dalam sintaks teks polos — mudah dibaca manusia, presisi untuk mesin.

```bash
npm install -g icvs-wasm            # WASM (browser/IDE)
pip install instructcanvas          # Python
cargo install icvs                  # CLI (Rust)
```

## Quick Start

```bash
# Instalasi CLI
cargo install icvs

# Validasi file
icvs validate rules.icvs

# Export untuk target tertentu
icvs export --target claude rules.icvs

# Lihat dependency graph
icvs visualize rules.icvs
```

Contoh file `.icvs` minimal:

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

## Kenapa InstructCanvas?

| Masalah Markdown | Solusi .icvs |
|---|---|
| Aturan ditulis dalam prosa ambigu | Struktur DAG yang presisi dan terdefinisi |
| File membengkak ribuan baris | Modular dengan `[include:]` |
| Logika kondisional tidak mungkin | `[node: type = condition]` dengan `$VARIABLE` |
| Harus nulis ulang per tool | Satu file untuk semua target (`claude`, `copilot`, `cursor`) |
| Rawan prompt injection | Parser strict, no hidden content |

## Sintaks Dasar

```plaintext
# Komentar

# Metadata proyek
#project: "my-api"

# Include file lain
[include: "./style.icvs"]

# Definisi node
[node: <id>]
  type = rule|blocklist|allowlist|condition|action
  content = "Instruksi..."
  severity = must|should|may          # untuk rule
  trigger_on = import|install|run     # untuk blocklist/allowlist
  if = $VARIABLE == "value"           # untuk condition
    then = -> target_node
    else = -> fallback_node

# Edge / dependency
[edge: <source> -> <target>]
[edge: <source> -> <target> with "label"]   # Edge dengan label

# Template include (EXPERIMENTAL)
[include: <path> @ <template>]

# Template variables (diganti saat runtime)
  content = "Use {{ FRAMEWORK }} for this"

# Target tool
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

## CLI

```bash
icvs validate <file>                    # Validasi + deteksi cycle
icvs export --target <name> <file>      # Export Markdown per target
icvs visualize <file>                   # Output DOT graph
icvs merge <file>                       # Semua node jadi satu dokumen

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
const result = applyTemplate(source, '{"FRAMEWORK": "React"}');
```

## Dokumentasi Lengkap

- [Syntax Reference](docs/syntax.md) — Detail grammar .icvs
- [CLI Guide](docs/cli.md) — Semua perintah dan opsi
- [API Reference](docs/api.md) — Rust, WASM, Python
- [Examples](docs/examples.md) — Contoh penggunaan dari real project

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

## Lisensi

MIT
