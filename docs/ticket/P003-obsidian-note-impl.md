# PHIẾU P003: Implement ship note — Obsidian vault log

> **ID format:** P003.
> **Branch:** `feat/P003-obsidian-note-impl`.
> **Implements:** P002 (drafted `docs/ticket/P002-obsidian-note.md`).

---

> **Loại:** Feature
> **Risk:** mutating (new CLI subcommand, CLI interface change, new module `src/note/`, new MCP tool, new config section)
> **Ưu tiên:** P1
> **Ảnh hưởng:** `src/main.rs`, `src/note/mod.rs` (new), `src/config.rs`, `src/mcp/tools.rs`, `src/mcp/server.rs`, `docs/*`
> **Dependency:** P002 (spec) — phiếu file đã trên main @ `c3b2531`

---

## Context

### Vấn đề
Anh đã OK triển P003. Các open questions P002 đã chốt:
1. ✅ Subcommand: **`ship note`** (không sub-verb, YAGNI)
2. ✅ Diacritics: **manual table zero-dep** (deviation — thấy `src/learn/mod.rs` có `shellexpand` + `dirs_home` sẵn, không cần crate; consistent với opt-level="z" size-optimization)
3. ✅ Làm luôn
4. ✅ Fresh `feat/P003-obsidian-note-impl` off main

### Deviations từ P002

- **Không add deps mới** (`dirs`, `unicode-normalization`) — reuse `shellexpand`/`dirs_home` pattern từ `src/learn/mod.rs`. Diacritic stripping dùng bảng tay cho Vietnamese + fallback `-` cho non-ASCII khác.
- Hook point cho `ship check` post-success: **`src/main.rs` match arm `Commands::Check`** (ngay sau `pipeline::check` trả Pass), chứ không phải trong `pipeline/mod.rs`. Lý do: MCP tool `ship_check` cũng gọi `pipeline::check`, nếu hook ở pipeline thì MCP cũng log note — undesirable. Hook ở CLI-level only, MCP user dùng `ship_note_export` tool riêng.

### Scope

Per P002 — 7 tasks. Full details tại `docs/ticket/P002-obsidian-note.md`.

---

## Verification Anchors — Re-verified cho P003

| # | Assumption | Verify | Kết quả |
|---|-----------|--------|---------|
| 1 | `src/main.rs` dòng 55 — `enum Commands` variants: Check, Init, Canary, Deploy, Learn, Serve | đọc main.rs | ✅ thêm `Note` variant cần `{project, ticket, message, vault_path}` args |
| 2 | `pipeline::check` signature: `pub fn check(config: &Config, opts: &PipelineOptions) -> Result<PipelineResult>` | pipeline/mod.rs:172 | ✅ sync, returns `PipelineResult` với `has_failures()` |
| 3 | `shellexpand` + `dirs_home` pattern ở `src/learn/mod.rs:92-105` | grep | ✅ zero-dep, reusable → promote lên module-common hoặc dup cục bộ (em dup cục bộ để tránh cross-module churn) |
| 4 | Config::Default impl manual cho mỗi sub-struct | config.rs:101-206 | ✅ follow cùng pattern cho `ObsidianConfig::Default` |
| 5 | MCP tool: `#[tool(name = "...")]` method trong `impl ShipServer` với `#[tool_router]` macro | mcp/server.rs:34-150 | ✅ |
| 6 | MCP params: pub struct với `#[derive(Debug, Deserialize, JsonSchema)]` ở `src/mcp/tools.rs` | tools.rs | ✅ |
| 7 | Test `test_tool_router_has_4_tools` hardcoded `4` | server.rs:229 | ✅ update thành 5 |
| 8 | Vault path `~/VibeNotes/10_Projects/ship/logs/` — `logs/` chưa exist | `ls` | ✅ tạo bằng `create_dir_all` khi write đầu tiên |
| 9 | obsidian-git auto-commit interval | `~/VibeNotes/.obsidian/plugins/obsidian-git/` | ✅ default 10 phút — atomic write đủ mitigate |
| 10 | `output::header`, `output::step_pass` | output.rs | ✅ reuse cho UX consistent |
| 11 | Error types: ShipError enum có Config, Git, Io, ... | error.rs | ✅ dùng `ShipError::Config` cho vault errors (hoặc eprintln + Ok(()) for graceful) |

---

## Nhiệm vụ

Chi tiết từng task ở `P002-obsidian-note.md`. Task files đã map chính xác:

| Task | File | Ghi chú |
|------|------|---------|
| 1 — Subcommand `ship note` | `src/main.rs` (register) + `src/note/mod.rs` (new) | Follow pattern `src/canary/mod.rs::run` |
| 2 — File format | `src/note/mod.rs` | Bỏ section "Learnings" nếu `--message` absent |
| 3 — Config `.ship.toml [obsidian]` | `src/config.rs` | `ObsidianConfig { auto_log, vault_path, project_slug }` + Default |
| 4 — Post-success hook | `src/main.rs` (`Commands::Check` arm) | KHÔNG hook trong `pipeline/mod.rs` — chỉ CLI-level để MCP `ship_check` không tự trigger |
| 5 — MCP tool `ship_note_export` | `src/mcp/tools.rs` + `src/mcp/server.rs` | Update test → `_has_5_tools` |
| 6 — Atomic write | `src/note/mod.rs` | `write(tmp) + rename(final)` |
| 7 — Docs | `CLAUDE.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/CHANGELOG.md` | |

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `src/main.rs` | Register `Note` Command variant + match arm. Hook in `Check` arm after pipeline::check Pass. |
| `src/note/mod.rs` | **MỚI** — full handler |
| `src/config.rs` | `ObsidianConfig` struct + Default + field trong Config |
| `src/mcp/tools.rs` | `NoteExportParams` struct |
| `src/mcp/server.rs` | `ship_note_export` tool method + update test to 5 |
| `CLAUDE.md` | Section "ship note workflow" |
| `README.md` | Usage example |
| `docs/ARCHITECTURE.md` | Sections 1, 3, 4, 5 |
| `docs/CHANGELOG.md` | P003 entry |
| `docs/DISCOVERIES.md` | Discovery Report |

## Files KHÔNG sửa

| File | Verify gì |
|------|----------|
| `Cargo.toml` | KHÔNG thêm deps — zero-dep approach |
| `src/canary/`, `src/deploy/`, `src/learn/` | Existing subcommands không đổi |
| `src/pipeline/` | Không touch — hook ở CLI-level |
| `docs/ticket/P002-obsidian-note.md` | Spec gốc, giữ làm reference |

---

## Luật chơi

1. Additive only — không break existing.
2. Graceful: vault fail → warn + exit 0, không fail ship.
3. Opt-in: `auto_log = false` default.
4. Atomic write tmp + rename.
5. Zero clippy warnings.
6. Zero new deps (deviation từ P002 — chốt rồi).
7. Hook CLI-level only, không pipeline-level (MCP không tự trigger).

---

## Nghiệm thu

### Automated (CLAUDE.md DoD)
- [ ] `cargo build --release` — zero warnings
- [ ] `cargo test` — 38 hiện tại + tests mới cho note/ pass
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] `cargo fmt -- --check` — clean
- [ ] No debug code

### Manual
- [ ] `ship note --project tarot --message "test"` từ `~/ship` → file tạo ở `~/VibeNotes/10_Projects/tarot/logs/<today>-<desc>.md`
- [ ] Vault missing → warn stderr, exit 0
- [ ] `ship check` với `auto_log = true` trong ship `.ship.toml` → log note sau check pass
- [ ] `ship check` với `auto_log = false` (default) → không log
- [ ] MCP tool list có 5 tools (test automated covers)
- [ ] Filename collision → suffix `-XXXX`
- [ ] Commit "Thêm tính năng đặc biệt" → filename ASCII `them-tinh-nang-dac-biet`

### Regression
- [ ] `test_tool_router_has_5_tools` pass
- [ ] 4 tools cũ (ship_check, ship_canary, ship_learn_add, ship_learn_search) vẫn có
- [ ] Existing CLI subcommands không đổi behavior

### Docs Gate
- [ ] CHANGELOG P003 entry
- [ ] ARCHITECTURE sections 1, 3, 4, 5
- [ ] DISCOVERIES Report
