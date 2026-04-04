# Conventions — Ship CLI

## Naming

| Item | Convention | Example |
|------|-----------|---------|
| Modules | snake_case | `docs_gate.rs`, `preflight.rs` |
| Structs | PascalCase | `PipelineResult`, `StepStatus` |
| Functions | snake_case | `run_tests()`, `detect_stack()` |
| Constants | UPPER_SNAKE_CASE | `DEFAULT_TIMEOUT_SECS` |
| Config keys | snake_case | `base_branch`, `timeout_secs` |
| CLI flags | kebab-case | `--dry-run`, `--skip-tests` |

## Module Organization

```
src/
├── main.rs          # CLI parsing only, delegates to modules
├── config.rs        # Config loading + defaults
├── detect.rs        # Project stack detection
├── pipeline/        # Ship pipeline steps (1 file per step)
├── canary/          # Health check modules
├── learn/           # Learnings JSONL management
├── mcp/             # MCP server (rmcp)
├── output.rs        # Terminal formatting
└── error.rs         # Error types
```

## Error Handling

- Use `thiserror` for custom error enum
- Never `unwrap()` or `expect()` in non-test code
- Propagate with `?` operator
- User-facing errors: clear message + suggested action
- Exit codes: 0 (success), 1 (pipeline failure), 2 (config/usage error)

## Testing

- Unit tests: `#[cfg(test)]` inline in each module
- Integration tests: `tests/` directory
- Use `tempfile::TempDir` for filesystem tests
- Never touch real git repos in tests
- Test names: `test_<function>_<scenario>`

## Git Commits

Conventional commits:
- `feat:` — new feature
- `fix:` — bug fix
- `refactor:` — code change (no new feature, no bug fix)
- `docs:` — documentation only
- `test:` — adding/fixing tests
- `chore:` — maintenance (deps, CI, etc.)

## Output Formatting

- `✅` for pass, `❌` for fail, `⚠️` for warning, `⏭️` for skip
- Step name in bold (if terminal supports)
- Duration in gray
- Errors in red with details
- Final summary: total steps, pass/fail count, duration

## Known Constraints

1. **gh CLI dependency:** `ship` does not implement GitHub API directly. Requires `gh` installed and `gh auth login` completed.
2. **Git in PATH:** All git operations use `std::process::Command`. Git must be in PATH.
3. **Unix-first:** Config examples use Unix paths. Windows paths work via PathBuf but examples may need adjustment.
4. **Single project scope:** Run `ship` from project root. Does not support monorepo workspace detection (yet).
5. **CHANGELOG format:** Expects Keep a Changelog format with `## X.Y.Z (YYYY-MM-DD)` headings.
