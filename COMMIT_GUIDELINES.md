# Git Commit Guidelines

## Conventional Commits Format

```
<type>(<scope>): <short description>

<detailed description with bullet points>
```

## Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation changes |
| `chore` | Build, CI, config changes |
| `refactor` | Code refactoring |
| `test` | Test additions/changes |
| `perf` | Performance improvements |
| `ci` | CI/CD configuration |

## Examples

### Feature
```
feat(mcp): add MCP server streaming support

- Implement WebSocket transport for MCP servers
- Add message handler for streaming responses
- Add tests for concurrent connections
```

### Fix
```
fix(query): resolve session_id collision in multi-turn queries

- Use UUID v4 instead of incremental IDs
- Add uniqueness validation in session creation
- Update tests to verify session isolation
```

### Documentation
```
docs: update README with API examples

- Add quick start section
- Include MCP tool creation examples
- Update installation instructions
```

### Chores
```
chore: fix clippy warnings and update repository metadata

- Remove unused imports in tool.rs, session.rs, protocol.rs
- Fix let_and_return in server.rs and client.rs
- Add derivable Default impl for SubagentConfig
- Update Cargo.toml with correct repository URL and author
```

## Best Practices

1. **Use imperative mood**: "add" not "added" or "adds"
2. **Keep it concise**: First line ≤ 72 characters
3. **Use body for details**: Explain WHY, not WHAT
4. **Reference issues**: Add "Closes #123" when applicable
5. **One logical change per commit**

## Commit Workflow

```bash
# 1. Review changes
git status && git diff HEAD && git log -n 3

# 2. Stage files (exclude .qwen/*)
git add <relevant-files>

# 3. Commit with message
git commit -m "type(scope): description

- Detail 1
- Detail 2
- Detail 3"

# 4. Verify
git status
```

## Things to Avoid

- ❌ Don't commit `.qwen/settings.json` or `.orig` files
- ❌ Don't use generic messages like "update files" or "fix stuff"
- ❌ Don't mix unrelated changes in one commit
- ❌ Don't commit commented-out code without explanation
