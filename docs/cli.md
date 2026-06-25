# CLI Guide — `icvs`

## Instalasi

```bash
# Dari source (Rust)
cargo install --git https://github.com/your-org/icvs

# Atau download binary dari releases
curl -L https://github.com/your-org/icvs/releases/latest/download/icvs-x86_64-linux.tar.gz | tar xz
sudo mv icvs /usr/local/bin/
```

## Perintah

### `icvs validate`

Validasi satu atau lebih file `.icvs`.

```bash
icvs validate ./rules.icvs
icvs validate ./project.icvs --strict    # strict mode
```

Output sukses:
```
✅ Valid: ./rules.icvs
   Nodes: 6
   Edges: 5
   Orphan nodes: none
```

Output error (cycle detected):
```
❌ Invalid: ./rules.icvs
   Error: Cycle detected: a -> b -> c -> a
Error: Validation error: File has 1 error(s)
```

### `icvs export`

Export instruksi untuk target tertentu dalam format Markdown.

```bash
icvs export --target claude ./rules.icvs                # stdout
icvs export --target copilot ./rules.icvs --output ./copilot-instructions.md
```

Output adalah Markdown yang siap digunakan sebagai instruksi untuk AI agent.

### `icvs visualize`

Generate dependency graph dalam format DOT.

```bash
icvs visualize ./rules.icvs                             # stdout
icvs visualize ./rules.icvs --output ./graph.dot        # ke file
```

Untuk render ke SVG:
```bash
icvs visualize ./rules.icvs | dot -Tsvg -o graph.svg
```

### `icvs merge`

Gabung semua node menjadi satu dokumen Markdown (tanpa filter target).

```bash
icvs merge ./rules.icvs
icvs merge ./rules.icvs --output ./all-instructions.md
```

## Exit Codes

| Code | Arti |
|---|---|
| 0 | Sukses |
| 1 | Error (parse error, validation error, file not found, etc.) |

## Contoh Workflow

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
