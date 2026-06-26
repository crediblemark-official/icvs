# CLI Guide — `icvs`

## Installation

```bash
# From source (Rust)
cargo install --git https://github.com/crediblemark-official/icvs

# Or download a binary from releases
curl -L https://github.com/crediblemark-official/icvs/releases/latest/download/icvs-x86_64-linux.tar.gz | tar xz
sudo mv icvs /usr/local/bin/
```

## Commands

### `icvs validate`

Validate one or more `.icvs` files.

```bash
icvs validate ./rules.icvs
icvs validate ./project.icvs --strict    # strict mode
```

Success output:
```
✅ Valid: ./rules.icvs
   Nodes: 6
   Edges: 5
   Orphan nodes: none
```

Error output (cycle detected):
```
❌ Invalid: ./rules.icvs
   Error: Cycle detected: a -> b -> c -> a
Error: Validation error: File has 1 error(s)
```

### `icvs export`

Export instructions for a specific target in Markdown format.

```bash
icvs export --target claude ./rules.icvs                # stdout
icvs export --target copilot ./rules.icvs --output ./copilot-instructions.md
```

The output is Markdown ready to use as instructions for an AI agent.

### `icvs visualize`

Generate a dependency graph in DOT format.

```bash
icvs visualize ./rules.icvs                             # stdout
icvs visualize ./rules.icvs --output ./graph.dot        # to file
```

Render to SVG:
```bash
icvs visualize ./rules.icvs | dot -Tsvg -o graph.svg
```

### `icvs merge`

Merge all nodes into a single Markdown document (no target filtering).

```bash
icvs merge ./rules.icvs
icvs merge ./rules.icvs --output ./all-instructions.md
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (parse error, validation error, file not found, etc.) |

## Example Workflows

### CI Pipeline

```yaml
# .github/workflows/icvs.yml
name: Validate InstructCanvas
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: cargo install icvs
      - run: icvs validate project.icvs
      - run: icvs export --target claude project.icvs --output CLAUDE.md
      - run: icvs export --target copilot project.icvs --output .github/copilot-instructions.md
```

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
for file in $(git diff --cached --name-only | grep '\.icvs$'); do
    icvs validate "$file" || exit 1
done
```
