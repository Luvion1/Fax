# Agent System - Usage Guide

Panduan lengkap untuk menggunakan sistem agen dengan efektif.

## 📋 Daftar Isi

1. [Quick Start](#quick-start)
2. [Arsitektur Sistem](#arsitektur-sistem)
3. [Daftar Agen Lengkap](#daftar-agen-lengkap)
4. [Cara Menggunakan](#cara-menggunakan)
5. [Best Practices](#best-practices)
6. [Quality Gates](#quality-gates)
7. [Contoh Workflow](#contoh-workflow)
8. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Untuk User/Developer

1. **Ajukan request ke Orchestrator (Luna)**
   - Jelaskan kebutuhan Anda dalam Bahasa Indonesia
   - Sertakan konteks dan batasan
   - Tentukan prioritas dan deadline

2. **Orchestrator akan:**
   - Menganalisis request
   - Memecah menjadi tugas-tugas
   - Menunjuk agen yang tepat
   - Mengoordinasikan eksekusi

3. **Terima hasil yang sudah direview:**
   - Kode yang sudah ditest
   - Dokumentasi lengkap
   - Siap untuk production

---

## Arsitektur Sistem

```
┌─────────────────────────────────────────────────────────────┐
│                    USER (Anda)                              │
│              Ajukan request dalam Bahasa Indonesia          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              ORCHESTRATOR (Luna)                            │
│  • Analisis request                                         │
│  • Pecah menjadi tugas                                      │
│  • Pilih agen yang tepat                                    │
│  • Koordinasi eksekusi                                      │
│  • Review & integrasi hasil                                 │
│  • Serahkan hasil final                                     │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐    ┌─────────────────┐    ┌───────────────┐
│  IMPLEMENT    │    │    QUALITY      │    │  INFRA &    │
│  SPECIALISTS  │    │    GATEKEEPERS  │    │  DATA       │
├───────────────┤    ├─────────────────┤    ├───────────────┤
│ software-eng  │    │ code-reviewer   │    │ devops-eng    │
│ frontend-eng  │    │ quality-ctrl    │    │ database-eng  │
│ backend-eng   │    │ security-eng    │    │ data-eng      │
│ mobile-eng    │    │ bug-hunter-pro  │    │ ml-eng        │
│ api-eng       │    │ test-engineer   │    │ docs-writer   │
│               │    │ qa-engineer     │    │ ux-engineer   │
│               │    │ reliability-eng │    │ platform-eng  │
│               │    │ performance-eng │    │               │
└───────────────┘    └─────────────────┘    └───────────────┘
```

---

## Daftar Agen Lengkap

### 🎯 Orchestrator

| Agen | File | Tujuan |
|------|------|--------|
| **Orchestrator (Luna)** | `orchestrator.md` | Memimpin orkestrasi, memilih agen, review final |

### ✅ Quality & Review

| Agen | File | Tujuan | Kapan Digunakan |
|------|------|--------|-----------------|
| **Quality Controller** | `quality-controller.md` | Quality gate final | Sebelum merge, kode kritis |
| **Code Reviewer** | `code-reviewer.md` | Review kode | Setiap PR |
| **Security Engineer** | `security-engineer.md` | Security review | Auth, data sensitif, API |
| **Bug Hunter Pro** | `bug-hunter-pro.md` | Cari bug | Debugging, vulnerability scan |
| **Test Engineer** | `test-engineer.md` | Implementasi test | Menulis unit/integration/E2E tests |
| **QA Engineer** | `qa-engineer.md` | QA planning | Test strategy, manual testing |
| **Reliability Engineer** | `reliability-engineer.md` | SRE | Monitoring, incident response |
| **Performance Engineer** | `performance-engineer.md` | Performance | Profiling, optimization |

### 💻 Implementation Specialists

| Agen | File | Tujuan | Kapan Digunakan |
|------|------|--------|-----------------|
| **Software Engineer** | `software-engineer.md` | General coding | Feature, bug fix, refactoring |
| **Frontend Engineer** | `frontend-engineer.md` | Web UI | React/Vue, web apps |
| **Backend Engineer** | `backend-engineer.md` | Server-side | API, database, business logic |
| **Mobile Engineer** | `mobile-engineer.md` | Mobile apps | iOS, Android, React Native |
| **API Engineer** | `api-engineer.md` | API design | REST, GraphQL API |

### 🏗️ Architecture & Design

| Agen | File | Tujuan | Kapan Digunakan |
|------|------|--------|-----------------|
| **Architect Engineer** | `architect-engineer.md` | System design | Arsitektur baru, major decisions |
| **UX Engineer** | `ux-engineer.md` | User experience | UI/UX design, accessibility |
| **Platform Engineer** | `platform-engineer.md` | Developer platform | Internal tools, DX |

### 🚀 Infrastructure & Data

| Agen | File | Tujuan | Kapan Digunakan |
|------|------|--------|-----------------|
| **DevOps Engineer** | `devops-engineer.md` | Infrastructure | CI/CD, deployment, monitoring |
| **Database Engineer** | `database-engineer.md` | Database | Schema, queries, optimization |
| **Data Engineer** | `data-engineer.md` | Data pipelines | ETL, data warehouse |
| **ML Engineer** | `ml-engineer.md` | Machine learning | ML models, AI features |

### 📚 Documentation

| Agen | File | Tujuan | Kapan Digunakan |
|------|------|--------|-----------------|
| **Documentation Writer** | `documentation-writer.md` | Technical writing | README, API docs, guides |

---

## Cara Menggunakan

### 1. Ajukan Request ke Orchestrator

**Format Request:**
```
Saya butuh [fitur/sistem] untuk [tujuan].

Konteks:
[jelaskan konteks]

Requirements:
- [requirement 1]
- [requirement 2]

Batasan:
- [batasan teknis/budget/timeline]

Prioritas: [High/Medium/Low]
Deadline: [tanggal jika ada]
```

**Contoh:**
```
Saya butuh API untuk sistem manajemen user.

Konteks:
Kami sedang membangun aplikasi SaaS dan butuh user management.

Requirements:
- CRUD user
- Role-based access control
- Email verification
- Password reset

Batasan:
- Harus pakai PostgreSQL
- Harus ada rate limiting
- Deadline 2 minggu

Prioritas: High
```

### 2. Orchestrator Akan Merespon

**Respon Orchestrator:**
```markdown
## Ringkasan Eksekutif

Saya akan membantu membangun sistem user management. Berikut rencana eksekusinya:

### Yang Dikerjakan

1. **Architect Engineer** → Desain arsitektur API
2. **Backend Engineer** → Implementasi endpoints
3. **Database Engineer** → Desain schema database
4. **Security Engineer** → Review keamanan
5. **Test Engineer** → Menulis tests
6. **Documentation Writer** → Dokumentasi API

### Timeline

- Design: 1 hari
- Implementation: 3 hari
- Testing: 1 hari
- Documentation: 1 hari

Total: 6 hari

Mari kita mulai dengan architect engineer untuk desain...
```

### 3. Agen Bekerja

Orchestrator akan memberikan instruksi lengkap ke setiap agen:

```markdown
## Task: Design User Management API

### Goal
Design RESTful API untuk user management dengan RBAC.

### Acceptance Criteria
- [ ] Endpoint CRUD lengkap
- [ ] Role-based access control
- [ ] Rate limiting
- [ ] Error handling komprehensif

### Technical Standards
- Follow REST conventions
- OpenAPI documentation
- Security best practices
```

### 4. Review & Integration

Setiap hasil agen akan direview:

```
Implementation → Code Review → Testing → Quality Control → Final
```

---

## Best Practices

### ✅ DO (Lakukan)

1. **Berikan konteks lengkap**
   - Jelaskan business requirements
   - Sertakan batasan teknis
   - Informasikan prioritas

2. **Gunakan quality gates**
   - Jangan skip code review
   - Selalu jalankan tests
   - Dapatkan approval quality controller

3. **Dokumentasikan technical debt**
   - Jika ada shortcut, dokumentasikan
   - Buat plan untuk payoff
   - Track dalam issue tracker

4. **Test coverage minimal 80%**
   - Untuk kode kritis: 100%
   - Unit tests untuk semua functions
   - Integration tests untuk APIs

5. **Review security untuk:**
   - Authentication/authorization
   - Input validation
   - Data sensitif
   - External APIs

### ❌ DON'T (Jangan Lakukan)

1. **Jangan skip review**
   - Tidak ada merge tanpa review
   - Tidak ada shortcut security

2. **Jangan hardcode credentials**
   - Gunakan environment variables
   - Gunakan secret management

3. **Jangan write code tanpa tests**
   - Tests dulu (TDD) atau bersamaan
   - Jangan commit tanpa tests

4. **Jangan ignore technical debt**
   - Dokumentasikan
   - Prioritaskan payoff
   - Jangan biarkan menumpuk

---

## Quality Gates

Setiap kode HARUS melewati quality gates ini:

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ Development  │ →  │ Code Review  │ →  │   Testing    │ →  │  Quality     │
│   (Agent)    │    │  (Reviewer)  │    │  (Engineer)  │    │   Control    │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                                                                    │
                                                                    ▼
                                                           ┌──────────────┐
                                                           │    Merge     │
                                                           │   Approved   │
                                                           └──────────────┘
```

### Gate 1: Development

**Oleh:** Implementation Agent (software-engineer, backend-engineer, dll)

**Checklist:**
- [ ] Kode mengikuti best practices
- [ ] Error handling komprehensif
- [ ] Input validation implemented
- [ ] Initial tests written

### Gate 2: Code Review

**Oleh:** code-reviewer

**Checklist:**
- [ ] Logic benar
- [ ] Code style konsisten
- [ ] Tidak ada duplication
- [ ] Functions focused (SRP)
- [ ] Naming descriptive

### Gate 3: Testing

**Oleh:** test-engineer

**Checklist:**
- [ ] Unit tests mencakup semua paths
- [ ] Integration tests untuk APIs
- [ ] Coverage >= 80% (critical: 100%)
- [ ] Semua tests passing

### Gate 4: Quality Control

**Oleh:** quality-controller

**Checklist:**
- [ ] Semua gates sebelumnya passed
- [ ] Security reviewed
- [ ] Documentation complete
- [ ] Technical debt documented
- [ ] **APPROVED FOR MERGE**

---

## Contoh Workflow

### Workflow 1: New Feature

```
User Request: "Saya butuh fitur export data ke CSV"

1. Orchestrator Analysis
   - Break down: backend export, frontend UI, tests, docs
   - Select agents: backend-eng, frontend-eng, test-eng, docs-writer

2. Backend Engineer
   - Create export endpoint
   - Implement streaming untuk large data
   - Add rate limiting

3. Frontend Engineer
   - Create export button
   - Handle download progress
   - Error handling

4. Test Engineer
   - Unit tests untuk export logic
   - Integration tests untuk endpoint
   - E2E test untuk user flow

5. Code Reviewer
   - Review semua kode
   - Request changes jika perlu

6. Quality Controller
   - Final review
   - Approve untuk merge

7. Documentation Writer
   - Update API docs
   - Add user guide
```

### Workflow 2: Bug Fix

```
User Report: "Export CSV timeout untuk data besar"

1. Bug Hunter Pro
   - Analyze logs
   - Identify root cause: tidak ada streaming
   - Reproduce bug

2. Backend Engineer
   - Implement streaming response
   - Add pagination untuk large data
   - Optimize query

3. Test Engineer
   - Add regression test
   - Test dengan large dataset
   - Performance test

4. Code Reviewer
   - Review fix
   - Verify tidak ada side effects

5. Quality Controller
   - Approve hotfix
```

### Workflow 3: Security Audit

```
User Request: "Audit security sebelum launch"

1. Security Engineer
   - Threat modeling
   - Security architecture review
   - Code review untuk vulnerabilities

2. Bug Hunter Pro
   - Vulnerability scanning
   - Penetration testing
   - Fuzzing

3. Backend Engineer
   - Fix vulnerabilities yang ditemukan
   - Implement security recommendations

4. Security Engineer
   - Verify fixes
   - Re-test

5. Quality Controller
   - Final security approval
```

---

## Troubleshooting

### Masalah: "Agen tidak merespon"

**Solusi:**
1. Periksa instruksi - apakah cukup jelas?
2. Periksa acceptance criteria - apakah measurable?
3. Re-delegate dengan instruksi lebih detail
4. Escalate ke orchestrator

### Masalah: "Kode tidak meet standards"

**Solusi:**
1. Kembalikan ke agen dengan feedback spesifik
2. Sertakan contoh perbaikan
3. Set deadline untuk revision
4. Re-review setelah fix

### Masalah: "Technical debt menumpuk"

**Solusi:**
1. Dokumentasikan semua debt dalam tracker
2. Prioritaskan payoff plan
3. Alokasikan waktu di setiap sprint
4. Quality controller monitor debt level

### Masalah: "Tests failing di CI"

**Solusi:**
1. Periksa test output
2. Reproduce locally
3. Fix test atau code
4. Re-run CI
5. Jika flaky, quarantine dan fix

---

## Contact & Support

### Documentation
- `INDEX.md` - Index semua agen
- `orchestrator.md` - Orchestrator prompt
- `[agent-name].md` - Prompt untuk setiap agen

### Escalation Path
1. Orchestrator (Luna) - First point of contact
2. Quality Controller - Quality issues
3. Architect Engineer - Technical decisions
4. Security Engineer - Security issues

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-02-18 | Initial release |

---

*Last updated: 2024-02-18*

*Untuk pertanyaan atau feedback, hubungi Orchestrator (Luna).*
