# Review: runnable (claude-opus-4.6)

## Grade: A-

## Verification Result

All 43 items verified, 0 errors. Clean pass via `./verus-ai/scripts/verify.sh runnable`.

## Issues Found

### Critical

None.

### High

- **Location:** `state_mut()` — omitted from exec model (documented in spec file, lines 56–60)
- **Description:** `state_mut()` returns `&mut ProcessState`, allowing arbitrary mutation of the inner process state (e.g., vmem mapping). This is omitted because Verus cannot express mutable reference return types. However, callers could break invariants (e.g., PID immutability) through this accessor. The verification trusts that callers of `state_mut()` do not violate `ProcessState` invariants, but this is not enforced.
- **Suggested Fix:** When `ProcessState` is independently verified, add a cross-module linking assertion confirming that mutations through `state_mut()` preserve the invariants assumed here (at minimum, PID immutability). Document this in the spec file's trust assumptions as an explicit cross-module verification obligation.

### Medium

- **Location:** `wakeup()` — exec file, line 497; `found` oracle parameter
- **Description:** The `found: bool` oracle parameter is required because `Seq::contains()` is spec-only and the sleeping list has no exec-level data structure. The precondition `found == spec_seq_contains(sleeping_thread_ids, tid)` constrains it correctly, but the caller must provide a correct search result, which shifts verification burden to the call site. This is the only remaining oracle parameter across all functions.
- **Suggested Fix:** Acceptable as-is. When verifying callers (e.g., the process manager), ensure the `found` value is derived from an actual search over the sleeping thread collection. The trust boundary is well-documented (spec lines 80–86). Consider adding an exec-level counter or hash set for sleeping thread IDs in a future iteration to eliminate the oracle entirely.

- **Location:** `EXIT_STATUS_INTERRUPTED()` — spec file, line 167
- **Description:** The constant `4` is hardcoded to match `ErrorCode::Interrupted` from `src/libs/sysapi/src/error.rs`. If the error code numbering changes, this spec becomes wrong silently. The spec file has a TODO acknowledging this (lines 164–166).
- **Suggested Fix:** Implement the suggested CI check: add a test or assertion in the verification pipeline that validates `EXIT_STATUS_INTERRUPTED() == ErrorCode::Interrupted as int`. Alternatively, import the constant from a shared verified module.

- **Location:** `find_thread()` / `find_thread_mut()` — omitted from exec model (spec file, lines 61–64)
- **Description:** These functions return `Option<ThreadRef<'_>>` / `Option<ThreadRefMut<'_>>`, which Verus cannot express. A spec-only model (`spec_find_thread`) verifies the search logic and variant selection, but no exec-level code verifies that the actual search implementation matches. The search priority (ready → interrupted → sleeping → zombie) is modeled correctly at the spec level.
- **Suggested Fix:** Acceptable for now. The spec model (spec file, lines 267–279) correctly captures the exhaustive search and priority ordering. When callers are verified, they should assert their results against `spec_find_thread` postconditions. Consider adding an exec-level wrapper that returns an enum tag (without the reference) for additional verification coverage.

### Low

- **Location:** `earliest_admission_time()` — spec-only model (spec file, lines 323–327)
- **Description:** The original function has a `unwrap_or(clock::now())` fallback for the case when the iterator returns no minimum. This is dead code given the `NonEmptyVecDeque` invariant. The spec correctly omits this fallback. Minor: the original's fallback behavior is not verified (though it is unreachable).
- **Suggested Fix:** No action needed. The spec model is correct because `wf()` guarantees `ready_thread_ids@.len() >= 1`, making the fallback unreachable. This is documented at spec file line 69.

- **Location:** `terminate()` postcondition — exec file, lines 414–415 and 424–425
- **Description:** In the `TerminateResult::Zombie` branch, the conditions `self.spec_interrupted_count() == 0` and `self.spec_sleeping_count() == 0` appear twice (both in the main ensures and repeated at the end). This is harmless redundancy.
- **Suggested Fix:** Remove the duplicate conditions at lines 424–425 for cleaner postconditions.

- **Location:** `run()` — exec file, line 367; `interrupt_reason: Ghost(0int)`
- **Description:** The interrupt reason is hardcoded to `0` in the ghost output. The postcondition does not constrain `interrupt_reason`, so this value is not observable. However, the choice of `0` is arbitrary and could be misleading to readers.
- **Suggested Fix:** No functional impact since the postcondition doesn't mention `interrupt_reason`. Optionally, add a comment noting this is an arbitrary witness value, or use a named constant like `UNCONSTRAINED_REASON`.

- **Location:** Thread ID disjointness — spec file, lines 337–343; `wf()` definition
- **Description:** Thread ID uniqueness across lists is NOT part of `wf()`. This is a deliberate trust assumption inherited from Rust's ownership model (a thread struct can only be in one collection at a time). The `spec_ids_disjoint()` predicate is provided separately for downstream proofs.
- **Suggested Fix:** Acceptable design choice. The documentation is thorough (spec lines 30–39). Downstream cross-module proofs should invoke `spec_ids_disjoint()` when needed. Consider adding a proof that operations preserve disjointness (given disjoint input, output is disjoint) to strengthen the trust story.

## Positive Observations

- **Comprehensive content-level postconditions:** All major state transitions (`run`, `terminate`, `wakeup`, `add_thread`) specify not just counts but exact sequence contents. For example, `run()` specifies that `result.ready_thread_ids == spec_remove_at(self.ready_thread_ids, sel)` and `terminate()` specifies `ip.interrupted_thread_ids == self.interrupted_thread_ids.add(self.sleeping_thread_ids)`. This goes well beyond simple count-level verification.

- **Minimal trust boundary:** Only one `external_body` (`clock_now()`) with a justified, minimal postcondition (`result >= 0`). No `assume` or `trusted` annotations anywhere. The oracle parameter in `wakeup()` is properly constrained by its precondition.

- **Oracle elimination:** The verification eliminated oracle parameters from `terminate()` (by using exec-level counters `interrupted_count`/`sleeping_count`) and from `run()` (by deriving the min-index via `lemma_earliest_ready_index_bounds`). Only `wakeup(found)` retains an oracle, with clear justification.

- **Strong inductive proofs:** `lemma_min_index_rec_bounds` and `lemma_seq_has_min` provide clean inductive proofs that the minimum-index selection is correct. `lemma_earliest_ready_index_bounds` bridges these to the module's domain.

- **Well-documented trust assumptions:** The spec file's header (lines 1–89) thoroughly documents the verification model, trust assumptions, exec coverage decisions, and oracle justifications. Each omission is explained with rationale.

- **Clean three-file split:** Exec code (693 lines), spec (512 lines), and proofs (750 lines) are well-separated. Specs define the abstract model and view types; proofs contain only lemmas; exec code contains the implementations with inline proof blocks limited to what's needed for verification.

- **Well-formedness preservation:** Every exec function that returns a `RunnableProcess` has `result.wf()` in its postcondition, ensuring the invariant is maintained across all transitions.

- **Correct semantic equivalence:** The verification model faithfully captures the original's semantics:
  - `terminate()` branching matches the original's `if let Some(interrupted_threads)` logic.
  - `run()` min-index selection matches the original's for-loop.
  - `wakeup()` find-and-remove matches the original's `remove_if` pattern.
  - Thread list ordering (e.g., ready-as-zombies before existing zombies in `terminate()`) matches the original's `append` order.

## Summary

This is a high-quality Verus verification of a complex kernel process state management module. The verification covers 6 of 11 original functions at the exec level (new, from_state, run, terminate, wakeup, add_thread), with 3 additional functions modeled at the spec level (find_thread, find_thread_mut, earliest_admission_time) and 2 trivial accessors (state, state_mut) documented as omitted. All omissions are justified by Verus type system limitations (reference returns, mutable borrows) and are clearly documented.

The strongest aspects are: (1) content-level postconditions that specify exact sequence contents rather than just counts, (2) systematic oracle elimination using exec-level counters and proof-level derivation, and (3) thorough documentation of trust assumptions and verification boundaries.

The main gap is the `state_mut()` omission, which creates a cross-module verification obligation: callers that mutate `ProcessState` must independently verify they preserve the invariants assumed here. This should be addressed when sibling modules are verified. The hardcoded `EXIT_STATUS_INTERRUPTED` constant is a minor maintainability risk that should be addressed with a CI cross-check.

Overall recommendation: merge-ready with the understanding that the `state_mut()` cross-module obligation and `EXIT_STATUS_INTERRUPTED` CI check are tracked as follow-up items.
