# PHIẾU P004: Dogfood ship note — CLI install + MCP wiring + SKILL guidance

> **ID format:** P004.
> **Branch:** `chore/P004-dogfood-ship-note`.
> **Follows:** P003 (`ship note` code merged @ `646d5a6`).

---

> **Loại:** Chore
> **Risk:** mutating (installs binary to `~/.cargo/bin`, adds config files at ship repo root, modifies SKILL)
> **Ưu tiên:** P1
> **Ảnh hưởng:** `~/.cargo/bin/ship` (rebuild), `.mcp.json` (new), `.ship.toml` (new), `SKILL.md`, `docs/CHANGELOG.md`, `docs/DISCOVERIES.md`
> **Dependency:** P003 (ship note merged) — ✅

---

## Context

### Vấn đề
P003 code merged nhưng Claude Code **không biết dùng**:
1. Binary `~/.cargo/bin/ship` vẫn version cũ (Apr 16, pre-P003) — CLI `ship note` chưa tồn tại; MCP `ship serve` expose 4 tools cũ (không có `ship_note_export`).
2. Ship repo không có `.mcp.json` — session Claude Code ở `~/ship` không có ship MCP tools (tarot có riêng `.mcp.json`).
3. Ship repo không có `.ship.toml` — `auto_log` default false → `ship check` không tự log vault.
4. `SKILL.md` không đề cập `ship note` — Claude Code không biết khi nào invoke.

### Giải pháp
Dogfood end-to-end trong ship repo: install binary, wire MCP, enable auto_log cho ship, update SKILL.md với decision tree.

### Scope
- CHỈ sửa: `~/.cargo/bin/ship` (rebuild qua cargo install), `.mcp.json` (new), `.ship.toml` (new), `SKILL.md`, CHANGELOG, DISCOVERIES.
- KHÔNG sửa: tarot repo, các project khác (sẽ làm phiếu riêng khi Sếp ready).

---

## Verification Anchors

| # | Assumption | Verify | Kết quả |
|---|-----------|--------|---------|
| 1 | `~/.cargo/bin/ship` là binary cũ (pre-P003) | `ls -la $(which ship)` | ✅ Apr 16 16:32, size ~3.5MB |
| 2 | Ship repo chưa có `.mcp.json` | `ls .mcp.json` | ✅ không tồn tại |
| 3 | Ship repo chưa có `.ship.toml` | `ls .ship.toml` | ✅ không tồn tại |
| 4 | Tarot `.mcp.json` đã register ship MCP | `cat ~/tarot/.mcp.json` | ✅ `"ship": { "command": "/Users/nguyenhuuanh/.cargo/bin/ship", "args": ["serve"] }` |
| 5 | SKILL.md không đề cập `ship note` | `grep -i note SKILL.md` | ✅ không có |
| 6 | CLAUDE.md có section "ship note workflow" | `grep "ship note" CLAUDE.md` | ✅ từ P003, đủ nội dung |
| 7 | Ship `Cargo.toml` install target standard | `grep "\[package\]" Cargo.toml` | ✅ cargo install --path . hoạt động |

---

## Nhiệm vụ

### Task 1 — Install binary

```bash
cd ~/ship && cargo install --path . --force
```

Lưu ý: `--force` override binary cũ. Sau install, verify:
```bash
ship --version    # phải là 0.1.0
ship note --help  # phải list subcommand
```

### Task 2 — Ship `.mcp.json`

Tạo file `.mcp.json` ở ship root. Follow pattern tarot nhưng minimal — chỉ register tools ship dev sẽ dùng:

```json
{
  "mcpServers": {
    "ship": {
      "command": "/Users/nguyenhuuanh/.cargo/bin/ship",
      "args": ["serve"]
    },
    "docs-gate": {
      "command": "/Users/nguyenhuuanh/docs-gate/target/release/docs-gate",
      "args": ["serve"]
    },
    "filesystem": {
      "command": "npx",
      "args": [
        "-y", "@modelcontextprotocol/server-filesystem",
        "/Users/nguyenhuuanh/ship"
      ]
    }
  }
}
```

Lý do skip: github (dùng `gh` CLI qua Bash), sequential-thinking (optional). Có thể add sau.

### Task 3 — Ship `.ship.toml`

```toml
name = "ship"
stack = "rust"
base_branch = "main"

[test]
command = "cargo test"
timeout_secs = 180

[docs_gate]
enabled = true
blocking = false

[obsidian]
auto_log = true
# vault_path default ~/VibeNotes
# project_slug default cwd dirname = "ship"
```

### Task 4 — Update SKILL.md

Thêm:
- Prerequisites: mention `~/VibeNotes` vault optional (for auto_log)
- **New section "Ship Note" sau "Post-deploy Canary"** — CLI examples + MCP fallback + decision tree
- Update "Integration with Other Skills": reference auto_log hook

### Task 5 — Docs

- CHANGELOG Unreleased entry
- DISCOVERIES Report
- (CLAUDE.md already has section from P003)

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `.mcp.json` | **MỚI** — register ship + docs-gate + filesystem |
| `.ship.toml` | **MỚI** — ship config với [obsidian] auto_log = true |
| `SKILL.md` | Thêm section "Ship Note" + update Prerequisites / Integration |
| `docs/CHANGELOG.md` | P004 entry |
| `docs/DISCOVERIES.md` | Discovery Report |
| `docs/ticket/P004-dogfood-ship-note.md` | **MỚI** — file này |

**System-level** (không commit):
- `~/.cargo/bin/ship` — rebuild via cargo install

## Files KHÔNG sửa

| File | Verify gì |
|------|----------|
| `src/**/*.rs` | Code P003 đã merged, không đụng |
| `~/tarot/.mcp.json`, `~/tarot/.ship.toml` | Tarot dogfood = phiếu riêng sau |
| `~/jarvis/*`, `~/BlockAds/*`, ... | Các project khác = phiếu riêng |

---

## Luật chơi

1. `cargo install --path . --force` OK — Sếp đã explicit authorize install.
2. `.mcp.json` + `.ship.toml` ở ship repo = self-dogfood only, không touch các project khác.
3. SKILL update additive — không break existing `/ship`, `/ship check` guidance.
4. Smoke test thật sau install: `ship check` ở ship repo phải log vào `~/VibeNotes/10_Projects/ship/logs/`.

---

## Nghiệm thu

### Automated
- [ ] `cargo build --release` — zero warnings (no code change, cached)
- [ ] `cargo test` — all pass (no change)
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] `cargo fmt -- --check` — clean

### Manual
- [ ] `ship --version` sau install → 0.1.0
- [ ] `ship note --help` → shows subcommand flags
- [ ] `ship check` ở ship repo → auto-log note vào vault
- [ ] `.mcp.json` valid JSON
- [ ] `.ship.toml` parses (run `ship check` trigger parse)

### Regression
- [ ] Existing subcommands unchanged (`ship`, `ship check`, `ship canary`, `ship learn`)
- [ ] Tarot `ship_check` MCP tool (when Claude Code restart) vẫn chạy với binary mới — 5 tools visible

### Docs Gate
- [ ] CHANGELOG P004 entry
- [ ] DISCOVERIES Report
- [ ] CLAUDE.md: section "ship note" đã đủ từ P003 — verify unchanged
