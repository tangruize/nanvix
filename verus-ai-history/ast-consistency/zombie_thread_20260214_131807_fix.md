# Exec Consistency Fix: zombie_thread

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 5

All 5 reported mismatches are semantically equivalent to the original source.
No executable logic was changed in the Verus version. Differences arise from
Verus verification modeling (type abstractions) and syntax requirements (named
return values, ghost annotations).

## Struct: `ZombieThread`

| Field | Original | Verus | Justification |
|-------|----------|-------|---------------|
| `status` | `ExitStatus` (private) | `pub int` | Modeling decision: `ExitStatus` abstracted to `int` (documented trust boundary). `pub` for proof ergonomics (documented in code). |
| `state` | `Box<ThreadState>` (private) | `pub ThreadState` | Modeling decision: `Box<T>` is transparent, modeled as `T` directly (documented trust boundary). `pub` for proof ergonomics (documented in code). |

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `from_state` | Documented equivalence | Exec body is identical: constructs struct with `status` and `state`. Differences: (1) `Self { status, state }` vs `ZombieThread { status: status, state: state }` — Rust field shorthand vs explicit, semantically identical. (2) Parameter types (`Box<ThreadState>` → `ThreadState`, `ExitStatus` → `int`) are documented modeling decisions. (3) `proof { reveal(ZombieThread::wf); }` is ghost code stripped at runtime. (4) `pub(super)` → `pub` — visibility widened for Verus module structure. |
| `id` | Documented equivalence | Exec body identical: `self.state.id()`. Only difference is Verus named return syntax `(result: ThreadIdentifier)` and `requires`/`ensures` annotations (ghost). |
| `thread_state` | Documented equivalence | Exec body identical: `&self.state`. Only difference is Verus named return syntax `(result: &ThreadState)` and `requires`/`ensures` annotations (ghost). |
| `harvest` | Documented equivalence | Same logical operation: consumes self, calls `take_kernel_stack()` then `take_user_stack()`, returns both as tuple. Original uses `mut self` with direct field access; Verus uses `self` then `let mut state = self.state` (move-then-mutate). Both consume ownership identically. Expression decomposition into `let` bindings is semantically equivalent. Return type `(Option<KernelStack>, Option<UserStack>)` → `(Option<int>, Option<int>)` is documented modeling decision. `proof { reveal(...) }` is ghost code. |
| `status` | Documented equivalence | Exec body identical: `self.status`. Return type `ExitStatus` → `int` is documented modeling decision. Named return syntax is Verus requirement. |

## Verification: PASS

```
verification results:: 17 verified, 0 errors
Duration: 9s
```
