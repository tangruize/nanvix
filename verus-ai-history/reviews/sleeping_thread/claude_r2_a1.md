# Review: sleeping_thread (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

(none)

### High

- **Location:** `sleeping.rs` (exec), boundary `ReadyThread::from_state` (lines 154–166)
  **Description:** The boundary model of `ReadyThread` in sleeping.rs omits the `admission_time` field entirely. The real `ReadyThread` (ready.rs:109–114) has `pub admission_time: int`, and the real `from_state` (ready.rs:291–310) calls `clock_now()` to initialize it. The sleeping.rs boundary model's `ReadyThread` is structurally different from the real type — it has only a `state` field. While the module header documents this as "Out of Scope," it creates a **type-level divergence**: any cross-module lemma linking the sleeping.rs boundary `ReadyThread` to the real `ReadyThread` would fail structurally because the types don't match. This is the single largest gap in the verification — if a future caller of `wakeup()` needs to reason about the returned `ReadyThread`'s admission time, the boundary model provides no information.
  **Suggested Fix:** Either (a) add `admission_time: int` to the boundary `ReadyThread` with a postcondition `result.admission_time >= 0` on `from_state`, matching the real module, or (b) add a formal cross-module linking mechanism (e.g., a ghost trait) that accounts for the structural difference. At minimum, document this as a known type mismatch in the cross-module TODO.

- **Location:** `sleeping.rs` (exec), boundary `InterruptedThread::from_state` (lines 194–207)
  **Description:** The boundary model's `InterruptedThread::from_state` postcondition includes `forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a)`, but the real `InterruptedThread::from_state` in interrupted.rs (lines 140–149) only ensures `result.spec_id() == state.spec_id()`, `result.spec_reason() == reason`, and `result.wf()`. The boundary model **assumes stronger postconditions** (mutex set preservation, drop safety preservation) than the real module actually proves. If the real module's `from_state` were ever changed to not preserve these (unlikely but possible), the sleeping.rs boundary would be unsound.
  **Suggested Fix:** Verify that the real interrupted.rs `from_state` does in fact structurally preserve all fields (it does — it's `InterruptedThread { state, reason }`), and add an explicit cross-module validation comment mapping each boundary postcondition to the structural proof in the real module. The existing CROSS-MODULE-CHECK comment is good but should be elevated to a formal validation step.

### Medium

- **Location:** `sleeping.rs` (exec), `thread_state_mut()` (lines 364–398)
  **Description:** `thread_state_mut()` is `#[verifier::external]` and returns `&mut ThreadState`, creating an unverified escape hatch. Any caller can mutate the thread state arbitrarily (change id, corrupt mutex accounting, invalidate wf()). The trust obligations are well-documented, and the suggestion to use `set_thread_data_area()` as a verified alternative is good. However, there is no way to audit at verification time whether callers honor the documented obligations.
  **Suggested Fix:** This is a known Verus limitation. The mitigation is adequate for now. When Verus supports `&mut T` returns, this should be prioritized for replacement. Consider adding a `// SAFETY-AUDIT: last reviewed <date>` annotation to track manual review.

- **Location:** `sleeping.spec.rs` (spec), `wf()` predicate (lines 134–137)
  **Description:** The well-formedness predicate requires `alarm.is_some() ==> alarm.unwrap() >= 0` but does not constrain the alarm value to be finite or within a reasonable range. While `int` in Verus is mathematical (unbounded), extremely large alarm values could model nonsensical timestamps. This is arguably correct for the abstraction level (alarm is just an opaque token that the scheduler interprets), but it means the spec cannot rule out "alarm in the year 9999" scenarios.
  **Suggested Fix:** This is acceptable for the current abstraction level. No change needed unless scheduler-level verification requires bounded timestamps. Document the design decision if not already clear.

- **Location:** `sleeping.proof.rs` (proof), `lemma_tda_roundtrip` (lines 222–236)
  **Description:** The TDA roundtrip lemma operates at the `ThreadState` level by constructing a `ThreadState { user_tda: tda, ..state }` directly, rather than going through `SleepingThread::set_thread_data_area` → `SleepingThread::get_thread_data_area`. While the lemma is correct, it tests the underlying state field update rather than the actual SleepingThread API roundtrip. The exec functions `set_thread_data_area` and `get_thread_data_area` do have sufficient postconditions that a caller-level roundtrip is provable, but this lemma doesn't directly demonstrate it.
  **Suggested Fix:** Add a complementary lemma that explicitly constructs a `SleepingThread`, calls `set_thread_data_area` (in proof via field update), and reads back via `spec_user_tda()` to demonstrate the end-to-end roundtrip at the SleepingThread abstraction level.

### Low

- **Location:** `sleeping.spec.rs` (spec), `INTERRUPT_REASON_KILLED()` and `INTERRUPT_REASON_TIMED_OUT()` (lines 68–74)
  **Description:** These constants duplicate the same constants defined in `interrupted.spec.rs` (lines 84–87). Both define `INTERRUPT_REASON_KILLED() -> 0` and `INTERRUPT_REASON_TIMED_OUT() -> 1`. Because each module uses `include!()` to incorporate its spec file, these are separate definitions that could drift apart.
  **Suggested Fix:** Extract these constants to a shared spec file (e.g., `thread_constants.spec.rs`) imported by both modules, or add a cross-module consistency check. The TODO comments acknowledging this are present and appropriate.

- **Location:** `sleeping.rs` (exec), `alarm` field type (line 91)
  **Description:** The alarm is modeled as `Option<int>` (mathematical integer), while the original uses `Option<SystemTime>`. The abstraction is sound (SystemTime is a wrapper around a non-negative value), and the non-negativity constraint is captured in `wf()`. However, the original `SystemTime` likely has a specific representation (e.g., nanoseconds since epoch) that is lost in the abstraction.
  **Suggested Fix:** No change needed. The abstraction is appropriate for module-level verification. If time arithmetic verification is needed at the scheduler level, a more concrete `SystemTime` model can be introduced there.

- **Location:** `sleeping.proof.rs` (proof), general observation
  **Description:** All proof lemmas have empty bodies — they are discharged automatically by Verus's SMT solver. While this demonstrates that the specs are well-structured and the properties follow directly from definitions, it also means none of the lemmas add proof insight beyond what the solver already knows. They serve primarily as documentation that the property was checked.
  **Suggested Fix:** No change needed. Auto-discharged lemmas are a sign of clean spec design. They serve as regression guards if specs change in the future.

## Positive Observations

- **Excellent documentation:** The module header (lines 1–58 in sleeping.rs) is exemplary. It clearly documents verified properties, out-of-scope items, the verification model, trust boundary decisions, and cross-module dependencies with TODO markers. This is among the best-documented verification efforts I've reviewed.

- **No `assume` or `external_body` in core module:** The sleeping thread verification has zero `assume` statements and zero `external_body` functions within its verification boundary. The only `#[verifier::external]` is `thread_state_mut()` which is outside the `verus!` block with comprehensive trust documentation.

- **Complete function coverage:** All 10 public functions from the original source have verified counterparts:
  1. `from_state` ✓
  2. `wakeup` ✓
  3. `interrupt` ✓
  4. `id` ✓
  5. `thread_state` ✓
  6. `thread_state_mut` ✓ (external with documented trust boundary)
  7. `join_cond` — intentionally omitted (Condvar is an opaque sync type; justified)
  8. `alarm` ✓
  9. `set_thread_data_area` ✓
  10. `get_thread_data_area` ✓

- **Strong state transition verification:** The `wakeup()` and `interrupt()` transitions preserve identity, well-formedness, mutex accounting, and drop safety. These are the essential correctness properties for a thread state machine.

- **Clean spec/proof/exec separation:** The three-file split is well-executed. Specs contain only spec functions and View types. Proofs contain only lemmas. Exec code contains structures and implementations. No leakage between layers.

- **Boundary models are conservative:** The `ReadyThread` and `InterruptedThread` boundary models only define what's needed for the sleeping module's verification, with clear cross-module validation TODOs.

- **Verification passes cleanly:** 32 verified items, 0 errors, in 4 seconds. No warnings or partial results.

## Summary

The sleeping_thread verification is a high-quality, well-documented effort that correctly captures the essential state transition safety properties of the `SleepingThread` type. All functions are covered (with `join_cond` justifiably omitted), there are no `assume` statements or `external_body` functions in the core verification, and the spec/proof/exec separation is clean.

The primary concern is the structural mismatch between boundary models and their real counterparts (the `ReadyThread` boundary omits `admission_time`), which creates a gap that must be bridged by manual cross-module review. The boundary `InterruptedThread::from_state` postconditions are stronger than what the real module explicitly proves, relying on structural transparency of the constructor. These are inherent challenges of modular verification with boundary models, and the extensive cross-module TODO documentation demonstrates awareness of the issue.

The `thread_state_mut()` escape hatch is the only unverified code path, and it's a Verus limitation rather than a verification design flaw. The trust boundary documentation is thorough.

**Recommendations:**
1. Prioritize cross-module boundary validation as sibling modules are verified — the TODOs are well-placed but need follow-through.
2. Consider adding `admission_time` to the boundary `ReadyThread` to enable future scheduler-level reasoning.
3. Add an end-to-end TDA roundtrip lemma at the `SleepingThread` abstraction level.
