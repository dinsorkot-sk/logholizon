# Product Hardening Plan — LOGHOLIZON

วันที่: 2026-09-05
สถานะ: Active
อ้างอิง: [`2026-09-05-rust-core-erp.md`](2026-09-05-rust-core-erp.md) · [`2026-09-05-ux-ui-fixes.md`](2026-09-05-ux-ui-fixes.md) · [`packages/app/design.md`](../../packages/app/design.md)

## Boundary

- `packages/core`: Rust HTTP service, domain rules, SQLite schema/migrations, transactions. งานที่ต้องเพิ่ม endpoint/domain logic อยู่ที่นี่
- `packages/app`: Nuxt UI + public HTTP gateway. UI อยู่ `packages/app/app`; gateway routes อยู่ `packages/app/server/api`
- ห้ามแก้ migration ที่ apply แล้ว — เพิ่ม migration ใหม่เท่านั้น
- ห้ามเพิ่ม SQL หรือ business rule ใน Nitro handlers
- ทุก non-trivial logic ต้องมี test อย่างน้อย 1 ตัว
- **Scope change (อนุมัติโดย user 2026-09-05):** เพิ่ม Auth + Roles — เดิม AGENTS.md ระบุ "no auth" ไว้

## ลำดับ (ตามที่ user เลือก)

1. 🔴 Backup/Restore UI + Settings — data safety
2. 🟠 Workflow Builder หน้าเฉพาะ — ตาม design.md Phase 3
3. 🟠 PM Dashboard — ตาม design.md Phase 4
4. 🔵 Auth + Roles (full) — scope change

---

## Phase A — Backup/Restore UI + Settings

### A.1 Core: admin endpoints

- **ไฟล์:** `packages/core/src/http.rs`, `packages/core/src/backup.rs`, `packages/core/src/main.rs`
- **งาน:**
  - `AppState` เพิ่ม `config: Config` (ต้องใช้ `database_url` หา db path)
  - `GET /v1/admin/status` → `{ version, database_path, integrity, entities, documents }`
  - `POST /v1/admin/backup` → `VACUUM INTO <db_dir>/backups/core-<ts>.db` → `{ path }`
  - `GET /v1/admin/backups` → list `{ name, size, modified }`
  - `GET /v1/admin/backups/{name}` → download (guard path traversal)
  - `POST /v1/admin/restore` `{ path, force }` → validate integrity → **staged restore**: copy ไป `<db_dir>/restore-pending.db` → `{ message }`
  - `POST /v1/admin/restart` → exit process หลัง response (process manager restart)
  - `main.rs` startup: ถ้ามี `restore-pending.db` → swap เข้าที่ก่อน connect pool → ลบ staging
- **Acceptance:** backup สร้างไฟล์ได้; restore ต้อง `force`; staged restore ถูก apply ตอน restart; test restore flow

### A.2 Gateway + client

- **ไฟล์:** `packages/app/server/api/admin/*`, `packages/app/server/core/client.ts`
- **งาน:** thin routes เรียก core; client methods `getAdminStatus`, `createBackup`, `listBackups`, `downloadBackup`, `restoreBackup`, `restartCore`
- **Acceptance:** ผ่าน `pnpm --dir packages/app run build`

### A.3 Settings page

- **ไฟล์:** `packages/app/app/pages/admin/settings.vue`, `packages/app/app/layouts/default.vue`
- **งาน:**
  - Status card: version, db path, integrity badge, entity/document counts
  - Backup: ปุ่ม "Create backup" → toast + path; รายการ backups (table) + Download / Restore
  - Restore: `UModal` ยืนยัน + ต้อง tick "I understand" (force) → หลังสำเร็จแสดง "Restart core to apply" + ปุ่ม Restart
  - เพิ่ม nav link Settings ใน sidebar
- **Acceptance:** backup → download ได้; restore ต้อง confirm; restart ทำงาน

---

## Phase B — Workflow Builder หน้าเฉพาะ

### B.1 Page

- **ไฟล์:** `packages/app/app/pages/admin/meta/workflow.vue`
- **งาน:** ตาม design.md Phase 3:
  - Entity selector (USelectMenu)
  - Vertical step list: state เป็น card (name mono, label, position, transition count) เรียงตาม position
  - Transition เป็น link ใต้ state: `from → to (action)` + ปุ่มลบ
  - `+ Add state`, `+ Add transition` (reuse endpoints จาก workflow CRUD ที่ทำแล้ว)
- **Acceptance:** เห็น flow draft → open → done แบบ visual; เพิ่ม/ลบ state/transition ได้

---

## Phase C — PM Dashboard

### C.1 Core: PM aggregates

- **ไฟล์:** `packages/core/src/repository.rs`, `packages/core/src/http.rs`
- **งาน:** `GET /v1/dashboard/pm` → `{ open, overdue, done_this_week, total }`
  - open = status != done
  - overdue = status != done AND due_date < today
  - done_this_week = status == done AND updated_at >= start of week (UTC)
- **Acceptance:** test aggregate logic

### C.2 Page

- **ไฟล์:** `packages/app/app/pages/app/pm.vue`
- **งาน:** summary cards (Open/Overdue/Done this week) + list view filter ตาม design.md Phase 4
- **Acceptance:** ตัวเลขถูกต้อง; คลิก card → ไป list ที่ filter แล้ว

---

## Phase D — Auth + Roles (scope change)

### D.1 Migration + core

- **ไฟล์:** `migrations/0006_auth.sql`, `packages/core/src/auth.rs` (ใหม่), `packages/core/src/http.rs`, `packages/core/src/seed.rs`
- **งาน:**
  - `_user` (id, username UNIQUE, password_hash, role admin|user, created_at), `_session` (token PK, user_id, created_at, expires_at)
  - `POST /v1/auth/register` (สร้าง user แรกเป็น admin), `POST /v1/auth/login` → `{ token, user }`, `POST /v1/auth/logout`, `GET /v1/auth/me`
  - Middleware: ทุก `/v1/*` ยกเว้น auth/health/version ต้องมี Bearer token
  - Role guard: admin เท่านั้นจัดการ meta (entities/fields/workflow/admin); user อ่าน/แก้ documents ได้
  - Password hash: argon2 (เพิ่ม dependency)
- **Acceptance:** login ได้; ไม่มี token → 401; user เรียก meta → 403; test auth flow

### D.2 UI

- **ไฟล์:** `packages/app/app/pages/login.vue`, `packages/app/app/middleware/auth.ts`, `packages/app/app/layouts/default.vue`, `packages/app/app/pages/admin/settings.vue`
- **งาน:**
  - Login page; session cookie (httpOnly) ผ่าน gateway
  - Route guard: ไม่มี session → redirect `/login`; role user → ซ่อน admin nav
  - User menu (avatar) + logout
- **Acceptance:** login → เข้า app; logout → กลับ login; user ไม่เห็น Entity Manager

---

## Gates

ทุก Phase ต้องผ่าน:

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir packages/app run build
pnpm run test
pnpm --dir packages/app run check
```

และตรวจด้วย browser: เปิด `/admin/settings`, `/admin/meta/workflow`, `/app/pm`, `/login` ตรวจ states ครบ

## หมายเหตุ

- Phase A–C ไม่มี dependency ต่อกัน ทำตามลำดับได้
- Phase D ทำท้ายสุด — ต้อง re-verify ทุกหน้า (auth guard กระทบทั้งหมด)
- Restore เป็น staged (swap ตอน restart) เพราะ Windows lock ไฟล์ DB ที่ pool เปิดอยู่