# Beta Test — v1.0.1

**Branch**: `v1.0.1` @ `80eb82a`  
**Verified**: 2026-10-05 (local, WSL Ubuntu, Rust 1.88.0, Node 22.17.0, pnpm 11.9.0)

## Readiness Gates

| Gate | Result |
|------|--------|
| `cargo fmt --all -- --check` | ✅ Clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ Clean |
| `cargo test --workspace -- --test-threads=1` | ✅ 157 passed, 0 failed |
| `pnpm install --frozen-lockfile` | ✅ Clean |
| `pnpm --dir packages/app run test` (Vitest) | ✅ 6 files, 33 tests passed |
| `pnpm --dir packages/app run check` (vue-tsc) | ✅ Clean |
| `pnpm --dir packages/app run build` | ✅ Built (8.74 MB) |
| `cargo build --workspace --release` | ✅ Built |
| E2E Playwright (`E2E_APP_MODE=preview`) | ✅ 11/11 passed |
| CLI smoke on a fresh DB | ✅ See below |

CLI smoke (fresh `CORE_DATABASE_URL`): `migrate` (twice, idempotent) →
`seed --demo` → `check` → `backup` (`VACUUM INTO`) → `restore` without
`--force` refused → `restore --force` (rollback copy preserved under
`backups/pre-restore-*.db`) → `check` = `database ok`.

## Beta Setup

### Docker (recommended for hosted beta)

```bash
docker compose up --build
```

Open http://localhost:3000. A fresh volume shows the first-run admin setup
form; no demo users are created. Data lives in the `logholizon-data` volume.

### Local

```bash
pnpm install
cargo run -p logholizon-cli -- migrate
cargo run -p logholizon-cli -- seed --demo
pnpm run dev
```

### Desktop (Tauri)

See `packages/desktop/README.md`. First run creates `admin` / `admin12345`
plus demo data.

## Test Accounts

| Source | Username | Password | Role |
|--------|----------|----------|------|
| `seed --demo` | `admin` | `admin123` | admin |
| `seed --demo` | `demo` | `demo1234` | user |
| Desktop first run | `admin` | `admin12345` | admin |

These are public defaults. Do not use `seed --demo` on any instance reachable
from the internet; reset them with `logholizon-cli reset-password` before
sharing a desktop build.

## Core Test Scenarios (8 Key Areas)

Testers should follow this standardized domain (**Clinic & Patient Management**) or a domain of their choice:

### 1. Module Builder (Zero-Code Domain Creation)
- Log in as `admin`. Go to `/admin/modules` -> **Create Module** (`clinic`, label: `Clinic Management`).
- Add Entity: `patient` (Label: `Patient`).
  - Fields: `hn` (Text, Unique, Required), `full_name` (Text, Required), `birth_date` (Date), `phone` (Text), `allergies` (Text).
- Add Entity: `appointment` (Label: `Appointment`).
  - Fields: `appointment_no` (Text, Required), `patient_id` (Relation -> `patient`), `schedule_time` (DateTime), `doctor_name` (Text), `status` (Select: `pending`, `confirmed`, `completed`, `cancelled`).
- Configure Form sections and Table views (sort by `schedule_time` descending).
- Publish module: Draft -> Review -> Published -> Enabled.
- **Verification**: `Clinic Management` appears in navigation; entities and views are accessible.

### 2. Runtime CRUD, Search, Filter & Excel/CSV Operations
- Navigate to the `Patient` list.
- Create 3 patient records manually; edit one and delete one with confirmation.
- Test Search (partial match on name or HN) and multi-field Filtering.
- Export records to `.xlsx`.
- Import batch records via CSV/Excel upload, verify preview mapping, and confirm import.

### 3. Role-Based Access Control (RBAC) & Field-Level Security
- Go to `/admin/roles`:
  - Create Role `Receptionist`: CRUD on `patient` and `appointment`, but no Delete; hide or set read-only on `allergies`.
  - Create Role `Doctor`: full CRUD including `allergies`.
- Go to `/admin/users` -> Create `nurse_joy` (`Receptionist`) and `dr_house` (`Doctor`).
- Log in as `nurse_joy`: confirm delete button is absent, `allergies` field is hidden/masked, and `/admin` routes are denied.
- Log in as `dr_house`: confirm full access.

### 4. Linear Workflow & Audit Trail
- Configure linear workflow on `appointment`: `Draft` -> `Scheduled` -> `In Consultation` -> `Completed` (or `Cancelled`).
- Set role permissions on transitions (only `Doctor` can transition to `In Consultation` and `Completed`).
- Execute transitions on a record; verify timeline history and `/admin/audit` logs.

### 5. Automations, Notifications & Webhooks
- Set up an automation rule on `appointment.created` -> trigger in-app notification to `dr_house`.
- Create a test appointment; check notification bell for `dr_house` and check delivery logs under `/admin/settings` -> Notification Deliveries.

### 6. Reports & Dashboards
- Create dashboard `Clinic Daily Overview`: add KPI card (Total Appointments) and Status distribution chart.
- Create tabular report for Patients grouped by registration month; export to CSV/Excel.

### 7. Ready-Made Package Installation (Solution Library)
- Go to `/admin/solutions` (Solution Library).
- Install `vehicle.module.json` or `dormitory.module.json`.
- Verify the package lifecycle completes automatically (Published & Enabled) and coexists with `Clinic Management`.

### 8. Database Backup, Restore & Integrity Check
- Under `/admin/settings` -> Database & Backups, click **Create Backup**.
- Create a new landmark record (e.g. Patient `TEST RECOVERY`).
- Perform Restore from the backup snapshot; verify rollback copy created and the landmark record is safely reverted.
- Run integrity check: verify `database ok`.

## Feedback & Issue Triage Framework

| Level | Definition | Target Resolution |
|---|---|---|
| **P0 - Blocker** | Crash, panic, database corruption/deadlock, login failure, security leak. | Hotfix within 24h (`v1.0.2`). |
| **P1 - Critical** | Core flow in the 8 scenarios cannot complete, no workaround. | Fix before GA (`v1.0.2`). |
| **P2 - Normal Bug** | Minor UI glitch, validation message wording, non-blocking edge case. | Backlog (`v1.0.2` or `v1.0.3`). |
| **P3 - Enhancement** | UX/UI theme revamp, visual workflow canvas, new field types. | Defer to `v1.1.0` (UI v2). |

## Known Limitations

- Workflow is linear only (no branching or canvas), by design.
- The UI v2 redesign (`docs/design/design-v2.md`) is a draft and not part of
  this beta.
- Latest GitHub Actions status for `v1.0.1` was not checked here; confirm CI is
  green on GitHub before inviting testers.

## Reporting Issues

Open a GitHub issue with:
1. Version/branch: `v1.0.1`
2. Deployment mode: Docker, Local, or Desktop Tauri
3. Request ID (`X-Request-Id` from error toast or `/admin/observability`)
4. Steps to reproduce, expected result, and actual result
5. Server/process logs or screenshot
