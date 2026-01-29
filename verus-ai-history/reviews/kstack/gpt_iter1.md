# Review: kstack (gpt-5.1-codex-max)

## Grade: C

## Issues Found

### Critical
- **Coverage / Equivalence – `new`**: Verified `KernelStack::new` takes `(base_addr, num_pages)` and assumes preconditions instead of allocating pages through `VirtMemoryManager` as in the original `new(mm)` (which can fail). Allocation, resource acquisition, and error paths are unmodeled, so the proof does not cover the real constructor behavior or failure modes. *Suggested fix*: Model allocator interaction (e.g., ghost/exec split calling `mm.alloc_kpages`) and prove alignment, contiguity, bounds, and error propagation for the real constructor.
- **Coverage – `Drop` / resource release**: The original `Drop` frees `KernelPage`s; the verified code has no destructor or proof that pages are released or that double-free/use-after-free cannot occur. *Suggested fix*: Add a verified `Drop` modeling ownership of pages and prove deallocation correctness (no leaks/double frees).

### High
- **Specification completeness – stack size constant**: Original uses `config::kernel::KSTACK_SIZE` derived from architecture config; verified code hardcodes `num_pages` and `PAGE_SIZE`, ignoring the configured stack size. Proofs could hold for values not matching the real build configuration. *Suggested fix*: Tie specs to the actual config constants (or assume/require equality) and prove size/top using those constants.
- **Abstraction loss – page objects/contiguity**: Original stores `Vec<KernelPage>`; verified model collapses to `(base_addr, num_pages)` and assumes contiguity without relating to `KernelPage` metadata or allocator guarantees. This omits invariants about per-page ownership and backing frames. *Suggested fix*: Model the `KernelPage` collection (ghost or abstract) and connect contiguity/alignment/ownership to allocator postconditions.

### Medium
- **Debug implementation unverified**: `fmt::Debug` exists in the original but is absent from verification, so formatting behavior (and its reliance on accessors) is unchecked. *Suggested fix*: Add a spec/exec `Debug` impl or prove the existing one respects invariants and does not panic.
- **Accessor coverage gap**: Original `size` and `base` are private but relied upon by `Debug`/`Drop`; verified versions are public and proven, yet no correspondence is shown to the original private methods. *Suggested fix*: Map the verified accessors to the original private helpers and show callers (Debug/Drop) are safe.

### Low
- **Stack-growth semantics clarity**: Original comment says “base is the highest address,” while both implementations compute `top = base + size` (base is the lowest). The proof assumes the lower-base convention without reconciling the documented intent. *Suggested fix*: Align docs/spec with the actual address orientation and assert the downward-growth convention in specs (e.g., SP starts at `top`).

## Positive Observations
- Core arithmetic/alignment invariants (page alignment, size alignment, top computation, non-empty, overflow avoidance) are specified and proven for the simplified model.
- Address queries (`contains`, `page_index`, `has_room`) have clear specs and are proved consistent with the abstract view.

## Summary
The verification passes but only for a simplified model that omits allocator interaction, page ownership, and destructor behavior. Key constructor and teardown behaviors of the real kstack are unverified, so coverage and semantic equivalence gaps remain. Strengthen the model to include allocator calls, per-page ownership/contiguity, config-driven sizing, and Drop semantics to achieve end-to-end correctness.
