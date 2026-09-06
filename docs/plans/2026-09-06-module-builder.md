# Module Builder — ERPNext-like Platform

วันที่: 2026-09-06
สถานะ: Active
อ้างอิง: [`2026-09-06-next-roadmap.md`](2026-09-06-next-roadmap.md) (เสร็จครบ 4 Phase)

## วิสัยทัศน์

LOGHOLIZON เป็น low-code platform แบบ ERPNext (DocType) / Odoo (Studio):
สร้าง module ขึ้นมาเองในระบบได้เลย ไม่ต้องเขียน code

## Boundary

- `packages/core`: Rust HTTP service, domain rules, SQLite schema/migrations, transactions
- `packages/app`: Nuxt UI + public HTTP gateway (UI ใน `app/`, routes ใน `server/api`)
- ห้ามแก้ migration ที่ apply แล้ว — เพิ่ม migration ใหม่เท่านั้น
- ห้ามเพิ่ม SQL หรือ business rule ใน Nitro handlers
- ทุก non-trivial logic ต้องมี test อย่างน้อย 1 ตัว

## Scope change

- ✅ **Reference field** — เดิมไม่มี → เพิ่ม: field ชี้ entity อื่น + validate + options endpoint
- ✅ **Field types เพิ่ม** — เดิมมี text/number/date/select → เพิ่ม checkbox/textarea/currency
- ✅ **Computed field** — เดิมไม่มี → เพิ่ม: template interpolation, on read
- ✅ **Module grouping** — เดิมไม่มี → เพิ่ม: entity.module + sidebar grouping
- ❌ ยังไม่ทำ: arithmetic computed, computed ใน search/sort/export, reference cascade policy

---

## Phase A — Reference field (กุญแจหลัก) ✅ Done 2026-09-06

- Migration `0013_module_builder.sql` (รวม 3 คอลัมน์: ref_entity + computed_expr + module)
- `Field`/`FieldWithPermission` + `ref_entity`; `validate_reference_field` (ต้องมี target, ไม่ชี้ตัวเอง, entity มีจริง)
- Existence check ใน `validate_payload_for_role` → 400 unknown reference
- `GET /v1/entities/{id}/options` (id + label, label = text field แรกหรือ id, limit 500, can_view)
- Gateway + client + admin field editor (ref_entity picker) + record form (USelectMenu ทั้ง 2 branches) + fieldLabel resolve
- Test `reference_field_validation`

## Phase B — Field types เพิ่ม ✅ Done 2026-09-06

- `validate_field_type` + checkbox/textarea/currency (boolean branch มีอยู่แล้ว)
- `document-form.ts`: checkbox default false + boolean coercion + strip computed
- UI: UCheckbox / UTextarea / UInput number; admin typeItems + badge colors
- Test `new_field_types_validate` + vitest boolean/computed cases (17 tests)

## Phase C — Computed field ✅ Done 2026-09-06

- `Field` + `computed_expr`; validate ต้องมี expr มี `{}`; incoming computed key → 400
- `compute_field_value` (template `{field}`) + `apply_computed_fields` ใน `get_document` + `list_documents_as_role`
- UI read-only display; admin computed_expr input
- Test `computed_field_interpolates_on_read`

## Phase D — Module grouping ✅ Done 2026-09-06

- `Entity` + `module`; `update_entity` รับ module; client `CoreEntity` + module
- Sidebar nested children (Other ท้ายสุด, กลุ่มเดียว flat เหมือนเดิม) + ⌘K suffix
- Entity Manager edit modal + module input
- Test `entity_module_roundtrip`

## Phase E — พิสูจน์: Inventory module ✅ Done 2026-09-06

- Seed `product` + `warehouse` + `stock_move` (reference fields, module=Stock, workflow draft→confirmed→done, ON CONFLICT upsert)
- entities 2→5; `seed_is_idempotent` + `admin_status_reports_counts` อัปเดต
- Browser verify: product/warehouse/stock_move สร้างผ่าน API, bad ref 400, computed summary, module=Stock

## Gates (ทุก phase)

```powershell
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir packages/app run build
pnpm run test
pnpm --dir packages/app run check
pnpm --dir packages/app run e2e
```

Gates ผ่านครบ 2026-09-06 (fmt, clippy, cargo test, vitest 17, typecheck, build, e2e 7)
