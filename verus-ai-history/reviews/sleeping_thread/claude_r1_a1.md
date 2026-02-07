# Review: sleeping_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

None.

### High

- **Location:** `thread_state_mut()` in exec (`sleeping.rs:369`)
  **Description:** `thread_state_mut()` is marked `#[verifier::external]`, completely bypassing Verus verification. This function returns `&mut ThreadState`, allowing callers to arbitrarily mutate the underlying state — including changing the thread ID, alarm, mutex set, and well-formedness invariant — with no machine-checked guarantees. The doc comments list trust obligations (preserve `wf()`, `spec_id()`, `spec_alarm()`), but these are English-only and unenforceable. Any caller using this method operates outside the verification boundary and could silently break all proven invariants.
  **Suggested Fix:** Refactor callers to use specific setter methods (like `set_thread_data_area`) with per-field postconditions instead of exposing raw `&mut ThreadState`. If the general mutable reference is truly needed, document which callers use it and add integration-level audit markers. Consider wrapping mutation through a callback pattern (`fn with_state_mut<F: FnOnce(&mut ThreadState)>(f: F)`) that could at least assert `wf()` on exit.

### Medium

- **Location:** `join_cond()` — omitted from verified code
  **Description:** The original `SleepingThread::join_cond()` (line 143) is entirely omitted from the verification model. While documented as a "sync boundary" omission, `join_cond()` is a public method that returns a `Condvar` used for thread join synchronization. Its absence means the verification says nothing about whether the condition variable identity is preserved across the sleeping state, which is relevant for correctness of thread join operations.
  **Suggested Fix:** Add at minimum a `#[verifier::external_body]` stub with a postcondition asserting identity preservation (e.g., the condvar is the same one from the underlying `ThreadState`). Even if `Condvar` cannot be fully modeled, a spec-level token could track its identity.

- **Location:** Boundary models `ReadyThread` and `InterruptedThread` in exec (`sleeping.rs:89-107`)
  **Description:** The boundary models are locally defined structs that duplicate types from sibling modules. Their `from_state` postconditions are trusted assumptions — if the real `ReadyThread::from_state` or `InterruptedThread::from_state` implementations differ, the proofs here are vacuously correct. The cross-module verification obligations are documented in comments but not machine-checked. Notably, the real `ReadyThread::from_state` calls `clock::now()` and stores an `admission_time`, which is omitted here. While this is documented as intentional, it means the verification cannot reason about temporal properties of the wakeup transition.
  **Suggested Fix:** When the sibling modules (`ready.rs`, `interrupted.rs`) are independently verified, create a cross-module consistency checker (e.g., a shared spec file or a proof that the boundary model's postconditions are implied by the real module's postconditions). Add a tracking issue or TODO for this cross-module obligation.

- **Location:** `InterruptReason` modeling as `int` (`sleeping.rs:106, sleeping.spec.rs:65-68`)
  **Description:** The original `InterruptReason` is an enum with exactly two variants (`Killed`, `TimedOut`). It is modeled as an `int` with a `spec_valid_reason` predicate constraining it to `{0, 1}`. While functionally adequate, using an unbounded `int` is weaker than a true enum model — nothing prevents the creation of an `InterruptedThread` with reason `42` at the type level; only the precondition on `interrupt()` prevents this. If a caller forgets the precondition, the type system provides no defense.
  **Suggested Fix:** Consider modeling `InterruptReason` as a Verus enum with two variants, which would provide compile-time exhaustiveness. Alternatively, the current approach is acceptable if all callers are verified to satisfy `spec_valid_reason`.

### Low

- **Location:** `SleepingThread` fields are `pub` (`sleeping.rs:70-75`)
  **Description:** The verified `SleepingThread` struct has `pub` fields for Verus proof ergonomics, while the original has private fields. This is documented and justified, but means the verification model does not enforce the original's encapsulation — proofs can directly construct `SleepingThread` values without going through `from_state()`, potentially bypassing the `wf()` invariant. All current lemmas construct values directly (e.g., `SleepingThread { state: state, alarm: alarm }`) rather than through `from_state()`.
  **Suggested Fix:** This is a known Verus limitation. No action needed unless Verus adds support for private fields in spec mode. The `wf()` preconditions on all methods provide defense-in-depth.

- **Location:** `spec_alarm` type mismatch (`sleeping.spec.rs:81`)
  **Description:** The alarm field is modeled as `Option<int>` while the original uses `Option<SystemTime>`. `SystemTime` in the original kernel likely has richer semantics (e.g., monotonicity, non-negative values). The `int` abstraction loses these constraints — a `SleepingThread` could have `alarm = Some(-1)`, which is nonsensical for a timestamp.
  **Suggested Fix:** Add a well-formedness conjunct like `self.alarm.is_some() ==> self.alarm.unwrap() >= 0` to `wf()`, or introduce a `spec_valid_alarm` predicate. This would prevent nonsensical alarm values in proofs.

- **Location:** `alarm()` return type in verified code (`sleeping.rs:298`)
  **Description:** The verified `alarm()` returns `Option<int>` directly, while the original returns `Option<SystemTime>`. The return is correct semantically (both return the alarm field), but the type difference means the verified code's callers will need an additional abstraction layer mapping `int` back to `SystemTime` for integration.
  **Suggested Fix:** Document this as a known abstraction gap. No functional issue for module-level verification.

## Positive Observations

- **Complete function coverage:** All 10 public/pub(super) functions from the original source have verified counterparts or documented trust boundary justifications (9 verified + 1 `#[verifier::external]` with detailed trust obligations).
- **Clean verification:** 32 verified, 0 errors. No `assume()` statements anywhere in the module. The single `#[verifier::external]` on `thread_state_mut()` is justified by a genuine Verus limitation (`&mut T` return types).
- **Strong state transition specifications:** `wakeup()` and `interrupt()` have comprehensive postconditions covering identity preservation, well-formedness, mutex accounting, and drop safety — the key safety properties for thread state transitions.
- **Thorough proof lemmas:** The proof file contains 18 lemmas covering construction, identity correctness, both state transitions (wakeup/interrupt), TDA round-trip, and view equality. All lemmas verify without additional proof body, indicating the specifications are well-structured.
- **Excellent documentation:** The module-level doc comments clearly describe the verification model, trust boundaries, cross-module obligations, and abstraction choices. The `thread_state_mut()` trust boundary documentation is particularly thorough.
- **Proper spec/proof/exec separation:** Specifications, proofs, and executable code are cleanly separated into three files with clear responsibilities. View types provide clean abstraction boundaries.
- **Mutex accounting propagation:** State transitions correctly propagate the mutex set and count invariants, ensuring no mutex accounting information is lost during wakeup or interrupt transitions.
- **Drop safety tracking:** The `spec_drop_safe` property is correctly propagated through all state transitions, supporting compositional reasoning about resource cleanup.

## Summary

This is a well-executed verification of a relatively straightforward state-wrapper module. The `SleepingThread` essentially manages state transitions (sleeping → ready via wakeup, sleeping → interrupted via interrupt) while preserving thread identity and resource accounting invariants. The verification captures these core properties comprehensively.

The main concern is the `#[verifier::external]` on `thread_state_mut()`, which creates a genuine verification gap — any caller using this method operates outside the proof boundary. This is a Verus limitation, not a verification design flaw, but it should be tracked for resolution. The boundary models for `ReadyThread` and `InterruptedThread` introduce trusted assumptions that need cross-module validation when those sibling modules are verified. The omission of `join_cond()` is a minor gap given its role in thread synchronization correctness.

Overall, the verification achieves its stated goals with no unsound shortcuts, clean separation of concerns, and thorough documentation of trust boundaries. The grade reflects the strong execution with deductions for the `thread_state_mut()` gap and unverified boundary model assumptions.
