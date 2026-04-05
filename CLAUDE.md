# Ship — Developer Guide

## Quick Reference

```bash
cargo build --release          # Build
cargo test                     # Run all tests
cargo clippy -- -D warnings    # Lint (zero warnings required)
cargo fmt -- --check           # Format check
```

## Role

Builder (thợ) nhận phiếu từ Architect (Sếp), implement trong scope.

## Definition of Done (per phiếu)

1. `cargo build --release` — zero warnings
2. `cargo test` — all pass
3. `cargo clippy -- -D warnings` — clean
4. No debug code (`dbg!`, `println!` for debug, `todo!`, `unimplemented!`)
5. Update `docs/CHANGELOG.md`
6. Update `docs/ARCHITECTURE.md` (Sections 7-9 if implementation changed)
7. Discovery Report filed
8. Git commit with conventional message

## Hard Stops (DỪNG, hỏi Sếp)

- New module/file outside phiếu scope
- New dependency in Cargo.toml
- CLI interface changes (new subcommand, flag changes)
- Config schema changes (.ship.toml format)
- Refactor code not in phiếu

## Docs Gate 2 Tầng

### Tầng 1 (Hard — must update before commit)
- Function signatures, data flow
- CLI interface (flags, subcommands)
- Config schema (.ship.toml)
- Data structures (structs, enums)
- Module map
- Runtime behavior
- Known constraints

### Tầng 2 (Soft — update later)
- Variable names
- Error message wording
- Code comments
- Internal formatting

### Mapping
| Change Type | Target File | Section |
|------------|-------------|---------|
| New module/struct | ARCHITECTURE.md | 1. Module Map + 5. Data Structures |
| CLI flag change | ARCHITECTURE.md | 3. CLI Interface |
| Config change | ARCHITECTURE.md | 4. Config Schema |
| Pipeline step change | ARCHITECTURE.md | 2. Data Flow |
| New constraint | ARCHITECTURE.md | 9. Known Constraints |
| Feature complete | CHANGELOG.md | New entry |
| Convention/gotcha | CONVENTION.md | Relevant section |

## Discovery Report (mandatory per phiếu)

```markdown
## Discovery Report
### Assumptions trong phiếu — ĐÚNG:
- [list correct assumptions]

### Assumptions trong phiếu — SAI:
- [list mismatches vs code, or "Không có"]

### Edge cases phát hiện thêm:
- [edge cases found, or "Không có"]

### Docs đã cập nhật:
- [which docs files, what changed]
```

## Phiếu Classification

| Type | Scope | Review |
|------|-------|--------|
| **read-only** | No side effects (queries, display, docs) | Self-review |
| **mutating** | Changes data/state (new features, refactor) | Sếp review |
| **destructive** | Deletes data, breaks API, changes schema | Sếp review + test plan |

## Language Rules

- Vietnamese with Sếp (all communication)
- English in code, comments, commit messages
- English in documentation (docs/*.md)

## Git Workflow

```bash
git checkout -b feat/{phieu-id}-{short-name}
# ... implement ...
cargo test && cargo clippy -- -D warnings
git add -A
git commit -m "feat: brief description"
git push origin feat/{phieu-id}-{short-name}
gh pr create --title "feat: ..." --body "..."
```

## Phase History

- Phase 1: MVP Ship Pipeline (done)
- Phase 2: Canary + Deploy (done — code complete, needs dogfooding)
- Phase 3: Intelligence — learnings, MCP server (done — code complete, needs dogfooding)
- Phase 4: Ecosystem — AI review, Skill registration, GitHub Actions, Telegram (planned)
- Current focus: dogfooding on real projects (tarot, jarvis, media-rating)

## Gotchas

1. **Git command execution:** Use `std::process::Command::new("git")`, NOT git2 crate. Simpler, matches user's git config.
2. **gh CLI required:** PR creation depends on `gh` being installed and authenticated.
3. **Cross-platform paths:** Use `PathBuf` and `std::path`, never hardcode `/` or `\`.
4. **Test isolation:** Each test creates `tempfile::TempDir`, never touches real filesystem.
5. **Config defaults:** Every field in `.ship.toml` is optional. Missing = sensible default.
6. **Error propagation:** Use `thiserror` for error types, `?` operator everywhere. Never `unwrap()` in non-test code.
