# Next Roadmap (referenced by AGENTS.md)

Status: superseded — no outstanding scoped work remains.

## Why this file exists

`AGENTS.md` references this path as the "active" roadmap. The file was
missing from the tree; this stub restores the reference so the doc set is
internally consistent, and records what happened to the items it was meant
to scope.

## Scoped items and their resolution

`AGENTS.md` names four items as explicitly in-scope for this roadmap:
Auth, multi-sheet Excel, visual form layout, and webhook notifications.
All four are already implemented and verified in the master plan
(`docs/plans/README.md`):

- **Auth** — `v0.0.29` "Production Auth and Security" (Phase B, COMPLETE):
  session lifecycle, brute-force rate limiting, CORS lockdown, HMAC webhook
  verification, SSRF/secret redaction. 20/20 security tests passing.
- **Visual form layout** — `v0.2.1` "Dynamic Form Layout Runtime" (COMPLETE):
  metadata-driven section grouping and field ordering, verified by
  `packages/app/tests/e2e/dynamic-ui.spec.ts`.
- **Webhook notifications** — Phase 12 "Notification & Webhook Runtime"
  (COMPLETE, 100%): HTTP webhook delivery, execution logs, isolation,
  inbound HMAC verification.
- **Multi-sheet Excel** — merged into the canonical `v0.0.28` baseline
  (`preview_workbook_xlsx`/`confirm_workbook_xlsx`/`export_workbook_xlsx`
  in `packages/core/src/http.rs` + `repository.rs`), confirmed present by
  ancestry check on 2026-09-13.

## Master plan status

`docs/plans/README.md` "100% Master Implementation Plan" reports all 20
phases COMPLETE as of `v0.2.7`. Post-`v1.0.0` stabilization work (Docker/
Tauri packaging verification, E2E hardening, Axum stress/load testing) is
also complete as of `v1.0.1` (see `/memories/session/plan.md` for the
session-local tracker).

## Working rule for any further roadmap items

Per `AGENTS.md`: "Keep workflow linear; no D1, branching, canvas, or API
tokens without explicit scope change." Do not begin new feature work
(visual drag-and-drop builder redesign, connector/webhook ecosystem
expansion, multi-tenancy portal, etc.) without an explicit scope change
from the user recorded in this file or a successor roadmap document. Do
not invent additional phases on top of the completed 100% master plan.
