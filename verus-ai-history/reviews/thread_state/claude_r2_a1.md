# Review: thread_state (claude-opus-4.6)

## Grade: A-

## Verification Result

**46 verified, 0 errors.** All exec functions, spec functions, and proof lemmas pass cleanly.
No `assume`, `external_body`, or `trusted` annotations found — the verification is fully
machine-checked with no escape hatches.

## Issues Found

### Critical

None.

### High

None.

### Medium

1. **Ghost mutex set not explicitly preserved in non-mutex operation postconditions**
   - **Location:** `take_kernel_stack`, `take_user_stack`, `set_interrupt_reason`,
     `take_interrupt_reason`, `store_thread_data_area` (exec, `state.rs`)
   - **Description:** These functions preserve `spec_locked_mutex_count()` and `wf()` in
     their postconditions, but do not state that the ghost mutex set membership is preserved:
     ```
     forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
     ```
     Under `wf()`, count preservation guarantees `|set_new| == |set_old|`, but does **not**
     uniquely determine `set_new == set_old`. A downstream verifier relying solely on the
     postcondition (e.g., if the function were `external_body` or in a separate crate)
     could not prove that a specific mutex address is still held after calling
     `take_kernel_stack`. In practice, since function bodies are visible to Verus within the
     same crate, this works—but the specification is incomplete for modular reasoning.
   - **Suggested Fix:** Add a mutex-set frame condition to each non-mutex-modifying function:
     ```rust
     forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
     ```
     Alternatively, add `self.locked_mutex_set@ =~= old(self).locked_mutex_set@` for
     stronger set equality.

2. **`take_mutex_guard` return type differs from original**
   - **Location:** `take_mutex_guard` (exec, `state.rs`)
   - **Description:** The original function returns `Option<MutexGuard>`, allowing a `None`
     result if the address is not found. The verified version returns nothing and requires
     `old(self).spec_has_mutex(address@)` as a precondition, eliminating the None path
     entirely. While this is documented as trust assumption T2, it means the verified API
     is strictly stronger—callers must prove they hold the mutex, whereas the original code
     handles the not-found case at runtime. If a caller ever violates T2 in the real kernel,
     the original code returns `None` gracefully, but the verified model provides no
     reasoning about that scenario.
   - **Suggested Fix:** Consider modeling the return as `Ghost<bool>` indicating whether the
     address was present, or document explicitly in the trust assumption that callers outside
     the verification boundary are responsible for the None case. The current approach is
     acceptable if all callers are verified with the T2 obligation.

### Low

1. **`store_mutex_guard` address parameter is Ghost-only at exec level**
   - **Location:** `store_mutex_guard` (exec, `state.rs`)
   - **Description:** The original takes `(MutexAddress, MutexGuard)` with a real runtime
     address used as a `BTreeMap` key. The verified version takes `Ghost<int>`, so at exec
     level only the counter is incremented—the specific address is erased. This means the
     verified exec code for `store_mutex_guard` is a simple `count += 1`, which diverges
     structurally from the original `BTreeMap::insert(address, guard)`. The protocol-level
     model is correct (ghost set tracks addresses, counter tracks size), but any future
     integration with concrete mutex verification would need to bridge this gap.
   - **Suggested Fix:** No immediate fix needed; the protocol-only model is well-documented
     and appropriate for the current verification scope. Note this as a limitation if
     end-to-end mutex tracking is added later.

2. **`new()` constructor has fewer parameters than original**
   - **Location:** `new` (exec, `state.rs`)
   - **Description:** The original constructor takes `context: ContextInformation` and
     `fpu_state: FpuState`, which are omitted in the verified version. These are opaque HAL
     types that cannot be meaningfully modeled in pure spec. The omission is documented in
     the module header but means the verified constructor cannot enforce any
     context/FPU-related invariants.
   - **Suggested Fix:** No fix needed; this is a deliberate and well-justified modeling
     decision. If context or FPU verification is added in the future, extend the constructor
     accordingly.

3. **`Debug` and `Drop` trait impls not directly modeled**
   - **Location:** Original `state.rs` lines 276–292
   - **Description:** The `fmt::Debug` impl is purely cosmetic and correctly omitted. The
     `Drop` impl's logic (checking `!self.locked_mutexes.is_empty()` and logging an error)
     is modeled via `check_drop_safe()` and `spec_drop_safe()`, with the equivalence proven
     by `lemma_check_drop_safe_models_drop`. However, the Drop impl's actual *behavior*
     (logging and continuing, not panicking) is not modeled—the verified model treats this
     as a boolean predicate rather than an effectful operation.
   - **Suggested Fix:** No fix needed; the boolean predicate correctly captures the
     safety-relevant property (whether mutexes are leaked). The logging side-effect is
     outside the verification scope.

## Positive Observations

- **Thorough frame conditions:** Every mutating function specifies which fields are preserved,
  preventing unintended aliasing of state changes. This is systematic and consistent across
  all 9 mutating functions.

- **Well-formedness invariant (`wf()`):** The ghost-set-to-counter tie is clean and provides
  the right level of abstraction. It enables the `spec_drop_safe` ↔ `check_drop_safe`
  equivalence proof without over-constraining the model.

- **Trust assumptions are preconditions, not assumes:** T1 (no double-lock) and T2
  (release-what-you-hold) are encoded as `requires` clauses, pushing the proof obligation
  to callers rather than silently assuming correctness. This is the proper approach—any
  caller that violates these must fail verification.

- **Comprehensive proof library (26 lemmas):** The proof file covers construction,
  ID immutability, Option take/store semantics, mutex set operations with per-address
  non-interference, well-formedness preservation across all operations, roundtrip
  properties, drop safety, view equality, and resource tracking. This provides a rich
  API for downstream verifiers.

- **`lemma_mutex_store_take_roundtrip`:** Proves that inserting then removing the same
  address restores the original set state (using set extensionality `=~=`). This is a
  key internal consistency property for the protocol model.

- **`lemma_check_drop_safe_models_drop`:** Connects the exec-level boolean check to the
  spec-level predicate, formally justifying that the verification model captures the
  original `Drop::drop()` invariant.

- **Clean three-way split:** Spec functions and view types in `state.spec.rs`, proof lemmas
  in `state.proof.rs`, exec code with contracts in `state.rs`. The `include!` mechanism
  keeps them in the same Verus module while maintaining file-level separation.

- **Excellent documentation:** The module header documents verified properties, verification
  model, trust assumptions, and verification scope. Each function has clear doc comments
  explaining the modeling decisions.

## Summary

This is a well-executed verification of the ThreadState protocol. The module correctly
identifies the verifiable core—state management accounting (ID immutability, Option semantics,
mutex set consistency, drop safety)—and abstracts away HAL-specific opaque types (context,
FPU, condvar) that cannot be meaningfully modeled in pure spec. The protocol-only model for
mutex guards is a sound design choice: it verifies the *accounting* of which mutexes are held
while leaving the guard payload opaque.

The primary gap is the incomplete mutex-set frame conditions on non-mutex operations
(Medium #1), which could hinder modular reasoning if the verification evolves toward
separate-crate or external-body boundaries. Adding explicit set-preservation postconditions
would close this gap with minimal effort. The T2 return-type divergence (Medium #2) is a
deliberate strengthening that is acceptable given the trust assumption documentation, but
should be revisited if the verification scope expands to include error-path reasoning.

Overall, 46/46 verification conditions pass with zero escape hatches, the trust assumptions
are clearly delineated and justified, and the proof library is comprehensive. Recommended
for integration with minor postcondition strengthening.
