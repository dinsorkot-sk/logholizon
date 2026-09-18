# Logholizon v0.2.0 Release Readiness

เอกสารนี้กำหนด release boundary และขั้นตอนตรวจสอบสำหรับ version ถัดจาก `v0.1.5` โดย version `v0.2.0` เป็นชื่อเป้าหมายเบื้องต้นจนกว่าจะมีการตัดสินใจเรื่อง compatibility อย่างเป็นทางการ

## Release boundary

Release นี้ครอบคลุม Rust core, Nuxt web gateway, SQLite persistence, authentication/authorization, audit, backup/restore และ Linux desktop validation ของ Tauri หากยังไม่มี artifact ที่ผ่านการตรวจบน Windows หรือ macOS ให้ประกาศเป็น known limitation แทนการอ้างว่ารองรับข้ามแพลตฟอร์ม

ฟีเจอร์ field-level หรือ record-level permission, multi-tenant isolation, auto-update และ code signing จะถือเป็น release feature เฉพาะเมื่อมี test และ artifact รองรับ หากไม่ผ่านเกณฑ์ให้ประกาศเป็น deferred limitation ใน release notes

## Required checks

ก่อน tag release ต้องผ่าน Rust fmt, clippy, workspace tests, release build, Nuxt test/typecheck/build, Playwright critical paths และ desktop frontend/Rust checks บน clean runner โดย CI ต้องเก็บ Nuxt output, desktop frontend และ Playwright diagnostics เป็น artifacts

สำหรับ release tag workflow ต้องสร้าง archive, SHA-256 checksums และ GitHub Release จาก tag เดียวกัน ห้ามใช้ไฟล์จาก working tree หรือ artifact ที่สร้างจาก commit คนละตัว

## Security checklist

ต้องตรวจว่า production session cookie มี `HttpOnly`, `Secure` และ `SameSite` เหมาะสม, authentication endpoints มี rate limit, protected `/v1` routes ตรวจ bearer session, CORS ใช้ allowlist และ response headers มี `nosniff`, frame protection, referrer policy และ HSTS เมื่อใช้ TLS

ต้องทดสอบ 401, 403, session expiry, password reset session invalidation, privilege escalation, IDOR และ permission denial logging โดยไม่ให้ token/password ปรากฏใน logs

## Data and recovery checklist

ต้องทดสอบ migration จากฐานข้อมูลของ release ก่อนหน้า, backup ด้วย `VACUUM INTO`, integrity check, corrupted backup rejection, failed import rollback และ restore rollback path ก่อนประกาศ production readiness

Production deployment ต้องมี volume ถาวรสำหรับ SQLite, backup retention ที่ไม่ลบ backup ล่าสุดที่ valid, disk-space monitoring และ runbook สำหรับหยุด writer, restore, ตรวจ readiness และ rollback

## Deployment smoke test

หลัง deploy ให้ตรวจ `/health`, `/ready`, login, role denial, document CRUD, workflow transition, dashboard และ logout โดยใช้บัญชีทดสอบที่ไม่ใช่ default credential หลัง initialization

## Rollback

หาก release มี error ให้หยุดการรับ write traffic, เก็บ database snapshot ปัจจุบัน, restore snapshot ที่ตรวจ integrity แล้ว, rollback application artifact ไปยัง tag ก่อนหน้า และตรวจ `/ready` กับ critical read paths ก่อนเปิด traffic กลับ

## Known limitations template

Release notes ต้องระบุ supported OS/architecture, SQLite single-host/single-writer limitation, deferred permission scopes, signing/auto-update status และ RTO/RPO ที่ทดสอบได้จริง
