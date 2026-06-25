# Changelog

## 0.2.0 (2026-06-25)

### Added
- **Markdown ↔ Icvs converter** (`md-to-icvs`, `icvs-to-md` commands)
- **AI Agent format export** (`convert` command — Claude/OpenAI/JSON)
- **Template engine** (`template` command — `{{ VAR }}` substitutions)
- **Benchmark framework** (`benchmark` command — perf, info density, DAG metrics)
- **LSP server** (`icvs-lsp` — diagnostics, completion, hover, go-to-def, rename, symbols, folding)
- WASM bindings: `md_to_icvs`, `icvs_to_md`, `convert_agent`, `apply_template`
- Python bindings: `md_to_icvs`, `icvs_to_md`, `convert_agent`, `apply_template`
- CI workflow (build, test, WASM, Python, smoke tests)
- License: MIT

### Changed
- Workspace with `icvs`, `icvs-lsp`, `icvs-wasm`, `instructcanvas` crates

## 0.1.0 (2026-06-24)

### Added
- Core parser (5 node types, edges, targets, conditions, includes)
- DAG validator (cycle detection, orphan detection, ref checking)
- Include resolver (circular detection, merge)
- Markdown + DOT exporter (per-target, merged, graph viz)
- CLI: `validate`, `export`, `visualize`, `merge`
- WASM bindings: `parse`, `validate`, `exportMarkdown`, `exportDot`, `exportMerge`
- Python bindings: `parse`, `validate`, `export_markdown`, `export_dot`, `export_merge`
- VSCode extension with syntax highlighting and graph preview
- Documentation: syntax, CLI, API, examples
- Preview page: D3 graph, split editor, live update, export SVG/MD
