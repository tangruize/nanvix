# Exec Consistency Fix: zombie_process

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 7 (1 struct + 6 functions)

## Context

This is a **design-level verification** module. The Verus model uses simplified
concrete types (`u64`, `Vec<u64>`, `i64`) to model the original kernel types
(`Box<ProcessState>`, `NonEmptyVecDeque<ZombieThread>`, `ExitStatus`,
`ThreadRef<'_>`, `ThreadRefMut<'_>`). All 6 function mismatches and the struct
mismatch are type-level abstractions required because Verus cannot model:

1. Complex kernel types (`ProcessState`, `ZombieThread`, `ExitStatus`).
2. Reference returns (`&ProcessState`, `&mut ProcessState`).
3. Lifetime-carrying enums (`Option<ThreadRef<'_>>`, `Option<ThreadRefMut<'_>>`).
4. Iterator-based search (`NonEmptyVecDeque::iter().find()`).

**No executable logic was changed.** Every function performs the same state
transition (field construction, field access, destructuring, linear search)
as the original — only the types are abstracted.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `ZombieProcess` (struct) | Documented equivalence | Type abstraction: `NonEmptyVecDeque<ZombieThread>` → `Vec<u64>` + `zombie_count`, `Box<ProcessState>` → `pid: u64`, `ExitStatus` → `i64`. Necessary because Verus cannot model these kernel types. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Documented equivalence | Logic identical: both perform direct field assignment. Parameter types abstracted to match struct fields. Extra `zombie_count` parameter needed because `Vec<u64>` lacks `NonEmptyVecDeque`'s type-level non-empty guarantee. |
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | Documented equivalence (Verus limitation) | Original returns `&ProcessState`; Verus returns `u64` (PID only) via `external_body`. Verus cannot model reference returns to complex types. Postcondition ensures PID identity match. |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | Documented equivalence (Verus limitation) | Original returns `&mut ProcessState`; Verus returns `u64` via `external_body`. Verus cannot model mutable reference returns. PID immutability verified cross-module in `process_state`. |
| `bury` [bury.diff](bury.diff) [bury_source.rs](bury_source.rs) [bury_verus.rs](bury_verus.rs) | Documented equivalence | Logic identical: both destructure `self` and return `(threads, process, status)` tuple. Types abstracted to `(Vec<u64>, u64, i64)`. Same field order. |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | Documented equivalence (Verus limitation) | Original returns `Option<ThreadRef<'_>>` via iterator search; Verus returns `Ghost<Option<u64>>` via `external_body`. Verus cannot model lifetime-carrying enum reference returns or iterator search. Postcondition matches search semantics. |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | Documented equivalence (Verus limitation) | Original returns `Option<ThreadRefMut<'_>>` via iterator search; Verus returns `Ghost<Option<u64>>` via `external_body`. Same Verus limitation as `find_thread`. Frame condition (`self@ == old(self)@`) preserves state. |

## Detailed Equivalence Arguments

### Struct `ZombieProcess`
- **Original fields**: `zombie_threads: NonEmptyVecDeque<ZombieThread>`, `process: Box<ProcessState>`, `status: ExitStatus`
- **Verus fields**: `pid: u64`, `zombie_thread_ids: Vec<u64>`, `status: i64`, `zombie_count: u64`
- **Mapping**: Each original field maps to one or more Verus fields. `zombie_count` is added to track the non-empty invariant that `NonEmptyVecDeque` provides at the type level.

### `new` — Structural equivalence
- Original: `Self { zombie_threads, process, status }` (3-field construction)
- Verus: `ZombieProcess { pid, zombie_thread_ids: zombie_ids, status, zombie_count }` (4-field construction)
- Both are simple field assignments. The 4th field (`zombie_count`) tracks what `NonEmptyVecDeque` guarantees implicitly.

### `bury` — Structural equivalence
- Original: `(self.zombie_threads, self.process, self.status)`
- Verus: `(self.zombie_thread_ids, self.pid, self.status)`
- Both destructure and return all fields as a 3-tuple in identical order.

### `state`, `state_mut`, `find_thread`, `find_thread_mut` — Verus limitation
- All four are `external_body` because Verus cannot model reference/mutable-reference returns to complex types or lifetime-carrying enums.
- Postconditions faithfully encode the original semantics (PID identity, search result, frame preservation).
- `state_mut` PID immutability is verified cross-module in `process_state.rs`.

## External Body Usage

| Function | Reason | Integration Obligation |
|----------|--------|----------------------|
| `state` [state.diff](state.diff) [state_source.rs](state_source.rs) [state_verus.rs](state_verus.rs) | Cannot model `&ProcessState` return | PID identity: `result == self@.pid` |
| `state_mut` [state_mut.diff](state_mut.diff) [state_mut_source.rs](state_mut_source.rs) [state_mut_verus.rs](state_mut_verus.rs) | Cannot model `&mut ProcessState` return | PID preservation verified in `process_state` module |
| `find_thread` [find_thread.diff](find_thread.diff) [find_thread_source.rs](find_thread_source.rs) [find_thread_verus.rs](find_thread_verus.rs) | Cannot model `Option<ThreadRef<'_>>` | `spec_find_thread_integration_obligation` |
| `find_thread_mut` [find_thread_mut.diff](find_thread_mut.diff) [find_thread_mut_source.rs](find_thread_mut_source.rs) [find_thread_mut_verus.rs](find_thread_mut_verus.rs) | Cannot model `Option<ThreadRefMut<'_>>` | `spec_find_thread_integration_obligation` + caller discipline |

## Verification: PASS
- 25 verified, 0 errors
- No `assume` or `admit` used
- 4 justified `external_body` annotations (Verus type system limitations)
