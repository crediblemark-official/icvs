# 示例

## 1. 基本编码规范

```plaintext
#project: "web-app"

[node: indentation]
  type = rule
  content = "使用 2 空格缩进，不使用制表符"
  severity = must

[node: naming]
  type = rule
  content = "变量使用 camelCase，组件使用 PascalCase"
  severity = must

[node: typescript]
  type = rule
  content = "所有文件必须使用 TypeScript，不是 JavaScript"
  severity = must

[node: no_jquery]
  type = blocklist
  content = "jQuery"
  trigger_on = import

[node: no_lodash]
  type = blocklist
  content = "lodash"
  trigger_on = import

[edge: indentation -> naming]
[edge: naming -> typescript]
[edge: typescript -> no_jquery]
[edge: no_jquery -> no_lodash]

[target: claude]
  resolve = [indentation, naming, typescript, no_jquery, no_lodash]
```

## 2. 按环境的条件规则

```plaintext
#project: "deploy-pipeline"

[node: is_prod]
  type = condition
  if = $ENVIRONMENT == "production"
    then = -> require_tests
    else = -> fast_lint

[node: require_tests]
  type = rule
  content = "部署前需要 100% 测试覆盖率"
  severity = must

[node: fast_lint]
  type = rule
  content = "仅运行基本 lint 检查"
  severity = should

[node: deploy]
  type = action
  content = "所有检查通过后运行 `npm run deploy`"

[edge: require_tests -> deploy]
[edge: fast_lint -> deploy]

[target: ci]
  resolve = [is_prod, require_tests, fast_lint, deploy]
```

## 3. 多工具单体仓库

```plaintext
#project: "frontend-monorepo"

[include: "./shared/style.icvs"]
[include: "./shared/security.icvs"]

[node: react_rules]
  type = rule
  content = "使用带钩子的函数组件，不使用类组件"
  severity = must

[node: test_coverage]
  type = rule
  content = "所有包的测试覆盖率需达到 80% 以上"
  severity = must

[edge: react_rules -> test_coverage]

# Claude：全栈上下文
[target: claude]
  resolve = [react_rules, test_coverage]

# Copilot：仅内联代码补全（无测试上下文）
[target: copilot]
  resolve = [react_rules]
  ignore = [test_coverage]

# Cursor：代理模式需要所有内容
[target: cursor]
  resolve = [react_rules, test_coverage]
```

## 4. 安全优先流水线

```plaintext
#project: "secure-api"

[node: no_secrets]
  type = rule
  content = "绝不允许提交 API 密钥、令牌或密码。请使用环境变量。"
  severity = must

[node: dependency_scan]
  type = action
  content = "每次提交前运行 `npm audit` 或 `snyk test`"

[node: forbid_deprecated]
  type = blocklist
  content = "已弃用的包：request、axios@<1.0、moment"
  trigger_on = install

[node: require_linting]
  type = rule
  content = "推送前使用安全插件运行 ESLint"
  severity = must

[edge: no_secrets -> dependency_scan]
[edge: dependency_scan -> forbid_deprecated]
[edge: forbid_deprecated -> require_linting]

[target: all]
  resolve = [no_secrets, dependency_scan, forbid_deprecated, require_linting]
```

## 5. 导出与构建

### 用于 Claude
```bash
icvs export --target claude project.icvs > CLAUDE.md
```

### 用于 Copilot
```bash
icvs export --target copilot project.icvs > .github/copilot-instructions.md
```

### 用于 Cursor
```bash
icvs export --target cursor project.icvs > .cursorrules
```

### 生成图形可视化
```bash
icvs visualize project.icvs --output graph.dot
dot -Tsvg graph.dot -o graph.svg   # 需要 graphviz
```

### 按条件导出（按分支）
```bash
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [ "$BRANCH" = "main" ]; then
    TARGET="production"
else
    TARGET="development"
fi
icvs export --target "$TARGET" project.icvs | \
  sed "s/\$BRANCH/$BRANCH/g" > instructions.md
```
