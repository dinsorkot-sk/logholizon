# UX/UI Design Spec v2 — LOGHOLIZON (ร่าง)

> สถานะ: **ร่างเพื่อพิจารณา** · 2026-10-05 · จะมาแทน `packages/app/design.md` (v1 ที่อิง Supabase) เมื่ออนุมัติ
> Stack เดิม: Nuxt 4 + Nuxt UI 4 (Tailwind 4) · ไม่เพิ่ม dependency ใหม่ ยกเว้นฟอนต์
> Moodboard ประกอบ: [`redesign-moodboard.html`](./redesign-moodboard.html)

หมายเหตุด้าน scope: ตาม `docs/plans/2026-09-06-next-roadmap.md` การ redesign UI ต้องบันทึกเป็น scope change ใน roadmap ก่อนเริ่มลงมือ

## 0. เหตุผลที่ต้องเปลี่ยนจาก v1

v1 ใช้ Supabase Dashboard เป็นต้นแบบ ซึ่งเหมาะกับ admin ที่แก้ schema แต่ LOGHOLIZON มีผู้ใช้สองกลุ่ม:

| กลุ่ม | งานหลัก | ต้องการ |
|---|---|---|
| **Builder** (admin) | สร้าง module, entity, field, form, workflow, role, report | ความแม่นยำ, เห็นโครงสร้างทั้งหมด, draft/publish ปลอดภัย |
| **Operator** (staff) | กรอก/ค้น/อนุมัติ record ทุกวัน | อ่านง่าย, รู้ว่าต้องทำอะไรต่อ, ไม่เห็นเมนูที่ไม่เกี่ยว |

ปัญหาที่พบใน v1:

- พื้นหลังเป็น dark อย่างเดียว ส่วนสีเขียวแบบ dev tool ทำให้ใช้งานต่อเนื่องนาน ๆ แล้วล้าตา
- `Public Sans` ไม่มีภาษาไทย ข้อความไทยจึง fallback ไปใช้ฟอนต์อื่นจนดูไม่สม่ำเสมอ
- Sidebar รวมเมนูใช้งานกับเมนู Admin 8 รายการไว้ด้วยกัน
- หน้า builder ยาวมาก (`entity.vue` 2,316 บรรทัด, `[entity].vue` 1,843 บรรทัด)
- Workflow แสดงเป็นแค่ badge จึงไม่ได้ใช้ประโยชน์จากการที่ workflow เป็นแบบ linear

## 1. แนวคิดหลัก: Workspace + Studio

ข้อมูลชุดเดียวกันแสดงผลได้สองโหมด โดยสลับที่ตัวสลับด้านล่าง sidebar ส่วน role ที่ไม่ใช่ admin จะเห็นเฉพาะ Workspace

| โหมด | Route | ต้นแบบ |
|---|---|---|
| **Workspace** | `/`, `/app/**` | Linear (shell, ⌘K) + Odoo (หน้า record แบบเอกสาร) + Airtable (view tabs) |
| **Studio** | `/studio/**` (ย้ายจาก `/admin/**`) | Retool (3 pane), Tally (เพิ่ม field ด้วย `/`), Linear (workflow states), Twenty (data model) |

**Signature element: Workflow track.** เส้นแนวนอนที่แสดง state ของ workflow (`Draft ── Submitted ── Approved ── Checked in`) ใช้ใน record header, list (ขนาดย่อ), Studio workflow editor และ empty state ของ workflow เพื่อสะท้อนชื่อ LOGHOLIZON (horizon) และจุดแข็งเรื่อง linear workflow

## 2. Design Tokens

### 2.1 Color (light เป็นค่าเริ่มต้น)

| Token | Light | Dark | ใช้กับ |
|---|---|---|---|
| `--canvas` | `#F2F3EF` | `#14181F` | พื้นหลัง app |
| `--surface` | `#FFFFFF` | `#1B2029` | panel, table, form |
| `--surface-2` | `#F7F8F5` | `#222833` | sidebar, read-only input, table header |
| `--line` | `#DCDFD8` | `#2C3340` | เส้นแบ่ง |
| `--line-strong` | `#C9CDC4` | `#3A4250` | border ของ input |
| `--ink` | `#18202B` | `#E8EBEF` | ข้อความหลัก |
| `--ink-2` | `#4A5361` | `#A9B1BC` | ข้อความรอง |
| `--ink-3` | `#7C8592` | `#7B8491` | label, meta, placeholder |
| `--horizon` | `#1F5F8B` | `#5B9BD0` | primary action, active, focus ring |
| `--horizon-soft` | `#E3EDF5` | `#1E3347` | selected row, filter ที่ตั้งค่าแล้ว |
| `--stamp` | `#C2410C` | `#F0844F` | required `*`, error, destructive |
| `--ok` / `--warn` | `#2F7D4F` / `#B7791F` | `#5BBE85` / `#E2A94B` | ใช้ระดับระบบเท่านั้น |

กติกา:

1. `--horizon` มีความหมายเดียว คือ "สิ่งที่กดได้หรือเลือกอยู่" ห้ามนำไปใช้ตกแต่ง
2. สีของ status และ select option มาจาก metadata ของ module (`color` ใน option) ห้าม hardcode
3. แต่ละ module มีสีประจำตัว (`module.color`) ซึ่งใช้ได้แค่จุดสี่เหลี่ยมใน sidebar และ icon ของ app
4. Contrast ของข้อความต้องผ่าน WCAG AA (`--ink-3` บน `--surface` ≥ 4.5:1)

Mapping สำหรับ Nuxt UI (`app.config.ts`): `primary: 'horizon'` (เพิ่ม scale 50–950 ใน `@theme`) และ `neutral: 'slate'`

### 2.2 Typography

| Role | Font | Weight / Size / LH |
|---|---|---|
| UI ทั่วไป, ไทย+อังกฤษ | **IBM Plex Sans Thai** (fallback: IBM Plex Sans, Noto Sans Thai) | 400 / 14px / 1.55 |
| Page title | Plex Sans Thai | 600 / 22px / 1.25 |
| Section heading | Plex Sans Thai | 600 / 15px |
| Label, meta | Plex Sans Thai | 400 / 12.5px, `--ink-3` |
| Identifier (API name, record ID, key) | **IBM Plex Mono** | 400 / 12.5px |
| ตัวเลขในตาราง | Plex Sans Thai `tabular-nums` | ชิดขวา |

- ใช้ฟอนต์ตระกูลเดียว (Plex) ทั้งไทย อังกฤษ และ mono เพื่อให้ baseline และน้ำหนักเท่ากัน
- Self-host ผ่าน `@nuxt/fonts` หรือไฟล์ใน `public/` เพราะ desktop (Tauri) ต้องทำงานแบบ offline
- ไม่ใช้ ALL CAPS กับ label เนื่องจากภาษาไทยไม่มีตัวพิมพ์ใหญ่ และจะทำให้หน้าตาไม่สม่ำเสมอ

### 2.3 Layout primitives

- Radius ไล่ตามลำดับชั้น: input/button `6px`, card/panel `8px`, modal/slideover `12px`
- ใช้ border 1px แทน shadow ยกเว้น popover และ slideover ซึ่งมี shadow 1 ระดับ
- Spacing: `4 / 8 / 12 / 16 / 24 / 32 / 48`
- Density ตั้งต่อผู้ใช้ได้: comfortable (row 40px, ค่าเริ่มต้น) / compact (row 32px)
- Sidebar: `232px` / collapsed `56px` · Inspector: `300px` · Record side panel: `720px` หรือเต็มจอบน mobile

## 3. Shell

```text
┌──────────────┬───────────────────────────────────────────┐
│ Workspace ▾  │ Breadcrumb             [Secondary] [Primary]│
│ ◷ งานของฉัน   ├───────────────────────────────────────────┤
│ ⌕ ค้นหา  ⌘K  │                                           │
│              │               Page content                │
│ Dormitory    │                                           │
│  ▪ Rooms     │                                           │
│  ▪ Bookings  │                                           │
│ Inventory    │                                           │
│  ▪ Items     │                                           │
│──────────────│                                           │
│[Workspace|Studio]                                        │
│ Avatar       │                                           │
└──────────────┴───────────────────────────────────────────┘
```

- **Workspace sidebar:** งานของฉัน, ค้นหา, และรายการ app (module) ที่ขยายดู entity ได้ ไม่มีเมนู admin
- **Studio sidebar:** Modules, Solution Library, Users & Roles, Audit Log, Observability, Settings
- **⌘K:** palette ชิดด้านบนจอ จัดผลลัพธ์เป็นกลุ่ม Records, Go to, Actions (เช่น "New booking") และ Studio (เฉพาะ admin)
- ต่อยอดจาก `UDashboardGroup` และ `UDashboardSidebar` เดิม โดยเปลี่ยนเฉพาะรายการเมนูและ token

## 4. Workspace pages

### 4.1 งานของฉัน (`/`) แทน `/dashboard`

- ส่วนแรกคือ "รอคุณดำเนินการ": record ที่อยู่ใน state ซึ่ง role ของผู้ใช้มี transition ให้กด เรียงจากเก่าไปใหม่
- ส่วนที่สองคือ "ล่าสุดที่คุณแก้ไข" 10 รายการ
- Dashboard ที่ admin ปักหมุดให้ role จะแสดงเป็น card ด้านล่าง
- ข้อมูลทั้งหมดมาจาก API ของ workflow และ audit ที่มีอยู่แล้ว ไม่ต้องเพิ่ม business logic ใน Nitro

### 4.2 Entity list (`/app/[entity]`)

```text
Dormitory / Rooms                              [Export] [New room]
All rooms │ Board by status │ Vacant this month │ + View
[Building: Tower A ×] [Status ≠ Maintenance ×] [+ Filter] [Sort]
─────────────────────────────────────────────────────────────────
Room   Building   Capacity   Status          Monthly rate
A-101  Tower A    2          ● Vacant            4,500.00
```

- View tabs มาจาก saved views ใน metadata และแต่ละ view จำ filter, sort และ column ของตัวเอง
- **Board view** ใช้ได้เมื่อ entity มี workflow หรือ select field การลากการ์ดจะเรียก transition API ถ้าไม่ได้รับอนุญาต การ์ดจะเด้งกลับและแสดง toast อธิบายเหตุผล
- **Calendar view** ใช้ได้เมื่อมี date field
- Filter ที่ตั้งค่าแล้วเป็น chip แบบเส้นทึบ พื้น `--horizon-soft` ส่วนที่ยังไม่ตั้งเป็นเส้นประ
- Bulk action bar จะลอยขึ้นมาที่ด้านล่างเมื่อเลือกตั้งแต่ 1 แถวขึ้นไป

### 4.3 Record (side panel, ขยายเป็นเต็มหน้าได้)

```text
BK-0412                                              [⤢] [×]
Somchai Jaidee · A-102
●───────●───────○───────○
Draft  Submitted Approved Checked in
[Approve] [Send back to draft]                         Saved
┌─────────────────────────────────┬──────────────────────┐
│ Resident *        Room *        │ [3 Invoices][1 Maint]│
│ Start date *      End date      │ History              │
│                   ⚠ ต้องหลัง start │ ● Draft→Submitted Nan │
│ ── Billing ──                   │   2 นาทีที่แล้ว         │
└─────────────────────────────────┴──────────────────────┘
```

- **Workflow track** อยู่บนสุด ทำให้เห็นทั้ง state ปัจจุบันและ state ถัดไป
- ปุ่ม transition แสดงเฉพาะที่ role นี้มีสิทธิ์ (ซ่อนปุ่มแทนการ disable ตาม v1) โดยปุ่มแรกเป็น primary
- Field เรียงตาม `form_layout.sections` ใน metadata ตัว `*` ใช้สี `--stamp` และ error แสดงใต้ field
- Autosave ใช้ debounce 800ms และแสดงสถานะ `Saving…` หรือ `Saved` ข้างปุ่ม action
- คอลัมน์ขวาแสดงจำนวน related record (นับจาก reference field ที่ชี้มาที่ entity นี้) และ timeline ที่ใช้ข้อมูลจาก audit log
- บน mobile คอลัมน์ขวาย้ายไปเป็น tab "History"

## 5. Studio pages

ทุก builder ใช้โครง **3 pane** เดียวกัน:

```text
┌ Module tree ─┬ Canvas ─────────────────┬ Inspector ────┐
│ Dormitory    │ Fields                  │ Status·select │
│  building 3  │ ⋮⋮ ↗ Building  Ref  Req │ Label         │
│ ▸room     5  │ ⋮⋮ Aa Number   Text Req │ API name (ro) │
│  resident 6  │ ⋮⋮ ◉ Status    Sel  Req │ Options ●●●   │
│ Workflow     │ พิมพ์ / เพื่อเพิ่ม field     │ Required  [on]│
│ Roles        │                         │               │
├──────────────┴─────────────────────────┴───────────────┤
│ มีการเปลี่ยนแปลง 2 รายการใน draft v1.3   [Review] [Publish] │
└────────────────────────────────────────────────────────┘
```

| Builder | Canvas | Inspector |
|---|---|---|
| Entity / Fields | รายการ field แบบ block ลากเรียงได้ เพิ่มด้วยการพิมพ์ `/` + ชื่อ type | คุณสมบัติของ field ที่เลือก |
| Form layout | Section และ field แบบ preview จริง (ใช้ `vue-draggable-plus` เดิม) | Section/field config |
| Views | Preview ตารางจริงพร้อม column | Columns, filter, sort, group |
| Workflow | รายการ state แนวตั้ง แสดง **track preview** ด้านบน ลากเรียงได้ และเลือก default state ได้ | Transition, role ที่กดได้, เงื่อนไข |
| Roles | Permission matrix: แถวคือ entity (จัดกลุ่มตาม module) คอลัมน์คือ View/Create/Edit/Delete/Transition/Export | Field-level redaction ของ entity ที่เลือก |
| Reports / Dashboards | Preview chart | Source, group by, aggregate |

กติกาของ Studio:

- **Publish bar** ติดอยู่ด้านล่างเสมอเมื่อ draft ต่างจาก published แสดงจำนวนการเปลี่ยนแปลง ปุ่ม "Review changes" เปิด diff และ "Publish vX.Y" ระบุเลขเวอร์ชันตรง ๆ เพื่อให้สอดคล้องกับ draft/publish ใน Module Runtime
- **API name** แสดงด้วย mono และเป็น read-only หลัง publish ส่วน label แก้ได้เสมอ
- ไม่ใช้ modal ใน Studio ยกเว้นการยืนยันลบหรือ publish
- ไม่มี canvas หรือ branching ใน workflow ตาม AGENTS.md

## 6. Component mapping (Nuxt UI 4)

| Element | Component | หมายเหตุ |
|---|---|---|
| Shell | `UDashboardGroup`, `UDashboardSidebar`, `UDashboardPanel` | ใช้เดิม |
| Mode switch | `UTabs` (pill) หรือ `UButtonGroup` | ท้าย sidebar |
| Workflow track | **ใหม่** `WorkflowTrack.vue` | props: `states`, `current`, `size: 'sm' \| 'md'` |
| Record timeline | **ใหม่** `RecordTimeline.vue` | ดึงจาก `/api/audit?record_id=` |
| View tabs | `UTabs` variant link | |
| Filter chip | `UButton` + `UPopover` | |
| Data table | raw `<table>` (คงตาม v1) | sticky header, row 40/32px |
| Board | **ใหม่** `RecordBoard.vue` + `vue-draggable-plus` | |
| Record panel | `USlideover` width 720px | ปุ่มขยายไป route เต็มหน้า |
| 3-pane builder | **ใหม่** `StudioLayout.vue` | slots: `tree`, `default`, `inspector`, `footer` |
| Field row | **ใหม่** `FieldBlock.vue` | |
| Permission matrix | raw `<table>` + `UCheckbox` (indeterminate) | |
| Command palette | `UCommandPalette` ใน `UModal` (position top) | |

## 7. States

- **First use:** อธิบายว่าหน้านี้ใช้ทำอะไรและมีปุ่มหลักหนึ่งปุ่ม เช่น "ยังไม่มี room — สร้าง room แรก" ฝั่ง Studio ให้เลือกเริ่มจาก Solution Library ได้
- **No results:** "ไม่พบ room ที่ตรงกับ filter" พร้อมปุ่ม "Clear filters"
- **Error:** บอกว่าเกิดอะไรขึ้นและวิธีแก้ เช่น "เชื่อมต่อ core ไม่ได้ — ตรวจสอบว่า core กำลังทำงาน" พร้อมปุ่ม Retry
- **Loading:** skeleton ที่มีรูปทรงเหมือนเนื้อหาจริง ไม่ใช้ spinner กลางจอ
- **403 transition:** ซ่อนปุ่มไว้ตั้งแต่แรก หากใช้การลากใน board จะเด้งกลับและแสดง toast

## 8. Accessibility & quality floor

- Focus ring 2px `--horizon` offset 2px บนทุก element ที่โต้ตอบได้
- ทุก action ที่มี shortcut ต้องมีปุ่มที่มองเห็นได้ด้วย
- ไม่ใช้สีเป็นตัวสื่อความหมายอย่างเดียว status ต้องมี label คู่กับจุดสีเสมอ
- เคารพ `prefers-reduced-motion` โดย motion มีที่เดียวคือ track เลื่อนไป state ถัดไปหลัง transition สำเร็จ
- รองรับความกว้างตั้งแต่ 360px โดย record panel เต็มจอ และ Studio inspector กลายเป็น bottom sheet

## 9. ลำดับการ implement

1. **Tokens + ฟอนต์:** แก้ `main.css` และ `app.config.ts` ตั้ง light เป็นค่าเริ่มต้น ทุกหน้าเปลี่ยนตามทันที
2. **Shell:** แยกเมนู Workspace/Studio ใน `layouts/default.vue` แล้ว alias `/admin/**` → `/studio/**`
3. **WorkflowTrack + RecordTimeline:** ใส่ใน record panel ของ `app/[entity].vue`
4. **StudioLayout:** แตก `admin/meta/entity.vue` เป็น `ModuleTree` / `FieldBlock` list / `FieldInspector`
5. **งานของฉัน:** แทน `dashboard.vue`
6. **Board view + Permission matrix**
7. อัปเดต e2e (`tests/e2e/*.spec.ts`) ให้ selector อิง role หรือ label แทน class

## 10. References

- Linear UI redesign: https://linear.app/now/how-we-redesigned-the-linear-ui
- Linear workflows: https://linear.app/docs/configuring-workflows
- Twenty CRM (OSS): https://github.com/twentyhq/twenty
- Frappe UI / Espresso (OSS, Vue+Tailwind): https://github.com/frappe/frappe-ui · https://frappe.io/design/espresso
- Odoo 18: https://www.odoo.com/odoo-18-release-notes
- Airtable Interfaces: https://workmanagementhub.com/airtable-interfaces-designer-guide-2026/
- Teable (OSS): https://github.com/teableio/teable · Baserow (OSS): https://github.com/baserow/baserow
- Retool command palette: https://retool.com/blog/designing-the-command-palette
- Nuxt UI dashboard template: https://github.com/nuxt-ui-templates/dashboard
- Permission UX: https://www.saasui.design/blog/saas-permissions-roles-ux-patterns
- Empty states: https://carbondesignsystem.com/patterns/empty-states-pattern/
- Schedule-X (calendar, OSS): https://github.com/schedule-x/schedule-x
