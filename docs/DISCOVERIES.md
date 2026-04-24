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
