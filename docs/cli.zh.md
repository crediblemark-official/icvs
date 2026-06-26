# CLI 指南 — `icvs`

## 安装

```bash
# 从源码安装（Rust）
cargo install --git https://github.com/crediblemark-official/icvs

# 或从发布页下载二进制文件
curl -L https://github.com/crediblemark-official/icvs/releases/latest/download/icvs-x86_64-linux.tar.gz | tar xz
sudo mv icvs /usr/local/bin/
```

## 命令

### `icvs validate`

验证一个或多个 `.icvs` 文件。

```bash
icvs validate ./rules.icvs
icvs validate ./project.icvs --strict    # 严格模式
```

成功输出：
```
✅ Valid: ./rules.icvs
   Nodes: 6
   Edges: 5
   Orphan nodes: none
```

错误输出（检测到循环）：
```
❌ Invalid: ./rules.icvs
   Error: Cycle detected: a -> b -> c -> a
Error: Validation error: File has 1 error(s)
```

### `icvs export`

以 Markdown 格式导出特定目标的指令。

```bash
icvs export --target claude ./rules.icvs                # stdout
icvs export --target copilot ./rules.icvs --output ./copilot-instructions.md
```

输出为可直接用作 AI 代理指令的 Markdown。

### `icvs visualize`

以 DOT 格式生成依赖图。

```bash
icvs visualize ./rules.icvs                             # stdout
icvs visualize ./rules.icvs --output ./graph.dot        # 输出到文件
```

渲染为 SVG：
```bash
icvs visualize ./rules.icvs | dot -Tsvg -o graph.svg
```

### `icvs merge`

将所有节点合并为单个 Markdown 文档（无目标过滤）。

```bash
icvs merge ./rules.icvs
icvs merge ./rules.icvs --output ./all-instructions.md
```

## 退出码

| 代码 | 含义 |
|------|------|
| 0 | 成功 |
| 1 | 错误（解析错误、验证错误、文件未找到等） |

## 工作流示例

### CI 流水线

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

### Pre-commit 钩子

```bash
#!/bin/sh
# .git/hooks/pre-commit
for file in $(git diff --cached --name-only | grep '\.icvs$'); do
    icvs validate "$file" || exit 1
done
```
