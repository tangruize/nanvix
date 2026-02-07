# Review: SleepingThread Verus Verification — Round 2, Attempt 2

**Reviewer:** Claude (automated)
**Date:** 2026-02-07
**Module:** `kernel::pm::thread::sleeping`
**Files reviewed:**
- `verus/split/kernel/pm/thread/sleeping.rs` (exec)
- `verus/split/kernel/pm/thread/sleeping.spec.rs` (spec)
- `verus/split/kernel/pm/thread/sleeping.proof.rs` (proof)

**Previous review:** `claude_r2_a1.md` (Grade: A-)

---

## Verification Status

```
verus --crate-type lib lib.rs --verify-module kernel::pm::thread::sleeping
verification results:: 33 verified, 0 errors
Status: PASSED
```

All 33 verification conditions pass successfully.

---

## Review of Previous Issues

### HIGH Issues

#### H1: Boundary ReadyThread Missing `admission_time` — **FIXED** ✅

**Previous concern:** Boundary `ReadyThread` lacked `admission_time`, producing a model weaker than the real `ReadyThread`.

**Fix applied:**
- Added `pub admission_time: int` field to boundary `ReadyThread` (sleeping.rs line 128).
- Added `spec_admission_time()` spec function (sleeping.spec.rs lines 158-160).
- Added `ReadyThreadView` with `admission_time` field (sleeping.spec.rs line 50).
- Updated `View` implementation to include `admission_time` (sleeping.spec.rs lines 163-170).
- Added `clock_now()` boundary function with `#[verifier::external_body]` that ensures `result >= 0` (sleeping.rs lines 79-85).
- `from_state` sets `admission_time: clock_now()` with postcondition `result.spec_admission_time() >= 0` (sleeping.rs line 182).

**Verdict:** Fully addressed. The boundary model now faithfully reflects the real `ReadyThread` structure.

**Minor note:** The boundary `ReadyThread::wf()` (sleeping.spec.rs line 178) only checks `self.state.wf()`, while the real verified `ReadyThread::wf()` (ready.spec.rs line 136) checks `self.state.wf() && self.admission_time >= 0`. This is not a soundness issue since `from_state` is the only constructor and it already ensures `admission_time >= 0`, but including it in `wf()` would be more complete for defensive modeling. Severity: Low.

---

#### H2: Boundary InterruptedThread Claims Stronger Postconditions — **MITIGATED** ✅

**Previous concern:** Boundary `InterruptedThread::from_state` ensures `spec_locked_mutex_count`, `spec_has_mutex`, and `spec_drop_safe`, but the real verified `InterruptedThread::from_state` only ensures `spec_id`, `spec_reason`, and `wf`.

**Fix applied:** Added thorough documentation (sleeping.rs lines 212-217) explaining structural transparency:

> *"The real InterruptedThread constructor is `InterruptedThread { state, reason }` and all spec accessors delegate to `self.state`. Since the boundary model constructs InterruptedThread with the same state field, these stronger postconditions hold by structural transparency."*

**Verification:** I confirmed this claim by checking the real verified module:
- `verus/split/kernel/pm/thread/interrupted.spec.rs`: `spec_locked_mutex_count`, `spec_has_mutex`, `spec_drop_safe` all delegate to `self.state` (e.g., `self.state.locked_mutex_count`).
- The real `InterruptedThread` struct is `{ state: ThreadState, reason: InterruptReason }`.
- Since the boundary constructs with the same `state` field, the stronger postconditions are provably true by structural transparency.

**Verdict:** Sound. The stronger postconditions are valid deductions from structural transparency, not unsound assumptions. Documentation makes the reasoning explicit.

---

### MEDIUM Issues

#### M1: No TDA Roundtrip at SleepingThread Level — **FIXED** ✅

**Previous concern:** `lemma_tda_roundtrip` existed at `ThreadState` level but not at `SleepingThread` level.

**Fix applied:** Added `lemma_sleeping_tda_roundtrip` (sleeping.proof.rs lines 242-260) which proves:
- After setting `user_tda`, the new value is retrievable via `spec_user_tda()`.
- Other fields (`spec_id`, `spec_alarm`, `spec_locked_mutex_count`, `spec_drop_safe`) are preserved.
- Well-formedness is maintained.

Auto-discharged by Verus (empty body), confirming the property follows directly from structural definitions.

**Verdict:** Fully addressed.

---

#### M2: `thread_state_mut` Uses `#[verifier::external]` — **UNCHANGED** (Acceptable)

Still marked `#[verifier::external]` (sleeping.rs line 418). Documentation explains this is a Verus limitation: `&mut` access to a struct field cannot be expressed in Verus's ownership model. The function is trivial (`&mut self.state`) and the safety argument is sound.

**Verdict:** No change needed. Verus limitation properly documented.

---

#### M3: Alarm Well-formedness Only Requires `>= 0` — **UNCHANGED** (Acceptable)

Alarm well-formedness still only requires `alarm.unwrap() >= 0` (sleeping.spec.rs line 102). This is appropriate for the current abstraction level where `SystemTime` is modeled as `int`.

**Verdict:** No change needed. Design decision is reasonable.

---

### LOW Issues

#### L1: Constant Duplication — **UNCHANGED** (Tracked)

`INTERRUPT_REASON_KILLED` and `INTERRUPT_REASON_TIMED_OUT` remain duplicated from `interrupted.rs`. TODO comments present for future consolidation.

#### L2: Alarm Type Abstraction — **UNCHANGED** (Acceptable)

`Option<int>` used for `SystemTime`. Appropriate abstraction.

#### L3: Empty Proof Bodies — **UNCHANGED** (Good Sign)

All lemmas auto-discharged. Indicates clean structural proofs.

---

## Check for New Issues

### N1: Boundary `ReadyThread::wf()` Missing `admission_time` Constraint — **NEW, Low**

As noted under H1, the boundary `ReadyThread::wf()` checks only `self.state.wf()` while the real verified `ReadyThread::wf()` also checks `self.admission_time >= 0`. Since `from_state` is the only constructor and ensures `admission_time >= 0`, this gap cannot be exploited. However, adding the constraint to `wf()` would strengthen defensive modeling.

### No Other New Issues

The fixes introduce no new soundness concerns. No `assume`, `admit`, `panic!`, `unwrap()`, or `expect()` statements found. The only `external_body` is on `clock_now()` (a justified boundary function). The only `external` is on `thread_state_mut()` (a Verus limitation).

---

## Soundness Assessment

| Category | Status |
|---|---|
| `assume` / `admit` | None ✅ |
| `#[verifier::external_body]` | 1 (`clock_now`) — justified boundary ✅ |
| `#[verifier::external]` | 1 (`thread_state_mut`) — Verus limitation ✅ |
| Verification result | 33/33 verified, 0 errors ✅ |
| Spec/Proof/Exec separation | Clean ✅ |
| Structural soundness | All boundary assumptions validated ✅ |

---

## Summary

The prover addressed all actionable issues from the A- review:

1. **H1 (ReadyThread admission_time):** Fully fixed with proper boundary function, spec, and postconditions.
2. **H2 (InterruptedThread postconditions):** Mitigated with excellent documentation. Structural transparency argument verified against real module source.
3. **M1 (TDA roundtrip):** Fully fixed with SleepingThread-level lemma.

Remaining items are either Verus limitations (M2), reasonable design decisions (M3, L2), tracked TODOs (L1), or positive indicators (L3). One new Low-severity item (N1) identified but is non-exploitable.

The module is sound, well-documented, and verification-complete.

---

## Grade: A

**Upgrade from A- justified by:**
- Both High-severity issues resolved (one fully fixed, one soundly mitigated with evidence).
- Requested Medium fix (TDA roundtrip lemma) implemented.
- No new significant issues introduced.
- 33/33 verification conditions pass.

**Not A+ because:**
- One `#[verifier::external]` annotation remains (Verus limitation, not author's fault, but still a verification gap).
- Minor `wf()` completeness gap in boundary ReadyThread (N1).
- Constant duplication not yet consolidated (tracked with TODO).
