# 语法参考 — InstructCanvas (.icvs)

## 基本格式

`.icvs` 文件是纯文本 UTF-8 格式，具有块结构和缩进。每一行属于以下类别之一：

1. **注释** — 以 `#` 开头
2. **块头** — 以 `[` 开头，以 `]` 结尾
3. **属性** — 属于前一个块头的缩进行
4. **空行** — 忽略

## 注释

```plaintext
# 这是注释
#project: "my-api"       # 不是注释 — 这是项目元数据
```

以 `#` 开头的行是注释。**除非**是 `#project:` — 这是特殊的元数据。

## 项目元数据

```plaintext
#project: "项目名称"
```

看起来像注释但实际上不是 — 它定义项目名称。每个文件只能有一个。

## 包含

```plaintext
[include: "path/to/file.icvs"]
```

模块化导入另一个 `.icvs` 文件。路径相对于源文件。解析器会检测循环包含。

## 节点

```plaintext
[node: <id>]
  type = <node_type>
  content = "<指令>"
  severity = <severity>       # 仅用于 type=rule
  trigger_on = <trigger>      # 仅用于 type=blocklist/allowlist
  if = $VARIABLE == "value"   # 仅用于 type=condition
    then = -> <node_id>
    else = -> <node_id>
```

### 节点类型

| 类型 | 描述 | 关键属性 |
|------|------|----------|
| `rule` | 编码规则 | `severity`, `content` |
| `blocklist` | 禁止的库/代码 | `trigger_on`, `content` |
| `allowlist` | 允许的库/代码 | `trigger_on`, `content` |
| `condition` | 基于环境变量的条件逻辑 | `if`, `then`, `else` |
| `action` | 代理必须执行的操作 | `content` |

### 严重级别

| 值 | 含义 |
|----|------|
| `must` | 必须 — 代理必须遵守 |
| `should` | 应该 — 强烈建议 |
| `may` | 可选 |

### 触发时机

| 值 | 含义 |
|----|------|
| `import` | 导入库时检查 |
| `install` | 安装包时检查 |
| `run` | 运行命令时检查 |

### 条件

```plaintext
[node: deploy_check]
  type = condition
  if = $BRANCH == "main"
    then = -> run_deploy
    else = -> skip_deploy
```

环境变量使用 `$` 前缀。支持的运算符：`==`、`!=`、`>=`、`<=`、`>`、`<`。

条件节点会隐式创建指向 `then` 和 `else` 节点的边。

## 边

```plaintext
[edge: <source_id> -> <target_id>]
[edge: <source_id> -> <target_id> with "label"]
```

定义节点之间的依赖关系。源节点在目标节点之前执行。图必须是**有向无环图（DAG）**— 验证器会拒绝循环。

## 目标

```plaintext
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

指定哪些节点适用于特定工具。

| 属性 | 描述 |
|------|------|
| `resolve` | 该目标包含的节点列表 |
| `ignore` | 排除的节点列表（resolve 的子集） |

示例：一个 `.icvs` 文件可以为不同工具设置不同目标：

```plaintext
[target: claude]
  resolve = [coding_style, forbidden_libs, prod_rule]

[target: copilot]
  resolve = [coding_style]
  ignore = [forbidden_libs]
```

## 验证规则

### 结构
- 每个节点必须有唯一的 `id`
- 边必须引用已存在的节点
- 图必须是 DAG（不允许有循环）
- 节点/目标块中的属性必须缩进

### 命名
- 节点 ID：字母数字、下划线、连字符（`^[a-zA-Z0-9_-]+$`）
- 目标名称：与节点 ID 规则相同

### 合并（包含）
- 不同文件中出现重复节点 ID → 错误
- 不同文件中出现重复目标名称 → 错误
- 循环包含 → 错误

## EBNF 语法

```ebnf
document     = { comment | include | node | edge | target | project } ;
comment      = "#" , { character } , newline ;
include      = "[include:" , string , "]" ;
node         = "[node:" , identifier , "]" , newline , { attribute } ;
edge         = "[edge:" , identifier , "->" , identifier , "]" ;
target       = "[target:" , identifier , "]" , newline , { target_attr } ;
project      = "#project:" , string ;

attribute    = ( "type" | "content" | "severity" | "trigger_on"
              | "if" | "then" | "else" ) , "=" , value ;
target_attr  = ( "resolve" | "ignore" ) , "=" , "[" , identifier , { "," , identifier } , "]" ;

value        = string | identifier | condition | arrow_ref ;
condition    = "$" , identifier , operator , string ;
arrow_ref    = "->" , identifier ;
operator     = "==" | "!=" | ">=" | "<=" | ">" | "<" ;
identifier   = ( letter | "_" ) , { letter | digit | "_" | "-" } ;
string       = "\"" , { character } , "\"" ;
```
