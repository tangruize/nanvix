# Exec Consistency Fix: thread_manager

## Summary
- Mismatches fixed: 0
- Missing functions added: 5 (moved from thread_manager.rs submodule into mod.rs)
- Missing structs added: 1 (ThreadManager moved into mod.rs)
- Documented equivalences: 5

## Root Cause

The verified code placed all exec functions (`new`, `create_thread`, `init`,
`thread_state`, `thread_state_mut`) and the `ThreadManager` struct in a separate
submodule `thread_manager.rs`, whereas the original source has them in
`src/kernel/src/pm/thread/mod.rs`. The AST consistency tool compares
`mod.rs` to `mod.rs` and thus reported all 5 functions and the struct as
MISSING_IN_VERUS.

## Fix Applied

Merged the content of `thread_manager.rs`, `thread_manager.spec.rs`, and
`thread_manager.proof.rs` into `mod.rs`, `mod.spec.rs`, and `mod.proof.rs`
respectively. Removed the `pub mod thread_manager;` declaration and deleted
the three `thread_manager.*` files. This restores the same file-level
structure as the original source.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `ThreadManager` (struct) | Moved from `thread_manager.rs` to `mod.rs` | Mirrors original source structure where struct is in `mod.rs`. |
| `new` | Moved from `thread_manager.rs` to `mod.rs` | Mirrors original `ThreadManager::new()` in `mod.rs:158-176`. Verification model omits `ContextInformation`/`FpuState` HAL types (out of scope) and uses `Option<int>` for stack/tda. Semantically equivalent for ID assignment logic. |
| `create_thread` | Moved from `thread_manager.rs` to `mod.rs` | Mirrors original `ThreadManager::create_thread()` in `mod.rs:194-213`. Adds overflow precondition (`next_id < i32::MAX`) that strengthens original (intentional: catches latent overflow bug). |
| `init` | Moved from `thread_manager.rs` to `mod.rs` | Mirrors original `init()` in `mod.rs:229-233`. Semantically identical: delegates to `ThreadManager::new()`. |
| `thread_state` | Already in `mod.rs` as `ThreadRefModel::thread_state()` | Models `ThreadRef::thread_state()` from original `mod.rs:80-88`. Uses value-based `ThreadState` instead of lifetime-parameterized references (Verus limitation: cannot express `&'a T` in enum variants). Dispatch semantics verified: identity preservation across all 5 variants. |
| `thread_state_mut` | Already in `mod.rs` as `ThreadRefMutModel::thread_state()` | Models `ThreadRefMut::thread_state_mut()` from original `mod.rs:123-131`. Read-only model due to Verus limitation (`&mut T` return types unsupported). Mutation is a documented trust boundary. |

## Documented Equivalences

1. **`ThreadRefModel` ↔ `ThreadRef`**: Value-based enum models lifetime-parameterized enum.
   Verus cannot express `&'a ReadyThread` in enum variants. Dispatch semantics (identity
   preservation) are verified; aliasing is enforced by Rust's borrow checker (outside Verus scope).

2. **`ThreadRefMutModel` ↔ `ThreadRefMut`**: Same as above, plus mutation trust boundary.
   Verus cannot express `&mut T` return types. Read aspect verified; mutation requires
   callers to preserve `wf()` and `spec_id()`.

3. **`ReadyThread` boundary model**: Wraps `ThreadState` instead of the full `ReadyThread`
   from `ready.rs`. Omits `admission_time` (scheduling, not relevant to ID assignment).
   Cross-module check documented: postconditions must match real `ReadyThread::new`.

4. **HAL type elision**: `ContextInformation`, `FpuState`, `KernelStack`, `UserStack`,
   `VirtualAddress` are abstracted to `Option<int>` or omitted entirely. These are
   opaque hardware abstraction types outside verification scope.

5. **Overflow strengthening**: `create_thread` adds `requires next_id < i32::MAX`.
   The original has no overflow check and wraps silently in release mode. This is
   an intentional strengthening, not a semantic change.

## Verification: PASS
- Module: `kernel::pm::thread`
- Result: 221 verified, 0 errors
- No assume, admit, or unjustified external_body used
