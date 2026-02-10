# Review: kcall_sleep (claude-opus-4.6)

## Grade: A-

## Previous Issues Status (Round 2 → Round 3)

### Medium Issue — FIXED

1. **`sleep_model()` success-path postcondition missing spec-level classification link** —
   **FIXED.** Exec line 417 now includes:
   ```
   spec_sleep_success_condition(...) ==> result.spec_classified_view()
       == spec_classify_pm_result(result.spec_pm_view())
   ```
   This is exactly the postcondition I suggested. It proves that on the success path, the
   3-arm classification was correctly applied to whatever PM result was obtained internally.
   This is non-trivial: it rules out a hypothetical `sleep_model` that returns a raw
   `TimedOut` without classification. Verified: the postcondition is machine-checked by
   Verus (15 verified, 0 errors).

### Low Issues — Both FIXED

2. **`sleep_end_to_end()` no postconditions** — **FIXED.** Now has two ensures clauses
   (exec lines 477-479):
   ```
   !matches!(result, SleepResultModel::TimedOut),
   result.spec_classified_view() == spec_classify_pm_result(result.spec_pm_view()),
   ```
   Both match the suggested fix exactly. The first propagates the core invariant that
   `TimedOut` is never exposed to callers. The second exposes the classification correctness.
   Verified by Verus.

3. **`lemma_sleep_result_trichotomy` misleading name** — **FIXED.** Renamed to
   `lemma_sleep_result_exhaustive` (proof line 176). Furthermore, the ensures clause was
   strengthened beyond what I suggested — it now proves both exhaustiveness AND mutual
   exclusion (proof lines 182-192):
   ```
   // Exhaustive: every result is success, killed, or generic error.
   spec_is_success(...) || spec_is_killed(...) || spec_is_generic_error(...),
   // Mutual exclusion: success and error are disjoint.
   !(spec_is_success(...) && spec_is_error(...)),
   // Mutual exclusion: killed and generic error are disjoint.
   !(spec_is_killed(...) && spec_is_generic_error(...)),
   ```
   This is stronger than the original dichotomy — it proves a proper three-way partition.
   Well done.

## New Issues Found

### Low

- **Location:** Module doc comment, Trust Boundary T5 (exec, lines 78-79)
  - **Description:** The doc says "with a precondition `seconds <= u32::MAX as u64` to match
    the 32-bit origin" but no such precondition actually exists on `sleep_model` or
    `sleep_end_to_end`. The actual precondition is
    `seconds as nat + nanoseconds as nat / NANOS_PER_SEC() <= u64::MAX as nat`.
    The documentation is slightly misleading about what precondition is enforced.
  - **Suggested Fix:** Update T5 documentation to say: "The model takes `(u64, u32)` and
    verifies the post-cast domain. Cast safety from 32-bit `usize` is at the ABI boundary,
    not verified here."

## Verification Summary

- **Verus output:** 15 verified, 0 errors
- **No `assume` statements:** Confirmed — none in exec, spec, or proof files
- **No unjustified `external_body`:** 3 external bodies (clock_now, checked_add_duration,
  process_manager_sleep), all at documented trust boundaries with appropriate postconditions
- **All original functions covered:** The single public function `sleep(seconds, nanoseconds)`
  is modeled by `sleep_model` and `sleep_end_to_end`

## Positive Observations

- **All Round 2 issues genuinely resolved:** Every fix was verified against the suggested
  approach. No issues were dismissed or worked around — each was addressed directly.
- **`lemma_sleep_result_exhaustive` exceeds expectations:** The mutual exclusion clauses
  strengthen the trichotomy proof beyond what was requested, proving a proper partition.
- **Complete postcondition coverage:** Both `sleep_model` and `sleep_end_to_end` now have
  postconditions covering overflow, success, and classification paths. All linked to
  spec-level functions.
- **Spec/proof/exec separation is exemplary:** Specs define the abstract types and
  classification logic; proofs verify properties about those specs; exec implements the
  concrete model and ties back to specs. No mixing of concerns.
- **Two-tier result type design:** `PmSleepResultView` (4 PM variants) →
  `spec_classify_pm_result` → `SleepResultView` (3 classified variants) cleanly separates
  the PM interface from the kcall interface.
- **Key properties all proven:**
  - Duration well-formedness (lemma_duration_new_wf)
  - Alarm well-formedness (lemma_alarm_wf)
  - Overflow → InvalidArgument (lemma_overflow_returns_invalid_argument)
  - TimedOut → Success (lemma_timed_out_is_success)
  - Killed → Error, NOT Success (lemma_killed_is_error)
  - Ok → Success (lemma_pm_success_is_success)
  - GenericError propagation (lemma_pm_error_propagates)
  - Exhaustive + mutually exclusive result partition (lemma_sleep_result_exhaustive)
  - Success ⟺ PmOk ∨ PmTimedOut (lemma_success_only_from_ok_or_timed_out)
  - Zero-duration validity (lemma_zero_duration_always_valid)
- **Documentation accuracy:** Module header, trust boundaries, API mapping table all
  accurately reflect the verified code (with the minor T5 wording issue noted above).

## Summary

The verification is now comprehensive and sound. All issues from Rounds 1 and 2 have been
genuinely fixed with machine-checked proofs. The spec correctly models the original's 3-arm
match semantics, the exec model faithfully implements it, and the proof lemmas establish all
key safety properties including exhaustive result classification with mutual exclusion.

The only remaining issue is a minor documentation inaccuracy in the T5 trust boundary
description. This does not affect soundness or verification quality.

**Grade rationale:** A- reflects a high-quality verification with correct semantics, strong
postconditions, comprehensive proof lemmas, and clean separation of concerns. The minor
documentation issue prevents A, but the verification itself is thorough and trustworthy.
