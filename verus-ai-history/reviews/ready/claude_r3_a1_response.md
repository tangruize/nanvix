# Response to Review: claude_r3_a1

## Verification Status: PASSED (34 verified, 0 errors)

---

## Issue Responses

### High: Pub struct fields weaken encapsulation

**Status: FIXED (Documentation)**

Added a note to the `ReadyThread` struct doc comment explaining that `pub` fields are a Verus modeling necessity and that construction should only occur via `new()`/`from_state()` which establish `wf()`. All methods require `wf()` as a precondition, so any directly-constructed instance with invalid state would fail precondition checks.

---

### Medium: EXIT_STATUS_INTERRUPTED hardcoded without mechanical linkage

**Status: FIXED (Documentation)**

Added a `CROSS-MODULE-CHECK` annotation to the spec constant:
```
/// CROSS-MODULE-CHECK: Confirm this value matches
/// `sys::error::ErrorCode::Interrupted as i32` if the error module changes.
```
This makes the audit obligation explicit and searchable alongside the existing boundary model annotations.

---

### Medium: Forwarding methods extend API beyond original

**Status: FIXED (Documentation)**

Relabeled all three forwarding methods as **"Verification-only API extension"** with explicit note that they are not present in the original `ReadyThread` and provide verified paths instead of `thread_state_mut()`. This makes the semantic divergence prominent.

---

### Medium: `clock_now()` lacks monotonicity

**Status: ACKNOWLEDGED (out of scope)**

The reviewer correctly identifies this as acceptable for current scope and provides a clear path for future strengthening (ghost global counter). No action needed — already documented in trust boundary sections.

---

### Low: `join_cond()` omission

**Status: ACKNOWLEDGED**

Reviewer says "no immediate action needed." Already documented. Will be addressed when sync subsystem is verified.

---

### Low: `run()` context pointer omission

**Status: ACKNOWLEDGED**

Reviewer says "no action for the current scope." Already documented with Modeling Note.

---

### Low: `thread_state_mut()` escape hatch

**Status: ACKNOWLEDGED**

Reviewer says "existing documentation and mitigation are good." AUDIT annotation with call sites already present from prior round.

---

### Low: Proof lemma style (structural copy vs exec function)

**Status: ACKNOWLEDGED**

Reviewer correctly identifies this as "a minor observation about proof style, not a correctness issue" and says "no immediate action." The connection between the structural `ThreadState { interrupt_reason: None, ..self.state }` and `take_interrupt_reason()`'s postconditions is sound.

---

## Summary

| Issue | Severity | Action | Change |
|-------|----------|--------|--------|
| Pub struct fields | High | Fixed | Added Verus necessity doc note |
| EXIT_STATUS_INTERRUPTED | Medium | Fixed | Added CROSS-MODULE-CHECK annotation |
| Forwarding methods divergence | Medium | Fixed | Relabeled as verification-only API extensions |
| clock_now() monotonicity | Medium | Acknowledged | Out of scope (reviewer agrees) |
| join_cond() | Low | Acknowledged | Future work (reviewer agrees) |
| run() pointer | Low | Acknowledged | Already documented (reviewer agrees) |
| thread_state_mut() | Low | Acknowledged | Already mitigated (reviewer agrees) |
| Proof lemma style | Low | Acknowledged | Not a correctness issue (reviewer agrees) |

**Verification: 34 verified, 0 errors.**
