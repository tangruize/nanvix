# Review: sleeping_thread — Round 2 (claude-opus-4.6)

## Grade: A

## Previous Review Summary

Round 1 (A-) raised 7 issues: 1 High (thread_state_mut external), 3 Medium (join_cond omitted, boundary models trusted, InterruptReason as int), 3 Low (pub fields, alarm non-negative, alarm return type).

The prover claimed to fix 4 issues and rejected 3 as not applicable. This review verifies those claims.

## Verification of Previous Issue Fixes

### 1. ✅ FIXED — `spec_alarm` non-negative constraint (Low → Resolved)

**Previous issue:** `wf()` did not prevent nonsensical negative alarm values like `Some(-1)`.

**Verification:** Confirmed fixed. The diff shows three coordinated changes:
- `sleeping.spec.rs:128-131`: `wf()` now includes `self.alarm.is_some() ==> self.alarm.unwrap() >= 0`.
- `sleeping.rs:215`: `from_state` precondition now requires `alarm.is_some() ==> alarm.unwrap() >= 0`.
- `sleeping.proof.rs:33`: `lemma_from_state_is_wf` precondition updated to match.

**Soundness check:** The new `wf()` constraint propagates correctly through all existing methods:
- `wakeup()` consumes self (alarm is discarded) — no issue.
- `interrupt()` consumes self (alarm is discarded) — no issue.
- `set_thread_data_area()` preserves alarm unchanged (`self.spec_alarm() == old(self).spec_alarm()`), and `store_thread_data_area` on `ThreadState` does not touch the alarm field. Since `old(self).wf()` now implies the alarm constraint, and alarm is unchanged, `self.wf()` still holds. **Verified sound.**
- `alarm()`, `id()`, `thread_state()`, `get_thread_data_area()` are read-only — no issue.

All 32 verification conditions still pass. **Fix is correct and complete.**

### 2. ✅ FIXED — `thread_state_mut()` documentation (High → Mitigated)

**Previous issue:** `thread_state_mut()` is `#[verifier::external]`, bypassing verification.

**Verification:** The prover correctly identified this as a Verus limitation and improved the documentation:
- Added "Partial mitigation" note pointing callers to the verified `set_thread_data_area()`.
- Changed the future-work comment to a proper `TODO:` marker for tracking.

**Assessment:** The underlying issue (unverified `&mut` return) remains a Verus limitation. The documentation improvements are appropriate. The prover did not attempt any unsound workaround, which is correct. The `TODO` format is consistent with project conventions.

**Residual risk:** Unchanged — this remains the module's primary verification gap. Accepted as inherent to the current Verus tooling.

### 3. ✅ FIXED — `join_cond()` omission (Medium → Tracked)

**Previous issue:** `join_cond()` entirely omitted with no tracking marker.

**Verification:** The prover added a `TODO:` comment in the module-level trust boundary documentation (`sleeping.rs:37-39`) explaining what would be needed and why it can't be done now. The reasoning is accurate — `Condvar` has no meaningful Verus model currently, so an `external_body` stub would add boilerplate without proof value.

**Assessment:** Acceptable. The original suggestion to add a stub was speculative; the prover's judgment that a stub without Condvar modeling adds no verifiable guarantee is correct.

### 4. ✅ FIXED — Boundary model cross-module TODOs (Medium → Tracked)

**Previous issue:** Cross-module verification obligations were documented in prose but lacked explicit tracking markers.

**Verification:** `TODO (cross-module)` markers added to both `ReadyThread` (`sleeping.rs:88-89`) and `InterruptedThread` (`sleeping.rs:107-108`). The markers are clear and searchable with `grep -r "TODO (cross-module)"`.

**Assessment:** Adequate tracking. The `CROSS-MODULE-CHECK:` markers on the `from_state` doc comments (which already existed) are now complemented by explicit TODOs on the struct definitions.

## Verification of Rejected Issues

### 5. ✅ REJECTION JUSTIFIED — `InterruptReason` as `int` (Medium → Accepted)

**Prover's claim:** "All callers of `interrupt()` are required to satisfy `spec_valid_reason(reason)` as a precondition."

**Verification:** Confirmed. The defense is actually three-layered:
1. `SleepingThread::interrupt()` requires `spec_valid_reason(reason)` (line 263).
2. `InterruptedThread::from_state()` requires `spec_valid_reason(reason)` (line 183).
3. `InterruptedThread::wf()` includes `spec_valid_reason(self.reason)` (line 200 of spec file).

Any verified caller must satisfy the precondition, and the wf() check ensures only valid reasons persist. The only bypass is direct struct construction via pub fields, which is the separate known Verus limitation.

**Assessment:** Rejection justified. The int-with-predicate model is sound under verified usage.

### 6. ✅ REJECTION JUSTIFIED — `pub` fields (Low → Accepted)

**Prover's claim:** Reviewer already said "no action needed."

**Verification:** Correct. The original review explicitly stated: "This is a known Verus limitation. No action needed."

### 7. ✅ REJECTION JUSTIFIED — `alarm()` return type (Low → Accepted)

**Prover's claim:** "Already documented as an abstraction gap."

**Verification:** Correct. The module-level docs state: "`SystemTime` alarm -> `Option<int>` (abstract timestamp)." This is a known modeling choice, not a bug.

## New Issues Introduced by Fixes

### None found.

The alarm `wf()` change is the only semantic modification. I verified it does not weaken any existing postcondition or break any proof obligation:
- The 32 verification conditions pass unchanged.
- The new precondition on `from_state` is strictly additive (callers must now also prove alarm validity).
- No existing method's postcondition was weakened.

## Remaining Known Limitations

These are documented, tracked, and inherent to either Verus or the modular verification approach:

1. **`thread_state_mut()` external** — Verus limitation. Tracked with TODO.
2. **`join_cond()` omitted** — Condvar not modelable. Tracked with TODO.
3. **Boundary models trusted** — Cross-module obligation. Tracked with TODO markers.
4. **`pub` fields** — Verus limitation. Defended by `wf()` preconditions.

None of these are actionable without Verus tooling changes or completion of sibling module verification.

## Positive Observations

All positives from Round 1 remain valid. Additionally:

- **Responsive to feedback:** All fixable issues were addressed correctly. Documentation improvements are substantive, not cosmetic.
- **Sound judgment on rejections:** The three rejections are well-reasoned and verified correct.
- **No regressions:** The alarm `wf()` change is properly propagated through all three files with no collateral damage.
- **Clean verification:** 32 verified, 0 errors. No `assume()`, no `admit()`. Single `#[verifier::external]` justified by Verus limitation.

## Summary

The prover addressed all actionable issues from Round 1 correctly and completely. The alarm non-negative constraint is the only semantic change, and it is properly propagated through the spec, exec, and proof files. All three rejections are justified upon verification. No new issues were introduced.

The grade improves from A- to A: the alarm well-formedness fix closes the one specification gap that was actionable, and the documentation improvements provide adequate tracking for the remaining Verus-limited items. The module is now a clean, well-documented verification with no unsound shortcuts and proper tracking of all trust boundaries.
