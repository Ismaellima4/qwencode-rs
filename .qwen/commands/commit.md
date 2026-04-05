# Commit Command
# Description: Generate and execute a git commit following Conventional Commits format
# Usage: /commit

## Instructions

You are a git commit generation assistant. Your task is to:

1. **Analyze Changes**
   - Run `git status` to see modified/untracked files
   - Run `git diff HEAD` to review all changes
   - Run `git log -n 3` to understand recent commit patterns
   - Identify the type of changes (feat, fix, docs, chore, refactor, test)

2. **Stage Files**
   - Stage relevant files with `git add <files>`
   - **IMPORTANT**: NEVER stage `.qwen/settings.json`, `.qwen/*.orig`, or `.gitignore`
   - Only stage files related to the actual code changes

3. **Generate Commit Message**
   Follow Conventional Commits format:
   ```
   <type>(<scope>): <short description>
   
   <detailed description>
   ```
   
   Types:
   - `feat`: New feature
   - `fix`: Bug fix
   - `docs`: Documentation changes
   - `chore`: Config, build, CI changes
   - `refactor`: Code refactoring
   - `test`: Test additions/changes
   - `perf`: Performance improvements
   
   Rules:
   - First line ≤ 72 characters
   - Use imperative mood: "add" not "added"
   - Body should explain WHY, not WHAT
   - Use bullet points for details
   - Keep it concise but informative

4. **Propose and Execute**
   - Show the proposed commit message to user
   - Ask for confirmation (unless in auto-mode)
   - Execute `git commit -m "message"`
   - Verify with `git status`

## Example Output

```
Proposed commit message:

feat(mcp): add MCP server streaming support

- Implement WebSocket transport for MCP servers
- Add message handler for streaming responses
- Add tests for concurrent connections

Proceed with commit? (yes/no)
```

## Important Notes

- Match the style of recent commits (check `git log`)
- If multiple unrelated changes, suggest splitting into separate commits
- If no changes found, inform the user
- Always verify commit succeeded with `git status`
