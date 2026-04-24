# DISCOVERIES — Ship

Nhật ký phát hiện từ thợ khi làm phiếu. **Mới nhất lên trên** (giống CHANGELOG).

Sếp đọc file này TRƯỚC KHI viết phiếu tiếp theo để cập nhật mental model — phiếu sau không lặp lại sai lệch phiếu trước đã phát hiện.

**Format mỗi entry:**

```markdown
## P<NNN> — <ngày YYYY-MM-DD> — <tiêu đề ngắn>

### Assumptions trong phiếu — ĐÚNG:
- [liệt kê từng assumption khớp với code thật]

### Assumptions trong phiếu — SAI so với code thật:
- [assumption X: phiếu ghi "...", thực tế "..." → đã sửa docs Y]
- [hoặc "Không có"]

### Edge cases / limitations phát hiện thêm:
- [phiếu không đề cập nhưng thợ phát hiện khi code]
- [hoặc "Không có"]

### Docs đã cập nhật theo discoveries:
- [file nào sửa, sửa gì]
- [hoặc "Không có"]
```

**Luật cứng:**
- Discovery Report KHÔNG optional. Thiếu = phiếu CHƯA XONG (theo CLAUDE.md DoD).
- Ghi vào file này, KHÔNG chỉ báo cáo trong chat (chat bị clear sau session).
- Chèn entry mới đầu file, sau block hướng dẫn này — KHÔNG xoá entries cũ.

---

## P002 — 2026-04-24 — ship note Obsidian integration (phiếu drafted)

> **Lưu ý:** P002 hôm nay mới là DRAFTING phiếu (type=docs, migration từ vault). Implementation CHƯA chạy. Discovery cho phần implementation sẽ append vào entry này khi thợ pickup.

### Assumptions trong phiếu — ĐÚNG (verified khi migrate):
- `src/main.rs` dùng clap `#[derive(Subcommand)] enum Commands` (dòng 11, 55-56) — ✅ phiếu gốc đoán đúng.
- 4 MCP tools hiện tại: ship_check, ship_canary, ship_learn_add, ship_learn_search (`src/mcp/server.rs` dòng 37, 79, 114, 125) — ✅.
- Config pattern nested sub-struct + `#[serde(default)]` (TestConfig, DocsGateConfig, ...) — ✅.
- Module layout: feature = dir với `mod.rs` (ví dụ `canary/mod.rs` export `pub async fn run` dòng 48) — ✅, sẽ follow cho `src/note/`.
- `chrono` 0.4 có sẵn với feature `serde` — ✅ dùng cho date format.
- Vault `~/VibeNotes/10_Projects/ship/` có README.md + tickets/ (logs/ chưa tồn tại, sẽ tạo khi write đầu tiên) — ✅.
- obsidian-git plugin enabled — ✅ (race mitigation bằng atomic write là đủ).

### Assumptions trong phiếu — SAI so với code thật:
- Phiếu gốc vault nói "src/commands/note.rs (mới)" — **SAI**: ship không có folder `src/commands/`. Pattern thật là `src/<feature>/mod.rs` (giống `src/canary/mod.rs`). → Phiếu mới P002 đã sửa thành `src/note/mod.rs`.
- Phiếu gốc vault nói "src/mcp/" chung chung — thực tế có 3 files rõ: `mod.rs`, `server.rs`, `tools.rs`. → Phiếu mới chỉ định: params ở `tools.rs`, tool method ở `server.rs`.
- Phiếu gốc nói post-success hook ở "`src/commands/check.rs`" — **SAI**: hook có thể ở `src/pipeline/mod.rs` (vì pipeline là nơi check chạy). → Phiếu mới ghi "thợ xác định đúng file" và suggest `src/pipeline/mod.rs`.

### Edge cases phát hiện thêm:
- `dirs` crate version cần pre-approve ("5") — nếu conflict với existing deps (serde/tokio/reqwest) → Hard Stop cho thợ.
- File format "Learnings" section: phiếu gốc in "N/A" nếu không có `--message`. Em refactor → BỎ section hoàn toàn (tránh rác visual trong vault). Small UX improvement.
- Test helper cho vault: nên dùng `tempfile::TempDir` (ship đã dùng pattern này — gotcha #4 CLAUDE.md).
- Counter `test_tool_router_has_4_tools` → phải update thành `_has_5_tools` — regression test cần touch.

### Docs đã cập nhật theo discoveries:
- Phiếu gốc trong vault (`~/VibeNotes/10_Projects/ship/tickets/SHIP-NOTE-OBSIDIAN-INTEGRATION.md`) CHƯA sửa — Sếp decide: archive hay giữ làm lịch sử (em note trong P002 phiếu mới, section "Nguồn gốc").
- CLAUDE.md + ARCHITECTURE.md CHƯA update (sẽ làm khi thợ implement — không phải scope drafting).

---

## P001 — 2026-04-24 — Phiếu standard + docs discipline

### Assumptions trong phiếu — ĐÚNG:
- Tất cả 8 Verification Anchors đều khớp. Đáng chú ý:
  - Ship template cũ (`docs/ticket/TEMPLATE.md`) đúng là 32 dòng basic, thiếu Verification Anchors / Tasks structure / Files tables / Regression
  - `.phieu-counter = 0`, `.gitignore` đã có `.phieu-counter` entry (do `phieu-init` anh chạy trước)
  - Shell fn `phieu` trong `~/.zshrc` có lookup order `TICKET_TEMPLATE.md` → `TEMPLATE.md` → new template hoạt động ngay cho phiếu kế tiếp
  - Tarot template + CLAUDE.md (mid-chat section) port được sang ship với điều chỉnh nhẹ (Rust toolchain thay vì pnpm, bỏ sync-claude-web, bỏ GitHub MCP token-economy)

### Assumptions trong phiếu — SAI so với code thật:
- Không có. Phiếu này docs-only, không có code assumption nào cần verify.

### Edge cases phát hiện thêm:
- `.gitignore` được `phieu-init` (anh chạy) tự modify — bỏ vào P001 commit vì thuộc cùng scope (thiết lập phiếu workflow).
- `docs/ticket/PHASE-1.md` còn nằm trong ticket folder (không phải archive) — giữ nguyên vì historical, không thuộc scope P001. Có thể housekeeping sau.
- Phiếu taxonomy 2-trục (Loại + Risk) hơi verbose trong frontmatter template — acceptable trade-off vì 2 trục này đo 2 chiều khác nhau và cùng hữu ích cho docs-gate / review routing.

### Docs đã cập nhật theo discoveries:
- Không có docs pre-existing sai lệch (phiếu docs-only). Tất cả thay đổi là additive:
  - `docs/ticket/TICKET_TEMPLATE.md` (mới)
  - `docs/DISCOVERIES.md` (mới, file này)
  - `CLAUDE.md` (thêm section Phiếu Workflow + Mid-chat, update Discovery Report + Classification)
  - `docs/CHANGELOG.md` (entry Unreleased)
  - `docs/ticket/TEMPLATE.md` xoá

---
