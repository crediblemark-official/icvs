# Examples

## 1. Basic Coding Standards

```plaintext
#project: "web-app"

[node: indentation]
  type = rule
  content = "Use 2-space indentation, no tabs"
  severity = must

[node: naming]
  type = rule
  content = "Use camelCase for variables, PascalCase for components"
  severity = must

[node: typescript]
  type = rule
  content = "All files must be TypeScript, not JavaScript"
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

## 2. Conditional Rules by Environment

```plaintext
#project: "deploy-pipeline"

[node: is_prod]
  type = condition
  if = $ENVIRONMENT == "production"
    then = -> require_tests
    else = -> fast_lint

[node: require_tests]
  type = rule
  content = "100% test coverage required before deploy"
  severity = must

[node: fast_lint]
  type = rule
  content = "Run basic lint check only"
  severity = should

[node: deploy]
  type = action
  content = "Run `npm run deploy` after all checks pass"

[edge: require_tests -> deploy]
[edge: fast_lint -> deploy]

[target: ci]
  resolve = [is_prod, require_tests, fast_lint, deploy]
```

## 3. Multi-Tool Monorepo

```plaintext
#project: "frontend-monorepo"

[include: "./shared/style.icvs"]
[include: "./shared/security.icvs"]

[node: react_rules]
  type = rule
  content = "Use functional components with hooks, no class components"
  severity = must

[node: test_coverage]
  type = rule
  content = "80%+ test coverage for all packages"
  severity = must

[edge: react_rules -> test_coverage]

# Claude: full stack context
[target: claude]
  resolve = [react_rules, test_coverage]

# Copilot: inline code completion only (no testing context)
[target: copilot]
  resolve = [react_rules]
  ignore = [test_coverage]

# Cursor: agent mode needs everything
[target: cursor]
  resolve = [react_rules, test_coverage]
```

## 4. Security-First Pipeline

```plaintext
#project: "secure-api"

[node: no_secrets]
  type = rule
  content = "Never commit API keys, tokens, or passwords. Use environment variables."
  severity = must

[node: dependency_scan]
  type = action
  content = "Run `npm audit` or `snyk test` before each commit"

[node: forbid_deprecated]
  type = blocklist
  content = "deprecated packages: request, axios@<1.0, moment"
  trigger_on = install

[node: require_linting]
  type = rule
  content = "Run ESLint with security plugin before push"
  severity = must

[edge: no_secrets -> dependency_scan]
[edge: dependency_scan -> forbid_deprecated]
[edge: forbid_deprecated -> require_linting]

[target: all]
  resolve = [no_secrets, dependency_scan, forbid_deprecated, require_linting]
```

## 5. Export & Build

### Untuk Claude
```bash
icvs export --target claude project.icvs > CLAUDE.md
```

### Untuk Copilot
```bash
icvs export --target copilot project.icvs > .github/copilot-instructions.md
```

### Untuk Cursor
```bash
icvs export --target cursor project.icvs > .cursorrules
```

### Generate graph visualization
```bash
icvs visualize project.icvs --output graph.dot
dot -Tsvg graph.dot -o graph.svg   # requires graphviz
```

### Conditional export (branch-specific)
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
