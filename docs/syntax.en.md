# Syntax Reference — InstructCanvas (.icvs)

## Basic Format

An `.icvs` file is plain UTF-8 text with block structure and indentation. Each line falls into one of these categories:

1. **Comment** — starts with `#`
2. **Block header** — starts with `[` and ends with `]`
3. **Attribute** — indented line belonging to the previous block header
4. **Empty line** — ignored

## Comments

```plaintext
# This is a comment
#project: "my-api"       # NOT a comment — this is project metadata
```

Lines starting with `#` are comments. **Except** `#project:` which is special metadata.

## Project Metadata

```plaintext
#project: "project-name"
```

Looks like a comment but isn't — defines the project name. Only one per file.

## Include

```plaintext
[include: "path/to/file.icvs"]
```

Imports another `.icvs` file modularly. Path is relative to the source file. The resolver detects circular includes.

## Node

```plaintext
[node: <id>]
  type = <node_type>
  content = "<instruction>"
  severity = <severity>       # only for type=rule
  trigger_on = <trigger>      # only for type=blocklist/allowlist
  if = $VARIABLE == "value"   # only for type=condition
    then = -> <node_id>
    else = -> <node_id>
```

### Node Types

| Type | Description | Key Attributes |
|------|-------------|----------------|
| `rule` | Coding rules to follow | `severity`, `content` |
| `blocklist` | Prohibited libraries/code | `trigger_on`, `content` |
| `allowlist` | Permitted libraries/code | `trigger_on`, `content` |
| `condition` | Conditional logic based on env | `if`, `then`, `else` |
| `action` | Actions the agent must execute | `content` |

### Severity

| Value | Meaning |
|-------|---------|
| `must` | Required — agent MUST comply |
| `should` | Recommended — strong suggestion |
| `may` | Optional |

### Trigger On

| Value | Meaning |
|-------|---------|
| `import` | Check when importing a library |
| `install` | Check when installing a package |
| `run` | Check when running a command |

### Condition

```plaintext
[node: deploy_check]
  type = condition
  if = $BRANCH == "main"
    then = -> run_deploy
    else = -> skip_deploy
```

Environment variables use the `$` prefix. Supported operators: `==`, `!=`, `>=`, `<=`, `>`, `<`.

Condition nodes implicitly create edges to the `then` and `else` nodes.

## Edge

```plaintext
[edge: <source_id> -> <target_id>]
[edge: <source_id> -> <target_id> with "label"]
```

Defines a dependency between nodes. The source executes before the target. The graph must be a **Directed Acyclic Graph (DAG)** — cycles are rejected by the validator.

## Target

```plaintext
[target: <tool_name>]
  resolve = [node_id, node_id, ...]
  ignore = [node_id, ...]
```

Specifies which nodes apply to a particular tool.

| Attribute | Description |
|-----------|-------------|
| `resolve` | List of nodes included for this target |
| `ignore` | List of nodes excluded (subset of resolve) |

Example: A single `.icvs` file can have different targets for different tools:

```plaintext
[target: claude]
  resolve = [coding_style, forbidden_libs, prod_rule]

[target: copilot]
  resolve = [coding_style]
  ignore = [forbidden_libs]
```

## Validation Rules

### Structural
- Each node must have a unique `id`
- Edges must reference existing nodes
- The graph must be a DAG (no cycles allowed)
- Attributes in node/target blocks must be indented

### Naming
- Node ID: alphanumeric, underscore, hyphen (`^[a-zA-Z0-9_-]+$`)
- Target name: same rules as node ID

### Merging (Include)
- Duplicate node IDs from different files → error
- Duplicate target names from different files → error
- Circular include → error

## Grammar EBNF

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
