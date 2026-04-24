# PHIẾU P002: ship note — Obsidian vault log integration

> **ID format:** P002.
> **Filename:** `docs/ticket/P002-obsidian-note.md`.
> **Branch:** `docs/P002-obsidian-note` (phiếu draft). Implementation branch sẽ là `feat/P002-obsidian-note` khi Sếp greenlight — rebase branch type sau.

---

> **Loại:** Feature
> **Risk:** mutating (thêm subcommand, CLI interface thay đổi, thêm dep `dirs` — Hard Stop → phiếu này = pre-approval)
> **Ưu tiên:** P1 (queue sau khi tarot v2.1.4 ship + dogfooding phase ổn)
> **Ảnh hưởng:** `src/main.rs`, `src/note/` (mới), `src/config.rs`, `src/mcp/server.rs`, `src/mcp/tools.rs`, `Cargo.toml`, `CLAUDE.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/CHANGELOG.md`
> **Dependency:** P001 (TICKET_TEMPLATE + DISCOVERIES format) — ✅ done

> **Status hôm nay (2026-04-24):** Phiếu drafted. Implementation CHƯA bắt đầu. Sếp review → greenlight → thợ pickup.

---

## Context

### Vấn đề hiện tại

Anh có 11 projects chính. Sau mỗi phiếu ship ra prod → không có audit trail cross-project. Vault Obsidian `~/VibeNotes/` đã có PARA structure (`10_Projects/<slug>/logs/`, `README.md`) nhưng update **manual qua Claude Code** — phải nhớ gọi, dễ quên.

### Giải pháp

**Extend `ship`** thay vì build tool mới. Lý do:

1. `ship` là step cuối pipeline (test + docs-gate + canary pass) → thời điểm phiếu chính thức "done" → signal tự nhiên để log.
2. Tránh tool sprawl (Sếp đã có 5 gates/tools: docs-gate, guard, ship, quality-gate, vps).
3. Reuse `ship learn` DB có sẵn (Phase 2 integration: export learnings kèm note).
4. Integration point tự nhiên với `ship check` post-success hook.

### Scope

- **CHỈ thêm**:
  - Subcommand `ship note` (manual trigger, export vault note)
  - Post-success hook trong `ship check` (opt-in qua `.ship.toml [obsidian] auto_log = true`)
  - MCP tool `ship_note_export`
  - Graceful degradation: vault missing / write fail → warning stderr, exit 0 (không fail ship)

- **KHÔNG**:
  - Build tool mới (không có `obsidian-gate` riêng)
  - Modify vault structure (chỉ write vào `10_Projects/<slug>/logs/`)
  - Sync git repo từ vault (out of scope — vault có obsidian-git plugin riêng)
  - Break existing `ship check`, `ship canary`, `ship learn` (additive only)

---

## Verification Anchors — Sếp đã verify lúc viết phiếu

| # | Assumption | Verify bằng lệnh nào | Kết quả |
|---|-----------|---------------------|---------|
| 1 | `src/main.rs` dùng clap `#[derive(Subcommand)] enum Commands` | `grep "Subcommand\|enum Commands" src/main.rs` | ✅ dòng 11, 55-56 |
| 2 | Ship MCP có 4 tools hiện tại (ship_check, ship_canary, ship_learn_add, ship_learn_search) | `grep '#\[tool' src/mcp/server.rs` | ✅ dòng 37, 79, 114, 125 |
| 3 | MCP tool params structs ở `src/mcp/tools.rs` — pattern: `CheckParams`, `CanaryParams`, `LearnAddParams`, `LearnSearchParams` | `grep "pub struct.*Params" src/mcp/tools.rs` | ✅ cần `NoteExportParams` mới theo cùng pattern |
| 4 | Config struct ở `src/config.rs` — nested sub-struct + `#[serde(default)]` pattern | `grep -B1 "Config {" src/config.rs` | ✅ TestConfig, DocsGateConfig, CanaryConfig, DeployConfig, LearnConfig — thêm `ObsidianConfig` cùng pattern |
| 5 | Module layout: mỗi feature 1 dir với `mod.rs` (e.g. `canary/mod.rs` export `pub async fn run`) | `ls src/canary/ && grep "pub.*fn run" src/canary/mod.rs` | ✅ dòng 48 — follow cùng pattern cho `src/note/mod.rs` |
| 6 | `dirs` crate chưa có trong Cargo.toml (cần để resolve `~/VibeNotes`) | `grep '^dirs' Cargo.toml` | ❌ KHÔNG có — phải thêm (Hard Stop, phiếu này = pre-approval) |
| 7 | `chrono` 0.4 đã có (cần cho date formatting filename) | `grep '^chrono' Cargo.toml` | ✅ dòng có, feature `serde` |
| 8 | Vault path tồn tại + structure | `ls ~/VibeNotes/10_Projects/ship/` | ✅ `README.md` + `tickets/` (logs/ sẽ tạo khi write đầu tiên) |
| 9 | obsidian-git plugin enabled (auto-commit 10p default) | `ls ~/VibeNotes/.obsidian/plugins/obsidian-git/` | ✅ |
| 10 | Test pattern: `#[cfg(test)] mod tests` trong mỗi module | `grep -rc 'cfg(test)' src/` | ✅ 10+ modules có tests, 38 tests hiện tại pass |
| 11 | CLI register pattern: thêm variant vào `enum Commands` + match arm trong main fn | `grep -A3 "enum Commands" src/main.rs` | ✅ cần đọc thêm lúc code |

---

## Nhiệm vụ

### Task 1 — Subcommand `ship note`

**File:** `src/main.rs` (register) + `src/note/mod.rs` (mới, handler)

**CLI signature:**

```bash
ship note --project <slug> [--ticket <id>] [--message <text>] [--vault-path <path>]
```

**Behavior:**
- `--project`: slug project. Fallback: `[ship].project` trong `.ship.toml` → dirname(cwd).
- Vault path resolve: arg `--vault-path` > env `OBSIDIAN_VAULT_PATH` > `.ship.toml [obsidian] vault_path` > default `~/VibeNotes`.
- Expand `~` qua `dirs::home_dir()` (crate mới).
- Check vault path exists + writable → nếu không → warning stderr + exit 0 (graceful).
- Generate filename: `<vault>/10_Projects/<slug>/logs/<YYYY-MM-DD>-<short-desc>.md`
  - `<short-desc>` derive từ commit subject (first 40 chars, kebab-case, strip diacritics).
  - Filename collision → append `-<random-4>` suffix.
- Write atomic: `tmp file + rename` (tránh race với obsidian-git).
- Print path file đã tạo.

**Lưu ý:**
- Follow pattern `src/canary/mod.rs`: `pub async fn run(...) -> Result<...>` hoặc sync nếu không có async cần.
- Strip diacritics tiếng Việt: dùng `unicode-normalization` hoặc manual table. **Hard Stop** nếu cần crate mới — Sếp confirm trong phiếu này: OK dùng `unicode-normalization` hoặc inline ASCII fallback table.

### Task 2 — File format spec

```markdown
---
date: YYYY-MM-DD
project: <slug>
ticket: <id-or-empty>
type: ship-note
tags: [project-log, ship]
---

# <slug> — <date>

## Changes
<commit subject on 1 line>

<commit body if any>

## Files changed
<git diff --stat HEAD~1..HEAD, first 20 lines max>

## Related
- Commit: <hash7> — <github url if remote parsed from `git remote -v`>
- Branch: <current branch>
- PR: <url from `gh pr view --json url` nếu có, else bỏ dòng>

<nếu --message pass, thêm section:>
## Learnings
<content of --message>
```

**Edge cases:**
- Không có commit (fresh repo) → skip "Changes" + "Files changed", stub note.
- `gh` CLI không có → bỏ dòng PR.
- Tiếng Việt có dấu → strip diacritics trước khi kebab-case filename.
- `--message` không pass → bỏ section "Learnings" (không in "N/A" — tránh rác visual trong vault).

### Task 3 — Config `.ship.toml [obsidian]`

**File:** `src/config.rs`

**Thêm:**
```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ObsidianConfig {
    pub auto_log: bool,
    pub vault_path: Option<String>,
    pub project_slug: Option<String>,
}
```

Default: `auto_log = false` (opt-in). Thêm `pub obsidian: ObsidianConfig` vào `Config` struct + `impl Default`.

**Lưu ý:** Follow pattern các sub-struct khác (serde default, Optional fields).

### Task 4 — Post-success hook trong `ship check`

**File:** `src/pipeline/mod.rs` hoặc nơi `check` kết thúc (thợ grep xác định).

**Behavior:**
- Nếu `config.obsidian.auto_log == true` → sau `ship check` pass (test + docs-gate OK) → call `note::run` internally.
- Không block ship nếu vault write fail (graceful).
- Log 1 dòng info khi success: `ℹ️  Logged to vault: <path>`.

### Task 5 — MCP tool `ship_note_export`

**File:** `src/mcp/tools.rs` (add `NoteExportParams`) + `src/mcp/server.rs` (add `#[tool]` method).

**Params:**
```rust
pub struct NoteExportParams {
    pub project_slug: Option<String>,
    pub ticket_id: Option<String>,
    pub message: Option<String>,
    pub vault_path: Option<String>,
}
```

**Expose:** `#[tool(name = "ship_note_export")]` để Claude Code subagent trigger qua MCP.

**Update:** Test `test_tool_router_has_4_tools` → `test_tool_router_has_5_tools`.

### Task 6 — obsidian-git race prevention

**Scenario:** ship write vault → obsidian-git plugin auto-commit (10p interval). Có thể race nếu ship write chưa xong mà git add.

**Mitigation:** Atomic write — write to `<target>.tmp` + `rename` (POSIX atomic). Hết race.

**KHÔNG cần** trailer marker, config exclude, hay doc dành riêng — over-engineering. Volume 1 note/phiếu vs 10p interval là zero conflict thực tế.

### Task 7 — Docs

- `CLAUDE.md` ship — thêm section "ship note workflow" (1 đoạn ngắn + link README).
- `README.md` ship — usage example.
- `docs/ARCHITECTURE.md`:
  - Section 1 (Module Map): thêm `note/`
  - Section 3 (CLI Interface): thêm `ship note` flags
  - Section 4 (Config Schema): thêm `[obsidian]` section
  - Section 5 (Data Structures): thêm `ObsidianConfig`, `NoteExportParams`
- `docs/CHANGELOG.md`: entry cho P002.

---

## Files cần sửa

| File | Thay đổi |
|------|---------|
| `Cargo.toml` | Thêm dep `dirs = "5"` (và `unicode-normalization` nếu chọn crate cho diacritics) |
| `src/main.rs` | Register subcommand `Note` variant trong `enum Commands` + match arm |
| `src/note/mod.rs` | **MỚI** — handler: resolve vault, derive filename, gen content, atomic write |
| `src/config.rs` | Thêm `ObsidianConfig` struct + `pub obsidian` field trong `Config` |
| `src/pipeline/mod.rs` | Post-success hook gọi `note::run` khi `auto_log == true` (thợ xác định đúng file) |
| `src/mcp/tools.rs` | Thêm `NoteExportParams` struct |
| `src/mcp/server.rs` | Thêm `#[tool] ship_note_export` method |
| `CLAUDE.md` | Section "ship note workflow" |
| `README.md` | Usage example |
| `docs/ARCHITECTURE.md` | Sections 1, 3, 4, 5 |
| `docs/CHANGELOG.md` | Entry P002 |

## Files KHÔNG sửa (verify only)

| File | Verify gì |
|------|----------|
| `src/canary/**`, `src/deploy/**`, `src/learn/**` | Existing subcommands behavior không đổi |
| `~/VibeNotes/**` | Chỉ write vào `10_Projects/<slug>/logs/`, không đụng structure khác |
| `.ship.toml` của 10 projects khác | Chỉ thay khi Sếp opt-in per project |
| `src/learn/store.rs` | `ship learn` DB schema không đụng (optional Phase 2 integration) |

---

## Luật chơi (Constraints)

1. **Additive only** — không modify existing subcommand behavior.
2. **Graceful degradation** — vault missing / write fail → warning stderr + exit 0. KHÔNG bao giờ fail ship vì vault.
3. **Opt-in default** — `auto_log = false`. Không disrupt existing users.
4. **Atomic write** — tmp + rename. Không partial file trong vault.
5. **Zero clippy warnings** — theo CLAUDE.md DoD.
6. **Test coverage** — mỗi module `src/note/` có `#[cfg(test)] mod tests` với tempfile vault.
7. **Strip diacritics filename** — cross-platform an toàn (APFS + NTFS + ext4 khác nhau, Unicode NFD/NFC traps).

---

## Nghiệm thu

### Automated (CLAUDE.md DoD)
- [ ] `cargo build --release` — zero warnings
- [ ] `cargo test` — all pass (38 hiện tại + tests mới cho `note/`)
- [ ] `cargo clippy -- -D warnings` — clean
- [ ] `cargo fmt -- --check` — clean
- [ ] No debug code

### Manual Testing
- [ ] `ship note --project tarot --ticket TEST-1 --message "hello"` → tạo file `~/VibeNotes/10_Projects/tarot/logs/<today>-<desc>.md` với content đúng format spec
- [ ] `OBSIDIAN_VAULT_PATH` unset + default `~/VibeNotes` missing → warning stderr, exit 0
- [ ] Vault path không writable → warning stderr, exit 0
- [ ] `ship check` với `auto_log = true` → post-success gọi note export, log path
- [ ] `ship check` với `auto_log = false` (default) → không call note
- [ ] MCP tool `ship_note_export` qua Claude Code MCP client → response OK
- [ ] Filename collision (chạy 2 lần cùng commit) → suffix -XXXX
- [ ] Commit subject có tiếng Việt "Thêm tính năng đặc biệt" → filename ASCII `them-tinh-nang-dac-biet`
- [ ] Test với obsidian-git plugin enable trên vault thật → verify không loop (chạy ship 3 lần trong 1h, check git log không spam)

### Regression
- [ ] `ship check`, `ship canary`, `ship learn` existing behavior không đổi
- [ ] `test_tool_router_has_5_tools` pass (updated from 4)
- [ ] Các project chưa enable `[obsidian]` → ship behavior y hệt trước

### Docs Gate (CLAUDE.md ship)
- [ ] `docs/CHANGELOG.md` — entry P002
- [ ] `docs/ARCHITECTURE.md` — Sections 1, 3, 4, 5 update
- [ ] `docs/CONVENTION.md` — thêm gotcha nếu phát hiện (diacritics, atomic write)
- [ ] `docs/DISCOVERIES.md` — Discovery Report P002 chèn đầu file

---

## Rủi ro

1. **Ship codebase structure khác em đoán** → Discovery Report bắt buộc, update docs trước code.
2. **Obsidian-git write race** → mitigation: atomic write (tmp + rename). Đã đủ.
3. **Tiếng Việt có dấu trong commit** → strip diacritics cross-platform, ASCII filename.
4. **Scope creep** → thợ KHÔNG refactor code ngoài 7 tasks. Nếu phát hiện cleanup cần thiết → ghi vào BACKLOG, không đụng.
5. **`dirs` crate version conflict** → phiếu pre-approve version `"5"`, nếu conflict với existing deps → DỪNG, hỏi Sếp.

---

## Rollout plan (sau khi code done)

### Phase 1 — Test trên tarot (1-3 ngày)
- Enable `[obsidian] auto_log = true` trong `~/tarot/.ship.toml`.
- Mỗi ship v2.1.4+ tự log vào vault.
- Sếp review vault note sau 1 tuần → fix edge cases.

### Phase 2 — Roll out các project khác (1 tuần)
- Enable per-project: BlockAds, creative-brain-api, docs-gate, guard, jarvis, media-rating-app, quality-gate, ship (self), vps.
- `soulsign-marketing` không có git remote → skip.
- Sếp update `.ship.toml` từng project.

### Phase 3 — Cross-project dashboard (optional, future)
- Script đọc tất cả `10_Projects/*/logs/*.md` → generate `10_Projects/DASHBOARD.md`.
- Hoặc Dataview plugin query (không cần code).
- Defer khi nào Sếp thấy cần.

---

## Cost estimate

- **Design verify (Discovery)**: 0.5 ngày (thợ grep code, confirm 11 anchors)
- **Code**: 1-1.5 ngày (subcommand + hook + config + MCP + tests)
- **Docs + CHANGELOG**: 0.5 ngày
- **Total**: 2-2.5 ngày

---

## Open Questions (Sếp decide trước khi implement)

1. **Tên subcommand**: `ship note` (không sub-verb, YAGNI — em đề xuất) hay `ship note export` (verb rõ ràng, để dành chỗ cho `ship note list/open` sau)?
2. **Diacritics crate**: `unicode-normalization` (chuẩn, +1 dep) hay inline ASCII fallback table (zero dep, tốn 20 dòng code)?
3. **Implementation picks up khi nào** — ngay sau P001 merge, hay chờ dogfooding tarot xong?
4. **Migrate branch type**: hôm nay branch là `docs/P002-obsidian-note` (phiếu draft). Khi implement — rename sang `feat/P002-obsidian-note` hay merge docs branch trước rồi tạo feat branch mới?

---

## Nguồn gốc

Phiếu gốc em viết ở `~/VibeNotes/10_Projects/ship/tickets/SHIP-NOTE-OBSIDIAN-INTEGRATION.md` (2026-04-24, pre-P001 template). Phiếu này là migration sang format TICKET_TEMPLATE mới (P001). Content giữ nguyên intent, format chuẩn hoá. Phiếu gốc có thể archive khỏi vault sau khi P002 merge.
