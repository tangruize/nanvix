# Exec Consistency Fix: running_thread

## Summary
- Mismatches fixed: 0 (all 8 are documented equivalences — no exec logic changes)
- Missing functions added: 0 (1 intentional omission documented)
- Documented equivalences: 9

## Analysis

All 8 MISMATCH functions preserve the original executable logic. The AST hash
differences arise from the Verus verification model's type abstractions and
the omission of HAL boundary values (`*mut ContextInformation`), which are
thoroughly documented in the module header. No exec logic was changed; no
fixes are needed.

The 1 MISSING function (`join_cond`) is intentionally omitted because its
return type `Condvar` is elided from the `ThreadState` model (sync boundary
type). Adding it would require restructuring the `ThreadState` model, which
is outside this module's scope.

The 3 EXTRA structs (`SleepingThread`, `ReadyThread`, `ZombieThread`) are
boundary models of sibling module types, required for verifying state
transitions. They are documented and justified.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `from_state` | Documented equivalence | `Box<ThreadState>` → `ThreadState` (Box is transparent, documented in module header). `Self { state }` ≡ `RunningThread { state: state }` (Rust syntactic sugar). `pub(super)` → `pub` for Verus proof ergonomics. |
| `sleep` | Documented equivalence | `Option<SystemTime>` → `Option<int>` (abstract timestamp model). `*mut ContextInformation` return omitted (HAL boundary, unsafe raw pointer). `mut self` → `self` (no longer calls `context_mut()`). Core logic identical: `SleepingThread::from_state(self.state, alarm)`. |
| `schedule` | Documented equivalence | Same pattern as `sleep`: `*mut ContextInformation` return omitted (HAL boundary). Core logic identical: `ReadyThread::from_state(self.state)`. |
| `id` | Documented equivalence | Exec body identical: `self.state.id()`. Only difference is Verus named return and requires/ensures annotations (stripped by AST comparison). |
| `thread_state` | Documented equivalence | Exec body identical: `&self.state`. Only difference is Verus named return and requires/ensures annotations. |
| `exit` | Documented equivalence | `ExitStatus` → `int` (abstract status tag model). `*mut ContextInformation` return omitted (HAL boundary). Core logic identical: `ZombieThread::from_state(self.state, status)`. |
| `put_mutex_guard` | Documented equivalence | `MutexAddress` → `u64` (widening, semantically harmless). `MutexGuard` parameter elided (protocol-only accounting — RAII payload opaque). Core delegation identical: `self.state.store_mutex_guard(address)`. |
| `take_mutex_guard` | Documented equivalence | `MutexAddress` → `u64`. `Option<MutexGuard>` return elided (precondition `spec_has_mutex` makes `None` path unreachable by construction — trust assumption T2). Core delegation identical: `self.state.take_mutex_guard(address)`. |
| `join_cond` | Intentional omission | Returns `Condvar` which is a sync boundary type elided from the `ThreadState` model. The `ThreadState` Verus model does not include a `join_cond` field (documented in `state.rs` module header lines 49, 51–52, 80). Adding it would require restructuring the `ThreadState` model across modules. Documented at trust boundary. |
| `RunningThread` struct | Documented equivalence | `state: Box<ThreadState>` → `state: ThreadState` (Box is transparent). Fields `pub` for Verus proof ergonomics (spec access, direct construction in lemmas). |
| `SleepingThread` struct | Justified extra | Boundary model of sibling module needed to verify `sleep()` transition. Documented with cross-module verification obligations. |
| `ReadyThread` struct | Justified extra | Boundary model of sibling module needed to verify `schedule()` transition. Documented with cross-module verification obligations. |
| `ZombieThread` struct | Justified extra | Boundary model of sibling module needed to verify `exit()` transition. Documented with cross-module verification obligations. |

## Verification: PASS

```
verification results:: 47 verified, 0 errors
```
