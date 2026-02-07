# Review: interrupted (claude-opus-4.6)

## Grade: A

## Issues Found

### Critical

None.

### High

None.

### Medium

- **ReadyThread boundary model is unlinked to the real ReadyThread module**
  - Location: `ReadyThread::from_state` (exec, lines 112–122)
  - Description: `ReadyThread` is defined as a local boundary model with its own
    `from_state` postconditions (preserves id, interrupt reason, and wf). When the
    real `ReadyThread` module is verified independently, there is no automated
    mechanism to ensure these boundary postconditions are implied by the real
    `ReadyThread::from_state` spec. If the real module has weaker postconditions
    or additional preconditions, the `resume()` proof could be unsound in context.
    The code comments acknowledge this ("Cross-module dependency") but it remains
    a manual obligation.
  - Suggested Fix: Add a cross-module consistency check (e.g., a proof lemma in
    the `ReadyThread` module that imports this boundary model's postconditions and
    proves they follow from the real spec), or document this as a standing
    verification obligation in a top-level tracking file.

### Low

- **`join_cond()` is omitted from the verification model**
  - Location: original `interrupted.rs` line 127; not present in verified code
  - Description: The original `join_cond(&self) -> Condvar` is entirely omitted.
    While `Condvar` is an opaque sync boundary type that cannot be meaningfully
    modeled in a pure spec, the omission means that any synchronization properties
    involving interrupted threads and thread-join are outside the verification
    boundary.
  - Suggested Fix: No code change needed — the omission is well-justified and
    thoroughly documented. Track as an explicit out-of-scope item if a
    synchronization verification effort is undertaken later.

- **`thread_state_mut` is `#[verifier::external]`**
  - Location: exec, lines 240–243
  - Description: `thread_state_mut(&mut self) -> &mut ThreadState` is marked
    `#[verifier::external]` due to Verus not supporting `&mut T` return types.
    This means callers can mutate the `ThreadState` without any machine-checked
    postconditions. The documentation is exemplary (listing intended
    postconditions, known call sites, and why FPU mutations are safe), but it
    remains a trust gap.
  - Suggested Fix: When Verus adds `&mut T` return support, replace with a
    verified function. In the interim, consider adding a `debug_assert!`-style
    runtime check for `wf()` at caller sites in unverified code, or refactoring
    callers to use setter methods with per-field postconditions.

- **Proof lemmas for `resume` manually reconstruct post-state (fragile coupling)**
  - Location: proof, lines 87–178 (all `lemma_resume_*` functions)
  - Description: Each resume lemma constructs the post-state as
    `ThreadState { interrupt_reason: Some(self.reason), ..self.state }`,
    mirroring the exec code's call to `set_interrupt_reason` followed by field
    copy. If the exec `resume` code changes (e.g., modifies additional fields),
    these lemmas must be updated in sync. The code includes a comment noting
    this, but the coupling is inherently fragile.
  - Suggested Fix: Consider restructuring the proofs to reference the
    `set_interrupt_reason` postconditions directly rather than constructing an
    independent post-state, which would make the proofs automatically track
    changes to the exec code.

- **`InterruptReason` modeled as `int` rather than a Verus datatype**
  - Location: spec, lines 84–87; exec, line 74
  - Description: The original `InterruptReason` is a Rust enum with two variants.
    It is modeled as `int` with a validity predicate `spec_valid_reason`. While
    functional, a Verus `enum` or `datatypes!` would provide stronger type safety
    and make exhaustive matching in proofs more natural. The current approach
    requires manual validity checks throughout.
  - Suggested Fix: Consider modeling as `pub enum SpecInterruptReason { Killed, TimedOut }`
    in a future refactor. This is low priority since the current approach is
    correct and the validity predicate is consistently enforced.

## Positive Observations

- **Excellent documentation**: Every trust boundary, modeling decision, and
  out-of-scope item is thoroughly documented in file headers and function-level
  comments. The `thread_state_mut` documentation is particularly exemplary,
  listing intended postconditions, known call sites, and why the trust gap is
  safe in practice.

- **No `assume`, `trusted`, or `external_body` in any of the three files**: The
  only escape hatch is the single `#[verifier::external]` for `thread_state_mut`,
  which is well-justified by a Verus language limitation.

- **Complete function coverage**: All 6 original functions are accounted for —
  5 verified and 1 (`join_cond`) explicitly documented as out-of-scope with
  clear justification.

- **Key safety property proven**: The `resume()` function correctly ensures that
  `result.spec_interrupt_reason() == Some(self.spec_reason())`, which is the
  essential state-transition safety property — the interrupt reason is correctly
  propagated from the `InterruptedThread` to the underlying `ThreadState`.

- **Rich spec surface**: Beyond basic getters, the spec provides mutex accounting
  transparency (`spec_locked_mutex_count`, `spec_has_mutex`), drop safety
  (`spec_drop_safe`), and stack ownership (`spec_kernel_stack`, `spec_user_stack`)
  — all passed through from the `ThreadState` dependency.

- **Clean split quality**: The spec/proof/exec separation is well-done. Specs
  define the abstract model and invariants, proofs establish lemmas about
  construction and state transitions, and exec code contains only the verified
  implementations. The `include!` mechanism keeps them integrated.

- **Verification passes cleanly**: 21 verified, 0 errors, completing in ~4 seconds.

## Summary

This is a high-quality verification of a relatively straightforward OS kernel
component. The `InterruptedThread` type is a state wrapper with a key
state-transition function (`resume`), and the verification correctly captures its
essential properties: identity preservation, reason propagation, and
well-formedness maintenance across the transition.

The modeling choices are sound — `Box<T>` as `T`, `InterruptReason` as
validity-constrained `int`, `ReadyThread` as a boundary model. The single
`#[verifier::external]` escape is well-justified. The main area for improvement
is cross-module consistency checking for the `ReadyThread` boundary model, which
currently relies on manual review to ensure alignment with the real
`ReadyThread` module's spec.
