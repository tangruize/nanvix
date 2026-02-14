# Review: thread_manager Exec Consistency (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

- None.

### Minor

1. **Verification command in consistency report is incorrect.** The report states
   "Module: `kernel::pm::thread`, Result: 221 verified, 0 errors" but the
   recommended verification command `./verus-ai/scripts/verify.sh thread_manager`
   fails because the module name is `kernel::pm::thread`, not `thread_manager`.
   The correct command is `./verus-ai/scripts/verify.sh kernel::pm::thread`.
   Verification passes with the correct command: 221 verified, 0 errors.

2. **`ThreadRefMutModel::thread_state()` vs original `thread_state_mut()` naming.**
   The verified model renames `thread_state_mut()` to `thread_state()` to reflect
   that only the read aspect is modeled. While functionally sound (the mutation
   trust boundary is well documented), a reviewer tracing from original to
   verified code must know to look for the renamed method. Consider adding an
   alias or a comment at the call site referencing the original name explicitly.

3. **`lemma_dispatch_preserves_wf` for `ThreadRefModel` is tautological.** The
   lemma requires `self.spec_state().wf()` and ensures `self.spec_state().wf()` —
   identical pre- and postcondition. It serves as documentation but proves nothing
   new. Not harmful, but could be strengthened (e.g., relating `wf()` of the
   returned `&ThreadState` from `thread_state()` to the spec_state).

### Observations (Non-Issues)

- **`next_id` made `pub`:** Required by Verus `View` trait. Documented in the
  struct's doc comment. Acceptable Verus framework constraint.

- **`ReadyThread` as a boundary model:** The boundary model in `mod.rs` wraps
  `ThreadState` instead of the full `ReadyThread` from `ready.rs`. This is
  properly documented with cross-module verification obligations (lines 276–289).
  The real `ready.rs::ReadyThread` is independently verified (221 count includes
  submodules). The postconditions of the boundary `new()` match those of the
  real `ready.rs` verified `ReadyThread::new`.

- **Overflow precondition strengthening:** The `create_thread` precondition
  `next_id < i32::MAX` is a deliberate strengthening. The original code has a
  latent overflow bug (wraps silently in release mode producing negative TIDs).
  The precondition makes the implicit assumption explicit. This is a legitimate
  and beneficial divergence, well documented in both the consistency report and
  the code (lines 60–64, 387–392).

- **HAL type elision:** `ContextInformation`, `FpuState`, `KernelStack`,
  `UserStack`, `VirtualAddress` abstracted to `Option<int>` or omitted. These
  are opaque hardware abstraction types correctly placed outside verification
  scope. The parameter count difference (4 vs 6 in `ReadyThread::new`,
  3 vs 4 in `create_thread`) follows from this elision.

## Criteria Assessment

| # | Criterion | Pass | Notes |
|---|-----------|------|-------|
| 1 | MISMATCH functions restored or equivalence documented | ✅ | No MISMATCHes reported in fix; all 5 MISSING functions moved from `thread_manager.rs` into `mod.rs`. |
| 2 | MISSING functions added with proper verification | ✅ | `ThreadManager` struct, `new`, `create_thread`, `init`, `thread_state`, `thread_state_mut` all present with postconditions verified. |
| 3 | Equivalence justifications sound | ✅ | Five documented equivalences are technically sound. Lifetime elision, value-based modeling, HAL abstraction, overflow strengthening all well-justified. |
| 4 | Exec code faithfully represents original | ✅ | All original functions modeled. Logic matches: ID assignment, monotonic increment, init delegation. Divergences are intentional strengthenings, not semantic changes. |
| 5 | Verification passes | ✅ | 221 verified, 0 errors (with corrected module name `kernel::pm::thread`). No `assume`, `admit`, or unjustified `external_body`. |

## Summary

The consistency fix correctly merges the previously separate `thread_manager.rs`
submodule back into `mod.rs`, restoring the file-level structure of the original
source. All six originally reported MISSING items (1 struct + 5 functions) are
now present in the correct file. The exec code faithfully models the original
with well-justified abstractions for HAL types, lifetime parameters, and mutable
references. The one intentional divergence (overflow precondition) is a
beneficial strengthening that catches a latent bug in the original. Verification
passes cleanly with 221 verified obligations and no errors. The only actionable
item is the incorrect verification command in the consistency report.
