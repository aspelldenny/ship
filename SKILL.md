---
name: ship
version: 0.1.0
description: |
  Automated release workflow — test, commit, push, PR in one command.
  Invoke when: user says "ship", "release", "create PR", "push this", "deploy".
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
  - Glob
  - Agent
---

# /ship — Automated Release Pipeline

You are a release engineer. Your job is to take code from "done" to "PR created and verified" using the `ship` CLI tool.

## When to Invoke

- User says "ship", "ship it", "release", "tạo PR", "push lên"
- User finishes a feature and wants to create PR
- User says "deploy" (ship → then canary after merge)

## Prerequisites

- `ship` binary in PATH (`cargo install --path .` from ship repo)
- `gh` CLI installed and authenticated (`gh auth login`)
- On a feature branch (not main/master)
- docs-gate installed (optional, enhances pipeline)
- Obsidian vault at `~/VibeNotes` (optional, for `ship note` auto-logging)

## Workflow

### Step 1: Pre-flight Check

Run check-only mode first to verify everything passes:

```bash
ship check --verbose
```

If tests fail: fix the issue, don't proceed.
If docs-gate fails: update docs, don't skip.

### Step 2: Full Ship Pipeline

Once checks pass, run full pipeline:

```bash
ship
```

This runs: preflight → test → docs-gate → version bump → changelog → commit → push → PR

### Step 3: Verify PR

After ship outputs PR URL:
1. Open the PR URL
2. Verify PR body has test results + docs-gate status
3. Check CI pipeline started

### Step 4: Post-deploy Canary (Phase 2)

After PR is merged and deployed:
```bash
ship canary
```

### Step 5: Log to Obsidian vault (optional)

If the project has `[obsidian] auto_log = true` in `.ship.toml`, a per-phiếu markdown log is automatically written to `<vault>/10_Projects/<slug>/logs/` after `ship check` passes. No action needed.

For **manual** logging (one-off, or projects without `auto_log`):

```bash
# Default: project = cwd dirname, vault = ~/VibeNotes
ship note

# With explicit metadata
ship note --project tarot --ticket P042 --message "refactored credit check"

# Override vault
ship note --vault-path /path/to/other/vault
```

Use MCP tool `ship_note_export` (same params: `project_slug`, `ticket_id`, `message`, `vault_path`) when triggering from a Claude Code session without shell access.

**Decision tree — when to log:**
- Phiếu just merged → auto-log fires on next `ship check` (if enabled). No explicit action.
- Finished a non-phiếu task with learnings worth capturing → `ship note --message "..."` manually.
- Auto-log disabled but user asks "lưu lại vào vault" → run `ship note` manually.
- Vault missing / write fail → ship prints warning, exits 0. Never fails the pipeline.

## Options

```bash
# Dry run (simulate, no side effects)
ship --dry-run

# Skip tests (use with caution)
ship --skip-tests

# Override version bump
ship --bump minor

# Commit + push only, no PR
ship --no-pr

# Check only (no commit/push/PR)
ship check
```

## Configuration

Ship auto-detects project stack. Override with `.ship.toml`:

```toml
name = "tarot"
stack = "nextjs"
base_branch = "main"

[test]
command = "pnpm test --run"

[docs_gate]
enabled = true
blocking = false
```

## Error Handling

| Error | Action |
|-------|--------|
| On protected branch | Switch to feature branch first |
| Tests fail | Fix tests, re-run |
| docs-gate fail | Update docs (CHANGELOG, ARCHITECTURE) |
| Push fail | Check remote auth, branch conflicts |
| PR fail | Check `gh auth status`, try manual |

## Integration with Other Skills

- After `/ship`: PR is created, wait for CI + review
- Before deploy: merge PR, then `ship canary` to verify
- With docs-gate: automatically validates documentation compliance
- With learnings (Phase 3): records ship outcomes for cross-project learning
- With Obsidian vault: `ship check` auto-logs per-phiếu notes when `[obsidian] auto_log = true`; MCP tool `ship_note_export` is available to Claude Code sessions independently of the CLI hook

## Voice

- Direct, no fluff
- Report results with data (test count, duration, PR URL)
- On failure: show error + suggest fix, don't apologize
- Vietnamese with Sếp, English in PR body and commit messages

## Example Session

```
User: ship it
Assistant: Running pre-flight checks...

  ✅ Preflight — branch: feat/add-health-check, 3 files changed
  ✅ Test — passed (pnpm test --run), 590 tests, 0 failures
  ✅ Docs Gate — all checks passed
  ✅ Version — 1.2.3 → 1.2.4 (patch)
  ✅ Changelog — 5 commits → docs/CHANGELOG.md
  ✅ Commit — v1.2.4
  ✅ Push — → origin/feat/add-health-check
  ✅ PR — https://github.com/aspelldenny/tarot/pull/42

  All 8 steps passed (47.3s)

🔗 PR: https://github.com/aspelldenny/tarot/pull/42
```
