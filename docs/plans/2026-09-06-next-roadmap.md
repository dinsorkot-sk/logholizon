# Next Roadmap — Visual Builder, Notifications, Reporting, Quality

วันที่: 2026-09-06
สถานะ: Active
อ้างอิง: [`2026-09-05-rust-core-erp.md`](2026-09-05-rust-core-erp.md) (เสร็จ) · [`2026-09-05-ux-ui-fixes.md`](2026-09-05-ux-ui-fixes.md) (เสร็จ) · [`2026-09-05-product-hardening.md`](2026-09-05-product-hardening.md) (เสร็จ) · [`packages/app/design.md`](../../packages/app/design.md)

## Boundary

- `packages/core`: Rust HTTP service, domain rules, SQLite schema/migrations, transactions. งานที่ต้องเพิ่ม endpoint/domain logic อยู่ที่นี่
- `packages/app`: Nuxt UI + public HTTP gateway. UI อยู่ `packages/app/app`; gateway routes อยู่ `packages/app/server/api`
- ห้ามแก้ migration ที่ apply แล้ว — เพิ่ม migration ใหม่เท่านั้น
- ห้ามเพิ่ม SQL หรือ business rule ใน Nitro handlers
- ทุก non-trivial logic ต้องมี test อย่างน้อย 1 ตัว
- หลักการออกแบบยึด `design.md`: List → side panel, สีเดียวมีความหมายเดียว, identifier เป็น mono ภาษาคนเป็น sans, Simple by default powerful when needed

## Scope change (เทียบกับ constraint เดิม)

- ✅ **Visual builder** — เดิม defer ไว้ ("visual builder") → เปิด scope: form layout designer
- ✅ **Notifications** — เดิม defer ไว้ (ไม่มีใน constraint เดิม) → เปิด scope: webhook-first, email ทีหลัง
- ❌ ยังไม่ทำ: branching workflow, D1/cloud deploy, API tokens — ต้อง scope change ใหม่เมื่อต้องการ

## ลำดับ (ตามที่ user เลือก)

1. **Phase 1 — Quality & Debt** (ฐานให้มั่นก่อน งานเล็ก)
2. **Phase 2 — Visual Builder** (หัวใจ low-code)
3. **Phase 3 — Notifications** (webhook-first)
4. **Phase 4 — Reporting & Analytics** (ต่อยอด dashboard/pm)

---

## Phase 1 — Quality & Debt

> เป้าหมาย: ลด debt ที่รู้อยู่ + ตั้ง E2E เป็น gate ก่อนขยาย feature

### 1.1 E2E tests (Playwright)

- **งาน:** เพิ่ม `@playwright/test` ใน `packages/app`; script `e2e` รันกับ dev server (core + app) + demo seed
- **Flow หลัก:** login (demo) → create record → edit → transition → delete; admin: entity CRUD, backup/restore, user management
- **Acceptance:** `pnpm --dir packages/app run e2e` ผ่านใน CI (job ใหม่ใน `.github/workflows/ci.yml`)

### 1.2 Refactor time helpers

- **งาน:** `[entity].vue` ยังมี local copy ของ `parseDate/relativeTime/absoluteTime/actionLabel` → ใช้ `app/utils/audit-time.ts` ร่วมกัน (ลบ local copy)
- **Acceptance:** ไม่มี duplicate helper; typecheck ผ่าน

### 1.3 Consistency ของ entity list API

- **งาน:** `admin/audit.vue` ยังเรียก `/api/meta/entities` (admin-only) — ตรวจว่าใช้ `/api/entities` ได้หรือต้องแยก; รวม endpoint ให้สม่ำเสมอ
- **Acceptance:** ทุกหน้าใช้ `/api/entities` สำหรับ list ที่ user เห็นได้; admin pages ใช้ `/api/meta/*` เฉพาะที่ต้อง manage

### 1.4 Gates

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir packages/app run build
pnpm run test
pnpm --dir packages/app run check
pnpm --dir packages/app run e2e
```

---

## Phase 2 — Visual Builder (Form Layout Designer)

> เป้าหมาย: จัด layout ของ form โดยไม่แตะ code — section/group, ลำดับ field, live preview

### 2.1 Layout metadata

- **งาน:** migration ใหม่ `0010_form_layout.sql`: `_entity_form_layout (entity_id PK, config JSON)` — config = `{ sections: [{ id, label, fields: [field_id...] }] }`; field ที่ไม่อยู่ใน config ต่อท้าย section "Other"
- **งาน:** core `repository.rs`: `get/update_entity_form_layout` (validate field ids อยู่ใน entity); `http.rs`: `GET/PUT /v1/meta/entities/{id}/form-layout` (admin)
- **Acceptance:** layout เก็บ/อ่านได้; field id ไม่รู้จัก → 400

### 2.2 Form designer UI

- **งาน:** tab ใหม่ "Form Layout" ใน `admin/meta/entity.vue` — สร้าง/ลบ section, ลาก field ระหว่าง section (ใช้ drag-drop ง่ายๆ หรือ up/down buttons ก่อน), live preview ของ form
- **Acceptance:** จัด layout → preview เห็นผลทันที; save → reload ยังอยู่

### 2.3 Runtime render

- **งาน:** `[entity].vue` form loop อ่าน layout config (ผ่าน `/api/entities/{id}` หรือ endpoint แยก) แทนการเรียงตาม metadata ตรงๆ; fallback = เรียงตาม field position เดิม
- **Acceptance:** entity ที่มี layout → form แสดงตาม section; entity ที่ไม่มี → เหมือนเดิม

### 2.4 Gates

- Gates เดิม + test: `form_layout_crud_and_validation` ใน `packages/core/tests/fields.rs`

---

## Phase 3 — Notifications (Webhook-first)

> เป้าหมาย: แจ้งเตือนระบบภายนอกเมื่อ workflow transition / record เปลี่ยน — webhook ก่อน, email ทีหลัง

### 3.1 Core outbox + hook

- **งาน:** migration `0011_notifications.sql`: `_notification_rule (id, entity_id, trigger, target_url, active, created_at)` + `_notification_delivery (id, rule_id, document_id, action, payload JSON, status, attempts, last_error, created_at)`
- **งาน:** hook ใน `transition_document` (และ create/update ตาม trigger config): insert delivery row + fire-and-forget POST
- **Acceptance:** transition → delivery row ถูกสร้าง; webhook ถูกเรียก

### 3.2 Delivery + retry

- **งาน:** worker loop (ต่อยอดจาก scheduled backup task): ส่ง pending deliveries, retry 3 ครั้ง (backoff), mark failed; timeout + size cap
- **Acceptance:** webhook fail → retry → failed หลัง 3 ครั้ง; delivery log ครบ

### 3.3 Rule CRUD + UI

- **งาน:** core `GET/POST /v1/meta/entities/{id}/notification-rules`, `PUT/DELETE /v1/meta/notification-rules/{id}`, `GET /v1/admin/notification-deliveries`; gateway + client methods
- **งาน:** UI tab "Notifications" ใน `admin/meta/entity.vue` (rule list + create modal) + delivery log ใน `/admin/settings` หรือหน้าแยก
- **Acceptance:** สร้าง rule → transition → เห็น delivery ใน log

### 3.4 Gates

- Gates เดิม + test: `notification_rule_crud` + `transition_creates_delivery` ใน `packages/core/tests/workflow.rs`

---

## Phase 4 — Reporting & Analytics

> เป้าหมาย: saved reports + charts ต่อยอดจาก `dashboard/pm` — ไม่สร้าง engine ใหม่

### 4.1 Core aggregation

- **งาน:** `GET /v1/reports/...`: count by status / by select field, series ตามเวลา (ต่อยอดจาก `count_documents_by_status`); enforce can_view
- **Acceptance:** aggregate ถูกต้อง + permission ตรง

### 4.2 Saved reports

- **งาน:** migration `0012_reports.sql`: `_report (id, entity_id, name, config JSON, created_by, created_at)`; core CRUD + `http.rs` routes (admin + user ตาม can_view)
- **Acceptance:** สร้าง/ลบ report; config เก็บ group-by/filter/chart type

### 4.3 UI

- **งาน:** `/app/reports` — report list + builder (entity, group-by, chart type) + chart cards (bar/line/pie ใช้ Nuxt UI หรือ lightweight chart lib) + export CSV ของผลลัพธ์
- **Acceptance:** สร้าง report → เห็น chart + export; permission ตาม can_view

### 4.4 Gates

- Gates เดิม + test: `report_crud_and_aggregation` ใน `packages/core/tests/documents.rs`

---

## Gates (ทุก phase)

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir packages/app run build
pnpm run test
pnpm --dir packages/app run check
```

## หมายเหตุ

- Email ใน Notifications: เริ่ม webhook ก่อน; SMTP เพิ่มทีหลังเมื่อมี use case จริง
- Visual builder เริ่มจาก up/down + section grouping ก่อน drag-drop เต็มรูปแบบ (ประหยัด effort)
- Reporting ใช้ aggregation ใน core (Rust) ไม่ใช่ client-side — ข้อมูลใหญ่ไม่ควรโหลดมาคำนวณใน browser