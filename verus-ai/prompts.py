# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Prompt templates for the Verus AI verification workflow.

Prompts are domain-agnostic and support the three-file split organization:
  - {name}.rs       (exec: implementation with requires/ensures)
  - {name}.spec.rs  (spec: View types, spec functions, invariants)
  - {name}.proof.rs (proof: lemmas and proof functions)
"""

PROVER_PROMPT = """
Verify {source_path} using Verus.

Target directory: {output_dir}

This is an OS kernel component. Identify and prove the core specifications,
invariants, and safety/liveness properties that matter for correctness.

== CRITICAL CONSTRAINTS ==

1. Do NOT modify executable struct definitions (fields, types, visibility).
   Only add ghost/tracked fields if absolutely necessary, and document each one.
2. Do NOT convert executable fields to ghost. The verified exec code must be
   semantically equivalent to the original source.
3. Specs must be ABSTRACT, not implementation mirrors. Use sets, sequences,
   and high-level predicates — not bit-level or field-level copies of the
   implementation. Define View types that hide internal representation.
4. Before defining new spec functions or lemmas, search vstd for existing
   ones (e.g., seq_to_set, set operations). Do NOT redefine what vstd provides.
5. Document every deviation from the original source in the module header,
   explaining why it is necessary for verification.

== THREE-FILE SPLIT ORGANIZATION ==

You MUST produce three separate files for each module:

1. **{output_dir}/{file_stem}.rs** - Implementation (exec) code
   - Copy the original source here as the starting point
   - Add `use vstd::prelude::*;` and `include!` for spec/proof files
   - Keep struct definitions, impl blocks, and exec functions inside a `verus! {{ }}` block
   - Add requires/ensures contracts for ALL functions (public and private)
   - Reference lemmas from proof file; do NOT inline proofs in exec code

2. **{output_dir}/{file_stem}.spec.rs** - Specification functions
   - Define View types (e.g., `pub struct {{type_name}}View`) with `#[verifier::ext_equal]`
   - Define `impl View for {{type_name}}` trait
   - Define `pub open spec fn` for abstract properties (wf, invariants, etc.)
   - Wrap everything in a `verus! {{ }}` block

3. **{output_dir}/{file_stem}.proof.rs** - Proof functions and lemmas
   - Define `proof fn` lemmas with requires/ensures
   - Place inside `verus! {{ }}` block
   - Use `impl {{type_name}}` blocks to add proof methods

Example file structure:
```rust
// {{file_stem}}.rs (exec)
use vstd::prelude::*;
include!("{{file_stem}}.spec.rs");
include!("{{file_stem}}.proof.rs");

verus! {{{{
    pub struct MyType {{ ... }}
    impl MyType {{{{
        pub fn new(...) -> (result: ...) requires ... ensures ... {{ ... }}
    }}}}
}}}}
```

```rust
// {{file_stem}}.spec.rs (spec)
use vstd::prelude::*;
verus! {{{{
    #[verifier::ext_equal]
    pub struct MyTypeView {{ ... }}
    impl View for MyType {{{{
        type V = MyTypeView;
        ...
    }}}}
}}}}
```

```rust
// {{file_stem}}.proof.rs (proof)
use vstd::prelude::*;
verus! {{{{
    impl MyType {{{{
        pub proof fn lemma_something(&self) requires ... ensures ... {{ ... }}
    }}}}
}}}}
```

== REQUIREMENTS ==
1. Define View types for abstract state representation
2. Define invariants that all operations must maintain
3. Add requires/ensures contracts for all functions (public and private impl fn)
4. NO `assume` or `external_body` for core module functions
   (OK for low-level dependencies like raw memory operations or HAL)

== NOTE ON DEPENDENCIES ==
Dependencies already verified: {dependencies}
If the module depends on types from other modules, you may use `#[verifier::external_body]`
ONLY for dependency boundary types that are not part of THIS module's core logic.

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}
  (This script runs verus, logs results, and commits changes automatically)

Iterate until verification passes. Document your work in the module comments.
""".strip()


REVIEWER_PROMPT = """
Review the Verus verification of {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

This is an OS kernel component. Evaluate if the verification captures the
essential correctness properties.

Review criteria:
1. COVERAGE: All functions in original source (public and private) have verified versions
2. SPECIFICATIONS: Specs capture intended behavior (not too weak, not too strong)
3. SOUNDNESS: No unjustified assume/external_body in core module
4. EQUIVALENCE: Verified exec code is semantically equivalent to original
5. INVARIANTS: State invariants are sufficient to prove correctness
6. PROPERTIES: Are the key safety and liveness properties identified and proven?
7. SPLIT QUALITY: Are spec/proof properly separated from exec code?
8. ABSTRACTION: Do specs use abstract reasoning (sets, sequences, predicates)
   rather than mirroring implementation details (bit operations, raw field access)?
   View types should hide internal representation.
9. EXEC INTEGRITY: Are executable struct definitions unchanged from the original?
   Ghost fields must be justified. No fields converted from exec to ghost.

Verification command: ./verus-ai/scripts/verify.sh {module_name}

For each issue found, provide:
- Priority: Critical / High / Medium / Low
- Location: Function or spec name (and which file: exec/spec/proof)
- Description: What is wrong or missing
- Suggested Fix: How to address it

Output format (write to {review_file}):
```markdown
# Review: {module_name} ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found

### Critical
- ...

### High
- ...

### Medium
- ...

### Low
- ...

## Positive Observations
- ...

## Summary
[Overall assessment and recommendations]
```
""".strip()


PROVER_FIX_PROMPT = """
A reviewer has identified issues in your Verus verification.

Review file: {review_file}
Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Please address each issue:
1. If the issue is valid, fix it and explain your change
2. If the issue is not applicable, explain why it can be rejected

After fixing, run: ./verus-ai/scripts/verify.sh {module_name}

Update the module until verification passes with all issues addressed.
""".strip()


PROVER_FIX_FRESH_PROMPT = """
You are starting a NEW verification session for {module_name}.

== CONTEXT ==
Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)
Dependencies already verified: {dependencies}

This is an OS kernel component. The existing verified code needs improvements
based on reviewer feedback.

== REVIEWER ISSUES ==
Review file(s): {review_file}

Please read the review file(s) carefully and address each issue:
1. Critical/High issues MUST be fixed
2. For Medium/Low issues, fix if valid or explain why rejection is justified
3. If the reviewer asks for missing functions, add them with proper verification

== REQUIREMENTS ==
1. Maintain or improve existing View types and invariants
2. All functions must have requires/ensures contracts
3. NO assume or unjustified external_body for core module functions
4. Verify semantic equivalence with original source
5. Maintain the three-file split: exec, spec, proof
6. Do NOT modify executable struct definitions unless absolutely necessary
7. Specs must be abstract (use sets, sequences) not implementation mirrors

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}

Iterate until verification passes (0 errors) and all reviewer issues are addressed.
Document significant changes in module comments.
""".strip()


PROVER_RETRY_PROMPT = """
Verus verification failed. Please fix the errors and try again.

Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)
Retry attempt: {retry_num} of {max_retries}

Verus output:
```
{verus_output}
```

Fix the verification errors and run: ./verus-ai/scripts/verify.sh {module_name}

Continue until verification passes (0 errors).
""".strip()


REVIEW_FOLLOWUP_PROMPT = """
The prover has addressed your previous review.

Previous review: {previous_review_file}
Updated module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Re-review with a critical and skeptical mindset:
1. Were previous issues ACTUALLY fixed, or just claimed to be fixed?
2. Verify the prover's claims - do not trust "not applicable" responses without evidence
3. Are there any new issues introduced by the fixes?
4. Is the verification now complete and sound?

Be fair but rigorous. If the prover rejects an issue, verify the rejection is justified.

Write your updated review to {review_file} using the same format.
Also write a simple result to {result_file}:
```
GRADE: [A+ / A / A- / B+ / B / B- / C / D / F]
PASSED: [YES / NO]
REMAINING_ISSUES: [count]
```
""".strip()


CHEATING_JUSTIFICATION_PROMPT = """
The verification of {module_name} contains potential cheating patterns that require justification.

Detected patterns:
{patterns_found}

For each pattern, explain:
1. Why it is necessary (or remove it if not necessary)
2. What guarantees are being assumed
3. How these assumptions can be validated

Write justifications as comments in the code or in a separate TRUST_BOUNDARY.md file.
""".strip()


#==================================================================================================
# Post-Processing Prompts: Simplification and Consistency Checking
#==================================================================================================

SIMPLIFY_PROOF_PROMPT = """
Simplify the Verus verification in {output_dir}/

Files:
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

Reference: {source_path}

== WHAT TO SIMPLIFY ==
1. Remove truly redundant lemmas (duplicate proofs of the same property)
2. Remove redundant postconditions implied by others in the same function
3. Condense verbose inline proofs into reusable lemmas in the proof file
4. Remove debug artifacts (unnecessary asserts, TODO comments, dead code)

== CRITICAL: WHAT IS "REDUNDANT"? ==
A lemma is redundant ONLY if another lemma proves the EXACT same thing:
- Same requires (preconditions)
- Same ensures (postconditions)

WARNING: "Never called" does NOT mean redundant! Verus verifies all lemmas regardless.
WARNING: Different requires = different property, even if ensures looks similar.

When in doubt, KEEP the lemma.

== CONSTRAINTS ==
- Do NOT add assume, admit, external_body
- Verification must still pass

== PROCESS ==
1. For each removal candidate, compare requires+ensures with existing lemmas
2. Remove only if truly identical
3. Run: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT ==
Write report to verus-ai-history/simplify/{module_name}_{timestamp}.md showing what was removed and why.
""".strip()


CHECK_CONSISTENCY_PROMPT = """
Check and FIX semantic consistency between the original source and verified Verus code.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

== YOUR TASK ==
1. Identify all inconsistencies between original and verified code
2. FIX inconsistencies that CAN be fixed (missing functions, wrong logic, etc.)
3. DOCUMENT inconsistencies that CANNOT be fixed (Verus limitations)
4. Run verification after fixes: ./verus-ai/scripts/verify.sh {module_name}

== CRITICAL CHECKS ==

1. **Function Coverage**: Every public AND private function in the original source must have
   a corresponding verified function.
   - **ACTION**: Add missing functions with proper verification
   - Do NOT skip complex functions

2. **Loop Transformations**: LLMs often transform `for` loops to `while` loops.
   - **ACTION**: Verify they are semantically equivalent (same iteration count, termination, side effects)
   - If not equivalent, FIX the loop logic

3. **Invented Functions**: Functions in Verus code that do NOT exist in original source.
   - spec fn / proof fn: OK (needed for verification)
   - New exec functions: REMOVE or justify why they preserve semantics

4. **Type Mismatches**: Check for silent type changes.
   - **FIXABLE**: Wrong integer types (u32 vs u64) -> Fix to match original
   - **UNFIXABLE**: Pointer to usize (Verus limitation) -> Document in report

5. **Control Flow Changes**: Early returns, error handling differences.
   - **ACTION**: Fix to match original control flow where possible

6. **Skipped Complexity**: Comments like "TODO: verify later" or "simplified for verification".
   - **ACTION**: Implement the skipped code properly

== UNFIXABLE ISSUES (Verus Limitations) ==
Some things cannot be fixed due to Verus/verification constraints. Document these:
- Raw pointers must be converted to usize (Verus doesn't support raw pointers well)
- Some unsafe code must use external_body
- Certain Rust features not supported in Verus

== CONSTRAINTS ==
- DO NOT add `assume`, `admit`, or unjustified `external_body`
- DO NOT change the semantic meaning of functions
- Verification MUST pass after your fixes

== OUTPUT ==

After making fixes, write a report to verus-ai-history/consistency/{module_name}_{timestamp}.md:

```markdown
# Consistency Check: {module_name}

## Summary
- Issues Found: N
- Issues Fixed: M
- Unfixable Issues: K

## Fixed Issues
| Issue | Location | Fix Applied |
|-------|----------|-------------|
| Missing function bar() | line 42 | Added verified implementation |
| Wrong loop bounds | alloc() | Fixed iteration count |

## Unfixable Issues (Verus Limitations)
| Issue | Location | Reason Cannot Fix |
|-------|----------|-------------------|
| Pointer as usize | Frame.addr | Verus doesn't support raw pointers |

## Function Coverage
| Original Function | Verified Function | Status |
|-------------------|-------------------|--------|
| fn foo()          | fn foo()          | OK     |
| fn bar()          | fn bar()          | FIXED  |

## Verification Status
- Before fixes: [PASS/FAIL]
- After fixes: [PASS/FAIL]

## Remaining Concerns
[Any issues that need human review]
```
""".strip()


STRENGTHEN_SPECS_PROMPT = """
Strengthen weak specifications in {output_dir}/

Files:
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

Reference: {source_path}

== WHAT TO STRENGTHEN ==
Find and fix WEAK postconditions that don't fully capture what the function guarantees.

Common weaknesses:
1. One-sided: `ensures ok ==> P` - what about error case?
2. Trivial: `ensures x >= 0` for usize (always true)
3. Missing state change: `ensures ok` without saying what changed
4. Vague error: `ensures err ==> true` (says nothing)

Look for liveness properties (good things happen when preconditions met):
- `has_capacity() ==> result.is_ok()` not just "may succeed"
- `is_allocated(i) ==> can_deallocate(i)` not just "may work"

== CONSTRAINTS ==
- Do NOT add assume, admit, external_body
- Verification must still pass

== PROCESS ==
1. Review each function's ensures clauses
2. Identify weak specs and strengthen them
3. Run: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT ==
Write report to verus-ai-history/strengthen/{module_name}_{timestamp}.md showing before/after for each change.
""".strip()


# Keep old name as alias for backwards compatibility.
STRENGTHEN_LIVENESS_PROMPT = STRENGTHEN_SPECS_PROMPT


#==================================================================================================
# Improvement Prompts: Abstraction and Exec Integrity
#==================================================================================================

IMPROVE_ABSTRACTION_PROMPT = """
Improve the spec abstraction for {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

== YOUR TASK ==

The current spec may mirror the implementation too closely (listing individual
fields in postconditions instead of using abstract reasoning). Your job is to
add View-level abstract state transition functions so that downstream modules
can write postconditions like:
    ensures result@ =~= old(self)@.spec_foo(args)
instead of listing every field change individually.

== STEPS ==

1. Read the .spec.rs file and identify all View types (e.g., FooView).
2. Read the .rs exec file and identify all pub fn that mutate or construct state.
3. For each exec function, add a corresponding `pub open spec fn` on the View
   type that returns the expected output View. Use struct update syntax:
     FooView {{ changed_field: new_val, ..*self }}
4. Prefer abstract types: Set<T> over raw bits, Seq<T> for ordered collections,
   high-level predicates over field-level conditions.
5. If bridging lemmas are needed (e.g., bit-level ↔ set-level), add them to
   the .proof.rs file.
6. Do NOT modify exec code in the .rs file. Do NOT change existing ensures.
   Only ADD new spec functions to the .spec.rs file (and optionally lemmas
   to .proof.rs).

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).

== OUTPUT ==
Write a brief summary of what was added as a comment at the top of the
.spec.rs file under the existing module doc comment.
""".strip()


IMPROVE_ABSTRACTION_REVIEW_PROMPT = """
Review the abstraction improvements made to {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

The prover has added abstract state transition spec functions to the View types.

Review criteria:
1. COMPLETENESS: Every exec function that changes state has a matching spec
   transition function on the View type.
2. CORRECTNESS: Each spec transition function accurately models what the exec
   function does (field changes, collection operations, etc.).
3. ABSTRACTION: Spec functions use abstract types (Set, Seq, predicates) rather
   than mirroring implementation details (bit ops, raw indices).
4. NO EXEC CHANGES: The .rs exec file must be unmodified from original.
5. VERIFICATION: Code still verifies (0 errors).

Verification command: ./verus-ai/scripts/verify.sh {module_name}

For each issue, provide:
- Priority: Critical / High / Medium / Low
- Location: Function or spec name (and which file)
- Description: What is wrong or missing
- Suggested Fix: How to address it

Write review to {review_file}.

Output format:
```markdown
# Review: {module_name} Abstraction ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found
### Critical
- ...
### High
- ...
### Medium
- ...
### Low
- ...

## Positive Observations
- ...

## Summary
[Overall assessment]
```
""".strip()


EXEC_INTEGRITY_PROMPT = """
Check exec code integrity for {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

== YOUR TASK ==

Compare the verified exec code ({output_dir}/{file_stem}.rs) against the
original source ({source_path}) and produce a detailed integrity report.

== STEPS ==

1. Identify ALL differences between the original source and the verified exec
   code. Focus on executable logic, not ghost/proof annotations.
2. For each difference, classify it:
   - GHOST_ANNOTATION: Added requires/ensures/invariant (acceptable)
   - GHOST_FIELD: Added ghost/tracked field to struct (needs justification)
   - LOGIC_CHANGE: Changed control flow, arithmetic, data structure (must fix or justify)
   - TYPE_CHANGE: Changed types (must document why)
   - MISSING_FUNCTION: Function in original but not in verified (must add)
   - INVENTED_FUNCTION: Exec function in verified but not in original (must remove or justify)
3. For LOGIC_CHANGE items: fix the verified code to match the original, then
   re-verify. If the change is necessary for verification, document why.
4. For MISSING_FUNCTION items: add the function with verification.
5. For INVENTED_FUNCTION items: remove unless justified.

== CONSTRAINTS ==
- Do NOT add assume, admit, or unjustified external_body.
- Verification must pass after fixes.

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT ==
Write report to {report_file}:

```markdown
# Exec Integrity: {module_name}

## Summary
- Total differences: N
- Acceptable (ghost annotations): M
- Fixed: K
- Unfixable (documented): J

## Differences
| # | Type | Location | Description | Action |
|---|------|----------|-------------|--------|
| 1 | GHOST_ANNOTATION | fn foo() | Added requires clause | Acceptable |
| 2 | LOGIC_CHANGE | fn bar() | Loop bound changed | Fixed |

## Verification Status
- Before: [PASS/FAIL]
- After: [PASS/FAIL]
```
""".strip()


EXEC_INTEGRITY_REVIEW_PROMPT = """
Review the exec integrity report for {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

Integrity report: {report_file}

Review criteria:
1. Were ALL differences between original and verified exec code identified?
2. Were LOGIC_CHANGE items correctly fixed or properly justified?
3. Are there any remaining semantic differences that were missed?
4. Is the exec code still faithful to the original implementation?
5. Does verification still pass?

Verification command: ./verus-ai/scripts/verify.sh {module_name}

Write review to {review_file}.

Output format:
```markdown
# Review: {module_name} Exec Integrity ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found
### Critical
- ...
### High
- ...

## Summary
[Overall assessment]
```
""".strip()


IMPROVE_ABSTRACTION_FIX_PROMPT = """
A reviewer has identified issues in your abstraction improvements for {module_name}.

Review file: {review_file}
Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Please address each issue:
1. If the issue is valid, fix it and explain your change
2. If the issue is not applicable, explain why

After fixing, run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).
""".strip()


EXEC_INTEGRITY_FIX_PROMPT = """
A reviewer has identified issues in your exec integrity check for {module_name}.

Review file: {review_file}
Integrity report: {report_file}
Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Please address each issue:
1. If the issue is valid, fix it and explain your change
2. If the issue is not applicable, explain why

After fixing, run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).
""".strip()



#==================================================================================================
# Spec Methodology and Exec Consistency Prompts
#==================================================================================================

SPEC_METHODOLOGY_PROMPT = """
Fix spec methodology violations for {module_name} per the project guidelines.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

== METHODOLOGY GUIDELINES (from specifying-and-proving-types.md) ==

Step 1 - ABSTRACTION:
  - View types must use abstract types: int not i32, Seq<int> not Vec<u64>.
  - view() must be `pub closed spec fn` (not open), so users can't see internals.
  - Hide implementation-only fields from View types.

Step 2 - INVARIANT:
  - Each type needs `pub closed spec fn inv(&self) -> bool` (or wf()).
  - Captures all internal consistency invariants.

Step 3 - PUBLIC SPECS:
  - Public method specs use only self@.field (view), not self.field (implementation).
  - Public methods require self.inv() and ensure self.inv() for &self/&mut self.
  - No Self spec functions other than inv() and view() in public specs.

Step 5 - NO CHEATING:
  - No assume(), admit(), or unjustified external_body.

== VIOLATIONS FOUND ==

The following violations were detected by automated analysis:

{violations_report}

== YOUR TASK ==

1. Read each violation and fix it in the appropriate file (.rs, .spec.rs, .proof.rs).
2. For view() openness: if it MUST stay open (e.g., View trait impl), document why.
3. For missing inv(): add one, or document why it's not needed for this type.
4. For self.field in public specs: rewrite using self@.field or self.view().field.
5. Do NOT break existing verification.

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).
""".strip()


SPEC_METHODOLOGY_REVIEW_PROMPT = """
Review spec methodology fixes for {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

Guidelines reference: specifying-and-proving-types.md

Review criteria:
1. Do View types use abstract types (int, Seq, Set, Map) not concrete (i32, Vec)?
2. Is view() declared as `pub closed spec fn` (or justified if open)?
3. Does inv()/wf() exist and is it `pub closed spec fn`?
4. Do public method specs avoid self.field, using self@.field instead?
5. Do public methods require/ensure inv()/wf() for self parameters?
6. Are there any remaining assume/admit/unjustified external_body?
7. Does verification still pass?

Verification command: ./verus-ai/scripts/verify.sh {module_name}

Write review to {review_file}.

Output format:
```markdown
# Review: {module_name} Spec Methodology ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found
### Critical
- ...
### High
- ...

## Summary
[Overall assessment]
```
""".strip()


SPEC_METHODOLOGY_FIX_PROMPT = """
A reviewer has identified issues in your spec methodology fixes for {module_name}.

Review file: {review_file}
Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Address each issue. After fixing, run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).
""".strip()


EXEC_CONSISTENCY_PROMPT = """
Fix exec code inconsistencies for {module_name} detected by AST analysis.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

== TREE-SITTER AST DIFF REPORT ==

The following inconsistencies were detected by comparing AST hashes of exec
functions between the original source and the verified version (ghost/proof
annotations are stripped before comparison):

{consistency_report}

== YOUR TASK ==

For each inconsistency:

1. **MISMATCH functions**: Compare the source and verus versions side by side.
   - If the verus version changed executable logic: RESTORE the original logic
     and update specs/proofs to verify the original code.
   - If the change is purely structural but semantically equivalent (e.g.,
     variable renaming, reordering): document WHY it is equivalent.
   - If the change was necessary for verification (Verus limitation): document
     the limitation and prove the equivalence informally.

2. **MISSING_IN_VERUS functions**: Add the missing function to the verus exec
   file with proper verification (requires/ensures).

3. **EXTRA_IN_VERUS exec functions**: Remove unless justified (helper functions
   extracted for verification are acceptable if documented).

== CONSTRAINTS ==
- Do NOT add assume, admit, or unjustified external_body.
- Verification must pass after fixes.
- For each change, write a brief justification comment.

== VERIFICATION ==
Run: ./verus-ai/scripts/verify.sh {module_name}

== OUTPUT ==
Write report to {report_file}:

```markdown
# Exec Consistency Fix: {module_name}

## Summary
- Mismatches fixed: N
- Missing functions added: M
- Documented equivalences: K

## Changes
| Function | Action | Justification |
|----------|--------|---------------|

## Verification: PASS/FAIL
```
""".strip()


EXEC_CONSISTENCY_REVIEW_PROMPT = """
Review exec consistency fixes for {module_name}.

Original source: {source_path}
Verified code directory: {output_dir}/
  - {file_stem}.rs (exec)
  - {file_stem}.spec.rs (spec)
  - {file_stem}.proof.rs (proof)

Consistency report: {report_file}

Review criteria:
1. Were all MISMATCH functions properly restored or equivalence documented?
2. Were MISSING functions added with proper verification?
3. Are equivalence justifications sound?
4. Does the exec code now faithfully represent the original source?
5. Does verification still pass?

Verification command: ./verus-ai/scripts/verify.sh {module_name}

Write review to {review_file}.

Output format:
```markdown
# Review: {module_name} Exec Consistency ({model_name})

## Grade: [A+ / A / A- / B+ / B / B- / C / D / F]

## Issues Found
### Critical
- ...

## Summary
[Overall assessment]
```
""".strip()


EXEC_CONSISTENCY_FIX_PROMPT = """
A reviewer has identified issues in your exec consistency fixes for {module_name}.

Review file: {review_file}
Report: {report_file}
Module files:
  - {output_dir}/{file_stem}.rs (exec)
  - {output_dir}/{file_stem}.spec.rs (spec)
  - {output_dir}/{file_stem}.proof.rs (proof)

Address each issue. After fixing, run: ./verus-ai/scripts/verify.sh {module_name}
Iterate until verification passes (0 errors).
""".strip()
