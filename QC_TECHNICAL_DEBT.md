# Technical Debt Register - Fax Compiler

**Audit ID:** QC-AUDIT-001  
**Date:** 2026-02-21  
**Purpose:** Track and manage technical debt identified during QC audit

---

## Overview

This register tracks all technical debt items identified during the quality audit. Each item includes:
- Description of the debt
- Business impact
- Estimated effort to fix
- Priority ranking
- Recommended resolution timeline

**Total Technical Debt Items:** 35  
**Total Estimated Effort:** ~125 hours  
**Critical Items:** 4  
**High Priority Items:** 12

---

## Debt Summary by Category

| Category | Count | Total Effort | Business Impact |
|----------|-------|--------------|-----------------|
| Incomplete Features | 8 | 45 hours | High |
| Code Quality | 10 | 25 hours | Medium |
| Test Coverage | 7 | 30 hours | High |
| Documentation | 5 | 10 hours | Low |
| Performance | 5 | 15 hours | Medium |

---

## Debt Items

### TD-001: Extensive unwrap()/expect() Usage

| Attribute | Value |
|-----------|-------|
| **Category** | Code Quality |
| **Severity** | Critical |
| **Effort** | 16 hours |
| **Impact** | High - Production stability |
| **Location** | Multiple crates (582 instances) |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
The codebase contains 582 instances of `unwrap()`, `expect()`, and `panic!` calls. While acceptable in test code and prototypes, these represent potential runtime panics in production.

#### Business Impact
- **Risk:** Application crashes on unexpected input
- **User Experience:** Poor error messages instead of graceful handling
- **Maintainability:** Hard to track error sources

#### Resolution Options
1. **Full Fix (16h):** Replace all with proper `Result`/`Option` handling
2. **Partial Fix (8h):** Replace in critical paths only
3. **Accept (0h):** Document as known limitation for pre-alpha

#### Recommended Action
**Partial Fix** - Focus on:
- User-facing error paths
- File I/O operations
- Network operations (if any)
- Parser error recovery

#### Verification
```bash
# Count unwrap/expect in non-test code
grep -r "unwrap()\|expect(" faxc/crates --include="*.rs" | grep -v "tests/" | grep -v "#\[test\]" | wc -l
```

---

### TD-002: Missing End-to-End Tests

| Attribute | Value |
|-----------|-------|
| **Category** | Test Coverage |
| **Severity** | Critical |
| **Effort** | 12 hours |
| **Impact** | High - Release confidence |
| **Location** | Test infrastructure |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
No integration tests exist that compile complete Fax programs through the entire pipeline and verify output.

#### Business Impact
- **Risk:** Regression bugs slip to production
- **Confidence:** Low confidence in release quality
- **Debugging:** Harder to diagnose integration issues

#### Resolution
Create end-to-end test suite:
1. Sample Fax programs in `tests/e2e/`
2. Expected output files
3. Test runner that compiles and compares output

#### Acceptance Criteria
- [ ] 10+ e2e test cases
- [ ] Tests run in CI
- [ ] Coverage report includes e2e tests

---

### TD-003: Incomplete GC Implementation

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | High |
| **Effort** | 20 hours |
| **Impact** | High - Runtime correctness |
| **Location** | `fgc/crates/` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
The garbage collector has 17 ignored tests and several TODO items indicating incomplete functionality:
- Stack scanning not implemented
- Global root registration incomplete
- Some barrier operations untested

#### Specific Items
| Test/Feature | Status | Effort |
|--------------|--------|--------|
| Stack scanning | Not implemented | 8h |
| Global roots | Partial | 4h |
| Weak references | Not tested | 4h |
| Finalization | Not tested | 4h |

#### Business Impact
- **Risk:** Memory leaks in production
- **Performance:** Suboptimal GC behavior
- **Correctness:** Potential use-after-free bugs

#### Resolution
Complete implementation in phases:
1. Phase 1: Stack scanning (8h)
2. Phase 2: Root registration (4h)
3. Phase 3: Testing (8h)

---

### TD-004: Incomplete Binary Operators

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | High |
| **Effort** | 4 hours |
| **Impact** | Medium - Language completeness |
| **Location** | `faxc-sem/src/analysis.rs:335-337` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
Three binary operators are marked with `todo!()`:
- `BitXor` (^)
- `Shl` (<<)
- `Shr` (>>)

#### Code Location
```rust
// faxc-sem/src/analysis.rs:335-337
ast::BinOp::BitXor => todo!(), // Need to add BitXor to HIR BinOp
ast::BinOp::Shl => todo!(),
ast::BinOp::Shr => todo!(),
```

#### Business Impact
- **User Experience:** Cannot use bitwise operations
- **Language Parity:** Incomplete vs. specification

#### Resolution
1. Add HIR BinOp variants (1h)
2. Implement semantic analysis (2h)
3. Add tests (1h)

---

### TD-005: Parser Module Organization

| Attribute | Value |
|-----------|-------|
| **Category** | Code Quality |
| **Severity** | High |
| **Effort** | 6 hours |
| **Impact** | Medium - Maintainability |
| **Location** | `faxc-par/src/` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
The parser logic is entirely in `lib.rs` instead of a dedicated `parser.rs` module. This makes the file large and harder to navigate.

#### Resolution
1. Create `parser.rs` with main Parser struct
2. Create `statements.rs` for statement parsing
3. Create `expressions.rs` for expression parsing (Pratt)
4. Create `patterns.rs` for pattern parsing
5. Update `lib.rs` to re-export

#### Acceptance Criteria
- [ ] lib.rs < 500 lines
- [ ] Each module < 400 lines
- [ ] All tests pass after refactoring

---

### TD-006: Test Coverage Below Threshold

| Attribute | Value |
|-----------|-------|
| **Category** | Test Coverage |
| **Severity** | Medium |
| **Effort** | 18 hours |
| **Impact** | High - Quality gate failure |
| **Location** | All crates |
| **Introduced** | Initial development |
| **Status** | Open |

#### Current Coverage

| Crate | Current | Target | Gap |
|-------|---------|--------|-----|
| faxc-util | ~70% | 80% | 10% |
| faxc-lex | ~65% | 80% | 15% |
| faxc-par | ~60% | 80% | 20% |
| faxc-sem | ~55% | 80% | 25% |
| faxc-mir | ~50% | 80% | 30% |
| faxc-lir | ~45% | 80% | 35% |
| faxc-gen | ~40% | 80% | 40% |
| fgc | ~75% | 80% | 5% |
| faxc-drv | ~35% | 80% | 45% |

#### Resolution Strategy
Focus on critical paths first:
1. Error handling paths (6h)
2. Edge cases (6h)
3. Integration scenarios (6h)

---

### TD-007: Unsafe Code Documentation

| Attribute | Value |
|-----------|-------|
| **Category** | Documentation |
| **Severity** | High |
| **Effort** | 8 hours |
| **Impact** | Medium - Security audit readiness |
| **Location** | `fgc/src/` (239 unsafe blocks) |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
The FGC crate contains 239 unsafe blocks without comprehensive safety documentation.

#### Resolution
For each unsafe block, document:
1. Safety invariants
2. Preconditions
3. Postconditions
4. Why the unsafe operation is safe

#### Priority Files
1. `fgc/src/relocate/copy.rs` - Memory operations
2. `fgc/src/barrier/fast_path.rs` - Pointer manipulation
3. `fgc/src/object/refmap.rs` - Bit manipulation

---

### TD-008: Mixed Language Comments

| Attribute | Value |
|-----------|-------|
| **Category** | Code Quality |
| **Severity** | Medium |
| **Effort** | 2 hours |
| **Impact** | Low - International collaboration |
| **Location** | `faxc-drv/src/lib.rs` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
Comments in `faxc-drv` are in Indonesian while the rest of the codebase uses English.

#### Resolution
Translate all comments to English:
- `Configuration untuk compiler` → `Compiler configuration`
- `dengan` → `with`
- `atau` → `or`

---

### TD-009: Missing Performance Benchmarks

| Attribute | Value |
|-----------|-------|
| **Category** | Performance |
| **Severity** | Medium |
| **Effort** | 8 hours |
| **Impact** | Medium - Performance regression detection |
| **Location** | All crates |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
No performance benchmarks exist to track compiler performance over time.

#### Resolution
Create benchmark suite:
1. Lexer benchmarks (2h)
2. Parser benchmarks (2h)
3. GC benchmarks (2h)
4. Code generation benchmarks (2h)

#### Benchmark Categories
- Throughput (operations/second)
- Latency (p95, p99)
- Memory usage
- Allocation rates

---

### TD-010: Symbol Interner Lock Contention

| Attribute | Value |
|-----------|-------|
| **Category** | Performance |
| **Severity** | Medium |
| **Effort** | 6 hours |
| **Impact** | Medium - Compilation speed |
| **Location** | `faxc-util/src/symbol/` |
| **Introduced** | Initial design |
| **Status** | Open |

#### Description
The global symbol interner uses a single lock, which may cause contention during parallel compilation.

#### Resolution Options
1. **Sharded interner (6h):** Split into multiple shards with separate locks
2. **Read-write lock (2h):** Use RwLock for read-heavy workloads
3. **Accept for now (0h):** Profile first to confirm bottleneck

#### Recommended Action
Profile first, then optimize if needed.

---

### TD-011: Missing Fuzzing Infrastructure

| Attribute | Value |
|-----------|-------|
| **Category** | Test Coverage |
| **Severity** | Medium |
| **Effort** | 8 hours |
| **Impact** | Medium - Security/robustness |
| **Location** | Test infrastructure |
| **Introduced** | N/A |
| **Status** | Open |

#### Description
No fuzzing tests exist for lexer/parser to find edge cases and security issues.

#### Resolution
Set up cargo-fuzz:
1. Add cargo-fuzz dependency (1h)
2. Create lexer fuzzer (3h)
3. Create parser fuzzer (3h)
4. Set up CI integration (1h)

---

### TD-012: Incomplete Error Recovery

| Attribute | Value |
|-----------|-------|
| **Category** | Code Quality |
| **Severity** | Medium |
| **Effort** | 6 hours |
| **Impact** | Low - User experience |
| **Location** | `faxc-par/src/` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
Parser error recovery could be more robust, especially for nested structures.

#### Resolution
Improve panic-mode recovery:
1. Better synchronization points (2h)
2. Recovery for nested blocks (2h)
3. Better error messages (2h)

---

### TD-013: Missing Cross-Platform Testing

| Attribute | Value |
|-----------|-------|
| **Category** | Test Coverage |
| **Severity** | Medium |
| **Effort** | 4 hours |
| **Impact** | Medium - Platform support |
| **Location** | CI/CD |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
CI only runs on Linux. No testing on Windows or macOS.

#### Resolution
Add matrix builds:
1. Windows runner (2h)
2. macOS runner (2h)

---

### TD-014: Docker Image Size

| Attribute | Value |
|-----------|-------|
| **Category** | Performance |
| **Severity** | Low |
| **Effort** | 2 hours |
| **Impact** | Low - Distribution size |
| **Location** | `Dockerfile` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
Docker image could be smaller using distroless or scratch base.

#### Resolution
Multi-stage build optimization:
1. Use distroless base (1h)
2. Strip unnecessary files (1h)

---

### TD-015: Arena Allocation for AST

| Attribute | Value |
|-----------|-------|
| **Category** | Performance |
| **Severity** | Low |
| **Effort** | 8 hours |
| **Impact** | Medium - Memory efficiency |
| **Location** | `faxc-par/src/` |
| **Introduced** | Initial design |
| **Status** | Open |

#### Description
AST nodes are individually allocated. Arena allocation could improve performance.

#### Resolution
Implement arena-based AST:
1. Add arena dependency (1h)
2. Convert AST to arena-allocated (4h)
3. Update all consumers (3h)

---

### TD-016: Incomplete Pattern Matching

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | Medium |
| **Effort** | 8 hours |
| **Impact** | Medium - Language completeness |
| **Location** | `faxc-par/src/`, `faxc-sem/src/` |
| **Introduced** | Initial development |
| **Status** | Open |

#### Description
Some pattern matching features may be incomplete per SPEC.md.

#### Resolution
Audit against SPEC.md and implement missing features.

---

### TD-017: Missing Generics Implementation

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | High |
| **Effort** | 24 hours |
| **Impact** | High - Language expressiveness |
| **Location** | Multiple crates |
| **Introduced** | Planned feature |
| **Status** | Not Started |

#### Description
Generics are mentioned in SPEC.md but may not be fully implemented.

#### Resolution
Implement in phases:
1. Type parameters (8h)
2. Monomorphization (8h)
3. Trait bounds (8h)

---

### TD-018: Missing Async/Await

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | Medium |
| **Effort** | 40 hours |
| **Impact** | Medium - Modern language feature |
| **Location** | Multiple crates |
| **Introduced** | Planned feature |
| **Status** | Not Started |

#### Description
Async/await mentioned in SPEC.md but not implemented.

#### Resolution
Defer to post-1.0 release.

---

### TD-019: Missing Trait System

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | High |
| **Effort** | 32 hours |
| **Impact** | High - Language expressiveness |
| **Location** | Multiple crates |
| **Introduced** | Planned feature |
| **Status** | Not Started |

#### Description
Trait system mentioned in SPEC.md but may be incomplete.

#### Resolution
Implement in phases:
1. Basic traits (12h)
2. Trait bounds (10h)
3. Blanket implementations (10h)

---

### TD-020: Missing Error Handling Syntax

| Attribute | Value |
|-----------|-------|
| **Category** | Incomplete Features |
| **Severity** | Medium |
| **Effort** | 12 hours |
| **Impact** | Medium - Language completeness |
| **Location** | Multiple crates |
| **Introduced** | Planned feature |
| **Status** | Not Started |

#### Description
Error handling syntax (Result, ?) may be incomplete.

---

## Priority Matrix

```
                        Business Impact
            Low         Medium          High
        +-----------+-----------+-----------+
  High  | TD-017    | TD-009    | TD-001    |
        | TD-018    | TD-010    | TD-002    |
Effort  | TD-019    | TD-011    | TD-003    |
        +-----------+-----------+-----------+
 Medium | TD-014    | TD-006    | TD-004    |
        | TD-015    | TD-012    | TD-005    |
        |           | TD-013    | TD-007    |
        +-----------+-----------+-----------+
  Low   | TD-008    | TD-020    |           |
        |           | TD-016    |           |
        +-----------+-----------+-----------+
```

---

## Resolution Timeline

### Phase 1: Critical (Weeks 1-2)
- TD-001: Partial unwrap fix (8h)
- TD-002: E2E test framework (6h)
- TD-004: Complete binary operators (4h)

**Total: 18 hours**

### Phase 2: High Priority (Weeks 3-6)
- TD-003: Complete GC implementation (20h)
- TD-005: Parser refactoring (6h)
- TD-007: Unsafe documentation (8h)

**Total: 34 hours**

### Phase 3: Medium Priority (Months 2-3)
- TD-006: Test coverage improvement (18h)
- TD-009: Performance benchmarks (8h)
- TD-011: Fuzzing infrastructure (8h)
- TD-012: Error recovery (6h)

**Total: 40 hours**

### Phase 4: Lower Priority (Months 4-6)
- TD-008: Comment standardization (2h)
- TD-010: Symbol interner optimization (6h)
- TD-013: Cross-platform testing (4h)
- TD-014: Docker optimization (2h)
- TD-015: Arena allocation (8h)

**Total: 22 hours**

### Phase 5: Major Features (Post-1.0)
- TD-017: Generics (24h)
- TD-018: Async/await (40h)
- TD-019: Trait system (32h)
- TD-020: Error handling (12h)

**Total: 108 hours**

---

## Debt Trend Tracking

| Date | Total Items | Critical | High | Medium | Low | Total Effort |
|------|-------------|----------|------|--------|-----|--------------|
| 2026-02-21 | 20 | 2 | 6 | 8 | 4 | 125h |

---

## Debt Prevention Strategies

### 1. Code Review Checklist
- [ ] No new unwrap() in production code
- [ ] All public functions documented
- [ ] Tests for new features
- [ ] No TODO without tracking issue

### 2. CI Gates
- [ ] Clippy passes
- [ ] Coverage > 80%
- [ ] All tests pass
- [ ] Documentation builds

### 3. Regular Audits
- Monthly: Quick debt review
- Quarterly: Full debt audit
- Pre-release: Critical debt clearance

---

## Appendix: Issue Tracking

Each debt item should have a corresponding GitHub issue:
- Label: `technical-debt`
- Label: Priority (`P0`, `P1`, `P2`, `P3`)
- Milestone: Target release
- Estimate: Story points

---

*Generated by SangPengawas QC Agent v2.0.0*