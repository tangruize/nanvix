# Review: zombie_process Exec Consistency (claude-opus-4.6)

## Grade: A

## Overview

Reviewed the exec consistency fixes for `zombie_process` by comparing the original
source (`src/kernel/src/pm/process/state/zombie.rs`) against the verified Verus
split (`verus/split/kernel/pm/process/state/zombie.rs`, `zombie.spec.rs`,
`zombie.proof.rs`), guided by the consistency fix report
(`zombie_process_20260214_155659_fix.md`).

## Verification Result

**PASS** — 25 verified, 0 errors. No `assume` or `admit` used. 4 justified
`external_body` annotations.

## Criteria Assessment

### 1. Were all MISMATCH functions properly restored or equivalence documented?

**Yes.** The report identifies 7 documented equivalences (1 struct + 6 functions).
All 7 are type-level abstractions — no executable logic was changed. Each
equivalence is documented inline in the exec code with `## AST Equivalence`
sections that show original vs. Verus signatures side-by-side. The mapping is:

| Original | Verus | Type Abstraction |
|----------|-------|-----------------|
| `ZombieProcess { zombie_threads, process, status }` | `ZombieProcess { pid, zombie_thread_ids, status, zombie_count }` | `NonEmptyVecDeque<ZombieThread>` → `Vec<u64>` + `zombie_count`, `Box<ProcessState>` → `u64`, `ExitStatus` → `i64` |
| `new(Box<ProcessState>, NonEmptyVecDeque<ZombieThread>, ExitStatus)` | `new(u64, Vec<u64>, i64, u64)` | Same field assignment, added `zombie_count` |
| `state(&self) -> &ProcessState` | `state(&self) -> u64` (external_body) | Cannot model reference return |
| `state_mut(&mut self) -> &mut ProcessState` | `state_mut(&mut self) -> u64` (external_body) | Cannot model mutable reference return |
| `bury(self) -> (NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus)` | `bury(self) -> (Vec<u64>, u64, i64)` | Direct destructuring, same order |
| `find_thread(&self, ThreadIdentifier) -> Option<ThreadRef<'_>>` | `find_thread(&self, u64) -> Ghost<Option<u64>>` (external_body) | Cannot model lifetime-carrying enum |
| `find_thread_mut(&mut self, ThreadIdentifier) -> Option<ThreadRefMut<'_>>` | `find_thread_mut(&mut self, u64) -> Ghost<Option<u64>>` (external_body) | Cannot model lifetime-carrying mutable enum |

All type abstractions are necessary and well-justified by Verus limitations.

### 2. Were MISSING functions added with proper verification?

**N/A.** The fix report states 0 missing functions were added. All 6 original
functions are represented in the Verus model (either with concrete bodies or
`external_body` with spec contracts). This is correct — the original has exactly
`new`, `state`, `state_mut`, `bury`, `find_thread`, `find_thread_mut`, and all
are present.

### 3. Are equivalence justifications sound?

**Yes, with one minor observation.** The justifications are thorough and
technically accurate:

- **`new`**: Logic-identical field assignment. The extra `zombie_count` parameter
  is justified because `Vec<u64>` lacks `NonEmptyVecDeque`'s type-level non-empty
  guarantee. Preconditions enforce `zombie_count as nat == zombie_ids@.len()` and
  `zombie_ids@.len() >= 1`, correctly modeling the non-empty invariant.

- **`bury`**: Both destructure `self` and return a 3-tuple. Field order matches:
  (threads/thread_ids, process/pid, status). Verified postconditions ensure each
  component matches the abstract view.

- **`state` / `state_mut`**: `external_body` with postconditions. The PID-only
  abstraction is explicitly documented as a deliberate scope limitation. The
  cross-module PID immutability argument for `state_mut` is detailed and references
  specific verified mutators in `process_state.rs`.

- **`find_thread` / `find_thread_mut`**: `external_body` with postconditions
  encoding search semantics. Integration obligations are well-decomposed into
  three parts: overall result match, per-element predicate equivalence, and
  caller discipline for mutable access.

**Minor observation**: The `find_thread` postcondition returns `Some(0u64)` as a
tag value indicating "found in zombie list." This is a reasonable encoding since
`ZombieProcess` only has one thread list (unlike `RunningProcess` which has
running + zombie lists). The value `0` is used as a discriminant rather than a
thread ID, which could be slightly confusing but is clearly documented.

### 4. Does the exec code now faithfully represent the original source?

**Yes.** Function-by-function comparison:

- **`new`**: Original performs `Self { zombie_threads, process, status }`. Verus
  performs `ZombieProcess { pid, zombie_thread_ids: zombie_ids, status, zombie_count }`.
  Both are direct field assignment from parameters. ✓

- **`state`**: Original returns `&self.process`. Verus is `external_body` returning
  `u64` with postcondition `result as int == self@.pid`. Faithful identity-level
  abstraction. ✓

- **`state_mut`**: Original returns `&mut self.process`. Verus is `external_body`
  returning `u64` with frame condition `self@ == old(self)@`. The frame condition
  is correct at the ZombieProcess level — internal ProcessState mutations are
  invisible at this abstraction layer. ✓

- **`bury`**: Original returns `(self.zombie_threads, self.process, self.status)`.
  Verus returns `(self.zombie_thread_ids, self.pid, self.status)`. Same
  destructuring pattern, same field order. ✓

- **`find_thread`**: Original uses `self.zombie_threads.iter().find(|t| t.id() == tid).map(ThreadRef::Zombie)`.
  Verus `external_body` postcondition: `Some(0)` iff `spec_has_zombie_thread(tid)`,
  `None` otherwise. This correctly captures the search semantics. ✓

- **`find_thread_mut`**: Original uses `self.zombie_threads.iter_mut().find(|t| t.id() == tid).map(ThreadRefMut::Zombie)`.
  Verus `external_body` postcondition mirrors `find_thread` with added frame
  condition `self@ == old(self)@`. This correctly models that `find_thread_mut`
  only returns a reference — actual mutation happens through that reference later,
  which is covered by the caller obligation. ✓

### 5. Does verification still pass?

**Yes.** 25 verified, 0 errors. All proofs pass without `assume` or `admit`.

## Issues Found

### Critical

None.

### Major

None.

### Minor

1. **`spec_has_zombie_thread` type mismatch between exec and spec levels** (cosmetic,
   not a correctness issue): The exec-level `spec_has_zombie_thread` on
   `ZombieProcess` takes `u64` (line 134 of `zombie.spec.rs`), while the
   view-level `spec_has_zombie_thread` on `ZombieProcessView` takes `int`
   (line 437). This is correct by design (exec operates on `u64`, spec on `int`),
   and the bridging lemma `lemma_has_zombie_thread_matches_view` (line 392 of
   `zombie.proof.rs`) proves equivalence. No action needed.

2. **`zombie_count` field not returned by `bury()`**: The original `bury()`
   consumes `self` entirely. The Verus `bury()` returns `(Vec<u64>, u64, i64)`,
   dropping `zombie_count`. This is correct because `zombie_count` is a
   verification-only artifact (tracking the Vec length), not an original field.
   The postcondition `result.0@.len() >= 1` preserves the non-empty guarantee.

### Observations

- The proof file is well-structured with clear sections and 13 lemmas covering
  construction, decomposition, search, integration obligations, and view bridging.
- The trust boundary documentation in the module header is exceptionally detailed,
  listing every public mutator of `ProcessState` and the cross-module verification
  evidence for PID immutability.
- The 4 `external_body` functions are all justified by genuine Verus type system
  limitations (reference returns, lifetime-carrying enums), not by convenience.
- Integration obligations are formalized as spec functions rather than comments,
  making them machine-checkable when integration proofs are written.

## Summary

The exec consistency fixes for `zombie_process` are thorough and correct. All 6
original functions are faithfully represented in the Verus model with appropriate
type abstractions. The 4 `external_body` annotations are justified by genuine
Verus limitations, and their postconditions correctly encode the original
semantics. The equivalence documentation is detailed with inline AST comparisons.
Verification passes cleanly with 25 verified, 0 errors, no `assume`/`admit`.
The fix report accurately describes the changes as "documented equivalences" with
no executable logic modifications. Grade: **A** — minor cosmetic observations
only, no substantive issues.
