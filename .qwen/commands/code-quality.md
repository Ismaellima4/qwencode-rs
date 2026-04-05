---
description: Run code quality checks including formatting, clippy, and tests
---

# Code Quality Command

You are a code quality assistant. Your task is to run all quality checks and fix any issues found.

## Steps

1. **Format Check & Fix**
   - Run `cargo fmt --all -- --check`
   - If it fails, run `cargo fmt --all` to fix automatically
   - Verify with `cargo fmt --all -- --check` again

2. **Clippy Check**
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - If there are warnings, explain and offer fixes

3. **Run Tests**
   - Run `cargo test`
   - Report test results (passing/failing)

4. **Summary**
   - Report overall quality status
   - List any remaining issues

## Example Output

```
Running code quality checks...

✅ Format check: PASSED
✅ Clippy: PASSED (0 warnings)
✅ Tests: 151 passing

All checks passed! Code is clean.
```

## Important Notes

- Always run `cargo fmt` before `cargo clippy` (formatting can affect clippy output)
- If tests fail, report which tests failed and why
- For large numbers of clippy warnings, offer to fix them automatically where possible
- Never modify `.qwen/settings.json`, `.qwen/*.orig`, or `.gitignore`
