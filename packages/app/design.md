# UX/UI Design Spec — ERP Low-Code Platform (LOGHOLIZON)

> อิงจาก Supabase Dashboard (`github.com/supabase/supabase`) · Stack: Nuxt 4 + Nuxt UI 4

## 0. ทำไมอิง Supabase Dashboard

ระบบนี้คือ **metadata-driven ERP** งานหลักคือจัดการ structured data ผ่าน UI เช่น สร้าง entity/field, ดูและแก้ record เป็นตาราง, ตั้งค่า permission, ดู log ซึ่งสอดคล้องกับโจทย์ของ Supabase Dashboard:

- **Table Editor** → Phase 2: Dynamic List/Form
- **Database → Tables/Columns editor** → Phase 1: Entity/Field Manager
- **Auth → Policies** → Role/permission
- **Logs Explorer** → Phase 3: Audit Log

## 1. Design Tokens

### 1.1 Color

| Token | Hex | ใช้กับ |
|---|---|---|
| `--bg-base` | `#1C1C1C` | พื้นหลัง app หลัก |
| `--bg-surface` | `#212121` | sidebar, topbar |
| `--bg-panel` | `#292929` | card, table row |
| `--bg-panel-hover` | `#2E2E2E` | row/card hover |
| `--border-default` | `#2E2E2E` | เส้นแบ่งทั่วไป |
| `--border-strong` | `#3A3A3A` | input/card border |
| `--text-primary` | `#EDEDED` | หัวข้อ, ค่าตาราง |
| `--text-secondary` | `#A0A0A0` | label, helper text |
| `--text-muted` | `#707070` | placeholder, disabled |
| `--brand` | `#3ECF8E` | primary action, active nav, focus ring |
| `--brand-hover` | `#34B87C` | brand hover |
| `--danger` | `#F16565` | destructive/error |
| `--warning` | `#F3B94B` | pending/warning |
| `--info` | `#5CA8F1` | informational badge |

Light theme (ทางเลือก): background `#FFFFFF`, panel `#F8F8F8`, border `#E4E4E4`, text `#1C1C1C` / secondary `#6B6B6B`, brand คงเดิม

**กติกา:** `--brand` ใช้เฉพาะ primary action, active nav, focus ring ห้ามใช้ตกแต่ง

### 1.2 Typography

| Role | Font | Weight/Size |
|---|---|---|
| UI ทั่วไป | Inter | 400/500, 13–14px |
| Page title | Inter | 600, 20px |
| Identifier/code | IBM Plex Mono หรือ JetBrains Mono | 400, 13px |
| Dashboard number | Inter | 600, 28–32px |

ชื่อ entity, field, API key ใช้ monospace เสมอ; label ภาษาคนใช้ Inter

### 1.3 Layout primitives

- Border-radius: `6px`
- Card: `1px solid var(--border-default)` ไม่ใช้ shadow
- Spacing: `4 / 8 / 12 / 16 / 24 / 32px`
- Sidebar: `220px` expanded / `56px` collapsed
- Content: full width ไม่ล็อก max-width

## 2. App Shell

```text
┌────┬──────────────────────────────────────────────────┐
│Icon│ Topbar: Breadcrumb · Search · [New] · Avatar    │
│Nav ├──────────────────────────────────────────────────┤
│    │                  Page content                    │
│56px│                                                  │
└────┴──────────────────────────────────────────────────┘
```

### Icon Nav

Fixed `56px`: logo/switcher, `Table` → `/app/[entity]`, `LayoutGrid` → `/admin/meta/entity`, `GitBranch` → `/admin/meta/workflow`, `ClipboardList` → `/app/pm`, `Settings`, avatar ด้านล่าง

### Sub-sidebar

`240px` เปิดตามบริบท:

- `/app/[entity]`: รายชื่อ entity พร้อม search และ group by module
- `/admin/meta`: Entity / Workflow / Role tabs

### Topbar

Breadcrumb, global search (`⌘K`), primary action ตามหน้า (`+ New Entity`, `+ New Record`), avatar menu

## 3. Page Designs

### Phase 1 — Entity Manager (`/admin/meta/entity`)

ใช้ layout **List → Detail** สองคอลัมน์ ไม่ใช้ modal:

- Sub-sidebar: search, entity list, `+ New Entity`
- Detail: entity name, tabs `Fields | Permissions | Views`
- Fields เป็นตาราง: Name (mono), Type, Required, Actions
- Type badge: `text` = `--info`, `select` = `--brand`, `number` = `--warning`
- Actions menu: Edit / Duplicate / Delete
- `+ Add field` เปิด `USlideover` ด้านขวา; ตารางด้านหลังยังมองเห็น
- Type selector มี icon: text `Aa`, select chevron, number `#`, date calendar

Components: raw `<table>`, `USlideover`, `UBadge`, `USelectMenu`

### Phase 2 — Dynamic List & Form (`/app/[entity]`)

List view ใช้แนวทาง Supabase Table Editor:

- Toolbar: Filter chip, Sort, column visibility (`⚙ columns`), New
- Select field แสดง dot + label โดยสีมาจาก metadata
- Row hover แสดง Edit; checkbox รองรับ bulk delete/export
- คลิก row เปิด dynamic form ใน `USlideover`; context ของ list ต้องไม่หาย
- Form เรียงตาม field order; label อยู่เหนือ input
- Required field มี `*` สีแดงต่อท้าย label
- Validation error อยู่ใต้ field ทันที
- Autosave debounce `800ms`; แสดง `Saving...` / `Saved`

Components: raw `<table>`, `USlideover`, `UInput`, `USelectMenu`, custom status dot

### Phase 3 — Workflow Builder (`/admin/meta/workflow`)

ใช้ vertical step list ไม่ใช้ drag-and-drop หรือ canvas:

```text
pm_approval
① draft
   ↓ submit
② submitted
   ↓ approve    ↓ reject
③ approved     ④ draft (back)
   ↓ complete
⑤ done
```

- State เป็น horizontal card: state name (mono), color badge, transition count
- Transition เป็น text link คลิกแก้ permission
- `+ Add state`, `+ Add transition`
- สอดคล้องกับ linear state machine; ไม่ออกแบบ branching ซับซ้อน

#### Status Badge + Audit Log

- Status = dot + label ถัดจาก record title
- แสดงเฉพาะ transition ที่ role มีสิทธิ์ เช่น `[Submit]`, `[Approve]`, `[Reject]`
- Audit log เป็น vertical timeline ใหม่ไปเก่า: `จาก draft → submitted โดย Nan · 2 นาทีที่แล้ว`

### Phase 4 — PM Dashboard (`/app/pm`) + Export/Import

Dashboard ประกอบด้วย summary cards `Open`, `Overdue`, `Done this week` และ list view จาก Phase 2 ที่ filter ล่วงหน้า

- ไม่ใช้ sparkline หากไม่มี time-series จริง
- `Export` อยู่ใน list toolbar; export current filter/columns เป็น CSV
- `Import` เปิด side panel: upload `.xlsx` → preview 5 แถว + column mapping → Confirm
- ไม่ทำ wizard หลายหน้า

## 4. Component Mapping — Nuxt UI 4

| Design element | Component | หมายเหตุ |
|---|---|---|
| Icon nav / sub-sidebar | `UNavigationMenu` | custom width |
| Data table | raw `<table>` | preferred over `UTable` |
| Field/record edit panel | `USlideover` | จากขวา |
| Type/select dropdown | `USelectMenu` | icon slot ต่อ option |
| Status/type badge | `UBadge` + custom dot | ไม่มี dot built-in |
| Filter builder | `UPopover` + `USelectMenu` | ประกอบเอง |
| Summary cards | `UCard` | number ใหญ่ + label |
| Workflow step list | custom component | `UButton variant="link"` |
| Autosave status | custom text + `UIcon` | check/spinner |

## 5. States ที่ต้องรองรับ

- Entity ไม่มี field: ข้อความ + `+ Add your first field` กลางพื้นที่
- List ไม่มี record: `ยังไม่มี work_order — [+ New]`
- Loading: skeleton rows และ dashboard cards ไม่ใช้ spinner กลางจอ
- Transition ไม่ได้รับอนุญาต (`403`): ซ่อนปุ่มตั้งแต่แรก ไม่ disable

## สรุปหลักการ

1. **List → side panel** แทน modal/หน้าใหม่ เพื่อรักษา context
2. **สีเดียวมีความหมายเดียว:** เขียว = primary action
3. **Identifier เป็น mono, ภาษาคนเป็น sans**
4. **UI สะท้อน scope:** workflow เป็น list; ไม่ทำ field-level permission ใน MVP
