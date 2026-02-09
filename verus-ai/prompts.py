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

== THREE-FILE SPLIT ORGANIZATION ==

You MUST produce three separate files for each module:

1. **{output_dir}/{file_stem}.rs** - Implementation (exec) code
   - Copy the original source here as the starting point
   - Add `use vstd::prelude::*;` and `include!` for spec/proof files
   - Keep struct definitions, impl blocks, and exec functions inside a `verus! {{ }}` block
   - Add requires/ensures contracts for ALL functions (public and private)
   - Reference lemmas from proof file; do NOT inline proofs in exec code

2. **{output_dir}/{file_stem}.spec.rs** - Specification functions
   - Define View types (e.g., `pub struct {type_name}View`) with `#[verifier::ext_equal]`
   - Define `impl View for {type_name}` trait
   - Define `pub open spec fn` for abstract properties (wf, invariants, etc.)
   - Wrap everything in a `verus! {{ }}` block

3. **{output_dir}/{file_stem}.proof.rs** - Proof functions and lemmas
   - Define `proof fn` lemmas with requires/ensures
   - Place inside `verus! {{ }}` block
   - Use `impl {type_name}` blocks to add proof methods

Example file structure:
```rust
// {file_stem}.rs (exec)
use vstd::prelude::*;
include!("{file_stem}.spec.rs");
include!("{file_stem}.proof.rs");

verus! {{
    pub struct MyType {{ ... }}
    impl MyType {{
        pub fn new(...) -> (result: ...) requires ... ensures ... {{ ... }}
    }}
}}
```

```rust
// {file_stem}.spec.rs (spec)
use vstd::prelude::*;
verus! {{
    #[verifier::ext_equal]
    pub struct MyTypeView {{ ... }}
    impl View for MyType {{
        type V = MyTypeView;
        ...
    }}
}}
```

```rust
// {file_stem}.proof.rs (proof)
use vstd::prelude::*;
verus! {{
    impl MyType {{
        pub proof fn lemma_something(&self) requires ... ensures ... {{ ... }}
    }}
}}
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
4. EQUIVALENCE: Verified code is semantically equivalent to original
5. INVARIANTS: State invariants are sufficient to prove correctness
6. PROPERTIES: Are the key safety and liveness properties identified and proven?
7. SPLIT QUALITY: Are spec/proof properly separated from exec code?

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

