# UX/UI Fix Plan — Nexa (LOGHOLIZON)

วันที่: 2026-09-05
สถานะ: Active
อ้างอิง: UX/UI Audit (2026-09-05) · [`2026-09-05-rust-core-erp.md`](2026-09-05-rust-core-erp.md) · [`packages/app/design.md`](../../packages/app/design.md)

## Boundary

- `packages/core`: Rust HTTP service, domain rules, SQLite schema/migrations, transactions. งานที่ต้องเพิ่ม endpoint/domain logic อยู่ที่นี่
- `packages/app`: Nuxt UI + public HTTP gateway. UI อยู่ `packages/app/app`; gateway routes อยู่ `packages/app/server/api`
- ห้ามแก้ migration ที่ apply แล้ว — เพิ่ม migration ใหม่เท่านั้น
- ห้ามเพิ่ม SQL หรือ business rule ใน Nitro handlers
- ทุก non-trivial logic ต้องมี test อย่างน้อย 1 ตัว
- หลักการออกแบบยึด `design.md`: List → side panel, สีเดียวมีความหมายเดียว, identifier เป็น mono ภาษาคนเป็น sans, Simple by default powerful when needed

## ลำดับความสำคัญ (Impact × Frequency × Effort)

1. 🔴 User blocking / data integrity (error state, feedback)
2. 🔴 Core low-code workflow (Field Editor)
3. 🟠 Data-dense list UX (pagination, search/filter/sort)
4. 🟠 Form & workflow UX (dirty state, transition, audit)
5. 🔵 Navigation & polish

---

## Phase 1 — Data Integrity & Feedback (Critical Bugs) ✅ Done

> เป้าหมาย: ผู้ใช้ไม่เห็นข้อมูลผิดพลาด และรู้เสมอว่า action สำเร็จ/ล้มเหลว
> สถานะ: เสร็จสิ้น 2026-09-05 · gates ผ่าน (build, test, typecheck, browser verify)

### 1.1 แยก error state ของ documents fetch

- **Problem:** `[entity].vue` ตรวจแค่ `pending` แล้ว `v-else` แสดงตาราง — fetch error กลายเป็น "No records yet"
- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - แยก 3 states: `pending` → skeleton, `error` → `UAlert` + ปุ่ม Retry, `success` → table
  - ใช้ `documentsStatus` และ `documentsError` ที่มีอยู่แล้ว
- **Acceptance:** เมื่อ Rust core ลง ตารางแสดง error alert ไม่ใช่ "No records yet"

### 1.2 เพิ่ม toast feedback ทุก action

- **Problem:** save/create/delete/transition/import ไม่มี success/error feedback
- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`, `packages/app/app/pages/admin/meta/entity.vue`
- **งาน:**
  - ใช้ `useToast()` ของ Nuxt UI
  - Success: "Saved", "Record created", "Record deleted", "Submitted", "Imported N rows"
  - Error: แสดง `cause.data.message` ที่ได้จาก API (ไม่ใช่ "Something went wrong")
- **Acceptance:** ทุก action สำคัญมี toast; error toast แสดงสาเหตุจริง

### 1.3 Replace native `confirm()` ด้วย Delete Modal

- **Problem:** `confirm('Delete this record permanently?')` — ไม่ undo, ไม่ consistent, keyboard trap
- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - ใช้ `UModal` ยืนยัน: แสดง record id/label + "การกระทำนี้ไม่สามารถย้อนกลับ"
  - ปุ่มยืนยัน `color="error"`, ปุ่มยกเลิก ghost
  - หลังลบสำเร็จ → toast + ปิด modal
- **Acceptance:** ลบต้องผ่าน modal ยืนยัน; ไม่มี native `confirm()` เหลือใน codebase

### 1.4 Dashboard empty state + entity selector

- **Problem:** `entityId` hardcode `work_order`; `counts` ว่าง → หน้าโล่ง
- **ไฟล์:** `packages/app/app/pages/dashboard.vue`
- **งาน:**
  - เพิ่ม entity selector (ใช้ `USelectMenu` จาก `/api/meta/entities`)
  - `counts` ว่าง → แสดง empty state "ยังไม่มีข้อมูล — สร้าง record แรก" + CTA ไป `/app/[entity]`
  - Loading → skeleton cards (ไม่ใช่ spinner กลางจอ)
  - Error → `UAlert` + Retry (มีอยู่แล้ว เก็บไว้)
- **Acceptance:** หน้าแรกไม่ว่างเปล่า; สลับ entity ได้; ทุก state มี UI

### 1.5 ปิด DevTools ใน production

- **ไฟล์:** `packages/app/nuxt.config.ts`
- **งาน:** `devtools: { enabled: process.env.NODE_ENV !== 'production' }`
- **Acceptance:** production build ไม่มีปุ่ม Nuxt DevTools

---

## Phase 2 — Field Editor (Core Low-code Workflow) ✅ Done

> เป้าหมาย: ผู้ใช้สร้าง entity + field + option ผ่าน UI ได้ครบวงจร — นี่คือหัวใจของ low-code platform
> สถานะ: เสร็จสิ้น 2026-09-05 · gates ผ่าน (build, test, typecheck, browser verify)

### 2.1 Rust core: Field CRUD endpoints

- **ไฟล์:** `packages/core/src/http.rs`, `packages/core/src/repository.rs`, `packages/core/src/error.rs`
- **งาน:**
  - `POST /v1/meta/entities/{id}/fields` — create field (name, type, required, position)
  - `PUT /v1/meta/fields/{id}` — update field (name, type, required, position)
  - `DELETE /v1/meta/fields/{id}` — delete field (cascade options)
  - `POST /v1/meta/fields/{id}/options` — create option (value, label)
  - `PUT /v1/meta/options/{id}` / `DELETE /v1/meta/options/{id}` — option CRUD
  - Validate: `type` ∈ {text, number, date, select}; `name` เป็น snake_case; select ต้องมี option ≥ 1
  - Error mapping: duplicate field name → `Conflict`; field not found → `NotFound`
- **Acceptance:** `cargo test --workspace` ผ่าน; มี test สำหรับ create field + duplicate name + option validation

### 2.2 Migration: field position

- **ไฟล์:** `migrations/0004_field_position.sql` (ใหม่)
- **งาน:** `ALTER TABLE _meta_field ADD COLUMN position INTEGER NOT NULL DEFAULT 0;` + backfill เรียงตาม `name`
- **Acceptance:** migration forward-only; `list_fields` เรียงตาม `position, name`

### 2.3 Gateway routes (thin)

- **ไฟล์:** `packages/app/server/api/meta/entities/[id]/fields.post.ts`, `packages/app/server/api/meta/fields/[id].put.ts`, `packages/app/server/api/meta/fields/[id].delete.ts`, `packages/app/server/api/meta/fields/[id]/options.post.ts` (ใหม่)
- **งาน:** parse → validate → call `coreClient()` → map response; ไม่มี business logic
- **Acceptance:** ผ่าน `pnpm --dir packages/app run build`

### 2.4 Entity Manager: List → Detail layout

- **ไฟล์:** `packages/app/app/pages/admin/meta/entity.vue`
- **งาน:**
  - แยกเป็น 2 คอลัมน์: ซ้าย = entity list + search + `+ New Entity`; ขวา = detail ของ entity ที่เลือก
  - Detail มี tabs: `Fields | Permissions | Views` (Permissions/Views แสดง placeholder "coming soon" ตาม scope)
  - Fields tab: ตาราง Name (mono), Type badge, Required, Actions (Edit/Duplicate/Delete)
  - Type badge สีตาม spec: `text` = info, `select` = brand, `number` = warning, `date` = neutral
  - `+ Add field` เปิด `USlideover` (ตารางด้านหลังยังเห็น)
  - Type selector มี icon: text `Aa`, select chevron, number `#`, date calendar
  - Select type → แสดง option editor (เพิ่ม/ลบ option value+label)
- **Acceptance:** สร้าง entity → เพิ่ม field → เห็น field ในตาราง → เปิด `/app/[entity]` เห็น form/table ตาม field

### 2.5 Entity Actions: Edit / Duplicate / Delete

- **ไฟล์:** `packages/app/app/pages/admin/meta/entity.vue` + gateway routes
- **งาน:**
  - `PUT /v1/meta/entities/{id}` (rename label/name), `DELETE /v1/meta/entities/{id}` (cascade)
  - Duplicate = create copy + suffix id
  - Delete ใช้ `UModal` ยืนยัน + เตือน cascade (fields, documents หาย)
- **Acceptance:** แก้ชื่อ/ลบ/duplicate entity ได้ผ่าน UI; delete มี confirm

### 2.6 Entity ID validation

- **ไฟล์:** `packages/app/app/pages/admin/meta/entity.vue` + `packages/core/src/repository.rs`
- **งาน:** validate `^[a-z][a-z0-9_]*$`; แสดง hint "lowercase, no spaces (e.g. work_order)"; error ใต้ field
- **Acceptance:** id ผิด format ไม่ submit; error message ชัดเจน

---

## Phase 3 — Data-dense List UX (ERP Core) ✅ Done

> เป้าหมาย: ตารางรองรับข้อมูลจำนวนมาก — pagination, search, filter, sort
> สถานะ: เสร็จสิ้น 2026-09-05 · gates ผ่าน (build, test, typecheck, browser verify)

### 3.1 Pagination

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - ส่ง `limit` (50) + `offset` ไป API (มีอยู่แล้วใน client)
  - แสดง "Showing 1–50 of 1,234" + ปุ่ม Prev/Next (หรือ `UPagination`)
  - แสดง pagination เฉพาะเมื่อ `total > limit`
- **Acceptance:** ข้อมูล >50 rows ดูครบ; ผู้ใช้รู้ว่ามีทั้งหมดกี่ row

### 3.2 Search / Filter / Sort

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue` + `packages/core/src/repository.rs` (query params)
- **งาน:**
  - Rust: `list_documents` รองรับ `search` (LIKE บน text fields), `status` filter, `sort_by` + `sort_dir`
  - UI: toolbar เหนือตาราง — search input, filter chip (ตาม field type: select → dropdown, date → range), sort dropdown
  - ตาม spec: Filter chip, Sort, column visibility (`⚙ columns`)
- **Acceptance:** ค้นหา/กรอง/เรียงได้; URL สะท้อน state (query params) เพื่อ share/link ได้

### 3.3 Label mapping (แทน raw value)

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - Status badge: map `field.options` value → label; ใช้ dot + label ตาม spec
  - Transition button: แสดง label (Submit, Mark Done) ไม่ใช่ raw action
  - Audit log: แสดง label + relative time
- **Acceptance:** ผู้ใช้ไม่เห็น `draft`/`submit` ดิบใน UI

### 3.4 Bulk actions + column visibility

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - Checkbox ต่อ row + select all; bulk delete (ผ่าน modal ยืนยัน), bulk export
  - `⚙ columns` toggle ซ่อน/แสดง column
- **Acceptance:** เลือกหลาย row → ลบ/export พร้อมกันได้

### 3.5 Empty state ที่อธิบายได้

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:** "ยังไม่มี work_order — [+ New record]" + CTA; แยกจาก error state
- **Acceptance:** empty ≠ error; มี CTA เสมอ

---

## Phase 4 — Form & Workflow UX

> เป้าหมาย: form ปลอดภัย (ไม่เสียข้อมูล) และ workflow เข้าใจง่าย

### 4.1 Dirty-state protection

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - ติดตามว่า form เปลี่ยนจากค่าเริ่มต้นหรือไม่
  - ปิด slideover (X/Close/Cancel) เมื่อ dirty → `UModal` "Discard changes?"
  - ตาม spec: autosave debounce 800ms + `Saving...`/`Saved` indicator (ทำเป็น option — ถ้า autosave ใช้ dirty-state ไม่ต้องถาม)
- **Acceptance:** ปิด slideover โดยไม่ตั้งใจไม่เสียข้อมูล

### 4.2 Sticky slideover footer

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:** ย้าย Cancel/Save/Delete/Transition ไป `#footer` ของ `USlideover` — sticky เสมอ ไม่ต้อง scroll
- **Acceptance:** form ยาว → ปุ่ม Save มองเห็นตลอด

### 4.3 Transition UX

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - Track ปุ่มที่กำลังทำงาน (`transitioningAction`) — ไม่ใช่ทุกปุ่มหมุนพร้อมกัน
  - Workflow fetch error → `UAlert` + Retry (ไม่หายเงียบ)
  - หลัง transition สำเร็จ → toast + status badge อัปเดต
- **Acceptance:** กด transition ปุ่มเดียวหมุน; error แสดง; success มี feedback

### 4.4 Audit log อ่านง่าย

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:** relative time ("2 นาทีที่แล้ว") + absolute บน tooltip; แสดง "จาก draft → open" ตาม spec
- **Acceptance:** อ่านลำดับเหตุการณ์ได้ทันที

### 4.5 Import preview error เป็น list

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:** error แสดงทีละบรรทัดพร้อม row number; ไม่ join ด้วย `'; '`
- **Acceptance:** ผู้ใช้รู้ว่าแถวไหนผิดและแก้อย่างไร

### 4.6 Status field ไม่ hardcode ชื่อ

- **ไฟล์:** `packages/core/src/repository.rs` (metadata) + `packages/app/app/pages/app/[entity].vue`
- **งาน:** กำหนด status field ผ่าน metadata (เช่น flag `is_status` ใน `_meta_field` หรืออ้างอิง workflow definition) แทน `field.name === 'status'`
- **Acceptance:** entity ที่ใช้ชื่อ field อื่น (state, workflow_status) ทำงานเหมือนกัน

---

## Phase 5 — Navigation & Polish

> เป้าหมาย: ผู้ใช้รู้ว่าอยู่ที่ไหน ค้นหาเร็ว และ UI consistent

### 5.1 Breadcrumb + Global Search (⌘K)

- **ไฟล์:** `packages/app/app/layouts/default.vue`, `packages/app/app/pages/app/[entity].vue`
- **งาน:**
  - Breadcrumb ใน navbar: `Entities / Work Order`
  - Command palette `⌘K`: ค้นหา entity + ไปหน้า
- **Acceptance:** ทุกหน้าแสดงตำแหน่ง; ค้นหา entity ด้วย ⌘K ได้

### 5.2 Sidebar error handling + footer

- **ไฟล์:** `packages/app/app/layouts/default.vue`
- **งาน:**
  - `useFetch('/api/meta/entities')` error → แสดง alert + retry (ไม่หายเงียบ)
  - เปลี่ยน footer "Rust core owns persistence" เป็น user-facing (version/environment) หรือลบ
- **Acceptance:** core down → sidebar แสดงปัญหา; footer ไม่มี dev note

### 5.3 Keyboard navigation

- **ไฟล์:** `packages/app/app/pages/app/[entity].vue`
- **งาน:** shortcut `n` = new record, `⌘K` = search, Esc = ปิด slideover (มีอยู่แล้วจาก Nuxt UI)
- **Acceptance:** power user ทำงานโดยไม่ใช้เมาส์สำหรับ action หลัก

### 5.4 UI token consistency

- **ไฟล์:** `packages/app/app/pages/**`
- **งาน:**
  - เปลี่ยน `text-gray-500` → token (`text-muted`/`text-secondary`)
  - `hover:bg-gray-50` → `hover:bg-[var(--ui-bg-elevated)]`
  - Refresh button pattern เดียวกันทั้ง dashboard และ entity manager
  - Placeholder ของ create-entity form ไม่ซ้ำกับข้อมูลจริง (เช่น `e.g. customer`)
  - ซ่อนตารางเมื่อ entity ไม่มี field → แสดง empty state กลางพื้นที่
- **Acceptance:** ไม่มี hardcode color ใน pages; component เดียวกันมีพฤติกรรมเดียวกัน

---

## Gates

ทุก Phase ต้องผ่าน:

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir packages/app run build
pnpm run test
```

และตรวจด้วย browser: เปิด `/dashboard`, `/admin/meta/entity`, `/app/work_order` ตรวจ states (loading/error/empty/success) ครบ

## หมายเหตุ

- Phase 1–2 เป็น prerequisite ของ UX ที่เหลือ (error state + field editor เปิดทางให้ form/table ทำงานจริง)
- Phase 3–4 ทำคู่กันได้ (list toolbar กับ form/workflow ไม่ทับกัน)
- Phase 5 ทำได้ตลอดระหว่างรอ review
- ทุก endpoint ใหม่ต้อง versioned ใต้ `/v1` และ map error ผ่าน `ApiError { code, message }` เดิม