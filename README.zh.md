# InstructCanvas (.icvs)

[![crates.io](https://img.shields.io/crates/v/icvs)](https://crates.io/crates/icvs)
[![npm](https://img.shields.io/npm/v/icvs-wasm)](https://www.npmjs.com/package/icvs-wasm)
[![PyPI](https://img.shields.io/pypi/v/instructcanvas)](https://pypi.org/project/instructcanvas/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

**基于有向无环图（DAG）的 AI 代理指令格式。**

InstructCanvas 是一种全新的指令文件格式（`.icvs`），它解决了 Markdown 在指导 AI 编码代理时的根本性缺陷。它以纯文本语法将指令表示为 **DAG** — 人类可读，机器精确。

```bash
npm install -g icvs-wasm            # WASM（浏览器/IDE）
pip install instructcanvas          # Python
cargo install icvs                  # CLI（Rust）
```

## 快速入门

```bash
# 安装 CLI
cargo install icvs

# 验证文件
icvs validate rules.icvs

# 为特定目标导出
icvs export --target claude rules.icvs

# 查看依赖图
icvs visualize rules.icvs
```

最小 `.icvs` 文件示例：

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

## 为什么选择 InstructCanvas？

| Markdown 问题 | .icvs 解决方案 |
|---------------|----------------|
| 规则用模糊的散文书写 | 精确、明确定义的 DAG 结构 |
| 文件膨胀到数千行 | 通过 `[include:]` 实现模块化 |
| 无法实现条件逻辑 | `[node: type = condition]` 配合 `$VARIABLE` |
| 每个工具需要重写 | 一个文件适用于所有目标（`claude`、`copilot`、`cursor`） |
| 容易受到提示注入攻击 | 严格解析器，无隐藏内容 |

## 基本语法

```plaintext
# 注释

# 项目元数据
#project: "my-api"

# 包含另一个文件
[include: "./style.icvs"]

# 节点定义
[node: <id>]
  type = rule|blocklist|allowlist|condition|action
  content = "指令..."
  severity = must|should|may          # 用于 rule
  trigger_on = import|install|run     # 用于 blocklist/allowlist
  if = $VARIABLE == "value"           # 用于 condition
    then = -> target_node
    else = -> fallback_node

# 边 / 依赖
[edge: <source> -> <target>]
[edge: <source> -> <target> with "label"]   # 带标签的边

# 模板包含（实验性）
[include: <path> @ <template>]

# 模板变量（运行时替换）
  content = "Use {{ FRAMEWORK }} for this"

# 目标工具
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

## CLI

```bash
icvs validate <file>                    # 验证 + 循环检测
icvs export --target <name> <file>      # 按目标导出 Markdown
icvs visualize <file>                   # 输出 DOT 图
icvs merge <file>                       # 将所有节点合并为一个文档

icvs md-to-icvs <file.md>               # Markdown → .icvs
icvs icvs-to-md <file.icvs>             # .icvs → Markdown

icvs convert --target claude <file>     # → Claude tool JSON
icvs convert --target openai <file>     # → OpenAI function calling
icvs convert --target json <file>       # → Generic JSON

icvs template -D FRAMEWORK=React <file> # 应用 {{ VAR }} 模板

icvs benchmark <file>                   # 性能与 DAG 指标
```

## 库

### Python
```python
import instructcanvas as icvs

# 解析
doc = icvs.parse(source)
print(doc.node_count(), doc.edge_count())

# 验证
result = icvs.validate(source)
print(result.is_valid, result.errors)

# 导出
md = icvs.export_markdown(source, 'claude')
dot = icvs.export_dot(source)
merged = icvs.export_merge(source)

# 转换
icvs_raw = icvs.md_to_icvs(markdown_source)
md_back = icvs.icvs_to_md(source)
agent_json = icvs.convert_agent(source, 'claude')
result = icvs.apply_template(source, {"FRAMEWORK": "React"})
```

### WASM（浏览器/IDE）
```js
import init, { parse, validate, exportMarkdown, exportDot, mdToIcvs, icvsToMd, convertAgent, applyTemplate } from 'icvs-wasm';

await init();
const doc = parse(source);
const result = validate(source);
const md = exportMarkdown(source, 'claude');
const dot = exportDot(source);
const icvs = mdToIcvs(markdownSource);
const agentTools = convertAgent(source, 'claude');

// 模板
const output = applyTemplate(source, '{"FRAMEWORK": "React"}');
```

## 完整文档

- [语法参考](docs/syntax.md) — .icvs 语法详情
- [CLI 指南](docs/cli.md) — 所有命令和选项
- [API 参考](docs/api.md) — Rust、WASM、Python
- [示例](docs/examples.md) — 真实使用案例

## 开发

```bash
git clone https://github.com/crediblemark-official/icvs.git
cd icvs

# 构建和测试
cargo build
cargo test

# WASM
wasm-pack build wasm/

# Python
cd py && maturin build --release
```

## 许可证

MIT
