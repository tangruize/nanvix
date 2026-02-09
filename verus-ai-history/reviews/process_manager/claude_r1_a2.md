# Review: process_manager (claude-opus-4.6) — Re-review (R1A2)

## Grade: A-

## Previous Issues — Resolution Status

### High (1 issue)

1. **`terminate_ready_to_interrupted` semantic mismatch** — **FIXED.**
   The prover replaced `terminate_ready_to_interrupted` (which incorrectly moved ready→interrupted) with `terminate_ready_stays_ready` (line 597), which takes `&self` and correctly models the original behavior as a queue-level no-op. The documentation (lines 585–592) accurately describes the original code path (`terminate() → Ok(interrupted) → resume() → push_back(ready)`) and why the net effect is no queue change. The fix is semantically correct.

### Medium (4 issues)

2. **Missing `number_buffered_messages` frame postconditions** — **FIXED.**
   Both `exit_thread_to_suspended` (line 452) and `exit_thread_to_zombie` (line 484) now include `self.number_buffered_messages == old(self).number_buffered_messages`. Verified by inspection and Verus (32 verified, 0 errors).

3. **Coverage gap — thread-level operations documentation** — **FIXED.**
   The spec file now contains a comprehensive "Trust Boundary T3" section (spec lines 18–39) enumerating all unverified thread-level functions with their queue-level effects. Each function is mapped to its corresponding verified transition or documented as having no queue change. This is thorough and accurate.

4. **Coverage gap — outer `ProcessManager` wrapper documentation** — **FIXED.**
   The spec file now contains a "Trust Boundary T2" section (spec lines 40–70) with a complete mapping of all ~30 outer `ProcessManager` methods to their verified inner counterparts. The `try_borrow`/`try_borrow_mut` pattern is explained with its single-threaded safety rationale. I verified the mapping against the original source (mod.rs:1529–1982) — it is accurate and complete.

5. **Coverage gap — error paths** — **ADDRESSED (documentation).**
   The spec file now contains an "Error Path Verification Model" section (spec lines 72–81) explaining that preconditions model success conditions and that error paths are trivially state-preserving. The argument is standard for verification and is sound for this codebase: error paths in the original code return `Err(...)` without mutating `ProcessManagerInner` fields. This is a documentation response rather than new proofs, but the justification is valid.

### Low (4 issues)

6. **`recv_message` documentation** — **FIXED.** Lines 682–688 now document that this models the implicit decrement through `ProcessState::receive_message()` with a specific code reference (unsafe.rs:650–658).

7. **`spec_counts_bounded` weak bound** — **ACKNOWLEDGED.** Not changed, but the existing bound (`number_buffered_messages < usize::MAX`) is sufficient for overflow prevention. Acceptable.

8. **Queue ordering not modeled** — **ADDRESSED.** Documented in spec file "Queue Ordering" section (spec lines 83–90) as an intentional abstraction acceptable for current verification goals.

9. **`interrupt_capable` immutability** — **FIXED.** All 16 mutation functions now include `self.interrupt_capable == old(self).interrupt_capable` in their postconditions. Verified by inspection: `create_process` (206), `schedule` (272), `sleep_running` (313), `sleep_thread_running` (344), `exit_running` (385), `exit_thread_running` (416), `exit_thread_to_suspended` (453), `exit_thread_to_zombie` (485), `wakeup_to_ready` (516), `resume_all_interrupted` (544), `terminate_ready` (577), `terminate_suspended` (625), `harvest_zombie` (653), `post_message` (677), `recv_message` (702), `alarm_interrupt` (741). All verified by Verus.

## New Issues Found

### Medium

_None._

### Low

- **Location**: `terminate_ready_stays_ready` (exec, line 597)
  - **Description**: This function takes `&self` (immutable reference), making it a tautological proof: "if `wf()` holds and nothing changes, `wf()` still holds." While this correctly models the queue-level semantics (process stays in ready), it provides zero verification power for this code path. The original code *does* perform internal mutations (terminates threads, transitions process through intermediate states). All of that complexity is entirely trusted under T3. This is the weakest verified function in the module.
  - **Suggested Fix**: No code change needed — this is an inherent limitation of the abstraction level. Consider adding a comment noting that this is a T3-boundary function where the queue-level proof is trivially correct.

- **Location**: Proof file header (proof, lines 7–21)
  - **Description**: The proof file header lists proven properties but does not mention `terminate_ready_stays_ready`. While the function is trivially correct and uses no proof lemmas, the header should be kept in sync with the function inventory for documentation completeness.
  - **Suggested Fix**: Add a line: `// - terminate_ready_stays_ready: non-kernel ready process stays ready (T3 boundary).`

## Positive Observations

- **All previous issues genuinely addressed**: Every issue from R1A1 was either fixed in code or addressed with substantive documentation. No issues were dismissed without justification.
- **Zero assume/external_body**: Still no soundness escape hatches. 32 verification conditions pass cleanly.
- **Comprehensive frame conditions**: Every mutation function now has complete postconditions covering all fields: `running_pid`, `ready_count`, `suspended_count`, `interrupted_count`, `zombie_count`, `next_pid`, `number_buffered_messages`, `interrupt_capable`, and relevant ghost sets. This is a significant improvement over R1A1.
- **Excellent trust boundary documentation**: The spec file now serves as a complete traceability document mapping every original function to its verification status. This is valuable for auditing and future extension.
- **Correct semantic fix for `terminate`**: The `terminate_ready_stays_ready` function correctly captures that the original code's ready→interrupted→resume→ready sequence is a queue-level no-op. This was the most important fix.
- **Strong well-formedness invariant**: The `wf()` predicate remains comprehensive and is now proven preserved across all transitions with complete frame conditions.

## Summary

All issues from the previous review (R1A1) have been addressed. The high-priority semantic mismatch in `terminate_ready_to_interrupted` is properly fixed with `terminate_ready_stays_ready`. The medium-priority frame condition gaps are filled (both `number_buffered_messages` and `interrupt_capable` postconditions added throughout). The documentation in the spec file is now comprehensive, providing full traceability from the original ~50 functions to verified transitions or documented trust boundaries.

The verification passes (32 verified, 0 errors) with zero `assume`/`external_body` usage. The remaining issues are cosmetic (proof header sync) and inherent to the chosen abstraction level (`terminate_ready_stays_ready` being trivially correct). The verification provides strong guarantees for the process manager's core state machine: process queue partitioning, kernel liveness, PID uniqueness, and arithmetic overflow safety.

**Upgrade from B+ to A-**: The thorough response to all feedback items, combined with the already solid verification foundation, justifies the grade improvement. The remaining gap to A is the inherent limitation of not verifying thread-level logic (T3) and the trivial nature of `terminate_ready_stays_ready`.
